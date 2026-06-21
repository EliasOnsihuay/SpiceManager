use crate::config::AppConfig;
use crate::errors::Result;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;

pub fn log(config: &AppConfig, scope: &str, message: impl AsRef<str>) -> Result<()> {
    let line = format!(
        "{} [{}] {}\n",
        Utc::now().to_rfc3339(),
        scope,
        message.as_ref()
    );
    print!("{line}");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config.log_file())?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

pub fn recent_lines(config: &AppConfig, max_lines: usize) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(config.log_file()) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    lines
}
