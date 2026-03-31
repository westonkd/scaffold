# Hook Reference

The `post-merge` hook is the distribution mechanism for engineers. It runs automatically after `git pull` in a service repo, clones or updates the shared skills repo, and symlinks the selected skills into `.claude/skills/`.

Requirements: `git`, `python3`, standard POSIX tools. No AWS credentials, no additional auth.

---

## What it does

On every `post-merge` in a service repo:

1. Reads the skills repo URL from `~/.scaffold/settings.json`
2. Clones the skills repo to `~/.scaffold/agent-skills/` (first run) or pulls it (subsequent runs)
3. Reads `scaffold.json` in the service repo root to determine which skills to link
4. Symlinks each selected skill into `.claude/skills/_<name>/`
5. Adds `.claude/skills/_*/` to `.git/info/exclude`

The hook never uses `set -e`. Every error is logged and the hook exits 0 so it never interrupts a `git pull`.

> **Primary distribution path:** For engineers with Claude Code installed, the preferred path is the plugin marketplace — add the marketplace once with `/plugin marketplace add` and install individual skills with `/plugin install`. The bash hook is the fallback for repos or environments where the plugin flow is unavailable.

---

## Installation

### npm projects

Add to `package.json`:

```json
{
  "scripts": {
    "postinstall": "cp node_modules/.../hooks/post-merge .git/hooks/post-merge && chmod +x .git/hooks/post-merge"
  }
}
```

Or using `install.sh` from this repo:

```bash
bash /path/to/scaffold/install.sh
```

`install.sh` copies the hook, makes it executable, and runs it immediately so skills are available right after setup.

### Other projects

Via `Makefile`:

```makefile
setup:
	cp path/to/scaffold/hooks/post-merge .git/hooks/post-merge
	chmod +x .git/hooks/post-merge
	bash .git/hooks/post-merge
```

---

## Configuration

### `~/.scaffold/settings.json`

Set by org tooling during developer onboarding. The hook reads `skills_repo` from this file:

```json
{
  "skills_repo": "git@github.com:your-org/agent-skills.git"
}
```

If the file is absent or `skills_repo` is unset, the hook falls back to the `SCAFFOLD_SKILLS_REPO` environment variable. If neither is set, the hook logs an error and exits 0.

### `scaffold.json`

Optional. Place in the service repo root to declare which skills to link:

```json
{
  "skills": ["platform", "payments", "billing"]
}
```

If `scaffold.json` is absent, all skills in the repo are linked. If present with an empty `skills` array, no skills are linked. If the file is present but unparseable, the hook warns and links all skills.

---

## Skill naming rules

Skill names in `scaffold.json` must match `[a-z0-9][a-z0-9-]*`. Names that do not match are warned and skipped. This is the same constraint enforced by CI when skills are published.

---

## Skills directory

Symlinked skills land in `.claude/skills/` with a `_` prefix:

```
.claude/skills/
├── _platform/      → ~/.scaffold/agent-skills/agent-skills/plugins/platform/skills/platform/
├── _payments/      → ~/.scaffold/agent-skills/agent-skills/plugins/payments/skills/payments/
└── public-skill/   ← committed directly to the service repo (no prefix)
```

The `_` prefix distinguishes synced skills (never committed) from committed skills (public, tracked in git). The hook adds `.claude/skills/_*/` to `.git/info/exclude` automatically.

---

## Behavior reference

| Condition | Behavior |
|---|---|
| `~/.scaffold/agent-skills/` does not exist | Clones the skills repo |
| `~/.scaffold/agent-skills/` already exists | Runs `git pull --rebase` |
| Pull fails | Logs a warning, continues with the cached version |
| Clone fails (repo unreachable) | Logs an error, exits 0 |
| `scaffold.json` absent | Links all skills |
| `scaffold.json` present, `skills: []` | Links no skills |
| `scaffold.json` present, no `skills` key | Warns, links all skills |
| `scaffold.json` malformed | Warns, links all skills |
| Skill name fails validation | Warns and skips that skill |
| Skill not found in skills repo | Warns and skips that skill |
| Target path is a real directory (not a symlink) | Warns and skips to avoid data loss |
| Target is an existing symlink | Replaced with `ln -sfn` |
| `.git/info/` does not exist | Created automatically |
| Hook run from a subdirectory | Operates on the repo root regardless |
| Hook run twice | Idempotent; exclude entry not duplicated |
