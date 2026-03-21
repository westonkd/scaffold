terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0"
    }
  }
}

resource "aws_s3_bucket" "this" {
  bucket        = var.bucket_name
  force_destroy = var.force_destroy
  tags          = var.tags
}

resource "aws_s3_bucket_versioning" "this" {
  bucket = aws_s3_bucket.this.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "this" {
  bucket = aws_s3_bucket.this.id

  rule {
    bucket_key_enabled = var.kms_key_arn != null

    apply_server_side_encryption_by_default {
      sse_algorithm     = var.kms_key_arn != null ? "aws:kms" : "AES256"
      kms_master_key_id = var.kms_key_arn
    }
  }
}

resource "aws_s3_bucket_public_access_block" "this" {
  bucket = aws_s3_bucket.this.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

data "aws_iam_policy_document" "bucket_policy" {
  dynamic "statement" {
    for_each = length(var.allowed_role_arns) > 0 ? [1] : []
    content {
      sid    = "AllowRoleListBucket"
      effect = "Allow"
      principals {
        type        = "AWS"
        identifiers = var.allowed_role_arns
      }
      actions = [
        "s3:ListBucket",
        "s3:GetBucketLocation",
      ]
      resources = [aws_s3_bucket.this.arn]
    }
  }

  dynamic "statement" {
    for_each = length(var.allowed_role_arns) > 0 ? [1] : []
    content {
      sid    = "AllowRoleObjectAccess"
      effect = "Allow"
      principals {
        type        = "AWS"
        identifiers = var.allowed_role_arns
      }
      actions = [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
      ]
      resources = ["${aws_s3_bucket.this.arn}/*"]
    }
  }
}

resource "aws_s3_bucket_policy" "this" {
  count  = length(var.allowed_role_arns) > 0 ? 1 : 0
  bucket = aws_s3_bucket.this.id
  policy = data.aws_iam_policy_document.bucket_policy.json

  depends_on = [aws_s3_bucket_public_access_block.this]
}
