use anyhow::{bail, Context, Result};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use crate::utils::{add_to_git_exclude, relative_path};

pub fn run(name: Option<&str>, force: bool) -> Result<()> {
    let cwd = std::env::current_dir().context("Could not determine current directory")?;

    let names: Vec<String> = match name {
        Some(raw) => vec![normalize_name(raw)],
        None => {
            let artifacts_path = cwd.join(".scaffold-artifacts");
            if !artifacts_path.exists() {
                bail!("No .scaffold-artifacts found. Run scaffold init to configure this repository.");
            }
            let content = std::fs::read_to_string(&artifacts_path)
                .context("Failed to read .scaffold-artifacts")?;
            parse_scopes(&content)
        }
    };

    let mut queue: VecDeque<String> = names.into();
    let mut seen: HashSet<String> = HashSet::new();

    while let Some(skill_name) = queue.pop_front() {
        if seen.contains(&skill_name) {
            continue;
        }
        seen.insert(skill_name.clone());

        let skill_src = scaffold_skill_dir(&skill_name)?;
        if !skill_src.exists() {
            bail!(
                "Skill '{}' not found in ~/.scaffold/. Run: scaffold pull {}",
                skill_name,
                skill_name
            );
        }
        let skill_md = skill_src.join("SKILL.md");
        if !skill_md.exists() {
            bail!(
                "Skill '{}' is missing SKILL.md. Run: scaffold pull {}",
                skill_name,
                skill_name
            );
        }

        if let Some(dep) = read_depends_on(&skill_md)? {
            let dep_name = normalize_name(&dep);
            if !seen.contains(&dep_name) {
                queue.push_back(dep_name);
            }
        }

        link_skill(&skill_name, &skill_src, &cwd, force)?;
    }

    Ok(())
}

fn link_skill(name: &str, skill_src: &std::path::Path, cwd: &std::path::Path, force: bool) -> Result<()> {
    let dst_dir = cwd.join(".claude").join("skills");
    std::fs::create_dir_all(&dst_dir)
        .with_context(|| format!("creating {}", dst_dir.display()))?;

    let dst_path = dst_dir.join(name);

    let dangling = dst_path.symlink_metadata().is_ok() && !dst_path.exists();
    let exists = dst_path.exists();

    if dangling {
        std::fs::remove_file(&dst_path)
            .with_context(|| format!("removing dangling symlink: {}", dst_path.display()))?;
    } else if exists {
        if !force {
            eprintln!(
                "warning: skipping '{}' — already linked. Use --force to replace.",
                name
            );
            return Ok(());
        }
        if dst_path.is_dir() {
            std::fs::remove_dir_all(&dst_path)
                .with_context(|| format!("removing {}", dst_path.display()))?;
        } else {
            std::fs::remove_file(&dst_path)
                .with_context(|| format!("removing {}", dst_path.display()))?;
        }
    }

    let rel_target = relative_path(&dst_dir, skill_src);
    std::os::unix::fs::symlink(&rel_target, &dst_path).with_context(|| {
        format!(
            "creating symlink {} -> {}",
            dst_path.display(),
            rel_target.display()
        )
    })?;

    add_to_git_exclude(cwd, &format!(".claude/skills/{}", name))?;

    println!("Linked {} → {}", name, dst_path.display());
    Ok(())
}

fn normalize_name(raw: &str) -> String {
    raw.trim().to_lowercase().replace(' ', "-")
}

fn scaffold_skill_dir(name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".scaffold").join(name))
}

fn parse_scopes(content: &str) -> Vec<String> {
    let mut in_scopes = false;
    let mut scopes = vec![];

    for line in content.lines() {
        if line.trim_start() == "scopes:" || line.trim() == "scopes:" {
            in_scopes = true;
            continue;
        }
        if in_scopes {
            if line.starts_with(' ') || line.starts_with('\t') {
                let trimmed = line.trim();
                if let Some(val) = trimmed.strip_prefix("- ") {
                    scopes.push(normalize_name(val));
                }
            } else {
                break;
            }
        }
    }

    scopes
}

fn read_depends_on(skill_md: &std::path::Path) -> Result<Option<String>> {
    let content = std::fs::read_to_string(skill_md)
        .with_context(|| format!("reading {}", skill_md.display()))?;

    let mut in_metadata = false;
    for line in content.lines() {
        if line.trim() == "---" {
            continue;
        }
        if line.starts_with("metadata:") {
            in_metadata = true;
        } else if in_metadata {
            if !line.starts_with(' ') && !line.starts_with('\t') {
                break;
            }
            let trimmed = line.trim();
            if let Some(val) = trimmed.strip_prefix("depends-on:") {
                let dep = val.trim().to_string();
                if !dep.is_empty() {
                    return Ok(Some(dep));
                }
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn normalize_lowercases_and_hyphenates() {
        assert_eq!(normalize_name("My Payments Platform"), "my-payments-platform");
        assert_eq!(normalize_name("  payments  "), "payments");
    }

    #[test]
    fn parse_scopes_extracts_list() {
        let content = "scopes:\n  - payments\n  - platform\n";
        assert_eq!(parse_scopes(content), vec!["payments", "platform"]);
    }

    #[test]
    fn parse_scopes_empty_file() {
        assert_eq!(parse_scopes(""), Vec::<String>::new());
    }

    #[test]
    fn parse_scopes_no_scopes_key() {
        assert_eq!(parse_scopes("other: value\n"), Vec::<String>::new());
    }

    #[test]
    fn read_depends_on_returns_dep() {
        let dir = tempdir().unwrap();
        let skill_md = dir.path().join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: payments\nmetadata:\n  type: project\n  depends-on: platform\n---\n",
        )
        .unwrap();
        assert_eq!(read_depends_on(&skill_md).unwrap(), Some("platform".to_string()));
    }

    #[test]
    fn read_depends_on_returns_none_when_absent() {
        let dir = tempdir().unwrap();
        let skill_md = dir.path().join("SKILL.md");
        fs::write(&skill_md, "---\nname: payments\nmetadata:\n  type: project\n---\n").unwrap();
        assert_eq!(read_depends_on(&skill_md).unwrap(), None);
    }

    #[test]
    fn link_skill_creates_symlink() {
        let src = tempdir().unwrap();
        let cwd = tempdir().unwrap();

        fs::write(src.path().join("SKILL.md"), "# Skill").unwrap();

        link_skill("payments", src.path(), cwd.path(), false).unwrap();

        let symlink = cwd.path().join(".claude/skills/payments");
        assert!(symlink.symlink_metadata().is_ok());
        assert!(symlink.is_dir());
    }

    #[test]
    fn link_skill_idempotent_without_force() {
        let src = tempdir().unwrap();
        let cwd = tempdir().unwrap();

        fs::write(src.path().join("SKILL.md"), "# Skill").unwrap();

        link_skill("payments", src.path(), cwd.path(), false).unwrap();
        link_skill("payments", src.path(), cwd.path(), false).unwrap();

        let symlink = cwd.path().join(".claude/skills/payments");
        assert!(symlink.symlink_metadata().is_ok());
    }

    #[test]
    fn link_skill_force_replaces_existing() {
        let src = tempdir().unwrap();
        let cwd = tempdir().unwrap();

        fs::write(src.path().join("SKILL.md"), "# Skill").unwrap();

        let dst_dir = cwd.path().join(".claude/skills");
        fs::create_dir_all(&dst_dir).unwrap();
        fs::write(dst_dir.join("payments"), "existing file").unwrap();

        link_skill("payments", src.path(), cwd.path(), true).unwrap();

        let symlink = cwd.path().join(".claude/skills/payments");
        assert!(fs::read_link(&symlink).is_ok());
    }

    #[test]
    fn link_skill_replaces_dangling_symlink() {
        let src = tempdir().unwrap();
        let cwd = tempdir().unwrap();

        fs::write(src.path().join("SKILL.md"), "# Skill").unwrap();

        let dst_dir = cwd.path().join(".claude/skills");
        fs::create_dir_all(&dst_dir).unwrap();
        std::os::unix::fs::symlink("/nonexistent/path", dst_dir.join("payments")).unwrap();

        link_skill("payments", src.path(), cwd.path(), false).unwrap();

        let symlink = cwd.path().join(".claude/skills/payments");
        assert!(symlink.exists());
    }

    #[test]
    fn link_skill_adds_git_exclude_entry() {
        let src = tempdir().unwrap();
        let cwd = tempdir().unwrap();

        fs::write(src.path().join("SKILL.md"), "# Skill").unwrap();
        fs::create_dir_all(cwd.path().join(".git/info")).unwrap();

        link_skill("payments", src.path(), cwd.path(), false).unwrap();

        let exclude =
            fs::read_to_string(cwd.path().join(".git/info/exclude")).unwrap();
        assert!(exclude.contains(".claude/skills/payments"));
    }
}
