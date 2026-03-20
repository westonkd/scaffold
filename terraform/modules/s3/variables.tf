variable "bucket_name" {
  type        = string
  description = "Globally unique S3 bucket name"

  validation {
    condition     = can(regex("^[a-z0-9][a-z0-9\\-]{1,61}[a-z0-9]$", var.bucket_name))
    error_message = "bucket_name must be 3-63 characters, start and end with a lowercase letter or digit, and contain only lowercase letters, digits, and hyphens."
  }
}

variable "allowed_role_arns" {
  type        = list(string)
  default     = []
  description = "IAM role ARNs granted bucket access via bucket policy. If empty, no bucket policy is created."
}

variable "kms_key_arn" {
  type        = string
  default     = null
  description = "KMS key ARN for SSE-KMS encryption. If null, SSE-S3 is used."
}

variable "force_destroy" {
  type        = bool
  default     = false
  description = "Allow Terraform to destroy the bucket even if it contains objects. Set to true only in non-production environments."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Resource tags"
}
