use crate::errors::{Result, SpiceError};
use crate::models::{ApplyAttempt, ApplyResult, ShellCommandResult};
use std::path::Path;

pub fn install_spicetify() -> ShellCommandResult {
    if cfg!(target_os = "windows") {
        crate::utils::shell::run(
            "powershell",
            &[
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "iwr -useb https://raw.githubusercontent.com/spicetify/cli/main/install.ps1 | iex",
            ],
        )
    } else {
        crate::utils::shell::run(
            "sh",
            &[
                "-c",
                "curl -fsSL https://raw.githubusercontent.com/spicetify/cli/main/install.sh | sh",
            ],
        )
    }
}

pub fn update_spicetify() -> ShellCommandResult {
    crate::utils::shell::run("spicetify", &["upgrade"])
}

pub fn install_marketplace() -> ShellCommandResult {
    if cfg!(target_os = "windows") {
        crate::utils::shell::run(
            "powershell",
            &[
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "iwr -useb https://raw.githubusercontent.com/spicetify/marketplace/main/resources/install.ps1 | iex",
            ],
        )
    } else {
        crate::utils::shell::run(
            "sh",
            &[
                "-c",
                "curl -fsSL https://raw.githubusercontent.com/spicetify/marketplace/main/resources/install.sh | sh",
            ],
        )
    }
}

pub fn ensure_adblock_config(config_path: &Path, extension_name: &str) -> Result<bool> {
    let mut content = std::fs::read_to_string(config_path).unwrap_or_default();
    if content.to_lowercase().contains(&extension_name.to_lowercase()) {
        return Ok(false);
    }
    if content.trim().is_empty() {
        content.push_str("[AdditionalOptions]\nextensions = adblock.js\n");
    } else if let Some(index) = content.find("extensions =") {
        let end = content[index..]
            .find('\n')
            .map(|offset| index + offset)
            .unwrap_or(content.len());
        let line = &content[index..end];
        let replacement = if line.trim_end().ends_with('|') {
            format!("{line}{extension_name}")
        } else {
            format!("{}|{extension_name}", line.trim_end())
        };
        content.replace_range(index..end, &replacement);
    } else {
        content.push_str("\n[AdditionalOptions]\n");
        content.push_str(&format!("extensions = {extension_name}\n"));
    }
    std::fs::write(config_path, content)?;
    Ok(true)
}

pub fn apply_with_fallbacks() -> ApplyResult {
    let attempts_def = [
        ("backup apply", vec!["backup", "apply"]),
        ("restore backup apply", vec!["restore", "backup", "apply"]),
        ("apply", vec!["apply"]),
    ];
    let mut attempted = Vec::new();
    for (label, args) in attempts_def {
        let result = crate::utils::shell::run("spicetify", &args);
        let success = result.success;
        attempted.push(ApplyAttempt {
            label: label.to_string(),
            result,
        });
        if success {
            return ApplyResult {
                attempted,
                success: true,
                summary: format!("Spicetify apply succeeded with `{label}`."),
            };
        }
    }
    ApplyResult {
        attempted,
        success: false,
        summary: "All Spicetify apply fallback attempts failed.".into(),
    }
}

pub fn require_spicetify_installed(installed: bool) -> Result<()> {
    if installed {
        Ok(())
    } else {
        Err(SpiceError::Message(
            "Spicetify is not installed; install workflow is required first.".into(),
        ))
    }
}
