# Command Reference

## `hoist [PATH]`

Symlink AI agent artifacts from one or more source directories into cwd, namespaced by source directory name.

| Argument | Description |
|---|---|
| `[PATH]` | *(optional)* Path to a directory to hoist from. If omitted, reads `hoist.json` from cwd. |

**Flags**

| Flag | Description |
|---|---|
| `--force` | Replace existing symlinks and re-merge hooks from scratch. |

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

| Strategy | Detects | Hoists |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/` — flat `.md` files or dirs containing `SKILL.md` | Skills → `.claude/skills/<repo>-<name>` |
| `anthropic/plugin` | `.claude-plugin/plugin.json` | Skills, agents, commands, and hooks from a single plugin repo |
| `anthropic/marketplace` | `.claude-plugin/marketplace.json` | All locally-sourced plugins listed in the marketplace file |

See [how it works](how-it-works.md) for details on namespacing, skip behavior, and adding new strategies.

---

## `hoist unhoist [PATH]`

Remove previously hoisted artifacts from the workspace. Covers symlinks in `.claude/skills/`, `.claude/agents/`, and `.claude/commands/`, as well as hook entries merged into `.claude/settings.json`.

| Argument | Description |
|---|---|
| `[PATH]` | *(optional)* Path whose artifacts should be removed. If omitted, reads `hoist.json` and prunes artifacts from any root no longer listed. |

**Flags**

| Flag | Description |
|---|---|
| `--dry-run` | Print what would be removed without removing anything. |

**Invocation modes**

| Invocation | Behavior |
|---|---|
| `hoist unhoist <path>` | Remove all artifacts whose symlink target resolves into `<path>`. |
| `hoist unhoist` | Read `hoist.json`; remove artifacts whose source is not listed as a root. |
| Either + `--dry-run` | Print `[symlink]` / `[hook]` lines for each artifact that would be removed. |

**Notes**

- Non-symlink files and real directories in artifact dirs are never removed.
- Broken/dangling symlinks (whose target no longer exists) are treated as orphaned and removed in prune mode.
- Hook entries in `.claude/settings.json` are matched by the canonical source path embedded when they were hoisted. All other settings in that file are preserved.

**Error conditions**

| Condition | Error |
|---|---|
| No-arg and `hoist.json` not in cwd | `hoist.json not found.` with usage hint |
| `<path>` does not exist | `directory not found: <path>` |
| `settings.json` is malformed JSON | parse error with context |
