# scaffold

Assemble self-contained, symlinked monorepo workspaces from a single JSON blueprint.

## The Problem

Working across multiple related repos is painful: symlinks break when moved, submodules have poor ergonomics, and ad-hoc shell scripts are brittle and hard to share. scaffold builds a portable, reproducible workspace from a single JSON blueprint — clone it once and get a directory tree where every repo is in the right place, sub-dependencies are linked into their parents, and the whole thing moves without breaking.

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

## Quickstart

Write a blueprint file (e.g. `koa-dev.json`):

```json
{
  "name": "koa-dev",
  "dependencies": [
    { "name": "koa", "source": "https://github.com/koajs/koa.git", "ref": "master" }
  ]
}
```

Build the workspace:

```bash
scaffold build koa-dev.json
```

## Commands

- `scaffold build <blueprint>` — Build a workspace from a blueprint file
- `scaffold hoist <workspace>` — Collect AI agent artifacts into the workspace root
- `scaffold update` — Update all repos and re-hoist in an existing workspace

## Documentation

- [Command reference](docs/commands.md)
- [Blueprint schema](docs/blueprint-schema.md)
- [How it works in depth](docs/how-it-works.md)
