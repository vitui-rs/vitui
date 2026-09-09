# 12. Verifying a component

Most of the defects found in this codebase were in the **instrument**, not in the component. This
chapter is the instruments that work and the traps that have bitten more than once.

## Four instruments

### 1. A counter, through the `Ink` seam

The third spelling of every component takes its writer as a parameter, so a test can substitute one
that records:

```rust
let mut tally = ink::Tally::default();
driver.frame(|cx| {
    meter::meter_into(&mut tally, cx, area, 0.37, &Default::default());
});
assert_eq!(tally.distinct(), usize::from(area.w) * usize::from(area.h));
assert_eq!(tally.excess(), 0);
```

Nine counters are worth having and they answer different questions: `writes`, `distinct cells
touched`, `verbs`, `marked`, `regions`, `tab stops`, `merges`, `content layers`, `allocations`.
`marked` — the engine's damaged-cell count — is not reachable from above the engine, and saying so
is better than approximating it.

### 2. A reference render

Draw the same scene twice — once with the component, once with an obviously-correct and far too slow
routine — and compare cell for cell. This is the instrument that catches the defects **no counter
disapproves of**, and every one of them made the broken build look *healthier*:

| Defect | What the counters said |
|---|---|
| An inverted scroll | drew nothing, at 12 µs against 63 |
| A stale tail, 71 of 80 rows wrong | 2.3× faster, marking 226× less |
| Twenty wheel clicks moving the offset by 0 | no counter has an opinion about an offset |

### 3. A golden screen

The composited picture, committed, regenerated behind an environment variable and reviewed as a
diff. It catches what a round trip cannot: the round trip asserts that what was serialised replays
into the frame that produced it, and it cannot see a defect the serialiser and the model share.

A golden **byte string** is refused, deliberately — the encoding is exactly the part that is allowed
to change.

### 4. A real terminal

Everything above is headless, and a headless gate cannot answer *what did my terminal claim*. If you
ship claims about terminals, something has to ask one.

## The traps

**A scanner looking for a literal contains that literal.** Assemble needles from fragments, or scan a
**bounded** region — a function's own body. One such gate here had two satisfiers and neither was a
call: the line holding the needle, and an `#[expect]` reason string naming the same function.

**A gate that cannot fail.** Reading a baseline *after* the call it measures compares a value with
itself. An equality between two derivations of one declaration holds for ever. An equality between
two things that do not exist holds. **Watch every gate fail** before you trust it — and note that
if warnings are denied, deleting the call under test turns an unused item into a build failure, so
empty the body instead.

**A gate over a state nothing has moved cannot see a defect that arrives on a change.** If the
property's defect arrives on an *edit*, inspect after an edit; a forward walk of a fresh corpus will
not find it.

**A check that re-runs the walk that produced the value cannot fail.** *Is this caret on a cluster
boundary* is a membership test against the same walk that placed it — 1 296 inspections reported
zero. When a property holds by construction, say so, and put the observations where a value enters
from **outside** the construction.

**Every gate plays the component at the full size of its context.** Two expressions that differ only
when the rectangle and the screen disagree are one expression. Play the subject inside something
smaller, at a size where one clamp binds and the other does not.

**A recorder and the defect share a coordinate system.** Union in root coordinates, or a child's
`(0, 0)` and its parent's are one cell.

**A clipped verb is recorded at the column it was asked for.** Harmless while a component writes
inside its own rectangle — and every overrun *is* a component writing outside one, which is exactly
when you are looking. Where a verb might overrun, write one cell at a time: per cell a write lands
whole or is discarded whole.

**Counters on the wrong side of the question.** `changed > 0` is green on the exact set it exists to
catch. Every *output* counter is blind to work that produces no output. And a **consumption** counter
is the same shape one axis over: *was the key consumed* cannot tell a key consumed to do something
from a key consumed to do nothing. Ask what the component did with it.

**`cargo test` does not run an example.** An `assert!` in an example binary is compiled by
`cargo clippy --all-targets` and evaluated by nobody. Several here rotted for eight tickets.

**Cumulative ledgers must be read as deltas**, and allocation windows counted per frame and warmed on
the *shape* rather than on two identical frames.

**A figure quoted from a design document is usually a prototype's screen.** Measure it. Assert what
reproduces as measured, print what does not beside it, and never bend the code to make an old
sentence true.

**A headless gate cannot see a wire-spelling bug**, because it posts the spelling the test author
typed. `Chord::typed` versus `Chord::key` is not testable this way; a conformance run against a real
terminal is.

## Hostile axes

Four axes, and every component should be asked which of them it declares:

| Axis | The question |
|---|---|
| **Scrolled** | does it draw correctly at a non-zero offset? |
| **Shrunk** | does it draw correctly when the rectangle is smaller than its content? |
| **Wheeled** | does a posted notch move it? |
| **Narrow** | does it stay inside its rectangle at a width where its label does not fit? |

Three of the four in this codebase were caught **only** by a reference render. And two of them are
invisible at the origin: at offset `(0, 0)` several possible sign conventions agree, and a gate
played there separates none of them. Play at a non-zero offset, with the origin as the control.

## Writing the test that finds things

The two habits that found the most:

**Play the component where an application puts it**, not where its author put it. A gate draws one
subject, at the origin, with nothing above it and nothing closing. An application draws two of them,
inside a panel, at a width that truncates, and closes a dialog over the top.

**Write an application.** Every category of defect in the list above was found by one — the identity
trap from two file panels, the deaf sink from a dialog beside a text field, the key-release
double-count from a list that scrolled two rows for one press. None of them was visible to a test
that draws one component and asserts about its cells.
