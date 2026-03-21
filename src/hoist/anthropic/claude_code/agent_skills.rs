use anyhow::{Context, Result};
use std::path::Path;

use crate::hoist::HoistStrategy;
use crate::utils::relative_path;

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

            let is_dir_skill = path.is_dir() && path.join("SKILL.md").is_file();
            let is_flat_skill =
                path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md");

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

            if let Ok(relative) = dst_path.strip_prefix(workspace_root) {
                crate::utils::add_to_git_exclude(workspace_root, &relative.to_string_lossy())?;
            } else {
                eprintln!(
                    "warning: could not compute relative path for git exclude: {}",
                    dst_path.display()
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs as unix_fs;
    use tempfile::tempdir;

    fn strategy() -> AgentSkillsStrategy {
        AgentSkillsStrategy
    }

    #[test]
    fn detect_returns_false_no_claude_dir() {
        let dir = tempdir().unwrap();
        assert!(!strategy().detect(dir.path()));
    }

    #[test]
    fn detect_returns_false_empty_skills_dir() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".claude/skills")).unwrap();
        assert!(!strategy().detect(dir.path()));
    }

    #[test]
    fn detect_returns_true_for_flat_md_skill() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join(".claude/skills");
        fs::create_dir_all(&skills).unwrap();
        fs::write(skills.join("my-skill.md"), "# Skill").unwrap();
        assert!(strategy().detect(dir.path()));
    }

    #[test]
    fn detect_returns_true_for_directory_skill() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join(".claude/skills/my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();
        assert!(strategy().detect(dir.path()));
    }

    #[test]
    fn detect_returns_false_for_dir_without_skill_md() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join(".claude/skills/my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        assert!(!strategy().detect(dir.path()));
    }

    #[test]
    fn detect_returns_false_for_non_md_file() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join(".claude/skills");
        fs::create_dir_all(&skills).unwrap();
        fs::write(skills.join("notes.txt"), "not a skill").unwrap();
        assert!(!strategy().detect(dir.path()));
    }

    #[test]
    fn hoist_creates_symlink_for_flat_skill() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("my-skill.md"), "# Skill").unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        let symlink = workspace.path().join(".claude/skills/myrepo-my-skill.md");
        assert!(symlink.symlink_metadata().is_ok(), "symlink should exist");
        assert!(symlink.is_file(), "symlink should resolve to a file");
    }

    #[test]
    fn hoist_creates_symlink_for_directory_skill() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skill_dir = repo.path().join(".claude/skills/my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        let symlink = workspace.path().join(".claude/skills/myrepo-my-skill");
        assert!(symlink.symlink_metadata().is_ok(), "symlink should exist");
        assert!(symlink.is_dir(), "symlink should resolve to a directory");
    }

    #[test]
    fn hoist_prefixes_with_repo_name() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("payments.md"), "# Skill").unwrap();

        strategy().hoist("canvas-lms", repo.path(), workspace.path(), false).unwrap();

        let symlink = workspace.path().join(".claude/skills/canvas-lms-payments.md");
        assert!(symlink.symlink_metadata().is_ok());
    }

    #[test]
    fn hoist_skips_existing_destination_without_force() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("my-skill.md"), "# New").unwrap();

        let dst_dir = workspace.path().join(".claude/skills");
        fs::create_dir_all(&dst_dir).unwrap();
        fs::write(dst_dir.join("myrepo-my-skill.md"), "existing").unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        let dst = dst_dir.join("myrepo-my-skill.md");
        assert!(fs::read_link(&dst).is_err(), "existing file should not be replaced with symlink");
        assert_eq!(fs::read_to_string(&dst).unwrap(), "existing");
    }

    #[test]
    fn hoist_overwrites_existing_with_force() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("my-skill.md"), "# New").unwrap();

        let dst_dir = workspace.path().join(".claude/skills");
        fs::create_dir_all(&dst_dir).unwrap();
        fs::write(dst_dir.join("myrepo-my-skill.md"), "existing").unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), true).unwrap();

        let dst = dst_dir.join("myrepo-my-skill.md");
        assert!(fs::read_link(&dst).is_ok(), "should now be a symlink");
    }

    #[test]
    fn hoist_replaces_dangling_symlink_without_force() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("my-skill.md"), "# Skill").unwrap();

        let dst_dir = workspace.path().join(".claude/skills");
        fs::create_dir_all(&dst_dir).unwrap();
        let dst = dst_dir.join("myrepo-my-skill.md");
        unix_fs::symlink("/nonexistent/path/does-not-exist.md", &dst).unwrap();
        assert!(!dst.exists(), "symlink should be dangling");

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        assert!(dst.exists(), "dangling symlink should be replaced");
        assert!(fs::read_link(&dst).is_ok());
    }

    #[test]
    fn hoist_skips_non_md_and_non_dir_skill_entries() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("notes.txt"), "not a skill").unwrap();
        fs::create_dir_all(skills_src.join("empty-dir")).unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        let dst_dir = workspace.path().join(".claude/skills");
        assert!(!dst_dir.join("myrepo-notes.txt").exists());
        assert!(!dst_dir.join("myrepo-empty-dir").exists());
    }

    #[test]
    fn hoist_adds_entry_to_git_exclude() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        fs::create_dir_all(workspace.path().join(".git/info")).unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("my-skill.md"), "# Skill").unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        let exclude = fs::read_to_string(workspace.path().join(".git/info/exclude")).unwrap();
        assert!(exclude.contains(".claude/skills/myrepo-my-skill.md"));
    }

    #[test]
    fn hoist_creates_destination_skills_dir() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("skill.md"), "# Skill").unwrap();

        strategy().hoist("repo", repo.path(), workspace.path(), false).unwrap();

        assert!(workspace.path().join(".claude/skills").is_dir());
    }

    #[test]
    fn hoist_multiple_skills_creates_multiple_symlinks() {
        let repo = tempdir().unwrap();
        let workspace = tempdir().unwrap();

        let skills_src = repo.path().join(".claude/skills");
        fs::create_dir_all(&skills_src).unwrap();
        fs::write(skills_src.join("alpha.md"), "# Alpha").unwrap();
        fs::write(skills_src.join("beta.md"), "# Beta").unwrap();

        strategy().hoist("myrepo", repo.path(), workspace.path(), false).unwrap();

        let dst = workspace.path().join(".claude/skills");
        assert!(dst.join("myrepo-alpha.md").symlink_metadata().is_ok());
        assert!(dst.join("myrepo-beta.md").symlink_metadata().is_ok());
    }
}
