use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Append `pattern` to `<workspace_root>/.git/info/exclude` if it is not already present.
///
/// Silently does nothing when `workspace_root` does not contain a `.git` directory (not a git
/// repo) so callers do not need to guard against non-git workspaces.
///
/// When `verbose` is true, prints what action was taken (or skipped and why).
pub fn add_to_git_exclude(workspace_root: &Path, pattern: &str, verbose: bool) -> Result<()> {
    let git_dir = workspace_root.join(".git");
    if !git_dir.is_dir() {
        if verbose {
            eprintln!(
                "  [verbose] git exclude: skipped '{}' — no .git directory at {}",
                pattern,
                workspace_root.display()
            );
        }
        return Ok(());
    }

    let info_dir = git_dir.join("info");
    std::fs::create_dir_all(&info_dir)
        .with_context(|| format!("creating {}", info_dir.display()))?;

    let exclude_path = info_dir.join("exclude");

    let existing = if exclude_path.is_file() {
        std::fs::read_to_string(&exclude_path)
            .with_context(|| format!("reading {}", exclude_path.display()))?
    } else {
        String::new()
    };

    if existing.lines().any(|line| line == pattern) {
        if verbose {
            eprintln!(
                "  [verbose] git exclude: '{}' already present in {}",
                pattern,
                exclude_path.display()
            );
        }
        return Ok(());
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(pattern);
    content.push('\n');

    std::fs::write(&exclude_path, content)
        .with_context(|| format!("writing {}", exclude_path.display()))?;

    if verbose {
        eprintln!(
            "  [verbose] git exclude: added '{}' to {}",
            pattern,
            exclude_path.display()
        );
    }

    Ok(())
}

/// Compute a relative path from `from_dir` to `to`, both absolute.
///
/// Strips the common prefix, emits one `../` per remaining component in
/// `from_dir`, then appends the remaining suffix of `to`.
pub fn relative_path(from_dir: &std::path::Path, to: &std::path::Path) -> PathBuf {
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let up_count = from_components.len() - common_len;
    let mut result = PathBuf::new();
    for _ in 0..up_count {
        result.push("..");
    }
    for component in &to_components[common_len..] {
        result.push(component);
    }
    result
}

/// Normalize a path by resolving `.` and `..` components without touching the filesystem.
/// Unlike `canonicalize`, this works on non-existent paths (e.g. broken symlink targets).
pub fn normalize_path(path: &std::path::Path) -> PathBuf {
    use std::path::Component;
    let mut components: Vec<Component> = vec![];
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Only pop if we have a non-root component to pop
                if matches!(components.last(), Some(Component::Normal(_))) {
                    components.pop();
                }
            }
            Component::CurDir => {}
            c => components.push(c),
        }
    }
    components.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_relative_path_sibling() {
        let from = Path::new("/a/repos/canvas-lms/gems/plugins");
        let to = Path::new("/a/repos/mra");
        let rel = relative_path(from, to);
        assert_eq!(rel, PathBuf::from("../../../mra"));
    }

    #[test]
    fn test_relative_path_same_level() {
        let from = Path::new("/a/b");
        let to = Path::new("/a/c");
        let rel = relative_path(from, to);
        assert_eq!(rel, PathBuf::from("../c"));
    }

    #[test]
    fn test_add_to_git_exclude_appends_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        std::fs::create_dir_all(workspace.join(".git").join("info")).unwrap();

        add_to_git_exclude(workspace, ".claude/skills/repo-skill", false).unwrap();

        let content =
            std::fs::read_to_string(workspace.join(".git").join("info").join("exclude")).unwrap();
        assert!(content.contains(".claude/skills/repo-skill"));
    }

    #[test]
    fn test_add_to_git_exclude_no_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        std::fs::create_dir_all(workspace.join(".git").join("info")).unwrap();

        add_to_git_exclude(workspace, ".claude/skills/repo-skill", false).unwrap();
        add_to_git_exclude(workspace, ".claude/skills/repo-skill", false).unwrap();

        let content =
            std::fs::read_to_string(workspace.join(".git").join("info").join("exclude")).unwrap();
        assert_eq!(
            content
                .lines()
                .filter(|l| *l == ".claude/skills/repo-skill")
                .count(),
            1
        );
    }

    #[test]
    fn test_add_to_git_exclude_no_git_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        // No .git directory — should succeed silently
        add_to_git_exclude(workspace, ".claude/skills/repo-skill", false).unwrap();
        assert!(!workspace.join(".git").join("info").join("exclude").exists());
    }

    #[test]
    fn test_add_to_git_exclude_creates_info_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let workspace = tmp.path();
        // .git exists but .git/info does not
        std::fs::create_dir_all(workspace.join(".git")).unwrap();

        add_to_git_exclude(workspace, ".claude/skills/repo-skill", false).unwrap();

        assert!(workspace.join(".git").join("info").join("exclude").exists());
    }
}
