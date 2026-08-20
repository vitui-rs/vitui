# `infra/` — the local CI host

A GitLab and one runner, in Docker, deployed with Terragrunt. It exists for one reason:

> **A CI job nobody has watched go green is not a gate, whatever it checks.**

This repository's history is three-for-three on that. Engine ticket 13 left a `cargo test
--workspace` step that had never passed; runtime R15 found a `cargo clippy --workspace
--all-targets` step that had never passed under its own `RUSTFLAGS: -D warnings`; components C11
found a `cargo deny` job that had never passed at all. Each was written, reviewed, and believed.

There is no hosted CI for this repository yet. So the gates run here.

## The one command

```bash
cd infra && make gitlab
```

It brings up GitLab, the runner and the project, then opens the UI. First boot is minutes, not
seconds — Omnibus reconfigures itself from scratch on a cold volume, and `make apply` prints dots
while it waits. Then:

```bash
make push          # push this repository and open the pipeline page
```

`root` / `Vt9xQr4mZk7pLs2d` at <http://localhost:8929>. (`make token` prints the push token.)

## Layout

The shape is the `catalog` / `live` split: reusable units in one tree, one directory per real
deployment in the other.

```
infra/
├── catalog/
│   ├── modules/          Terraform. What a thing is.
│   │   ├── gitlab/           the instance, its volumes, and the wait for it to answer
│   │   ├── gitlab-runner/    the runner, and the token that lets it in
│   │   └── gitlab-project/   the project to push to, and a token to push with
│   └── units/            Terragrunt. How a module is wired to a site.
│       ├── gitlab/
│       ├── gitlab-runner/
│       └── gitlab-project/
└── live/
    └── local/ci/         The one deployment.
        ├── terragrunt.stack.hcl    which units, and under what names
        └── env.hcl                 every value that is site-specific
```

There is one location and it is `local`, because this stack answers a question that is about *this
machine*. A second location would be a second machine, not a second environment.

## Five things that are decisions, not defaults

**`external_url` and `clone_url` are different addresses for the same instance.** GitLab bakes
`external_url` into every link it generates, so it has to be what the browser uses —
`http://localhost:8929`. A job container is a different container: `localhost` there is the job.
So the runner is given `clone_url = http://vitui-gitlab:8929` explicitly. Inheriting it fails at
`git fetch` with a connection refused and nothing upstream of that says why.

**The runner's token is minted through `gitlab-rails runner`, not the API.** GitLab 16 replaced the
shared registration token with per-runner authentication tokens, and there is no way to mint one
over the REST API before an API token exists. `gitlab-rails runner` is the only door open at that
point, and it is the same door the project bootstrap uses.

**Neither token is in Terraform state.** Both are written to `live/local/ci/.local/`, which is
gitignored, by a `local-exec` that never returns them to Terraform. A local state file in a working
tree is exactly the wrong place for a credential.

**Readiness is not the container's healthcheck.** The image reports `healthy` as soon as nginx
answers, and nginx answers minutes before Rails does. On the first boot here it went green at 45
seconds, everything downstream ran against a half-booted instance, and the database migrations were
still in flight. So the probe is `/-/readiness` followed by one `gitlab-rails runner` — the second
half because that is the door the bootstrap actually uses, and a probe that does not test the thing
its dependents need is a probe that lies at the worst moment.

**The root password looks like line noise on purpose.** It was `vitui-local-ci-root`, and GitLab's
admin seed refused it: *"Password must not contain commonly used combinations of words and
letters."* That failure is worth knowing about because of where it lands — the seed is a step inside
`gitlab-ctl reconfigure`, so the reconfigure aborts, the container restart-loops, and nothing
anywhere says "bad password" until you read the migration log inside the container.

## What this does not cover

The GitHub workflow's `test` job is a three-OS matrix. A local Docker runner is one OS and one
architecture — here, linux/arm64. **Green here means the gates pass; it does not mean they pass on
Windows.** `.github/workflows/ci.yml` remains the definition of record for that.

The two files run the same three gates, and that much is kept in step by hand. Their *versions* are
deliberately not: GitHub tracks `dtolnay/rust-toolchain@stable` and a floating
`cargo-deny-action@v2`, while this side pins the toolchain and cargo-deny to exact versions. The
floating side is the early warning that a new release has something to say about us; the pinned side
is the reproducible gate. Expecting them to agree would cost both properties.

## Targets

| | |
|---|---|
| `make gitlab` | everything up, browser open |
| `make plan` / `make apply` | the stack |
| `make stop` / `make start` | stop paying for the RAM, keep everything — seconds to come back |
| `make destroy` | tear the stack down **including the volumes**; the next apply is a first boot |
| `make clean` | destroy, plus the generated stack, the state and the bootstrap artifacts |
| `make push` | push this repository and open the pipeline page |
| `make status` | containers, and whether the runner is verified |
| `make token` | print the push token |

## Requirements

Docker (this machine runs Colima), Terragrunt 1.x, Terraform 1.5+. Terragrunt runs `tofu` by
default and this host has Terraform, so the Makefile exports `TG_TF_PATH=terraform`. It also
exports `DOCKER_HOST` from `docker context inspect`, because the Terraform Docker provider does not
read Docker CLI contexts and Colima's socket is not `/var/run/docker.sock`.
