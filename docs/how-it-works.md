# How it Works

## Workspace Layout

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

Top-level dependencies appear as symlinks directly in the workspace root. Sub-dependencies are linked into their parent's working tree at the path specified by the `path` field in the blueprint.

All symlinks are relative, so the workspace directory is fully portable — move or copy it anywhere and the links remain valid.

## Global Cache

On first build, scaffold clones each repo from its remote URL into `~/.scaffold/projects/<name>/`. This cache is shared across all workspaces.

On subsequent builds (or during `scaffold update`), scaffold runs `git fetch` + `git pull` on the cached clone rather than re-cloning. This keeps builds fast and avoids redundant network traffic.

The cache location can be overridden with the `SCAFFOLD_HOME` environment variable.

## Local Clones

Each workspace gets its own `repos/` directory containing local clones of every repo. These are created with `git clone --local` from the cache, making them fast to create and self-contained — they don't share object storage with the cache or with other workspaces.

The local clones are what you actually work in. The symlinks at the workspace root and within repo trees point into `repos/`.

## Symlink Strategy

scaffold uses two kinds of symlinks:

**Top-level symlinks** — created at the workspace root for each top-level dependency in the blueprint. For example, if `koa` is a top-level dependency, `koa-dev/koa` → `repos/koa`.

**Sub-dependency symlinks** — created inside a parent repo's working tree when a dependency specifies a `path` field. For example, with `"path": "packages"`, scaffold creates `repos/koa/packages/router` → `../../router`. The target path is computed to be relative from the symlink location to the sibling in `repos/`, keeping the link valid regardless of where the workspace lives.

## Hoist Strategy System

`scaffold hoist` copies AI agent artifacts from individual repos up to the workspace root, namespaced by repo name. This makes workspace-level tooling (e.g. Claude Code) aware of skills and configs defined per-repo.

**Detection**: Each hoist strategy scans repos for specific file patterns. A strategy only activates for repos where it detects matching files — repos without relevant files are silently skipped.

**Namespacing**: Hoisted files are renamed to include the source repo as a prefix (e.g. `koa-test-skill.md`) to avoid collisions between repos that define files with the same name.

**Current strategies**

| Strategy | Detects | Copies to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<workspace>/.claude/skills/<repo>-<filename>` |

**Skip behavior**: If a destination file already exists, hoist prints a warning to stderr and skips that file. Running `scaffold update` re-hoists with overwrite enabled, replacing previously hoisted files with the current versions from each repo.
