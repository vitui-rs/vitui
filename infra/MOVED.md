# This stack moved to `~/Projects/devkit`

There used to be a Terragrunt stack here that stood up a GitLab CE and a runner in Docker, just for
this repository. It is gone, and this file is the signpost left in its place.

Every repository on this machine now shares **one** GitLab instance:

| | |
|---|---|
| URL | <http://gitlab.localhost:8940> |
| This repo's project | `repos/vitui` |
| Start / stop it | `devkit up` / `devkit down` |
| Manual | `~/Projects/devkit/README.md` |

```sh
. ~/.local/state/devkit/env    # GITLAB_HOST, DEVKIT_TOKENS, DEVKIT_GROUP, DEVKIT_URL
```

## If you are an agent whose context predates this

Commands that no longer exist, and what replaced them:

| was | now |
|---|---|
| `cd infra && make gitlab` / `make apply` / `make up` | `devkit up` |
| `cd infra && make stop` | `devkit down` |
| `cd infra && make destroy` / `make clean` | **gone on purpose** — there is no destroy verb |
| `eval "$(cd infra && make -s env)"` | `. ~/.local/state/devkit/env` then `export GITLAB_TOKEN=$(cat "$DEVKIT_TOKENS/vitaly/token")` |
| `eval "$(cd infra && make -s agent-env)"` | same, with `$DEVKIT_TOKENS/agent/token` |
| `cd infra && make push` | plain `git push devkit` — devkit never runs git |
| `http://localhost:8929` / `:8930` | `http://gitlab.localhost:8940` |
| `root/vitui` | `repos/vitui` |

Delete this file whenever it has served its purpose. Nothing reads it.
