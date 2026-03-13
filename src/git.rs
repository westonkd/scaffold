use std::path::Path;
use anyhow::{Context, Result};

fn git(args: &[&str], cwd: Option<&Path>) -> Result<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let status = cmd
        .status()
        .with_context(|| format!("failed to run: git {}", args.join(" ")))?;
    if !status.success() {
        anyhow::bail!("git {} failed with status {}", args.join(" "), status);
    }
    Ok(())
}

pub fn clone(source: &str, dest: &Path) -> Result<()> {
    git(
        &["clone", source, dest.to_str().unwrap()],
        None,
    )
    .with_context(|| format!("cloning {} to {}", source, dest.display()))
}

pub fn clone_local(source: &Path, dest: &Path) -> Result<()> {
    git(
        &["clone", "--local", source.to_str().unwrap(), dest.to_str().unwrap()],
        None,
    )
    .with_context(|| format!("local-cloning {} to {}", source.display(), dest.display()))
}

pub fn checkout(repo: &Path, ref_: &str) -> Result<()> {
    git(&["checkout", ref_], Some(repo))
        .with_context(|| format!("checking out {} in {}", ref_, repo.display()))
}

pub fn fetch(repo: &Path) -> Result<()> {
    git(&["fetch", "origin"], Some(repo))
        .with_context(|| format!("fetching in {}", repo.display()))
}

pub fn pull(repo: &Path) -> Result<()> {
    git(&["pull"], Some(repo))
        .with_context(|| format!("pulling in {}", repo.display()))
}
