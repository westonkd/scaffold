variable "repository" {
  type        = string
  description = "Name of the GitHub skills repository"
}

variable "github_owner" {
  type        = string
  description = "GitHub organization or user name that owns the repository"
}

variable "create_repository" {
  type        = bool
  default     = true
  description = "Whether to create the repository. Set to false to manage branch protection on an existing repository."
}

variable "description" {
  type        = string
  default     = "Agent skills repository"
  description = "Repository description. Used only when create_repository = true."
}

variable "required_status_checks" {
  type        = list(string)
  default     = ["validate-and-sync"]
  description = "GitHub Actions status check context names required to pass before merge"
}

variable "tags" {
  type        = map(string)
  default     = {}
  description = "Map whose keys are applied as GitHub repository topics. Values are unused."
}
