# Product Requirements Document: Scaffold

## Overview

Scaffold is an open-source platform for managing AI context artifacts — PRDs, ADRs, tech plans, design docs — across an engineering organization. It provides a standard format and tooling that makes these artifacts accessible to both humans and AI agents, without ever committing them to source repositories.

The primary use case is a large organization where designers, engineers, and product managers are adopting AI into the SDLC and need a shared, agent-native home for context documents.

---

## Problem Statement

As AI agents become first-class participants in the SDLC, context artifacts (PRDs, ADRs, tech plans) need to be:

- **Agent-readable** without polluting agent context with irrelevant content
- **Human-editable** without requiring engineering skills or a PR workflow
- **Never committed** to source repositories, especially open-source ones
- **Easy to update and share** across teams and tools

No existing solution satisfies all four constraints simultaneously.

---

## Goals

- Provide a standard, agent-native artifact format organizations can adopt
- Make artifact management accessible to non-technical contributors via a web editor
- Give engineers a fast CLI workflow that integrates with their existing tooling
- Be fully self-hostable by any organization with minimal infrastructure effort
- Be free and open source; commercial hosted offering may follow later

## Backwards Compatibility

This PRD represents a deliberate redesign of the scaffold CLI. There is no backwards compatibility requirement with prior versions. Implementors should proceed without concern for existing behavior, data formats, or CLI interfaces.

---

## Non-Goals

- Agent activation / skill invocation mechanics (handled by the agent framework)
- CLI authentication mechanics (engineers assume AWS IAM roles via existing org tooling)
- Change notifications or real-time collaboration (users implement their own; e.g., a session-start hook)
- Versioning UI / rollback UX (S3 versioning is enabled from day one as a safety net; user-facing rollback is deferred)
- Per-project access control (all authenticated users can read and write all skills for now)
- Merge conflict resolution (last-write-wins is the intentional model)

---

## Users & Personas

| Persona | Primary Surface | Key Need |
|---|---|---|
| Engineer | CLI (`scaffold`) | Fast sync of skills into local repos; local editing workflow |
| Product Manager / Designer | Web editor | Browse, create, and edit artifacts without CLI knowledge |
| Platform / DevOps | Terraform + Helm | Self-host and maintain the infrastructure |

---

## Terminology

| Term | Definition |
|---|---|
| **Project** | The user-facing concept. One project = one set of related context artifacts. |
| **Skill** | The underlying implementation format for a project. Not exposed in user-facing surfaces. |
| `platform` skill | A special skill containing cross-cutting ADRs and org-wide standards shared across projects. |

---

## Artifact Format

### Agent Skills

All artifacts are packaged as Agent Skills (`agentskills.io`). The format provides progressive disclosure:

- Only `name` + `description` (~100 tokens) load at agent startup
- Full `SKILL.md` body loads only when the skill is activated
- Reference files load on demand

### One Skill Per Project

Each project is a single skill. Individual artifact types live as reference files within it:

```
payments/
├── SKILL.md                          # Executive summary: what is this, key decisions, status
├── references/
│   ├── prd.md
│   ├── tech-plan.md
│   └── adrs/
│       ├── payment-processor-selection.md
│       └── retry-strategy.md
└── assets/
    └── architecture-diagram.png
```

`SKILL.md` is a lean orientation doc. Detailed content lives in `references/`. Individual ADR files (not a single `adrs.md`) keep context granular so agents only load what they need.

### The Platform Skill

Cross-cutting standards and org-wide ADRs live in a dedicated `platform` skill. Project skills declare a dependency on it:

```yaml
metadata:
  depends-on: platform
```

`scaffold sync` resolves `depends-on` declarations and pulls transitive dependencies automatically.

### Skill Frontmatter

```yaml
---
name: payments
description: >
  Context for the payments platform. Use when working on payment processing,
  billing, or any integration with payment providers.
metadata:
  type: project            # project | platform
  scope: repo-payments, repo-billing
  status: active           # active | archived
  depends-on: platform
  tags: payments, billing, fintech
---
```

The `tags` field is a comma-separated list of user-defined labels. Tags are mirrored as S3 object tags on `SKILL.md` at push time (see [S3 Object Tags](#s3-object-tags)).

### Skill Naming Rules

- Characters: alphanumeric, spaces allowed in input (normalized to hyphens)
- Stored as: lowercase, leading/trailing whitespace stripped, spaces replaced with hyphens (e.g., `my-payments-platform`)
- Display: the UI may render names as Title Case
- Unique within the organization's S3 bucket

---

## Storage: S3

Skills are stored in a private S3 bucket. No artifacts are ever committed to any repository.

### Design Decisions

| Decision | Rationale |
|---|---|
| S3, not Git | No PR/review overhead; direct PUT on save; S3 versioning handles rollback |
| Last-write-wins | Collisions are rare for this artifact type; conflict resolution is unnecessary complexity |
| S3 versioning enabled from day one | Safety net for accidental overwrites; rollback UX is deferred but the data is preserved |
| No branching or approval workflow | Deliberate simplification; these are living documents, not code |
| Push replaces the full skill folder | Each push syncs the entire skill directory to S3 and deletes any objects in that prefix that are no longer present locally (`s3 sync --delete` semantics) |

### S3 Object Tags

`SKILL.md` is the canonical object for a skill and carries two S3 object tags set at push time:

| Tag key | Source | Example value |
|---|---|---|
| `Description` | `description` field from `SKILL.md` frontmatter | `"Context for the payments platform..."` |
| `Tags` | `metadata.tags` from `SKILL.md` frontmatter (comma-separated) | `"payments,billing,fintech"` |

These tags enable lightweight filtering and discovery directly in S3 (e.g., via `aws s3api get-object-tagging`) without needing to download and parse every `SKILL.md`.

### Bucket Layout

```
s3://<bucket>/
  platform/
    SKILL.md
    references/
      ...
  payments/
    SKILL.md
    references/
      prd.md
      tech-plan.md
      adrs/
        payment-processor-selection.md
  billing/
    SKILL.md
    ...
```

---

## CLI: `scaffold`

Engineers interact with artifacts via `scaffold` commands. `hoist` functionality is composed internally for linking skills into repos.

### Settings: `~/.scaffold/settings.json`

All CLI commands that interact with S3 read the S3 bucket name from `~/.scaffold/settings.json`. This file is the single source of truth for machine-level scaffold configuration.

**Format:**
```json
{
  "bucket": "my-org-scaffold-artifacts"
}
```

**Behavior:**
- Any command that requires S3 access exits with a clear error if `~/.scaffold/settings.json` is missing or if `bucket` is not set
- The file is created manually by the engineer (or by org tooling) during initial setup; no scaffold command writes it automatically
- All other `~/.scaffold/` contents (skill directories, etc.) coexist with this file

---

### `scaffold new <name>`

Creates a new project locally and pushes it to S3.

Non-interactive by default — all options are flags:

```bash
scaffold new payments
scaffold new payments --description "Payment processing platform"
scaffold new payments --description "Payment processing platform" --minimal
```

**Behavior:**
- Normalizes `<name>`: lowercase, trim whitespace, spaces → hyphens
- Generates directory structure in `~/.scaffold/<name>/`:
  - `SKILL.md` with frontmatter pre-filled from `name` and `--description`
  - `references/prd.md`, `references/tech-plan.md`, `references/adrs/` with placeholder content
  - `--minimal` flag: creates only `SKILL.md`, skips reference scaffolding
- `--description` defaults to empty string if omitted
- Syncs the skill directory to S3 immediately after local creation (`s3 sync --delete` semantics); sets `Description` and `Tags` object tags on `SKILL.md`
- Has non-interactive mode available as the default

**Validation:**
- Skill name must match `[a-z0-9][a-z0-9-]*` (post-normalization)
- Exits with error if a skill with the same name already exists in S3

---

### `scaffold init`

Initializes artifact config for the current repository.

```bash
scaffold init
```

**Behavior:**
- Creates a `.scaffold-artifacts` config file in the current directory
- Prompts the engineer to select which scopes apply (from available skills listed in S3)
- `.scaffold-artifacts` is the **only** artifact-related file that ever gets committed to a repo
- OSS repos: no `.scaffold-artifacts` file; nothing is ever synced or committed

**Config format:**
```yaml
# .scaffold-artifacts
scopes:
  - payments
  - platform
```

---

### `scaffold sync`

Pulls skills from S3 and links them into the current repository.

```bash
scaffold sync
```

**Behavior:**
- Reads `.scaffold-artifacts` to determine which scopes to pull
- Resolves `depends-on` declarations and pulls transitive dependencies
- Always pulls fresh from S3 — no cache, no TTL
- Stores skills in `~/.scaffold/` (shared across all repos on the machine)
- Composes `hoist` to symlink skills into the current repo and add them to `.git/info/exclude` (never committed, invisible to git)
- If no `.scaffold-artifacts` found: exits with hint — `"No .scaffold-artifacts found. Run scaffold init to configure this repository."`

---

### `scaffold list`

Lists skills.

```bash
scaffold list           # skills currently installed in ~/.scaffold
scaffold list --remote  # all available skills in S3 (requires IAM role)
```

Output columns: name, type, status, scope.

---

### `scaffold config`

Gets and sets values in `~/.scaffold/settings.json`.

```bash
scaffold config get bucket               # prints the current bucket name
scaffold config set bucket <bucket-name> # sets the bucket name
```

**Behavior:**
- `get <key>`: prints the value for the given key; exits with error if the key is not set
- `set <key> <value>`: writes the value for the given key to `~/.scaffold/settings.json`, creating the file if it does not exist
- Unknown keys are rejected with an error listing valid keys

---

### `scaffold push`

Pushes local skill changes back to S3.

```bash
scaffold push
scaffold push <name>    # push a specific skill
```

For engineers who prefer editing locally and syncing up. Syncs the full skill directory to S3 (`s3 sync --delete` semantics — objects no longer present locally are removed from the bucket prefix) and updates the `Description` and `Tags` object tags on `SKILL.md`.

---

### Authentication

Engineers assume AWS IAM roles using existing org tooling. S3 bucket policies are attached to those roles. No separate auth system is needed for the CLI. The mechanics of role assumption are out of scope.

---

## Web Editor

A web application for non-technical contributors to browse, create, and edit skills.

### Authentication

SAML — integrates with the organization's existing identity provider (Okta, Azure AD, etc.).

### Features

**Browse**
- List all skills with filter by name (substring match)
- Filter controls for `type` (project / platform) and `status` (active / archived)

**Edit**
- Edit any file within a skill (`SKILL.md` or any reference file) in a markdown editor
- Save writes directly to S3 via the backend (SAML session → backend → S3)
- No draft state — save is immediate and authoritative

**Create (new skill wizard)**
1. Enter project name (validated and normalized per naming rules; error shown if name already exists)
2. `SKILL.md` pre-filled with correct frontmatter template
3. Optionally add reference files (PRD, tech plan, ADRs) — same option as `--minimal` vs full scaffold
4. Save → creates skill directory structure in S3

**Utility actions**
- **"Copy as Markdown"** button on every file — copies raw markdown to clipboard for pasting into any AI tool
- **"Download for Agent"** button on every skill — produces a formatted context bundle for the skill and all its `depends-on` dependencies as a single downloadable file

### Writes

All writes go directly to S3 via the web app backend. No intermediate queue or approval step.

---

## Infrastructure & Self-Hosting

Self-hosting is a first-class goal. Any organization should be able to run this stack with minimal effort.

### Terraform Modules

| Module | Resources |
|---|---|
| `s3` | Private bucket, versioning enabled, bucket policy |
| `iam` | Read/write role for CLI users; read-only role for broader access |
| `saml` | Configurable IdP metadata URL (works with any SAML provider) |
| `web` | ECS deployment with ALB and TLS termination |

Modules are independently deployable so organizations can adopt incrementally (e.g., just `s3` + `iam` first).

### Helm Chart (Kubernetes)

For organizations running k8s instead of ECS:

- Standard `Deployment`, `Service`, `Ingress`
- `ConfigMap` for environment-specific config (bucket name, SAML IdP URL, etc.)
- `Secret` references compatible with External Secrets Operator
- Sane resource requests/limits defaults
- Horizontal pod autoscaling included
- Works against any standard k8s distribution (EKS, GKE, AKS, self-managed)

### Self-Hosting Checklist

A self-hoster needs to provide:

1. S3-compatible bucket (AWS S3, MinIO, or any compatible store)
2. IAM roles or equivalent (for CLI auth)
3. A SAML IdP (for web editor auth)
4. A container runtime (ECS, k8s, or equivalent) for the web editor

Everything else is handled by the Terraform modules and Helm chart.

### Licensing

MIT license. Self-hosting is free for any organization. A hosted SaaS offering may be offered commercially in the future; self-hosted use is and will remain free.

---

## Auth Summary

| Surface | Mechanism |
|---|---|
| Web editor | SAML |
| CLI | AWS IAM role (assumed via existing eng tooling) |
| S3 bucket | IAM policies attached to roles |

---

## CLI & Web App Independence

The CLI and web app do not share a library. Each has its own implementation of operations like "create new project." This is an intentional simplification — shared packages add coordination overhead that is not justified until there is a clear, proven need.

---

## Build Sequence

| Phase | Deliverable | Unlocks |
|---|---|---|
| 1 | S3 bucket + skill structure; create `platform` skill with real ADRs | Validates format end-to-end |
| 2 | `scaffold init` + `scaffold sync` | Unblocks engineers immediately |
| 3 | `scaffold list` | Local and remote skill discovery |
| 4 | `scaffold push` | Local editing workflow for engineers |
| 5 | Web editor (browse + edit first; new skill wizard second pass) | Non-technical contributor on-ramp |
| 6 | `depends-on` auto-pull | Useful once the skill graph has enough nodes |
