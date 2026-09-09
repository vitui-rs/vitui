# The component handbook

The worked example behind [`docs/guide/`](../../docs/guide) — five components written against
`vitui-runtime` alone, and the loop that drives them.

```sh
cargo run              # the application
cargo run -- --probe   # one headless frame, drawn through a counter, numbers printed
cargo test             # four gates, one per rule the guide claims
cargo clippy --all-targets
```

`Tab` moves the focus, `←`/`→` and `+`/`-` drive the stepper, `j`/`k` and the wheel drive the list,
the button opens a menu, and `q` or `Ctrl+Q` leaves.

## What is here

| File | The rule it is there to carry |
|---|---|
| `src/ink.rs` | The seam that lets something other than the terminal watch a component draw |
| `src/meter.rs` | Drawing, roles, glyphs, and the partition rule — every cell once |
| `src/pill.rs` | Interest, `Response`, one paint resolved before any cell is written |
| `src/stepper.rs` | A declared keyboard contract, and declining what you cannot act on |
| `src/picker.rs` | Caller-owned offset, virtualisation, the wheel |
| `src/menu.rs` | An overlay body, and the two states an overlay owner needs |
| `src/main.rs` | The loop, the once-clear, focus seating, and the four tests |

## Why it is a detached crate

Two reasons, and the first is the one that matters.

**It is meant to be copied.** So it has to build the way a stranger's crate builds: a
`[dependencies]` table naming what it needs, no workspace inheritance, and nothing reachable that a
dependent would not have.

**It depends on `vitui-runtime` and not on `vitui-components`.** That is the claim the guide makes —
a component is a function over a `Ctx`, and the component library is one consumer of that surface
rather than a privileged one. If anything here needed a name only `vitui-components` exports, the
guide would be wrong.

Being a workspace member would also make it a twenty-second application in `crates/vitui-apps`,
where the list of applications is something several gates join on.

## The numbers the probe prints

```
verbs            61
cells written    1160
distinct cells   1160
written twice    0
regions declared 4
tab stops        3
```

`cells written == distinct cells` and `written twice == 0` is the partition rule holding across four
components at once: 40 + 40 + 120 + 960 cells, each painted exactly once. It is the cheapest gate in
the guide and it catches the most common component defect there is.
