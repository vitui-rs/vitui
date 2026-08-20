# The local CI host, and the pipeline run that makes the CI gates real.
#
# The hosted-runner question this answers: a CI job nobody has watched go green is not a gate. This
# stack is somewhere to watch it, on a machine that is here.
#
# Entry point: `make gitlab`.

SHELL := /bin/sh

STACK    := live/local/ci
PROJECT  := vitui
GITLAB   := http://localhost:8929
REPO_ROOT := $(abspath $(CURDIR)/..)
LOCAL    := $(CURDIR)/$(STACK)/.local

# The Terraform Docker provider does not read Docker CLI contexts, and this machine runs Colima.
# Both halves have to agree on which daemon they mean, so the context's endpoint is exported here
# rather than assumed to be `/var/run/docker.sock`.
export DOCKER_HOST := $(shell docker context inspect --format '{{.Endpoints.docker.Host}}' 2>/dev/null)

# Terragrunt 1.x runs `tofu` by default. This host has Terraform.
export TG_TF_PATH := terraform

.PHONY: help gitlab up plan apply destroy clean open push pipeline status token url

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

destroy: ## terragrunt stack destroy — leaves the named volumes, so the next apply is not a first boot
	cd $(STACK) && terragrunt stack run --non-interactive destroy -- -auto-approve

clean: destroy ## destroy, then remove the generated stack, the state and the volumes
	cd $(STACK) && terragrunt stack clean
	-docker volume rm vitui-gitlab-config vitui-gitlab-logs vitui-gitlab-data
	rm -rf $(LOCAL)

# ── The repository, and the pipeline ────────────────────────

push: ## push this repository to the local GitLab and start a pipeline
	@test -f '$(LOCAL)/push-token' || { echo 'no push token — run `make apply` first' >&2; exit 1; }
	@cd '$(REPO_ROOT)' && \
	  url="http://root:$$(cat '$(LOCAL)/push-token')@localhost:8929/root/$(PROJECT).git" && \
	  git remote remove local 2>/dev/null; \
	  git remote add local "$$url" && \
	  git push --force local HEAD:refs/heads/master && \
	  git remote set-url local 'http://localhost:8929/root/$(PROJECT).git'
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
