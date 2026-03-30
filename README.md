# Scaffold

Scaffold is an open-source platform for managing AI context artifacts — PRDs, ADRs, tech plans, design docs — across an engineering organization. It gives these artifacts a standard format and a shared home that is accessible to both humans and AI agents, without ever committing them to source repositories.

## How it works

Skills (the artifact format) live in a dedicated internal git repository. A CI job validates every push and syncs the result to S3. Engineers get skills distributed into their repos automatically via a git hook — no additional tooling or credentials required.

```
Skills git repo → CI (validate + sync) → S3 bucket
                                              ↓
                                   git hook (post-merge)
                                              ↓
                               .claude/skills/_<name>/  (symlinked, gitignored)
```

Three audiences interact with skills through distinct paths:

| Audience | Path |
|---|---|
| **Engineers** | Clone the skills repo; `git push` to publish. A `post-merge` hook in service repos distributes skills automatically on `git pull`. |
| **PMs / Designers** | Edit via the GitHub web editor (v1) or a purpose-built web editor (v2). |
| **Cloud agents** | Read directly from S3 using an IAM role. |

## Repository layout

```
scaffold/
├── hooks/
│   └── post-merge        # Git hook for engineer skill distribution
├── install.sh            # Hook installer for service repos
├── terraform/
│   ├── modules/
│   │   ├── s3/           # S3 bucket module
│   │   └── iam/          # IAM roles module
│   └── examples/
│       └── scaffold/     # Example root module
└── docs/
    ├── hook.md           # Hook reference
    ├── skill-format.md   # Skill format reference
    └── terraform.md      # Infrastructure reference
```

## Documentation

- [Skills repository](docs/skills-repo.md) — layout of the skills git repo, CI setup, branch protection, contribution workflow
- [Hook reference](docs/hook.md) — how the `post-merge` hook works, installation, and `scaffold.json`
- [Skill format](docs/skill-format.md) — skill directory layout, `SKILL.md` frontmatter, naming rules
- [Terraform modules](docs/terraform.md) — S3 and IAM module reference
- [PRD](docs/prd.md) — full product requirements and architecture decisions

## License

MIT
