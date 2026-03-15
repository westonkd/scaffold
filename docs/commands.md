# Command Reference

## `hoist [PATH]`

Symlink AI agent artifacts from one or more source directories into cwd, namespaced by source directory name.

| Argument | Description |
|---|---|
| `[PATH]` | *(optional)* Path to a directory to hoist from. If omitted, reads `hoist.json` from cwd. |

**Invocation modes**

| Invocation | Source | Destination |
|---|---|---|
| `hoist` | roots listed in `hoist.json` in cwd | cwd |
| `hoist <path>` | `<path>` | cwd |

**hoist.json schema**

```json
{
  "roots": [
    "./canvas-lms",
    "./my-other-repo"
  ]
}
```

Each root is resolved relative to cwd.

**Error conditions**

| Condition | Error |
|---|---|
| No-arg and `hoist.json` not in cwd | `hoist.json not found. Create one or provide a path argument.` |
| Root or path does not exist | `directory not found: <path>` |

**Hoist strategies**

| Strategy | Detects | Symlinks to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/` — flat `.md` files or directories containing `SKILL.md` | `<cwd>/.claude/skills/<root>-<entry>` |

See [how it works](how-it-works.md) for details on namespacing, skip behavior, and adding new strategies.
