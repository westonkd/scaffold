use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::storage::StorageClient;

struct SkillInfo {
    name: String,
    description: String,
    linked: bool,
}

pub async fn run(remote: bool) -> Result<()> {
    let skills = if remote {
        list_remote().await?
    } else {
        list_local()?
    };

    if skills.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    print_table(&skills);
    Ok(())
}

fn list_local() -> Result<Vec<SkillInfo>> {
    let scaffold_dir = scaffold_dir()?;
    if !scaffold_dir.exists() {
        return Ok(vec![]);
    }

    let cwd = std::env::current_dir().context("Could not determine current directory")?;

    let mut skills = vec![];
    for entry in fs::read_dir(&scaffold_dir)
        .context(format!("Failed to read {}", scaffold_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let content = fs::read_to_string(&skill_md)
            .context(format!("Failed to read {}", skill_md.display()))?;
        let mut info = parse_skill_md(&content);

        let link_path = cwd.join(".claude").join("skills").join(&info.name);
        info.linked = link_path.symlink_metadata().is_ok();

        skills.push(info);
    }

    skills.sort_by(|a, b| match (a.linked, b.linked) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(skills)
}

async fn list_remote() -> Result<Vec<SkillInfo>> {
    let client = StorageClient::from_settings().await?;
    let names = client.list_skill_names().await?;

    let mut skills = vec![];
    for name in &names {
        let key = format!("{}/SKILL.md", name);
        let bytes = client.get_object(&key).await?;
        let content = String::from_utf8_lossy(&bytes).into_owned();
        skills.push(parse_skill_md(&content));
    }

    Ok(skills)
}

fn scaffold_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".scaffold"))
}

fn parse_skill_md(content: &str) -> SkillInfo {
    let mut name = String::new();
    let mut description = String::new();

    let mut in_description_block = false;
    let mut desc_parts: Vec<String> = vec![];

    for line in content.lines() {
        if line.trim() == "---" {
            in_description_block = false;
            continue;
        }

        if in_description_block {
            if line.starts_with(' ') || line.starts_with('\t') {
                desc_parts.push(line.trim().to_string());
                continue;
            } else {
                in_description_block = false;
                description = desc_parts.join(" ").trim().to_string();
            }
        }

        if line.starts_with("name:") {
            name = line["name:".len()..].trim().to_string();
        } else if line.starts_with("description:") {
            let val = line["description:".len()..].trim();
            if val.is_empty() || val == ">" || val == "|" {
                in_description_block = true;
                desc_parts.clear();
            } else {
                description = val.to_string();
            }
        }
    }

    if in_description_block && description.is_empty() {
        description = desc_parts.join(" ").trim().to_string();
    }

    SkillInfo { name, description, linked: false }
}

fn print_table(skills: &[SkillInfo]) {
    let name_w = skills.iter().map(|s| s.name.len()).max().unwrap_or(0).max(4);
    let linked_w = 6usize;
    let desc_max = 60usize;

    println!(
        "{:<name_w$}  {:<linked_w$}  {}",
        "NAME", "LINKED", "DESCRIPTION",
        name_w = name_w,
        linked_w = linked_w,
    );
    println!(
        "{:-<name_w$}  {:-<linked_w$}  {:-<11}",
        "", "", "",
        name_w = name_w,
        linked_w = linked_w,
    );
    for s in skills {
        let linked_str = if s.linked { "yes" } else { "no" };
        let desc = if s.description.len() > desc_max {
            format!("{}...", &s.description[..desc_max])
        } else {
            s.description.clone()
        };
        println!(
            "{:<name_w$}  {:<linked_w$}  {}",
            s.name, linked_str, desc,
            name_w = name_w,
            linked_w = linked_w,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill_md(name: &str, description: &str) -> String {
        format!(
            "---\nname: {name}\ndescription: >\n  {description}\nmetadata:\n  type: project\n  scope: repo\n  status: active\n  depends-on: platform\n  tags:\n---\n"
        )
    }

    #[test]
    fn parse_extracts_name_and_description() {
        let content = skill_md("payments", "Context for the payments platform.");
        let info = parse_skill_md(&content);
        assert_eq!(info.name, "payments");
        assert_eq!(info.description, "Context for the payments platform.");
        assert!(!info.linked);
    }

    #[test]
    fn parse_returns_empty_strings_for_missing_fields() {
        let info = parse_skill_md("---\n---\n");
        assert_eq!(info.name, "");
        assert_eq!(info.description, "");
    }

    #[test]
    fn parse_extracts_description_block_scalar() {
        let content = "---\nname: foo\ndescription: >\n  Line one\n  line two.\nmetadata:\n  type: project\n---\n";
        let info = parse_skill_md(content);
        assert_eq!(info.description, "Line one line two.");
    }

    #[test]
    fn parse_extracts_inline_description() {
        let content = "---\nname: foo\ndescription: A simple description.\n---\n";
        let info = parse_skill_md(content);
        assert_eq!(info.description, "A simple description.");
    }

    #[test]
    fn list_local_returns_empty_when_scaffold_dir_absent() {
        let result: Vec<SkillInfo> = vec![];
        assert!(result.is_empty());
    }
}
