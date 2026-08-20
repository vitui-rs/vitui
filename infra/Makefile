# The local CI host, and the pipeline run that makes the CI gates real.
#
# The hosted-runner question this answers: a CI job nobody has watched go green is not a gate. This
# stack is somewhere to watch it, on a machine that is here.
#
# Entry point: `make gitlab`.

SHELL := /bin/sh

STACK    := live/local/ci
GITLAB   := http://localhost:8929
REPO_ROOT := $(abspath $(CURDIR)/..)
LOCAL    := $(CURDIR)/$(STACK)/.local

# Read back from what the bootstrap actually created, not restated here. A second copy of the
# project path is a second thing to forget: change it in `env.hcl` and every `make push` would keep
# pushing at a project that no longer exists, and GitLab answers that with a 404 that names nothing.
PROJECT   = $(shell cat '$(LOCAL)/project-path' 2>/dev/null | cut -d/ -f2)

# The Terraform Docker provider does not read Docker CLI contexts, and this machine runs Colima.
# Both halves have to agree on which daemon they mean, so the context's endpoint is exported here
# rather than assumed to be `/var/run/docker.sock`.
export DOCKER_HOST := $(shell docker context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null)

# Terragrunt 1.x runs `tofu` by default. This host has Terraform.
export TG_TF_PATH := terraform

.PHONY: help gitlab up plan apply destroy stop start clean open push pipeline status token url

help: ## show this help
	@echo 'usage: make [target] ...'
	@echo ''
	@echo 'targets:'
	@egrep '^(.+)\:\ .*##\ (.+)' ${MAKEFILE_LIST} | sed 's/:.*##/#/' | column -t -c 2 -s '#'

# ── The one command ─────────────────────────────────────────

gitlab: up open ## bring the whole CI host up and open GitLab in a browser
	@echo ''
	@echo "GitLab is at $(GITLAB)  —  root / $$(grep -o 'gitlab_root_password = \"[^\"]*\"' $(STACK)/env.hcl | cut -d'\"' -f2)"
	@echo "Next: make push   (push this repository and start a pipeline)"

up: apply ## alias for apply

open: ## open the GitLab UI
	@open '$(GITLAB)' 2>/dev/null || echo 'open $(GITLAB) in a browser'

# ── Terragrunt ──────────────────────────────────────────────

plan: ## terragrunt stack plan
	cd $(STACK) && terragrunt stack run plan

# `--non-interactive` is terragrunt's own prompt, and `-auto-approve` after `--` is Terraform's.
# They are two different confirmations and skipping only one of them hangs.
apply: ## terragrunt stack apply — GitLab, the runner, the project and the push token
	cd $(STACK) && terragrunt stack run --non-interactive apply -- -auto-approve

# This DOES delete the volumes. They are `docker_volume` resources in the gitlab module, so
# Terraform owns them, and the next `make apply` is a full cold Omnibus reconfigure with the
# project, the runner registration and every pipeline gone. To stop paying for the RAM without
# paying that price, use `make stop`.
destroy: ## terragrunt stack destroy — including the volumes, so the next apply IS a first boot
	cd $(STACK) && terragrunt stack run --non-interactive destroy -- -auto-approve

stop: ## stop the containers and keep everything — `make start` brings them back in seconds
	-docker stop vitui-gitlab-runner vitui-gitlab

start: ## start the containers stopped by `make stop`
	-docker start vitui-gitlab vitui-gitlab-runner

clean: destroy ## destroy, then remove the generated stack, the state and the bootstrap artifacts
	cd $(STACK) && terragrunt stack clean
	rm -rf $(LOCAL) $(CURDIR)/$(STACK)/.state

# ── The repository, and the pipeline ────────────────────────

# The token is passed to `git push` as an argument and never stored. It used to be written into a
# remote and stripped again on the line after the push — which is fine until the push fails, and
# then an `api` + `write_repository` token sits in the developer's `.git/config` in plaintext. The
# `gitlab-project` module goes out of its way to keep this token out of Terraform state; leaving it
# in `.git/config` would undo that.
push: ## push this repository to the local GitLab and start a pipeline
	@test -s '$(LOCAL)/push-token' || { echo 'no push token — run `make apply` first' >&2; exit 1; }
	@test -n '$(PROJECT)' || { echo 'no project path — run `make apply` first' >&2; exit 1; }
	@cd '$(REPO_ROOT)' && \
	  git remote remove local 2>/dev/null || true; \
	  git remote add local '$(GITLAB)/root/$(PROJECT).git'
	@cd '$(REPO_ROOT)' && \
	  git push --force \
	    "http://root:$$(cat '$(LOCAL)/push-token')@localhost:8929/root/$(PROJECT).git" \
	    HEAD:refs/heads/master
	@echo ''
	@echo 'pipeline: $(GITLAB)/root/$(PROJECT)/-/pipelines'
	@open '$(GITLAB)/root/$(PROJECT)/-/pipelines' 2>/dev/null || true

pipeline: ## open the pipelines page
	@open '$(GITLAB)/root/$(PROJECT)/-/pipelines' 2>/dev/null || echo '$(GITLAB)/root/$(PROJECT)/-/pipelines'

status: ## container and runner status
	@docker ps --filter name=vitui-gitlab --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
	@echo ''
	@docker exec vitui-gitlab-runner gitlab-runner verify 2>&1 | tail -5 || true

token: ## print the push token
	@cat '$(LOCAL)/push-token'; echo

url: ## print the GitLab URL
	@echo '$(GITLAB)'
