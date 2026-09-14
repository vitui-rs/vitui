// Read code out of the repository at build time, so that a page quoting a program quotes **the**
// program.
//
// The repository's own arrangement for this is `crates/vitui/tests/quickstart.rs`: the README's
// quickstart is `crates/vitui-apps/examples/counter.rs` with its comment lines removed, and
// `cargo test` compares the two. That works because the README is one block of one file. A tutorial
// is not — it is the same program in five slices with prose between them — so the same property is
// bought here the other way round: **the slices are cut out of the file at build time**, and a
// marker that stops matching is a build that stops.
//
// The comment stripping is `without_comments`' rule, deliberately: comment lines, never trailing
// comments, and the runs of blank lines the removal leaves collapsed to one. The examples in this
// workspace carry more comment than code — `counter.rs` is 181 lines of which 86 are comment — and
// the argument in them is written for a reader of the repository, not for a reader learning the
// loop for the first time.

import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'

/**
 * The repository root, found by walking up from the working directory.
 *
 * **Not from `import.meta.url`**, which is the obvious spelling and the wrong one: this module is
 * bundled into the build output, so at render time that URL is a chunk under `dist/` and every path
 * relative to it lands three directories from where the author meant. Walking for the workspace
 * manifest is the one answer that survives being bundled.
 */
function repository() {
  let dir = process.cwd()
  for (let i = 0; i < 6; i += 1) {
    if (existsSync(join(dir, 'Cargo.toml')) && existsSync(join(dir, 'crates'))) return dir
    dir = dirname(dir)
  }
  throw new Error(`no Cargo workspace above ${process.cwd()}, so the site cannot quote its code`)
}

const REPO = repository()

/** A file of the repository, by its path from the root. */
export function file(path) {
  try {
    return readFileSync(join(REPO, path), 'utf8')
  } catch {
    throw new Error(`${path} is quoted by the site and is not in the repository`)
  }
}

/** Comment lines out, runs of blank lines collapsed. */
export function stripped(source) {
  const kept = source
    .split('\n')
    .filter((line) => !line.trimStart().startsWith('//'))
    .map((line) => line.trimEnd())

  const out = []
  for (const line of kept) {
    if (line === '' && (out.length === 0 || out.at(-1) === '')) continue
    out.push(line)
  }
  return out.join('\n').trim()
}

/**
 * One slice of a file: from the first line holding `from` to the first line holding `to` after it,
 * both inclusive.
 *
 * Both markers are **code** — a signature, a closing brace at column zero — rather than line
 * numbers, because a line number goes wrong silently and a marker goes wrong loudly.
 */
export function slice(path, from, to, { strip = true } = {}) {
  const source = strip ? stripped(file(path)) : file(path)
  const lines = source.split('\n')

  // A marker beginning with `^` is anchored at the start of the line, which is what separates the
  // brace that closes a function from the four that close a block inside it.
  const holds = (marker) => (line) => (marker.startsWith('^') ? line.startsWith(marker.slice(1)) : line.includes(marker))

  const start = lines.findIndex(holds(from))
  if (start === -1) throw new Error(`${path} has no line holding ${JSON.stringify(from)}`)
  const rest = lines.slice(start + 1).findIndex(holds(to))
  if (rest === -1) throw new Error(`${path} has no line holding ${JSON.stringify(to)} after ${JSON.stringify(from)}`)

  return lines.slice(start, start + 1 + rest + 1).join('\n')
}

/** A whole file, comments out. */
export function program(path) {
  return stripped(file(path))
}

/** One value out of a TOML-ish manifest line, for the dependency table a reader has to type. */
export function manifestLine(path, key) {
  const line = file(path)
    .split('\n')
    .find((l) => l.trimStart().startsWith(`${key} `) || l.trimStart().startsWith(`${key}=`))
  if (!line) throw new Error(`${path} has no ${key} line`)
  return line.trim()
}
