//! The component library: windows, panels, charts, lists, trees, forms, pickers.
//!
//! # Status
//!
//! Being built one ticket at a time from `.scratch/vitui-components-architecture/spec.md`, whose
//! map is closed; the backlog is `.scratch/vitui-components-impl/`, forty-three tickets.
//!
//! **Four of the twenty-nine components are written** — [`text::text`], [`text::chip`],
//! [`input::button`] and [`structure::panel`], components ticket 10 — and the dense screen is now
//! drawn *through* them rather than through their construction. They are spec §1's four rules with
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
//! - [`gates`] — §21's register: **eighty gates as rows, fifty-five of them evaluated**, seven
//!   pinned red with their failing sets, six unreachable across the crate line with what would have
//!   to become public, and fourteen with nothing yet to run over. An instrument is a value with a
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
//! - [`scenes`] — the normative scene list: **twenty-nine screens, twenty-seven of them §21's
//!   table, nineteen `Unsubjected`, seven `Red` and three `Evaluated`**, each with the size it is
//!   played at, the content it stands up, the gestures it plays and the property it decided. Three
//!   of them exist because a defect survived every gate then in force by not being on any screen
//!   anybody had built.
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
//!   and four *resolved* tickets wrote anyway. Its five scenes are **red**, and two different
//!   reasons are kept apart: four wait for `collection` (components 12) and the wheel gate waits for
//!   the fix (components 20). Its own finding is that *writes flat 1k -> 1M* is **green** on a
//!   listing that declares a million hit entries, which is why the equality it registers is on
//!   `regions`.
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
pub mod counters;
pub mod dense;
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
pub mod runner;
pub mod scenes;
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
