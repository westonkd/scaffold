variable "bucket_arn" {
  type        = string
  description = "S3 bucket ARN from the s3 module output"
}

variable "cli_trusted_principal_arns" {
  type        = list(string)
  description = "Existing org IAM principals that may assume the CLI roles"
}

variable "web_trusted_principal_arns" {
  type        = list(string)
  description = "Principals that may assume the web app role"
}

variable "create_readonly_role" {
  type        = bool
  default     = false
  description = "Whether to create the scaffold_cli_ro read-only role"
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Resource tags"
}
