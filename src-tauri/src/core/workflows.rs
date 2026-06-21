use crate::config::AppConfig;
use crate::core::{app_update, compatibility, diagnostics, spicetify};
use crate::errors::Result;
use crate::logging;
use crate::models::{
    AppUpdateState, DiagnosticsReport, EnvironmentState, HealthStatus, ManagedFeatureState,
    WorkflowKind, WorkflowReport,
};
use crate::platform;
use crate::state_store::StateStore;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkflowEngine {
    pub config: AppConfig,
    pub store: StateStore,
}

impl WorkflowEngine {
    pub fn new() -> Result<Self> {
        let config = AppConfig::load()?;
        let store = StateStore::new(config.clone());
        Ok(Self { config, store })
    }

    pub fn detect_state(&self) -> Result<WorkflowReport> {
        self.workflow(WorkflowKind::Detect, |engine, previous| {
            let env = engine.detect_environment(previous.as_ref(), None)?;
            engine.store.save_environment(&env)?;
            Ok((env, None, "Environment detection completed.".into(), vec![], vec![]))
        })
    }

    pub fn install_all(&self) -> Result<WorkflowReport> {
        self.workflow(WorkflowKind::Install, |engine, previous| {
            let mut warnings = Vec::new();
            let initial = engine.detect_environment(previous.as_ref(), None)?;
            if !initial.spicetify.installed {
                let result = spicetify::install_spicetify();
                if !result.success {
                    warnings.push(format!("Spicetify install did not complete: {}", result.stderr));
                }
            }
            let after_spicetify = engine.detect_environment(previous.as_ref(), None)?;
            if !matches!(after_spicetify.marketplace.state, ManagedFeatureState::Installed) {
                let result = spicetify::install_marketplace();
                if !result.success {
                    warnings.push(format!("Marketplace install did not complete: {}", result.stderr));
                }
            }
            if let Some(config_path) = &after_spicetify.spicetify.config_path {
                if let Err(err) = spicetify::ensure_adblock_config(config_path, "adblock.js") {
                    warnings.push(format!("Could not update adblock config: {err}"));
                }
            }
            let apply = spicetify::apply_with_fallbacks();
            let mut errors = Vec::new();
            if !apply.success {
                errors.push(apply.summary.clone());
            }
            let mut env = engine.detect_environment(previous.as_ref(), Some(apply.success))?;
            env.spicetify.last_apply_summary = Some(apply.summary.clone());
            engine.store.save_environment(&env)?;
            Ok((env, Some(apply), "Install workflow completed.".into(), warnings, errors))
        })
    }

    pub fn update_all(&self) -> Result<WorkflowReport> {
        self.workflow(WorkflowKind::Update, |engine, previous| {
            let mut warnings = Vec::new();
            let initial = engine.detect_environment(previous.as_ref(), None)?;
            spicetify::require_spicetify_installed(initial.spicetify.installed)?;
            let update = spicetify::update_spicetify();
            if !update.success {
                warnings.push(format!("Spicetify update failed: {}", update.stderr));
            }
            let repaired = engine.repair_managed_config(&initial, &mut warnings)?;
            let apply = spicetify::apply_with_fallbacks();
            let mut errors = Vec::new();
            if !apply.success {
                errors.push(apply.summary.clone());
            }
            let mut env = engine.detect_environment(Some(&repaired), Some(apply.success))?;
            env.spicetify.last_apply_summary = Some(apply.summary.clone());
            engine.store.save_environment(&env)?;
            Ok((env, Some(apply), "Update workflow completed.".into(), warnings, errors))
        })
    }

    pub fn repair_all(&self) -> Result<WorkflowReport> {
        self.workflow(WorkflowKind::Repair, |engine, previous| {
            let mut warnings = Vec::new();
            let initial = engine.detect_environment(previous.as_ref(), None)?;
            spicetify::require_spicetify_installed(initial.spicetify.installed)?;
            let repaired = engine.repair_managed_config(&initial, &mut warnings)?;
            let apply = spicetify::apply_with_fallbacks();
            let mut errors = Vec::new();
            if !apply.success {
                errors.push(apply.summary.clone());
            }
            let mut env = engine.detect_environment(Some(&repaired), Some(apply.success))?;
            env.spicetify.last_apply_summary = Some(apply.summary.clone());
            engine.store.save_environment(&env)?;
            Ok((env, Some(apply), "Repair workflow completed.".into(), warnings, errors))
        })
    }

    pub fn validate_all(&self) -> Result<WorkflowReport> {
        self.workflow(WorkflowKind::Validate, |engine, previous| {
            let env = engine.detect_environment(previous.as_ref(), None)?;
            engine.store.save_environment(&env)?;
            let warnings = collect_warnings(&env);
            Ok((env, None, "Validation completed.".into(), warnings, vec![]))
        })
    }

    pub fn recover_compatibility(&self) -> Result<WorkflowReport> {
        self.workflow(WorkflowKind::CompatibilityRecovery, |engine, previous| {
            let mut warnings = Vec::new();
            let mut initial = engine.detect_environment(previous.as_ref(), None)?;
            compatibility::mark_recovery_started(&mut initial.compatibility);
            engine.store.save_environment(&initial)?;
            let update = spicetify::update_spicetify();
            if !update.success {
                warnings.push(format!("Recovery update failed: {}", update.stderr));
            }
            let repaired = engine.repair_managed_config(&initial, &mut warnings)?;
            let apply = spicetify::apply_with_fallbacks();
            let mut errors = Vec::new();
            if !apply.success {
                errors.push(apply.summary.clone());
            }
            let mut env = engine.detect_environment(Some(&repaired), Some(apply.success))?;
            env.spicetify.last_apply_summary = Some(apply.summary.clone());
            engine.store.save_environment(&env)?;
            Ok((
                env,
                Some(apply),
                "Compatibility recovery completed.".into(),
                warnings,
                errors,
            ))
        })
    }

    pub fn doctor_report(&self) -> Result<DiagnosticsReport> {
        let report = self.workflow(WorkflowKind::Doctor, |engine, previous| {
            let env = engine.detect_environment(previous.as_ref(), None)?;
            engine.store.save_environment(&env)?;
            Ok((
                env,
                None,
                "Doctor diagnostics refreshed.".into(),
                vec![],
                vec![],
            ))
        })?;
        let _ = report;
        diagnostics::build(&self.config, &self.store)
    }

    pub fn export_diagnostics(&self) -> Result<std::path::PathBuf> {
        diagnostics::export(&self.config, &self.store)
    }

    pub fn app_update_check(&self, owner: Option<String>, repo: Option<String>) -> Result<AppUpdateState> {
        let started_at = Utc::now();
        logging::log(&self.config, "app-update", "checking GitHub Releases")?;
        let owner = owner.unwrap_or_else(|| self.config.github_owner.clone());
        let repo = repo.unwrap_or_else(|| self.config.github_repo.clone());
        let state = match app_update::check_latest(&self.config, &owner, &repo) {
            Ok(state) => state,
            Err(err) => AppUpdateState {
                status: crate::models::AppUpdateStatusKind::CheckFailed,
                current_version: env!("CARGO_PKG_VERSION").to_string(),
                latest_version: None,
                release_url: None,
                selected_asset: None,
                downloaded_asset_path: None,
                last_checked_at: Some(Utc::now()),
                last_error: Some(err.to_string()),
            },
        };
        self.store.save_app_update(&state)?;
        self.append_app_update_workflow(
            WorkflowKind::AppUpdateCheck,
            started_at,
            state.last_error.is_none(),
            format!("App update check finished with status {:?}.", state.status),
            state.last_error.clone(),
        )?;
        Ok(state)
    }

    pub fn app_update_download(&self) -> Result<AppUpdateState> {
        let started_at = Utc::now();
        logging::log(&self.config, "app-update", "downloading selected app update asset")?;
        let current = self.store.load_app_update()?;
        let in_progress = AppUpdateState {
            status: crate::models::AppUpdateStatusKind::DownloadInProgress,
            last_checked_at: Some(Utc::now()),
            ..current.clone()
        };
        self.store.save_app_update(&in_progress)?;
        let next = match app_update::download_selected(&self.config, &current) {
            Ok(next) => next,
            Err(err) => AppUpdateState {
                status: crate::models::AppUpdateStatusKind::InstallFailed,
                last_checked_at: Some(Utc::now()),
                last_error: Some(err.to_string()),
                ..current
            },
        };
        self.store.save_app_update(&next)?;
        self.append_app_update_workflow(
            WorkflowKind::AppUpdateDownload,
            started_at,
            next.last_error.is_none(),
            format!("App update download finished with status {:?}.", next.status),
            next.last_error.clone(),
        )?;
        Ok(next)
    }

    pub fn app_update_status(&self) -> Result<AppUpdateState> {
        self.store.load_app_update()
    }

    pub fn current_status(&self) -> Result<(Option<EnvironmentState>, AppUpdateState)> {
        Ok((self.store.load_environment()?, self.store.load_app_update()?))
    }

    fn workflow<F>(&self, kind: WorkflowKind, op: F) -> Result<WorkflowReport>
    where
        F: FnOnce(&Self, Option<EnvironmentState>) -> Result<(
            EnvironmentState,
            Option<crate::models::ApplyResult>,
            String,
            Vec<String>,
            Vec<String>,
        )>,
    {
        let started_at = Utc::now();
        logging::log(&self.config, "workflow", format!("starting {kind:?}"))?;
        let previous = self.store.load_environment()?;
        let outcome = op(self, previous);
        let ended_at = Utc::now();
        let report = match outcome {
            Ok((environment, apply_result, summary, warnings, errors)) => WorkflowReport {
                id: Uuid::new_v4(),
                kind,
                started_at,
                ended_at,
                success: errors.is_empty(),
                summary,
                warnings,
                errors,
                apply_result,
                environment: Some(environment),
            },
            Err(err) => WorkflowReport {
                id: Uuid::new_v4(),
                kind,
                started_at,
                ended_at,
                success: false,
                summary: "Workflow failed.".into(),
                warnings: vec![],
                errors: vec![err.to_string()],
                apply_result: None,
                environment: self.store.load_environment()?,
            },
        };
        self.store.append_workflow(&report)?;
        logging::log(
            &self.config,
            "workflow",
            format!("finished {:?}: {}", report.kind, report.summary),
        )?;
        Ok(report)
    }

    fn detect_environment(
        &self,
        previous: Option<&EnvironmentState>,
        apply_success: Option<bool>,
    ) -> Result<EnvironmentState> {
        let spotify = platform::detect_spotify()?;
        let mut spicetify = platform::detect_spicetify()?;
        if let Some(prev) = previous {
            spicetify.last_apply_summary = prev.spicetify.last_apply_summary.clone();
        }
        let marketplace = platform::detect_marketplace(&spicetify);
        let adblock = platform::detect_adblock(&spicetify);
        let mut env = EnvironmentState {
            platform: platform::current_platform(),
            detected_at: Utc::now(),
            overall_health: HealthStatus::Unknown,
            spotify,
            spicetify,
            marketplace,
            adblock,
            compatibility: previous
                .map(|p| p.compatibility.clone())
                .unwrap_or_default(),
            recommended_next_action: None,
        };
        env.overall_health = derive_health(&env);
        env.recommended_next_action = recommend(&env);
        compatibility::evaluate(&mut env, previous, apply_success);
        Ok(env)
    }

    fn repair_managed_config(
        &self,
        initial: &EnvironmentState,
        warnings: &mut Vec<String>,
    ) -> Result<EnvironmentState> {
        if !matches!(initial.marketplace.state, ManagedFeatureState::Installed) {
            let result = spicetify::install_marketplace();
            if !result.success {
                warnings.push(format!("Marketplace repair failed: {}", result.stderr));
            }
        }
        if let Some(config_path) = &initial.spicetify.config_path {
            if let Err(err) = spicetify::ensure_adblock_config(config_path, "adblock.js") {
                warnings.push(format!("Adblock repair failed: {err}"));
            }
        }
        self.detect_environment(Some(initial), None)
    }

    fn append_app_update_workflow(
        &self,
        kind: WorkflowKind,
        started_at: chrono::DateTime<Utc>,
        success: bool,
        summary: String,
        error: Option<String>,
    ) -> Result<()> {
        let report = WorkflowReport {
            id: Uuid::new_v4(),
            kind,
            started_at,
            ended_at: Utc::now(),
            success,
            summary: summary.clone(),
            warnings: vec![],
            errors: error.into_iter().collect(),
            apply_result: None,
            environment: self.store.load_environment()?,
        };
        self.store.append_workflow(&report)?;
        logging::log(&self.config, "workflow", summary)?;
        Ok(())
    }
}

fn derive_health(env: &EnvironmentState) -> HealthStatus {
    if !env.spotify.installed || !env.spicetify.installed {
        return HealthStatus::NeedsAction;
    }
    if !env.spotify.likely_usable_for_spicetify {
        return HealthStatus::Warning;
    }
    if matches!(
        env.marketplace.state,
        ManagedFeatureState::Broken | ManagedFeatureState::Failed | ManagedFeatureState::Missing
    ) || matches!(
        env.adblock.state,
        ManagedFeatureState::Broken | ManagedFeatureState::Failed | ManagedFeatureState::Missing
    ) {
        return HealthStatus::Warning;
    }
    HealthStatus::Healthy
}

fn recommend(env: &EnvironmentState) -> Option<String> {
    if !env.spotify.installed {
        Some("Install Spotify classic desktop first, then run Install.".into())
    } else if !env.spotify.likely_usable_for_spicetify {
        Some("Install classic desktop Spotify for full Spicetify support.".into())
    } else if !env.spicetify.installed {
        Some("Run Install to set up Spicetify, Marketplace, and adblock.".into())
    } else if matches!(
        env.marketplace.state,
        ManagedFeatureState::Broken | ManagedFeatureState::Missing
    ) {
        Some("Run Repair to restore Marketplace.".into())
    } else if matches!(env.adblock.state, ManagedFeatureState::Missing) {
        Some("Run Repair to restore adblock configuration.".into())
    } else {
        Some("Run Validate after Spotify updates or when behavior changes.".into())
    }
}

fn collect_warnings(env: &EnvironmentState) -> Vec<String> {
    let mut warnings = Vec::new();
    warnings.extend(env.spotify.warnings.clone());
    warnings.extend(env.spicetify.warnings.clone());
    warnings.extend(env.marketplace.warnings.clone());
    warnings.extend(env.adblock.warnings.clone());
    if let Some(reason) = &env.compatibility.reason {
        warnings.push(reason.clone());
    }
    warnings
}
