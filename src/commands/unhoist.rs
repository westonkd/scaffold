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

/// Resolve a symlink to an absolute, canonical path.
///
/// Canonicalizes so that path-component symlinks (e.g. `/var` → `/private/var` on macOS)
/// are resolved before `starts_with` comparisons. Falls back to `normalize_path` for
/// broken symlinks where the target does not exist and `canonicalize` would fail.
fn resolve_symlink_abs(symlink: &Path) -> Option<PathBuf> {
    let target = std::fs::read_link(symlink).ok()?;
    let abs = if target.is_absolute() {
        target
    } else {
        symlink.parent()?.join(target)
    };
    Some(abs.canonicalize().unwrap_or_else(|_| normalize_path(&abs)))
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
    use std::fs;
    use std::os::unix::fs as unix_fs;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_symlink_abs_relative() {
        let p = PathBuf::from("/workspace/.claude/skills/../../../repo/.claude/skills/skill");
        let normalized = normalize_path(&p);
        assert_eq!(normalized, PathBuf::from("/repo/.claude/skills/skill"));
    }

    fn setup_symlink_in_skills(
        source_file: &std::path::Path,
        workspace: &std::path::Path,
        link_name: &str,
    ) {
        let skills_dir = workspace.join(".claude/skills");
        fs::create_dir_all(&skills_dir).unwrap();
        unix_fs::symlink(source_file, skills_dir.join(link_name)).unwrap();
    }

    #[test]
    fn remove_symlinks_under_removes_matching_symlink() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let canon_source = source.path().canonicalize().unwrap();
        let skill_file = canon_source.join("skill.md");
        fs::write(&skill_file, "# Skill").unwrap();

        setup_symlink_in_skills(&skill_file, workspace.path(), "repo-skill.md");

        remove_symlinks_under(&canon_source, workspace.path(), false).unwrap();

        assert!(!workspace.path().join(".claude/skills/repo-skill.md").symlink_metadata().is_ok());
    }

    #[test]
    fn remove_symlinks_under_leaves_unrelated_symlinks() {
        let source = tempdir().unwrap();
        let other_source = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let canon_source = source.path().canonicalize().unwrap();
        let canon_other = other_source.path().canonicalize().unwrap();

        let skill_file = canon_source.join("skill.md");
        fs::write(&skill_file, "# Skill").unwrap();
        let other_file = canon_other.join("other.md");
        fs::write(&other_file, "# Other").unwrap();

        setup_symlink_in_skills(&skill_file, workspace.path(), "repo-skill.md");
        setup_symlink_in_skills(&other_file, workspace.path(), "other-skill.md");

        remove_symlinks_under(&canon_source, workspace.path(), false).unwrap();

        assert!(!workspace.path().join(".claude/skills/repo-skill.md").symlink_metadata().is_ok());
        assert!(workspace.path().join(".claude/skills/other-skill.md").symlink_metadata().is_ok());
    }

    #[test]
    fn remove_symlinks_under_dry_run_does_not_remove() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let canon_source = source.path().canonicalize().unwrap();
        let skill_file = canon_source.join("skill.md");
        fs::write(&skill_file, "# Skill").unwrap();

        setup_symlink_in_skills(&skill_file, workspace.path(), "repo-skill.md");

        remove_symlinks_under(&canon_source, workspace.path(), true).unwrap();

        assert!(workspace.path().join(".claude/skills/repo-skill.md").symlink_metadata().is_ok());
    }

    #[test]
    fn remove_symlinks_under_no_op_when_skills_dir_absent() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let canon_source = source.path().canonicalize().unwrap();

        remove_symlinks_under(&canon_source, workspace.path(), false).unwrap();
    }

    #[test]
    fn prune_symlinks_removes_orphaned_symlink() {
        let valid_source = tempdir().unwrap();
        let orphan_source = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let canon_valid = valid_source.path().canonicalize().unwrap();
        let canon_orphan = orphan_source.path().canonicalize().unwrap();

        let valid_file = canon_valid.join("valid.md");
        fs::write(&valid_file, "# Valid").unwrap();
        let orphan_file = canon_orphan.join("orphan.md");
        fs::write(&orphan_file, "# Orphan").unwrap();

        setup_symlink_in_skills(&valid_file, workspace.path(), "repo-valid.md");
        setup_symlink_in_skills(&orphan_file, workspace.path(), "repo-orphan.md");

        let valid_roots: HashSet<PathBuf> = [canon_valid].into_iter().collect();
        prune_symlinks(&valid_roots, workspace.path(), false).unwrap();

        assert!(workspace.path().join(".claude/skills/repo-valid.md").symlink_metadata().is_ok());
        assert!(!workspace.path().join(".claude/skills/repo-orphan.md").symlink_metadata().is_ok());
    }

    #[test]
    fn prune_symlinks_dry_run_keeps_orphan() {
        let valid_source = tempdir().unwrap();
        let orphan_source = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let canon_valid = valid_source.path().canonicalize().unwrap();
        let canon_orphan = orphan_source.path().canonicalize().unwrap();

        let valid_file = canon_valid.join("valid.md");
        fs::write(&valid_file, "# Valid").unwrap();
        let orphan_file = canon_orphan.join("orphan.md");
        fs::write(&orphan_file, "# Orphan").unwrap();

        setup_symlink_in_skills(&valid_file, workspace.path(), "repo-valid.md");
        setup_symlink_in_skills(&orphan_file, workspace.path(), "repo-orphan.md");

        let valid_roots: HashSet<PathBuf> = [canon_valid].into_iter().collect();
        prune_symlinks(&valid_roots, workspace.path(), true).unwrap();

        assert!(workspace.path().join(".claude/skills/repo-orphan.md").symlink_metadata().is_ok());
    }

    fn write_settings(workspace: &std::path::Path, settings: &serde_json::Value) {
        let settings_path = workspace.join(".claude/settings.json");
        fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
        fs::write(&settings_path, serde_json::to_string_pretty(settings).unwrap()).unwrap();
    }

    fn read_settings(workspace: &std::path::Path) -> serde_json::Value {
        let content = fs::read_to_string(workspace.join(".claude/settings.json")).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn remove_hooks_under_removes_matching_hook_entries() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let canon_source = source.path().canonicalize().unwrap();

        let hook_cmd = format!("{}/run.sh", canon_source.display());
        let settings = serde_json::json!({
            "hooks": {
                "PostToolUse": [hook_cmd, "other-command"]
            }
        });
        write_settings(workspace.path(), &settings);

        remove_hooks_under(&canon_source, workspace.path(), false).unwrap();

        let result = read_settings(workspace.path());
        let entries = result["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].as_str().unwrap().contains("other-command"));
    }

    #[test]
    fn remove_hooks_under_dry_run_does_not_modify_settings() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let canon_source = source.path().canonicalize().unwrap();

        let hook_cmd = format!("{}/run.sh", canon_source.display());
        let settings = serde_json::json!({
            "hooks": {"PostToolUse": [hook_cmd]}
        });
        write_settings(workspace.path(), &settings);

        remove_hooks_under(&canon_source, workspace.path(), true).unwrap();

        let result = read_settings(workspace.path());
        assert_eq!(result["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn remove_hooks_under_no_op_when_settings_absent() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let canon_source = source.path().canonicalize().unwrap();

        remove_hooks_under(&canon_source, workspace.path(), false).unwrap();
    }

    #[test]
    fn prune_hooks_removes_orphaned_hook_entries() {
        let valid_source = tempdir().unwrap();
        let orphan_source = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let canon_valid = valid_source.path().canonicalize().unwrap();
        let canon_orphan = orphan_source.path().canonicalize().unwrap();

        let valid_hook = format!("{}/run.sh", canon_valid.display());
        let orphan_hook = format!("{}/run.sh", canon_orphan.display());
        let settings = serde_json::json!({
            "hooks": {
                "PostToolUse": [valid_hook, orphan_hook]
            }
        });
        write_settings(workspace.path(), &settings);

        let valid_roots: HashSet<PathBuf> = [canon_valid.clone()].into_iter().collect();
        prune_hooks(&valid_roots, workspace.path(), false).unwrap();

        let result = read_settings(workspace.path());
        let entries = result["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        let remaining = entries[0].as_str().unwrap();
        assert!(remaining.contains(canon_valid.to_str().unwrap()));
    }

    #[test]
    fn prune_hooks_keeps_all_hooks_when_all_roots_valid() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let canon_source = source.path().canonicalize().unwrap();

        let hook1 = format!("{}/a.sh", canon_source.display());
        let hook2 = format!("{}/b.sh", canon_source.display());
        let settings = serde_json::json!({
            "hooks": {"PostToolUse": [hook1, hook2]}
        });
        write_settings(workspace.path(), &settings);

        let valid_roots: HashSet<PathBuf> = [canon_source].into_iter().collect();
        prune_hooks(&valid_roots, workspace.path(), false).unwrap();

        let result = read_settings(workspace.path());
        assert_eq!(result["hooks"]["PostToolUse"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn remove_hooks_under_does_not_match_path_prefix_substring() {
        let source = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        let canon_source = source.path().canonicalize().unwrap();

        let canon_source_str = canon_source.to_string_lossy();
        let similar_path = format!("{}-other/run.sh", canon_source_str);
        let exact_path = format!("{}/run.sh", canon_source_str);

        let settings = serde_json::json!({
            "hooks": {
                "PostToolUse": [exact_path, similar_path]
            }
        });
        write_settings(workspace.path(), &settings);

        remove_hooks_under(&canon_source, workspace.path(), false).unwrap();

        let result = read_settings(workspace.path());
        let entries = result["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 1, "only exact-prefix hook should be removed");
        assert!(entries[0].as_str().unwrap().contains("-other/"));
    }
}
