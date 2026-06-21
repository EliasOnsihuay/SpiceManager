use crate::config::AppConfig;
use crate::errors::Result;
use crate::models::{AppUpdateState, EnvironmentState, WorkflowReport};

#[derive(Clone)]
pub struct StateStore {
    config: AppConfig,
}

impl StateStore {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn load_environment(&self) -> Result<Option<EnvironmentState>> {
        read_optional(self.config.state_file())
    }

    pub fn save_environment(&self, state: &EnvironmentState) -> Result<()> {
        write_json(self.config.state_file(), state)
    }

    pub fn load_workflows(&self) -> Result<Vec<WorkflowReport>> {
        Ok(read_optional(self.config.workflow_file())?.unwrap_or_default())
    }

    pub fn append_workflow(&self, report: &WorkflowReport) -> Result<()> {
        let mut reports = self.load_workflows()?;
        reports.push(report.clone());
        if reports.len() > 50 {
            reports = reports.split_off(reports.len() - 50);
        }
        write_json(self.config.workflow_file(), &reports)
    }

    pub fn load_app_update(&self) -> Result<AppUpdateState> {
        Ok(read_optional(self.config.app_update_file())?.unwrap_or_default())
    }

    pub fn save_app_update(&self, state: &AppUpdateState) -> Result<()> {
        write_json(self.config.app_update_file(), state)
    }
}

fn read_optional<T: serde::de::DeserializeOwned>(path: std::path::PathBuf) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn write_json<T: serde::Serialize>(path: std::path::PathBuf, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(value)?;
    std::fs::write(path, content)?;
    Ok(())
}
