# scaffold

Manage AI context artifacts (PRDs, ADRs, tech plans) across an engineering organization. Skills are stored in a private S3 bucket and linked into repos on demand — never committed.

> [!IMPORTANT]
> Scaffold is experimental. APIs and formats may change.

---

## Installation

Build from source:

```bash
cargo build --release
cp target/release/scaffold ~/.local/bin/
```

---

## Configuration

All commands that access S3 require a bucket to be configured:

```bash
scaffold config set bucket my-org-scaffold-artifacts
```

Settings are stored in `~/.scaffold/settings.json`.

---

## Commands

### `scaffold new <name>`

Create a new skill and push it to S3.

```bash
scaffold new payments
scaffold new payments --description "Payment processing platform"
scaffold new payments --minimal   # SKILL.md only, no reference files
```

### `scaffold pull [<name>]`

Download skills from S3 into `~/.scaffold/`.

```bash
scaffold pull              # pull all skills in the bucket
scaffold pull payments     # pull a specific skill
```

### `scaffold link [<name>]`

Link a skill into the current working directory via symlink. The symlink is placed at `.claude/skills/<name>` and added to `.git/info/exclude` so it is never committed.

```bash
scaffold link payments     # link a specific skill
scaffold link              # link all scopes from .scaffold-artifacts (see below)
scaffold link --force      # replace existing symlinks
```

`scaffold link` without an argument reads `.scaffold-artifacts` from the current directory. Create this file manually to configure which skills are linked in a given repo:

```yaml
# .scaffold-artifacts
scopes:
  - payments
  - platform
```

This is the only scaffold-related file that should be committed to a repository. Running `scaffold link` will link each listed scope and automatically resolve any `depends-on` dependencies declared in a skill's frontmatter.

**Typical workflow for a new repo:**

```bash
# 1. Create .scaffold-artifacts listing the skills you want
cat > .scaffold-artifacts << 'EOF'
scopes:
  - payments
  - platform
EOF

# 2. Pull the skills from S3
scaffold pull payments
scaffold pull platform

# 3. Link them into this repo
scaffold link
```

### `scaffold list`

List locally installed skills and their link status in the current directory.

```bash
scaffold list           # locally installed skills (~/.scaffold/)
scaffold list --remote  # all skills available in S3
```

Output shows name, whether the skill is linked in the current directory, and description. Linked skills appear first.

### `scaffold push [<name>]`

Push local skill changes back to S3.

```bash
scaffold push              # push all locally modified skills
scaffold push payments     # push a specific skill
```

### `scaffold edit <name>`

Pull a skill from S3 and open it for editing.

```bash
scaffold edit payments
```

Opens the skill directory in `$VISUAL` or `$EDITOR`. Does not push automatically — run `scaffold push` when done.

### `scaffold config`

Get or set configuration values.

```bash
scaffold config get bucket
scaffold config set bucket my-org-scaffold-artifacts
```

---

## Skill format

Each skill is a directory in `~/.scaffold/<name>/`:

```
payments/
├── SKILL.md                    # frontmatter + executive summary
├── references/
│   ├── prd.md
│   ├── tech-plan.md
│   └── adrs/
│       └── payment-processor-selection.md
└── assets/
    └── architecture-diagram.png
```

`SKILL.md` frontmatter:

```yaml
---
name: payments
description: >
  Context for the payments platform. Use when working on payment
  processing, billing, or any integration with payment providers.
metadata:
  type: project
  status: active
  scope: repo-payments, repo-billing
  depends-on: platform
  tags: payments, billing, fintech
---
```

The `depends-on` field causes `scaffold link` to automatically link the referenced skill as well.
