//! The component library: windows, panels, charts, lists, trees, forms, pickers.
//!
//! # Status
//!
//! Being built one ticket at a time from `.scratch/vitui-components-architecture/spec.md`, whose
//! map is closed; the backlog is `.scratch/vitui-components-impl/`, forty-three tickets.
//!
//! **Twenty-five of the twenty-nine components are written** — [`text::text`], [`text::chip`],
//! [`input::button`] and [`structure::panel`] (components ticket 10), [`collect::collection`]
//! (components ticket 12), [`chart::chart`] and [`chart::plot`] (components ticket 28),
//! [`collect::table`] (components ticket 15), [`collect::tree`] (components ticket 17),
//! [`scroll::scroll_area`], [`scroll::scrollbar`] and [`scroll::sticky`] (components ticket 19),
//! [`disclose::collapsible`] (components ticket 22), [`input::field`] (components ticket 24) and
//! [`input::select`] with [`overlay::overlay`] (components ticket 26), and
//! [`files::file_preview_pane`] with [`files::file_picker`] (components ticket 32),
//! [`input::slider`] (components ticket 33), and the six Tier 2 rows — [`input::checkbox`],
//! [`input::radio`], [`input::switch`], [`indicate::meter`], [`indicate::sparkline`] and
//! [`structure::rule`] (components ticket 34) — and the
//! dense screen, the listing, the grid, the forest, the two scroll-area screens, the accordion and
//! the preview pane's three screens are
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
//! - [`composed`] — Tier 2's own claim as a value: what each of the six reaches and what it may not
//!   mint, with a scan that opens the file and answers both halves. *Composed of proved mechanisms*
//!   is the claim that turns out to be false when it is false, and a scan for absences alone goes
//!   green when the whole section is deleted.
//! - [`obligations`] — §17's five obligations and the sixth stated after the map closed, as queries
//!   over the freeze, each returning a count or an equality. **Six of the seven are green — O1 since
//!   components ticket 36, O3 since 37, O4 since 38, both halves of O2 since 39 and O6 since 44 —
//!   and the one left says so out loud** rather than returning green over an empty population. It is
//!   O5, which §17 says is worth more than the other four together.
//! - [`volume`] — **O6's evidence and its instrument**: *every component that takes a data volume
//!   holds sixty hertz at a million inputs*, gated as a growth relation **and** a per-input ceiling,
//!   both counts. It exists because this crate could prove a frame's **output** was flat in the data
//!   volume and could not prove its **work** was: `chart`'s rasteriser painted the whole column
//!   prefix for every point, **952.61 ms against 2.72**, drew the identical picture, and was counted
//!   as one visit a point by the counter that exists to price the fold. Its population is the one
//!   obligation's that is **derived** rather than written out, and the derivation answered **seven**
//!   where the ticket that asked for it named six.
//! - [`doc`] — **O1's evidence as a value**: a documentation page per built component, located by
//!   the freeze's own `families` column rather than listed, with a scan that opens each file and
//!   reports what is in it. It is the value [`obligations::DOC_TESTED`] is compared against, because
//!   a hand-written evidence list is a claim about twenty-eight files. Its own finding is the
//!   **axis line**: O1 asks that a page state its hostile axes, and **thirteen of the twenty-eight
//!   built components declare none at all** — so a page that mentions the axes it has is silent on
//!   nearly half the freeze, and silence is indistinguishable from a page that forgot. Every page
//!   says `none` out loud and the gate is an equality against the freeze **in both directions**.
//! - [`contract`] — **O4's evidence as a value**: thirteen components' key bindings as declared
//!   data, the help rendered from it through the runtime's own [`vitui_runtime::keys::write_help`],
//!   and a sweep of a hundred and sixty-two triggers that **runs the shipped component** and reads
//!   `Driver::unhandled` for the other half of the equality. A `registered` list derived from the
//!   declaration would agree with it for ever. Its control arm is a `button` — *a tab stop that
//!   reads no key* — because the focus walk takes `Tab` before any component sees it, and without
//!   subtracting that all thirteen contracts register ten chords they have never heard of. It found
//!   four chord leaks in code that was already green, the sharpest of which answered `Ctrl+Left`
//!   with a cluster.
//! - [`gallery`] — **O2's evidence as a value**: one screen carrying every built row of the freeze,
//!   twenty-eight panels in the freeze's own order, each with the function that draws the shipped
//!   component. The application (`crates/vitui-apps/examples/gallery.rs`) iterates the table and
//!   mints no panel of its own, which is the direction neither equality over the table can see. It
//!   lives here rather than in the application because §21 names two defects to be measured *on the
//!   assembled gallery* — the sentinel and the palette swap, register rows 7 and 8, green since
//!   components 40 and 41 — and both are components tickets whose gate is `cargo test`. Its own findings are that **`ColorDepth` had been
//!   reachable since runtime issue 22** while register row 45 filed §16's nine-cell matrix as
//!   unreachable and named that very issue as its inverter, and that **the colour axis is not
//!   observable on a canvas at all**: `Theme::resolve` returns the same paint for all thirteen roles
//!   at all four depths, because quantisation is the engine's and happens before the mirror.
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
//! - [`scenes`] — the normative scene list: **thirty-three screens, twenty-seven of them §21's
//!   table, eight `Unsubjected`, six `Red` and nineteen `Evaluated`**, each with the size it is
//!   played at, the content it stands up, the gestures it plays and the property it decided. Three
//!   of them exist because a defect survived every gate then in force by not being on any screen
//!   anybody had built. **Scenes 8 and 9 — the million-node forest and the fold over 349 524 rows —
//!   went green together with components ticket 17**, which is one fact and not two: both were
//!   pinned on `tree` being undeclared. **Scene 6, the wheel gate, was the one row of the red list
//!   whose failing set was the defect itself** rather than a missing subject — components 12 turned
//!   the other four of components 11's five and deliberately left it, and components **20** turned
//!   it nine tickets later by checking the shipped code. Every scene still red is waiting for a
//!   subject.
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
//! - [`disclose`] — **`collapsible`, and it is one machine for four components** (components
//!   ticket 22). Accordion, tree node, code folding and inplace edit differ in *what their collapsed
//!   content is*, which is [`disclose::SPLIT`] as a value and [`disclose::Collapses`] as its one
//!   configuration — three rows, three arms, no fourth on either side. There is **no transition
//!   state**: `set` flips `open` at the instant the gesture lands and only the height moves, checked
//!   at every frame of a 200 ms collapse *and* by a source scan for a stored third one, whose two
//!   needles are assembled from fragments because a scanner that named them would report the module
//!   it defends.
//!
//!   **Three of §8's figures are replaced by findings rather than reproduced.** Its byte pair
//!   `5 / 72` is `4` live and `48` with the slot, and the pair **cannot both be a `size_of` of one
//!   type** — an `Option<Tween<u16>>` field costs its forty bytes empty, so a record that is 5 B
//!   without a tween is a record whose tween lives somewhere else and nothing here has anywhere to
//!   put one. Its watermark's *83 cells, 8 rows wrong, 3 frames* is a prototype's **body**; what
//!   reproduces is §9's own sentence on the height axis — *a measured extent is taken inside the
//!   rectangle the decision produced* — and over a body that fills what it is handed the watermark
//!   **latches at 19 rows where 4 are right, permanently**, while over a body that draws only its
//!   content the two arms are indistinguishable, which is what makes the rule unconditional. And its
//!   *0 ring probes against 405* is **unreachable from any header gesture**: all three self-close
//!   gestures leave the focus off the body before the vanish rule looks, each for a different
//!   reason, so the arm that pays belongs to a collapse nobody clicked for and what `Focus::Header`
//!   actually buys is the keyboard — on a header that is not a tab stop the click's answer is `None`.
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
//!   they are reanchored. **Both scenes are green since components 22, and green through the
//!   subject**: `draw_into` calls [`disclose::collapsible_into`] and the arm that does not cull is
//!   [`disclose::defective::zero_rect`], so every figure above is a measurement of the component.
//!   [`accordion::Live`] is what a `Screen` cannot describe — a collapse that takes two hundred
//!   milliseconds, a click that lands on the frame it lands on, and the one arm where the vanish rule
//!   answers at all.
//!
//! - [`listing`] — **the collection's screen**: 40x80, one collection and eighty rows a window, and
//!   the four hostile axes standing on it instead of in a table — 75 of 80 rows for the inverted
//!   scroll sign, 71 of 80 (2 840 cells) for the stale tail, and one cell a row for the missing
//!   ellipsis. **All five of its scenes are green**, and the two reasons they went green were kept
//!   apart from the start: four waited for `collection` (components 12) and the wheel gate waited
//!   for the *fix* (components 20), which is [`wheel`] and no longer lives here. Its own finding is
//!   that *writes flat 1k -> 1M* is **green** on a
//!   listing that declares a million hit entries, which is why the equality it registers is on
//!   `regions`. **Four of the five went green with components 12**, which declared
//!   [`collect::collection`] and rewrote both arms of `listing::draw_into` to draw through it —
//!   `Volume::Windowed` is the component and `Volume::WholeContent` is its `defective` twin, one
//!   value apart.
//!
//! - [`wheel`] — **the wheel gate**, over two shipped components and both axes: twenty *posted*
//!   clicks move the offset twenty, and a reveal fires only when a keyboard gesture asked for one.
//!   It is the sharpest instance of the argument this whole map rests on — `CONTEXT.md` forbids the
//!   unconditional `scroll_into_view` and **four *resolved* tickets wrote it anyway**, so the fix is
//!   a gate with two subjects behind it rather than a line in a checklist. Its own finding is the
//!   pair spec §21 had no way to state, because the click used to be arithmetic and a delta added
//!   to an offset has no second axis to be wrong on: **a body dead downward is alive sideways** —
//!   asking for content row 0 every frame settles a vertical click at 0 against 20 and a horizontal
//!   one at 20, and the transpose does the opposite.
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
//! - [`memos`] — **ADR 0030's rule as a population**: *a memo carries the theme in its key iff its
//!   value is made of paints or glyphs*, five rows joined to [`inventory::INVENTORY`] because
//!   components 41's criterion is *a gate enumerates them from `INVENTORY` rather than from a grep*.
//!   Its finding is that **not one shipped memo in this crate holds a paint or a cluster** — a
//!   sub-cell bit grid, an axis domain and a wrap index — so the rule is true here **vacuously**,
//!   and the memo it is about had to be built as [`gallery::Keying`] for register row 8 to have a
//!   gate that can fail. The completeness check is the grep, and it found `crate::order`'s own scan
//!   claiming *no memo here takes a bare `Revision`* while `src/chart/raster.rs` built two of them:
//!   one non-recursive `read_dir` over `src/`, which is [`inventory`]'s own recorded defect
//!   arriving a second time in a different file.
//!
//! - [`area`] — **the scroll area's screens**: the bar fixpoint over 5 475 600 viewport x extent
//!   pairs (0 failures, 3 passes, 0 unneeded bars), the spelling a reader writes instead — which
//!   keeps both bars over content that fits and loses 19 of 20 rows and 19 of 20 columns for ever —
//!   `Σ h` against the row count at row **799 999 of 999 999** with **no counter separating the two
//!   builds**, and an overlay bar re-damaging **398 cells a frame against 0**. **All four scenes
//!   are green since components ticket 19**, and green *through the subject*: the sweep runs
//!   [`scroll::decide`], the extent screen draws through [`scroll::scroll_area`], and the two-areas
//!   screen's reserved arm is that component while its overlay arm had to be written out by hand,
//!   because there is no overlay option and there never will be. Three of §9's figures do not
//!   reproduce and each disagreement is a result: **3 535 is a span-model picture of a ~423-cell
//!   double write** and the engine ships the structure that makes the span model 1.00x; the thumb's
//!   *up to 7 of 69* is the figure at **45.6%** of the reachable range and 14 at its end; and the
//!   wrong pairing's microseconds are another screen's, so the gate is the mechanism — **100 000
//!   rows iterated against 69**. It also found the two defects on this page that are not remembered
//!   numbers: [`area::SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY`], and the instrument's own — a `Pen`
//!   recording a verb where it was *called* rather than where it landed.
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
//!   and §12's five configurations standing on it instead of in a table. It draws through
//!   [`input::select`] and [`overlay::overlay`] since components 26, so **its scene is up and the
//!   list has no red row left**. Four of the five `regions`/`stops` pairs reproduce exactly —
//!   317/316, 319/317, 319/318 — and they are arithmetic rather than measurements, the one region
//!   that is not a stop being the menu bar itself. Beside them: the trap's named exception at
//!   **2 stops visited of 318 declared**, which is §21's *name the exception; do not loosen the
//!   gate* on the first screen that can run it; the opening cliff **counted 30 of 30** rather than
//!   timed; the scrim in three spellings at 600 / 0 / 0 re-damaged and 24 600 / 24 000 / 600
//!   written; **99 flips in 100 frames** for an overlay that declares and covers its anchor, and
//!   **1** for the same rectangle at [`overlay::Kind::Transient`], which is what makes Axis A a
//!   fact about the kind rather than a warning about placement; **3 of 8 frames** for a dialog
//!   owned by the menu row that opened it; and three different ids for the three answers to *where
//!   does the keyboard go when a modal closes*. **Three of §12's seven columns contradict the
//!   shipped code and are asserted as measured**: `allocations`, whose zero was the bump arena
//!   ADR 0034 deleted; `content layers`, whose 2 / 4 / 3 counted a shadow layer `OverlayOpts` has
//!   no field for; and the **menu delta**, 4/2 against §12's 6/6, because §5 collapses a menu into a
//!   `Mode` of `collection` and a collection declares one hit entry however many rows it has.
//! - [`overlay`] — **the family, as three kinds on two axes over eight entries** (components 26),
//!   and the shell every one of them draws inside: the blur position, the reserved bar, the barrier
//!   and the trap, and **not one cell of the interior**. Modality is one `bool` on the request and
//!   forces no construction. [`overlay::popup_size`] is the sizing function §12 requires and
//!   [`overlay::gutter`] the bar decision that moves into the body, in **0 passes against §9's
//!   `<= 3`** — a popup has no second axis to couple through. Its refusals are priced: a size taken
//!   from the drawn extent is granted `(20, 0)` **for ever**, one taken from the content leaves
//!   **1 of 4** rows reachable by nothing, a press-qualified blur dismisses a popup the pointer is
//!   standing on, a catcher layer **swallows** the press it exists to report, a body holding a
//!   `Copy` of the offset moves it **0 in 20 notches**, and a modal missing either half of its two
//!   verbs loses the pointer or the keyboard but never both.
//!
//! - [`files`] — **`file_preview_pane` and `file_picker`, and every defect is at a seam between two
//!   of five pieces** (components ticket 32). R3's *composition with no new mechanism* is the class
//!   that turns out to be false when it is false, so the picker is checked as a **source scan** for
//!   the four mechanisms it may not mint and as a **subtraction** over what it declares: 6 regions
//!   decomposing as the owner's shut face, the shell's blur position, the collection's one entry
//!   however many rows it has, and the pane's three.
//!
//!   **Five seams, five decisions.** [`files::Preview::shows`] is §15's eight bytes on the payload —
//!   an answer to a question *nobody ever asked* arrives with a current generation, because the job
//!   was started for the right question and answered a different one, and *a question that was never
//!   asked is not out of order*, so no test on the answer can see it. [`files::Reset::OnTheLanding`]
//!   is the offset, and the four spellings beside it each produce their own defect on a 4 000-row
//!   file, an 800-row file and a 74-row viewport. [`files::PaneState::land`] is a **top-of-view
//!   verb** and not a step inside the component, because `Task::take` is destructive and a caller
//!   with a status row on each side of the pane straddles a mid-draw take: 20 torn frames of 20.
//!   [`files::Bump::OnTheLanding`] is R09's `Edit` — *a landing that did not happen is still a drop*
//!   — 20 folds against 119 on two screens that are cell-identical. And **nothing here owns the job
//!   or the memo**: ten tab switches cost 1 spawn, 1 decode and 1 fold against 10 of each, because
//!   R02's sweep releases what an `Id` stopped drawing and *a job's lifetime is the question's, a
//!   memo's is the data's, and neither is the widget's*.
//!
//!   **§15's *unclamped* is not a spelling of the offset at all.** `scroll_area` clamps against the
//!   extent it is handed on every frame and the clamp is free, so the only way to *the body draws
//!   nothing* is [`files::Extent::Unbounded`] — and it costs two things rather than one, because an
//!   unbounded extent leaves the area no tail either. The shrink frame writes **21 484 of 24 000**,
//!   and the 2 516 nobody writes is the document's own: §15's own pair is 4 166 against 1 650 on a
//!   screen that did not partition, and **the gap is 2 516 in both**.
//!
//! - [`media`] — **the media family, and none of it is a row of the freeze** (components ticket 30).
//!   §14's *no v1 component* as a value: [`media::MEMBERS`] is empty and the module ships anyway,
//!   because F11's sixteen survey entries reduce to *constructions* rather than to components.
//!   [`media::picture`] is the one caller on this map whose every cell is outside the theme — one
//!   [`vitui_runtime::Theme::custom`] a cell, one verb a cell, **no `fill` available at any size**
//!   and no interactive region at all — and the family is legible as a **census**: a picture spends
//!   one a cell, a QR [`media::QR_CUSTOMS`], a barcode [`media::BARCODE_CUSTOMS`] and the audio
//!   three **none**, because their colours are roles. The barcode's two is the subtraction that
//!   makes the contrast a rule rather than a list: it carries no information across a cell's own
//!   height, so two of a QR's four cell states are unreachable — the same fact that gives it runs
//!   where a picture has none.
//!
//!   **The ladder is derived rather than branched**, which keeps register row 26's exception at one
//!   file: [`media::sub_rows`] is the bar ladder under a ceiling of [`media::COLOURS_PER_CELL`], so
//!   1 / 8 / 8 becomes **1 / 2 / 2** and `Extended == Unicode` follows from arithmetic. A picture is
//!   sampled through [`media::Pixels`] rather than handed a buffer, because a buffer makes the frame
//!   cost the *image* — and that type parameter is the reason `crate::picture::DECLARATIONS` needles
//!   `pub fn picture<P: Pixels>(`: a scene green on `pub fn picture(` would have been green by
//!   deleting the one thing about the signature that is load-bearing.
//!
//!   [`media::player`] is §14's chrome, **six parts of ten**, with [`media::player::PARTS`] as the
//!   column the survey did not have. [`media::player::scrub`] is the mechanism §17 froze `slider` in
//!   Tier 3 for, and it is `Response::local` over `Response::rect` and **nothing else** — press
//!   jumps to `20/299`, move carries to `60/299`, release moves nothing. What is left for components
//!   33 is the thumb, the keyboard, the step and the orientation.
//!   [`media::player::defective::chrome_collecting_into`] is kept because it was wrong: the shape it
//!   replaced allocates on the frames that draw the chapter list, so over a run that is mostly short
//!   the **total** is the defect and `allocs / n` is 0.
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
#![deny(missing_docs)]

pub mod accordion;
pub mod app;
pub mod area;
pub mod clusters;
pub mod composed;
pub mod contract;
pub mod counters;
pub mod dense;
pub mod doc;
pub mod document;
pub mod edit;
pub mod forest;
pub mod form;
pub mod frame;
pub mod gallery;
pub mod gates;
pub mod glyphs;
pub mod golden;
pub mod grid;
pub mod ink;
pub mod inventory;
pub mod keys;
pub mod listing;
pub mod memos;
pub mod obligations;
pub mod order;
pub mod picture;
pub mod popup;
pub mod preview;
pub mod runner;
pub mod scenes;
pub mod series;
pub mod state;
pub mod volume;
pub mod wheel;

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
