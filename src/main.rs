mod blueprint;
mod commands;
mod git;
mod hoist;
mod store;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "scaffold",
    about = "Scaffold monorepo workspaces from blueprints"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a workspace from a blueprint JSON file
    Build {
        /// Path to the blueprint JSON file
        blueprint: String,
    },
    /// Hoist AI artifacts from a workspace or plain repo into the current directory
    Hoist {
        /// Workspace name, path to blueprint JSON, or path to a plain repo directory.
        /// If omitted, hoists from the current directory (which must be a scaffold workspace).
        path: Option<String>,
    },
    /// Update repos and re-hoist artifacts in an existing scaffold workspace
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { blueprint } => commands::build::run(&blueprint)?,
        Commands::Hoist { path } => commands::hoist::run(path.as_deref())?,
        Commands::Update => commands::update::run()?,
    }
    Ok(())
}
