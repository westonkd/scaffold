use std::path::Path;
use anyhow::{Context, Result};

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
                    e.path().is_file()
                        && e.path().extension().and_then(|s| s.to_str()) == Some("md")
                })
            })
            .unwrap_or(false)
    }

    fn hoist(&self, repo_name: &str, repo_root: &Path, workspace_root: &Path) -> Result<()> {
        let src_dir = repo_root.join(".claude").join("skills");
        let dst_dir = workspace_root.join(".claude").join("skills");

        std::fs::create_dir_all(&dst_dir)
            .with_context(|| format!("creating destination dir: {}", dst_dir.display()))?;

        let entries = std::fs::read_dir(&src_dir)
            .with_context(|| format!("reading skills dir: {}", src_dir.display()))?;

        for entry in entries {
            let entry = entry.with_context(|| format!("reading entry in {}", src_dir.display()))?;
            let path = entry.path();

            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .with_context(|| format!("getting filename for {}", path.display()))?;

            let dst_filename = format!("{}-{}", repo_name, filename);
            let dst_path = dst_dir.join(&dst_filename);

            if dst_path.exists() {
                eprintln!(
                    "warning: skipping '{}' — destination already exists: {}",
                    filename,
                    dst_path.display()
                );
                continue;
            }

            std::fs::copy(&path, &dst_path).with_context(|| {
                format!("copying {} to {}", path.display(), dst_path.display())
            })?;
        }

        Ok(())
    }
}
