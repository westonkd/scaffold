# Command Reference

## `hoist [PATH]`

Hoist AI agent artifacts from repos into the current directory, namespaced by repo name.

| Argument | Description |
|---|---|
| `[PATH]` | *(optional)* Path to a workspace or plain repo directory. If omitted, reads `hoist.json` from cwd. |

**Invocation modes**

| Invocation | Source | Destination |
|---|---|---|
| `hoist` | roots listed in `hoist.json` in cwd | cwd |
| `hoist <path>` | `<path>` (workspace or plain repo) | cwd |

**hoist.json schema**

```json
{
  "roots": [
    "./canvas-lms",
    "./my-other-repo"
  ]
}
```

Each root is resolved relative to cwd. If a root contains a `repos/` subdirectory it is treated as a workspace (all repos hoisted); otherwise it is treated as a single plain repo.

**Error conditions**

| Condition | Error |
|---|---|
| No-arg and `hoist.json` not in cwd | `hoist.json not found. Create one or provide a path argument.` |
| Root or path does not exist | `directory not found: <path>` |

**Hoist strategies**

Each strategy only activates for repos where it detects relevant files.

| Strategy | Detects | Symlinks to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<cwd>/.claude/skills/<repo>-<filename>` |

**Notes**

- If a destination path already exists, a warning is printed to stderr and the entry is skipped (no overwrite).
- All symlinks are relative so the directory structure is fully portable.
