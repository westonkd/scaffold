locals {
  bucket_object_arn = "${var.bucket_arn}/*"
}

# --- scaffold_cli_rw ---

data "aws_iam_policy_document" "cli_rw_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "AWS"
      identifiers = var.cli_trusted_principal_arns
    }
  }
}

resource "aws_iam_role" "cli_rw" {
  name               = "scaffold_cli_rw"
  assume_role_policy = data.aws_iam_policy_document.cli_rw_assume.json
  tags               = var.tags
}

data "aws_iam_policy_document" "cli_rw" {
  statement {
    effect = "Allow"
    actions = [
      "s3:GetObject",
      "s3:PutObject",
      "s3:DeleteObject",
    ]
    resources = [local.bucket_object_arn]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:ListBucket"]
    resources = [var.bucket_arn]
  }
}

resource "aws_iam_policy" "cli_rw" {
  name   = "scaffold_cli_rw"
  policy = data.aws_iam_policy_document.cli_rw.json
  tags   = var.tags
}

resource "aws_iam_role_policy_attachment" "cli_rw" {
  role       = aws_iam_role.cli_rw.name
  policy_arn = aws_iam_policy.cli_rw.arn
}

# --- scaffold_cli_ro (optional) ---

data "aws_iam_policy_document" "cli_ro_assume" {
  count = var.create_readonly_role ? 1 : 0

  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "AWS"
      identifiers = var.cli_trusted_principal_arns
    }
  }
}

resource "aws_iam_role" "cli_ro" {
  count              = var.create_readonly_role ? 1 : 0
  name               = "scaffold_cli_ro"
  assume_role_policy = data.aws_iam_policy_document.cli_ro_assume[0].json
  tags               = var.tags
}

data "aws_iam_policy_document" "cli_ro" {
  count = var.create_readonly_role ? 1 : 0

  statement {
    effect    = "Allow"
    actions   = ["s3:GetObject"]
    resources = [local.bucket_object_arn]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:ListBucket"]
    resources = [var.bucket_arn]
  }
}

resource "aws_iam_policy" "cli_ro" {
  count  = var.create_readonly_role ? 1 : 0
  name   = "scaffold_cli_ro"
  policy = data.aws_iam_policy_document.cli_ro[0].json
  tags   = var.tags
}

resource "aws_iam_role_policy_attachment" "cli_ro" {
  count      = var.create_readonly_role ? 1 : 0
  role       = aws_iam_role.cli_ro[0].name
  policy_arn = aws_iam_policy.cli_ro[0].arn
}

# --- scaffold_web ---

data "aws_iam_policy_document" "web_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "AWS"
      identifiers = var.web_trusted_principal_arns
    }
  }
}

resource "aws_iam_role" "web" {
  name               = "scaffold_web"
  assume_role_policy = data.aws_iam_policy_document.web_assume.json
  tags               = var.tags
}

data "aws_iam_policy_document" "web" {
  statement {
    effect = "Allow"
    actions = [
      "s3:GetObject",
      "s3:PutObject",
      "s3:DeleteObject",
    ]
    resources = [local.bucket_object_arn]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:ListBucket"]
    resources = [var.bucket_arn]
  }
}

resource "aws_iam_policy" "web" {
  name   = "scaffold_web"
  policy = data.aws_iam_policy_document.web.json
  tags   = var.tags
}

resource "aws_iam_role_policy_attachment" "web" {
  role       = aws_iam_role.web.name
  policy_arn = aws_iam_policy.web.arn
}
