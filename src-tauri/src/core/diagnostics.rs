use crate::config::AppConfig;
use crate::errors::Result;
use crate::logging;
use crate::models::DiagnosticsReport;
use crate::state_store::StateStore;
use chrono::Utc;
use std::path::PathBuf;

pub fn build(config: &AppConfig, store: &StateStore) -> Result<DiagnosticsReport> {
    Ok(DiagnosticsReport {
        generated_at: Utc::now(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        environment: store.load_environment()?,
        app_update: store.load_app_update()?,
        recent_workflows: store.load_workflows()?,
        log_excerpt: logging::recent_lines(config, 200),
        state_dir: config.state_dir.clone(),
        log_file: config.log_file(),
    })
}

pub fn export(config: &AppConfig, store: &StateStore) -> Result<PathBuf> {
    let report = build(config, store)?;
    let file = config.state_dir.join(format!(
        "diagnostics-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    std::fs::write(&file, serde_json::to_string_pretty(&report)?)?;
    Ok(file)
}
