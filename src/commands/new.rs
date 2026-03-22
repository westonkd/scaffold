use anyhow::{bail, Context, Result};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use crate::s3::S3Client;

pub async fn run(raw_name: &str, description: &str, minimal: bool, verbose: bool) -> Result<()> {
    let name = normalize_name(raw_name);
    validate_name(&name)?;

    let skill_root = skill_dir(&name)?;
    if skill_root.exists() {
        bail!("A skill named '{}' already exists locally at {}", name, skill_root.display());
    }

    let client = S3Client::from_settings().await?;
    if verbose {
        eprintln!("[verbose] bucket: {}", client.bucket);
        eprintln!("[verbose] region: {}", client.region);
    }

    let skill_md_key = format!("{}/SKILL.md", name);
    if client.object_exists(&skill_md_key).await? {
        bail!("A skill named '{}' already exists.", name);
    }

    create_local(&skill_root, &name, description, minimal)?;
    push_to_s3(&client, &skill_root, &name, description, verbose).await?;

    println!("Created skill '{}' at {}", name, skill_root.display());
    Ok(())
}

fn normalize_name(raw: &str) -> String {
    raw.trim().to_lowercase().replace(' ', "-")
}

fn validate_name(name: &str) -> Result<()> {
    let valid = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
        && name.starts_with(|c: char| c.is_ascii_alphanumeric());

    if !valid || name.is_empty() {
        bail!(
            "Invalid skill name '{}'. Names must match [a-z0-9][a-z0-9-]* after normalization.",
            name
        );
    }
    Ok(())
}

fn skill_dir(name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".scaffold").join(name))
}

fn create_local(root: &Path, name: &str, description: &str, minimal: bool) -> Result<()> {
    fs::create_dir_all(root).context("Failed to create skill directory")?;

    let skill_md = skill_md_content(name, description);
    fs::write(root.join("SKILL.md"), skill_md).context("Failed to write SKILL.md")?;

    if !minimal {
        let refs = root.join("references");
        let adrs = refs.join("adrs");
        fs::create_dir_all(&adrs).context("Failed to create references/adrs directory")?;

        fs::write(refs.join("prd.md"), "# PRD\n").context("Failed to write references/prd.md")?;
        fs::write(refs.join("tech-plan.md"), "# Tech Plan\n")
            .context("Failed to write references/tech-plan.md")?;
        fs::write(adrs.join(".gitkeep"), "")
            .context("Failed to write references/adrs/.gitkeep")?;
    }

    Ok(())
}

fn skill_md_content(name: &str, description: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: >\n  {description}\nmetadata:\n  type: project\n  scope:\n  status: active\n  depends-on: platform\n  tags:\n---\n"
    )
}

async fn push_to_s3(client: &S3Client, root: &Path, name: &str, description: &str, verbose: bool) -> Result<()> {
    let skill_md_key = format!("{}/SKILL.md", name);
    if verbose {
        eprintln!("[verbose] uploading {}", skill_md_key);
    }
    upload_file(
        client,
        &root.join("SKILL.md"),
        &skill_md_key,
        Some(&skill_tags(description, "")),
    )
    .await?;

    let refs = root.join("references");
    if refs.exists() {
        upload_dir(client, &refs, &format!("{}/references", name), verbose).await?;
    }

    Ok(())
}

fn skill_tags(description: &str, tags: &str) -> String {
    let desc_encoded = urlencoding_simple(description);
    let tags_encoded = urlencoding_simple(tags);
    format!("Description={}&Tags={}", desc_encoded, tags_encoded)
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | ' ') {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}

fn upload_dir<'a>(
    client: &'a S3Client,
    dir: &'a Path,
    prefix: &'a str,
    verbose: bool,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        for entry in
            fs::read_dir(dir).context(format!("Failed to read directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let child_prefix = format!("{}/{}", prefix, file_name.to_string_lossy());

            if path.is_dir() {
                upload_dir(client, &path, &child_prefix, verbose).await?;
            } else {
                if verbose {
                    eprintln!("[verbose] uploading {}", child_prefix);
                }
                upload_file(client, &path, &child_prefix, None).await?;
            }
        }
        Ok(())
    })
}

async fn upload_file(client: &S3Client, path: &Path, key: &str, tags: Option<&str>) -> Result<()> {
    let body = fs::read(path).context(format!("Failed to read {}", path.display()))?;
    let content_type = if path.extension().and_then(|e| e.to_str()) == Some("md") {
        "text/markdown"
    } else {
        "application/octet-stream"
    };
    client.put_object(key, body, content_type, tags).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn normalize_trims_and_lowercases() {
        assert_eq!(normalize_name("  My Payments Platform  "), "my-payments-platform");
    }

    #[test]
    fn normalize_handles_hyphens() {
        assert_eq!(normalize_name("already-hyphenated"), "already-hyphenated");
    }

    #[test]
    fn normalize_single_word() {
        assert_eq!(normalize_name("Payments"), "payments");
    }

    #[test]
    fn normalize_multiple_spaces_become_multiple_hyphens() {
        assert_eq!(normalize_name("a b c"), "a-b-c");
    }

    #[test]
    fn validate_rejects_leading_hyphen() {
        assert!(validate_name("-bad").is_err());
    }

    #[test]
    fn validate_rejects_special_chars() {
        assert!(validate_name("bad!name").is_err());
    }

    #[test]
    fn validate_rejects_empty_string() {
        assert!(validate_name("").is_err());
    }

    #[test]
    fn validate_rejects_spaces() {
        assert!(validate_name("my skill").is_err());
    }

    #[test]
    fn validate_accepts_valid_name() {
        assert!(validate_name("my-payments-platform").is_ok());
        assert!(validate_name("payments").is_ok());
        assert!(validate_name("p1").is_ok());
    }

    #[test]
    fn validate_accepts_digit_start() {
        assert!(validate_name("1st-project").is_ok());
    }

    #[test]
    fn validate_accepts_all_digits() {
        assert!(validate_name("123").is_ok());
    }

    #[test]
    fn skill_md_contains_name_and_description() {
        let content = skill_md_content("payments", "Handles payment processing");
        assert!(content.contains("name: payments"));
        assert!(content.contains("Handles payment processing"));
    }

    #[test]
    fn skill_md_has_all_required_frontmatter_fields() {
        let content = skill_md_content("my-skill", "A description");
        assert!(content.contains("type: project"));
        assert!(content.contains("status: active"));
        assert!(content.contains("depends-on: platform"));
        assert!(content.contains("scope:"));
        assert!(content.contains("tags:"));
    }

    #[test]
    fn skill_md_empty_description_produces_valid_yaml() {
        let content = skill_md_content("my-skill", "");
        assert!(content.contains("name: my-skill"));
        assert!(content.starts_with("---\n"));
        assert!(content.contains("\n---\n"));
    }

    #[test]
    fn create_local_full_structure() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("test-skill");
        create_local(&root, "test-skill", "A test skill", false).unwrap();

        assert!(root.join("SKILL.md").exists());
        assert!(root.join("references/prd.md").exists());
        assert!(root.join("references/tech-plan.md").exists());
        assert!(root.join("references/adrs/.gitkeep").exists());
    }

    #[test]
    fn create_local_full_reference_files_have_content() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("test-skill");
        create_local(&root, "test-skill", "", false).unwrap();

        let prd = fs::read_to_string(root.join("references/prd.md")).unwrap();
        let tech = fs::read_to_string(root.join("references/tech-plan.md")).unwrap();
        assert!(!prd.is_empty());
        assert!(!tech.is_empty());
    }

    #[test]
    fn create_local_minimal_structure() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("test-skill");
        create_local(&root, "test-skill", "", true).unwrap();

        assert!(root.join("SKILL.md").exists());
        assert!(!root.join("references").exists());
    }

    #[test]
    fn create_local_skill_md_has_correct_name() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("my-skill");
        create_local(&root, "my-skill", "desc", false).unwrap();

        let content = fs::read_to_string(root.join("SKILL.md")).unwrap();
        assert!(content.contains("name: my-skill"));
        assert!(content.contains("desc"));
    }

    #[test]
    fn skill_tags_encodes_description_and_tags() {
        let tags = skill_tags("My description", "payments,billing");
        assert!(tags.starts_with("Description="));
        assert!(tags.contains("&Tags="));
        assert!(tags.contains("payments"));
    }

    #[test]
    fn skill_tags_empty_values_produce_valid_format() {
        let tags = skill_tags("", "");
        assert_eq!(tags, "Description=&Tags=");
    }

    #[test]
    fn urlencoding_passes_through_alphanumeric() {
        assert_eq!(urlencoding_simple("abc123"), "abc123");
        assert_eq!(urlencoding_simple("hello-world"), "hello-world");
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        let result = urlencoding_simple("a&b=c");
        assert!(result.contains("%26"));
        assert!(result.contains("%3D"));
    }

    #[test]
    fn urlencoding_passes_spaces_through() {
        let result = urlencoding_simple("hello world");
        assert!(result.contains(' '));
    }
}
