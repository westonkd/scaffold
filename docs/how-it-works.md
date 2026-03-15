# How it Works

## Overview

`hoist` symlinks AI agent artifacts from individual directories up to cwd, namespaced by source directory name. No files are copied — symlinks mean edits in the source are immediately visible, and relative path references within a skill directory remain valid because the OS resolves the symlink to the real path before following relative references.

## Hoist Strategy System

Each hoist strategy is responsible for one type of artifact. Strategies are registered in `hoist::all_strategies()` and run against every root.

**Detection**: A strategy scans the source directory for specific file patterns. If nothing matches, the strategy is silently skipped for that root — no output, no error.

**Namespacing**: Hoisted entries are renamed with the source directory as a prefix (e.g. `canvas-lms-run-specs`) to avoid collisions when multiple roots define entries with the same name.

**Skip behavior**: If a destination path already exists, hoist prints a warning to stderr and skips that entry. Re-run with the destination removed to refresh a symlink.

**Current strategies**

| Strategy | Detects | Symlinks to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/` — flat `.md` files or directories containing `SKILL.md` | `<cwd>/.claude/skills/<root>-<entry>` |

## Symlinks

All symlinks are relative, computed by `utils::relative_path()`. This means the directory structure is fully portable — move or copy the whole tree anywhere and the links remain valid.

For example, hoisting from `canvas-lms/gems/plugins/my-plugin` into `canvas-lms/` produces a symlink like:

```
canvas-lms/.claude/skills/my-plugin-run-specs
  → ../../gems/plugins/my-plugin/.claude/skills/run-specs
```

## Adding a Strategy

Implement the `HoistStrategy` trait in a new file under `src/hoist/<vendor>/<agent>/`:

```rust
pub trait HoistStrategy {
    fn name(&self) -> &str;
    fn detect(&self, repo_root: &Path) -> bool;
    fn hoist(&self, repo_name: &str, repo_root: &Path, workspace_root: &Path, force: bool) -> Result<()>;
}
```

Then register it in `hoist::all_strategies()` in `src/hoist/mod.rs`.
