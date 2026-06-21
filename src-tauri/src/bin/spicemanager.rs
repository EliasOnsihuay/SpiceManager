use clap::{Parser, Subcommand};
use spicemanager::core::workflows::WorkflowEngine;

#[derive(Parser)]
#[command(name = "spicemanager")]
#[command(author = "Elias Onsihuay / Kodhu Technologies")]
#[command(version)]
#[command(about = "Third-party utility for managing Spotify + Spicetify setups.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Detect,
    Install,
    Update,
    Repair,
    Validate,
    Status,
    Doctor,
    #[command(name = "app-update")]
    AppUpdate {
        #[command(subcommand)]
        command: AppUpdateCommand,
    },
}

#[derive(Subcommand)]
enum AppUpdateCommand {
    Check {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        repo: Option<String>,
    },
    Download,
    Status,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("SpiceManager failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> spicemanager::Result<()> {
    let cli = Cli::parse();
    let engine = WorkflowEngine::new()?;
    match cli.command {
        Command::Detect => print_json(&engine.detect_state()?),
        Command::Install => print_json(&engine.install_all()?),
        Command::Update => print_json(&engine.update_all()?),
        Command::Repair => print_json(&engine.repair_all()?),
        Command::Validate => print_json(&engine.validate_all()?),
        Command::Status => {
            let (environment, app_update) = engine.current_status()?;
            println!("Spotify / Spicetify ecosystem status:");
            print_json(&environment)?;
            println!("\nSpiceManager app self-update status:");
            print_json(&app_update)
        }
        Command::Doctor => {
            let report = engine.doctor_report()?;
            print_json(&report)
        }
        Command::AppUpdate { command } => match command {
            AppUpdateCommand::Check { owner, repo } => {
                println!("SpiceManager app self-update check:");
                print_json(&engine.app_update_check(owner, repo)?)
            }
            AppUpdateCommand::Download => {
                println!("SpiceManager app self-update download:");
                print_json(&engine.app_update_download()?)
            }
            AppUpdateCommand::Status => {
                println!("SpiceManager app self-update status:");
                print_json(&engine.app_update_status()?)
            }
        },
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> spicemanager::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
