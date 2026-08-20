# The project this repository is pushed to, and the token it is pushed with. Not part of the CI
# host itself — it is the bootstrap that makes the host reachable from a working copy, and it lives
# on the stack so that one `apply` leaves nothing to do by hand.

locals {
  # `env.hcl` sits in the stack directory, one level above the generated units. `local_dir` is
  # derived from its path rather than from `get_terragrunt_dir()`, so that it points at the stack
  # directory and survives `terragrunt stack generate`, which rewrites everything below it.
  env_file  = find_in_parent_folders("env.hcl")
  env       = read_terragrunt_config(local.env_file).locals
  local_dir = "${dirname(local.env_file)}/.local"
}

dependency "gitlab" {
  config_path = "../gitlab"

  mock_outputs = {
    container_name = "gitlab"
    external_url   = "http://localhost:8929"
    ready          = "not-yet"
  }
  mock_outputs_allowed_terraform_commands = ["validate", "plan"]
}

terraform {
  source = "${get_terragrunt_dir()}/../../../../../catalog/modules/gitlab-project"
}

remote_state {
  backend = "local"

  config = {
    path = "${get_terragrunt_dir()}/terraform.tfstate"
  }
}

generate "backend" {
  path      = "backend.tf"
  if_exists = "overwrite"

  contents = <<-EOT
    terraform {
      backend "local" {}
    }
  EOT
}

inputs = {
  gitlab_container = dependency.gitlab.outputs.container_name
  gitlab_ready     = dependency.gitlab.outputs.ready
  external_url     = dependency.gitlab.outputs.external_url
  project_path     = local.env.project_path
  local_dir        = local.local_dir
}
