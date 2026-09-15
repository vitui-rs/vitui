# Status

What has been run, on what, and the numbers behind the two sentences in the README's status section.
This page is the report on the instrument; the README is not the place for it.

## Release

**Released 2026-09-15 at 0.1.0.** `vitui`, `vitui-engine`, `vitui-runtime` and `vitui-components`
are on crates.io with a library in them; the `0.0.1` under each is the placeholder that held the name
and contains no code. There is no stability promise before 0.x.

| crate | state |
|---|---|
| `vitui-engine` | **implementation-complete.** All 26 implementation tickets resolved, all 31 verification-register entries wired, none pinned red. |
| `vitui-runtime` | **implementation-complete.** All 21 tickets resolved. `data`, `layout`, `theme` with its fourteen schemes, `keys`, `ctx`, `id`, `route`, `focus`, `sizing`, `work`, `anim`, `overlay` and `scroll`; the register is 50 entries and the scene list 20, both green. |
| `vitui-components` | **implementation-complete.** All 46 tickets resolved, and the v1 freeze is **29 of 29 components built**, as a value the tests iterate. The register is 238 rows, 233 evaluated with none pinned red, beside 5 unreachable from a crate that cannot name the engine. Every component is exercised under every hostile axis it can meet — 34 of 34 pairs — and all seven documentation and verification obligations are met. |
| `vitui` | facade re-export of the three. |
| `vitui-apps` | 21 applications, one file each, and the surface's only consumer. Never published. |

## What has been run

### Three operating systems, since 2026-09-10

`cargo test --workspace --no-fail-fast` is **green on `windows-latest`**, alongside `ubuntu-latest`
and `macos-latest`. Before that date no hosted runner had ever executed a test in this workspace and
every green run behind the numbers below was a shared local GitLab on one machine: linux/arm64, one
OS, one architecture.

**The seven failures on the first Windows run were all the instrument, and none was the library.**
Six were line endings: nothing here had set a policy for text, GitHub's Windows runners default
`core.autocrlf` to `true`, and the shared idiom of every source-scanning gate is
`source.split_once("\n#[cfg(test)]\n")` — a needle CRLF never matches, so a scan written to stop at
the test module scanned itself. `* text=auto eol=lf` is the whole fix. The seventh was an
unnormalised path walk. Two more rounds followed, both the same shape: the ConPTY quirk row — which
`cfg!(windows) && version.is_none()` makes the floor under every Windows lookup, and which no run had
ever touched — and a wire harness that was comparing SGR separators while claiming to compare colour
narrowing, because four quirk entries force the semicolon form and one of them is ConPTY.

**None of that is an observation of a terminal**, which is why production ticket 16 stays open on its
other five criteria.

### Seven terminal emulator families, none of them on Windows

`conform/` is the fourth party: eight arms over Ghostty, Ghostty through tmux, tmux, kitty,
Terminal.app, WezTerm, Alacritty and iTerm2, each with a committed report. Until 2026-08-23 no
instrument here had ever compared the engine's bytes against a real emulator's screen — the
round-trip suite, the reference compositor and the terminal model they are checked against all live
inside the crate, so a case where the model and the serializer are wrong *in the same direction* was
invisible to every gate.

Terminal.app is the arm that disagrees, WezTerm is the arm that answers wrongly, Alacritty is the one
whose capture is the terminal's own grid rather than an escape stream, and iTerm2 is the one whose
capture is a **projection** of the cell — which is why its three unanswered rows split one and two
across *the terminal cannot* and *this suite cannot see*.

The arms drive emulators that do not exist on Windows and none is executed in CI on any runner — they
are soaks. What has value there is the portable half, and every test in it is a unit test in `src/`:
103 of them. **Windows Terminal is the one supported terminal nobody has driven through this suite,
and its attribute facts are still inference.**

### No architecture question is open

None on any of the three maps. The last five were the components map's and all five resolved on
2026-09-05: an edit that costs the data volume is the other half of the flat-frame obligation and not
a seventh one; `Esc` over a plain `collection` is now declined whenever there is no selection to
clear, so a list in a modal no longer swallows it; `file_picker`'s popup has a keyboard; a `table`
writes the part of its band no column claims; and the demand column answers *draws*, with what a
caller must be able to spell kept beside it.

## What the measurements do not cover

Two things a reader of the README's performance tables should know, because a number without its
exceptions is a claim rather than a measurement.

**The zero-allocation gate reported two, once, in seven runs, and the two were the test harness's
own.** It is a count and not a timing, so the flaky-gate argument that covers the timing budgets
never covered it — and the answer was neither of the two hypotheses that stood here: not the engine's
threads, and not the gate. `libtest` spawns a thread per test even under `--test-threads=1`, and that
thread's first blocking receive allocates once per process; if the parent is descheduled past the
child's window opening, those land in the first window the process opens. The probe now excludes the
process's first thread from every attributed reading, and excludes nothing else — a thread the
*subject* started stays in.

**`macos-latest` cannot sustain 60 Hz**, so the idle and animation jobs fail there on a machine fact
rather than on the percentage; `ubuntu-latest` can. That makes those two a coin toss on hosted
runners. The matrix answers portability; the M1 Max in the README answers performance, and the two
questions are kept apart deliberately.

**The first frame writes all 4 800 cells of a screen the alternate-screen switch had already
blanked** — about 4 900 bytes, once per session. Not fixed: it is a decision about what the engine
may assume of a terminal.

## The registers

A timing is a gate only at cliff granularity, with the headroom written next to the number. The
engine's register is 31 entries, 27 of which gate a build and 4 of which report: **three of the 27
are timings** and the rest are counts, ratios, equalities, orderings, absences and compile outcomes
— a gate tuned to a measurement is a flaky test that gets disabled within a month. The shape of each
is the register's own `qualifier` column, so that split is countable rather than asserted here.

`cargo run --release --example budget -p vitui-engine` prints the register and the ledger. They are
values the tests iterate rather than documents a reader is asked to trust, and every instrument in
them names a file a test opens.
