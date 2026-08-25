//! The component library: windows, panels, charts, lists, trees, forms, pickers.
//!
//! # Status
//!
//! Being built one ticket at a time from `.scratch/vitui-components-architecture/spec.md`, whose
//! map is closed; the backlog is `.scratch/vitui-components-impl/`, forty-three tickets.
//!
//! **Nine of the twenty-nine components are written** — [`text::text`], [`text::chip`],
//! [`input::button`] and [`structure::panel`] (components ticket 10), [`collect::collection`]
//! (components ticket 12), [`chart::chart`] and [`chart::plot`] (components ticket 28),
//! [`collect::table`] (components ticket 15) and [`collect::tree`] (components ticket 17) — and the
//! dense screen, the listing, the grid and the forest are
//! now drawn *through* them
//! rather than through their construction. They are spec §1's four rules with
//! **two stated substitutions**: [`Rect`] stands in for `Rect`, which cannot be named from a
//! package whose dependency table is `vitui-runtime` and nothing else; and
//! [`structure::Panel`] stands in for a bare `Response`, because §2's *the cells it does not write
//! are named in its return value* is unwritable in one and the closure form that would make it
//! writable is refused on two measurements ([`frame`]). Neither is the rule being ignored, and each
//! says so where a reader coming from §1 will look.
//!
//! Beside them is [`app::Clears`], which is not a component and cannot be: *the correct build clears
//! once, on its first frame and on a resize* is a statement about a **sequence** of frames, so it
//! needs a value the application keeps.
//!
//! The rest of what exists is the instruments the other twenty-five are enumerable against, and
//! every one of them is a value rather than a paragraph:
//!
//! - [`INVENTORY`] — spec §17's v1 freeze as a value: **twenty-nine components in three tiers**,
//!   eleven columns each, with [`MOVED`] and [`COMPOSITIONS`] beside it. Not a paragraph, because
//!   *every obligation this map has stated as a sentence has been broken by someone who had read
//!   it* (ADR 0033).
//! - [`obligations`] — §17's five obligations as queries over the freeze, each returning a count or
//!   an equality. **Not one of them can be met yet, and every one of them says so out loud** rather
//!   than returning green over an empty population.
//! - [`gates`] — §21's register: **a hundred and twenty-one gates as rows, a hundred and one of
//!   them evaluated**,
//!   six pinned red with their failing sets, six unreachable across the crate line with what
//!   would have to become public, and eight with nothing yet to run over. An instrument is a
//!   file in it, so a row that has stopped running turns the register red here. **One of those rows
//!   was not unreachable and had said it was for five tickets** — see that module's header, because
//!   the shape it names is the one an `Unreachable` invites. **A fourth was red and is now green,
//!   and inverting it rewrote the gate**: row 61 asserted an *absence*, and a row still asserting
//!   the four primitives do not exist would now be asserting they are gone.
//! - [`glyphs`] — §16's catalogue as a value: **six families over twenty entries and ten
//!   distinctions**, with the demand column of [`INVENTORY`] joined against it. A distinction
//!   survives the whole matrix iff it is carried on both axes (ADR 0032), and seven of the ten name
//!   a glyph pair — so the gate that matters is **cross-family** collapse and not the pairwise
//!   version, which fires on every border because the nine box-drawing entries are all `+` at ASCII
//!   on purpose.
//! - [`counters`] — §20's nine per-frame counters, **eight of which this crate can read**. The
//!   ninth, `marked`, panics rather than answering `0`, and so does the sentinel probe.
//! - [`scenes`] — the normative scene list: **thirty-two screens, twenty-seven of them §21's
//!   table, eight `Unsubjected`, eleven `Red` and thirteen `Evaluated`**, each with the size it is
//!   played at, the content it stands up, the gestures it plays and the property it decided. Three
//!   of them exist because a defect survived every gate then in force by not being on any screen
//!   anybody had built. **Scenes 8 and 9 — the million-node forest and the fold over 349 524 rows —
//!   went green together with components ticket 17**, which is one fact and not two: both were
//!   pinned on `tree` being undeclared. Of the eleven still red, only the wheel gate is red because
//!   the defect is *real* rather than because a subject is missing — components 12 turned the other
//!   four of components 11's five and deliberately left it.
//! - [`runner`] — *render one scene two ways and compare it cell for cell*, reporting **n cells over
//!   m rows**. Three of the four hostile axes were caught only by this, and every one of them made
//!   the defective build look **healthier**. The reference arm is this crate's own, and
//!   `runner`'s header names the four barriers that make the engine's unreachable — the first three
//!   of which are about `mod reference;` and the fourth about ADR 0023.
//! - [`dense`] — **the dense screen**: 300×80, 338 interactive regions, and ADR 0026's five
//!   re-damage instances each standing on a screen instead of in a sentence — 9 024 / 1 095 / 324 /
//!   600 / 15 against a correct arm's **0**, with the naive twin kept beside it and proved equal
//!   cell for cell at 300×80 and at 120×40. It is the screen components ticket 10 is proved
//!   against, and **its three scenes now stand**: the screen is drawn through `text`, `chip`,
//!   `button` and `panel`, and every defective arm is a defective *component* rather than a branch
//!   in the screen. One figure moved when the components landed and it is written down rather than
//!   absorbed — [`dense::CHIP_FILLED_FACE`] is 1 095 where ticket 09 measured 1 149.
//! - The module tree, one module per family (§19), joined to the freeze by
//!   [`Component::families`].
//!
//! - [`accordion`] — **the accordion of twelve sections, and the fold set beside it**: 300x80,
//!   twelve headers over bodies of thirty-four widgets each, and the one hostile axis whose defect
//!   **no golden-cell gate can see**. A closed body that is drawn into an `h = 0` rectangle instead
//!   of skipped declares **408 more hit entries and 408 more ring entries** — the same subtraction
//!   twice, because `Ctx::interact` appends to both before either looks at the rectangle — on a
//!   surface that is **0 cells over 0 rows** apart. Of §20's nine counters, `regions` and
//!   `tab stops` see it and nothing that reads a cell does. §8's six declaration figures are this
//!   screen's plus one constant pair fitted from the first of them, and the pair predicts the other
//!   five to the unit, the mid-transition **273 against 247** included. Beside it, §21's fold row
//!   reproduces exactly and needs no component at all: ten lines inserted at line 24 of a
//!   200 000-line document leave **4 166 of 4 167** folds on a line that opens no block, and 0 when
//!   they are reanchored. Its two scenes are **red**, both waiting for `collapsible`
//!   (components 22).
//!
//! - [`listing`] — **the collection's screen**: 40x80, one collection and eighty rows a window, and
//!   the four hostile axes standing on it instead of in a table — 75 of 80 rows for the inverted
//!   scroll sign, 71 of 80 (2 840 cells) for the stale tail, one cell a row for the missing
//!   ellipsis, and **0 against 20** for the unconditional `scroll_into_view` `CONTEXT.md` forbids
//!   and four *resolved* tickets wrote anyway. One of its five scenes is still red, and the two
//!   reasons were kept apart from the start: four waited for `collection` (components 12, done)
//!   and the wheel gate waits for
//!   the fix (components 20). Its own finding is that *writes flat 1k -> 1M* is **green** on a
//!   listing that declares a million hit entries, which is why the equality it registers is on
//!   `regions`. **Four of the five are green since components 12**, which declared
//!   [`collect::collection`] and rewrote both arms of `listing::draw_into` to draw through it —
//!   `Volume::Windowed` is the component and `Volume::WholeContent` is its `defective` twin, one
//!   value apart.
//!
//! - [`collect`] — **`collection`, the component the rest of this library is mostly made of**:
//!   one component, one [`collect::Mode`] and **thirteen match arms**, counted by opening the file
//!   rather than declared. `list`, option list, menu, multi-select, tabs, radio group and segmented
//!   control are those four modes plus the caller's row drawer, and no row of [`INVENTORY`] carries
//!   one of the seven names. Select-all is **one span and 16 bytes at any length** against a
//!   `HashSet`'s 13 381 µs and 18.9 MB at a million rows; one hit entry a collection whatever the
//!   volume, with per-row hover resolved by arithmetic on **this frame's** pointer; the scan cursor
//!   at one `partition_point` a frame against one a row; and 3 200 writes / 81 regions / 1 stop /
//!   0 allocations at 1 000, 100 000 and 1 000 000 rows, at 52.8–53.0 µs.
//!
//! - [`collect::table`] — **`table` = `collection` + column rectangles, and the `+` is paid in
//!   verbs** (components ticket 15). The same rectangle costs **24 000 cells and 160 verbs** as one
//!   column and **24 000 and 1 600** as twelve, so the cells are identical and the verbs are the
//!   difference — §6's structure, on a screen that is one table rather than §6's two tables and
//!   three bars. Three bands, the middle one a **view** opened per row; a pinned column that may
//!   not be elastic because [`collect::Pin::width`] is the only reader of its width; cell selection
//!   as **one run and sixteen bytes** against a flattened index's **1 000 000 and 16 MB** on one
//!   header click, with the loss (forty runs against one for a whole row) asserted beside the win;
//!   and an editing slot that is `Option<(row, col)>` where the row half is [`collect::collection`]'s
//!   own revalidation and the column half is a key. **Two of §6's three remembered figures did not
//!   reproduce and each is written down**: *a pass per band measures 4% cheaper* is inside ±1.5%
//!   over sixty interleaved rounds with its **sign flipping between runs**, so the refusal stands on
//!   §6's argument rather than on a clock; and the identity numbers are this screen's population
//!   rather than the prototype's.
//!
//!   Its own finding is a defect one crate down. **`Ctx::with_key` inside a scroll scope draws
//!   nothing past the first screenful** — `with_id` re-childs at `self.area()`, which is the
//!   *content's* origin there — so a container cannot key a child per row inside a virtualised
//!   body: 0 cells of 8 at an offset of 100, and 8 of 8 at zero, which is why nothing had seen it.
//!   `collection` had already worked around it without naming it; `table` cannot take the same
//!   workaround, so it mints each cell's id with `Id::keyed` and hands it over on
//!   [`collect::Cell::id`]. Filed as `.scratch/vitui-runtime-architecture/issues/31` and pinned as
//!   register row 112.
//!
//! - [`order`] — **the order, the index and the memo**: `table`'s sort order, `tree`'s flatten
//!   index, `textarea`'s wrap index, `collapsible`'s fold index and `table`'s prefix sum are one
//!   `{ node, depth, flags, h }` record, and [`order::USES`] is the five names against it as a
//!   value so that *which fields does a wrap index spend* is answerable by the machine. [`order::Rows`]
//!   is `{ len, rev }`, **one `u64` compared once a frame**, and it is the only thing that makes a
//!   stale position noticeable — a sort changes no data and no length, so the frame after one draws
//!   a perfectly correct list with the wrong rows selected. The four reconcile policies are there
//!   with their settled defaults and **the one that is refused**, and the memo is
//!   [`order::Keyed`], which takes the whole key and **records the input it was built at** —
//!   because `recomputes` goes *down* when a key has forgotten one.
//!
//! - [`area`] — **the scroll area's screens**: the bar fixpoint over 5 475 600 viewport x extent
//!   pairs (0 failures, 3 passes, 0 unneeded bars), the spelling a reader writes instead — which
//!   keeps both bars over content that fits and loses 19 of 20 rows and 19 of 20 columns for ever —
//!   `Σ h` against the row count at row **799 999 of 999 999** with **no counter separating the two
//!   builds**, and an overlay bar re-damaging **398 cells a frame against 0**. Its four scenes are
//!   red and all four wait for `scroll_area`, `scrollbar` and `sticky` (components 19). Three of
//!   §9's figures do not reproduce and each disagreement is a result: **3 535 is a span-model
//!   picture of a ~423-cell double write** and the engine ships the structure that makes the span
//!   model 1.00x; the thumb's *up to 7 of 69* is the figure at **45.6%** of the reachable range and
//!   14 at its end; and the wrong pairing's microseconds are another screen's, so the gate is the
//!   mechanism — **100 000 rows iterated against 69**. It also found the one defect on this page
//!   that is not a remembered number: [`area::SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY`].
//! - [`forest`] — **the tree's screen**: 300×80 at **depth 59 999**, where the one defect on this
//!   map that only a *scene* can see is stood up. An unclamped indent asks for **9 599 840 cells
//!   against 24 000** and every other counter this crate can read prefers it — same writes, same
//!   distinct, same regions, same stops, **fewer verbs** and less time, every run — while at depth
//!   ten the two builds are one frame field for field. Its own finding is that the instrument
//!   saturates before the defect does: `Tally::asked` folds a `u16`, so 119 998 a row is reported
//!   as 65 535. Beside it, the fold: **1 run against 249 940** for a half-tree selection, and
//!   **349 526 of 500 000** back from the round trip under the splice the prototype shipped. Both
//!   scenes are **red**, both waiting for `tree` (components 17).
//! - [`popup`] — **the overlay family's screen**: 300x80, 312 chips, two `select`s and a menu bar,
//!   and §12's five configurations standing on it instead of in a table. **Every `regions` and every
//!   `stops` figure of that table reproduces exactly** — 317/316, 319/317, 323/322, 319/318 — and
//!   they are arithmetic rather than measurements, the one region that is not a stop being the menu
//!   bar itself. Beside them: the trap's named exception at **2 stops visited of 318 declared**,
//!   which is §21's *name the exception; do not loosen the gate* on the first screen that can run
//!   it; the opening cliff **counted 30 of 30** rather than timed; the scrim in three spellings at
//!   600 / 0 / 0 re-damaged and 24 600 / 24 000 / 600 written; **99 flips in 100 frames** for an
//!   overlay that declares and covers its anchor; **3 of 8 frames** for a dialog owned by the menu
//!   row that opened it; and three different ids for the three answers to *where does the keyboard
//!   go when a modal closes*. Its scene is **red**, waiting for `select` and `overlay`
//!   (components 26). **Two of §12's seven columns contradict the shipped runtime and are asserted
//!   as measured**: `allocations`, whose zero was the bump arena ADR 0034 deleted, and `content
//!   layers`, whose 2 / 4 / 3 counted a shadow layer `OverlayOpts` has no field for.
//!
//! **And all seven of spec §3's helpers now exist**, which is the first code here that a component
//! will call rather than be measured by: [`text::fit`] and [`frame::block`] are the two partition
//! primitives; [`state::press`] returns **one role** so that the face a widget draws and the face it
//! asks to be awarded cannot disagree; [`frame::face_paint`] is the one place a row's five
//! independent bits collapse to a paint, over all **32** states rather than C02's four;
//! [`scroll::bar`] draws the thumb before the track; [`keys::text`] is `CTRL | ALT` with Shift
//! excluded, which is **one bit narrower than R12's `INTENT` and that bit is the whole helper**; and
//! [`nav::cursor`] is what a `Group` moves with, arming the one `deadline_for` a type-ahead buffer
//! owes. Each carries the number the alternative costs, watched firing: 15 cells for a border run
//! over its own title, 16 224 for a `block` that clears what it hands over, **8 a frame for as long
//! as a pointer rests** on a chip whose two statements disagree, 224 for a groove written under its
//! own thumb, `"value 0shi"` against `"value 0hi"` for a field that reads `code` alone — and
//! `"i"` where `"Hi"` is right for the over-correction beside it — and **138 tab stops against 36**
//! for twelve collections that did not open a `Group`.
//!
//! # The two rules a reader of this crate needs first
//!
//! **The crate line is `vitui-runtime` and nothing else** — constraint C6, and it is checked by a
//! test in the runtime that reads this package's `[dependencies]` table. It does not depend on
//! `vitui-engine`, which is why a component-facing crate cannot *name* an engine type even where it
//! can hold one.
//!
//! **Every component and every helper writes a partition of its rectangle** (§2, ADR 0026). Each
//! cell it is responsible for is written exactly once, and the cells it does not write are named in
//! its return value. The rule arrived in three widenings and the third came from a defect its own
//! author wrote fifteen cells after recording the rule.

// **No `unsafe` in any shipped crate above the engine** (ticket 21, ADR 0034). `forbid` and not
// `deny`, so nothing inside the crate can turn it back on with an `allow`; it subsumes the
// `unsafe_op_in_unsafe_fn` this line used to carry.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod app;
pub mod area;
pub mod clusters;
pub mod counters;
pub mod dense;
pub mod document;
pub mod forest;
pub mod form;
pub mod frame;
pub mod gates;
pub mod glyphs;
pub mod grid;
pub mod ink;
pub mod inventory;
pub mod keys;
pub mod listing;
pub mod obligations;
pub mod order;
pub mod popup;
pub mod runner;
pub mod scenes;
pub mod series;
pub mod state;

// **One module per family, and the family is the module** (spec §19). The tree follows the survey's
// fifteen families so that a reader who knows what they want finds it without a search, and
// `INVENTORY`'s `families` column is the join that makes the mapping checkable rather than a naming
// convention. Four of them are empty today and each says why in its own header — F8 navigation, F11
// media, F13 system and F14 terminal-native ship no v1 component of their own, and F15 has no
// module here at all because its twenty-three entries emit no cells and are the runtime's.
//
// **The directory names are a spelling and not a decision.** §19 is explicit that no ticket
// ratified `architecture.md` §2's list and that renaming one reopens nothing. What is settled is
// the join.
pub mod canvas;
pub mod chart;
pub mod collect;
pub mod disclose;
pub mod files;
pub mod indicate;
pub mod input;
pub mod media;
pub mod monitor;
pub mod nav;
pub mod overlay;
pub mod scroll;
pub mod structure;
pub mod text;

mod family;

pub use family::Family;
pub use vitui_runtime::Rect;
// Re-exported at the root because every list, gate and ticket on this backlog names them, and
// `inventory::` in front of each is noise at the one place they are read. The module stays public:
// a reader looking for *why the axes are columns* should land on its documentation.
pub use inventory::{Axis, COMPOSITIONS, Component, INVENTORY, Layer, MOVED, Moved, Tier};
