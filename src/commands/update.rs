use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::blueprint::{BlueprintConfig, Dependency};
use crate::store::ScaffoldStore;
use crate::git;
use crate::hoist;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir().with_context(|| "getting current directory")?;

    let blueprint_path = cwd.join("blueprint.json");
    if !blueprint_path.exists() {
        anyhow::bail!(
            "blueprint.json not found in current directory.\nRun this command from within a scaffold workspace."
        );
    }

    let repos_dir = cwd.join("repos");
    if !repos_dir.exists() {
        anyhow::bail!(
            "repos/ directory not found.\nIs this a valid scaffold workspace?"
        );
    }

    let content = std::fs::read_to_string(&blueprint_path)
        .with_context(|| format!("reading blueprint file: {}", blueprint_path.display()))?;
    let config: BlueprintConfig = serde_json::from_str(&content)
        .with_context(|| "parsing blueprint JSON")?;

    let mut unique_repos: HashMap<String, Dependency> = HashMap::new();
    collect_deps(&config.dependencies, &mut unique_repos);

    let store = ScaffoldStore::new();

    // Update cached repos
    for (name, dep) in &unique_repos {
        let cache = store.project_cache(name);
        if cache.exists() {
            println!("Updating cached repo: {}", name);
            git::fetch(&cache)?;
            if let Some(ref_) = &dep.ref_ {
                git::checkout(&cache, ref_)?;
            }
            git::pull(&cache)?;
        } else {
            println!("Cloning repo: {} from {}", name, dep.source);
            git::clone(&dep.source, &cache)?;
            if let Some(ref_) = &dep.ref_ {
                git::checkout(&cache, ref_)?;
            }
        }
    }

    // Update local clones in ./repos/
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

        println!("Updating local clone: {}", repo_name);
        git::fetch(&repo_root)?;
        if let Some(dep) = unique_repos.get(&repo_name) {
            if let Some(ref_) = &dep.ref_ {
                git::checkout(&repo_root, ref_)?;
            }
        }
        git::pull(&repo_root)?;

        hoist::run_all_strategies(&repo_name, &repo_root, &cwd, true)?;
    }

    println!("Updated workspace: {}", cwd.display());
    Ok(())
}

fn collect_deps(deps: &[Dependency], map: &mut HashMap<String, Dependency>) {
    for dep in deps {
        map.entry(dep.name.clone()).or_insert_with(|| dep.clone());
        if let Some(sub_deps) = &dep.dependencies {
            collect_deps(sub_deps, map);
        }
    }
}
