---
status: accepted
date: 2026-08-27
---

# O1's population is what is built, and a page states its hostile axes as a line

Components ticket 36 discharges spec §17's obligation O1 — *a rustdoc page with a compiled example,
for every component* — and it is the first of the five to turn green. It stood at zero for
thirty-five tickets, twenty-five of which shipped a component that was entitled to add a row to
`obligations::DOC_TESTED` and none of which did, because **a list filled by whichever ticket happened
to write a doctest is a list nobody audits**.

Three decisions the discharge forced are recorded here. Each is easy to undo by a later edit that
looks like a tidy-up, and one of them is a rule about *how* a page is written rather than about what
it says.

## The population is `built`, and getting it the other way round is the vacuity failure in mirror image

§17 states O2's second equality over `built` — *everything `built` must have a panel* — and states
O1's count over nothing at all: *components with 0 doc-tests == 0*. The reading was therefore owed
rather than given, and it is the same population, for the same reason.

`spinner` is the one row of the twenty-nine that no ticket has built. Its mechanism is *a component
that owns a clock*, it is still §22's, and it is components ticket 42's to prototype. **A doc page
for a function that does not exist is not a page anybody can write**, so asking for one puts a
permanent row in the failing set that no ticket on this backlog can invert.

`obligations`'s whole argument is that a query which cannot fail is not evidence — `Verdict::of`
refuses a green verdict over an empty population in its constructor, before it ever looks at the
failing set. **A query stuck red is the same failure in mirror image**: it reads red whatever
happens, so nobody reads it, and a real regression arrives inside a number that was already wrong.

What keeps the choice honest is that the population **moves**. `doc::pages` filters `INVENTORY` on
`built`, so the day `spinner` ships the query asks about twenty-nine with no edit anywhere. The
alternative — a hand-written exception list naming `spinner` — was refused for `MOVED`'s own reason
one file over: an exception list is a place for a second row to be added quietly.

## A page states its hostile axes as one machine-readable line, checked as an equality both ways

O1's fifth criterion is *every component's page states its hostile axes, so a reader knows which of
the four apply before they hit one*. Written as *mention the axes you have*, it is met vacuously by
**thirteen of the twenty-eight built components**, which declare none at all — nearly half the
freeze, where silence and a page that forgot are the same thing on the screen.

So every page carries one line, `**Hostile axes:** …`, whose whole content is a comma-separated list
of the freeze's own words in the freeze's own order, or the word `none`; everything explaining it
goes on the lines beneath. `doc::stated_axes` reads that line and `doc::Found::agrees_about_axes`
compares it to `Component::declares` **in both directions** — a page listing an axis its row does not
set fails exactly as loudly as one omitting an axis its row does.

Two spellings were refused. **A scan for the axis words anywhere in the doc comment** passes on the
day somebody writes *it does not narrow*, and fails on a page that explains why an axis does not
apply — the two mistakes have opposite signs and one scan cannot separate them. **Deriving the line
at build time from the freeze** removes the disagreement the gate exists to find: an equality between
two things derived from each other holds, which is O4's own finding, recorded in `obligations`
before this ticket existed.

## The evidence is two values a test compares, and the second one opens the files

`DOC_TESTED` is written out, in `INVENTORY`'s order, for the reason `AXIS_SCENES` is: a `const fn`
over the freeze would make the population and the evidence one expression. What holds it honest is
`doc::doc_tested`, which derives the same list by opening each page's file, finding the declaration
and reading the doc comment above it; `doc::tests::the_written_list_and_the_scan_agree` is the
comparison.

Three properties of that scan are decisions rather than details:

- **The needle is the name and the boundary is either delimiter.** `pub fn <id>(` **or**
  `pub fn <id><`. This is components 33's rule, met here for the fifth time on this map:
  `file_preview_pane` is `pub fn file_preview_pane<T, F>(` and a needle ending in `(` could never
  have matched it, so a gate green on the parenthesis needle would have been green *by deleting the
  type parameter*.
- **The file is the freeze's and not the name's.** There is a `pub fn text(` in `document.rs`,
  a `pub fn chip(` in `state.rs` and a `pub fn table(` in `gates.rs`, and none of them is a
  component. `Component::module()` — the first entry of the freeze's own `families` column — is what
  disambiguates, so the join that finds a page is the join §19 already gates the module tree with.
- **A `compile_fail` fence is the opposite of evidence.** This crate carries fifteen of them and they
  are the negative gates. A page whose only fence is hostile has proved that a spelling does **not**
  compile, which is exactly not the claim O1 makes, so only a fence rustdoc will run is counted.

## What this does not decide

`deny(missing_docs)` versus `warn(missing_docs)` is **not** a decision, because it was measured to be
the same gate: `[workspace.lints.rust] warnings = "deny"` already turns the warning into an error,
and an undocumented `pub fn` failed the build identically under both. `deny` ships because the two
agree only while that table exists — written `warn`, the crate stops being documented the day
somebody relaxes a lint table three files up, and nothing would say so.

And there is **no sixth CI job**. `cargo test --doc` is spelled `cargo test --workspace` in the
`test` job, whose own comment already records that the invocation runs the doctests and that the
doctests are where the negative gates live. `.gitlab-ci.yml`'s note above its jobs says a sixth means
this pipeline alone can saturate a runner shared with every other repo on the machine.
