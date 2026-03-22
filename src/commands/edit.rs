use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use crate::commands::pull;

const DIRECTORY_AWARE: &[&str] =
    &["code", "cursor", "zed", "windsurf", "nvim", "vim", "emacs"];

pub async fn run(raw_name: &str, verbose: bool) -> Result<()> {
    let name = normalize_name(raw_name);

    pull::run(Some(&name), verbose).await?;

    let skill_root = skill_dir(&name)?;
    let editor = resolve_editor()?;
    let target = path_for_editor(&editor, &skill_root);

    std::process::Command::new(&editor)
        .arg(&target)
        .status()
        .context(format!("Failed to launch editor '{}'", editor))?;

    Ok(())
}

fn normalize_name(raw: &str) -> String {
    raw.trim().to_lowercase().replace(' ', "-")
}

fn skill_dir(name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".scaffold").join(name))
}

fn resolve_editor() -> Result<String> {
    for var in &["VISUAL", "EDITOR"] {
        if let Ok(val) = std::env::var(var) {
            let val = val.trim().to_string();
            if !val.is_empty() {
                return Ok(val);
            }
        }
    }

    let path_var = std::env::var("PATH").unwrap_or_default();
    let candidates = ["code", "cursor", "zed", "windsurf", "nvim", "vim", "vi"];

    for candidate in &candidates {
        if is_on_path(candidate, &path_var) {
            return Ok(candidate.to_string());
        }
    }

    bail!("No editor found. Set the EDITOR environment variable.")
}

fn is_on_path(binary: &str, path_var: &str) -> bool {
    std::env::split_paths(path_var)
        .any(|dir| dir.join(binary).is_file())
}

fn editor_basename(editor: &str) -> &str {
    Path::new(editor).file_name().and_then(|n| n.to_str()).unwrap_or(editor)
}

fn path_for_editor(editor: &str, skill_root: &Path) -> PathBuf {
    let base = editor_basename(editor);
    if DIRECTORY_AWARE.contains(&base) {
        skill_root.to_path_buf()
    } else {
        skill_root.join("SKILL.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn normalize_lowercases_and_hyphenates() {
        assert_eq!(normalize_name("  My Skill  "), "my-skill");
    }

    #[test]
    fn path_for_editor_directory_aware_returns_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        for editor in DIRECTORY_AWARE {
            let result = path_for_editor(editor, &root);
            assert_eq!(result, root, "expected dir for editor '{}'", editor);
        }
    }

    #[test]
    fn path_for_editor_unknown_returns_skill_md() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let result = path_for_editor("nano", &root);
        assert_eq!(result, root.join("SKILL.md"));
    }

    #[test]
    fn path_for_editor_helix_returns_skill_md() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let result = path_for_editor("hx", &root);
        assert_eq!(result, root.join("SKILL.md"));
    }

    #[test]
    fn editor_basename_strips_path() {
        assert_eq!(editor_basename("/usr/bin/nvim"), "nvim");
        assert_eq!(editor_basename("nvim"), "nvim");
        assert_eq!(editor_basename("/opt/homebrew/bin/code"), "code");
    }

    #[test]
    fn path_for_editor_full_path_nvim_opens_dir() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let result = path_for_editor("/usr/bin/nvim", &root);
        assert_eq!(result, root);
    }

    #[test]
    fn path_for_editor_full_path_nano_opens_skill_md() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let result = path_for_editor("/usr/bin/nano", &root);
        assert_eq!(result, root.join("SKILL.md"));
    }
}
