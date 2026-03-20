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
  default     = "us-east-1"
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

variable "cli_trusted_principal_arns" {
  type        = list(string)
  description = "IAM principal ARNs (users or roles) allowed to assume the CLI roles"
}

variable "web_trusted_principal_arns" {
  type        = list(string)
  description = "IAM principal ARNs allowed to assume the web app role"
}

variable "create_readonly_role" {
  type        = bool
  default     = false
  description = "Set to true to also create the scaffold_cli_ro read-only role"
}

variable "kms_key_arn" {
  type        = string
  default     = null
  description = "Optional KMS key ARN for SSE-KMS bucket encryption"
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

# The s3 module is called without allowed_role_arns here because the role ARNs
# are not yet known — they are created by the iam module below. The bucket
# policy is applied in this root module after both modules resolve, avoiding
# the circular dependency.
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
  cli_trusted_principal_arns = var.cli_trusted_principal_arns
  web_trusted_principal_arns = var.web_trusted_principal_arns
  create_readonly_role       = var.create_readonly_role
  kms_key_arn                = var.kms_key_arn
  max_session_duration       = var.max_session_duration
  tags                       = var.tags
}

# Bucket policy applied here (root level) so it can reference both the bucket
# ARN from module.s3 and the role ARNs from module.iam without a cycle.
locals {
  allowed_role_arns = compact([
    module.iam.cli_rw_role_arn,
    module.iam.cli_ro_role_arn,
    module.iam.web_role_arn,
  ])
}

data "aws_iam_policy_document" "bucket_policy" {
  count = length(local.allowed_role_arns) > 0 ? 1 : 0

  statement {
    sid    = "AllowRoleListBucket"
    effect = "Allow"

    principals {
      type        = "AWS"
      identifiers = local.allowed_role_arns
    }

    actions = [
      "s3:ListBucket",
      "s3:GetBucketLocation",
    ]

    resources = [module.s3.bucket_arn]
  }

  statement {
    sid    = "AllowRoleObjectAccess"
    effect = "Allow"

    principals {
      type        = "AWS"
      identifiers = local.allowed_role_arns
    }

    actions = [
      "s3:GetObject",
      "s3:PutObject",
      "s3:DeleteObject",
    ]

    resources = ["${module.s3.bucket_arn}/*"]
  }
}

resource "aws_s3_bucket_policy" "scaffold" {
  count  = length(local.allowed_role_arns) > 0 ? 1 : 0
  bucket = module.s3.bucket_name
  policy = data.aws_iam_policy_document.bucket_policy[0].json

  depends_on = [module.s3]
}

output "bucket_name" {
  value = module.s3.bucket_name
}

output "bucket_arn" {
  value = module.s3.bucket_arn
}

output "cli_rw_role_arn" {
  value = module.iam.cli_rw_role_arn
}

output "cli_ro_role_arn" {
  value = module.iam.cli_ro_role_arn
}

output "web_role_arn" {
  value = module.iam.web_role_arn
}
