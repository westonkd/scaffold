mod commands;
mod hoist;
mod s3;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scaffold", about = "Manage AI context artifacts across your organization")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new skill and push it to S3.
    New {
        /// Skill name (spaces normalized to hyphens, lowercased).
        name: String,

        /// Short description of the skill.
        #[arg(long, default_value = "")]
        description: String,

        /// Create only SKILL.md; skip reference file scaffolding.
        #[arg(long)]
        minimal: bool,
    },

    /// Hoist AI agent artifacts into the current directory.
    Hoist {
        /// Path to a workspace or plain repo directory. If omitted, reads hoist.json from cwd.
        path: Option<String>,

        /// Replace existing symlinks and re-merge hooks from scratch.
        #[arg(long)]
        force: bool,
    },

    /// Remove hoisted artifacts from the workspace.
    Unhoist {
        /// Path to un-hoist. If omitted, reads hoist.json and prunes orphaned artifacts.
        path: Option<String>,

        /// Print what would be removed without removing anything.
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, description, minimal } => {
            commands::new::run(&name, &description, minimal).await
        }
        Commands::Hoist { path, force } => {
            commands::hoist::run(path.as_deref(), force)
        }
        Commands::Unhoist { path, dry_run } => {
            commands::unhoist::run(path.as_deref(), dry_run)
        }
    }
}
