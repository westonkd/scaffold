# Command Reference

## `scaffold build <blueprint>`

Build a workspace from a blueprint file.

| Argument | Description |
|---|---|
| `<blueprint>` | Path to the blueprint JSON file |

**Environment variables**

| Variable | Description |
|---|---|
| `SCAFFOLD_HOME` | Override the default `~/.scaffold` store location |

**Notes**

- If `<cwd>/<name>` already exists, the command exits with an error. Delete or rename that directory before rebuilding.
- If a repo is already cached, scaffold runs `git fetch` + `git pull` rather than re-cloning.
- The blueprint file is copied into the workspace as `blueprint.json`.

---

## `scaffold hoist [path]`

Collects AI agent artifacts (e.g. Claude Code skills) and copies them into the current directory, namespaced by repo name.

| Argument | Description |
|---|---|
| `[path]` | *(optional)* Workspace name, absolute/relative path to a workspace or plain repo directory, or path to a blueprint JSON file. If omitted, the current directory is used as the source. |

**Invocation modes**

| Invocation | Source | Destination |
|---|---|---|
| `scaffold hoist` | cwd (must be a scaffold workspace) | cwd |
| `scaffold hoist <path>` | `<path>` (workspace or plain repo) | cwd |

When `<path>` is a name with no slashes, it is resolved relative to cwd. When `<path>` ends with `.json`, it is read as a blueprint file and the workspace name is derived from it.

If the resolved path contains a `repos/` subdirectory it is treated as a scaffold workspace and all repos are hoisted. Otherwise it is treated as a single plain repo.

**Error conditions**

| Condition | Error |
|---|---|
| No-arg and `blueprint.json` not in cwd | `blueprint.json not found. Run this command from within a scaffold workspace.` |
| No-arg and `repos/` not in cwd | `repos/ directory not found. Is this a valid scaffold workspace?` |
| Path does not exist | `path not found: <path>. Build it first with \`scaffold build\`.` |

**Hoist strategies**

Each strategy only activates for repos where it detects relevant files.

| Strategy | Detects | Copies to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<cwd>/.claude/skills/<repo>-<filename>` |

**Notes**

- If a destination file already exists, a warning is printed to stderr and the file is skipped (no overwrite).
- Use `scaffold update` to re-hoist with overwrite enabled.

---

## `scaffold update`

Run from within an existing scaffold workspace (where `blueprint.json` is present). Updates all repos and re-hoists artifacts, overwriting previously hoisted files.

**Steps**

1. Reads `./blueprint.json` in the current directory
2. Updates each repo in the global cache (`~/.scaffold/projects/`) — fetches from remote and pulls
3. Updates each local clone in `./repos/` — fetches from the cache and pulls
4. Re-hoists all artifacts with overwrite enabled

```bash
cd my-workspace
scaffold update
```

**Environment variables**

| Variable | Description |
|---|---|
| `SCAFFOLD_HOME` | Override the default `~/.scaffold` store location |

**Notes**

- Must be run from the workspace root (the directory containing `blueprint.json` and `repos/`).
