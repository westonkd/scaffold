# Scaffold

Scaffold is an open-source platform for managing AI context artifacts — PRDs, ADRs, tech plans, design docs — across an engineering organization. It gives these artifacts a standard format and a shared home that is accessible to both humans and AI agents, without ever committing them to source repositories.

## How it works

Skills (the artifact format) live in a dedicated internal git repository structured as a **Claude Code plugin marketplace**. Each project is a plugin; the repo root carries a `marketplace.json` listing every plugin. A CI job validates every push and syncs the extracted skills to S3. Engineers install skills via the Claude Code plugin system — no credentials or extra tooling required.

```
Skills git repo (plugin marketplace)
        ↓ git push
CI: validate plugin.json + SKILL.md → extract skills → sync to S3 → rebuild _index.json
                                                                           ↓
                                                          /plugin install <name>@company-skills
                                                                           ↓
                                                   .claude/skills/_<name>/  (symlinked, gitignored)
```

Three audiences interact with skills through distinct paths:

| Audience | Path |
|---|---|
| **Engineers** | Clone the skills repo; `git push` to publish. Install skills with `/plugin install` via Claude Code; a `post-merge` bash hook is available as a fallback. |
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
