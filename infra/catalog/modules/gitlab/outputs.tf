output "container_name" {
  description = "Name of the running GitLab container, for `docker exec`."
  value       = docker_container.gitlab.name
}

output "network_name" {
  description = "Name of the shared CI network."
  value       = docker_network.ci.name
}

output "external_url" {
  description = "The URL a browser on the host uses."
  value       = local.external_url
}

output "internal_url" {
  description = "The URL a runner job container uses. Not the same as `external_url`: `localhost` inside a job container is the job, not GitLab."
  value       = local.internal_url
}

output "ready" {
  description = "Opaque value that only exists once GitLab is serving and `gitlab-rails runner` works. Depend on this, not on the container: the container is up long before GitLab is."
  value       = null_resource.wait_for_ready.id
}
