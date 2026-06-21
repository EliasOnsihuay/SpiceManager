use crate::errors::{Result, SpiceError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub author: String,
    pub maintainer: String,
    pub github_owner: String,
    pub github_repo: String,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub update_stage_dir: PathBuf,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let app_name = "SpiceManager".to_string();
        let author = "Elias Onsihuay".to_string();
        let maintainer = "Kodhu Technologies".to_string();
        let base = platform_config_dir()?;
        let cache = platform_cache_dir().unwrap_or_else(|_| base.join("cache"));
        let log_dir = base.join("logs");
        let update_stage_dir = cache.join("updates");

        for dir in [&base, &cache, &log_dir, &update_stage_dir] {
            std::fs::create_dir_all(dir)?;
        }

        Ok(Self {
            app_name,
            author,
            maintainer,
            github_owner: "EliasOnsihuay".to_string(),
            github_repo: "SpiceManager".to_string(),
            state_dir: base,
            cache_dir: cache,
            log_dir,
            update_stage_dir,
        })
    }

    pub fn state_file(&self) -> PathBuf {
        self.state_dir.join("state.json")
    }

    pub fn workflow_file(&self) -> PathBuf {
        self.state_dir.join("workflows.json")
    }

    pub fn app_update_file(&self) -> PathBuf {
        self.state_dir.join("app_update.json")
    }

    pub fn log_file(&self) -> PathBuf {
        self.log_dir.join("spicemanager.log")
    }
}

fn platform_config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .or_else(dirs::data_dir)
        .ok_or_else(|| SpiceError::Message("could not resolve local app data directory".into()))?;
    #[cfg(target_os = "windows")]
    return Ok(base.join("Kodhu Technologies").join("SpiceManager"));
    #[cfg(target_os = "macos")]
    return Ok(base.join("com.kodhu.spicemanager"));
    #[cfg(all(unix, not(target_os = "macos")))]
    return Ok(base.join("spicemanager"));
    #[allow(unreachable_code)]
    Ok(base.join("spicemanager"))
}

fn platform_cache_dir() -> Result<PathBuf> {
    dirs::cache_dir()
        .map(|p| p.join("spicemanager"))
        .ok_or_else(|| SpiceError::Message("could not resolve cache directory".into()))
}
