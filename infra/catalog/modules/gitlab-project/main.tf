locals {
  out_project_path = "${var.local_dir}/project-path"
  out_token        = "${var.local_dir}/push-token"
}

resource "null_resource" "bootstrap" {
  triggers = {
    # Re-run when GitLab is replaced, and when either name changes. Not on every apply: this
    # reissues a token, and reissuing one behind the user's back invalidates the remote they
    # already configured.
    gitlab_ready = var.gitlab_ready
    project_path = var.project_path
    token_name   = var.token_name
  }

  provisioner "local-exec" {
    interpreter = ["/bin/sh", "-c"]
    command     = <<-EOT
      set -eu
      mkdir -p '${var.local_dir}'

      # Copied in and run from a file rather than passed as an argument: the script is Ruby inside
      # a shell command inside a Terraform heredoc, and every layer of that has its own idea about
      # quoting.
      docker cp '${path.module}/scripts/bootstrap.rb' '${var.gitlab_container}:/tmp/vitui-bootstrap.rb'

      docker exec \
        -e PROJECT_PATH='${var.project_path}' \
        -e TOKEN_NAME='${var.token_name}' \
        -e OUT_PROJECT_PATH=/tmp/vitui-project-path \
        -e OUT_TOKEN=/tmp/vitui-push-token \
        '${var.gitlab_container}' gitlab-rails runner /tmp/vitui-bootstrap.rb

      docker cp '${var.gitlab_container}:/tmp/vitui-project-path' '${local.out_project_path}'
      docker cp '${var.gitlab_container}:/tmp/vitui-push-token' '${local.out_token}'
      chmod 600 '${local.out_token}'

      docker exec '${var.gitlab_container}' rm -f \
        /tmp/vitui-bootstrap.rb /tmp/vitui-project-path /tmp/vitui-push-token
    EOT
  }
}
