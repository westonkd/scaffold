# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

All development is Docker-based via Make targets:

```bash
make build        # Build the dev Docker image
make dev          # Open interactive shell in dev container
make watch        # Start cargo-watch daemon (live recompilation on src/ changes)
make test         # Run cargo test
make clippy       # Run cargo clippy (warnings as errors)
make fmt          # Run cargo fmt
make release      # Build musl-static release binary → dist/hoist
make clean        # cargo clean + remove dist/hoist
make clean-volumes # Remove Docker volumes (clears cargo cache)
```

To run a single test:
```bash
docker compose run --rm dev cargo test <test_name>
```

## Architecture

`hoist` is a CLI tool that symlinks AI agent artifacts from repos into the current directory. Invocation: `hoist [PATH]`.

**Execution flow in `commands/hoist.rs`:**
1. No-arg mode: read `hoist.json` from cwd, iterate `roots`, call `hoist_from_root()` for each
2. Path-arg mode: resolve the given path, call `hoist_from_root()` on it
3. `hoist_from_root()`: if the path has a `repos/` subdirectory → workspace mode (iterate each repo); otherwise → single-repo mode
4. For each repo, run all registered hoist strategies (`hoist::run_all_strategies(..., force=false)`)

**Hoist strategy system (`src/hoist/`):**

Strategies are namespaced by vendor and agent: `hoist/<vendor>/<agent>/<strategy>.rs`. Each implements the `HoistStrategy` trait:
- `detect(&Path) -> bool` — return `true` if the strategy applies to this repo
- `hoist(repo_name, repo_root, workspace_root, force: bool) -> Result<()>` — perform the symlink; when `force=true`, overwrite existing destination files

Registered in `hoist::all_strategies()`. Currently implemented:

| Module | Strategy name | Behavior |
|---|---|---|
| `hoist/anthropic/claude_code/agent_skills.rs` | `anthropic/claude_code/agent_skills` | Symlinks `.claude/skills/*.md` → `<cwd>/.claude/skills/<repo>-<filename>`; warns and skips on conflict |

**Key design decisions:**
- All symlinks are **relative** (computed by `utils::relative_path()`) so the directory is fully portable
- Hoist strategies are opt-in per repo (detect-then-act); repos with no matching artifacts are silently skipped
- Hoist skips (with a stderr warning) rather than overwriting existing destination files; `force=true` bypasses this

**Module responsibilities:**
- `main.rs` — clap CLI, dispatches to `commands::hoist::run(path)`
- `commands/hoist.rs` — path resolution, `hoist.json` parsing, workspace/plain-repo dispatch
- `hoist/mod.rs` — `HoistStrategy` trait, strategy registry, `run_all_strategies()`
- `utils.rs` — `relative_path()` utility with unit tests
