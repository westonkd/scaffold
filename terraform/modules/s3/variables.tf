variable "bucket_name" {
  type        = string
  description = "Globally unique S3 bucket name"
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

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Resource tags"
}
