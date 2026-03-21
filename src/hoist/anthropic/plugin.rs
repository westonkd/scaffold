use anyhow::{Context, Result};
use std::path::Path;

use crate::hoist::HoistStrategy;
use crate::utils::relative_path;

pub struct PluginStrategy;

impl HoistStrategy for PluginStrategy {
    fn name(&self) -> &str {
        "anthropic/plugin"
    }

    fn detect(&self, repo_root: &Path) -> bool {
        repo_root
            .join(".claude-plugin")
            .join("plugin.json")
            .is_file()
    }

    fn hoist(
        &self,
        repo_name: &str,
        repo_root: &Path,
        workspace_root: &Path,
        force: bool,
    ) -> Result<()> {
        hoist_plugin_dir(repo_name, repo_root, workspace_root, force)
    }
}

/// Hoist all artifacts from a plugin directory into the workspace.
///
/// Called by `PluginStrategy::hoist` for standalone plugin repos, and by
/// `MarketplaceStrategy::hoist` for each locally-sourced plugin in a marketplace.
pub fn hoist_plugin_dir(
    plugin_name: &str,
    plugin_root: &Path,
    workspace_root: &Path,
    force: bool,
) -> Result<()> {
    hoist_skills(plugin_name, plugin_root, workspace_root, force)?;
    hoist_md_artifacts("agents", plugin_name, plugin_root, workspace_root, force)?;
    hoist_md_artifacts("commands", plugin_name, plugin_root, workspace_root, force)?;
    hoist_hooks(plugin_name, plugin_root, workspace_root, force)?;
    Ok(())
}

fn hoist_skills(
    plugin_name: &str,
    plugin_root: &Path,
    workspace_root: &Path,
    force: bool,
) -> Result<()> {
    let src_dir = plugin_root.join("skills");
    if !src_dir.is_dir() {
        return Ok(());
    }

    let dst_dir = workspace_root.join(".claude").join("skills");
    std::fs::create_dir_all(&dst_dir)
        .with_context(|| format!("creating destination dir: {}", dst_dir.display()))?;

    for entry in std::fs::read_dir(&src_dir)
        .with_context(|| format!("reading skills dir: {}", src_dir.display()))?
    {
        let entry = entry.with_context(|| format!("reading entry in {}", src_dir.display()))?;
        let path = entry.path();

        if !path.is_dir() || !path.join("SKILL.md").is_file() {
            continue;
        }

        let skill_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .with_context(|| format!("getting name for {}", path.display()))?;

        let dst_path = dst_dir.join(format!("{}-{}", plugin_name, skill_name));
        create_symlink(
            &path,
            &dst_path,
            &dst_dir,
            workspace_root,
            force,
            skill_name,
        )?;
    }

    Ok(())
}

fn hoist_md_artifacts(
    artifact_type: &str,
    plugin_name: &str,
    plugin_root: &Path,
    workspace_root: &Path,
    force: bool,
) -> Result<()> {
    let src_dir = plugin_root.join(artifact_type);
    if !src_dir.is_dir() {
        return Ok(());
    }

    let dst_dir = workspace_root.join(".claude").join(artifact_type);
    std::fs::create_dir_all(&dst_dir)
        .with_context(|| format!("creating destination dir: {}", dst_dir.display()))?;

    for entry in std::fs::read_dir(&src_dir)
        .with_context(|| format!("reading {} dir: {}", artifact_type, src_dir.display()))?
    {
        let entry = entry.with_context(|| format!("reading entry in {}", src_dir.display()))?;
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let basename = path
            .file_name()
            .and_then(|s| s.to_str())
            .with_context(|| format!("getting name for {}", path.display()))?;

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(basename);

        let dst_path = dst_dir.join(format!("{}-{}.md", plugin_name, stem));
        create_symlink(&path, &dst_path, &dst_dir, workspace_root, force, basename)?;
    }

    Ok(())
}

fn create_symlink(
    src: &Path,
    dst: &Path,
    dst_dir: &Path,
    workspace_root: &Path,
    force: bool,
    name: &str,
) -> Result<()> {
    if dst.exists() && !force {
        eprintln!(
            "warning: skipping '{}' — destination already exists: {}",
            name,
            dst.display()
        );
        return Ok(());
    }

    if dst.exists() {
        if dst.is_dir() {
            std::fs::remove_dir_all(dst)
                .with_context(|| format!("removing existing destination: {}", dst.display()))?;
        } else {
            std::fs::remove_file(dst)
                .with_context(|| format!("removing existing destination: {}", dst.display()))?;
        }
    }

    // Also remove a dangling symlink (exists() returns false for broken symlinks)
    if dst.symlink_metadata().is_ok() {
        std::fs::remove_file(dst)
            .with_context(|| format!("removing existing symlink: {}", dst.display()))?;
    }

    let rel_target = relative_path(dst_dir, src);
    std::os::unix::fs::symlink(&rel_target, dst).with_context(|| {
        format!(
            "creating symlink {} -> {}",
            dst.display(),
            rel_target.display()
        )
    })?;

    if let Ok(relative) = dst.strip_prefix(workspace_root) {
        crate::utils::add_to_git_exclude(workspace_root, &relative.to_string_lossy())?;
    } else {
        eprintln!(
            "warning: could not compute relative path for git exclude: {}",
            dst.display()
        );
    }

    Ok(())
}

fn hoist_hooks(
    plugin_name: &str,
    plugin_root: &Path,
    workspace_root: &Path,
    force: bool,
) -> Result<()> {
    let hooks_json_path = plugin_root.join("hooks").join("hooks.json");
    if !hooks_json_path.is_file() {
        return Ok(());
    }

    let plugin_root_abs = plugin_root
        .canonicalize()
        .with_context(|| format!("canonicalizing plugin root: {}", plugin_root.display()))?;
    let plugin_root_str = plugin_root_abs.to_string_lossy().into_owned();

    // Read and substitute ${CLAUDE_PLUGIN_ROOT} with the absolute plugin path
    let raw = std::fs::read_to_string(&hooks_json_path)
        .with_context(|| format!("reading hooks: {}", hooks_json_path.display()))?;
    let substituted = raw.replace("${CLAUDE_PLUGIN_ROOT}", &plugin_root_str);

    let plugin_hooks: serde_json::Value = serde_json::from_str(&substituted)
        .with_context(|| format!("parsing hooks.json: {}", hooks_json_path.display()))?;

    let plugin_hooks_map = match plugin_hooks
        .get("hooks")
        .and_then(|v| v.as_object())
        .cloned()
    {
        Some(m) => m,
        None => return Ok(()),
    };

    // Read or create workspace .claude/settings.json
    let settings_path = workspace_root.join(".claude").join("settings.json");
    std::fs::create_dir_all(settings_path.parent().unwrap())
        .with_context(|| "creating .claude dir")?;

    let mut settings: serde_json::Value = if settings_path.is_file() {
        let s = std::fs::read_to_string(&settings_path)
            .with_context(|| format!("reading settings.json: {}", settings_path.display()))?;
        serde_json::from_str(&s).with_context(|| "parsing settings.json")?
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    let settings_obj = settings.as_object_mut().unwrap();

    // _hoist_registry tracks each plugin's exact hook contributions so dedup and
    // force-removal are reliable regardless of whether hook entries reference the plugin path.
    settings_obj
        .entry("_hoist_registry")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    let already_present = settings_obj["_hoist_registry"]
        .as_object()
        .map(|r| r.contains_key(&plugin_root_str))
        .unwrap_or(false);

    if already_present && !force {
        eprintln!(
            "warning: skipping hooks from '{}' — already merged (re-run with --force to re-merge)",
            plugin_name
        );
        return Ok(());
    }

    settings_obj
        .entry("hooks")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    // With force=true, remove the previously-contributed entries using the registry,
    // then clear the registry entry so we can re-add cleanly.
    if force {
        let old_contributions = settings_obj["_hoist_registry"]
            .get(&plugin_root_str)
            .cloned();

        if let Some(old_map) = old_contributions.as_ref().and_then(|v| v.as_object()) {
            let hooks_map = settings_obj["hooks"].as_object_mut().unwrap();
            for (event, old_entries) in old_map {
                if let (Some(existing_arr), Some(old_arr)) = (
                    hooks_map.get_mut(event).and_then(|v| v.as_array_mut()),
                    old_entries.as_array(),
                ) {
                    existing_arr.retain(|e| !old_arr.contains(e));
                }
            }
        }

        settings_obj["_hoist_registry"]
            .as_object_mut()
            .unwrap()
            .remove(&plugin_root_str);
    }

    // Extend each event array with the plugin's hook entries and record in registry
    let hooks_map = settings_obj["hooks"].as_object_mut().unwrap();
    for (event, plugin_entries) in &plugin_hooks_map {
        let existing = hooks_map
            .entry(event.clone())
            .or_insert_with(|| serde_json::Value::Array(vec![]));
        if let (Some(existing_arr), Some(new_arr)) =
            (existing.as_array_mut(), plugin_entries.as_array())
        {
            existing_arr.extend(new_arr.iter().cloned());
        }
    }

    settings_obj["_hoist_registry"]
        .as_object_mut()
        .unwrap()
        .insert(
            plugin_root_str,
            serde_json::Value::Object(
                plugin_hooks_map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
        );

    let output =
        serde_json::to_string_pretty(&settings).with_context(|| "serializing settings.json")?;
    std::fs::write(&settings_path, output)
        .with_context(|| format!("writing settings.json: {}", settings_path.display()))?;

    println!(
        "    merged hooks from '{}' into {}",
        plugin_name,
        settings_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_plugin_json(dir: &std::path::Path) {
        fs::create_dir_all(dir.join(".claude-plugin")).unwrap();
        fs::write(dir.join(".claude-plugin/plugin.json"), "{}").unwrap();
    }

    #[test]
    fn detect_returns_false_without_plugin_json() {
        let dir = tempdir().unwrap();
        assert!(!PluginStrategy.detect(dir.path()));
    }

    #[test]
    fn detect_returns_false_with_only_directory() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".claude-plugin")).unwrap();
        assert!(!PluginStrategy.detect(dir.path()));
    }

    #[test]
    fn detect_returns_true_with_plugin_json() {
        let dir = tempdir().unwrap();
        make_plugin_json(dir.path());
        assert!(PluginStrategy.detect(dir.path()));
    }

    #[test]
    fn hoist_plugin_dir_succeeds_with_empty_plugin() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();
        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();
        assert!(!workspace.path().join(".claude/skills").exists());
        assert!(!workspace.path().join(".claude/agents").exists());
        assert!(!workspace.path().join(".claude/commands").exists());
    }

    #[test]
    fn hoist_plugin_hoists_skills_directory() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skill_dir = plugin.path().join("skills/my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let symlink = workspace.path().join(".claude/skills/my-plugin-my-skill");
        assert!(symlink.symlink_metadata().is_ok());
        assert!(symlink.is_dir());
    }

    #[test]
    fn hoist_plugin_skips_skill_dir_without_skill_md() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(plugin.path().join("skills/bare-dir")).unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        assert!(!workspace.path().join(".claude/skills/my-plugin-bare-dir").exists());
    }

    #[test]
    fn hoist_plugin_hoists_agents() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let agents_dir = plugin.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        fs::write(agents_dir.join("reviewer.md"), "# Agent").unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let symlink = workspace.path().join(".claude/agents/my-plugin-reviewer.md");
        assert!(symlink.symlink_metadata().is_ok());
        assert!(symlink.is_file());
    }

    #[test]
    fn hoist_plugin_hoists_commands() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let cmds_dir = plugin.path().join("commands");
        fs::create_dir_all(&cmds_dir).unwrap();
        fs::write(cmds_dir.join("deploy.md"), "# Command").unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let symlink = workspace.path().join(".claude/commands/my-plugin-deploy.md");
        assert!(symlink.symlink_metadata().is_ok());
        assert!(symlink.is_file());
    }

    #[test]
    fn hoist_plugin_skips_non_md_in_agents() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(plugin.path().join("agents")).unwrap();
        fs::write(plugin.path().join("agents/config.json"), "{}").unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let dst = workspace.path().join(".claude/agents");
        assert!(!dst.exists() || fs::read_dir(&dst).unwrap().count() == 0);
    }

    #[test]
    fn hoist_plugin_skips_non_md_in_commands() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(plugin.path().join("commands")).unwrap();
        fs::write(plugin.path().join("commands/script.sh"), "#!/bin/sh").unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let dst = workspace.path().join(".claude/commands");
        assert!(!dst.exists() || fs::read_dir(&dst).unwrap().count() == 0);
    }

    #[test]
    fn hoist_merges_hooks_into_settings_json() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(plugin.path().join("hooks")).unwrap();
        let hooks = serde_json::json!({
            "hooks": {
                "PostToolUse": [{"type": "command", "command": "echo done"}]
            }
        });
        fs::write(
            plugin.path().join("hooks/hooks.json"),
            serde_json::to_string(&hooks).unwrap(),
        )
        .unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let settings_path = workspace.path().join(".claude/settings.json");
        assert!(settings_path.is_file());
        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(!settings["hooks"]["PostToolUse"].as_array().unwrap().is_empty());
    }

    #[test]
    fn hoist_substitutes_plugin_root_in_hooks() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let plugin_abs = plugin.path().canonicalize().unwrap();

        fs::create_dir_all(plugin.path().join("hooks")).unwrap();
        let hooks_raw = r#"{"hooks": {"PostToolUse": ["${CLAUDE_PLUGIN_ROOT}/run.sh"]}}"#;
        fs::write(plugin.path().join("hooks/hooks.json"), hooks_raw).unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let content =
            fs::read_to_string(workspace.path().join(".claude/settings.json")).unwrap();
        assert!(content.contains(plugin_abs.to_str().unwrap()));
        assert!(!content.contains("${CLAUDE_PLUGIN_ROOT}"));
    }

    #[test]
    fn hoist_hooks_not_duplicated_on_second_call() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(plugin.path().join("hooks")).unwrap();
        let hooks = serde_json::json!({
            "hooks": {
                "PostToolUse": [{"type": "command", "command": "echo done"}]
            }
        });
        fs::write(
            plugin.path().join("hooks/hooks.json"),
            serde_json::to_string(&hooks).unwrap(),
        )
        .unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();
        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let content =
            fs::read_to_string(workspace.path().join(".claude/settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn hoist_hooks_remerges_without_duplication_on_force() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(plugin.path().join("hooks")).unwrap();
        let hooks = serde_json::json!({
            "hooks": {
                "PostToolUse": [{"type": "command", "command": "echo done"}]
            }
        });
        fs::write(
            plugin.path().join("hooks/hooks.json"),
            serde_json::to_string(&hooks).unwrap(),
        )
        .unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();
        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), true).unwrap();

        let content =
            fs::read_to_string(workspace.path().join(".claude/settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn hoist_hooks_merges_into_existing_settings() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(workspace.path().join(".claude")).unwrap();
        let existing = serde_json::json!({"someKey": "someValue"});
        fs::write(
            workspace.path().join(".claude/settings.json"),
            serde_json::to_string(&existing).unwrap(),
        )
        .unwrap();

        fs::create_dir_all(plugin.path().join("hooks")).unwrap();
        let hooks = serde_json::json!({
            "hooks": {"PostToolUse": [{"type": "command", "command": "echo hi"}]}
        });
        fs::write(
            plugin.path().join("hooks/hooks.json"),
            serde_json::to_string(&hooks).unwrap(),
        )
        .unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        let content =
            fs::read_to_string(workspace.path().join(".claude/settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["someKey"], "someValue");
        assert!(!settings["hooks"]["PostToolUse"].as_array().unwrap().is_empty());
    }

    #[test]
    fn hoist_hooks_no_op_when_hooks_json_absent() {
        let plugin = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        hoist_plugin_dir("my-plugin", plugin.path(), workspace.path(), false).unwrap();

        assert!(!workspace.path().join(".claude/settings.json").exists());
    }
}
