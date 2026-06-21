use crate::core::workflows::WorkflowEngine;
use crate::errors::Result;
use crate::models::{AppUpdateState, DiagnosticsReport, EnvironmentState, WorkflowReport};

#[tauri::command]
pub fn detect_state() -> Result<WorkflowReport> {
    WorkflowEngine::new()?.detect_state()
}

#[tauri::command]
pub fn install_all() -> Result<WorkflowReport> {
    WorkflowEngine::new()?.install_all()
}

#[tauri::command]
pub fn update_all() -> Result<WorkflowReport> {
    WorkflowEngine::new()?.update_all()
}

#[tauri::command]
pub fn repair_all() -> Result<WorkflowReport> {
    WorkflowEngine::new()?.repair_all()
}

#[tauri::command]
pub fn validate_all() -> Result<WorkflowReport> {
    WorkflowEngine::new()?.validate_all()
}

#[tauri::command]
pub fn recover_compatibility() -> Result<WorkflowReport> {
    WorkflowEngine::new()?.recover_compatibility()
}

#[tauri::command]
pub fn doctor_report() -> Result<DiagnosticsReport> {
    WorkflowEngine::new()?.doctor_report()
}

#[tauri::command]
pub fn export_diagnostics() -> Result<String> {
    WorkflowEngine::new()?
        .export_diagnostics()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn app_update_check(owner: Option<String>, repo: Option<String>) -> Result<AppUpdateState> {
    WorkflowEngine::new()?.app_update_check(owner, repo)
}

#[tauri::command]
pub fn app_update_download() -> Result<AppUpdateState> {
    WorkflowEngine::new()?.app_update_download()
}

#[tauri::command]
pub fn app_update_status() -> Result<AppUpdateState> {
    WorkflowEngine::new()?.app_update_status()
}

#[tauri::command]
pub fn current_status() -> Result<(Option<EnvironmentState>, AppUpdateState)> {
    WorkflowEngine::new()?.current_status()
}
