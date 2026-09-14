// Stage the repository's markdown into the docs collection.
//
// **A chapter has one home and this is not it.** `docs/guide/`, `docs/adr/`, `docs/spec/` and
// `CONTEXT.md` are the authority, they are what a reader of the repository sees, and nothing here
// may become a second copy somebody edits. So the staged tree is a build artefact: it is
// regenerated from scratch on every build, it is ignored by git, and an edit made inside it is
// gone the next time anyone runs `npm run build`.
//
// What the transform does is exactly the three things a file written for a repository cannot do on
// a site, and nothing else:
//
//   1. **A title.** Starlight's schema requires one in frontmatter; a markdown file in a repository
//      states it as its first `#` heading. The heading is lifted out and becomes the frontmatter,
//      so the page has one `<h1>` rather than two.
//   2. **Links that resolve.** `[the frame](02-the-frame.md)` is correct on GitHub and a 404 on a
//      site, where the same page is `/guide/02-the-frame/`. Anything pointing outside what the site
//      carries becomes a link into the repository at the commit the build ran on.
//   3. **Angle brackets that survive.** A markdown renderer treats `<Left>` as an HTML tag and
//      deletes it. Inside code — fenced or inline — it is already safe, and outside it is escaped.
//
// Everything else is passed through untouched, which is the property that lets the same file be
// read in three places.

import { readFileSync, writeFileSync, mkdirSync, rmSync, readdirSync, existsSync, copyFileSync } from 'node:fs'
import { dirname, join, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
export const REPO = join(here, '..', '..')
const DOCS = join(here, '..', 'src', 'content', 'docs')
const PUBLIC = join(here, '..', 'public', 'img')

/** The repository at the revision the site was built from, for links the site does not carry. */
const BLOB = 'https://github.com/vitui-rs/vitui/blob/master'
/** Where an edit to a staged page has to be made, which is never the staged copy. */
const EDIT = 'https://github.com/vitui-rs/vitui/edit/master'

/** The staged directories. Everything under them is deleted and rewritten on every run. */
const STAGED = ['guide', 'adr', 'spec', 'reference']

/**
 * The first `# ` heading, and the body with that line removed.
 *
 * A file with no heading is a staging error rather than a page with a filename for a title: every
 * document in this repository opens with one, and a file that does not is a file this script has
 * been pointed at by mistake.
 */
function title(source, from) {
  const lines = source.split('\n')
  const at = lines.findIndex((l) => l.startsWith('# '))
  if (at === -1) throw new Error(`${from} has no '# ' heading, so it has no title to stage`)
  const text = lines[at].slice(2).trim()
  lines.splice(at, 1)
  // The blank line the heading left behind, so the body does not open on one.
  while (lines.length && lines[0].trim() === '') lines.shift()
  return { text, body: lines.join('\n') }
}

/** YAML frontmatter, if the file carries any, as a map of the scalar keys it holds. */
function frontmatter(source) {
  if (!source.startsWith('---\n')) return { keys: {}, rest: source }
  const end = source.indexOf('\n---\n', 4)
  if (end === -1) return { keys: {}, rest: source }
  const keys = {}
  for (const line of source.slice(4, end).split('\n')) {
    const at = line.indexOf(':')
    if (at > 0) keys[line.slice(0, at).trim()] = line.slice(at + 1).trim()
  }
  return { keys, rest: source.slice(end + 5) }
}

/**
 * Split a markdown source into code and not-code, so a rewrite can be applied to prose alone.
 *
 * Fenced blocks and inline spans both count as code. The alternative — one regular expression over
 * the whole file — rewrites the contents of examples, which is how a code sample stops compiling
 * for a reader who copies it.
 */
function segments(source) {
  const out = []
  const fence = /^(\s*)(```+|~~~+)/
  let prose = []
  let inFence = null
  for (const line of source.split('\n')) {
    const m = fence.exec(line)
    if (inFence) {
      out.push({ code: true, text: line })
      if (m && m[2].startsWith(inFence)) inFence = null
      continue
    }
    if (m) {
      if (prose.length) out.push({ code: false, text: prose.join('\n') }), (prose = [])
      out.push({ code: true, text: line })
      inFence = m[2]
      continue
    }
    prose.push(line)
  }
  if (prose.length) out.push({ code: false, text: prose.join('\n') })
  return out
}

/**
 * Apply `f` to the prose of a markdown source, leaving every fenced code block alone.
 *
 * `spans: false` steps over inline code as well. The two callers want different answers and the
 * difference is not a nicety: a link whose text is inline code — `` [`CONTEXT.md`](../CONTEXT.md) ``
 * is one, and the guide's index is built out of them — is **split in three** by a pass that treats
 * spans as opaque, so the link regex sees `[` and `](…)` in different pieces and matches neither.
 * Escaping angle brackets wants the opposite, because `Vec<T>` in a span is already safe.
 */
function overProse(source, f, { spans = false } = {}) {
  return segments(source)
    .map((s) => {
      if (s.code) return s.text
      if (spans) return f(s.text)
      // Inline spans, split on runs of backticks so that ``a ` b`` survives.
      return s.text
        .split(/(`+[^`]*`+)/g)
        .map((part, i) => (i % 2 === 1 ? part : f(part)))
        .join('')
    })
    .join('\n')
}

/** `<Left>` is a tag to a markdown renderer and a key name to a reader. */
function escapeAngles(text) {
  // An autolink (`<https://…>`) is the one angle bracket that means what it says.
  return text.replace(/<(?!https?:\/\/)/g, '&lt;')
}

/**
 * Rewrite one link target.
 *
 * The rule is *a link the site can serve points at the site, and everything else points at the
 * repository* — never at a path that resolves in neither place, which is what a relative link into
 * `docs/` becomes once a file is served from a route.
 */
function retarget(href, from) {
  if (/^(https?:|mailto:|#|\/)/.test(href)) return href

  const [path, hash = ''] = href.split('#')
  const anchor = hash ? `#${hash}` : ''
  const slug = (file) => basename(file, '.md').toLowerCase()

  // Within the guide: a sibling chapter. From anywhere else: the same chapter by its path.
  if (from === 'guide' && /^\d\d-[^/]+\.md$/.test(path)) return `/guide/${slug(path)}/${anchor}`
  if (from === 'guide' && path === 'README.md') return `/guide/${anchor}`
  if (/(^|\/)guide\/\d\d-[^/]+\.md$/.test(path)) return `/guide/${slug(path)}/${anchor}`
  // A decision record, from inside the set or from a document citing one.
  if (from === 'adr' && /^\d{4}-[^/]+\.md$/.test(path)) return `/adr/${slug(path)}/${anchor}`
  if (/(^|\/)adr\/\d{4}-[^/]+\.md$/.test(path)) return `/adr/${slug(path)}/${anchor}`
  // The ADR directory, from anywhere.
  if (/(^|\/)adr\/?$/.test(path)) return `/adr/${anchor}`
  // The glossary, wherever it is reached from.
  if (/(^|\/)CONTEXT\.md$/.test(path)) return `/reference/glossary/${anchor}`
  // A spec, from anywhere.
  const spec = /(^|\/)spec\/(engine|runtime|components)\.md$/.exec(path)
  if (spec) return `/spec/${spec[2]}/${anchor}`

  // Everything else is a file in the repository: source, an example directory, a script. The site
  // does not carry those and will not pretend to.
  const cleaned = path.replace(/^(\.\.\/)+/, '').replace(/^\.\//, '')
  return `${BLOB}/${cleaned}${anchor}`
}

/** Every markdown link and image in the prose, retargeted. */
function retargetLinks(source, from) {
  return overProse(
    source,
    (text) =>
      text.replace(/(!?\[[^\]]*\]\()([^)\s]+)(\))/g, (_, open, href, close) => open + retarget(href, from) + close),
    { spans: true },
  )
}

/** A one-sentence description for the page's metadata, or nothing when the file has no prose. */
function description(body) {
  // A **paragraph**, not a line: markdown wraps prose at the column the author liked, so the first
  // line of a paragraph is usually half a sentence and reads as one in a search result.
  const paragraphs = []
  let current = []
  let inFence = null
  for (const line of body.split('\n')) {
    const t = line.trim()
    // A fence's *contents* are not prose, and skipping only the fence line itself takes the first
    // line of a code block as the page's description — which is what chapter 2 shipped with for
    // exactly one build.
    const fence = /^(```+|~~~+)/.exec(t)
    if (inFence) {
      if (fence && fence[1].startsWith(inFence)) inFence = null
      continue
    }
    if (fence) {
      inFence = fence[1]
      if (current.length) paragraphs.push(current.join(' ')), (current = [])
      continue
    }
    if (!t) {
      if (current.length) paragraphs.push(current.join(' ')), (current = [])
      continue
    }
    if (/^(```|~~~|[#>|\-*]|!\[|---)/.test(t)) {
      if (current.length) paragraphs.push(current.join(' ')), (current = [])
      continue
    }
    current.push(t)
  }
  if (current.length) paragraphs.push(current.join(' '))

  for (const paragraph of paragraphs) {
    const plain = paragraph
      .replace(/\*\*|\*|`/g, '')
      .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
      .replace(/&lt;/g, '<')
      .replace(/\s+/g, ' ')
      .trim()
    // A status line is metadata wearing a sentence's clothes, and it describes every spec alike.
    if (plain.length < 40 || /^status:/i.test(plain)) continue
    if (plain.length <= 160) return plain
    const cut = plain.slice(0, 157)
    const at = cut.lastIndexOf(' ')
    return `${(at > 100 ? cut.slice(0, at) : cut).trimEnd()}…`
  }
  return null
}

/** YAML for a string that may carry anything, quoted the one way that always holds. */
const yaml = (s) => `"${s.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`

/**
 * Stage one file.
 *
 * `extra` lands under the title as ordinary markdown — the ADRs' `status` and `date`, which are
 * frontmatter in the repository and would be rejected by Starlight's schema, so they become a line
 * a reader can see instead of metadata nobody renders.
 */
function stage(source, target, { from, order, extra = '', home }) {
  const raw = readFileSync(source, 'utf8')
  const { keys, rest } = frontmatter(raw)
  const { text, body } = title(rest, source)
  const staged = escapeAngles(retargetLinks(body, from))
  const desc = description(staged)

  const head = ['---', `title: ${yaml(text)}`]
  if (desc) head.push(`description: ${yaml(desc)}`)
  // **The edit link points at the home and not at the staged copy.** Starlight's default is *the
  // file this page was built from*, which for everything under here is a build artefact — so the
  // one link on the page whose whole job is *go and fix this* would send a reader somewhere their
  // change is deleted on the next build.
  if (home) head.push(`editUrl: ${yaml(`${EDIT}/${home}`)}`)
  if (order !== undefined) head.push('sidebar:', `  order: ${order}`)
  head.push('---', '')

  const note = typeof extra === 'function' ? extra(keys) : extra
  mkdirSync(dirname(target), { recursive: true })
  writeFileSync(target, `${head.join('\n')}${note}${staged.trimEnd()}\n`)
  return { title: text, slug: basename(target, '.md'), description: desc, status: keys.status }
}

/** Everything under the staged directories, so a renamed source cannot leave a page behind. */
function clean() {
  for (const dir of STAGED) rmSync(join(DOCS, dir), { recursive: true, force: true })
}

/**
 * The index over the decision records, which is generated for the reason the gallery is: a
 * hand-written list of fifty-three is a list that loses one.
 */
function adrIndex(records) {
  const rows = records.map((r) => `| ${r.number} | [${r.title}](/adr/${r.slug}/) | ${r.status ?? ''} |`)
  return [
    '---',
    'title: "Decision records"',
    'description: "Fifty-three decisions that are hard to reverse and surprising without the argument behind them."',
    'tableOfContents: false',
    'editUrl: false',
    '---',
    '',
    'Fifty-three decisions that are hard to reverse and surprising without the argument behind them.',
    'Each record states what was decided, what it cost, and what was measured — several of them exist',
    'because the obvious answer was tried first and lost.',
    '',
    '| № | Decision | |',
    '|---|---|---|',
    ...rows,
    '',
  ].join('\n')
}

function markdownIn(dir) {
  return readdirSync(dir)
    .filter((f) => f.endsWith('.md'))
    .sort()
}

/**
 * The recordings and the two architecture diagrams, copied rather than imported.
 *
 * They are binaries and they have a home already — `docs/img/`, where the repository's own README
 * embeds them. A copy into `public/` is a build artefact on the staged markdown's terms: ignored by
 * git, rewritten every build, and never the thing anybody edits.
 */
function images() {
  const from = join(REPO, 'docs', 'img')
  rmSync(PUBLIC, { recursive: true, force: true })
  mkdirSync(PUBLIC, { recursive: true })
  const files = readdirSync(from).filter((f) => /\.(gif|png|svg|webp|jpg)$/.test(f))
  for (const f of files) copyFileSync(join(from, f), join(PUBLIC, f))
  return files.length
}

function main() {
  clean()

  // The guide. `README.md` is its index; the thirteen chapters sort by their own numbers.
  const guide = join(REPO, 'docs', 'guide')
  stage(join(guide, 'README.md'), join(DOCS, 'guide', 'index.md'), {
    from: 'guide',
    order: 0,
    home: 'docs/guide/README.md',
  })
  for (const file of markdownIn(guide)) {
    if (file === 'README.md') continue
    stage(join(guide, file), join(DOCS, 'guide', file), { from: 'guide', home: `docs/guide/${file}` })
  }

  // The three specs, which are the authority for the three layers.
  const spec = join(REPO, 'docs', 'spec')
  for (const file of markdownIn(spec)) {
    stage(join(spec, file), join(DOCS, 'spec', file), { from: 'spec', home: `docs/spec/${file}` })
  }

  // The glossary. Its words are used exactly as it defines them, everywhere.
  stage(join(REPO, 'CONTEXT.md'), join(DOCS, 'reference', 'glossary.md'), {
    from: 'reference',
    home: 'CONTEXT.md',
  })

  // The decision records, with their frontmatter turned into a line on the page.
  const adr = join(REPO, 'docs', 'adr')
  const records = []
  for (const file of markdownIn(adr)) {
    const number = basename(file, '.md').slice(0, 4)
    records.push({
      number,
      file,
      ...stage(join(adr, file), join(DOCS, 'adr', file), {
        from: 'adr',
        home: `docs/adr/${file}`,
        extra: (keys) =>
          keys.status || keys.date
            ? `**${keys.status ?? 'unknown'}**${keys.date ? ` · ${keys.date}` : ''}\n\n`
            : '',
      }),
    })
  }
  writeFileSync(join(DOCS, 'adr', 'index.md'), adrIndex(records))

  const pictures = images()
  console.log(
    `staged: ${markdownIn(guide).length} guide, ${markdownIn(spec).length} spec, ${records.length} adr, 1 glossary, ${pictures} images`,
  )
}

if (existsSync(join(REPO, 'Cargo.toml'))) main()
else throw new Error(`the repository is not where this script expects it: ${REPO}`)
