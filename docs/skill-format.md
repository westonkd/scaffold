# Skill Format Reference

Skills follow the [Agent Skills](https://agentskills.io) format. Each skill is a directory of plain files — never archived or zipped. The format provides progressive disclosure: only the description (~100 tokens) loads at agent startup; the full `SKILL.md` body and reference files load on demand.

In the skills repo, each skill is wrapped in a Claude Code plugin. The plugin carries the skill's metadata (`category`, `keywords`, `scope`, etc.) in a structured `plugin.json`; `SKILL.md` contains only what is needed for agent loading.

---

## Directory layout

```
plugins/payments/                         ← plugin root
├── .claude-plugin/
│   └── plugin.json                       ← metadata and discovery fields
└── skills/
    └── payments/                         ← skill root (synced to S3)
        ├── SKILL.md                      ← executive summary + description frontmatter
        ├── references/
        │   ├── prd.md
        │   ├── tech-plan.md
        │   └── adrs/
        │       ├── payment-processor-selection.md
        │       └── retry-strategy.md
        └── assets/
            └── architecture-diagram.png
```

`SKILL.md` is a lean orientation document. Detailed content lives in `references/`. Keep individual ADRs as separate files so agents only load what they need.

---

## `SKILL.md` frontmatter

Only `description` is required. All rich metadata lives in `plugin.json`.

```yaml
---
description: >
  Context for the payments platform. Use when working on payment processing,
  billing, or any integration with payment providers.
---
```

### Fields

| Field | Required | Description |
|---|---|---|
| `description` | Yes | One or two sentences. This is loaded at agent startup — make it agent-useful. |

---

## `plugin.json` schema

`plugin.json` lives at `plugins/<name>/.claude-plugin/plugin.json` and carries all metadata fields. It is the authoritative source for skill discovery, indexing, and validation.

```json
{
  "name": "payments",
  "description": "Context for the payments platform. Use when working on payment processing, billing, or any integration with payment providers.",
  "version": "1.0.0",
  "category": "project",
  "status": "active",
  "keywords": ["payments", "billing", "fintech"],
  "scope": ["repo-payments", "repo-billing"],
  "default": false
}
```

### Fields

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Skill name. Must match `[a-z0-9][a-z0-9-]*`. Must equal the plugin directory name. |
| `description` | Yes | One or two sentences. Copied into `_index.json` and marketplace listings. |
| `version` | Yes | Semantic version string (e.g. `"1.0.0"`). |
| `category` | Yes | `project` for team/product context; `platform` for org-wide standards; `utility` for agent infrastructure knowledge. |
| `status` | No | Default `active`. Skills are removed by deleting them from the skills repo. |
| `keywords` | No | Array of strings. Mirrored as the `tags` field in `_index.json` and as S3 object tags on `SKILL.md`. |
| `scope` | No | Array of repo names this skill is relevant to. |
| `default` | No | Boolean. If `true`, symlinked into every service repo automatically regardless of `scaffold.json`. |

---

## Naming rules

- Characters: alphanumeric and hyphens (`[a-z0-9][a-z0-9-]*`)
- Lowercase only
- Leading/trailing whitespace stripped on input
- Spaces in input normalized to hyphens (e.g., `my payments platform` → `my-payments-platform`)
- Must be unique within the organization's skills repo
- Plugin directory name, `plugin.json` `name` field, and skill subdirectory name must all match

---

## The platform skill

Cross-cutting standards and org-wide ADRs live in a plugin named `platform` with `"category": "platform"` in `plugin.json`. To include it in a service repo, add `"platform"` to `scaffold.json`:

```json
{
  "skills": ["platform", "payments"]
}
```

---

## S3 storage layout

CI extracts skills from the plugin structure — taking `plugins/<name>/skills/<name>/` — and syncs the flat extracted tree to S3. The bucket layout mirrors those extracted skill directories:

```
s3://<bucket>/
  _index.json           ← manifest rebuilt by CI on every sync
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
```

Each file within a skill is a separate S3 object. CI uses `s3 sync --delete` semantics so the bucket always reflects the current state of `plugins/` in the skills repo.

---

## `_index.json`

The manifest file at the bucket root. Rebuilt by CI on every successful sync by reading every `plugin.json`. Used by the web app and cloud agents for skill discovery without fetching individual `SKILL.md` files.

```json
{
  "version": 1,
  "updated_at": "2026-03-30T00:00:00Z",
  "skills": [
    {
      "name": "payments",
      "description": "Context for the payments platform.",
      "type": "project",
      "status": "active",
      "tags": ["payments", "billing", "fintech"],
      "scope": ["repo-payments", "repo-billing"],
      "updated_at": "2026-03-30T00:00:00Z"
    }
  ]
}
```

`_index.json` is a reserved key. The `[a-z0-9][a-z0-9-]*` naming rule prevents any skill from conflicting with it.
