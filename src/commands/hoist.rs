use anyhow::{Context, Result};

use crate::blueprint::BlueprintConfig;
use crate::hoist;

pub fn run(workspace: &str) -> Result<()> {
    let cwd = std::env::current_dir().with_context(|| "getting current directory")?;

    let workspace_path = if workspace.contains('/') || workspace.ends_with(".json") {
        let content = std::fs::read_to_string(workspace)
            .with_context(|| format!("reading blueprint file: {}", workspace))?;
        let config: BlueprintConfig =
            serde_json::from_str(&content).with_context(|| "parsing blueprint JSON")?;
        cwd.join(&config.name)
    } else {
        cwd.join(workspace)
    };

    if !workspace_path.exists() {
        anyhow::bail!(
            "workspace not found: {}\nBuild it first with `scaffold build`.",
            workspace_path.display()
        );
    }

    let repos_dir = workspace_path.join("repos");
    if !repos_dir.exists() {
        anyhow::bail!(
            "repos directory not found: {}\nIs this a valid scaffold workspace?",
            repos_dir.display()
        );
    }

    println!("Hoisting from workspace: {}", workspace_path.display());

    let entries = std::fs::read_dir(&repos_dir)
        .with_context(|| format!("reading repos dir: {}", repos_dir.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| format!("reading entry in {}", repos_dir.display()))?;
        let repo_root = entry.path();

        if !repo_root.is_dir() {
            continue;
        }

        let repo_name = repo_root
            .file_name()
            .and_then(|s| s.to_str())
            .with_context(|| format!("getting repo name for {}", repo_root.display()))?
            .to_string();

        hoist::run_all_strategies(&repo_name, &repo_root, &workspace_path, false)?;
    }

    Ok(())
}
