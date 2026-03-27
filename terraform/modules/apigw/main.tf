terraform {
  required_version = ">= 1.3"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0"
    }
    null = {
      source  = "hashicorp/null"
      version = ">= 3.0"
    }
  }
}

locals {
  repo_root = abspath("${path.module}/../../..")

  authorizer_zip_path = "${local.repo_root}/lambda/authorizer/dist/authorizer.zip"
  s3_proxy_zip_path   = "${local.repo_root}/lambda/s3-proxy/dist/s3-proxy.zip"

  # Per-Lambda source hashes, computed from source files (always present at
  # plan time, unlike the built zip artifacts).
  authorizer_source_hash = sha256(join("", [
    for f in sort(fileset("${local.repo_root}/lambda/authorizer/src", "**")) :
    filesha256("${local.repo_root}/lambda/authorizer/src/${f}")
  ]))

  s3_proxy_source_hash = sha256(join("", [
    for f in sort(fileset("${local.repo_root}/lambda/s3-proxy/src", "**")) :
    filesha256("${local.repo_root}/lambda/s3-proxy/src/${f}")
  ]))
}

# ── Lambda builds ─────────────────────────────────────────────────────────────

resource "null_resource" "authorizer_build" {
  triggers = {
    src_hash = local.authorizer_source_hash
    pkg_hash = filesha256("${local.repo_root}/lambda/authorizer/package.json")
  }

  provisioner "local-exec" {
    command = <<-CMD
      docker build \
        --target export \
        --output type=local,dest=${local.repo_root}/lambda/authorizer/dist \
        ${local.repo_root}/lambda/authorizer
    CMD
  }
}

resource "null_resource" "s3_proxy_build" {
  triggers = {
    src_hash = local.s3_proxy_source_hash
    pkg_hash = filesha256("${local.repo_root}/lambda/s3-proxy/package.json")
  }

  provisioner "local-exec" {
    command = <<-CMD
      docker build \
        --target export \
        --output type=local,dest=${local.repo_root}/lambda/s3-proxy/dist \
        ${local.repo_root}/lambda/s3-proxy
    CMD
  }
}

# ── CloudWatch log groups ─────────────────────────────────────────────────────

resource "aws_cloudwatch_log_group" "apigw_access" {
  name              = "/aws/apigateway/${var.name_prefix}-scaffold-api"
  retention_in_days = var.log_retention_days
  tags              = var.tags
}

resource "aws_cloudwatch_log_group" "authorizer_lambda" {
  name              = "/aws/lambda/${var.name_prefix}-scaffold-authorizer"
  retention_in_days = var.log_retention_days
  tags              = var.tags
}

resource "aws_cloudwatch_log_group" "s3_proxy_lambda" {
  name              = "/aws/lambda/${var.name_prefix}-scaffold-s3-proxy"
  retention_in_days = var.log_retention_days
  tags              = var.tags
}

# ── IAM: Authorizer Lambda execution role ─────────────────────────────────────

data "aws_iam_policy_document" "lambda_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "authorizer_lambda" {
  name               = "${var.name_prefix}_apigw_authorizer"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume.json
  tags               = var.tags
}

data "aws_iam_policy_document" "authorizer_lambda_logs" {
  statement {
    effect    = "Allow"
    actions   = ["logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.authorizer_lambda.arn}:*"]
  }
}

resource "aws_iam_policy" "authorizer_lambda_logs" {
  name   = "${var.name_prefix}_apigw_authorizer_logs"
  policy = data.aws_iam_policy_document.authorizer_lambda_logs.json
  tags   = var.tags
}

resource "aws_iam_role_policy_attachment" "authorizer_lambda_logs" {
  role       = aws_iam_role.authorizer_lambda.name
  policy_arn = aws_iam_policy.authorizer_lambda_logs.arn
}

# ── IAM: S3 Proxy Lambda execution role ───────────────────────────────────────

resource "aws_iam_role" "s3_proxy_lambda" {
  name               = "${var.name_prefix}_apigw_s3_proxy"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume.json
  tags               = var.tags
}

data "aws_iam_policy_document" "s3_proxy_lambda" {
  statement {
    effect    = "Allow"
    actions   = ["logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.s3_proxy_lambda.arn}:*"]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:GetObject", "s3:PutObject", "s3:PutObjectTagging", "s3:DeleteObject"]
    resources = ["${var.bucket_arn}/*"]
  }

  statement {
    effect    = "Allow"
    actions   = ["s3:ListBucket", "s3:GetBucketLocation"]
    resources = [var.bucket_arn]
  }
}

resource "aws_iam_policy" "s3_proxy_lambda" {
  name   = "${var.name_prefix}_apigw_s3_proxy"
  policy = data.aws_iam_policy_document.s3_proxy_lambda.json
  tags   = var.tags
}

resource "aws_iam_role_policy_attachment" "s3_proxy_lambda" {
  role       = aws_iam_role.s3_proxy_lambda.name
  policy_arn = aws_iam_policy.s3_proxy_lambda.arn
}

# ── IAM: API Gateway CloudWatch logging (account-scoped) ─────────────────────
#
# aws_api_gateway_account is a singleton per AWS account per region. If another
# Terraform deployment in the same account already manages this resource, set
# create_api_gateway_account = false to avoid conflicts.

data "aws_iam_policy_document" "apigw_assume" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["apigateway.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "apigw_cloudwatch" {
  count = var.create_api_gateway_account ? 1 : 0

  name               = "${var.name_prefix}_apigw_cloudwatch"
  assume_role_policy = data.aws_iam_policy_document.apigw_assume.json
  tags               = var.tags
}

resource "aws_iam_role_policy_attachment" "apigw_cloudwatch" {
  count = var.create_api_gateway_account ? 1 : 0

  role       = aws_iam_role.apigw_cloudwatch[0].name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonAPIGatewayPushToCloudWatchLogs"
}

resource "aws_api_gateway_account" "this" {
  count = var.create_api_gateway_account ? 1 : 0

  cloudwatch_role_arn = aws_iam_role.apigw_cloudwatch[0].arn

  depends_on = [aws_iam_role_policy_attachment.apigw_cloudwatch]
}

# ── Lambda: authorizer ────────────────────────────────────────────────────────

resource "aws_lambda_function" "authorizer" {
  function_name = "${var.name_prefix}-scaffold-authorizer"
  role          = aws_iam_role.authorizer_lambda.arn
  filename      = local.authorizer_zip_path
  handler       = "index.handler"
  runtime       = "nodejs22.x"
  timeout       = 10
  memory_size   = 256

  source_code_hash = local.authorizer_source_hash

  environment {
    variables = {
      JWKS_URI        = var.jwks_uri
      JWT_ISSUER      = var.jwt_issuer
      JWT_AUDIENCE    = var.jwt_audience
      VPN_CIDR_BLOCKS = join(",", var.vpn_cidr_blocks)
    }
  }

  logging_config {
    log_group  = aws_cloudwatch_log_group.authorizer_lambda.name
    log_format = "JSON"
  }

  tags = var.tags

  depends_on = [
    null_resource.authorizer_build,
    aws_cloudwatch_log_group.authorizer_lambda,
    aws_iam_role_policy_attachment.authorizer_lambda_logs,
  ]
}

# ── Lambda: S3 proxy ──────────────────────────────────────────────────────────

resource "aws_lambda_function" "s3_proxy" {
  function_name = "${var.name_prefix}-scaffold-s3-proxy"
  role          = aws_iam_role.s3_proxy_lambda.arn
  filename      = local.s3_proxy_zip_path
  handler       = "index.handler"
  runtime       = "nodejs22.x"
  timeout       = 30
  memory_size   = 256

  source_code_hash = local.s3_proxy_source_hash

  environment {
    variables = {
      BUCKET_NAME = var.bucket_name
    }
  }

  logging_config {
    log_group  = aws_cloudwatch_log_group.s3_proxy_lambda.name
    log_format = "JSON"
  }

  tags = var.tags

  depends_on = [
    null_resource.s3_proxy_build,
    aws_cloudwatch_log_group.s3_proxy_lambda,
    aws_iam_role_policy_attachment.s3_proxy_lambda,
  ]
}

# ── API Gateway v2 (HTTP API) ─────────────────────────────────────────────────

resource "aws_apigatewayv2_api" "this" {
  name          = "${var.name_prefix}-scaffold-api"
  protocol_type = "HTTP"
  description   = "Scaffold API Gateway — Lambda S3 proxy with JWT + VPN authorizer"
  tags          = var.tags
}

resource "aws_apigatewayv2_authorizer" "this" {
  api_id                            = aws_apigatewayv2_api.this.id
  authorizer_type                   = "REQUEST"
  authorizer_uri                    = aws_lambda_function.authorizer.invoke_arn
  identity_sources                  = ["$request.header.Authorization"]
  name                              = "${var.name_prefix}-scaffold-authorizer"
  authorizer_payload_format_version = "2.0"
  enable_simple_responses           = true

  # 0 = no caching; every request invokes the Lambda authorizer. Increase to
  # e.g. 300 to cache results for 5 minutes (keyed on Authorization header).
  authorizer_result_ttl_in_seconds = 0
}

resource "aws_lambda_permission" "apigw_authorizer" {
  statement_id  = "AllowAPIGatewayInvokeAuthorizer"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.authorizer.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.this.execution_arn}/*"
}

resource "aws_lambda_permission" "apigw_s3_proxy" {
  statement_id  = "AllowAPIGatewayInvokeProxy"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.s3_proxy.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.this.execution_arn}/*"
}

# ── S3 proxy integration and route ────────────────────────────────────────────

resource "aws_apigatewayv2_integration" "s3_proxy" {
  api_id                 = aws_apigatewayv2_api.this.id
  integration_type       = "AWS_PROXY"
  integration_uri        = aws_lambda_function.s3_proxy.invoke_arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "default" {
  api_id             = aws_apigatewayv2_api.this.id
  route_key          = "$default"
  target             = "integrations/${aws_apigatewayv2_integration.s3_proxy.id}"
  authorizer_id      = aws_apigatewayv2_authorizer.this.id
  authorization_type = "CUSTOM"
}

# ── Stage ─────────────────────────────────────────────────────────────────────

resource "aws_apigatewayv2_stage" "default" {
  api_id      = aws_apigatewayv2_api.this.id
  name        = "$default"
  auto_deploy = true

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.apigw_access.arn
    format = jsonencode({
      requestId       = "$context.requestId"
      ip              = "$context.identity.sourceIp"
      requestTime     = "$context.requestTime"
      httpMethod      = "$context.httpMethod"
      routeKey        = "$context.routeKey"
      status          = "$context.status"
      responseLength  = "$context.responseLength"
      authorizerError = "$context.authorizer.error"
    })
  }

  tags = var.tags

  depends_on = [aws_api_gateway_account.this]
}
