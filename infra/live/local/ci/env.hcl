locals {

  # ───────────────────────────────────────────────
  # Docker connection
  # ───────────────────────────────────────────────
  # The Terraform provider does not read Docker CLI contexts, and this machine runs Colima, whose
  # socket is not `/var/run/docker.sock`. The Makefile exports `DOCKER_HOST` from
  # `docker context inspect` so that both halves talk to the same daemon; the default is the plain
  # Linux path, for a host where they already agree.
  docker_host = get_env("DOCKER_HOST", "unix:///var/run/docker.sock")

  # ───────────────────────────────────────────────
  # Images
  # ───────────────────────────────────────────────
  # Pinned, not `latest`. A CI host that upgrades itself between two runs turns a red pipeline into
  # a question about which GitLab it was.
  gitlab_image = "gitlab/gitlab-ce:19.2.4-ce.0"
  runner_image = "gitlab/gitlab-runner:v19.3.0"

  # What a job gets when `.gitlab-ci.yml` does not name an image.
  runner_default_image = "rust:1-bookworm"

  # ───────────────────────────────────────────────
  # Names and ports
  # ───────────────────────────────────────────────
  network_name     = "vitui-ci"
  gitlab_container = "vitui-gitlab"
  runner_container = "vitui-gitlab-runner"

  # 8929 rather than 80: this is a developer machine with other things on it, and GitLab's own
  # documented port for a container install is 8929.
  gitlab_http_port = 8929
  gitlab_ssh_port  = 2224

  # ───────────────────────────────────────────────
  # Credentials
  # ───────────────────────────────────────────────
  # A throwaway password for a throwaway instance that listens on localhost only. It is written
  # here on purpose: the alternative is that it is generated, forgotten, and the UI is unreachable
  # the one time somebody needs to look at a failed job.
  #
  # It looks like line noise because GitLab rejects anything that reads like words — the first
  # attempt was `vitui-local-ci-root` and the admin seed refused it with "Password must not contain
  # commonly used combinations of words and letters". That failure is worth knowing about because
  # of *where* it lands: the seed is a step inside `gitlab-ctl reconfigure`, so a rejected password
  # aborts the reconfigure, the container restart-loops, and nothing anywhere says "bad password"
  # until you read the migration log inside the container.
  gitlab_root_password = "Vt9xQr4mZk7pLs2d"

  # ───────────────────────────────────────────────
  # Runner
  # ───────────────────────────────────────────────
  # Three, because the pipeline has three jobs and they are independent. At 1 the pipeline is as
  # long as the sum of its jobs.
  runner_concurrent = 3

  # ───────────────────────────────────────────────
  # The project pushed here
  # ───────────────────────────────────────────────
  project_path = "vitui"
}
