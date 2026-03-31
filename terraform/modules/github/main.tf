terraform {
  required_providers {
    github = {
      source  = "integrations/github"
      version = ">= 6.0"
    }
  }
}

resource "github_repository" "skills" {
  count = var.create_repository ? 1 : 0

  name                   = var.repository
  description            = var.description
  visibility             = "private"
  allow_auto_merge       = true
  delete_branch_on_merge = true
  topics                 = keys(var.tags)
}

data "github_repository" "skills" {
  count     = var.create_repository ? 0 : 1
  full_name = "${var.github_owner}/${var.repository}"
}

locals {
  repository_node_id = var.create_repository ? github_repository.skills[0].node_id : data.github_repository.skills[0].node_id
}

resource "github_branch_protection" "main" {
  repository_id = local.repository_node_id
  pattern       = "main"

  enforce_admins      = true
  allows_deletions    = false
  allows_force_pushes = false

  required_status_checks {
    strict   = true
    contexts = var.required_status_checks
  }
}
