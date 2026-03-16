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
        create_symlink(&path, &dst_path, &dst_dir, force, skill_name)?;
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
        create_symlink(&path, &dst_path, &dst_dir, force, basename)?;
    }

    Ok(())
}

fn create_symlink(src: &Path, dst: &Path, dst_dir: &Path, force: bool, name: &str) -> Result<()> {
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
    if !settings_obj.contains_key("hooks") {
        settings_obj.insert(
            "hooks".to_string(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
    }

    let settings_hooks = settings_obj
        .get_mut("hooks")
        .unwrap()
        .as_object_mut()
        .unwrap();

    // Check if this plugin's hooks are already present
    let already_present = settings_hooks.values().any(|entries| {
        entries
            .as_array()
            .map(|arr| arr.iter().any(|e| e.to_string().contains(&plugin_root_str)))
            .unwrap_or(false)
    });

    if already_present && !force {
        eprintln!(
            "warning: skipping hooks from '{}' — already merged (re-run with --force to re-merge)",
            plugin_name
        );
        return Ok(());
    }

    // With force=true, remove existing entries for this plugin identified by its absolute path
    if force {
        for entries in settings_hooks.values_mut() {
            if let Some(arr) = entries.as_array_mut() {
                arr.retain(|entry| !entry.to_string().contains(&plugin_root_str));
            }
        }
    }

    // Extend each event array with the plugin's hook entries
    for (event, plugin_entries) in &plugin_hooks_map {
        let existing = settings_hooks
            .entry(event.clone())
            .or_insert_with(|| serde_json::Value::Array(vec![]));
        if let (Some(existing_arr), Some(new_arr)) =
            (existing.as_array_mut(), plugin_entries.as_array())
        {
            existing_arr.extend(new_arr.iter().cloned());
        }
    }

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
