pub mod linux;
pub mod macos;
pub mod windows;

use crate::errors::Result;
use crate::models::{
    AdblockState, EnvironmentState, ManagedFeatureState, MarketplaceState, Platform,
    SpicetifyState, SpotifyState,
};
use chrono::Utc;

pub fn current_platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else {
        Platform::Unknown(std::env::consts::OS.to_string())
    }
}

pub fn detect_spotify() -> Result<SpotifyState> {
    match current_platform() {
        Platform::Windows => windows::detect_spotify(),
        Platform::MacOS => macos::detect_spotify(),
        Platform::Linux => linux::detect_spotify(),
        Platform::Unknown(os) => Ok(SpotifyState {
            installed: false,
            install_kind: crate::models::SpotifyInstallKind::Unsupported,
            version: None,
            executable_path: None,
            resources_path: None,
            prefs_path: None,
            running: false,
            likely_usable_for_spicetify: false,
            warnings: vec![format!("{os} is not yet supported.")],
            notes: vec![],
        }),
    }
}

pub fn detect_spicetify() -> Result<SpicetifyState> {
    let binary = find_in_path("spicetify");
    let version = binary.as_ref().and_then(|_| {
        let result = crate::utils::shell::run("spicetify", &["-v"]);
        result
            .success
            .then(|| first_versionish_line(&result.stdout).unwrap_or(result.stdout))
    });
    let config_path = default_spicetify_config();
    let configured = config_path.as_ref().map(|p| p.exists()).unwrap_or(false);
    Ok(SpicetifyState {
        installed: binary.is_some(),
        version,
        binary_path: binary,
        config_path,
        configured,
        last_apply_summary: None,
        warnings: if configured {
            vec![]
        } else {
            vec!["Spicetify config has not been created yet.".into()]
        },
        notes: vec![],
    })
}

pub fn detect_marketplace(spicetify: &SpicetifyState) -> MarketplaceState {
    let Some(config) = &spicetify.config_path else {
        return MarketplaceState {
            state: ManagedFeatureState::Uncertain,
            path: None,
            warnings: vec!["Spicetify config path is unknown.".into()],
            notes: vec![],
        };
    };
    let base = config
        .parent()
        .map(|p| p.join("CustomApps").join("marketplace"));
    let path = base.filter(|p| p.exists());
    let config_mentions = crate::utils::fs::path_contains(config, "marketplace");
    let state = match (path.is_some(), config_mentions) {
        (true, true) => ManagedFeatureState::Installed,
        (true, false) => ManagedFeatureState::Broken,
        (false, true) => ManagedFeatureState::Broken,
        (false, false) => ManagedFeatureState::Missing,
    };
    MarketplaceState {
        state,
        path,
        warnings: if config_mentions {
            vec![]
        } else {
            vec!["Marketplace is not listed in the Spicetify config.".into()]
        },
        notes: vec![],
    }
}

pub fn detect_adblock(spicetify: &SpicetifyState) -> AdblockState {
    let extension_name = "adblock.js".to_string();
    let Some(config) = &spicetify.config_path else {
        return AdblockState {
            state: ManagedFeatureState::Uncertain,
            extension_name,
            config_entry_present: false,
            warnings: vec!["Spicetify config path is unknown.".into()],
            notes: vec![],
        };
    };
    let extension_path = config
        .parent()
        .map(|parent| parent.join("Extensions").join(&extension_name));
    let config_present = crate::utils::fs::path_contains(config, &extension_name);
    let file_present = extension_path.as_ref().is_some_and(|path| path.exists());
    AdblockState {
        state: if config_present && file_present {
            ManagedFeatureState::Configured
        } else if config_present || file_present {
            ManagedFeatureState::Broken
        } else if config.exists() {
            ManagedFeatureState::Missing
        } else {
            ManagedFeatureState::Uncertain
        },
        extension_name,
        config_entry_present: config_present,
        warnings: if config_present && file_present {
            vec![]
        } else if config_present && !file_present {
            vec!["adblock.js is configured, but the extension file is missing.".into()]
        } else if file_present && !config_present {
            vec!["adblock.js exists, but is not listed in the Spicetify config.".into()]
        } else {
            vec!["rxri adblockify extension/config entry was not found.".into()]
        },
        notes: vec![],
    }
}

pub fn empty_environment() -> EnvironmentState {
    let spicetify = SpicetifyState {
        installed: false,
        version: None,
        binary_path: None,
        config_path: None,
        configured: false,
        last_apply_summary: None,
        warnings: vec![],
        notes: vec![],
    };
    EnvironmentState {
        platform: current_platform(),
        detected_at: Utc::now(),
        overall_health: crate::models::HealthStatus::Unknown,
        spotify: SpotifyState {
            installed: false,
            install_kind: crate::models::SpotifyInstallKind::Unknown,
            version: None,
            executable_path: None,
            resources_path: None,
            prefs_path: None,
            running: false,
            likely_usable_for_spicetify: false,
            warnings: vec![],
            notes: vec![],
        },
        marketplace: detect_marketplace(&spicetify),
        adblock: detect_adblock(&spicetify),
        spicetify,
        compatibility: Default::default(),
        recommended_next_action: None,
    }
}

fn find_in_path(binary: &str) -> Option<std::path::PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let binary_name = if cfg!(windows) && !binary.ends_with(".exe") {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    };
    std::env::split_paths(&path_var)
        .map(|p| p.join(&binary_name))
        .find(|p| p.exists())
}

fn default_spicetify_config() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|dir| dir.join("spicetify").join("config-xpui.ini"))
}

fn first_versionish_line(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.chars().any(|c| c.is_ascii_digit()))
        .map(|s| s.to_string())
}
