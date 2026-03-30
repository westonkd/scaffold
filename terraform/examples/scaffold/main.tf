terraform {
  required_version = ">= 1.3"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0"
    }
  }

  # Uncomment and configure to store state remotely:
  # backend "s3" {
  #   bucket = "your-terraform-state-bucket"
  #   key    = "scaffold/terraform.tfstate"
  #   region = "us-east-1"
  # }
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  type        = string
  default     = "us-west-2"
  description = "AWS region to deploy into"
}

variable "bucket_name" {
  type        = string
  description = "Globally unique S3 bucket name for Scaffold storage"
}

variable "name_prefix" {
  type        = string
  default     = "scaffold"
  description = "Prefix for IAM role and policy names"
}

variable "ci_trusted_principal_arns" {
  type        = list(string)
  default     = []
  description = "IAM principal ARNs for CI that may assume the read/write role via sts:AssumeRole. Optional when github_oidc_subjects is set."
}

variable "github_oidc_subjects" {
  type        = list(string)
  description = "GitHub OIDC sub claims for CI. Example: [\"repo:your-org/agent-skills:ref:refs/heads/main\"]. Creates a github-oidc provider and configures federated trust on the ci_rw role."
}

variable "agent_trusted_principal_arns" {
  type        = list(string)
  default     = []
  description = "IAM principal ARNs for cloud agents — granted read-only S3 access. When empty, the read-only role is not created."
}

variable "web_trusted_principal_arns" {
  type        = list(string)
  default     = []
  description = "IAM principal ARNs for the web app backend — granted read-only S3 access. When empty, the web role is not created."
}

variable "kms_key_arn" {
  type        = string
  default     = null
  description = "Optional KMS key ARN for SSE-KMS bucket encryption. If null, SSE-S3 is used."
}

variable "max_session_duration" {
  type        = number
  default     = 3600
  description = "Maximum session duration in seconds for assumed roles (900–43200)"
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources"
}

module "github_oidc" {
  source = "../../modules/github-oidc"
  tags   = var.tags
}

# The s3 module is called without allowed_role_arns here because the role ARNs
# are not yet known — they are created by the iam module below. The bucket
# policy is applied at root level after both modules resolve, avoiding the
# circular dependency.
module "s3" {
  source = "../../modules/s3"

  bucket_name = var.bucket_name
  kms_key_arn = var.kms_key_arn
  tags        = var.tags
}

module "iam" {
  source = "../../modules/iam"

  bucket_arn                 = module.s3.bucket_arn
  name_prefix                = var.name_prefix
  cli_trusted_principal_arns = var.ci_trusted_principal_arns
  github_oidc_provider_arn   = module.github_oidc.oidc_provider_arn
  github_oidc_subjects       = var.github_oidc_subjects
  web_trusted_principal_arns = var.web_trusted_principal_arns
  create_readonly_role       = length(var.agent_trusted_principal_arns) > 0
  kms_key_arn                = var.kms_key_arn
  max_session_duration       = var.max_session_duration
  tags                       = var.tags
}

locals {
  allowed_role_arns = compact([
    module.iam.cli_rw_role_arn,
    module.iam.cli_ro_role_arn,
    module.iam.web_role_arn,
  ])
}

data "aws_iam_policy_document" "bucket_policy" {
  statement {
    sid    = "AllowRoleListBucket"
    effect = "Allow"

    principals {
      type        = "AWS"
      identifiers = local.allowed_role_arns
    }

    actions   = ["s3:ListBucket", "s3:GetBucketLocation"]
    resources = [module.s3.bucket_arn]
  }

  statement {
    sid    = "AllowRoleObjectAccess"
    effect = "Allow"

    principals {
      type        = "AWS"
      identifiers = local.allowed_role_arns
    }

    actions   = ["s3:GetObject", "s3:PutObject", "s3:PutObjectTagging", "s3:DeleteObject"]
    resources = ["${module.s3.bucket_arn}/*"]
  }
}

resource "aws_s3_bucket_policy" "scaffold" {
  bucket = module.s3.bucket_name
  policy = data.aws_iam_policy_document.bucket_policy.json

  depends_on = [module.s3]
}

output "bucket_name" {
  value = module.s3.bucket_name
}

output "bucket_arn" {
  value = module.s3.bucket_arn
}

output "ci_rw_role_arn" {
  value       = module.iam.cli_rw_role_arn
  description = "Assume this role in CI (GitHub Actions) to sync skills to S3"
}

output "agent_ro_role_arn" {
  value       = module.iam.cli_ro_role_arn
  description = "Assume this role in cloud agent compute environments for read-only S3 access"
}

output "web_role_arn" {
  value       = module.iam.web_role_arn
  description = "Assume this role in the web app backend for read-only S3 access"
}
