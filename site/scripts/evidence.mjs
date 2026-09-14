// The two suites' committed reports, parsed into what a page can render.
//
// `compare/REPORT.md` and `conform/REPORT-<arm>.md` are **generated and committed**: neither suite
// gates anything, because a third party's version cannot be allowed to fail this repository's pull
// requests, and a worsening number arriving as a review-visible diff is what makes the claim
// falsifiable at all. That makes them the only honest source for a page that quotes them — a table
// retyped into a website is a table that stops agreeing with the run it came from, and the
// disagreement is invisible precisely because nobody regenerates a website.
//
// So the numbers are read at build time. A report whose shape changes enough that the parse finds
// nothing is a **build failure** rather than an empty table: a benchmark page with no rows would be
// published, and nobody would notice for a month.

import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const REPO = join(here, '..', '..')
const OUT = join(here, '..', 'src', 'generated')

/** The markdown table that follows a heading, as headers plus rows of cells. */
function tableAfter(source, heading) {
  const lines = source.split('\n')
  const at = lines.findIndex((l) => l.startsWith('#') && l.includes(heading))
  if (at === -1) throw new Error(`no heading holding ${JSON.stringify(heading)}`)

  const cells = (line) =>
    line
      .trim()
      .replace(/^\|/, '')
      .replace(/\|$/, '')
      .split('|')
      .map((c) => c.trim())

  let i = at + 1
  while (i < lines.length && !lines[i].trim().startsWith('|')) {
    if (lines[i].startsWith('#')) throw new Error(`no table under ${JSON.stringify(heading)}`)
    i += 1
  }
  const headers = cells(lines[i])
  i += 2 // the alignment row
  const rows = []
  while (i < lines.length && lines[i].trim().startsWith('|')) {
    rows.push(cells(lines[i]))
    i += 1
  }
  if (!rows.length) throw new Error(`the table under ${JSON.stringify(heading)} has no rows`)
  return { headers, rows }
}

/** One `- **Key:** value` line of a report's preamble. */
function fact(source, key) {
  const line = source.split('\n').find((l) => l.startsWith(`- **${key}:**`))
  if (!line) throw new Error(`no '${key}' line in the report`)
  return line.slice(`- **${key}:**`.length).trim()
}

function comparative() {
  const report = readFileSync(join(REPO, 'compare', 'REPORT.md'), 'utf8')
  const findings = readFileSync(join(REPO, 'compare', 'FINDINGS.md'), 'utf8')

  // The caveats are a list under a heading whose words are the point: *what this run does not say*.
  // Both runs carry one and the second says "everything the first run's list says, unchanged", so
  // the page shows both lists in order.
  const caveats = []
  const lines = findings.split('\n')
  for (let i = 0; i < lines.length; i += 1) {
    if (!/^## What this run (still )?does not say/.test(lines[i])) continue
    for (let j = i + 1; j < lines.length && !lines[j].startsWith('## '); j += 1) {
      const t = lines[j].trim()
      if (t.startsWith('- ')) caveats.push(t.slice(2))
    }
  }
  if (caveats.length < 4) throw new Error('compare/FINDINGS.md carries fewer caveats than it used to')

  return {
    machine: fact(report, 'Machine'),
    frames: fact(report, 'Frames per scene'),
    cpuScene: fact(report, 'CPU scene'),
    latency: fact(report, 'Latency trials'),
    marginal: tableAfter(report, 'Bytes a frame, steady state — declared tier `truecolor`'),
    cpu: tableAfter(report, 'Process CPU over the fixed scene'),
    keystroke: tableAfter(report, 'Keystroke to wire'),
    caveats,
  }
}

function conformance() {
  const dir = join(REPO, 'conform')
  const arms = readdirSync(dir)
    .filter((f) => /^REPORT-.*\.md$/.test(f))
    .sort()
    .map((file) => {
      const source = readFileSync(join(dir, file), 'utf8')
      const arm = fact(source, 'Arm')
      const evidence = fact(source, 'These rows are evidence about')

      // Per scene: the agreement line the suite prints, and the rows it could not ask about. A
      // `cannot ask` is not a disagreement — it is the capture surface refusing the question — and
      // the distinction is the whole reason this suite has eight arms rather than one.
      const scenes = []
      const lines = source.split('\n')
      for (let i = 0; i < lines.length; i += 1) {
        const scene = /^## Scene (\d+) — (.+)$/.exec(lines[i])
        if (!scene) continue
        let agreed = null
        let total = null
        let cannotAsk = 0
        let byDesign = 0
        for (let j = i + 1; j < lines.length && !lines[j].startsWith('## '); j += 1) {
          const m = /^\*\*(\d+)\/(\d+) agreed/.exec(lines[j].trim())
          if (m && agreed === null) [, agreed, total] = m
          // Table rows only. The phrase occurs in the prose around a scene too, and a page that
          // counted those would report a number larger than the scene has rows.
          if (lines[j].trim().startsWith('|')) {
            cannotAsk += (lines[j].match(/cannot ask/g) ?? []).length
            byDesign += (lines[j].match(/by design/g) ?? []).length
          }
        }
        scenes.push({
          number: scene[1],
          title: scene[2],
          agreed: agreed === null ? null : Number(agreed),
          total: total === null ? null : Number(total),
          cannotAsk,
          byDesign,
        })
      }
      if (!scenes.length) throw new Error(`${file} has no scene sections`)
      return { file, arm, evidence, scenes }
    })

  if (arms.length < 8) throw new Error(`conform/ has ${arms.length} arms and the site expects eight`)
  return arms
}

mkdirSync(OUT, { recursive: true })
const benchmarks = comparative()
const terminals = conformance()
writeFileSync(join(OUT, 'benchmarks.json'), `${JSON.stringify(benchmarks, null, 2)}\n`)
writeFileSync(join(OUT, 'terminals.json'), `${JSON.stringify(terminals, null, 2)}\n`)
console.log(
  `parsed: ${benchmarks.marginal.rows.length} scenes of the comparative suite, ${benchmarks.caveats.length} caveats, ${terminals.length} terminal arms`,
)
