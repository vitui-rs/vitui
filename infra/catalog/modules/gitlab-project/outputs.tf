output "project_path_file" {
  description = "File holding the created project's full path."
  value       = local.out_project_path
}

output "token_file" {
  description = "File holding the push token. Read by the Makefile; deliberately not a Terraform output, so it never enters state."
  value       = local.out_token
}

output "web_url" {
  description = "The project's page in the GitLab UI."
  value       = "${var.external_url}/root/${var.project_path}"
}

output "pipelines_url" {
  description = "Where the three jobs are watched going green."
  value       = "${var.external_url}/root/${var.project_path}/-/pipelines"
}
