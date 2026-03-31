output "repository_name" {
  value       = var.repository
  description = "Repository name"
}

output "repository_full_name" {
  value       = var.create_repository ? github_repository.skills[0].full_name : "${var.github_owner}/${var.repository}"
  description = "Full repository name (owner/repo)"
}

output "ssh_clone_url" {
  value       = var.create_repository ? github_repository.skills[0].ssh_clone_url : "git@github.com:${var.github_owner}/${var.repository}.git"
  description = "SSH clone URL. Set as skills_repo in ~/.scaffold/settings.json and as SCAFFOLD_SKILLS_REPO in the hook."
}

output "html_url" {
  value       = var.create_repository ? github_repository.skills[0].html_url : "https://github.com/${var.github_owner}/${var.repository}"
  description = "Web URL of the skills repository"
}
