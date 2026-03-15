# hoist

Hoist AI agent artifacts (Claude Code skills, etc.) from repos into your current directory.

**hoist is experimental!** This is a pattern I'm iterating on.

## The Problem

LLM agents don't reliably discover artifacts like skills and `AGENTS.md` files when they're nested in sub-directories across many repos. `hoist` surfaces those artifacts to a common root where agents can find them.

## Usage

### With a `hoist.json` config

Create `hoist.json` in your working directory:

```json
{
  "roots": [
    "./canvas-lms",
    "./my-other-repo"
  ]
}
```

Then run:

```bash
hoist
```

Each root is resolved relative to cwd. If a root contains a `repos/` subdirectory it is treated as a workspace (all repos hoisted); otherwise it is treated as a single plain repo.

### With a path argument

```bash
hoist ./some-repo
```

Hoists from the given directory into cwd. If the directory has a `repos/` subdirectory, it is treated as a workspace; otherwise it is treated as a single plain repo.

## How Hoisting Works

`hoist` symlinks AI agent artifacts from individual repos up to the current directory, namespaced by repo name. This makes workspace-level tooling (e.g. Claude Code) aware of skills and configs defined per-repo.

Using symlinks rather than copies means edits to a skill in a repo are immediately visible, and relative path references within a skill directory remain valid.

**Namespacing**: Hoisted files are renamed to include the source repo as a prefix (e.g. `canvas-lms-test-skill.md`) to avoid collisions.

**Skip behavior**: If a destination path already exists, hoist prints a warning to stderr and skips that entry.

### Hoist strategies

| Strategy | Detects | Symlinks to |
|---|---|---|
| `anthropic/claude_code/agent_skills` | `.claude/skills/*.md` in a repo | `<cwd>/.claude/skills/<repo>-<filename>` |

## Installation

```bash
cargo build --release
cp target/release/hoist ~/.local/bin/
```

## Documentation

- [Command reference](docs/commands.md)
- [How it works in depth](docs/how-it-works.md)
