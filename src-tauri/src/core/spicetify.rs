use crate::errors::{Result, SpiceError};
use crate::models::{ApplyAttempt, ApplyResult, ShellCommandResult};
use std::path::Path;

pub const MARKETPLACE_CUSTOM_APP: &str = "marketplace";
pub const MARKETPLACE_ADBLOCK_EXTENSION: &str = "adblock.js";

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

pub fn restore_spotify_ui() -> ShellCommandResult {
    crate::utils::shell::run("spicetify", &["restore"])
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
    let content = std::fs::read_to_string(config_path).unwrap_or_default();
    let mut document = IniDocument::parse(&content);
    let mut changed = false;
    changed |=
        document.ensure_pipe_value("AdditionalOptions", "custom_apps", MARKETPLACE_CUSTOM_APP);
    changed |= document.ensure_pipe_value("AdditionalOptions", "extensions", extension_name);
    if changed {
        if config_path.exists() {
            let backup = config_path.with_extension("ini.spicemanager.bak");
            let _ = std::fs::copy(config_path, backup);
        }
        std::fs::write(config_path, document.to_string())?;
    }
    Ok(changed)
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

#[derive(Debug)]
struct IniDocument {
    lines: Vec<String>,
}

impl IniDocument {
    fn parse(content: &str) -> Self {
        Self {
            lines: content.lines().map(str::to_string).collect(),
        }
    }

    fn ensure_pipe_value(&mut self, section: &str, key: &str, value: &str) -> bool {
        let section_header = format!("[{section}]");
        let section_index = self
            .lines
            .iter()
            .position(|line| line.trim().eq_ignore_ascii_case(&section_header));

        let section_index = match section_index {
            Some(index) => index,
            None => {
                if !self.lines.is_empty()
                    && self
                        .lines
                        .last()
                        .is_some_and(|line| !line.trim().is_empty())
                {
                    self.lines.push(String::new());
                }
                self.lines.push(section_header);
                self.lines.push(format!("{key} = {value}"));
                return true;
            }
        };

        let section_end = self.lines[section_index + 1..]
            .iter()
            .position(|line| {
                let trimmed = line.trim();
                trimmed.starts_with('[') && trimmed.ends_with(']')
            })
            .map(|offset| section_index + 1 + offset)
            .unwrap_or(self.lines.len());

        let Some(key_index) = self.lines[section_index + 1..section_end]
            .iter()
            .position(|line| {
                line.trim_start()
                    .to_lowercase()
                    .starts_with(&format!("{key} ="))
            })
            .map(|offset| section_index + 1 + offset)
        else {
            self.lines.insert(section_end, format!("{key} = {value}"));
            return true;
        };

        let line = self.lines[key_index].clone();
        let Some((left, right)) = line.split_once('=') else {
            self.lines[key_index] = format!("{key} = {value}");
            return true;
        };
        let mut entries: Vec<String> = right
            .split('|')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(str::to_string)
            .collect();
        if entries
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(value))
        {
            return false;
        }
        entries.push(value.to_string());
        self.lines[key_index] = format!("{}= {}", left.trim_end(), entries.join("|"));
        true
    }
}

impl std::fmt::Display for IniDocument {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.lines.join("\n"))?;
        formatter.write_str("\n")
    }
}
