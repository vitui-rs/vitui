variable "image" {
  description = "GitLab CE image reference. Pinned by the caller; `latest` drifts and a CI host that drifts is not a gate."
  type        = string
}

variable "container_name" {
  description = "Name of the GitLab container. Also its hostname on the CI network, which is what runner jobs clone from."
  type        = string
}

variable "network_name" {
  description = "Docker network the GitLab container, the runner and every job container share."
  type        = string
}

variable "http_port" {
  description = "Host port for GitLab's HTTP listener. GitLab is told to serve on this same port inside the container so that its own generated URLs match what a browser on the host sees."
  type        = number
}

variable "ssh_port" {
  description = "Host port forwarded to GitLab's SSH listener."
  type        = number
}

variable "root_password" {
  description = "Initial password for the `root` user. This is a throwaway local instance; it is not a secret worth protecting, and it is written here rather than typed once and forgotten."
  type        = string
  sensitive   = true
}

variable "shm_size" {
  description = "Shared memory, in megabytes. GitLab's bundled Postgres wants more than Docker's 64 MB default."
  type        = number
  default     = 512
}

variable "ready_timeout_seconds" {
  description = "How long to wait for GitLab to become usable. A first boot on a cold volume reconfigures Omnibus from scratch and loads the database schema; it is minutes, not seconds."
  type        = number
  default     = 900
}
