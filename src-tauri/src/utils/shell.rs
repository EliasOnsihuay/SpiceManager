use crate::models::ShellCommandResult;
use chrono::Utc;
use std::process::Command;

pub fn run(command: &str, args: &[&str]) -> ShellCommandResult {
    let started_at = Utc::now();
    let output = Command::new(command).args(args).output();
    let ended_at = Utc::now();
    match output {
        Ok(output) => ShellCommandResult {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code(),
            started_at,
            ended_at,
            success: output.status.success(),
        },
        Err(err) => ShellCommandResult {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            stdout: String::new(),
            stderr: err.to_string(),
            exit_code: None,
            started_at,
            ended_at,
            success: false,
        },
    }
}
