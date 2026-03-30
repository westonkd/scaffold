# Terraform Reference

Two reusable modules cover the core infrastructure. Both are independently deployable.

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
| `cli_trusted_principal_arns` | `list(string)` | Yes | — | Principals that may assume the `cli_rw` role (e.g., GitHub Actions runner role ARN). |
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

`terraform/examples/scaffold/` wires the two modules together into a deployable configuration. The bucket policy is applied at the root level (not inside `modules/s3`) to avoid a circular dependency between the S3 and IAM modules.

### Usage

```hcl
module "s3" {
  source = "../../modules/s3"
  bucket_name = var.bucket_name
}

module "iam" {
  source                     = "../../modules/iam"
  bucket_arn                 = module.s3.bucket_arn
  cli_trusted_principal_arns = var.ci_trusted_principal_arns
  create_readonly_role       = length(var.agent_trusted_principal_arns) > 0
  web_trusted_principal_arns = var.web_trusted_principal_arns
}
```

### Example `terraform.tfvars`

```hcl
bucket_name = "acme-scaffold-skills"

ci_trusted_principal_arns = [
  "arn:aws:iam::123456789012:role/github-actions-runner"
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

1. Create an S3-compatible bucket (`modules/s3`)
2. Create IAM roles (`modules/iam`):
   - `cli_trusted_principal_arns` → your CI runner's IAM role
   - `agent_trusted_principal_arns` → your cloud agent compute role(s)
   - `web_trusted_principal_arns` → your web app backend role (v2 only)
3. Configure the skills git repo with branch protection and required CI status checks (see [PRD](prd.md) — `github` Terraform module, not yet implemented)
4. Set up CI to validate and sync on every push to `main` (see [PRD](prd.md) — CI: Validate and Sync)
5. Distribute `hooks/post-merge` to service repos via `install.sh` or a `postinstall` npm script
