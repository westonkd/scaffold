mod commands;
mod hoist;
mod utils;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "hoist",
    about = "Hoist AI agent artifacts into the current directory"
)]
struct Cli {
    /// Path to a workspace or plain repo directory. If omitted, reads hoist.json from cwd.
    path: Option<String>,

    /// Replace existing symlinks and re-merge hooks from scratch.
    #[arg(long)]
    force: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::hoist::run(cli.path.as_deref(), cli.force)
}
