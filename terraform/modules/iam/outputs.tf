output "cli_rw_role_arn" {
  value       = aws_iam_role.cli_rw.arn
  description = "ARN of the scaffold_cli_rw role"
}

output "cli_ro_role_arn" {
  value       = var.create_readonly_role ? aws_iam_role.cli_ro[0].arn : null
  description = "ARN of the scaffold_cli_ro role (null if not created)"
}

output "web_role_arn" {
  value       = length(aws_iam_role.web) > 0 ? aws_iam_role.web[0].arn : null
  description = "ARN of the scaffold_web role (null if not created)"
}
