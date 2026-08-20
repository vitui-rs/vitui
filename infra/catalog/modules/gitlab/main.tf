locals {
  # GitLab bakes its `external_url` into clone URLs, redirects and the links in its own UI, so it
  # has to be the address the browser actually uses — port included. The listen port below is set
  # to the same number so the container listens where the URL says it does and the port mapping is
  # 1:1. A mismatch here does not fail: it serves a UI whose every link is wrong.
  external_url = "http://localhost:${var.http_port}"

  # What a job container uses. A job runs in its own container on the same network, where
  # `localhost` is the job itself. It reaches GitLab by container name instead.
  internal_url = "http://${var.container_name}:${var.http_port}"

  omnibus_config = join("\n", [
    "external_url '${local.external_url}'",
    # `gitlab_rails['nginx'][...]`, not `nginx[...]`: the top-level keys are deprecated as of
    # GitLab 19 and warn on every reconfigure. `mattermost[...]` is not deprecated but *removed* —
    # setting it at all aborts the reconfigure and the container restart-loops.
    "gitlab_rails['nginx']['listen_port'] = ${var.http_port}",
    "gitlab_rails['nginx']['listen_https'] = false",
    "gitlab_rails['gitlab_shell_ssh_port'] = ${var.ssh_port}",
    "gitlab_rails['initial_root_password'] = '${var.root_password}'",
    # A local CI host does not need to be told about itself. Every one of these is a process that
    # boots, holds memory and lengthens the first `gitlab-ctl reconfigure` for no return here.
    "prometheus_monitoring['enable'] = false",
    "gitlab_kas['enable'] = false",
    "registry['enable'] = false",
    # Two workers is enough for one developer and one runner, and it is roughly a gigabyte less
    # resident than the default, which sizes itself from the host's core count.
    "puma['worker_processes'] = 2",
    "sidekiq['max_concurrency'] = 5",
  ])
}

resource "docker_network" "ci" {
  name = var.network_name
}

resource "docker_image" "gitlab" {
  name = var.image
  # The image is pinned by tag and pulled once; do not re-resolve it on every plan.
  keep_locally = true
}

# Named volumes rather than bind mounts: GitLab's data directory is a Postgres cluster and a git
# repository store, and both are slow and permission-fragile across the macOS bind-mount layer.
resource "docker_volume" "config" {
  name = "${var.container_name}-config"
}

resource "docker_volume" "logs" {
  name = "${var.container_name}-logs"
}

resource "docker_volume" "data" {
  name = "${var.container_name}-data"
}

resource "docker_container" "gitlab" {
  name     = var.container_name
  image    = docker_image.gitlab.image_id
  hostname = var.container_name
  restart  = "unless-stopped"
  shm_size = var.shm_size

  env = ["GITLAB_OMNIBUS_CONFIG=${local.omnibus_config}"]

  networks_advanced {
    name = docker_network.ci.name
  }

  ports {
    internal = var.http_port
    external = var.http_port
  }

  ports {
    internal = 22
    external = var.ssh_port
  }

  volumes {
    volume_name    = docker_volume.config.name
    container_path = "/etc/gitlab"
  }

  volumes {
    volume_name    = docker_volume.logs.name
    container_path = "/var/log/gitlab"
  }

  volumes {
    volume_name    = docker_volume.data.name
    container_path = "/var/opt/gitlab"
  }
}

# Readiness, and it is deliberately not the container's healthcheck.
#
# The image's HEALTHCHECK reports `healthy` as soon as nginx answers, and nginx answers minutes
# before Rails does — on the first boot of this stack it went green at 45 seconds, everything
# downstream ran against a half-booted instance, and the database migrations were still going. The
# probe has to mean what the dependents need, so it is in two parts: `/-/readiness` for "Rails is
# serving", and then one `gitlab-rails runner` for "the door the bootstrap actually uses is open".
resource "null_resource" "wait_for_ready" {
  triggers = {
    container_id = docker_container.gitlab.id
  }

  provisioner "local-exec" {
    interpreter = ["/bin/sh", "-c"]
    command     = <<-EOT
      set -eu
      deadline=$(( $(date +%s) + ${var.ready_timeout_seconds} ))

      printf 'waiting for %s to serve /-/readiness ' '${var.container_name}'
      while :; do
        if docker exec '${var.container_name}'              curl -fsS -o /dev/null "http://localhost:${var.http_port}/-/readiness" 2>/dev/null; then
          printf ' ok
'
          break
        fi
        if [ "$(date +%s)" -ge "$deadline" ]; then
          printf '
gave up after ${var.ready_timeout_seconds}s
' >&2
          printf 'look at: docker logs %s
' '${var.container_name}' >&2
          printf 'and at:  docker exec %s gitlab-ctl status
' '${var.container_name}' >&2
          exit 1
        fi
        printf '.'
        sleep 5
      done

      # `gitlab-rails runner` is how the runner token and the project are created, and it loads the
      # whole Rails environment. If it cannot run, `/-/readiness` was optimistic.
      printf 'checking gitlab-rails is usable '
      while :; do
        if docker exec '${var.container_name}' gitlab-rails runner 'exit 0' >/dev/null 2>&1; then
          printf ' ok
'
          exit 0
        fi
        if [ "$(date +%s)" -ge "$deadline" ]; then
          printf '
gitlab-rails never became usable
' >&2
          exit 1
        fi
        printf '.'
        sleep 10
      done
    EOT
  }
}
