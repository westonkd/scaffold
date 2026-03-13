# scaffold

🏗️ Compose multi-repo development environments and manage AI agent artifacts across them.

## The Problem

Agentic code workflows introduce three problems that existing tools don't solve together:

1. **Project composition** — Large projects often span multiple repos. Assembling them into a coherent dev environment with correct dependency structure is tedious and error-prone.
2. **Agent artifact discovery** — LLM agents don't reliably discover artifacts like skills and `AGENT.md` files when they're nested in sub-directories across many repos.
3. **Proprietary agent artifacts** — Agent artifacts sometimes contain proprietary context to be effective. That's a conflict when the artifact lives alongside open-source code.

scaffold addresses all three: `build` assembles repos into a single workspace, `hoist` surfaces agent artifacts to the workspace root where agents can find them, and the blueprint-per-workspace model keeps proprietary agent config separate from source repos.

## Quickstart

**1. Write a blueprint file** (e.g. `koa-dev.json`):

```json
{
  "name": "koa-dev",
  "dependencies": [
    {
      "name": "koa",
      "source": "https://github.com/koajs/koa.git",
      "ref": "master"
    },
    {
      "name": "compose",
      "source": "https://github.com/koajs/compose.git",
      "ref": "master"
    }
  ]
}
```

**2. Build the workspace:**

```bash
scaffold build koa-dev.json
```

This clones both repos into `~/.scaffold/projects/`, creates local clones under `koa-dev/repos/`, and symlinks `koa` and `compose` at the workspace root.

**3. Hoist agent artifacts to the workspace root:**

```bash
scaffold hoist koa-dev
```

Any agent skills or other artifacts found in each repo are symlinked into `koa-dev/.claude/skills/`, namespaced by repo name (e.g. `koa-test-skill.md`, `compose-test-skill.md`).

**4. Later, refresh the workspace** (pull all repos and re-hoist):

```bash
cd koa-dev
scaffold update
```

## How it Works

```
~/.scaffold/
└── projects/               ← git clone cache
    ├── koa/
    ├── router/
    └── compose/

<cwd>/
└── koa-dev/                ← built workspace
    ├── repos/              ← local clones (internal)
    │   ├── koa/
    │   ├── router/
    │   └── compose/
    ├── koa                 → repos/koa      (symlink)
    └── compose             → repos/compose  (symlink)

# Sub-dependency symlink, relative within repos/:
repos/koa/packages/router  →  ../../router
```

All symlinks are relative, so the workspace is fully portable.

## Installation

```bash
cargo build --release
cp target/release/scaffold ~/.local/bin/
```

## Commands

- `scaffold build <blueprint>` — Build a workspace from a blueprint file
- `scaffold hoist [path]` — Collect AI agent artifacts into `cwd`. With no argument, `cwd` must be a scaffold workspace. With an argument, treats `path` as a workspace (if it has a `repos/` directory) or a plain repo, and hoists artifacts into `cwd`.
- `scaffold update` — Update all repos and re-hoist in an existing workspace

## Documentation

- [Command reference](docs/commands.md)
- [Blueprint schema](docs/blueprint-schema.md)
- [How it works in depth](docs/how-it-works.md)
