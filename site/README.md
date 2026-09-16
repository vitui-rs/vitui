# The documentation site

[Astro Starlight](https://starlight.astro.build) over the markdown this repository already has.
It builds to static files and is served from **<https://vitui-rs.github.io>**.

```sh
cd site
npm install
npm run dev      # localhost:4321, with the staged content rebuilt first
npm run build    # ./dist, and a broken internal link fails it
```

## A chapter has one home, and it is not here

Nothing under `docs/` is copied into this project by hand. `npm run build` runs
`scripts/stage.mjs`, which reads the repository's own files and writes them into the content
collection with three transformations and no others:

| | why |
|---|---|
| the first `#` heading becomes frontmatter | Starlight needs a title where a repository file states one as a heading |
| `[the frame](02-the-frame.md)` becomes `/guide/02-the-frame/` | correct on GitHub, a 404 on a site; anything the site does not carry becomes a link into the repository |
| `<Left>` outside code becomes `&lt;Left>` | a markdown renderer treats it as a tag and deletes it |

The staged trees — `src/content/docs/{guide,adr,spec,reference}` and `public/img` — are **git-ignored
build output**. Editing one is editing something that will be gone on the next build; edit
`docs/guide/…`, `docs/adr/…`, `docs/spec/…` or `CONTEXT.md` instead.

## Four data files, and two of them are committed

`src/generated/` holds what a page cannot read for itself.

| file | from | committed |
|---|---|---|
| `inventory.json` | `vitui_components::INVENTORY`, via `cargo run -p vitui-components --example inventory_json` | yes |
| `apps.json` | `vitui_apps::APPS`, via `cargo run -p vitui-apps --bin apps_json` | yes |
| `benchmarks.json` | `compare/REPORT.md` and `compare/FINDINGS.md`, parsed by `scripts/evidence.mjs` | no |
| `terminals.json` | `conform/REPORT-*.md`, parsed by the same script | no |

The split is the toolchain and not a preference. The two parsed files are plain node over committed
markdown, so they are regenerated on every build and there is nothing to keep in step. The two Rust
ones need cargo, which the machine that serves the site does not have — so they are committed, and
because a committed derivation is a copy, `npm run check` regenerates both and fails if either has
stopped matching the crate it came from. `npm run generate` rewrites them; the diff is what gets
committed.

**The component page's population is the freeze.** A component added to `INVENTORY` and left off the
site is a failing build rather than a page nobody noticed, which is the same property every other
obligation in this workspace is built on.

## Where it is served from, and what changes when

`vitui-rs/vitui-rs.github.io` is an **organisation page repository**, so it serves at the apex with
no base path — which is why the site is not a project page of the workspace repository, where every
URL would carry `/vitui/` and the first forgotten one would be a silent 404.

The workspace repository is what holds this directory; the pages repository holds only built output.
`.github/workflows/site.yml` builds here and pushes `dist/` there. That push needs a deploy key, and
**it exists as of 2026-09-15** — this is the record of how, not a step still owed:

```sh
ssh-keygen -t ed25519 -N "" -C "vitui site deploy" -f /tmp/vitui-pages
gh repo deploy-key add /tmp/vitui-pages.pub --repo vitui-rs/vitui-rs.github.io --title "site deploy" --allow-write
gh secret set PAGES_DEPLOY_KEY --repo vitui-rs/vitui --body "$(cat /tmp/vitui-pages)"
rm /tmp/vitui-pages /tmp/vitui-pages.pub
```

**Deploy keys are an organisation policy before they are a repository setting.** The second command
answers `HTTP 422: Deploy keys are disabled for this repository` while
`orgs/vitui-rs.deploy_keys_enabled_for_repositories` is false, which is a checkbox under the
organisation's *Settings → Repository → Deploy keys* and is not discoverable from the repository it
fails on. `--allow-write` is not optional either: a read-only key authenticates and then fails to
push.

Without the secret the workflow still builds the site and uploads it as an artefact, and the deploy
step is skipped. That is deliberate: a deploy that cannot run should be visibly absent rather than
red — and it is why the condition sits on the job's `env` rather than the step's, since a step's own
`env:` block is not what its `if:` can read.

**A manual run does not deploy.** The condition begins `github.event_name == 'push'`, so
`workflow_dispatch` builds and stops; the deploy follows a push to `master` that touches one of the
paths at the top of the workflow. That is not a limitation to work around — a fork's pull request
cannot reach the secret, and the same condition is what makes running this on one safe.

## This is not one of the gates

The gate set for this repository is `.gitlab-ci.yml`, on a runner that is not on the internet.
`.github/workflows/` holds what a local runner cannot do, and a documentation site that GitHub
serves is squarely that. The two checks here — the link validator and the generated-data ratchet —
are real, and they are the site's own.
