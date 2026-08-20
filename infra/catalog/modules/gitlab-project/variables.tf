variable "gitlab_container" {
  description = "Name of the running GitLab container to bootstrap through."
  type        = string
}

variable "gitlab_ready" {
  description = "Readiness handle from the gitlab module. Threaded through as a variable so the dependency is on GitLab *answering*, not on its container existing."
  type        = string
}

variable "project_path" {
  description = "Path of the project to create under the `root` namespace."
  type        = string
}

variable "token_name" {
  description = "Name of the personal access token issued for pushing. Reissued on every apply."
  type        = string
  default     = "local-ci-push"
}

variable "local_dir" {
  description = "Host directory for the bootstrap artifacts — the project path and the push token. Gitignored: the token is a credential, and it must not live in Terraform state either."
  type        = string
}

variable "external_url" {
  description = "The URL a browser on the host uses, for composing the push remote."
  type        = string
}
