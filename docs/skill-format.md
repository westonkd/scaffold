# Skill Format Reference

Skills follow the [Agent Skills](https://agentskills.io) format. Each skill is a directory of plain files — never archived or zipped. The format provides progressive disclosure: only the name and description (~100 tokens) load at agent startup; the full `SKILL.md` body and reference files load on demand.

---

## Directory layout

```
payments/
├── SKILL.md                          # Executive summary + frontmatter
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

```yaml
---
name: payments
description: >
  Context for the payments platform. Use when working on payment processing,
  billing, or any integration with payment providers.
metadata:
  type: project            # project | platform
  scope: repo-payments, repo-billing
  status: active
  tags: payments, billing, fintech
---
```

### Fields

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Skill name. Must be unique within the org. Must match `[a-z0-9][a-z0-9-]*`. |
| `description` | Yes | One or two sentences. This is the only thing loaded at agent startup — make it agent-useful. |
| `metadata.type` | Yes | `project` for team/product context; `platform` for org-wide standards. |
| `metadata.status` | Yes | Always `active`. Skills are removed by deleting them from the skills repo; recovery is via git history and S3 versioning. |
| `metadata.scope` | No | Comma-separated list of repo names this skill is relevant to. |
| `metadata.tags` | No | Comma-separated user-defined labels. Mirrored as S3 object tags on `SKILL.md` at CI sync time. |

---

## Naming rules

- Characters: alphanumeric and hyphens (`[a-z0-9][a-z0-9-]*`)
- Lowercase only
- Leading/trailing whitespace stripped on input
- Spaces in input normalized to hyphens (e.g., `my payments platform` → `my-payments-platform`)
- Must be unique within the organization's skills repo

---

## The platform skill

Cross-cutting standards and org-wide ADRs live in a dedicated skill named `platform`. It uses `type: platform` in frontmatter. To include it in a service repo, add `"platform"` to `scaffold.json`:

```json
{
  "skills": ["platform", "payments"]
}
```

---

## S3 storage layout

CI syncs the `agent-skills/` subdirectory of the skills repo to the S3 bucket root. The bucket layout mirrors the contents of that directory exactly:

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

Each file within a skill is a separate S3 object. CI uses `s3 sync --delete` semantics so the bucket always mirrors `agent-skills/` in the skills repo exactly.

---

## `_index.json`

The manifest file at the bucket root. Rebuilt by CI on every successful sync. Used by the web app and cloud agents for skill discovery without fetching individual `SKILL.md` files.

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
