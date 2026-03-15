# How it Works

## Hoist Strategy System

`hoist` symlinks AI agent artifacts from individual repos up to the current directory, namespaced by repo name. This makes workspace-level tooling (e.g. Claude Code) aware of skills and configs defined per-repo.

Using symlinks rather than copies means edits to a skill in a repo are immediately visible through the workspace, and relative path references within a skill directory (e.g. `../hooks/foo.sh`) remain valid because the OS resolves the symlink to the real path before following relative references.

**Detection**: Each hoist strategy scans repos for specific file patterns. A strategy only activates for repos where it detects matching files — repos without relevant files are silently skipped.

**Namespacing**: Hoisted files are renamed to include the source repo as a prefix (e.g. `canvas-lms-test-skill.md`) to avoid collisions between repos that define files with the same name.

**Current strategies**

| Strategy | Detects | Symlinks to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<cwd>/.claude/skills/<repo>-<filename>` |

**Skip behavior**: If a destination path already exists, hoist prints a warning to stderr and skips that entry.

## Workspace vs. Plain Repo

When given a root path (from `hoist.json` or a path argument), hoist checks for a `repos/` subdirectory:

- **Workspace**: if `repos/` exists, hoist iterates every directory inside it and applies all strategies to each.
- **Plain repo**: if `repos/` is absent, the root itself is treated as a single repo.

## Symlinks

All symlinks are relative (computed by `utils::relative_path()`), so the directory structure is fully portable — move or copy it anywhere and the links remain valid.
