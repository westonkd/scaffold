use anyhow::{Context, Result};
use std::path::Path;

use crate::hoist::HoistStrategy;

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

            if path.is_dir() && path.join("SKILL.md").is_file() {
                // Directory-based skill: copy the whole directory as {repo_name}-{skill_dir}/
                let dst_skill_dir = dst_dir.join(format!("{}-{}", repo_name, entry_name));

                if dst_skill_dir.exists() && !force {
                    eprintln!(
                        "warning: skipping '{}' — destination already exists: {}",
                        entry_name,
                        dst_skill_dir.display()
                    );
                    continue;
                }

                copy_dir_all(&path, &dst_skill_dir).with_context(|| {
                    format!(
                        "copying {} to {}",
                        path.display(),
                        dst_skill_dir.display()
                    )
                })?;
            } else if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("md")
            {
                // Flat .md skill file
                let dst_path = dst_dir.join(format!("{}-{}", repo_name, entry_name));

                if dst_path.exists() && !force {
                    eprintln!(
                        "warning: skipping '{}' — destination already exists: {}",
                        entry_name,
                        dst_path.display()
                    );
                    continue;
                }

                std::fs::copy(&path, &dst_path).with_context(|| {
                    format!("copying {} to {}", path.display(), dst_path.display())
                })?;
            }
        }

        Ok(())
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)
        .with_context(|| format!("creating directory: {}", dst.display()))?;
    for entry in
        std::fs::read_dir(src).with_context(|| format!("reading directory: {}", src.display()))?
    {
        let entry = entry.with_context(|| format!("reading entry in {}", src.display()))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).with_context(|| {
                format!("copying {} to {}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}
