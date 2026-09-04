//! Spec §17's v1 freeze as a value: twenty-nine components in three tiers, eleven columns each.
//!
//! > **Every obligation this map has stated as a sentence has been broken by someone who had read
//! > it.**
//!
//! That is [ADR 0033]'s first line and it is the whole reason this file is a `const` rather than a
//! paragraph. `CONTEXT.md` forbids the unconditional scroll-into-view and four prototypes wrote
//! one; C02 decided every helper writes a partition and four collections left a tail outside it;
//! C01's author wrote a 15-cell double write straight after writing the rule against it; and C10
//! wrote *the inventory is a value, not a paragraph* and then resolved as a research document, so
//! its five obligations had nothing to be enumerated against until this ticket wrote the value.
//!
//! # What a list buys that a sentence cannot
//!
//! **C11 can write a perfect gate and still not know which twenty-nine components to run it
//! against.** Every later ticket on this backlog is enumerable only against [`INVENTORY`], and the
//! four hostile axes are *columns* for that reason and no other: each was established by a defect
//! that passed every gate then in force **and looked healthier than the correct build** (§17, §21,
//! and [`Axis`] carries the four figures one by one).
//!
//! # Three companion values, and why they are not columns
//!
//! The struct is spec §17's, field for field, and three facts the spec states have nowhere to go in
//! it. Each is a value beside it rather than a twelfth column, so that the shape §17 froze stays
//! the shape:
//!
//! - [`MOVED`] — the exception list the `built` column is gated against. §17: *nothing is `built`
//!   at a tier that says otherwise **without the row saying so***, and this is the row saying so.
//! - [`COMPOSITIONS`] — the edges of the L0..L5 graph. A `layer` column alone makes the DAG
//!   *sayable* and not *checkable*: a gate needs edges to refuse, and the spec states them one
//!   section at a time (`table` = `collection` + column rectangles, `chart` over `plot`, `form` =
//!   `field` + a cursor). Only stated edges are here; inventing one to make the gate look busier is
//!   the guess this ticket was told not to make.
//! - [`Axis`] — the four hostile axes as a value, so that O5 can be a count over them rather than
//!   four hand-written queries.
//!
//! [ADR 0033]: https://github.com/vitui/vitui/blob/master/docs/adr/0033-the-inventory-is-a-value-and-every-obligation-is-a-query-over-it.md

use vitui_runtime::Glyph;

use crate::Family;

/// Which of §17's three tiers a component was frozen at.
///
/// **The tier is part of the freeze** (C10, C11) and a product call can move the boundary; it
/// cannot move the column. See [`Component::built`] for what happens when a later ticket disagrees
/// with a tier — which has already happened three times.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Tier {
    /// **Built, each the subject of a resolved ticket.** Sixteen rows.
    One,
    /// **Composed of proved mechanisms.** Nine rows.
    ///
    /// The tier is not a quality ranking. It means the mechanisms a component composes were
    /// measured by a Tier 1 ticket, and the risk in shipping one is that the composition turns out
    /// to introduce something new — *which is the claim that turns out to be false when it is
    /// false*. Tickets 34 and 35 each carry a gate that it introduces none.
    Two,
    /// **At risk, each naming an unmeasured mechanism and an owner.** Four rows.
    Three,
}

/// One of `COMPONENT-HIERARCHY.md` §4's six layers.
///
/// > **A component is one of six layers and may depend only on lower ones** (spec §1).
///
/// Read strictly, that sentence refuses spec §6's own `table` = `collection` + column rectangles —
/// `architecture.md` puts both at L2 and §6 states the edge. So the gate implements this ticket's
/// wording instead, which is the wording that survives contact with the spec's own compositions:
/// **an edge from a lower layer to a higher one is refused**, same-layer edges are allowed, and
/// acyclicity is checked separately rather than being smuggled in through strictness.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Layer {
    /// A pure drawer. No state at all, and it still returns a `Response` (spec §1, rule 4).
    L0,
    /// One piece of state, held by the caller and handed over as `&mut` beside the data.
    L1,
    /// Virtualised: it costs its visible window and never the data volume.
    L2,
    /// A container: it hands a rectangle to somebody else's draw.
    L3,
    /// It owns a layer — an overlay request, a scrim, a barrier.
    L4,
    /// Compound: composed of other components, with no mechanism of its own.
    L5,
}

impl Layer {
    /// The rung, 0..=5, for the messages a gate prints.
    pub const fn rung(self) -> u8 {
        self as u8
    }
}

/// One of the four hostile axes, each named by the defect that established it.
///
/// **O5 is worth more than the other four obligations together** (§17), and this enum is why it can
/// be a count. Every arm's documentation carries the figures, because the argument for the axis
/// *is* the figures: in three of the four cases the defective build was **faster and marked less**,
/// so no counter in the stack disapproved of it and three of the four were caught only by an
/// equality against a reference render.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Axis {
    /// **Scrolled.** The inverted scroll sign: **12.21 us / 3 058 writes against 62.96 / 20 418**,
    /// drawing nothing and *faster*. Found three times independently.
    Scrolled,
    /// **Shrunk.** The stale tail: **71 of 80 rows** left on screen, the defective build **2.3x
    /// faster marking 226x less**.
    ///
    /// Spelled *content shrinking inside a rectangle that does not move*, which is §21's own
    /// correction: the version written against a terminal resize passes on all twelve panels
    /// because `Gallery::resize` allocates a new `Surface` and the residue has nowhere to survive.
    /// **That spelling tests the resize path and not the defect.**
    Shrunk,
    /// **Wheeled.** The unconditional scroll-into-view: **twenty wheel clicks move the offset 0
    /// against 16**, in four resolved tickets' code and in a rule `CONTEXT.md` already carried.
    Wheeled,
    /// **Narrow.** §13's overlap is **green at 300x80 and red at 60x20** — the one axis on which a
    /// component's construction changes rather than its contents.
    Narrow,
}

impl Axis {
    /// Every axis, which is what O5 iterates.
    pub const ALL: [Axis; 4] = [Axis::Scrolled, Axis::Shrunk, Axis::Wheeled, Axis::Narrow];

    /// The word §17 uses for it.
    pub const fn name(self) -> &'static str {
        match self {
            Axis::Scrolled => "scrolled",
            Axis::Shrunk => "shrunk",
            Axis::Wheeled => "wheeled",
            Axis::Narrow => "narrow",
        }
    }
}

/// One row of spec §17's freeze.
///
/// The eleven columns are §17's, in §17's order. Three facts that would otherwise want a twelfth
/// live beside the list instead — see this module's header.
#[derive(Clone, Copy, Debug)]
pub struct Component {
    /// Its name, which is also the function a caller writes and the key every other list joins on.
    pub id: &'static str,
    /// Which of the three tiers §17 froze it at.
    pub tier: Tier,
    /// Whether it has been built, **beside** the tier rather than derived from it.
    ///
    /// > C10 froze two rows at Tier 3 and C16 resolved afterwards having built them, and prose had
    /// > nowhere for a later ticket to write.
    ///
    /// A table holds a disagreement that prose cannot. The gate is not *built implies Tier 1* — it
    /// is *built implies Tier 1 **or** a row in [`MOVED`] naming the ticket that moved it*, so the
    /// disagreement is an assertion with both names in it rather than a contradiction between two
    /// documents.
    pub built: bool,
    /// Its rung in the L0..L5 dependency graph. See [`Layer`] and [`COMPOSITIONS`].
    pub layer: Layer,
    /// Every family it expresses. **The first is its home**, and the module tree joins on it.
    pub families: &'static [Family],
    /// Its per-component glyph demand set.
    ///
    /// **Not a global list**, and §17 says why: a global *36 of 190 pairs collapse at ASCII* is
    /// expected and harmless, and what it cannot see is that `tree` draws both `Ellipsis` and
    /// `ArrowRight` and ASCII spelled both `>`. The gate that needs this column is *within-component
    /// cross-family collapse == 0 at every rung*.
    ///
    /// **The column is filled, and ticket 05 is where it stopped being short.** `vitui_runtime::Glyph`
    /// shipped **seven** entries and spec §16 took the table to **twenty** — the four arrows, the
    /// four box corners, `Ellipsis`, the four tees and `Cross` — so the demands that had no name
    /// have one: `panel` and `table` name their corners, `table` its tees and its cross, `tree` the
    /// `ArrowRight` chevron beside its `ArrowDown`, `select` and `pagination` their arrows, and
    /// eight rows the `Ellipsis` they truncate with.
    ///
    /// **Twenty of the twenty-nine rows draw at least one glyph, and every one of the twenty entries
    /// has a caller** — `crate::glyphs::tests::the_demand_column_is_filled_and_joined_against_the_table`
    /// is what stops an entry existing for a reader rather than for a component. The tripwire that
    /// pinned this column at seven is now
    /// `tests::the_glyph_table_is_twenty_entries_and_every_one_of_them_has_a_demander`, holding the
    /// same shape one number further on.
    ///
    /// `tree` needed no new entry of its own: its chevron pair *is* `ArrowDown`/`ArrowRight` (§16).
    ///
    /// **The column is what a component draws, and components architecture 20 is where that stopped
    /// being an assumption.** It had also carried §16's *allocation* — which row owns which entry of
    /// the table — and the two readings are the same sentence only while every allocated entry has
    /// a drawer. Four did not: `tree`'s three indent guides and `select`'s stepper `ArrowUp`, struck
    /// there. [`crate::glyphs::UNDRAWN`] is the disagreement that is left, held as a table rather
    /// than as prose (ADR 0033), because the join below is declaration-to-declaration and cannot see
    /// it.
    pub glyphs: &'static [Glyph],
    /// How many **different things** it builds across the three declared repertoires, 1..=3.
    ///
    /// Derived from `CONTEXT.md`'s rungs — *ASCII only*, *Unicode with box drawing and block
    /// elements*, *everything including braille and emoji* — and from nothing else. A glyph is not
    /// a construction: `CONTEXT.md` says a glyph has *a spelling at every repertoire level, every
    /// spelling exactly one cell, and no spelling blank*, and §16 measured that the Unicode and
    /// Extended spellings differ in **0 of 20 entries**. What earns a third rung is a **branch** —
    /// the number of samples asked of the data changing — and §13 measured that on the rendered
    /// surface: Unicode against Extended is **0 cells different in the chart pane and 882 in the
    /// plot pane**.
    ///
    /// So `chart` is 2, `meter` is 2, and `plot` is 3 **whatever the runtime's table says**. §22
    /// records `theme::subrows` as putting the eighth blocks at the wrong rung — they are the same
    /// Unicode block, U+2580..U+2588 — and the freeze does not wait on it. Runtime ticket 17's
    /// module map ships `theme` and `theme::registry` and names no `subrows` at all, so there is
    /// nothing here to be wrong against either way.
    pub constructions: u8,
    /// [`Axis::Shrunk`]: its content can become smaller than the rectangle it was handed, inside a
    /// rectangle that does not move.
    pub can_shrink: bool,
    /// [`Axis::Wheeled`]: it owns the offset a wheel event moves.
    pub owns_offset: bool,
    /// [`Axis::Scrolled`]: it draws a window onto content larger than its rectangle, selected by an
    /// offset.
    pub scrolled: bool,
    /// [`Axis::Narrow`]: its construction is a function of its width, so it can be green at 300x80
    /// and red at 60x20.
    pub narrow: bool,
}

impl Component {
    /// Whether it declares this hostile axis, so O5 can count rather than branch four ways.
    pub const fn declares(&self, axis: Axis) -> bool {
        match axis {
            Axis::Scrolled => self.scrolled,
            Axis::Shrunk => self.can_shrink,
            Axis::Wheeled => self.owns_offset,
            Axis::Narrow => self.narrow,
        }
    }

    /// The module it is homed in: the module of the first family it declares.
    ///
    /// `None` is unreachable for any row of [`INVENTORY`] and the gate says so — the only family
    /// with no module is F15, whose entries emit no cells and are the runtime's.
    pub const fn module(&self) -> Option<&'static str> {
        match self.families.first() {
            Some(first) => first.module(),
            None => None,
        }
    }
}

/// A row that is `built` at a tier saying otherwise, with the ticket that moved it.
///
/// **The freeze does not get rewritten when a later ticket disagrees with it**; the disagreement
/// gets written down. Both names end up in one assertion, which is what ADR 0033 means by *a table
/// holds a disagreement that prose cannot*.
#[derive(Clone, Copy, Debug)]
pub struct Moved {
    /// The component's id.
    pub id: &'static str,
    /// What moved, in §17's own words.
    pub why: &'static str,
}

/// The rows §17 records as built at a tier that says otherwise.
///
/// **Three of them are Tier 3's and two mechanisms**, which is why §17's sentence reads *two of the
/// four have since moved*: drag capture is one mechanism and C16's pair is the other. What remains
/// genuinely at risk is `spinner`, whose mechanism is *a component that owns a clock* — prototyped
/// nowhere on the map, still §22's, and ticket 42's to build.
///
/// # The other six are Tier 2's, and they are here because the gate's vocabulary has no other word
///
/// **A tier is a claim about the mechanism and `built` is a claim about the code**, and §17 makes
/// them two columns precisely so that they can move independently. `nothing_is_built_at_a_tier_that_says_otherwise_without_the_row_saying_so`
/// reads any row that is not Tier 1 as claiming *not built* — which is right for Tier 3, whose
/// definition is *at risk, naming an unmeasured mechanism*, and is **not** right for Tier 2, whose
/// definition is *composed of proved mechanisms* and says nothing about whether anybody has written
/// it yet.
///
/// So building a Tier 2 row is an ordinary event in §17's vocabulary and an exception in the gate's,
/// and ticket 35 will make it nine of nine — at which point this list is *every row that is not
/// Tier 1 and is built*, which is what the column already says. **Recorded rather than bent**: the
/// alternative is promoting six rows into Tier 1, and §17's freeze table is a value a test counts.
pub const MOVED: &[Moved] = &[
    Moved {
        id: "slider",
        why: "its mechanism was drag capture and §14 built it: press jumps to where it landed \
              (20/299), the move carries it (60/299), the release moves nothing — computed from \
              `Response::local` alone, with no press origin and no fifth cross-frame fact",
    },
    Moved {
        id: "file_picker",
        why: "C16 is its named owner and §15 resolved having built it: `collection` + `overlay` + \
              the preview pane, with no mechanism in it that is new",
    },
    Moved {
        id: "file_preview_pane",
        why: "C16 is its named owner and §15 built it with 23 gates; every defect it found was at \
              a seam between two of the five pieces rather than inside one",
    },
    Moved {
        id: "checkbox",
        why: "ticket 34 built it. Tier 2 is *composed of proved mechanisms* and names no mechanism \
              this did not already have: `state::press` for the face and the deferred award, the \
              theme's own `Glyph::Tick`, and `fit` for the label",
    },
    Moved {
        id: "radio",
        why: "ticket 34 built it, as the same machine `checkbox` is with one glyph changed. A radio \
              *set* is `collection` at `Mode::Options`, which is §5's collapse and not this row",
    },
    Moved {
        id: "switch",
        why: "ticket 34 built it. The one of the three with an empty `glyphs` column, because its \
              state is carried by two words, the side its knob sits on and the face — three axes, \
              one of them the palette",
    },
    Moved {
        id: "meter",
        why: "ticket 34 built it as `chart`'s prefix construction at two rungs: the ladder is \
              `geom(Kind::Bars, ..)` and the only thing this row adds is the horizontal spelling of \
              a partial cell, which is a different contiguous run of block elements",
    },
    Moved {
        id: "sparkline",
        why: "ticket 34 built it as `chart` at a small rectangle with no axes, no gutter and no \
              axis loop — the same raster memo and the same body loop, reached rather than copied",
    },
    Moved {
        id: "rule",
        why: "ticket 34 built it as `fit`'s four skippable parts with a `Glyph` where the padding \
              was, over one row or one column, plus `elide`'s one-cell marker",
    },
    Moved {
        id: "status_bar",
        why: "ticket 35 built it as §9's one band construction with the axis argument taken \
              verbatim — `scroll::sticky`, a view rather than arithmetic — plus one hit entry for \
              the bar and never one per segment",
    },
    Moved {
        id: "pagination",
        why: "ticket 35 built it as `collection`'s store, its thirteen-arm `apply` and its one \
              drain loop laid out on the other axis: the row loop is vertical by construction and a \
              transpose is not a rectangle split",
    },
    Moved {
        id: "form",
        why: "ticket 35 built it as §18 R3's own example — `field` + `nav::cursor` + the focus ring \
              the draw builds — with a state exactly as big as the type-ahead buffer `nav::cursor` \
              cannot borrow from the ring",
    },
    Moved {
        id: "spinner",
        why: "ticket 46 built it, and it is the one row here that was genuinely at risk rather than \
              merely mis-tiered: §17 froze it at Tier 3 for an unmeasured mechanism, ticket 42 \
              measured it, and the permission is narrower than the question — a component may own \
              an **anchor** and may not own a **clock**. `SpinState` is `Option<Steps>` and 32 B, \
              against the 40 B tween slot `disclose::Collapse` already carries as a field",
    },
];

/// One stated edge of the L0..L5 graph: `of` is built on `on`.
///
/// `source` is the section that states it. **Only stated edges are here.** A component's real call
/// graph is larger — every Tier 2 row is *composed of proved mechanisms* — but a mechanism is not a
/// component, and an edge nobody wrote down is an edge a gate cannot honestly refuse.
#[derive(Clone, Copy, Debug)]
pub struct Composition {
    /// The composing component's id.
    pub of: &'static str,
    /// The composed component's id.
    pub on: &'static str,
    /// Where the spec states the edge.
    pub source: &'static str,
}

/// Every composition edge the spec states, which is what makes the `layer` column checkable.
///
/// A `layer` column with no edges is a claim about a graph nobody drew. These are the edges §5–§18
/// and the impl backlog's tickets 34 and 35 write in so many words, and
/// `tests::no_composition_runs_from_a_lower_layer_to_a_higher_one` is the gate over them.
pub const COMPOSITIONS: &[Composition] = &[
    Composition {
        of: "table",
        on: "collection",
        source: "§6 — `table` = `collection` + column rectangles",
    },
    Composition {
        of: "tree",
        on: "collection",
        source: "§7 — `tree` = `collection` + a flatten index",
    },
    Composition {
        of: "scroll_area",
        on: "scrollbar",
        source: "§9 — `scroll::bar` is the scrollbar every scrollable draws",
    },
    Composition {
        of: "select",
        on: "overlay",
        source: "§12 — the popup is the owner's, and `PopupState` is the body's",
    },
    Composition {
        of: "select",
        on: "collection",
        source: "§5 — one `Mode` absorbs listbox, menu, submenu and the palette body",
    },
    Composition {
        of: "chart",
        on: "plot",
        source: "§13 — `plot` is the sub-cell rasteriser and an L0 leaf, not a composite",
    },
    Composition {
        of: "meter",
        on: "chart",
        source: "ticket 34 — `chart`'s prefix construction at 2 rungs",
    },
    Composition {
        of: "sparkline",
        on: "chart",
        source: "ticket 34 — `chart` at a small rectangle, no axes, no gutter, no axis loop",
    },
    Composition {
        of: "status_bar",
        on: "sticky",
        source: "ticket 35 — the same band construction, one rectangle split and one hit entry",
    },
    Composition {
        of: "pagination",
        on: "collection",
        source: "ticket 35 — `collection` at a small length plus `nav::cursor`",
    },
    Composition {
        of: "form",
        on: "field",
        source: "§18 R3 — `form` is `field` + `nav::cursor` + the focus ring the draw builds",
    },
    Composition {
        of: "file_picker",
        on: "collection",
        source: "§15, §18 R3 — `file_picker` is `collection` + `overlay` + the preview pane",
    },
    Composition {
        of: "file_picker",
        on: "overlay",
        source: "§15, §18 R3 — `file_picker` is `collection` + `overlay` + the preview pane",
    },
    Composition {
        of: "file_picker",
        on: "file_preview_pane",
        source: "§15, §18 R3 — `file_picker` is `collection` + `overlay` + the preview pane",
    },
];

/// Spec §17's v1 freeze: **twenty-nine components in three tiers**, sixteen, nine and four.
///
/// # Six collapses are folded in, five measured and one a judgement
///
/// A collapse under §18's R1 — *it is one of the twenty-nine under another name* — is only
/// legitimate when a ticket measured it. §17 records six:
///
/// | collapse | where it was measured |
/// |---|---|
/// | accordion → `collapsible` | §8, one state machine and the whole of family F4 |
/// | tabs → `collection` | §5, thirteen match arms of one `Mode` |
/// | menu → `collection` | §5, the same `Mode` |
/// | textarea → `field` | §11, one flag rather than a second component |
/// | tooltip → `overlay` | §12, two axes rather than eighteen components |
/// | **gauge → `meter`** | **nothing — this one is a judgement rather than a measurement** |
///
/// **The sixth is marked and stays marked.** It is the only collapse in the freeze that rests on an
/// argument instead of a number, and ticket 34 carries the same sentence forward so that the
/// marking cannot be lost by a later edit. `tests::the_six_collapses_are_recorded_and_the_judgement_is_marked`
/// reads this doc comment and fails if the marking goes.
///
/// # Three additions no frequency count produces
///
/// `chip` (built three times, the subject of three defects), `plot` (§13: the chart pane is 0 cells
/// different at Extended and the plot pane differs in 882) and `sticky` (§9). **None appears in any
/// inventory's core list**, which is what makes the freeze a decision rather than an arithmetic —
/// and the inherited frequency rule does not reproduce anyway: re-run against the only population
/// that can carry it, with a deliberately generous synonym set so the count is an *upper* bound, it
/// puts **21 of the 28 below nine of fourteen**.
///
/// # The columns, and where each came from
///
/// - `tier`, `built` — §17's freeze table and its three moved rows ([`MOVED`]).
/// - `layer` — `COMPONENT-HIERARCHY.md` §4 through spec §1, with `architecture.md` §4's per-item
///   assignments where it made them (`plot` an L0 leaf, `collection` L2, `scroll_area` L3).
/// - `families` — §18's family-by-family table. The first is the home; see [`crate::Family`].
/// - `glyphs` — the nameable part of each demand set. Ticket 05 owns the size (see
///   [`Component::glyphs`]).
/// - `constructions` — `CONTEXT.md`'s three rungs (see [`Component::constructions`]).
/// - the four axis flags — the defect that established each, cited per row.
pub const INVENTORY: &[Component] = &[
    // ---- Tier 1: built, each the subject of a resolved ticket. Sixteen rows. ----
    Component {
        id: "text",
        tier: Tier::One,
        built: true,
        layer: Layer::L0,
        families: &[Family::F1Text],
        glyphs: &[Glyph::Ellipsis],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        // §16's one-cell rule, which only fires at a width where the label truncates: a three-cell
        // `...` where one cell was reserved moves **468 cells over 78 rows** with writes, verbs and
        // marked identical at 23 402 / 2 163 / 0 either way. Inside a narrowed context the clip
        // discards the two extra silently and the reader gets a hard cut that looks deliberate;
        // outside one the same three overrun the neighbour.
        narrow: true,
    },
    Component {
        id: "panel",
        tier: Tier::One,
        built: true,
        layer: Layer::L0,
        families: &[Family::F2Structure],
        // The border. The four corners and the four tees are §16's growth to twenty and are not
        // nameable yet; the two rules are.
        glyphs: &[
            Glyph::HLine,
            Glyph::VLine,
            Glyph::TopLeft,
            Glyph::TopRight,
            Glyph::BottomLeft,
            Glyph::BottomRight,
        ],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        // A panel's title truncation is `text`'s construction, not a second one. Duplicating the
        // flag would buy O5 a second scene for one defect, which is the opposite of what a per-axis
        // scene count is for.
        narrow: false,
    },
    Component {
        id: "chip",
        tier: Tier::One,
        built: true,
        // Its face and its label, and nothing across frames. §2's 2 648-cell re-damage was a fill
        // before a draw — a partition defect, not state.
        layer: Layer::L0,
        families: &[Family::F1Text, Family::F6Input],
        glyphs: &[Glyph::Ellipsis],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        // §2 names the defect in the axis's own word: *a chip that does not narrow, whose label
        // runs into its sibling's rectangle* — **432 cells re-damaged every steady frame**.
        narrow: true,
    },
    Component {
        id: "button",
        tier: Tier::One,
        built: true,
        // `state::press` — the press/hover/active machine, one struct beside the call.
        layer: Layer::L1,
        families: &[Family::F6Input],
        glyphs: &[],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "field",
        tier: Tier::One,
        built: true,
        // §11: `input` and `textarea` are one component, and the textarea half carries a wrap index
        // over a 1 MB document. It costs its visible window.
        layer: Layer::L2,
        families: &[Family::F6Input, Family::F1Text],
        glyphs: &[],
        constructions: 1,
        // A 1 MB textarea edited down to one line, inside a rectangle that does not move.
        can_shrink: true,
        // The caret must be brought into view and `CONTEXT.md` forbids doing it unconditionally —
        // which is the wheel defect's own rule, one component over.
        owns_offset: true,
        scrolled: true,
        // §20's scene 13: a 300 → 120 resize with a wrap memo draws **625 rows where 875 are
        // needed**, because the memo key did not carry the width. A width-keyed construction, which
        // is the axis exactly.
        narrow: true,
    },
    Component {
        id: "collection",
        tier: Tier::One,
        built: true,
        layer: Layer::L2,
        families: &[Family::F7Collections, Family::F6Input, Family::F3Scrolling],
        glyphs: &[Glyph::Bullet, Glyph::Tick, Glyph::Ellipsis],
        constructions: 1,
        // The stale tail: **71 of 80 rows**, the defective build 2.3x faster marking 226x less.
        can_shrink: true,
        // The unconditional scroll-into-view: **0 against 16** in twenty wheel clicks, in four
        // resolved tickets' code.
        owns_offset: true,
        // The inverted scroll sign: 12.21 us / 3 058 writes against 62.96 / 20 418, found three
        // times independently.
        scrolled: true,
        // Three of the four axes are literally this component's defects; the fourth is not. A
        // collection's rows truncate through `text::fit`, which is `text`'s flag.
        narrow: false,
    },
    Component {
        id: "table",
        tier: Tier::One,
        built: true,
        layer: Layer::L2,
        families: &[Family::F7Collections, Family::F13System],
        // The header rule and the column separators. §16's four tees and `Cross` are the rest and
        // are not nameable yet.
        glyphs: &[
            Glyph::HLine,
            Glyph::VLine,
            Glyph::TopLeft,
            Glyph::TopRight,
            Glyph::BottomLeft,
            Glyph::BottomRight,
            Glyph::TeeTop,
            Glyph::TeeBottom,
            Glyph::TeeLeft,
            Glyph::TeeRight,
            Glyph::Cross,
            Glyph::Ellipsis,
        ],
        constructions: 1,
        can_shrink: true,
        owns_offset: true,
        scrolled: true,
        // §6's own scene: a twelve-column 1M-row table pinned at both edges with horizontal
        // overflow — **verbs as the currency, 913 → 2 497**, and the band's 345 re-damaged cells.
        // The pinned-column construction only exists at a width where columns overflow.
        narrow: true,
    },
    Component {
        id: "tree",
        tier: Tier::One,
        built: true,
        layer: Layer::L2,
        families: &[Family::F7Collections],
        // **The chevron pair and the cut, and no indent guide** — components architecture 20,
        // resolved. `VLine`, `TeeLeft` and `BottomLeft` stood here for four tickets as *the indent
        // guides* and were drawn by nothing: the shipped indent is one `Ink::run` of spaces, and
        // §7 forbids every route to a correct guide column (see `crate::glyphs::UNDRAWN` for the
        // four that were refuted). A demand for a glyph that never reaches a cell is a sentence
        // about a screen this crate does not draw, so the column lost them rather than the
        // component gaining a fifth field.
        glyphs: &[Glyph::ArrowDown, Glyph::ArrowRight, Glyph::Ellipsis],
        constructions: 1,
        // §20's scene 9: a fold over 349 524 rows, splice against permutation — 1 run / 0.04 us
        // against 297 180 / 24 338.
        can_shrink: true,
        owns_offset: true,
        scrolled: true,
        // §16's cross-family collapse, which is `tree`'s: the shadow table's ASCII ellipsis was
        // `>`, exactly an ASCII `ArrowRight`, so **468 truncated labels ended in the collapsed-node
        // marker**. How many labels truncate is a function of the width.
        narrow: true,
    },
    Component {
        id: "select",
        tier: Tier::One,
        built: true,
        // It requests an overlay. §12's alternative — a catcher layer — costs 386 912 layer bytes
        // and swallows a click, against 2 592 bytes and two clicks for blur.
        layer: Layer::L4,
        families: &[Family::F6Input, Family::F9Overlays],
        // The chevron pair and the cut. `ArrowRight`/`ArrowDown` says closed or open, and it is
        // the second row after `tree` to draw both `Ellipsis` and `ArrowRight`, so it is the second
        // component C09's `>` would have put the collapsed marker on the end of a truncated option
        // in.
        //
        // **`ArrowUp` went with `tree`'s three** (components architecture 20). §16's *`ArrowUp`/
        // `ArrowDown` steps the popup's list* describes `scroll::scrollbar`'s stepper caps, and a
        // popup's gutter is `scroll::bar_into` — `Thumb` and `Track`, no caps. `ArrowDown` stays
        // because it is drawn, as the **chevron** rather than as a stepper.
        glyphs: &[
            Glyph::ArrowDown,
            Glyph::ArrowRight,
            Glyph::Tick,
            Glyph::Ellipsis,
        ],
        constructions: 1,
        can_shrink: false,
        // §12 states the wheel defect inside its own section: §7's literal `Copy`-only body moves
        // the offset **0 in 20 wheel clicks**.
        owns_offset: true,
        scrolled: true,
        // Placement near an edge is `rt::overlay::place`'s arithmetic and is gated in the runtime;
        // no width-dependent construction of `select`'s own is recorded.
        narrow: false,
    },
    Component {
        id: "overlay",
        tier: Tier::One,
        built: true,
        layer: Layer::L4,
        families: &[
            Family::F9Overlays,
            Family::F5Indicators,
            Family::F8Navigation,
        ],
        glyphs: &[],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "scroll_area",
        tier: Tier::One,
        built: true,
        layer: Layer::L3,
        families: &[Family::F3Scrolling],
        glyphs: &[],
        constructions: 1,
        // §9 states the shrink axis inside its own section: the tail is not free — the range from
        // the extent to the end of the viewport is inside the rectangle, and the component that
        // owns the rectangle must write it.
        can_shrink: true,
        // §9's watermark: over a body that virtualises rows and not columns it reads (400, 69)
        // against a (246, 69) viewport — the area is **dead downward, twenty wheel clicks move the
        // offset 0**, and alive sideways.
        owns_offset: true,
        scrolled: true,
        // §9's reserved auto-hiding bars: a measured extent and a hideable reserved bar are
        // incompatible, because the measurement is taken inside the rectangle the decision
        // produced. Keeping both loses a row and a column permanently, on a screen that looks
        // correct — and what "permanently" costs is a function of how few rows and columns there
        // are.
        narrow: true,
    },
    Component {
        id: "scrollbar",
        tier: Tier::One,
        built: true,
        // R05's grab. The offset it moves is the area's, which is the next flag down.
        layer: Layer::L1,
        families: &[Family::F3Scrolling],
        glyphs: &[
            Glyph::ArrowUp,
            Glyph::ArrowDown,
            Glyph::ArrowLeft,
            Glyph::ArrowRight,
            Glyph::Thumb,
            Glyph::Track,
        ],
        constructions: 1,
        can_shrink: false,
        // §9: a band that were a second scroll area would win the wheel from the body it is a
        // header of. The bar is not a second scroll area and does not own the offset.
        owns_offset: false,
        scrolled: false,
        // §20's scene 17: the bar fixpoint over **5 475 600 viewport x extent pairs**, 0 failures
        // in at most 3 passes. The fixpoint exists because thumb geometry is viewport-dependent,
        // and the unit error that motivated it is out by up to **7 cells of 69**, smoothly and
        // plausibly.
        narrow: true,
    },
    Component {
        id: "sticky",
        tier: Tier::One,
        built: true,
        layer: Layer::L3,
        families: &[Family::F3Scrolling],
        glyphs: &[Glyph::HLine],
        constructions: 1,
        can_shrink: false,
        // §9, R17 §5: **one hit entry for all four bands**, because a band that were a second
        // scroll area would win the wheel from the body it is a header of. The `false` here is the
        // recorded decision, not an omission.
        owns_offset: false,
        // §9: a band shares one of the two offsets and pins the other to zero, **and it must be a
        // view** — drawn by arithmetic instead it has identical writes, identical verbs, identical
        // output and **3 243 cells re-damaged every steady frame**.
        scrolled: true,
        narrow: false,
    },
    Component {
        id: "collapsible",
        tier: Tier::One,
        built: true,
        layer: Layer::L3,
        families: &[Family::F4Disclosure],
        glyphs: &[Glyph::ArrowDown, Glyph::ArrowRight],
        constructions: 1,
        // §20's scene 11: an accordion of twelve sections at 0 / 6 / 12 open — **closed content is
        // not drawn, 478 hit entries against 70**. Content shrinking inside a rectangle that does
        // not move, which is the axis's own words.
        can_shrink: true,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "chart",
        tier: Tier::One,
        built: true,
        // The raster memo is the caller's; what `chart` holds across a frame is one key.
        layer: Layer::L1,
        families: &[Family::F10Charts, Family::F5Indicators, Family::F13System],
        // The axes. §16's tees and corners are the rest.
        glyphs: &[Glyph::HLine, Glyph::VLine],
        // **2.** ASCII against Unicode is 7 276 cells over 80 of 80 rows — ADR 0009 literally, a
        // different construction. Unicode against Extended is **0 cells in the chart pane**.
        constructions: 2,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        // **The defect that established the axis.** §13's overlap is green at 300x80 and red at
        // 60x20; the frame is 204 us / 21 872 writes wide and 13.8 us / 1 136 writes narrow, flat
        // at 1k, 100k and 1M in both.
        narrow: true,
    },
    Component {
        id: "plot",
        tier: Tier::One,
        built: true,
        // `architecture.md` §4.7 is explicit: an L0 leaf, not a composite.
        layer: Layer::L0,
        families: &[Family::F10Charts, Family::F5Indicators, Family::F13System],
        glyphs: &[],
        // **3, and it is the only 3 in the freeze.** §13 measured Unicode against Extended on the
        // rendered surface: **882 cells differ in the plot pane**, which is one bit of vertical
        // resolution bought at the cost of series colour — a braille cell has one `Paint` for all
        // eight dots where a quadrant carries a foreground and a background, and the 25 cells where
        // two series share one are what pays.
        constructions: 3,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        // §20's scene 15 carries it: two million-point series at 300x80 and 60x20, **and 60x20 is
        // where C08's overlap is red**.
        narrow: true,
    },
    // ---- Tier 2: composed of proved mechanisms. Nine rows, six of them built. ----
    Component {
        id: "checkbox",
        tier: Tier::Two,
        built: true,
        // `state::press` plus a `Glyph` pair plus a `Role` (ticket 34).
        layer: Layer::L1,
        families: &[Family::F6Input],
        glyphs: &[Glyph::Tick],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "radio",
        tier: Tier::Two,
        built: true,
        // Standalone it is a `press`. **A radio *set* is `collection` at `Mode::Radio`** (ticket
        // 34) — which is §5's R1 collapse rather than an edge out of this row, so no
        // `COMPOSITIONS` entry: the row is the widget, and the set is a different call.
        layer: Layer::L1,
        families: &[Family::F6Input],
        glyphs: &[Glyph::Bullet],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "switch",
        tier: Tier::Two,
        built: true,
        layer: Layer::L1,
        families: &[Family::F6Input],
        glyphs: &[],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "meter",
        tier: Tier::Two,
        built: true,
        // `chart`'s prefix construction at two rungs (ticket 34), so it sits at `chart`'s rung.
        layer: Layer::L1,
        families: &[Family::F5Indicators, Family::F13System],
        glyphs: &[Glyph::Thumb, Glyph::Track],
        // **2, and this is the number §17 names.** Block elements are the Unicode rung by
        // `CONTEXT.md`'s own definition — *Unicode with box drawing and block elements* — so an
        // operator who promises block elements has promised all of them, and there is no third
        // thing for a meter to build.
        constructions: 2,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "sparkline",
        tier: Tier::Two,
        built: true,
        layer: Layer::L1,
        families: &[Family::F5Indicators, Family::F10Charts],
        glyphs: &[],
        // **2 by `meter`'s argument, and the spec does not name this number.** Ticket 34 says a
        // sparkline is `chart` at a small rectangle with no axes, no gutter and no axis loop; the
        // prefix ladder is 2 / 9 / 9 and the top two rungs are the same block, so the third rung is
        // §13's and `plot` earns it alone. Recorded here as derived rather than stated.
        constructions: 2,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "rule",
        tier: Tier::Two,
        built: true,
        // `fit`'s remainder over one row or one column, plus a `Glyph` (ticket 34).
        layer: Layer::L0,
        families: &[Family::F2Structure],
        // **`Ellipsis` arrived with components ticket 35 and it is a correction rather than a
        // change**: `rule_into` has gone through `crate::glyphs::elide` since it was written, so a
        // caption wider than its line has always been able to draw a marker this column did not
        // declare. Under-declaring is the quiet direction — §16's within-component collapse gate
        // runs over the column, so it was running over two glyphs where the component draws three.
        glyphs: &[Glyph::HLine, Glyph::VLine, Glyph::Ellipsis],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "status_bar",
        tier: Tier::Two,
        built: true,
        // Ticket 35: **the same construction as a sticky header or footer** — a rectangle split
        // that shares one of the two offsets and pins the other to zero, and it must be a view.
        layer: Layer::L3,
        families: &[Family::F2Structure, Family::F13System],
        // **`Ellipsis` beside the separator**, because a segment is written through
        // `crate::text::fit_into` and a segment wider than its share is elided. The demand column
        // is what §16's within-component collapse gate runs over, so a component that can draw a
        // marker and does not declare one is a gate running over less than the component draws —
        // and `text`, `chip`, `select` and `file_picker` all declare it for the same reason.
        glyphs: &[Glyph::VLine, Glyph::Ellipsis],
        constructions: 1,
        can_shrink: false,
        // One hit entry, not one per segment (ticket 35), and a band never wins the wheel.
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "pagination",
        tier: Tier::Two,
        built: true,
        // `collection` at a small length plus `nav::cursor`; no second store and no second
        // navigation model (ticket 35).
        layer: Layer::L5,
        families: &[Family::F7Collections, Family::F8Navigation],
        // **`Ellipsis` beside the two arrows**, and this row is where §16's C09 pair would land if
        // it came back: a page number elided next to a `›` stepper is `tree`'s `Ellipsis`-spelled-
        // `>` defect on a pager. It does not collide — components ticket 05 spelled `Ellipsis` `~`
        // at ASCII — and `crate::glyphs::within_component_collapses` is what keeps saying so.
        glyphs: &[Glyph::ArrowLeft, Glyph::ArrowRight, Glyph::Ellipsis],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "form",
        tier: Tier::Two,
        built: true,
        // **§18 R3's own example**: a composition of shipped components with no new mechanism.
        layer: Layer::L5,
        families: &[Family::F6Input],
        // **`Ellipsis` and nothing else.** A form draws no glyph of its own — the fields are
        // `field`'s and the ring is the runtime's — but a label wider than the label column goes
        // through `crate::text::fit_into`, which elides. See
        // `crate::composed::tests::a_tier_two_row_declares_the_ellipsis_it_can_draw_and_no_row_declares_one_it_cannot`.
        glyphs: &[Glyph::Ellipsis],
        constructions: 1,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    // ---- Tier 3: at risk, each naming an unmeasured mechanism and an owner. Four rows. ----
    Component {
        id: "slider",
        tier: Tier::Three,
        // Built at Tier 3. See `MOVED`: drag capture is measured and needed nothing added to the
        // runtime.
        built: true,
        layer: Layer::L1,
        families: &[Family::F6Input],
        // **`VLine` joined the demand set when the component was written** (components ticket 33): a
        // vertical slider's groove is a column, and `HLine` stacked down one is a picture of a
        // dashed line. `scroll_area` already demands both, so the pair collapses at no rung.
        glyphs: &[Glyph::HLine, Glyph::VLine, Glyph::Thumb],
        constructions: 1,
        can_shrink: false,
        // A slider's value is not an offset and no wheel event moves it.
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "spinner",
        tier: Tier::Three,
        // **The last row of the freeze to be built, and it left Tier 3's at-risk column before it
        // left the unbuilt column.** Its mechanism — *a component that owns a clock* — was §17's
        // at-risk entry and §22's *named with an owner and not prototyped*; components ticket 42
        // prototyped it and ticket 46 shipped it. The rule it lands on is *stored state may be an
        // anchor, never a phase*, which is what §8 and §9 were already obeying rather than a
        // permission granted here: `SpinState` is 32 B against the 40 B tween slot
        // `disclose::Collapse` already carries as a field. See `MOVED` and ADR 0051.
        built: true,
        layer: Layer::L1,
        families: &[Family::F5Indicators],
        // **Empty, and it stays empty for a reason rather than for want of a table.** A `Glyph` is
        // one lookup with no spelling
        // blank; a spinner needs an ordered set of n spellings that differ *from each other*, which
        // is not one lookup and not n of them — cycling the four arrows at a reader tells them
        // nothing four times. So the ladder is the component's own table, exactly as
        // `chart::raster::RUNGS` is `chart`'s and `crate::media::sub_rows` is the picture's.
        glyphs: &[],
        // **3, and it is derived rather than argued — which is what ticket 46 owed and 42 could
        // not.** §16's rule points at 1 on the grounds that every spelling is exactly one cell and
        // no spelling is blank; both are true of the shipped ladder and *neither decides it*. What
        // decides it is whether the rung changes what is **built**, and
        // `crate::indicate::constructions` counts the distinct ladders of
        // `crate::indicate::LADDERS` — asserted against this number by
        // `crate::indicate::tests::a_spinner_is_three_constructions_and_the_ladder_is_its_own`, the
        // way `chart`, `plot`, `meter` and `sparkline` each assert theirs against a shipped table.
        //
        // **It was 2 in ticket 42 and the correction is the rung boundary, not the count.** The
        // prototype put the braille spinner at `Unicode | Extended`, and the engine's own
        // `GlyphSet` says `Unicode` is *Unicode a normal text font covers* while `Extended` is
        // *braille, block elements, emoji, powerline* — so a terminal that promised the middle rung
        // would have rendered tofu, which is the one failure a ladder exists to prevent. The middle
        // rung is the quadrant blocks, a four-position orbit rather than a re-spelling of the ASCII
        // rotating line, so the three ladders are three distinct tables. That is `plot`'s shape and
        // not `meter`'s: braille is what a spinner spends 256 states a cell on.
        constructions: 3,
        can_shrink: false,
        owns_offset: false,
        scrolled: false,
        narrow: false,
    },
    Component {
        id: "file_picker",
        tier: Tier::Three,
        // Built at Tier 3. See `MOVED`.
        built: true,
        layer: Layer::L5,
        families: &[Family::F12Files, Family::F7Collections, Family::F9Overlays],
        glyphs: &[Glyph::Bullet, Glyph::Ellipsis],
        constructions: 1,
        can_shrink: true,
        owns_offset: true,
        scrolled: true,
        narrow: false,
    },
    Component {
        id: "file_preview_pane",
        tier: Tier::Three,
        // Built at Tier 3 with 23 gates. See `MOVED`.
        built: true,
        layer: Layer::L3,
        families: &[Family::F12Files, Family::F11Media],
        glyphs: &[Glyph::VLine, Glyph::Ellipsis],
        constructions: 1,
        // §15 states the shrink axis in one clause: **a landing *is* a shrink**, from another
        // thread for the first time. Left unclamped the body draws nothing at all — 1 650 writes
        // against 4 166.
        can_shrink: true,
        // §15: the offset belongs to neither side, and four spellings produce four different
        // defects on a 4 000-row file, an 800-row file and a 74-row viewport.
        owns_offset: true,
        scrolled: true,
        narrow: false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn row(id: &str) -> &'static Component {
        INVENTORY
            .iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("`{id}` is not a row of the freeze"))
    }

    /// **Twenty-nine rows: sixteen, nine and four.**
    ///
    /// The split is asserted and not only the total, which is the engine's arrangement and for its
    /// reason: a thirtieth row has to say which side of a line it is on. §17's own arithmetic is
    /// what makes the count worth gating — the inherited set was *32, not 33*, a sum presented as a
    /// union and repeated in four documents, and `scrollbar` was in both halves of it.
    #[test]
    fn the_freeze_is_twenty_nine_components_in_three_tiers() {
        assert_eq!(INVENTORY.len(), 29);
        let count = |t: Tier| INVENTORY.iter().filter(|c| c.tier == t).count();
        assert_eq!(count(Tier::One), 16, "Tier 1");
        assert_eq!(count(Tier::Two), 9, "Tier 2");
        assert_eq!(count(Tier::Three), 4, "Tier 3");

        let ids: BTreeSet<&str> = INVENTORY.iter().map(|c| c.id).collect();
        assert_eq!(
            ids.len(),
            INVENTORY.len(),
            "two rows share an id, which is the defect §17 found in the inherited count: a sum \
             presented as a union"
        );
    }

    /// **The freeze, row for row, against §17's table.**
    ///
    /// A count alone would pass on twenty-nine rows all named `text`. This is the membership, in
    /// §17's own order and spelling.
    #[test]
    fn every_row_of_the_freeze_is_one_of_the_names_seventeen_froze() {
        const TIER_ONE: [&str; 16] = [
            "text",
            "panel",
            "chip",
            "button",
            "field",
            "collection",
            "table",
            "tree",
            "select",
            "overlay",
            "scroll_area",
            "scrollbar",
            "sticky",
            "collapsible",
            "chart",
            "plot",
        ];
        const TIER_TWO: [&str; 9] = [
            "checkbox",
            "radio",
            "switch",
            "meter",
            "sparkline",
            "rule",
            "status_bar",
            "pagination",
            "form",
        ];
        const TIER_THREE: [&str; 4] = ["slider", "spinner", "file_picker", "file_preview_pane"];

        for (tier, names) in [
            (Tier::One, &TIER_ONE[..]),
            (Tier::Two, &TIER_TWO[..]),
            (Tier::Three, &TIER_THREE[..]),
        ] {
            let got: Vec<&str> = INVENTORY
                .iter()
                .filter(|c| c.tier == tier)
                .map(|c| c.id)
                .collect();
            assert_eq!(got, names, "{tier:?} is not §17's row set");
        }
    }

    /// **`built` is gated against the tier, and the exception is named rather than the gate
    /// loosened.**
    ///
    /// §21's third refinement, applied to a column instead of a threshold. The rule is *nothing is
    /// `built` at a tier that says otherwise **without the row saying so***, so this is not *built
    /// implies Tier 1*: it is *built implies Tier 1 or a row of [`MOVED`]*, and [`MOVED`] carries
    /// what moved in §17's own words. Three rows use the exception and one Tier 3 row does not —
    /// `spinner`, which is the point of keeping the tier when the column disagrees with it.
    #[test]
    fn nothing_is_built_at_a_tier_that_says_otherwise_without_the_row_saying_so() {
        let moved: BTreeSet<&str> = MOVED.iter().map(|m| m.id).collect();
        assert_eq!(moved.len(), MOVED.len(), "`MOVED` names a row twice");

        for c in INVENTORY {
            let excused = moved.contains(c.id);
            if c.built {
                assert!(
                    c.tier == Tier::One || excused,
                    "`{}` is built at {:?} and `MOVED` does not say why. Either the tier is wrong \
                     — and §17 says a product call can move the boundary but not the column — or \
                     the row owes an entry in `MOVED` naming the ticket that built it",
                    c.id,
                    c.tier
                );
            } else {
                assert!(
                    c.tier != Tier::One,
                    "`{}` is frozen at Tier 1, which §17 defines as *built, each the subject of a \
                     resolved ticket*, and the column says otherwise",
                    c.id
                );
                assert!(
                    !excused,
                    "`{}` is in `MOVED` and is not built. `MOVED` is the exception list for a \
                     column that disagrees with its tier, not a place to record intent",
                    c.id
                );
            }
        }

        // The three §17 records, by name, so that losing one is a diff that deletes an assertion.
        assert_eq!(
            moved,
            BTreeSet::from([
                "slider",
                "file_picker",
                "file_preview_pane",
                "checkbox",
                "radio",
                "switch",
                "meter",
                "sparkline",
                "rule",
                "status_bar",
                "pagination",
                "form",
                "spinner",
            ])
        );
        for m in MOVED {
            assert!(
                m.why.len() > 40,
                "`{}` moved and the row does not say what moved",
                m.id
            );
        }
        // Twenty-five built, four not. **ADR 0033 says thirteen entries have nothing to show**,
        // which would put the built count at sixteen — the Tier 1 count exactly, and it could not
        // be reached even at nineteen without contradicting §17's own three sentences about
        // `slider`, `file_picker` and `file_preview_pane`. Recorded here rather than resolved by
        // bending a column: ticket 34 built six of Tier 2's nine and ticket 35 the other three, and
        // ticket 46 built `spinner`, the one row that was genuinely at risk rather than merely
        // mis-tiered. **So the number that reproduces from the freeze is twenty-nine, which is
        // every row of it** — and this assertion and `INVENTORY.len()` are the same number for the
        // first time.
        assert_eq!(INVENTORY.iter().filter(|c| c.built).count(), 29);
        assert_eq!(
            INVENTORY.iter().filter(|c| c.built).count(),
            INVENTORY.len()
        );
    }

    /// **Every `built` row is declared in the module that homes it — and this is the gate components
    /// ticket 33 shipped because its own row had been lying for two tickets.**
    ///
    /// §14 measured drag capture and concluded *`slider` leaves Tier 3*, so components ticket 30 set
    /// `built: true` on the row and listed the name in [`crate::input::MEMBERS`]. **No `slider`
    /// existed.** Nothing here could see it, and the two joins that look as though they should are
    /// each blind for a stated reason:
    ///
    /// - `the_module_tree_and_the_families_column_agree` compares a module's `MEMBERS` list against
    ///   the `families` **column** and never against the module's source, so a name in both places
    ///   agrees with itself;
    /// - `nothing_is_built_at_a_tier_that_says_otherwise_without_the_row_saying_so` accepts any row
    ///   that appears in [`MOVED`], and `MOVED` records *what argument moved it* — which for `slider`
    ///   was true: the mechanism really was measured. **A mechanism being built is not the component
    ///   being built**, and that is the distinction the column had no gate for.
    ///
    /// Obligation O2 would have caught it eventually — *everything `built` must have a panel* — and
    /// it is [`crate::obligations`]'s and still `Unmet`, which is exactly the shape ADR 0033 exists
    /// to refuse: *every obligation this map has stated as a sentence has been broken by someone who
    /// had read it.*
    ///
    /// # The needle is the name and the boundary is either delimiter
    ///
    /// A join over twenty-nine rows cannot dictate nineteen signatures. `pub fn picture(` could
    /// never have matched the shipped `pub fn picture<P: Pixels>(` — components ticket 30's finding,
    /// met for the fourth time by ticket 32 — so what this looks for is `pub fn <id>` followed by
    /// **`(` or `<`**, and both spellings are watched being accepted. Anything narrower is a gate
    /// that a type parameter deletes.
    #[test]
    fn every_built_row_is_declared_in_the_module_that_homes_it() {
        let mut checked = 0;
        for c in INVENTORY {
            if !c.built {
                continue;
            }
            let module = c.module().unwrap_or_else(|| {
                panic!(
                    "`{}` is built and its first family has no module here",
                    c.id
                )
            });
            let source = read(&format!("crates/vitui-components/src/{module}.rs"));
            let plain = format!("pub fn {}(", c.id);
            let generic = format!("pub fn {}<", c.id);
            assert!(
                crate::dense::declares(&source, &plain)
                    || crate::dense::declares(&source, &generic),
                "`{}` is `built` and `src/{module}.rs` declares neither `{plain}` nor `{generic}`.                  Either the component does not exist — which is what this gate was written for — or                  it is homed in a module the `families` column does not name",
                c.id
            );
            checked += 1;
        }
        assert_eq!(
            checked, 29,
            "**twenty-nine built rows**, and every one of them checked — the freeze is complete \
             since components ticket 46, so this count and `INVENTORY.len()` are the same number \
             for the first time"
        );

        // **Both spellings are accepted, watched.** A join that took only the parenthesis is a join
        // a type parameter deletes, and this crate has already shipped that needle twice.
        assert!(crate::dense::declares(
            "pub fn slider(cx: &mut Ctx) {}",
            "pub fn slider("
        ));
        assert!(crate::dense::declares(
            "pub fn picture<P: Pixels>(cx: &mut Ctx) {}",
            "pub fn picture<"
        ));
        // And a mention is not a declaration, which is `dense::declares`'s own rule.
        assert!(!crate::dense::declares(
            "// pub fn slider(cx: &mut Ctx)",
            "pub fn slider("
        ));
        // **And the column agrees in the other direction too, which is what gives the gate teeth.**
        // Four rows are not built; if any of them were declared, `built` would be understating the
        // crate rather than overstating it — the same drift with the sign flipped, and a gate that
        // only looked at the `true` rows could not see it. Zero of one, counted rather than assumed
        // — and it was the direction that fired when ticket 35 declared its three components before
        // moving their column, which is the reverse-direction half doing exactly its job.
        let declared_but_not_built: Vec<&str> = INVENTORY
            .iter()
            .filter(|c| !c.built)
            .filter(|c| {
                let Some(module) = c.module() else {
                    return false;
                };
                let source = read(&format!("crates/vitui-components/src/{module}.rs"));
                crate::dense::declares(&source, &format!("pub fn {}(", c.id))
                    || crate::dense::declares(&source, &format!("pub fn {}<", c.id))
            })
            .map(|c| c.id)
            .collect();
        assert!(
            declared_but_not_built.is_empty(),
            "{declared_but_not_built:?} are declared and the `built` column says they are not"
        );
        // **Zero, and it was one until components ticket 46.** The reverse arm is not decoration
        // now that it can no longer fire from this side: what it is watching is a row arriving
        // `built: false` — a thirtieth component, or a row struck back — with a declaration already
        // in the tree. That is the direction ticket 35 tripped, declaring its three components
        // before moving their column.
        assert_eq!(INVENTORY.iter().filter(|c| !c.built).count(), 0);
        assert!(
            crate::dense::declares(
                "pub fn nothing_declares_this(cx: &mut Ctx) {}",
                "pub fn nothing_declares_this("
            ),
            "the needle the reverse arm runs on no longer matches anything, so an undeclared row \
             and a declared one would read the same"
        );
    }

    /// **The DAG: an edge from a lower layer to a higher one is refused.**
    ///
    /// The gate needs edges, and [`COMPOSITIONS`] is where the spec's stated ones live. Two
    /// assertions, because the wording that survives the spec's own compositions is not strict
    /// enough to give acyclicity for free:
    ///
    /// 1. no edge runs from a lower rung to a higher one;
    /// 2. the edge set is acyclic, which same-layer edges would otherwise let through.
    #[test]
    fn no_composition_runs_from_a_lower_layer_to_a_higher_one() {
        for e in COMPOSITIONS {
            let of = row(e.of);
            let on = row(e.on);
            assert!(
                of.layer >= on.layer,
                "`{}` is L{} and is built on `{}` at L{} ({}). **A component may depend only on \
                 lower layers**, and this edge runs the other way — either the layers are wrong or \
                 the composition is",
                of.id,
                of.layer.rung(),
                on.id,
                on.layer.rung(),
                e.source
            );
            assert!(of.id != on.id, "`{}` is built on itself", of.id);
        }

        // Acyclicity, by repeated removal of sinks. Same-layer edges are legitimate — §6's `table`
        // = `collection` is one — so the rung comparison above cannot carry this on its own.
        let mut edges: Vec<(&str, &str)> = COMPOSITIONS.iter().map(|e| (e.of, e.on)).collect();
        let mut settled: BTreeSet<&str> = BTreeSet::new();
        loop {
            let sinks: Vec<&str> = INVENTORY
                .iter()
                .map(|c| c.id)
                .filter(|id| !settled.contains(id))
                .filter(|id| !edges.iter().any(|(of, _)| of == id))
                .collect();
            if sinks.is_empty() {
                break;
            }
            for s in sinks {
                settled.insert(s);
                edges.retain(|(_, on)| *on != s);
            }
        }
        assert_eq!(
            settled.len(),
            INVENTORY.len(),
            "the composition graph has a cycle: {:?} never became a sink",
            INVENTORY
                .iter()
                .map(|c| c.id)
                .filter(|id| !settled.contains(id))
                .collect::<Vec<_>>()
        );

        for e in COMPOSITIONS {
            assert!(
                e.source.contains('§') || e.source.contains("ticket"),
                "the edge `{}` → `{}` does not name where it is stated, which is the whole \
                 difference between a stated edge and a guessed one",
                e.of,
                e.on
            );
        }
    }

    /// **`constructions` is 1..=3, and three rows carry the numbers §17 names.**
    ///
    /// The three are the criterion: `chart` 2, `meter` 2, `plot` 3. All three are derived from
    /// `CONTEXT.md`'s rungs — *ASCII only*, *Unicode with box drawing and block elements*,
    /// *everything including braille and emoji* — and from no table in the runtime. §22 records
    /// `theme::subrows` as putting the eighth blocks at the wrong rung; runtime ticket 17's module
    /// map ships `theme` and `theme::registry` and names no `subrows` at all, so this column is not
    /// waiting on it in either direction.
    #[test]
    fn constructions_is_one_to_three_and_the_block_ladder_is_two() {
        for c in INVENTORY {
            assert!(
                (1..=3).contains(&c.constructions),
                "`{}` builds {} different things across three declared repertoires",
                c.id,
                c.constructions
            );
        }
        assert_eq!(row("chart").constructions, 2, "chart");
        assert_eq!(row("meter").constructions, 2, "meter");
        assert_eq!(row("plot").constructions, 3, "plot");

        // **Two rows earn the third rung, and they earn it for the same reason.** §13's
        // measurement is `plot`'s: Unicode against Extended is 0 cells different in the chart pane
        // and 882 in the plot pane, because braille is 256 states a cell where block elements are
        // 8. `spinner` is components ticket 46's and the same sentence — ten braille frames where
        // the quadrant blocks give four — which is why it is `plot`'s shape and not `meter`'s,
        // whose horizontal ladder gets nothing from the top rung and reads `1 / 8 / 8`.
        //
        // Both are **derived**: `chart::raster::geom` for `plot`, `crate::indicate::constructions`
        // for `spinner`, each asserted against this column in the module that owns the table.
        let three: Vec<&str> = INVENTORY
            .iter()
            .filter(|c| c.constructions == 3)
            .map(|c| c.id)
            .collect();
        assert_eq!(three, vec!["plot", "spinner"]);
    }

    /// **The four hostile axes are columns, and every one of them is set somewhere.**
    ///
    /// An axis no row declares is an axis O5 can never ask about, which is the failure mode ADR
    /// 0033 is built around: *a gate is only enumerable against a list*.
    #[test]
    fn every_hostile_axis_is_declared_by_at_least_one_row() {
        for axis in Axis::ALL {
            let rows: Vec<&str> = INVENTORY
                .iter()
                .filter(|c| c.declares(axis))
                .map(|c| c.id)
                .collect();
            assert!(
                !rows.is_empty(),
                "no row declares `{}`, so O5 can never ask about it",
                axis.name()
            );
        }

        // The four defect sets, by name. Each was established by a defect that passed every gate
        // then in force **and looked healthier**, so a row quietly leaving one of these sets is
        // exactly the diff nobody would question.
        let of = |axis: Axis| -> BTreeSet<&str> {
            INVENTORY
                .iter()
                .filter(|c| c.declares(axis))
                .map(|c| c.id)
                .collect()
        };
        assert_eq!(
            of(Axis::Scrolled),
            BTreeSet::from([
                "field",
                "collection",
                "table",
                "tree",
                "select",
                "scroll_area",
                "sticky",
                "file_picker",
                "file_preview_pane",
            ])
        );
        assert_eq!(
            of(Axis::Shrunk),
            BTreeSet::from([
                "field",
                "collection",
                "table",
                "tree",
                "scroll_area",
                "collapsible",
                "file_picker",
                "file_preview_pane",
            ])
        );
        assert_eq!(
            of(Axis::Wheeled),
            BTreeSet::from([
                "field",
                "collection",
                "table",
                "tree",
                "select",
                "scroll_area",
                "file_picker",
                "file_preview_pane",
            ])
        );
        assert_eq!(
            of(Axis::Narrow),
            BTreeSet::from([
                "text",
                "chip",
                "field",
                "table",
                "tree",
                "scroll_area",
                "scrollbar",
                "chart",
                "plot",
            ])
        );
    }

    /// **The module tree and the `families` column agree, in both directions.**
    ///
    /// Spec §19: *a module is a family*, and the `families` column is the join that makes the
    /// mapping checkable rather than a naming convention. Run one way only, this would pass on a
    /// module that lists a component nobody homed there.
    #[test]
    fn the_module_tree_and_the_families_column_agree() {
        let mut homed: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for c in INVENTORY {
            assert!(
                !c.families.is_empty(),
                "`{}` declares no family, so it has no home",
                c.id
            );
            let mut seen = BTreeSet::new();
            for f in c.families {
                assert!(seen.insert(*f), "`{}` declares {f:?} twice", c.id);
            }
            let module = c.module().unwrap_or_else(|| {
                panic!(
                    "`{}` is homed in F15, whose entries emit no cells and are the runtime's",
                    c.id
                )
            });
            homed.entry(module).or_default().push(c.id);
        }

        for family in Family::ALL {
            let Some(module) = family.module() else {
                assert!(
                    family.members().is_empty(),
                    "{:?} has no module here and lists members",
                    family
                );
                continue;
            };
            let expected: Vec<&str> = homed.get(module).cloned().unwrap_or_default();
            let listed: Vec<&str> = family.members().to_vec();
            assert_eq!(
                listed, expected,
                "`src/{module}.rs` and the `families` column disagree about {:?}. The module lists \
                 {listed:?}; the rows homed there are {expected:?}",
                family
            );
        }

        // Every module named by the tree is a file that exists, which is `register.rs`'s rule one
        // layer up: an instrument that names something vague cannot stop being true.
        for family in Family::ALL {
            let Some(module) = family.module() else {
                continue;
            };
            let source = read(&format!("crates/vitui-components/src/{module}.rs"));
            assert!(
                source.contains("pub const MEMBERS"),
                "`src/{module}.rs` does not declare `MEMBERS`, so the join has nothing to run over"
            );
        }

        // Five families ship no v1 component: F8, F11, F13, F14 and F15. **§17 says two** — F11 and
        // F12 — and that sentence does not reproduce against §17's own table, which puts
        // `file_picker` and `file_preview_pane` in F12 and gives F8, F13 and F14 nothing. Asserted
        // rather than quietly worked around.
        let empty: Vec<u8> = Family::ALL
            .iter()
            .filter(|f| f.members().is_empty())
            .map(|f| f.number())
            .collect();
        assert_eq!(empty, vec![8, 11, 13, 14, 15]);
        assert_eq!(
            homed.values().map(Vec::len).sum::<usize>(),
            INVENTORY.len(),
            "a component is homed in exactly one module"
        );
    }

    /// **The six collapses are recorded, and the judgement is marked as one.**
    ///
    /// The criterion asks for a doc comment on [`INVENTORY`] and a doc comment is prose, which is
    /// the shape ADR 0033 exists to distrust. So the test opens this file and reads it: five
    /// collapses that a ticket measured, one that is an argument, and the word `judgement` beside
    /// the sixth. §17's own sentence is *five measured and one a judgement*, and ticket 34 repeats
    /// it so that the marking cannot be lost by an edit in one place.
    #[test]
    fn the_six_collapses_are_recorded_and_the_judgement_is_marked() {
        let source = read("crates/vitui-components/src/inventory.rs");
        let doc = source
            .split_once("pub const INVENTORY")
            .expect("the value is in this file")
            .0;
        let doc = doc
            .rsplit_once("/// Spec §17's v1 freeze")
            .expect("the doc comment opens with §17's sentence")
            .1;

        for collapse in [
            "accordion → `collapsible`",
            "tabs → `collection`",
            "menu → `collection`",
            "textarea → `field`",
            "tooltip → `overlay`",
            "gauge → `meter`",
        ] {
            assert!(
                doc.contains(collapse),
                "`INVENTORY`'s doc comment does not record the collapse {collapse}. Six are folded \
                 into the freeze and a reader who cannot see them will file one as a missing row"
            );
        }

        let gauge = doc
            .split_once("gauge → `meter`")
            .expect("checked above")
            .1
            .lines()
            .next()
            .unwrap_or_default();
        assert!(
            gauge.contains("judgement") && gauge.contains("not a measurement")
                || gauge.contains("judgement rather than a measurement"),
            "the gauge collapse is the only one in the freeze that rests on an argument rather \
             than a number, and its row no longer says so: {gauge:?}"
        );
        assert!(
            doc.contains("`chip`") && doc.contains("`plot`") && doc.contains("`sticky`"),
            "the three additions no frequency count produces are what make the freeze a decision \
             rather than an arithmetic, and the doc comment no longer names them"
        );
    }

    /// **The glyph table is twenty entries and every one of them has a demander.**
    ///
    /// The tripwire ticket 01 left, one number further on. It read *still the seven these demand
    /// sets were written against*, and it fired: §16 took the table to twenty — the four arrows, the
    /// four box corners, `Ellipsis`, the four tees and `Cross` — and ticket 05 grew the rows with
    /// it, so `panel` names a corner and `tree` names its `ArrowRight` chevron.
    ///
    /// **What is kept is the shape, not the number.** A demand set short for a stated reason is a
    /// different thing from one that is wrong, and the difference stops being visible the moment the
    /// table moves without these rows moving. The second half is new and is the direction that was
    /// unguardable while the column was a stub: **an entry no row draws is a table row written for a
    /// reader rather than for a caller**, which is how a catalogue grows entries nine private
    /// fallback tables then disagree about.
    #[test]
    fn the_glyph_table_is_twenty_entries_and_every_one_of_them_has_a_demander() {
        assert_eq!(
            Glyph::ALL.len(),
            20,
            "`vitui_runtime::Glyph` has moved off spec §16's twenty. The `glyphs` column of \
             `INVENTORY` is joined against it row by row — grow the rows, then this number"
        );

        let mut demanded = BTreeSet::new();
        for c in INVENTORY {
            let mut seen = BTreeSet::new();
            for g in c.glyphs {
                assert!(
                    seen.insert(format!("{g:?}")),
                    "`{}` demands {g:?} twice",
                    c.id
                );
                demanded.insert(format!("{g:?}"));
            }
        }
        assert_eq!(
            demanded.len(),
            Glyph::ALL.len(),
            "an entry of the table is drawn by no row of the freeze"
        );
        assert_eq!(
            INVENTORY.iter().filter(|c| !c.glyphs.is_empty()).count(),
            21,
            "twenty-one of the twenty-nine rows draw at least one glyph. **It was twenty until \
             components ticket 35**, and the row that arrived is `form` — with `Ellipsis` alone, \
             because a form draws no glyph of its own and a label wider than its column is elided"
        );
    }

    /// **Occurrences of the repertoire type's path in `vitui-components` == 0, and no private
    /// fallback module.**
    ///
    /// §17's first glyph gate, green here and **pinned red on the register** — nine crates of
    /// twelve grew a private fallback module with six glyph literals and a `match` on the
    /// repertoire, byte-identical in all nine. It runs from this ticket because it costs six lines
    /// and because a gate that starts green is the only kind that can catch the first violation
    /// rather than the ninth. Ticket 05 owns inverting the register's red row; this is the same
    /// count over the one crate that has to keep it at zero.
    ///
    /// **Neither needle is spelled anywhere in this file, including here**, and watching this test
    /// fail is how that was found: the version whose doc comment quoted the two literals reported
    /// `src/inventory.rs` as the violator. A source scan that its own documentation satisfies is
    /// the mirror image of `register.rs`'s vacuous scan one crate down — that one was always green,
    /// this one was always red, and neither was looking at the code.
    ///
    /// # One file is excepted, and the exception is not decided here — components ticket 27
    ///
    /// `CONTEXT.md` says both halves of a collision in two adjacent paragraphs: **Repertoire** — *a
    /// component branches on it* — and **Glyph** — *the sub-cell ladders are the case … a component
    /// names no repertoire*. [`crate::chart::raster::geom`] is that ladder, and §21's refinement 3
    /// says what to do: **name the exception; do not loosen the gate.**
    ///
    /// The exception is argued in **one** place, `crate::gates`'s own scan, which asserts the exact
    /// three lines that may spell a repertoire in that file. This one skips the file by name and
    /// says where the real check is, because *two* gates each keeping their own idea of the
    /// exception is how the two come to disagree — which is the failure mode the register's near
    /// miss records. What is not skipped is the half this test shares with it: no private fallback
    /// module, in that file or in any other.
    ///
    /// # It was not recursive, and until this crate had a subdirectory nothing could tell
    ///
    /// The walk was one `read_dir` over `src/`, so a component in a subdirectory was never scanned
    /// at all. `chart/` is **the first subdirectory this crate has ever had** — components ticket 28
    /// added it — and the defect became reachable and was found in the same commit: the exception
    /// pointed at `src/series.rs`, which by then named a repertoire only in a doc comment, while
    /// `src/chart/raster.rs`, which names three, was outside the walk. A gate that agrees with its
    /// twin about a file neither of them opened is the shape this crate's register already records
    /// twice.
    #[test]
    fn no_component_source_names_the_repertoire() {
        /// Every `.rs` file under `dir`, recursively.
        fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.expect("a readable directory entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }

        // The one file `crate::gates` excepts, by name. See this test's documentation.
        const LADDER: &str = "chart/raster.rs";
        let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut files = Vec::new();
        walk(&src, &mut files);
        assert!(
            files.len() > 20,
            "the walk found only {} files, so whatever it reports is about the walk",
            files.len()
        );
        assert!(
            files.iter().any(|p| p.parent() != Some(src.as_path())),
            "the walk reached no subdirectory, and this crate has one. A non-recursive scan of \
             `src/` reports zero about every module it never opened"
        );

        let mut excepted = 0usize;
        for path in &files {
            let source = std::fs::read_to_string(path).expect("a readable source file");
            let name = path
                .strip_prefix(&src)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            // **Both needles are assembled rather than written.** A scan for a literal that its
            // own message contains finds itself in every file that carries the gate — which is the
            // vacuous-source-scan failure `register.rs` records one layer down, arriving from the
            // other side.
            let repertoire = format!("{}{}", "Glyph", "Set::");
            let private_table = format!("{} {}", "mod", "missing");
            // Code, not prose: a line that is a comment names nothing, which is what separates the
            // ladder from the four files that only *talk* about it.
            let names_it = source
                .lines()
                .map(str::trim_start)
                .any(|line| !line.starts_with("//") && line.contains(&repertoire));
            if name == LADDER {
                excepted += 1;
                assert!(
                    names_it,
                    "`src/{LADDER}` is excepted from the repertoire count and no longer needs to \
                     be. Drop the exception here and in `crate::gates`, in that order"
                );
            } else {
                assert!(
                    !names_it,
                    "`src/{name}` names the repertoire type by path. A component names a role and \
                     a glyph and never a repertoire — the theme owns the table because there was \
                     nowhere else to put one. The one exception is `src/{LADDER}`, and it is \
                     argued in `crate::gates`"
                );
            }
            assert!(
                !source.contains(&private_table),
                "`src/{name}` grows a private fallback module, which is the exact shape nine \
                 crates of twelve shipped byte-identically: six glyph literals and a match on the \
                 repertoire"
            );
        }
        assert_eq!(
            excepted, 1,
            "the one named exception was never reached, so this scan agrees with `crate::gates` \
             about a file neither of them looked at"
        );
    }
}
