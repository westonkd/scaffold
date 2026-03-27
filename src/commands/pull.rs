use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::storage::StorageClient;

pub async fn run(name: Option<&str>, verbose: bool) -> Result<()> {
    let client = StorageClient::from_settings().await?;

    match name {
        Some(raw) => {
            let normalized = normalize_name(raw);
            pull_skill(&client, &normalized, verbose).await?;
        }
        None => {
            let names = client.list_skill_names().await?;
            if names.is_empty() {
                println!("No skills found in bucket.");
                return Ok(());
            }
            for skill_name in &names {
                pull_skill(&client, skill_name, verbose).await?;
            }
        }
    }

    Ok(())
}

fn normalize_name(raw: &str) -> String {
    raw.trim().to_lowercase().replace(' ', "-")
}

fn scaffold_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".scaffold"))
}

fn skill_dir(name: &str) -> Result<PathBuf> {
    Ok(scaffold_dir()?.join(name))
}

async fn pull_skill(client: &StorageClient, name: &str, verbose: bool) -> Result<()> {
    let skill_md_key = format!("{}/SKILL.md", name);
    if !client.object_exists(&skill_md_key).await? {
        bail!("Skill '{}' not found in S3.", name);
    }

    let prefix = format!("{}/", name);
    let keys = client.list_objects(&prefix).await?;

    let root = skill_dir(name)?;
    let remote_relative: HashSet<String> = keys
        .iter()
        .map(|k| k.strip_prefix(&prefix).unwrap_or(k).to_string())
        .collect();

    for key in &keys {
        let relative = key.strip_prefix(&prefix).unwrap_or(key);
        let local_path = root.join(relative);

        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory {}", parent.display()))?;
        }

        if verbose {
            eprintln!("[verbose] downloading {}", key);
        }

        let bytes = client.get_object(key).await?;
        fs::write(&local_path, &bytes)
            .context(format!("Failed to write {}", local_path.display()))?;
    }

    delete_orphans(&root, &remote_relative, &root)?;

    println!("Pulled '{}' ({} files).", name, keys.len());
    Ok(())
}

fn delete_orphans(dir: &Path, remote: &HashSet<String>, root: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).context(format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();

        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            delete_orphans(&path, remote, root)?;
            if fs::read_dir(&path).map(|mut d| d.next().is_none()).unwrap_or(false) {
                fs::remove_dir(&path).ok();
            }
        } else if !remote.contains(&relative) {
            fs::remove_file(&path)
                .context(format!("Failed to remove orphan {}", path.display()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::tempdir;

    #[test]
    fn normalize_lowercases_and_hyphenates() {
        assert_eq!(normalize_name("  My Skill  "), "my-skill");
    }

    #[test]
    fn normalize_preserves_hyphens() {
        assert_eq!(normalize_name("already-good"), "already-good");
    }

    #[test]
    fn delete_orphans_removes_local_file_not_in_remote() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::write(root.join("SKILL.md"), "content").unwrap();
        fs::write(root.join("extra.md"), "orphan").unwrap();

        let remote: HashSet<String> = ["SKILL.md".to_string()].into();
        delete_orphans(root, &remote, root).unwrap();

        assert!(root.join("SKILL.md").exists());
        assert!(!root.join("extra.md").exists());
    }

    #[test]
    fn delete_orphans_keeps_files_present_in_remote() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::write(root.join("SKILL.md"), "content").unwrap();

        let remote: HashSet<String> = ["SKILL.md".to_string()].into();
        delete_orphans(root, &remote, root).unwrap();

        assert!(root.join("SKILL.md").exists());
    }

    #[test]
    fn delete_orphans_handles_nested_directories() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("references")).unwrap();
        fs::write(root.join("SKILL.md"), "content").unwrap();
        fs::write(root.join("references/prd.md"), "prd").unwrap();
        fs::write(root.join("references/stale.md"), "stale").unwrap();

        let remote: HashSet<String> =
            ["SKILL.md".to_string(), "references/prd.md".to_string()].into();
        delete_orphans(root, &remote, root).unwrap();

        assert!(root.join("SKILL.md").exists());
        assert!(root.join("references/prd.md").exists());
        assert!(!root.join("references/stale.md").exists());
    }

    #[test]
    fn delete_orphans_removes_empty_directories_after_cleanup() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("empty-dir")).unwrap();
        fs::write(root.join("SKILL.md"), "content").unwrap();

        let remote: HashSet<String> = ["SKILL.md".to_string()].into();
        delete_orphans(root, &remote, root).unwrap();

        assert!(!root.join("empty-dir").exists());
    }
}
