# CLAUDE.md

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
make release      # Build musl-static release binary → dist/scaffold
make clean        # cargo clean + remove dist/scaffold
make clean-volumes # Remove Docker volumes (clears cargo cache)
```

To run a single test:
```bash
docker compose run --rm dev cargo test <test_name>
```

## Architecture

`scaffold` is a CLI tool that builds portable, symlinked monorepo workspaces from a JSON blueprint. It has three commands: `scaffold build`, `scaffold hoist`, and `scaffold update`.

**Execution flow in `commands/build.rs`:**
1. Parse blueprint JSON (`blueprint.rs`) into `BlueprintConfig` / `Dependency` structs
2. Recursively collect all unique repos (DFS over nested `dependencies`)
3. Clone/update each repo into a global cache at `~/.scaffold/projects/` (or `$SCAFFOLD_HOME/projects/`)
4. `git clone --local` each cached repo into `<cwd>/<blueprint_name>/repos/<repo_name>/`
5. Create relative symlinks: top-level repos get `<blueprint>/<repo_name>` → `repos/<repo_name>`; sub-dependencies get symlinked into their parent's directory tree at the specified `path`
6. Copy the blueprint JSON into `<workspace>/blueprint.json`

**Execution flow in `commands/hoist.rs`:**
1. Resolve the workspace path — accepts either a workspace name (`koa-dev`) or a path to a blueprint JSON file (`koa-dev.json`)
2. Iterate each directory under `<workspace>/repos/`
3. For each repo, run all registered hoist strategies (`hoist::run_all_strategies(..., force=false)`)

**Execution flow in `commands/update.rs`:**
1. Read `./blueprint.json` from CWD; bail if not found
2. Validate `./repos/` exists
3. Collect all unique repos via `collect_deps()` (same DFS as build)
4. For each repo, update the global cache (`~/.scaffold/projects/<name>`): `git fetch` + optional `git checkout <ref>` + `git pull` (clones fresh if cache is missing)
5. For each local clone in `./repos/`: `git fetch` + optional `git checkout <ref>` + `git pull`
6. Re-hoist all repos with `force=true` (overwrites existing files)

**Hoist strategy system (`src/hoist/`):**

Strategies are namespaced by vendor and agent: `hoist/<vendor>/<agent>/<strategy>.rs`. Each implements the `HoistStrategy` trait:
- `detect(&Path) -> bool` — return `true` if the strategy applies to this repo
- `hoist(repo_name, repo_root, workspace_root, force: bool) -> Result<()>` — perform the copy; when `force=true`, overwrite existing destination files

Registered in `hoist::all_strategies()`. Currently implemented:

| Module | Strategy name | Behavior |
|---|---|---|
| `hoist/anthropic/claude_code/agent_skills.rs` | `anthropic/claude_code/agent_skills` | Copies `.claude/skills/*.md` → `<workspace>/.claude/skills/<repo>-<filename>`; warns and skips on conflict |

**Key design decisions:**
- All symlinks are **relative** (computed by `store::relative_path()`) so the workspace directory is fully portable
- Cache uses `git fetch` + `git pull` on existing clones for efficiency
- Building refuses to overwrite an existing blueprint directory
- Hoist strategies are opt-in per repo (detect-then-act); repos with no matching artifacts are silently skipped
- Hoist skips (with a stderr warning) rather than overwriting existing destination files; `force=true` bypasses this
- `scaffold update` propagates changes remote → cache → local clone (local clones' origin is the cache, not the remote)

**Module responsibilities:**
- `main.rs` — clap CLI, `build`, `hoist`, and `update` subcommands
- `blueprint.rs` — serde structs for the JSON config (`ref` field uses `#[serde(rename)]` since it's a Rust keyword)
- `store.rs` — `ScaffoldStore` (cache path resolution) + `relative_path()` utility with unit tests
- `git.rs` — thin wrappers around the `git` CLI binary
- `commands/build.rs` — workspace build orchestration
- `commands/hoist.rs` — workspace resolution and per-repo strategy dispatch
- `commands/update.rs` — in-place workspace refresh (cache + local clones + re-hoist)
- `hoist/mod.rs` — `HoistStrategy` trait, strategy registry, `run_all_strategies()`

## Blueprint Schema

```json
{
  "name": "workspace-name",
  "dependencies": [
    {
      "name": "repo-id",
      "source": "https://github.com/org/repo",
      "ref": "main",
      "path": "packages/sub-dir",
      "dependencies": []
    }
  ]
}
```

`ref` and `path` are optional. `path` controls where the symlink is placed inside the parent repo's working tree (used for monorepos with nested packages).
