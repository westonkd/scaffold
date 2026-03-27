mod commands;
mod credentials;
mod hoist;
mod s3;
mod settings;
mod storage;
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

        /// Print S3 region, bucket, and each uploaded file.
        #[arg(long, short)]
        verbose: bool,
    },

    /// Pull a skill from S3 and open it for editing.
    Edit {
        /// Skill name to open for editing.
        name: Option<String>,

        /// Print each file downloaded during the pull step.
        #[arg(long, short)]
        verbose: bool,
    },

    /// List skills.
    List {
        /// List all available skills in S3 instead of locally installed skills.
        #[arg(long)]
        remote: bool,
    },

    /// Pull skills from S3 into ~/.scaffold/.
    Pull {
        /// Skill name to pull. If omitted, pulls all skills in the bucket.
        name: Option<String>,

        /// Print each file as it is downloaded.
        #[arg(long, short)]
        verbose: bool,
    },

    /// Link a skill into the current working directory.
    Link {
        /// Skill name to link. If omitted, links all scopes from .scaffold-artifacts.
        name: Option<String>,

        /// Replace existing symlinks.
        #[arg(long, short)]
        force: bool,
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

    /// Get or set scaffold configuration values.
    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommands,
    },

    /// Store or clear the API Gateway access token.
    Login {
        /// Token to store. If omitted, reads from stdin.
        #[arg(long)]
        token: Option<String>,

        /// Remove the stored token.
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Subcommand)]
enum ConfigSubcommands {
    /// Print the value of a configuration key.
    Get {
        /// Configuration key (e.g. bucket).
        key: String,
    },

    /// Set the value of a configuration key.
    Set {
        /// Configuration key (e.g. bucket).
        key: String,
        /// Value to assign.
        value: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, description, minimal, verbose } => {
            commands::new::run(&name, &description, minimal, verbose).await
        }
        Commands::Edit { name, verbose } => match name {
            Some(n) => commands::edit::run(&n, verbose).await,
            None => {
                eprintln!("Error: skill name is required. Usage: scaffold edit <name>");
                std::process::exit(1);
            }
        },
        Commands::List { remote } => {
            commands::list::run(remote).await
        }
        Commands::Pull { name, verbose } => {
            commands::pull::run(name.as_deref(), verbose).await
        }
        Commands::Link { name, force } => {
            commands::link::run(name.as_deref(), force)
        }
        Commands::Hoist { path, force } => {
            commands::hoist::run(path.as_deref(), force)
        }
        Commands::Unhoist { path, dry_run } => {
            commands::unhoist::run(path.as_deref(), dry_run)
        }
        Commands::Config { subcommand } => match subcommand {
            ConfigSubcommands::Get { key } => commands::config::get(&key),
            ConfigSubcommands::Set { key, value } => commands::config::set(&key, &value),
        },
        Commands::Login { token, clear } => {
            commands::login::run(token.as_deref(), clear)
        }
    }
}
