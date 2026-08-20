terraform {
  # Providers are pinned to exact versions for the same reason the images in `env.hcl` are: a CI
  # host that changes under you is not a gate. `.terraform.lock.hcl` cannot do this job here — it
  # is generated inside `.terragrunt-stack/`, which is not tracked — so the constraint is the pin.
  #
  # This was not hypothetical. The constraint here read `>= 3.0` for kreuzwerker/docker and had
  # already resolved to 4.5.0.
  required_version = ">= 1.5"

  required_providers {
    null = {
      source  = "hashicorp/null"
      version = "3.3.1"
    }
  }
}
