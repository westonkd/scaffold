output "oidc_provider_arn" {
  value       = aws_iam_openid_connect_provider.github.arn
  description = "ARN of the GitHub Actions OIDC provider. Pass to modules/iam as github_oidc_provider_arn."
}
