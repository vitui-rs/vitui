# The runner, and the token that lets it in. Depends on GitLab *answering*, not on its container
# existing — see the `ready` output it consumes.

locals {
  # `env.hcl` sits in the stack directory, one level above the generated units. `local_dir` is
  # derived from its path rather than from `get_terragrunt_dir()`, so that it points at the stack
  # directory and survives `terragrunt stack generate`, which rewrites everything below it.
  env_file  = find_in_parent_folders("env.hcl")
  env       = read_terragrunt_config(local.env_file).locals
  local_dir = "${dirname(local.env_file)}/.local"
  state_dir = "${dirname(local.env_file)}/.state/${basename(get_terragrunt_dir())}"
}

dependency "gitlab" {
  config_path = "../gitlab"

  # So that `plan` on a clean checkout describes the whole stack instead of failing on an output
  # that does not exist yet.
  mock_outputs                            = {
    container_name = "gitlab"
    network_name   = "ci"
    internal_url   = "http://gitlab:8929"
    ready          = "not-yet"
  }
  mock_outputs_allowed_terraform_commands = ["validate", "plan"]
}

terraform {
  source = "${get_terragrunt_dir()}/../../../../../catalog/modules/gitlab-runner"
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
  image            = local.env.runner_image
  container_name   = local.env.runner_container
  gitlab_container = dependency.gitlab.outputs.container_name
  gitlab_ready     = dependency.gitlab.outputs.ready
  gitlab_url       = dependency.gitlab.outputs.internal_url
  network_name     = dependency.gitlab.outputs.network_name
  default_image    = local.env.runner_default_image
  concurrent       = local.env.runner_concurrent
  local_dir        = local.local_dir
}
