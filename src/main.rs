mod blueprint;
mod commands;
mod git;
mod store;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "scaffold", about = "Scaffold monorepo workspaces from blueprints")]
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { blueprint } => commands::build::run(&blueprint)?,
    }
    Ok(())
}
