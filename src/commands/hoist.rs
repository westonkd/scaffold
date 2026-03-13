use anyhow::{Context, Result};

use crate::blueprint::BlueprintConfig;
use crate::hoist;

pub fn run(path: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir().with_context(|| "getting current directory")?;

    match path {
        None => {
            // No-arg mode: cwd must be a scaffold workspace
            if !cwd.join("blueprint.json").exists() {
                anyhow::bail!(
                    "blueprint.json not found.\nRun this command from within a scaffold workspace."
                );
            }

            let repos_dir = cwd.join("repos");
            if !repos_dir.exists() {
                anyhow::bail!("repos/ directory not found.\nIs this a valid scaffold workspace?");
            }

            println!("Hoisting from workspace: {}", cwd.display());

            let entries = std::fs::read_dir(&repos_dir)
                .with_context(|| format!("reading repos dir: {}", repos_dir.display()))?;

            for entry in entries {
                let entry =
                    entry.with_context(|| format!("reading entry in {}", repos_dir.display()))?;
                let repo_root = entry.path();

                if !repo_root.is_dir() {
                    continue;
                }

                let repo_name = repo_root
                    .file_name()
                    .and_then(|s| s.to_str())
                    .with_context(|| format!("getting repo name for {}", repo_root.display()))?
                    .to_string();

                hoist::run_all_strategies(&repo_name, &repo_root, &cwd, false)?;
            }

            Ok(())
        }

        Some(path) => {
            // Explicit-path mode: resolve source, hoist into cwd
            let resolved_path = if path.ends_with(".json") {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("reading blueprint file: {}", path))?;
                let config: BlueprintConfig =
                    serde_json::from_str(&content).with_context(|| "parsing blueprint JSON")?;
                cwd.join(&config.name)
            } else {
                let p = std::path::Path::new(path);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    cwd.join(path)
                }
            };

            if !resolved_path.exists() {
                anyhow::bail!(
                    "path not found: {}\nBuild it first with `scaffold build`.",
                    resolved_path.display()
                );
            }

            let repos_dir = resolved_path.join("repos");
            if repos_dir.exists() {
                // Workspace mode: iterate repos/, hoist each into cwd
                println!("Hoisting from workspace: {}", resolved_path.display());

                let entries = std::fs::read_dir(&repos_dir)
                    .with_context(|| format!("reading repos dir: {}", repos_dir.display()))?;

                for entry in entries {
                    let entry = entry
                        .with_context(|| format!("reading entry in {}", repos_dir.display()))?;
                    let repo_root = entry.path();

                    if !repo_root.is_dir() {
                        continue;
                    }

                    let repo_name = repo_root
                        .file_name()
                        .and_then(|s| s.to_str())
                        .with_context(|| format!("getting repo name for {}", repo_root.display()))?
                        .to_string();

                    hoist::run_all_strategies(&repo_name, &repo_root, &cwd, false)?;
                }
            } else {
                // Single-repo mode: treat resolved_path as a plain repo, hoist into cwd
                let repo_name = resolved_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .with_context(|| format!("getting repo name for {}", resolved_path.display()))?
                    .to_string();

                println!("Hoisting from repo: {}", resolved_path.display());
                hoist::run_all_strategies(&repo_name, &resolved_path, &cwd, false)?;
            }

            Ok(())
        }
    }
}
