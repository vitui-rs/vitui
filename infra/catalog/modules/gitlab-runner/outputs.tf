output "container_name" {
  description = "Name of the running runner container."
  value       = docker_container.runner.name
}

output "config_path" {
  description = "Host path of the generated `config.toml`."
  value       = local.config_path
}
