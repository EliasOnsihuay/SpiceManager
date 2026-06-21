use crate::models::{
    AdblockState, CompatibilityState, CompatibilityStatus, EnvironmentState, HealthStatus,
    LastKnownGoodState, ManagedFeatureState, MarketplaceState,
};
use chrono::Utc;

pub fn evaluate(
    env: &mut EnvironmentState,
    previous: Option<&EnvironmentState>,
    apply_success: Option<bool>,
) {
    let mut state = previous
        .map(|p| p.compatibility.clone())
        .unwrap_or_default();
    state.previous_spotify_version = previous.and_then(|p| p.spotify.version.clone());
    state.previous_spicetify_version = previous.and_then(|p| p.spicetify.version.clone());
    state.current_spotify_version = env.spotify.version.clone();
    state.current_spicetify_version = env.spicetify.version.clone();
    state.updated_at = Utc::now();

    let spotify_changed = state.previous_spotify_version.is_some()
        && state.previous_spotify_version != state.current_spotify_version;
    let apply_failed = apply_success == Some(false);
    let apply_succeeded = apply_success == Some(true);
    let marketplace_regressed = regressed(previous.map(|p| &p.marketplace), &env.marketplace);
    let adblock_regressed = adblock_regressed(previous.map(|p| &p.adblock), &env.adblock);

    if apply_succeeded {
        state.recent_apply_failures = 0;
        state.hold_mode_active = false;
        state.reason = None;
        state.status = CompatibilityStatus::Compatible;
        state.last_known_good = Some(LastKnownGoodState {
            recorded_at: Utc::now(),
            spotify_version: env.spotify.version.clone(),
            spicetify_version: env.spicetify.version.clone(),
            marketplace_state: env.marketplace.state.clone(),
            adblock_state: env.adblock.state.clone(),
            summary: "Latest apply succeeded and managed features were re-detected.".into(),
        });
    } else if apply_failed {
        state.recent_apply_failures = state.recent_apply_failures.saturating_add(1);
        if spotify_changed
            || marketplace_regressed
            || adblock_regressed
            || state.recent_apply_failures >= 2
        {
            state.hold_mode_active = true;
            state.status = CompatibilityStatus::HoldModeActive;
            state.reason = Some(reason(
                spotify_changed,
                marketplace_regressed,
                adblock_regressed,
                state.recent_apply_failures,
            ));
        } else {
            state.status = CompatibilityStatus::NeedsValidation;
            state.reason = Some("Spicetify apply failed once; validation is needed.".into());
        }
    } else if spotify_changed {
        state.status = CompatibilityStatus::NeedsValidation;
        state.reason = Some("Spotify version changed since the previous detection.".into());
    } else if state.hold_mode_active {
        state.status = CompatibilityStatus::HoldModeActive;
    } else if env.spotify.installed && env.spicetify.installed {
        state.status = CompatibilityStatus::NeedsValidation;
    } else {
        state.status = CompatibilityStatus::Unknown;
    }

    env.compatibility = state;
    if env.compatibility.hold_mode_active {
        env.overall_health = HealthStatus::Hold;
        env.recommended_next_action = Some("Hold mode is active. Run compatibility recovery after updating Spicetify or when upstream support catches up.".into());
    }
}

pub fn mark_recovery_started(state: &mut CompatibilityState) {
    state.status = CompatibilityStatus::RecoveryInProgress;
    state.last_recovery_attempt = Some(Utc::now());
    state.updated_at = Utc::now();
}

fn regressed(previous: Option<&MarketplaceState>, current: &MarketplaceState) -> bool {
    matches!(
        previous.map(|p| &p.state),
        Some(ManagedFeatureState::Installed | ManagedFeatureState::Configured)
    ) && !matches!(
        current.state,
        ManagedFeatureState::Installed | ManagedFeatureState::Configured
    )
}

fn adblock_regressed(previous: Option<&AdblockState>, current: &AdblockState) -> bool {
    matches!(
        previous.map(|p| &p.state),
        Some(ManagedFeatureState::Installed | ManagedFeatureState::Configured)
    ) && !matches!(
        current.state,
        ManagedFeatureState::Installed | ManagedFeatureState::Configured
    )
}

fn reason(
    spotify_changed: bool,
    marketplace_regressed: bool,
    adblock_regressed: bool,
    failures: u32,
) -> String {
    let mut parts = Vec::new();
    if spotify_changed {
        parts.push("Spotify changed since the last known good state");
    }
    if marketplace_regressed {
        parts.push("Marketplace regressed after the change");
    }
    if adblock_regressed {
        parts.push("adblock config regressed after the change");
    }
    if failures >= 2 {
        parts.push("repeated apply failures were observed");
    }
    format!("Hold mode active because {}.", parts.join(", "))
}
