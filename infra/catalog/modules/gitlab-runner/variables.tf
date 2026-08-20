variable "image" {
  description = "gitlab-runner image reference."
  type        = string
}

variable "container_name" {
  description = "Name of the runner container."
  type        = string
}

variable "gitlab_container" {
  description = "Name of the GitLab container, used to mint the runner's authentication token."
  type        = string
}

variable "gitlab_ready" {
  description = "Readiness handle from the gitlab module."
  type        = string
}

variable "gitlab_url" {
  description = "The URL the runner and its job containers use to reach GitLab. This is the container-name URL, not the browser one."
  type        = string
}

variable "network_name" {
  description = "Docker network shared by GitLab, the runner and every job container."
  type        = string
}

variable "description" {
  description = "Runner description. Also the key this module finds an already-created runner by, so changing it mints a second runner rather than renaming the first."
  type        = string
  default     = "vitui-local"
}

variable "default_image" {
  description = "Image a job gets when its `.gitlab-ci.yml` does not name one."
  type        = string
}

variable "concurrent" {
  description = "How many jobs run at once. The three CI jobs are independent, so this is what decides whether the pipeline is one job long or three."
  type        = number
  default     = 3
}

variable "local_dir" {
  description = "Host directory for the runner's config. Bind-mounted into the container, and gitignored: it holds the runner's authentication token."
  type        = string
}

variable "docker_socket" {
  description = "Host path of the Docker socket. The runner needs it to start job containers as siblings of itself."
  type        = string
  default     = "/var/run/docker.sock"
}
