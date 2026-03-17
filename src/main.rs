mod commands;
mod hoist;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "hoist",
    about = "Hoist AI agent artifacts into the current directory"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to a workspace or plain repo directory. If omitted, reads hoist.json from cwd.
    path: Option<String>,

    /// Replace existing symlinks and re-merge hooks from scratch.
    #[arg(long)]
    force: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Remove hoisted artifacts from the workspace.
    ///
    /// With a path, removes all artifacts whose source resolves into that directory.
    /// Without a path, reads hoist.json and removes artifacts from any root no longer listed.
    Unhoist {
        /// Path to un-hoist. If omitted, reads hoist.json and prunes orphaned artifacts.
        path: Option<String>,

        /// Print what would be removed without removing anything.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Unhoist { path, dry_run }) => {
            commands::unhoist::run(path.as_deref(), dry_run)
        }
        None => commands::hoist::run(cli.path.as_deref(), cli.force),
    }
}
