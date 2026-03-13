use anyhow::{Context, Result};
use std::path::Path;

use crate::hoist::HoistStrategy;
use crate::store::relative_path;

pub struct AgentSkillsStrategy;

impl HoistStrategy for AgentSkillsStrategy {
    fn name(&self) -> &str {
        "anthropic/claude_code/agent_skills"
    }

    fn detect(&self, repo_root: &Path) -> bool {
        let skills_dir = repo_root.join(".claude").join("skills");
        if !skills_dir.is_dir() {
            return false;
        }
        std::fs::read_dir(&skills_dir)
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    let path = e.path();
                    // Flat .md file directly in skills/
                    (path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md"))
                        // Directory-based skill containing a SKILL.md
                        || (path.is_dir() && path.join("SKILL.md").is_file())
                })
            })
            .unwrap_or(false)
    }

    fn hoist(
        &self,
        repo_name: &str,
        repo_root: &Path,
        workspace_root: &Path,
        force: bool,
    ) -> Result<()> {
        let src_dir = repo_root.join(".claude").join("skills");
        let dst_dir = workspace_root.join(".claude").join("skills");

        std::fs::create_dir_all(&dst_dir)
            .with_context(|| format!("creating destination dir: {}", dst_dir.display()))?;

        let entries = std::fs::read_dir(&src_dir)
            .with_context(|| format!("reading skills dir: {}", src_dir.display()))?;

        for entry in entries {
            let entry = entry.with_context(|| format!("reading entry in {}", src_dir.display()))?;
            let path = entry.path();
            let entry_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .with_context(|| format!("getting name for {}", path.display()))?
                .to_string();

            let is_dir_skill =
                path.is_dir() && path.join("SKILL.md").is_file();
            let is_flat_skill = path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("md");

            if !is_dir_skill && !is_flat_skill {
                continue;
            }

            let dst_path = dst_dir.join(format!("{}-{}", repo_name, entry_name));

            if dst_path.exists() && !force {
                eprintln!(
                    "warning: skipping '{}' — destination already exists: {}",
                    entry_name,
                    dst_path.display()
                );
                continue;
            }

            // Remove existing destination if force is set
            if dst_path.exists() {
                if dst_path.is_dir() {
                    std::fs::remove_dir_all(&dst_path).with_context(|| {
                        format!("removing existing destination: {}", dst_path.display())
                    })?;
                } else {
                    std::fs::remove_file(&dst_path).with_context(|| {
                        format!("removing existing destination: {}", dst_path.display())
                    })?;
                }
            }

            // Also remove a dangling symlink (exists() returns false for broken symlinks)
            if dst_path.symlink_metadata().is_ok() {
                std::fs::remove_file(&dst_path).with_context(|| {
                    format!("removing existing symlink: {}", dst_path.display())
                })?;
            }

            let rel_target = relative_path(&dst_dir, &path);
            std::os::unix::fs::symlink(&rel_target, &dst_path).with_context(|| {
                format!(
                    "creating symlink {} -> {}",
                    dst_path.display(),
                    rel_target.display()
                )
            })?;
        }

        Ok(())
    }
}
