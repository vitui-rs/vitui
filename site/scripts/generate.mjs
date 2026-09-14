// The two values the site joins on, taken out of the crates that hold them.
//
// `INVENTORY` is the component freeze and `APPS` is the application list. Both are Rust `const`s
// that other gates in this workspace already read, and a static-site generator cannot. So two tiny
// programs print them as JSON and the result is **committed**: the site builds on a machine with
// node and no cargo — a GitHub Pages job, somebody's laptop — and a build that needed a Rust
// toolchain to render a table would be a build nobody can reproduce.
//
// A committed derivation is a copy, and a copy goes stale. `--check` is the ratchet that stops it:
// it regenerates into memory and fails if the result differs from what is on disk, which is the
// arrangement every golden screen in this repository already uses. The CI job runs `--check`; a
// person who changed the freeze runs `npm run generate` and commits the diff.

import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const REPO = join(here, '..', '..')
const OUT = join(here, '..', 'src', 'generated')

const SOURCES = [
  {
    file: 'inventory.json',
    what: 'the component freeze',
    argv: ['run', '-q', '-p', 'vitui-components', '--example', 'inventory_json'],
    regenerate: 'npm run generate',
  },
  {
    file: 'apps.json',
    what: 'the application list',
    argv: ['run', '-q', '-p', 'vitui-apps', '--bin', 'apps_json'],
    regenerate: 'npm run generate',
  },
]

const check = process.argv.includes('--check')
mkdirSync(OUT, { recursive: true })

let stale = 0
for (const source of SOURCES) {
  const fresh = execFileSync('cargo', source.argv, { cwd: REPO, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 })
  const path = join(OUT, source.file)

  if (!check) {
    writeFileSync(path, fresh)
    console.log(`wrote ${source.file} — ${source.what}`)
    continue
  }

  let committed = ''
  try {
    committed = readFileSync(path, 'utf8')
  } catch {
    committed = ''
  }
  if (committed === fresh) {
    console.log(`current: ${source.file} — ${source.what}`)
  } else {
    stale += 1
    console.error(`stale: ${source.file} no longer matches ${source.what}. Run \`${source.regenerate}\` and commit the diff.`)
  }
}

if (stale) process.exit(1)
