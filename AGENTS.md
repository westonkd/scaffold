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

`scaffold` is a CLI tool that builds portable, symlinked monorepo workspaces from a JSON blueprint. It has one command: `scaffold build <blueprint.json>`.

**Execution flow in `commands/build.rs`:**
1. Parse blueprint JSON (`blueprint.rs`) into `BlueprintConfig` / `Dependency` structs
2. Recursively collect all unique repos (DFS over nested `dependencies`)
3. Clone/update each repo into a global cache at `~/.scaffold/projects/` (or `$SCAFFOLD_HOME/projects/`)
4. `git clone --local` each cached repo into `<cwd>/<blueprint_name>/repos/<repo_name>/`
5. Create relative symlinks: top-level repos get `<blueprint>/<repo_name>` → `repos/<repo_name>`; sub-dependencies get symlinked into their parent's directory tree at the specified `path`

**Key design decisions:**
- All symlinks are **relative** (computed by `store::relative_path()`) so the workspace directory is fully portable
- Cache uses `git fetch` + `git pull` on existing clones for efficiency
- Building refuses to overwrite an existing blueprint directory

**Module responsibilities:**
- `main.rs` — clap CLI, single `build` subcommand
- `blueprint.rs` — serde structs for the JSON config (`ref` field uses `#[serde(rename)]` since it's a Rust keyword)
- `store.rs` — `ScaffoldStore` (cache path resolution) + `relative_path()` utility with unit tests
- `git.rs` — thin wrappers around the `git` CLI binary
- `commands/build.rs` — all orchestration logic

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
