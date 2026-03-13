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
    /// Hoist AI artifacts from workspace repos up to the workspace root
    Hoist {
        /// Workspace name or path to blueprint JSON
        workspace: String,
    },
    /// Update repos and re-hoist artifacts in an existing scaffold workspace
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { blueprint } => commands::build::run(&blueprint)?,
        Commands::Hoist { workspace } => commands::hoist::run(&workspace)?,
        Commands::Update => commands::update::run()?,
    }
    Ok(())
}
