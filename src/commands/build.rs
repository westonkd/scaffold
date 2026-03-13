use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use crate::blueprint::{BlueprintConfig, Dependency};
use crate::store::{relative_path, ScaffoldStore};
use crate::git;

pub fn run(blueprint_path: &str) -> Result<()> {
    let content = std::fs::read_to_string(blueprint_path)
        .with_context(|| format!("reading blueprint file: {}", blueprint_path))?;
    let config: BlueprintConfig = serde_json::from_str(&content)
        .with_context(|| "parsing blueprint JSON")?;

    let store = ScaffoldStore::new();

    // Blueprint workspace lands in cwd/<name>
    let bp_dir = std::env::current_dir()
        .with_context(|| "getting current directory")?
        .join(&config.name);

    // Guard: refuse to overwrite an existing blueprint dir
    if bp_dir.exists() {
        anyhow::bail!(
            "blueprint directory already exists: {}\nDelete or rename it before building again.",
            bp_dir.display()
        );
    }

    // 1. Collect all unique repos (DFS)
    let mut unique_repos: HashMap<String, Dependency> = HashMap::new();
    collect_deps(&config.dependencies, &mut unique_repos);

    // 2. Cache repos
    let projects_dir = store.projects_dir();
    std::fs::create_dir_all(&projects_dir)
        .with_context(|| format!("creating projects dir: {}", projects_dir.display()))?;

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

    // 3. Create blueprint workspace
    let repos_dir = bp_dir.join("repos");
    std::fs::create_dir_all(&repos_dir)
        .with_context(|| format!("creating repos dir: {}", repos_dir.display()))?;

    for (name, dep) in &unique_repos {
        let cache = store.project_cache(name);
        let dest = repos_dir.join(name);
        println!("Creating local clone for: {}", name);
        git::clone_local(&cache, &dest)?;
        if let Some(ref_) = &dep.ref_ {
            git::checkout(&dest, ref_)?;
        }
    }

    // 4. Create symlinks (recursive DFS)
    create_symlinks(&config.dependencies, &bp_dir, None)?;

    // 5. Copy blueprint JSON into workspace root
    std::fs::copy(blueprint_path, bp_dir.join("blueprint.json"))
        .with_context(|| format!(
            "copying blueprint to workspace: {}",
            bp_dir.join("blueprint.json").display()
        ))?;

    println!("Built '{}' at: {}", config.name, bp_dir.display());
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

fn create_symlinks(
    deps: &[Dependency],
    bp_dir: &Path,
    parent_name: Option<&str>,
) -> Result<()> {
    for dep in deps {
        let repos_dir = bp_dir.join("repos");

        match parent_name {
            None => {
                // Top-level: <bp>/<dep.name> -> repos/<dep.name>
                let link_path = bp_dir.join(&dep.name);
                let target = relative_path(bp_dir, &repos_dir.join(&dep.name));
                println!("Symlinking: {} -> {}", link_path.display(), target.display());
                std::os::unix::fs::symlink(&target, &link_path)
                    .with_context(|| format!("creating symlink {}", link_path.display()))?;
            }
            Some(parent) => {
                // Sub-dep: <bp>/repos/<parent>/<dep.path>/<dep.name> -> relative to <dep.name>
                let dep_path = dep.path.as_deref().unwrap_or("");
                let link_dir: PathBuf = repos_dir.join(parent).join(dep_path);
                let link_path = link_dir.join(&dep.name);
                let target_dir = repos_dir.join(&dep.name);
                let target = relative_path(&link_dir, &target_dir);
                println!("Symlinking: {} -> {}", link_path.display(), target.display());
                std::fs::create_dir_all(&link_dir)
                    .with_context(|| format!("creating dir: {}", link_dir.display()))?;
                std::os::unix::fs::symlink(&target, &link_path)
                    .with_context(|| format!("creating symlink {}", link_path.display()))?;
            }
        }

        // Recurse into sub-deps
        if let Some(sub_deps) = &dep.dependencies {
            create_symlinks(sub_deps, bp_dir, Some(&dep.name))?;
        }
    }
    Ok(())
}
