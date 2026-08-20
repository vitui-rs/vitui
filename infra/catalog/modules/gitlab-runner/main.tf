locals {
  config_dir      = "${var.local_dir}/runner"
  config_path     = "${local.config_dir}/config.toml"
  config_template = "${local.config_dir}/config.toml.template"
}

# The config is rendered here with the token left as a placeholder, so that the shape of the
# runner's configuration is reviewable in the repository and in `terraform plan`. Only the
# substitution happens out of band, in `null_resource.authenticate` below.
resource "local_file" "config_template" {
  filename        = local.config_template
  file_permission = "0600"

  content = templatefile("${path.module}/templates/config.toml.tftpl", {
    concurrent    = var.concurrent
    description   = var.description
    gitlab_url    = var.gitlab_url
    default_image = var.default_image
    network_name  = var.network_name
  })
}

resource "null_resource" "authenticate" {
  triggers = {
    gitlab_ready    = var.gitlab_ready
    description     = var.description
    config_template = local_file.config_template.content_md5

    # Without this, deleting `.local/` by hand is unrecoverable while GitLab keeps running:
    # `local_file` rewrites the template with byte-identical content, so its md5 does not move, no
    # other trigger moves either, and Terraform reports no changes. The runner container then comes
    # up with no `[[runners]]` block and every pipeline sits pending forever with nothing anywhere
    # saying why. `fileexists` is evaluated at plan time, so the file going missing is a change.
    #
    # It costs one extra run: false on the first apply, true on the second, stable after that.
    # Re-running is harmless here — the script finds the existing runner by description.
    config_present = fileexists(local.config_path)
  }

  provisioner "local-exec" {
    interpreter = ["/bin/sh", "-c"]
    command     = <<-EOT
      set -eu
      mkdir -p '${local.config_dir}'

      docker cp '${path.module}/scripts/create-runner.rb' '${var.gitlab_container}:/tmp/vitui-create-runner.rb'
      docker exec \
        -e RUNNER_DESCRIPTION='${var.description}' \
        -e OUT_TOKEN=/tmp/vitui-runner-token \
        '${var.gitlab_container}' gitlab-rails runner /tmp/vitui-create-runner.rb
      docker cp '${var.gitlab_container}:/tmp/vitui-runner-token' '${local.config_dir}/.runner-token'
      docker exec '${var.gitlab_container}' rm -f /tmp/vitui-create-runner.rb /tmp/vitui-runner-token

      token=$(cat '${local.config_dir}/.runner-token')
      rm -f '${local.config_dir}/.runner-token'

      # `|` as the delimiter, and the token is base62 with dashes and underscores, so it cannot
      # contain one. `sed -i ''` is the BSD form; this host is macOS.
      sed "s|__RUNNER_TOKEN__|$token|" '${local.config_template}' > '${local.config_path}'
      chmod 600 '${local.config_path}'
    EOT
  }
}

resource "docker_image" "runner" {
  name         = var.image
  keep_locally = true
}

resource "docker_container" "runner" {
  name    = var.container_name
  image   = docker_image.runner.image_id
  restart = "unless-stopped"

  depends_on = [null_resource.authenticate]

  lifecycle {
    # `depends_on` orders the first creation and nothing after it: when the token is reissued, the
    # container keeps running with the config it read at startup. gitlab-runner does watch
    # `config.toml`, but recovering a deleted `.local/` needed a manual `docker restart` before this
    # was here — and "it usually reloads" is not a property to leave a CI host resting on.
    replace_triggered_by = [null_resource.authenticate]
  }

  networks_advanced {
    name = var.network_name
  }

  # The docker executor starts each job as a *sibling* container on the host daemon, not as a child
  # of this one. That is why the socket is here and `privileged` is not: nothing needs to run
  # docker-in-docker for a Rust build.
  volumes {
    host_path      = var.docker_socket
    container_path = "/var/run/docker.sock"
  }

  volumes {
    host_path      = local.config_dir
    container_path = "/etc/gitlab-runner"
  }
}
