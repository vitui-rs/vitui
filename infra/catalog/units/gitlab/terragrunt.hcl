# The GitLab instance itself. Everything else on this stack waits on it.

locals {
  # `env.hcl` sits in the stack directory, one level above the generated units. `local_dir` is
  # derived from its path rather than from `get_terragrunt_dir()`, so that it points at the stack
  # directory and survives `terragrunt stack generate`, which rewrites everything below it.
  env_file  = find_in_parent_folders("env.hcl")
  env       = read_terragrunt_config(local.env_file).locals
  local_dir = "${dirname(local.env_file)}/.local"
  state_dir = "${dirname(local.env_file)}/.state/${basename(get_terragrunt_dir())}"
}

terraform {
  # Five levels up from a generated unit: `.terragrunt-stack` -> the stack dir -> the location ->
  # `live` -> `infra`.
  source = "${get_terragrunt_dir()}/../../../../../catalog/modules/gitlab"
}

# State goes beside `env.hcl`, NOT in `${get_terragrunt_dir()}`.
#
# The unit's own directory is under `.terragrunt-stack/`, which `.gitignore` labels "output, not
# source" and which `terragrunt stack clean` deletes wholesale. Keeping state there means anyone who
# removes regenerable output also removes the only record of the containers, network and volumes
# that were created — the next apply then fails with `Conflict. The container name
# "/vitui-gitlab" is already in use` and there is no `destroy` path left to fix it with.
remote_state {
  backend = "local"

  config = {
    path = "${local.state_dir}/terraform.tfstate"
  }
}

generate "provider" {
  path      = "provider.tf"
  if_exists = "overwrite"

  contents = <<-EOT
    provider "docker" {
      host = "${local.env.docker_host}"
    }
  EOT
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
  image          = local.env.gitlab_image
  container_name = local.env.gitlab_container
  network_name   = local.env.network_name
  http_port      = local.env.gitlab_http_port
  ssh_port       = local.env.gitlab_ssh_port
  root_password  = local.env.gitlab_root_password
}
