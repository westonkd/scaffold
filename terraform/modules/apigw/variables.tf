variable "name_prefix" {
  type        = string
  default     = "scaffold"
  description = "Prefix for all named resources"
}

variable "bucket_name" {
  type        = string
  description = "Name of the target S3 bucket"
}

variable "bucket_arn" {
  type        = string
  description = "ARN of the target S3 bucket"
}

variable "jwks_uri" {
  type        = string
  description = "JWKS endpoint URI for JWT signature verification"
}

variable "jwt_issuer" {
  type        = string
  description = "Expected JWT issuer (iss claim)"
}

variable "jwt_audience" {
  type        = string
  description = "Expected JWT audience (aud claim)"
}

variable "vpn_cidr_blocks" {
  type        = list(string)
  default     = []
  description = "IPv4 CIDR blocks permitted by the VPN check. Empty list disables VPN enforcement."
}

variable "log_retention_days" {
  type        = number
  default     = 30
  description = "CloudWatch log retention in days"
}

variable "create_api_gateway_account" {
  type        = bool
  default     = true
  description = "Whether to manage the aws_api_gateway_account resource. This is account-scoped — only one can exist per region. Set to false if another module or deployment already manages it."
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Tags applied to all resources"
}
