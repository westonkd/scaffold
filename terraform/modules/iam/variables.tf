variable "bucket_arn" {
  type        = string
  description = "S3 bucket ARN from the s3 module output"
}

variable "name_prefix" {
  type        = string
  default     = "scaffold"
  description = "Prefix for IAM role and policy names (e.g. 'scaffold' produces 'scaffold_cli_rw')"
}

variable "cli_trusted_principal_arns" {
  type        = list(string)
  description = "Existing org IAM principals that may assume the CLI roles"

  validation {
    condition     = length(var.cli_trusted_principal_arns) > 0
    error_message = "cli_trusted_principal_arns must contain at least one principal ARN."
  }
}

variable "web_trusted_principal_arns" {
  type        = list(string)
  description = "Principals that may assume the web app role. When empty, the web role is not created."
  default     = []
}

variable "create_readonly_role" {
  type        = bool
  default     = false
  description = "Whether to create the scaffold_cli_ro read-only role"
}

variable "kms_key_arn" {
  type        = string
  default     = null
  description = "KMS key ARN used for the bucket. When set, KMS permissions are added to the cli_rw and web policies."
}

variable "max_session_duration" {
  type        = number
  default     = 3600
  description = "Maximum session duration in seconds for assumed roles (900–43200)."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Resource tags"
}
