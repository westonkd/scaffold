# scaffold

Assemble self-contained, symlinked monorepo workspaces from a blueprint config.

## How it works

`scaffold build` reads a JSON blueprint, clones all referenced git repositories into a local cache (`~/.scaffold/projects/`), then assembles a workspace in `<cwd>/<name>/` where top-level repos are accessible as relative symlinks and sub-dependencies are linked into their parent's working tree.

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

All symlinks are relative, so the blueprint directory is fully portable.

## Installation

```bash
cargo build --release
# Optionally copy to your PATH:
cp target/release/scaffold ~/.local/bin/
```

## Usage

```
scaffold build <blueprint>
scaffold hoist <workspace>
```

### `scaffold build <blueprint>`

| Argument | Description |
|---|---|
| `<blueprint>` | Path to the blueprint JSON file |

**Environment variables**

| Variable | Description |
|---|---|
| `SCAFFOLD_HOME` | Override the default `~/.scaffold` store location |

### `scaffold hoist <workspace>`

Collects AI agent artifacts (e.g. Claude Code skills) from each repo in a built workspace and copies them to the workspace root, namespaced by repo name.

| Argument | Description |
|---|---|
| `<workspace>` | Workspace name (directory under cwd) or path to a blueprint JSON file |

Each strategy only activates for repos where it detects relevant files. Currently supported:

| Strategy | Detects | Copies to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<workspace>/.claude/skills/<repo>-<filename>` |

If a destination file already exists, a warning is printed to stderr and the file is skipped (no overwrite).

### Example

Given the blueprint below saved as `koa-dev.json`:

```json
{
  "name": "koa-dev",
  "dependencies": [
    {
      "name": "koa",
      "source": "https://github.com/koajs/koa.git",
      "ref": "master",
      "dependencies": [
        {
          "name": "router",
          "source": "https://github.com/koajs/router.git",
          "path": "packages",
          "ref": "master"
        }
      ]
    },
    {
      "name": "compose",
      "source": "https://github.com/koajs/compose.git",
      "ref": "master"
    }
  ]
}
```

Build the workspace:

```bash
scaffold build koa-dev.json
```

This will:
1. Clone (or update) each repo into `~/.scaffold/projects/`
2. Create local clones under `./koa-dev/repos/`
3. Symlink `koa` and `compose` at the `koa-dev/` root
4. Symlink `router` into `koa/packages/` so it appears as a local package
5. Copy `koa-dev.json` into `koa-dev/blueprint.json`

Then hoist AI artifacts from each repo up to the workspace root:

```bash
scaffold hoist koa-dev
# or equivalently:
scaffold hoist koa-dev.json
```

If `koa` and `compose` each have `.claude/skills/test-skill.md`, this produces:

```
koa-dev/.claude/skills/
  koa-test-skill.md
  compose-test-skill.md
```

## Blueprint schema

```json
{
  "name": "string",
  "dependencies": [
    {
      "name": "string",
      "source": "string (git remote URL)",
      "ref": "string (branch / tag / commit, optional)",
      "path": "string (location inside parent repo to symlink into, optional)",
      "dependencies": []
    }
  ]
}
```

| Field | Required | Description |
|---|---|---|
| `name` | yes | Repo directory name and symlink name |
| `source` | yes | Git remote URL |
| `ref` | no | Branch, tag, or commit to check out (defaults to the remote default branch) |
| `path` | no | Relative path inside the parent repo where this dep is symlinked |
| `dependencies` | no | Nested sub-dependencies (recursive) |

## Notes

- Running `scaffold build` when `<cwd>/<name>` already exists will exit with an error. Delete or rename that directory before rebuilding.
- Re-running on an already-cached project will `git fetch` + `git pull` rather than re-cloning.
