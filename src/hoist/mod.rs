pub mod anthropic;

use anyhow::{Context, Result};
use std::path::Path;

pub trait HoistStrategy {
    fn name(&self) -> &str;
    fn detect(&self, repo_root: &Path) -> bool;
    fn hoist(
        &self,
        repo_name: &str,
        repo_root: &Path,
        workspace_root: &Path,
        force: bool,
    ) -> Result<()>;
}

pub fn all_strategies() -> Vec<Box<dyn HoistStrategy>> {
    vec![
        Box::new(anthropic::claude_code::agent_skills::AgentSkillsStrategy),
        Box::new(anthropic::plugin::PluginStrategy),
        Box::new(anthropic::marketplace::MarketplaceStrategy),
    ]
}

pub fn run_all_strategies(
    repo_name: &str,
    repo_root: &Path,
    workspace_root: &Path,
    force: bool,
) -> Result<()> {
    for strategy in all_strategies() {
        if strategy.detect(repo_root) {
            println!("  [{}] hoisting from {}", strategy.name(), repo_name);
            strategy
                .hoist(repo_name, repo_root, workspace_root, force)
                .with_context(|| {
                    format!(
                        "strategy '{}' failed on repo '{}'",
                        strategy.name(),
                        repo_name
                    )
                })?;
        }
    }
    Ok(())
}
