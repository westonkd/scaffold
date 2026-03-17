use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::utils::normalize_path;

#[derive(Deserialize)]
struct HoistConfig {
    roots: Vec<String>,
}

/// Directories under the workspace that may contain hoisted symlinks.
const ARTIFACT_DIRS: &[&str] = &["skills", "agents", "commands"];

pub fn run(path: Option<&str>, dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir().with_context(|| "getting current directory")?;

    if dry_run {
        println!("[dry-run] no files will be removed");
    }

    match path {
        Some(p) => {
            let source_root = resolve_source_root(p, &cwd)?;
            println!("Removing artifacts from: {}", source_root.display());
            remove_symlinks_under(&source_root, &cwd, dry_run)?;
            remove_hooks_under(&source_root, &cwd, dry_run)?;
        }
        None => {
            let config_path = cwd.join("hoist.json");
            if !config_path.exists() {
                anyhow::bail!(
                    "hoist.json not found. To prune orphaned artifacts:\n\n  \
                     • Provide a path argument:  hoist unhoist ./some-repo\n  \
                     • Or run from a directory with hoist.json"
                );
            }

            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("reading hoist.json: {}", config_path.display()))?;
            let config: HoistConfig =
                serde_json::from_str(&content).with_context(|| "parsing hoist.json")?;

            let valid_roots: HashSet<PathBuf> = config
                .roots
                .iter()
                .map(|r| {
                    let p = if Path::new(r).is_absolute() {
                        PathBuf::from(r)
                    } else {
                        cwd.join(r)
                    };
                    // Best-effort canonicalize; fall back to normalized path
                    p.canonicalize().unwrap_or_else(|_| normalize_path(&p))
                })
                .collect();

            prune_symlinks(&valid_roots, &cwd, dry_run)?;
            prune_hooks(&valid_roots, &cwd, dry_run)?;
        }
    }

    Ok(())
}

/// Resolve a user-supplied path to a canonicalized PathBuf.
fn resolve_source_root(path: &str, cwd: &Path) -> Result<PathBuf> {
    let p = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };

    if !p.exists() {
        anyhow::bail!("directory not found: {}", p.display());
    }

    p.canonicalize()
        .with_context(|| format!("canonicalizing: {}", p.display()))
}

/// Resolve a relative symlink target to an absolute path without hitting the filesystem
/// (so it works for broken symlinks too).
fn resolve_symlink_abs(symlink: &Path) -> Option<PathBuf> {
    let target = std::fs::read_link(symlink).ok()?;
    let abs = if target.is_absolute() {
        target
    } else {
        symlink.parent()?.join(target)
    };
    Some(normalize_path(&abs))
}

/// Remove symlinks in all artifact dirs whose resolved target starts with `source_root`.
fn remove_symlinks_under(source_root: &Path, workspace_root: &Path, dry_run: bool) -> Result<()> {
    let claude_dir = workspace_root.join(".claude");
    for &subdir in ARTIFACT_DIRS {
        let dir = claude_dir.join(subdir);
        if !dir.is_dir() {
            continue;
        }
        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            // symlink_metadata succeeds even for broken symlinks
            if path.symlink_metadata().is_err() {
                continue;
            }
            if let Some(resolved) = resolve_symlink_abs(&path) {
                if resolved.starts_with(source_root) {
                    remove_artifact(&path, dry_run)?;
                }
            }
        }
    }
    Ok(())
}

/// Remove symlinks in all artifact dirs whose resolved target does NOT start with
/// any path in `valid_roots`.
fn prune_symlinks(
    valid_roots: &HashSet<PathBuf>,
    workspace_root: &Path,
    dry_run: bool,
) -> Result<()> {
    let claude_dir = workspace_root.join(".claude");
    for &subdir in ARTIFACT_DIRS {
        let dir = claude_dir.join(subdir);
        if !dir.is_dir() {
            continue;
        }
        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.symlink_metadata().is_err() {
                continue;
            }
            if let Some(resolved) = resolve_symlink_abs(&path) {
                let under_valid_root = valid_roots.iter().any(|r| resolved.starts_with(r));
                if !under_valid_root {
                    remove_artifact(&path, dry_run)?;
                }
            }
        }
    }
    Ok(())
}

fn remove_artifact(path: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("  [symlink] {}", path.display());
    } else {
        std::fs::remove_file(path).with_context(|| format!("removing {}", path.display()))?;
        println!("  removed {}", path.display());
    }
    Ok(())
}

/// Remove hook entries from .claude/settings.json whose embedded path starts with `source_root`.
fn remove_hooks_under(source_root: &Path, workspace_root: &Path, dry_run: bool) -> Result<()> {
    // Append "/" so "/repos/canvas" doesn't match "/repos/canvas-lms/..."
    let root_prefix = format!("{}/", source_root.to_string_lossy());
    remove_hooks_matching(workspace_root, dry_run, |entry_str| {
        entry_str.contains(root_prefix.as_str())
    })
}

/// Remove hook entries from .claude/settings.json whose embedded path does NOT
/// start with any path in `valid_roots`.
fn prune_hooks(valid_roots: &HashSet<PathBuf>, workspace_root: &Path, dry_run: bool) -> Result<()> {
    // Append "/" so "/repos/canvas" doesn't match "/repos/canvas-lms/..."
    let valid_prefixes: Vec<String> = valid_roots
        .iter()
        .map(|r| format!("{}/", r.to_string_lossy()))
        .collect();
    remove_hooks_matching(workspace_root, dry_run, |entry_str| {
        !valid_prefixes
            .iter()
            .any(|r| entry_str.contains(r.as_str()))
    })
}

/// Generic hook-entry removal. Removes entries from .claude/settings.json["hooks"]
/// where `should_remove(entry.to_string())` returns true.
fn remove_hooks_matching<F>(workspace_root: &Path, dry_run: bool, should_remove: F) -> Result<()>
where
    F: Fn(&str) -> bool,
{
    let settings_path = workspace_root.join(".claude").join("settings.json");
    if !settings_path.is_file() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&settings_path)
        .with_context(|| format!("reading {}", settings_path.display()))?;
    let mut settings: serde_json::Value =
        serde_json::from_str(&content).with_context(|| "parsing settings.json")?;

    let hooks_map = match settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        Some(m) => m,
        None => return Ok(()),
    };

    let mut removed_count = 0usize;
    for (_event, entries) in hooks_map.iter_mut() {
        if let Some(arr) = entries.as_array_mut() {
            let before = arr.len();
            if dry_run {
                for entry in arr.iter() {
                    if should_remove(&entry.to_string()) {
                        println!("  [hook]    {}", entry);
                    }
                }
            } else {
                arr.retain(|entry| !should_remove(&entry.to_string()));
                removed_count += before - arr.len();
            }
        }
    }

    if !dry_run && removed_count > 0 {
        let output =
            serde_json::to_string_pretty(&settings).with_context(|| "serializing settings.json")?;
        std::fs::write(&settings_path, output)
            .with_context(|| format!("writing {}", settings_path.display()))?;
        println!("  updated {}", settings_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_symlink_abs_relative() {
        // Simulates a symlink at /workspace/.claude/skills/repo-skill
        // pointing to ../../../repo/.claude/skills/skill (relative)
        // resolve_symlink_abs can't be tested without a real symlink, but
        // we can test normalize_path which it depends on.
        let p = PathBuf::from("/workspace/.claude/skills/../../../repo/.claude/skills/skill");
        let normalized = normalize_path(&p);
        assert_eq!(normalized, PathBuf::from("/repo/.claude/skills/skill"));
    }
}
