use anyhow::{Context, Result};
use serde::Deserialize;

use crate::hoist;

#[derive(Deserialize)]
struct HoistConfig {
    roots: Vec<String>,
}

pub fn run(path: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir().with_context(|| "getting current directory")?;

    match path {
        None => {
            // No-arg mode: read hoist.json from cwd
            let config_path = cwd.join("hoist.json");
            if !config_path.exists() {
                anyhow::bail!(
                    "hoist.json not found. Create one or provide a path argument."
                );
            }

            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("reading hoist.json: {}", config_path.display()))?;
            let config: HoistConfig =
                serde_json::from_str(&content).with_context(|| "parsing hoist.json")?;

            for root in &config.roots {
                let root_path = if std::path::Path::new(root).is_absolute() {
                    std::path::PathBuf::from(root)
                } else {
                    cwd.join(root)
                };

                if !root_path.exists() {
                    anyhow::bail!("root not found: {}", root_path.display());
                }

                hoist_from_root(&root_path, &cwd)?;
            }

            Ok(())
        }

        Some(path) => {
            // Explicit-path mode: resolve source, hoist into cwd
            let resolved_path = {
                let p = std::path::Path::new(path);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    cwd.join(path)
                }
            };

            if !resolved_path.exists() {
                anyhow::bail!("directory not found: {}", resolved_path.display());
            }

            hoist_from_root(&resolved_path, &cwd)
        }
    }
}

fn hoist_from_root(root: &std::path::Path, cwd: &std::path::Path) -> Result<()> {
    let repos_dir = root.join("repos");
    if repos_dir.exists() {
        // Workspace mode: iterate repos/, hoist each into cwd
        println!("Hoisting from workspace: {}", root.display());

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

            hoist::run_all_strategies(&repo_name, &repo_root, cwd, false)?;
        }
    } else {
        // Single-repo mode: treat root as a plain repo, hoist into cwd
        let repo_name = root
            .file_name()
            .and_then(|s| s.to_str())
            .with_context(|| format!("getting repo name for {}", root.display()))?
            .to_string();

        println!("Hoisting from repo: {}", root.display());
        hoist::run_all_strategies(&repo_name, root, cwd, false)?;
    }

    Ok(())
}
