use anyhow::{Context, Result};
use std::path::Path;

use super::plugin;
use crate::hoist::HoistStrategy;

pub struct MarketplaceStrategy;

impl HoistStrategy for MarketplaceStrategy {
    fn name(&self) -> &str {
        "anthropic/marketplace"
    }

    fn detect(&self, repo_root: &Path) -> bool {
        repo_root
            .join(".claude-plugin")
            .join("marketplace.json")
            .is_file()
    }

    fn hoist(
        &self,
        repo_name: &str,
        repo_root: &Path,
        workspace_root: &Path,
        force: bool,
    ) -> Result<()> {
        let marketplace_path = repo_root.join(".claude-plugin").join("marketplace.json");
        let raw = std::fs::read_to_string(&marketplace_path)
            .with_context(|| format!("reading marketplace.json: {}", marketplace_path.display()))?;

        let marketplace: MarketplaceJson =
            serde_json::from_str(&raw).with_context(|| "parsing marketplace.json")?;

        for entry in marketplace.plugins {
            let source_str = match entry.source.as_str() {
                Some(s) => s.to_string(),
                None => {
                    // Remote source (object) — skip
                    continue;
                }
            };

            if !source_str.starts_with("./") {
                // Not a relative local path — skip
                continue;
            }

            // Strip the leading "./" and resolve relative to repo_root
            let relative = source_str.trim_start_matches("./");
            let plugin_dir = repo_root.join(relative);

            if !plugin_dir.is_dir() {
                eprintln!(
                    "warning: plugin '{}' source '{}' does not exist, skipping",
                    entry.name, source_str
                );
                continue;
            }

            // Label: "<marketplace-repo-name>-<plugin-name>"
            let plugin_label = format!("{}-{}", repo_name, entry.name);
            println!("    [plugin] hoisting '{}'", plugin_label);

            plugin::hoist_plugin_dir(&plugin_label, &plugin_dir, workspace_root, force)
                .with_context(|| format!("hoisting plugin '{}'", plugin_label))?;
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct MarketplaceJson {
    plugins: Vec<MarketplaceEntry>,
}

#[derive(serde::Deserialize)]
struct MarketplaceEntry {
    name: String,
    source: serde_json::Value,
}
