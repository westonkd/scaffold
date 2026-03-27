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
- IdP configuration or VPN infrastructure (both are operated externally; scaffold consumes them)
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

`scaffold pull` and `scaffold link` both resolve `depends-on` declarations and pull/link transitive dependencies automatically.

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

All CLI commands that interact with S3 read the S3 bucket name and API Gateway endpoint from `~/.scaffold/settings.json`. This file is the single source of truth for machine-level scaffold configuration. Auth tokens are stored separately in the OS credential store, not in this file.

**Format:**
```json
{
  "bucket": "my-org-scaffold-artifacts",
  "api_gateway_url": "https://api.example.com/scaffold"
}
```

**Behavior:**
- Any command that requires S3 access exits with a clear error if `~/.scaffold/settings.json` is missing or if `bucket` or `api_gateway_url` is not set
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
- OSS repos: no `.scaffold-artifacts` file; nothing is ever linked or committed
- This file is the setup step for `scaffold link` (no-arg mode)

**Config format:**
```yaml
# .scaffold-artifacts
scopes:
  - payments
  - platform
```

---

### `scaffold list`

Lists skills.

```bash
scaffold list           # skills currently installed in ~/.scaffold
scaffold list --remote  # all available skills in S3 (requires IAM role)
```

Output columns (in order): **name**, **linked** (is the skill linked into the CWD via `scaffold link`?), **description**. Linked skills are shown first, followed by unlinked. Remote-only skills (`--remote`) always show as unlinked.

---

### `scaffold link [<name>]`

Links one or all locally-installed skills into the current working directory via symlink.

```bash
scaffold link <name>   # link a specific skill
scaffold link          # link all scopes from .scaffold-artifacts
```

**Behavior:**
- With `<name>`:
  - Normalizes `<name>` (same rules as `scaffold new`)
  - Exits with error if the skill does not exist in `~/.scaffold/` (hint: run `scaffold pull <name>` first)
  - Creates a symlink from the current working directory into `~/.scaffold/<name>/` using the agent skills strategy
  - Adds the symlink path to `.git/info/exclude` so it is never committed
  - Idempotent: running twice on the same skill in the same directory is a no-op

- Without `<name>`:
  - Reads `.scaffold-artifacts` to determine which scopes to link
  - Exits with hint if no `.scaffold-artifacts` found: `"No .scaffold-artifacts found. Run scaffold init to configure this repository."`
  - Links each configured scope (same behavior as with `<name>`, one per scope)
  - Resolves `depends-on` declarations and links transitive dependencies

**Flags:**
- `--force` / `-f`: replace existing symlinks (handles stale links)

---

### `scaffold config`

Gets and sets values in `~/.scaffold/settings.json`.

```bash
scaffold config get bucket                            # prints the current bucket name
scaffold config set bucket <bucket-name>              # sets the bucket name
scaffold config get api_gateway_url                   # prints the current API Gateway URL
scaffold config set api_gateway_url <url>             # sets the API Gateway URL
```

**Behavior:**
- `get <key>`: prints the value for the given key; exits with error if the key is not set
- `set <key> <value>`: writes the value for the given key to `~/.scaffold/settings.json`, creating the file if it does not exist
- Unknown keys are rejected with an error listing valid keys
- Valid keys: `bucket`, `api_gateway_url`

---

### `scaffold login`

Explicitly initiates the device authorization flow to authenticate with the Instructure identity service.

```bash
scaffold login
```

**Behavior:**
- Initiates the device authorization flow: prints a verification URL and user code
- Polls the identity service until the engineer completes the browser flow or the code expires
- On success: stores the token securely in the OS credential store and prints a confirmation
- On failure / timeout: exits with a clear error message
- If a valid, refreshable token already exists: prints a confirmation and exits without re-authenticating (use `--force` to re-authenticate anyway)

**Flags:**
- `--force` / `-f`: discard any existing token and re-authenticate

This command is optional for day-to-day use — authentication happens lazily on the first S3 operation. `scaffold login` is useful for pre-authenticating before going into an environment where browser access is unavailable.

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

All CLI commands that interact with S3 do so through an AWS API Gateway. The API Gateway uses a custom authorizer that validates JWTs and enforces VPN-only access. Engineers never interact with S3 directly.

**Token acquisition:**
- Tokens are fetched from the Instructure identity service using the OAuth 2.0 device authorization flow (RFC 8628)
- Tokens are long-lived (weeks) to minimize re-authentication friction
- Token refresh happens transparently; the engineer is only prompted to re-authenticate when the token cannot be refreshed

**Token storage:**
- Tokens are stored securely in the OS credential store (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows)
- No tokens are written to `~/.scaffold/settings.json` or any plain-text file

**Lazy authorization:**
- Authentication is not required at CLI startup
- The CLI defers token validation until the moment a command attempts to interact with an AWS resource (API Gateway / S3)
- Commands that operate only on local state (e.g., `scaffold link`, `scaffold config get`) never trigger authentication

**Auth flow (first use or expired token):**
1. CLI detects a missing or unrefreshable token when an S3 operation is about to be made
2. CLI initiates the device authorization flow: prints a URL and user code, then polls for completion
3. Engineer opens the URL in a browser, authenticates with the org IdP, and approves the device
4. CLI receives and securely stores the token, then proceeds with the original command

**VPN enforcement:**
- The API Gateway custom authorizer rejects requests that do not originate from the company VPN, in addition to validating the JWT
- Engineers on the public internet cannot interact with S3 regardless of token validity

---

### `scaffold pull`

Pulls skills from S3 into `~/.scaffold/`.

```bash
scaffold pull              # pull all skills in the remote bucket
scaffold pull <name>       # pull a specific skill by name
```

**Behavior:**
- Normalizes `<name>` (same rules as `scaffold new`)
- Resolves which skills to pull:
  - With `<name>`: pulls that single skill
  - Without `<name>`: lists all top-level prefixes in the bucket and pulls each one
- For each skill being pulled:
  - Lists all objects under the `<name>/` prefix in S3
  - Downloads each object to `~/.scaffold/<name>/`, creating directories as needed
  - Overwrites any existing local files (last-write-wins)
  - Deletes local files under `~/.scaffold/<name>/` that no longer exist in S3 (sync semantics — mirrors push behavior)
- Always pulls fresh from S3 — no caching

**Validation:**
- With `<name>`: exits with error if `<name>/SKILL.md` does not exist in S3 (skill not found)
- Without `<name>`: exits with an informational message if the bucket is empty (no skills found)

**Output:**
- Prints each skill pulled and the count of files downloaded
- `--verbose` flag prints each individual file path as it is downloaded

**Flags:**
- `--verbose` / `-v`: print each file downloaded

### `scaffold edit <name>`

Opens the specified skill for editing. Pulls from S3 first to ensure the local copy is current.

```bash
scaffold edit <name>
```

**Behavior:**
- Normalizes `<name>` (same rules as `scaffold new`)
- Exits with error if the skill does not exist in S3
- Pulls the skill from S3 into `~/.scaffold/<name>/` (identical to `scaffold pull <name>`)
- Resolves the editor (see Editor Resolution below)
- Opens the skill directory or `SKILL.md` (see What Is Opened below) and waits for the editor to exit
- Does **not** push automatically after editing; use `scaffold push` to sync changes back

**Editor Resolution (in priority order):**
1. `$VISUAL` environment variable
2. `$EDITOR` environment variable
3. First of the following found on `PATH`: `code`, `cursor`, `zed`, `windsurf`, `nvim`, `vim`, `vi`
4. If none found: exits with `"No editor found. Set the EDITOR environment variable."`

**What Is Opened:**
- **Directory-aware editors** — opens `~/.scaffold/<name>/` (the full skill directory):
  `code`, `cursor`, `zed`, `windsurf`, `nvim`, `vim`, `emacs`
- **All other editors** — opens `~/.scaffold/<name>/SKILL.md` directly

The rationale: directory-aware editors show a file tree sidebar or file browser, making it natural to navigate between `SKILL.md` and reference files. Editors like `nano` or `helix` do not handle directories meaningfully, so opening `SKILL.md` directly is more useful.

**Flags:**
- `--verbose` / `-v`: passed through to the internal pull step (prints each downloaded file)


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
| CLI | JWT via Instructure identity service (device authorization OIDC flow); token stored in OS credential store |
| API Gateway | Custom authorizer validates JWT + enforces VPN origin |
| S3 bucket | API Gateway IAM role; engineers never access S3 directly |

---

## CLI & Web App Independence

The CLI and web app do not share a library. Each has its own implementation of operations like "create new project." This is an intentional simplification — shared packages add coordination overhead that is not justified until there is a clear, proven need.

---

## Build Sequence

| Phase | Deliverable | Unlocks |
|---|---|---|
| 1 | S3 bucket + skill structure; create `platform` skill with real ADRs | Validates format end-to-end |
| 2 | `scaffold init` + `scaffold pull` + `scaffold link` | Unblocks engineers immediately |
| 3 | `scaffold list` | Local and remote skill discovery |
| 4 | `scaffold push` | Local editing workflow for engineers |
| 5 | Web editor (browse + edit first; new skill wizard second pass) | Non-technical contributor on-ramp |
| 6 | `depends-on` auto-pull | Useful once the skill graph has enough nodes |
