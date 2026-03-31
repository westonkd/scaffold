# Terraform Reference

Four reusable modules cover the core infrastructure. All are independently deployable.

---

## `modules/github`

Creates the skills GitHub repository with branch protection, required CI status checks, auto-merge, and auto-delete of merged branches.

### Resources

- `github_repository` — private skills repository with auto-merge and delete-branch-on-merge enabled (created only when `create_repository = true`)
- `data.github_repository` — references an existing repository (used when `create_repository = false`)
- `github_branch_protection` — protects `main`: requires CI status checks to pass, disallows force-pushes and deletions, enforces rules for admins

### Variables

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `repository` | `string` | Yes | — | Name of the GitHub skills repository. |
| `github_owner` | `string` | Yes | — | GitHub organization or user name. |
| `create_repository` | `bool` | No | `true` | Whether to create the repository. Set to `false` to manage branch protection on an existing repo. |
| `description` | `string` | No | `"Agent skills repository"` | Repository description. Used only when `create_repository = true`. |
| `required_status_checks` | `list(string)` | No | `["validate-and-sync"]` | Status check context names required to pass before merge. Must match the `name` of the GitHub Actions job. |
| `tags` | `map(string)` | No | `{}` | Map whose keys are applied as GitHub repository topics. |

### Outputs

| Name | Description |
|---|---|
| `repository_name` | Repository name |
| `repository_full_name` | Full repository name (`owner/repo`) |
| `ssh_clone_url` | SSH clone URL. Set as `skills_repo` in `~/.scaffold/settings.json` during onboarding. |
| `html_url` | Web URL of the skills repository |

### Provider configuration

The module requires the `integrations/github` provider. Configure it in the root module:

```hcl
provider "github" {
  owner = var.github_owner
}
```

Set `GITHUB_TOKEN` in the environment before running `terraform apply`.

---

## `modules/github-oidc`

Creates the GitHub Actions OIDC provider in AWS IAM. This is account-level infrastructure — only one provider per AWS account is allowed. Providing it as a standalone module lets adopters skip it if the provider already exists (e.g., managed elsewhere in their Terraform).

### Resources

- `aws_iam_openid_connect_provider` — OIDC provider for `https://token.actions.githubusercontent.com`

### Variables

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `tags` | `map(string)` | No | `{}` | Resource tags. |

### Outputs

| Name | Description |
|---|---|
| `oidc_provider_arn` | ARN of the GitHub OIDC provider. Pass to `modules/iam` as `github_oidc_provider_arn`. |

---

## `modules/s3`

Creates the S3 bucket used as the validated read-replica for skills.

### Resources

- `aws_s3_bucket` — private bucket
- `aws_s3_bucket_versioning` — versioning enabled (safety net for accidental CI overwrites)
- `aws_s3_bucket_server_side_encryption_configuration` — SSE-S3 by default; SSE-KMS when `kms_key_arn` is set
- `aws_s3_bucket_public_access_block` — all public access blocked
- `aws_s3_bucket_policy` — created only when `allowed_role_arns` is non-empty

### Variables

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `bucket_name` | `string` | Yes | — | Globally unique S3 bucket name. Must match `[a-z0-9][a-z0-9\-]{1,61}[a-z0-9]`. |
| `allowed_role_arns` | `list(string)` | No | `[]` | IAM role ARNs granted access via bucket policy. No policy is created when empty. |
| `kms_key_arn` | `string` | No | `null` | KMS key ARN for SSE-KMS. If null, SSE-S3 is used. |
| `force_destroy` | `bool` | No | `false` | Allow Terraform to destroy the bucket even when it contains objects. Non-production only. |
| `tags` | `map(string)` | No | `{}` | Resource tags. |

### Outputs

| Name | Description |
|---|---|
| `bucket_name` | Name of the S3 bucket |
| `bucket_arn` | ARN of the S3 bucket |

---

## `modules/iam`

Creates IAM roles for the three principals that access the bucket.

### Roles created

| Role | Purpose | Permissions |
|---|---|---|
| `<prefix>_cli_rw` | CI (GitHub Actions) — syncs skills to S3 on every push to the skills repo | `GetObject`, `PutObject`, `PutObjectTagging`, `DeleteObject`, `ListBucket` |
| `<prefix>_cli_ro` | Cloud agents — read skills at runtime | `GetObject`, `ListBucket` |
| `<prefix>_web` | Web app backend — serves browse and edit views | `GetObject`, `PutObject`, `PutObjectTagging`, `DeleteObject`, `ListBucket` |

`cli_ro` is only created when `create_readonly_role = true`. `web` is only created when `web_trusted_principal_arns` is non-empty.

### Variables

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| `bucket_arn` | `string` | Yes | — | ARN of the S3 bucket (from `module.s3.bucket_arn`). |
| `name_prefix` | `string` | No | `"scaffold"` | Prefix for role and policy names. |
| `cli_trusted_principal_arns` | `list(string)` | No | `[]` | IAM principals that may assume the `cli_rw` role via `sts:AssumeRole`. Optional when `github_oidc_subjects` is set. |
| `github_oidc_provider_arn` | `string` | No | `null` | ARN of the GitHub Actions OIDC provider (from `modules/github-oidc`). When set alongside `github_oidc_subjects`, adds federated trust to the `cli_rw` role. |
| `github_oidc_subjects` | `list(string)` | No | `[]` | OIDC `sub` claims trusted by the `cli_rw` role. Example: `["repo:your-org/agent-skills:ref:refs/heads/main"]`. Supports wildcards. At least one of `cli_trusted_principal_arns` or `github_oidc_subjects` must be non-empty. |
| `web_trusted_principal_arns` | `list(string)` | No | `[]` | Principals that may assume the `web` role. When empty, the role is not created. |
| `create_readonly_role` | `bool` | No | `false` | Whether to create the `cli_ro` role for cloud agents. |
| `kms_key_arn` | `string` | No | `null` | KMS key ARN. When set, KMS permissions are added to the `cli_rw` and `web` policies. |
| `max_session_duration` | `number` | No | `3600` | Maximum session duration in seconds (900–43200). |
| `tags` | `map(string)` | No | `{}` | Resource tags. |

### Outputs

| Name | Description |
|---|---|
| `cli_rw_role_arn` | ARN of the CI read/write role |
| `cli_ro_role_arn` | ARN of the cloud agent read-only role (`null` if not created) |
| `web_role_arn` | ARN of the web app role (`null` if not created) |

---

## Example root module

`terraform/examples/scaffold/` wires all four modules together into a deployable configuration. The bucket policy is applied at the root level (not inside `modules/s3`) to avoid a circular dependency between the S3 and IAM modules.

### Usage

```hcl
module "github" {
  source       = "../../modules/github"
  repository   = var.skills_repository_name
  github_owner = var.github_owner
  tags         = var.tags
}

module "github_oidc" {
  source = "../../modules/github-oidc"
  tags   = var.tags
}

module "s3" {
  source      = "../../modules/s3"
  bucket_name = var.bucket_name
}

module "iam" {
  source                     = "../../modules/iam"
  bucket_arn                 = module.s3.bucket_arn
  github_oidc_provider_arn   = module.github_oidc.oidc_provider_arn
  github_oidc_subjects       = var.github_oidc_subjects
  create_readonly_role       = length(var.agent_trusted_principal_arns) > 0
  web_trusted_principal_arns = var.web_trusted_principal_arns
}
```

### Example `terraform.tfvars`

```hcl
bucket_name = "acme-scaffold-skills"

github_oidc_subjects = [
  "repo:your-org/agent-skills:ref:refs/heads/main"
]

agent_trusted_principal_arns = [
  "arn:aws:iam::123456789012:role/ecs-task-execution"
]

web_trusted_principal_arns = [
  "arn:aws:iam::123456789012:role/scaffold-web-app"
]

tags = {
  project = "scaffold"
  env     = "production"
}
```

---

## Self-hosting checklist

1. Create the skills repository and branch protection (`modules/github`) — set `GITHUB_TOKEN` before applying
2. Create the GitHub Actions OIDC provider (`modules/github-oidc`) — once per AWS account
3. Create an S3 bucket (`modules/s3`)
4. Create IAM roles (`modules/iam`):
   - `github_oidc_provider_arn` + `github_oidc_subjects` → CI (GitHub Actions) federated access
   - `agent_trusted_principal_arns` → your cloud agent compute role(s) (optional)
   - `web_trusted_principal_arns` → your web app backend role (v2 only, optional)
5. Set the `ci_rw_role_arn` output as `CI_ROLE_ARN` in your skills repo's GitHub Actions secrets
6. Add `ci/validate.py` and `ci/build-index.py` from this repo to your skills repo's `ci/` directory
7. Set up CI to validate and sync on every push to `main` (see [Skills Repository](skills-repo.md) — CI: validate and sync)
8. Engineer distribution: the skills repo is a Claude Code plugin marketplace — engineers run `/plugin marketplace add` + `/plugin install`. Distribute `hooks/post-merge` via `install.sh` or a `postinstall` npm script as a fallback for environments without Claude Code.
