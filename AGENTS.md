# hoist — Agent Guide

## Project Overview

`hoist` is a Rust CLI tool that symlinks AI agent artifacts (Claude Code skills, plugins, marketplace plugins) from one or more source repositories into a working directory. This lets teams keep agent artifacts in their own repos while making them discoverable from a shared workspace — without committing proprietary context into a shared repo.

Each artifact is namespaced by its source repo name (e.g., `canvas-lms-run-specs`) to prevent collisions across repos.

## Architecture

The core abstraction is the `HoistStrategy` trait (`src/hoist/mod.rs`). Each strategy:

1. **Detects** whether a repo contains the artifacts it handles (`detect`)
2. **Hoists** those artifacts into the workspace by creating relative symlinks (`hoist`)

Strategies are registered in `all_strategies()` and run against every configured root. A repo may trigger multiple strategies.

### Current Strategies

| Strategy name | Detects | Hoists |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/` with `.md` files or `SKILL.md` dirs | Skills → `.claude/skills/<repo>-<name>` |
| `anthropic/plugin` | `.claude-plugin/plugin.json` | Skills, agents, commands, hooks from a single plugin repo |
| `anthropic/marketplace` | `.claude-plugin/marketplace.json` | All locally-sourced plugins listed in the marketplace |

## Key Files

| File | Purpose |
|---|---|
| `src/main.rs` | CLI entry point (clap); routes to `hoist` or `unhoist` subcommands |
| `src/commands/hoist.rs` | Loads `hoist.json` or resolves an explicit path; calls `run_all_strategies` per root |
| `src/commands/unhoist.rs` | `hoist unhoist` — removes symlinks and hook entries by source root; supports explicit and prune modes |
| `src/hoist/mod.rs` | `HoistStrategy` trait definition and `all_strategies()` registry |
| `src/hoist/anthropic/claude_code/agent_skills.rs` | Agent skills strategy |
| `src/hoist/anthropic/plugin.rs` | Plugin strategy; `hoist_plugin_dir` is also called by the marketplace strategy |
| `src/hoist/anthropic/marketplace.rs` | Marketplace strategy; iterates local plugin entries |
| `src/utils.rs` | `relative_path()` — portable symlink paths; `normalize_path()` — resolves `..` without filesystem access (used by unhoist for broken symlinks) |
| `hoist.json` | Example multi-root config (used by `make test` fixture at `spec/anthropic`) |

## Build & Development

All build and test work runs inside Docker containers via the Makefile. **Never run `cargo` directly on the host.**

```sh
make build    # Build the dev Docker image
make dev      # Open an interactive shell in the dev container
make watch    # Start a cargo-watch daemon (rebuilds on file changes)
make test     # Run cargo test in the container
make fmt      # Run cargo fmt
make clippy   # Run cargo clippy --all-targets -- -D warnings
make release  # Build a static release binary → dist/hoist
make clean    # cargo clean + remove dist/hoist
```

## Testing

Unit tests live alongside their source files in `#[cfg(test)]` blocks. Run them with:

```sh
make test
```

`spec/anthropic/` provides fixture data for integration-style tests (a mock repo with `.claude/skills/`). CI runs `fmt`, `clippy`, and `test` in that order.

## Adding a New Strategy

1. Create a new file under `src/hoist/` (e.g., `src/hoist/anthropic/my_thing.rs`).

2. Implement the `HoistStrategy` trait:

   ```rust
   use crate::hoist::HoistStrategy;
   use crate::utils::relative_path;

   pub struct MyThingStrategy;

   impl HoistStrategy for MyThingStrategy {
       fn name(&self) -> &str { "anthropic/my_thing" }

       fn detect(&self, repo_root: &std::path::Path) -> bool {
           // return true if this repo contains the artifacts you handle
       }

       fn hoist(
           &self,
           repo_name: &str,
           repo_root: &std::path::Path,
           workspace_root: &std::path::Path,
           force: bool,
       ) -> anyhow::Result<()> {
           // symlink artifacts using relative_path() for portability
           Ok(())
       }
   }
   ```

3. Register the strategy in `all_strategies()` in `src/hoist/mod.rs`:

   ```rust
   pub fn all_strategies() -> Vec<Box<dyn HoistStrategy>> {
       vec![
           // existing strategies...
           Box::new(anthropic::my_thing::MyThingStrategy),
       ]
   }
   ```

4. Add test fixtures under `spec/` if the strategy needs real file layout to test against.

## Conventions

- **Relative symlinks** — always use `utils::relative_path()` so symlinks remain valid after the workspace is moved.
- **Namespace by repo/plugin name** — destination entries are prefixed with the source repo or plugin name (e.g., `canvas-lms-run-specs`).
- **Idempotent with warnings** — if a destination already exists and `--force` is not set, a warning is printed and the entry is skipped; no error is raised.
- **`--force` replaces** — existing symlinks (and broken dangling symlinks) are removed before re-creating.
- **`${CLAUDE_PLUGIN_ROOT}` substitution** — hook paths in `hooks/hooks.json` may contain this variable; it is replaced with the plugin's absolute path before merging into the workspace's `.claude/settings.json`.
- **Unhoist is symlink-only** — `hoist unhoist` only removes symlinks and hook entries; it never removes regular files or non-symlink directories in artifact dirs. Path-component-safe matching (trailing `/` on canonical paths) prevents a shorter root name from accidentally matching a longer one (e.g., `canvas` vs `canvas-lms`).
