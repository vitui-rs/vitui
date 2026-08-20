---
status: accepted
date: 2026-08-19
---

# The collection is one component; `table` and `tree` are it plus an index

`list`, option list, selection list, menu, submenu, context menu, multi-select, tabs, radio set and
segmented control are **one component and one `Mode`**. `table` is that component plus a
caller-supplied column rectangle split. `tree` is that component plus a caller-owned flatten index.
Neither is a sibling of the collection, and neither gets an inventory entry of its own beyond the
name.

The row signature is `fn(&mut Ctx, Rect, usize, Face)`. The store is three facts — a cursor, an
anchor, and a **sorted, disjoint run list** of what is selected.

## Why

**The policy difference between a radio group and a file manager is thirteen match arms.** That is
the whole of it: a menu keeps only the cursor, options and single-select keep one selected element,
multi-select uses all three facts. Shipping six components would be shipping thirteen match arms
six times.

**The run list is forced by one property**: select-all is one run whatever the length, and every
other gesture adds at most one — so the store is proportional to gestures and never to rows. At 1M
rows, `Ctrl+A` is **0.08 µs / 16 B** against a `HashSet`'s **23 438 µs / 16.5 MB** — one keystroke at
234× the frame budget. A `Vec<usize>` is worse than it looks: select-all over 1M costs **8 221 µs on
a steady frame, 82× the budget — but only once the list is scrolled.**

**`table`'s `+` is paid in verbs, not cells.** The same rectangle and the same 23 030 cells is 418
verbs as a list, 640 as a tree and 2 497 as a twelve-column table, because damage is marked per verb
and ADR 0026 makes each column two verbs (text, then padding). Two verbs a cell is the floor.

**`tree`'s `+` is eight bytes a row, and the `depth` field is not for the indent.** It is there so a
collapse can find the interval it removes without touching the forest: 115 µs against 1 187 at
349 524 rows. A table's order is the same record with three fields unused, and a wrap index is the
same record with one field — **three names for one mechanism is one too many**.

**A collection declares one hit entry.** `Response::local` is this frame's answer, so per-row hover
is arithmetic with no frame lag: 229 regions against 389 for a frame-old answer. That one decision
keeps three later defects small — a closed popup costs 4 hit entries and not 400, a table 59 and not
1 195, a text widget 79 and not 621.

**Per-row state is one slot, never a map.** Per-row state over a million rows is state proportional
to data, which the invariant forbids; and the runtime already enforces the other half, since a row
that did not draw cannot be clicked, focused or hovered. `CollState` is 208 bytes at every length.

## Consequences

**The `Face` handed to the row drawer is five non-exclusive bits**, not an enum. A row can be
selected *and* hovered, and can be the keyboard cursor without being selected — an enum names 4 of
the 32 states. `face_paint`'s precedence is part of this decision: disabled > selected+active >
selected > cursor > hovered > base.

**In-cell editing is `Option<(row, col)>` and no wider**, because only one row can hold an editor.
The column half is a **key**, not a position.

**Cell selection is a run list per column key**, never a flattened `row·ncols + col` — at a million
rows one column-header click is 1 000 000 runs, 16 MB and 62 535 µs against 1 run, 16 B and 0.08 µs.

**The frames are flat.** 63.90 µs at 1k, 100k and 1M rows with identical writes and regions; 165.6 µs
for a twelve-column table, flat over both rows and declared columns; 48.79 µs for a million-node
forest at depth 59 999.

**Six inventory entries collapse into one**, which is most of why the v1 freeze is twenty-nine
components rather than thirty-five.

**What this does not buy**: a table needs `visible_cols` as much as `visible_rows` (without it, 120
declared columns cost 3.3× the frame), and a pinned band must be a **view** rather than arithmetic
(otherwise 345 cells are re-damaged every steady frame on identical writes, verbs and time).
