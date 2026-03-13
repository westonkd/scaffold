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

## `scaffold hoist <workspace>`

Collects AI agent artifacts (e.g. Claude Code skills) from each repo in a built workspace and copies them to the workspace root, namespaced by repo name.

| Argument | Description |
|---|---|
| `<workspace>` | Workspace name (directory under cwd) or path to a blueprint JSON file |

**Hoist strategies**

Each strategy only activates for repos where it detects relevant files.

| Strategy | Detects | Copies to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<workspace>/.claude/skills/<repo>-<filename>` |

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
