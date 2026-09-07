//! **F4 disclosure**, 21 entries, expressed by `collapsible` alone.
//!
//! The reduction is R1, and **the family exists in order to say so**: spec §18's own sentence is
//! *every one of these is the same state machine*, which is the shape of claim that has to be
//! measured rather than asserted. §8 measured it; components ticket 22 built it.
//!
//! # One machine, and the split is not between the four components
//!
//! Accordion, tree node, code folding and inplace edit are one machine. What separates them is
//! **what their collapsed content is**, and that is [`SPLIT`] — a value the tests iterate rather
//! than a table in a comment, because §17's own rule is that every obligation stated as a sentence
//! has been broken by someone who had read it.
//!
//! | the collapsed content is | who collapses it | example |
//! |---|---|---|
//! | rows in a caller-owned index | the caller, on request | tree node, code folding |
//! | a region of components | the component, on the frame | accordion, panel minimise |
//! | both | both | inplace edit |
//!
//! [`Collapses`] is that column as a configuration, and it is the **only** thing that differs
//! between the four: one [`Collapse`], one [`Collapse::set`], one draw. `Collapses::OnRequest`
//! leaves the state untouched and reports the gesture; `Collapses::OnTheFrame` applies it before it
//! draws. **A component may not perform an edit; a collapse of a region is not one** — §8's rule,
//! narrowed by C05's two forced reasons (`Vec::splice` allocates, and `&mut` while the draw holds
//! the index shared is `E0502`), both of which belong to the index and neither of which a region
//! has.
//!
//! **The index row already ships, one family over.** [`crate::collect::tree`] is §8's first row
//! expressed through the collection lineage: a fold leaves `order::Ask::Collapse` in a one-slot
//! request and the caller splices after the draw. What `Collapses::OnRequest` is for is
//! the same configuration reached from *this* side — a section **header** over rows in a
//! caller-owned index, which is code folding rather than a tree node — and it is the same machine
//! rather than a second one, which is what makes the split checkable instead of a claim.
//!
//! # There is no transition state, and the gate is a scan
//!
//! No `Collapsing` and no `Expanding`. A transition state has to be **stored**, which means the
//! machine can be found halfway between two states with no clock running — the shape R11 refused
//! when it refused an animation object. [`Collapse::set`] flips `open` **at the instant the gesture
//! lands** and starts a tween from the current height, so `open` is never ambiguous and only the
//! height moves.
//!
//! The invariant is registered twice, from two sides: behaviourally, over every frame of a 200 ms
//! collapse, and as a **source scan** for a stored third state — which is why the two
//! words above appear in this file only inside comments, and why
//! [`third_state_declarations`] is watched finding one.
//!
//! **An animated fold is refused**, and the reason is in the doc comment rather than left as a
//! silent absence: the removed rows would have to still be in the index while they shrink, and the
//! splice is an edit a frame may not perform. **A fold steps; a region animates.** See
//! [`Collapses::OnRequest`].
//!
//! # The height is an argument, and the watermark is §9's finding on the other axis
//!
//! The runtime has no measure pass, so a section's open height comes from a **sizing
//! function beside the body** — [`sizing::check`]'s own `FnMut(u16) -> u16`, which is the shape the
//! runtime already gates a component against.
//!
//! [`Height::Watermark`] is the spelling a reader writes instead: hand the body a rectangle and read
//! how far it reached. §8 prices it at *83 cells, 8 rows wrong, settled in 3 frames* against the
//! sizing function's *22, 0, 2*, and **those three figures are a prototype's body and do not
//! reproduce here** — see [`SPEC_WATERMARK_ROWS_WRONG`]. What reproduces is stronger, and it is §9's own
//! sentence one axis over:
//!
//! > A measured extent is taken **inside the rectangle the decision produced**.
//!
//! Fed back into itself over a body that fills what it is handed — which is what a padding ring
//! *is* — the watermark **latches at the first rectangle it ever saw and never comes down**:
//! [`WATERMARK_LATCHED_ROWS`] rows where [`CONTENT_ROWS`] are right, permanently, on a screen that
//! looks correct. Over a body that draws only its content the two arms are indistinguishable, which
//! is what makes the rule unconditional rather than a preference — the identical shape §9 states for
//! a hideable reserved bar.
//!
//! It is **off by default** for that reason and for its price: one dry run through
//! [`Ctx::measured`] a frame, reported by `examples/collapsible_numbers.rs`.
//!
//! # Focus, and the vanish rule that no header gesture can reach
//!
//! **A collapsible closed by a gesture on its own header focuses that header** — which is what a
//! focusable widget does on a click anyway — and then R08's vanish rule never fires: left to it, the
//! focus lands on the *next surviving entry*, one section too far, while the section the user acted
//! on is still on screen one row above.
//!
//! Following §8's own parenthesis to the end is this module's finding. **All three self-close
//! gestures leave the focus off the body before the vanish rule looks**, each for a different
//! reason: a click on a focusable header is awarded the focus; a click on a header that is *not* a
//! tab stop **defocuses**, because a press that lands on nothing interested is read as intent; and
//! `Enter` needs the header to hold the focus already. So the zero is the gate and the figure beside
//! it ([`SPEC_VANISH_PROBES`]) is **not reachable from a header gesture at all** — the arm that pays
//! it is a section closed by something that is not one, which is exclusive mode, and
//! [`crate::accordion`] runs it at §8's own scale.
//!
//! What [`Focus::Header`] buys is measured rather than argued, and it is not the probe count: on a
//! header that is not a tab stop the runtime's answer is `None`, so **the click loses the keyboard
//! entirely** — architecture issue 25's own finding one component over — and the rule is what puts
//! it somewhere.
//!
//! **`Stash` does not generalise to a region, and [`Drop`](Focus::Drop) is this component's
//! answer.** C05 could stash a selection because a selection is positions in an index the component
//! was handed. A region collapsible is handed a **closure**, and the ids inside it are minted at
//! that closure's own call sites — `Ctx::id` mints from `Location::caller()` — so the component has
//! nothing to key a stash on. The caller can, and does, by capturing `Frame::focus`. There is no
//! region `Stash` and [`WhyThereIsNoRegionStash`] keeps it unwritable.
//!
//! [`sizing::check`]: vitui_runtime::sizing::check
//! [`Ctx::measured`]: vitui_runtime::Ctx::measured
//! [`Frame::vanish_probes`]: vitui_runtime::ctx::Frame::vanish_probes

use std::time::{Duration, Instant};

use vitui_runtime::anim::Tween;
use vitui_runtime::keys::{Code, Edge};
use vitui_runtime::{Ctx, Glyph, Id, Interest, Rect, Response, Role};

use crate::frame::{Face, face_paint};
use crate::ink::{Direct, Ink};
use crate::text::{FitOpts, Justify, fit_into};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["collapsible"];

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// §8's three-row split, as a value
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Who performs the collapse.** §8's middle column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    /// The caller, on request. The component asks and the caller drains the ask after the draw.
    Caller,
    /// The component, on the frame the gesture lands.
    Component,
    /// Both: a region inside a row of an index.
    Both,
}

impl Who {
    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Who::Caller => "the caller, on request",
            Who::Component => "the component, on the frame",
            Who::Both => "both",
        }
    }

    /// **The configuration this row of the split is expressed by.** The join that makes §8's table
    /// checkable: three rows, three arms, and no fourth on either side.
    pub const fn collapses(self) -> Collapses {
        match self {
            Who::Caller => Collapses::OnRequest,
            Who::Component => Collapses::OnTheFrame,
            Who::Both => Collapses::Both,
        }
    }
}

/// One row of §8's split.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Split {
    /// What the collapsed content is.
    pub content: &'static str,
    /// Who collapses it.
    pub who: Who,
    /// The components that are this row.
    pub examples: &'static [&'static str],
}

/// **§8's three-row table, as a value.** Criterion 1: *the three-row table above as its documented
/// split*, and a documented split a test can iterate is the only kind that survives a ticket.
pub const SPLIT: [Split; 3] = [
    Split {
        content: "rows in a caller-owned index",
        who: Who::Caller,
        examples: &["tree node", "code folding"],
    },
    Split {
        content: "a region of components",
        who: Who::Component,
        examples: &["accordion", "panel minimise"],
    },
    Split {
        content: "both",
        who: Who::Both,
        examples: &["inplace edit"],
    },
];

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the machine
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Everything a section keeps across frames: whether it is open, how tall it is drawn, and an
/// optional tween.**
///
/// Per section, and **never per row of content** — §8's own words, and the half that matters is the
/// second. A fold set is a `Vec<u32>` of line numbers beside the caller's document
/// ([`crate::accordion::Folds`]); what is per section is this.
///
/// # `open` is never ambiguous, and there are two fields rather than three
///
/// [`Collapse::set`] writes `open` immediately and leaves the height to the tween, so there is no
/// state in which the machine is *between* open and closed. The height is the **drawn** height:
/// while a tween runs it is what the tween says, and when the tween is dropped it is where the tween
/// landed. That is why there is no stored target — [`Tween::to`] is the target while one runs, and
/// afterwards `h` *is* it.
///
/// ```
/// use std::time::{Duration, Instant};
/// use vitui_components::disclose::Collapse;
///
/// let now = Instant::now();
/// let mut c = Collapse::shut();
/// assert!(!c.open() && c.height() == 0);
///
/// // The gesture lands: `open` flips now, and only the height moves.
/// c.set(now, true, 10, Duration::from_millis(200));
/// assert!(c.open(), "never ambiguous");
/// assert_eq!(c.height(), 0, "and the height has not moved yet");
///
/// // Halfway.
/// assert_eq!(c.advance(now + Duration::from_millis(100)), Some(now + Duration::from_millis(100)));
/// assert_eq!(c.height(), 5);
///
/// // Arrived, and the screen may sleep: no further frame is asked for.
/// assert_eq!(c.advance(now + Duration::from_millis(200)), None);
/// assert_eq!(c.height(), 10);
/// assert!(!c.animating());
/// ```
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Collapse {
    /// Whether the section is open. **Flipped at the instant the gesture lands.**
    open: bool,
    /// The height it is drawn at, in rows. Zero is closed.
    h: u16,
    /// The tween, while one runs. `None` is a steady section, open or shut.
    ///
    /// **The slot is not free when it is empty**, which is the one place §8's byte pair cannot both
    /// be true — see [`COLLAPSE_BYTES`].
    tween: Option<Tween<u16>>,
}

impl Collapse {
    /// A closed section.
    pub const fn shut() -> Collapse {
        Collapse {
            open: false,
            h: 0,
            tween: None,
        }
    }

    /// A section already open at `h`, with nothing running.
    pub const fn open_at(h: u16) -> Collapse {
        Collapse {
            open: true,
            h,
            tween: None,
        }
    }

    /// Whether it is open. **Never ambiguous**: there is no third answer and no state between two.
    pub const fn open(&self) -> bool {
        self.open
    }

    /// The height it is drawn at, in rows.
    pub const fn height(&self) -> u16 {
        self.h
    }

    /// Whether a tween is running.
    pub const fn animating(&self) -> bool {
        self.tween.is_some()
    }

    /// **The gesture landed.** `open` flips now; the height tweens from where it is to `to`.
    ///
    /// A zero `dur` **steps**, which is what a fold does and what a region does when the theme asks
    /// for no motion. It leaves no tween, so a stepped collapse asks for no further frame at all.
    pub fn set(&mut self, now: Instant, open: bool, to: u16, dur: Duration) {
        self.open = open;
        if dur.is_zero() || to == self.h {
            self.h = to;
            self.tween = None;
            return;
        }
        self.tween = Some(Tween::new(now, dur, self.h, to));
    }

    /// [`Collapse::set`] with `open` flipped and the same duration.
    pub fn toggle(&mut self, now: Instant, to_open: u16, dur: Duration) {
        let open = !self.open;
        self.set(now, open, if open { to_open } else { 0 }, dur);
    }

    /// **Move the height to `now`, and answer the frame it still wants.**
    ///
    /// `None` means *nothing is running*, which is what lets the screen sleep — a component that
    /// asks for a wake without asking this is a spin ([`Tween::done`]).
    ///
    /// The interpolation is done here rather than by `Tween::value` because the runtime ships that
    /// method for `f32` and `i32` and **not** for `u16`, which is the width §8 names. A height is a
    /// row count and `Rect::h` is a `u16`, so the type is right and the arithmetic is three lines:
    /// [`Tween::phase`], [`Tween::from`], [`Tween::to`].
    pub fn advance(&mut self, now: Instant) -> Option<Instant> {
        let t = self.tween?;
        if t.done(now) {
            self.h = t.to();
            self.tween = None;
            return None;
        }
        let (from, to) = (f64::from(t.from()), f64::from(t.to()));
        let at = from + (to - from) * f64::from(t.phase(now));
        self.h = at.round().clamp(0.0, f64::from(u16::MAX)) as u16;
        t.wake(now)
    }

    /// **The target the height is moving to**, or the height itself when nothing runs.
    pub fn target(&self) -> u16 {
        self.tween.map_or(self.h, |t| t.to())
    }
}

/// **What a [`Collapse`] costs. Live state, and the slot beside it.**
///
/// §8 states *5 B of live state, 72 B with a tween slot* and **neither number reproduces**, for two
/// different reasons, both of which are recorded rather than engineered away:
///
/// - The live half is `open: bool` and `h: u16` — **three bytes of field and four of struct**. Five
///   is what a third `u16` would cost, and there is no third: `Tween::to` is the target while a
///   tween runs and `h` is it afterwards, so a stored target would be a fourth spelling of a number
///   that already has two.
/// - The tween half is [`TWEEN_BYTES`], and `Tween<u16>` is what the runtime ships: two 16-byte
///   moments, two `u16`s and an easing. Seventy-two is a wider tween than this map has.
///
/// **And the pair cannot both be a `size_of` of one type at all**, which is the part worth keeping.
/// `Option<Tween<u16>>` is a field, so it costs its forty bytes whether or not it holds one: a
/// `Collapse` with no tween running is [`COLLAPSE_BYTES`] and not [`LIVE_BYTES`]. Reading §8's pair
/// as two sizes of one record is reading it as *a record whose tween lives somewhere else*, and
/// nothing on this map has anywhere else to put one.
pub const COLLAPSE_BYTES: usize = size_of::<Collapse>();

/// **The four bytes of padding between the two halves**, so the subtraction is stated rather than
/// left as a discrepancy a reader has to work out. `Tween` carries two `Instant`s, so the record's
/// alignment is eight and [`LIVE_BYTES`] rounds up to it.
pub const PADDING_BYTES: usize = COLLAPSE_BYTES - LIVE_BYTES - TWEEN_BYTES;

/// **The live half of [`Collapse`], as a type of its own so the number is a `size_of` and not a
/// sum.** `open` and the height, which is everything the machine needs to answer a frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Live {
    open: bool,
    h: u16,
}

/// The live half's size. **Four**, against §8's five. See [`COLLAPSE_BYTES`].
pub const LIVE_BYTES: usize = size_of::<Live>();

/// What the tween slot costs, empty or full. **Forty.**
pub const TWEEN_BYTES: usize = size_of::<Option<Tween<u16>>>();

/// §8's live figure. A **disagreement**, asserted as one: see [`COLLAPSE_BYTES`].
pub const SPEC_LIVE_BYTES: usize = 5;
/// §8's with-a-tween figure. A disagreement, asserted as one.
pub const SPEC_TWEEN_BYTES: usize = 72;

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the four configurations
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Who applies the collapse**, which is [`SPLIT`]'s middle column as a configuration.
///
/// Three arms and not four components. The difference between a tree node and an accordion section
/// is exactly this field.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Collapses {
    /// **The region case.** The component flips [`Collapse::open`] and starts the tween before it
    /// draws, so the frame the gesture lands on is already the collapsed frame. The default.
    #[default]
    OnTheFrame,
    /// **The index case.** The state is left untouched and the gesture is reported; the caller
    /// splices its own index after the draw — `order::Ask` and `order::Asked` are the slot, and
    /// [`crate::collect::tree`] is the shipped consumer.
    ///
    /// **No tween is started, and that is a refusal rather than an omission.** An animated fold
    /// would need the removed rows to still be in the index while they shrink, and the splice is an
    /// edit a frame may not perform. **A fold steps; a region animates.**
    OnRequest,
    /// **Both**: the region collapses on the frame *and* the gesture is reported, which is the
    /// inplace-edit row of [`SPLIT`] — a detail region inside a row of an index.
    Both,
}

impl Collapses {
    /// All three, so a report iterates rather than samples.
    pub const ALL: [Collapses; 3] = [Collapses::OnTheFrame, Collapses::OnRequest, Collapses::Both];

    /// Whether the component applies the collapse to its own state.
    pub const fn applies(self) -> bool {
        matches!(self, Collapses::OnTheFrame | Collapses::Both)
    }

    /// Whether the gesture is reported for the caller to apply to an index.
    pub const fn reports(self) -> bool {
        matches!(self, Collapses::OnRequest | Collapses::Both)
    }

    /// Whether a tween may run under this configuration. **False for a fold.**
    pub const fn animates(self) -> bool {
        self.applies()
    }

    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Collapses::OnTheFrame => "on the frame",
            Collapses::OnRequest => "on request",
            Collapses::Both => "both",
        }
    }
}

/// **Where the open height comes from.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Height {
    /// **The rule.** The sizing function beside the body, which is the argument `collapsible` takes.
    #[default]
    Sized,
    /// **The negative case, off by default.** A dry run through `Ctx::measured` inside the rectangle
    /// the last decision produced — see this module's header for why that latches.
    Watermark,
}

impl Height {
    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Height::Sized => "a sizing function",
            Height::Watermark => "the drawn extent",
        }
    }
}

/// **Where the focus goes when a section closes.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Focus {
    /// **The rule.** The gesture's own header takes the focus, so nothing vanishes and the vanish
    /// rule is never reached: `vanish_probes == 0`.
    #[default]
    Header,
    /// **The negative case.** Whatever was focused inside the body vanishes with it, and R08's
    /// vanish rule lands the focus on the *next surviving entry* — one section too far, while the
    /// section the user acted on is still on screen one row above.
    Drop,
}

impl Focus {
    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Focus::Header => "the header takes it",
            Focus::Drop => "dropped to the vanish rule",
        }
    }
}

/// **There is no region `Stash`, and this is what keeps it unwritable.**
///
/// A region collapsible is handed a closure and the ids inside it are minted at that closure's own
/// call sites, so the component has nothing to key a stash on. The caller can, by capturing
/// `Frame::focus` before the collapse and putting it back after.
///
/// **Protects:** [`Collapse::set`], [`Focus::Drop`].
///
/// The twin, naming both by path so a rename fails **here** rather than turning the hostile case
/// below into one that passes because the item vanished:
///
/// ```
/// use std::time::{Duration, Instant};
/// use vitui_components::disclose::{Collapse, DiscloseOpts, Focus};
///
/// fn protected(c: &mut Collapse, now: Instant) -> Focus {
///     Collapse::set(c, now, false, 0, Duration::ZERO);
///     DiscloseOpts { focus: Focus::Drop, ..DiscloseOpts::default() }.focus
/// }
///
/// let mut c = Collapse::open_at(10);
/// assert_eq!(protected(&mut c, Instant::now()), Focus::Drop);
/// assert!(!c.open());
/// ```
///
/// and the hostile half: a policy for a region's identity is not on the options, because the
/// component does not own that identity.
///
/// ```compile_fail,E0560
/// use vitui_components::disclose::DiscloseOpts;
/// use vitui_components::order::Policy;
///
/// let _ = DiscloseOpts { stash: Policy::Stash, ..DiscloseOpts::default() };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WhyThereIsNoRegionStash;

/// [`collapsible`]'s options. Spec §1's rule 3: a `Default` struct, never a required builder.
///
/// **The title is not here** — it is the section's data, and rule 2 puts data in the argument list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DiscloseOpts {
    /// Who applies the collapse.
    pub collapses: Collapses,
    /// Where the open height comes from. [`Height::Sized`] by default.
    pub height: Height,
    /// Where the focus goes when a section closes. [`Focus::Header`] by default.
    pub focus: Focus,
    /// How long a region's height takes to move. **Zero steps**, which is what a fold does.
    pub dur: Duration,
    /// The role the header's chevron and title are drawn in.
    pub head: Role,
    /// The role the header's padding is drawn in.
    pub pad: Role,
    /// What the header declares. `CLICK | FOCUS`: a section header is a tab stop.
    pub interest: Interest,
}

impl Default for DiscloseOpts {
    fn default() -> DiscloseOpts {
        DiscloseOpts {
            collapses: Collapses::OnTheFrame,
            height: Height::Sized,
            focus: Focus::Header,
            dur: Duration::ZERO,
            head: Role::Title,
            pad: Role::Body,
            interest: Interest::CLICK.with(Interest::FOCUS),
        }
    }
}

/// **The gesture that landed on a header, and what it means.**
///
/// Reported whatever [`Collapses`] says, so a caller can react to a region collapse it did not have
/// to perform. Under [`Collapses::OnRequest`] it is the *only* thing that happened: the state is
/// untouched and this is the request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Toggle {
    /// What the section is being asked to become.
    pub open: bool,
    /// Whether the component has already applied it to the [`Collapse`] it was handed.
    pub applied: bool,
}

/// **What [`collapsible`] answers: what happened to the header, what it handed over, and how much of
/// its rectangle it used.**
///
/// Three fields because §1's rule 4 and §2's *the cells it does not write are named in its return
/// value* are two obligations and a container owes both — and a section owes the second **twice**:
/// the body it handed over, and every row of `rect` below [`Disclosure::used`], which belongs to
/// whoever stacked the sections.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Disclosure {
    /// What happened to the header. Rule 4.
    pub response: Response,
    /// **The body it handed over and did not write.** Empty when the section is closed.
    pub body: Rect,
    /// **Rows of `rect` this section occupied**: one for the header plus the body's height. Every
    /// row below is the caller's, and this is how it is named.
    pub used: u16,
    /// The gesture, if one landed on the header this frame.
    pub gesture: Option<Toggle>,
    /// **The frame the section still wants**, or `None` because nothing is running.
    pub wake: Option<Instant>,
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the component
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **A header row, and a body that is drawn inside the rectangle it was handed — or is not called at
/// all.**
///
/// **Hostile axes:** `shrunk`.
///
/// Scene 11, and it is the axis whose defect **no golden-cell gate can see**: a closed body drawn into
/// an `h = 0` rectangle instead of skipped declares **408 more hit entries and 408 more ring entries**
/// on a surface that is 0 cells over 0 rows apart.
///
/// `size` is the sizing function: given the body's width it answers the body's height in rows, which
/// is the shape [`sizing::check`](vitui_runtime::sizing::check) sweeps a component against. `body`
/// is called **only when the section has height**, which is the whole of §8's *closed content is not
/// drawn*:
///
/// > A body is handed a rectangle and draws inside it.
///
/// A closure that is not called costs nothing at all, and the alternative — calling it with an
/// `h = 0` rectangle and letting the clip reject every write — is invisible to every instrument that
/// reads a surface: **408 hit entries and 408 ring entries nobody can reach**, on two surfaces that
/// are 0 cells over 0 rows apart, because `Ctx::interact` appends to the hit index and to the ring
/// before either looks at the rectangle. See [`defective::zero_rect`] and [`crate::accordion`].
///
/// ```
/// use vitui_components::disclose::{Collapse, collapsible};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::{Rect, Role};
///
/// let mut driver = Driver::headless(40, 12).expect("a sink attaches");
///
/// let mut st = Collapse::open_at(3);
/// driver.frame(|cx| {
///     let d = collapsible(
///         cx,
///         Rect::new(0, 0, 40, 12),
///         &mut st,
///         "General",
///         &mut |_w| 3,
///         &mut |cx| {
///             let body = cx.theme().paint(Role::Body);
///             cx.text(0, 0, "three rows of content", body);
///         },
///     );
///     // A header row and three rows of body, and the body is named rather than written.
///     assert_eq!(d.used, 4);
///     assert_eq!((d.body.w, d.body.h), (40, 3));
/// });
///
/// // Closed, the body is not called at all: nothing inside it declares anything.
/// let mut st = Collapse::shut();
/// let mut calls = 0u32;
/// driver.frame(|cx| {
///     let d = collapsible(
///         cx,
///         Rect::new(0, 0, 40, 12),
///         &mut st,
///         "General",
///         &mut |_w| 3,
///         &mut |_cx| calls += 1,
///     );
///     assert_eq!(d.used, 1);
///     assert!(d.body.is_empty());
/// });
/// assert_eq!(calls, 0, "closed content is not drawn");
/// assert_eq!(driver.inspect().stop_count(), 1, "the header, and nothing else");
/// ```
#[track_caller]
pub fn collapsible(
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut Collapse,
    title: &str,
    size: &mut dyn FnMut(u16) -> u16,
    body: &mut dyn FnMut(&mut Ctx<'_, '_>),
) -> Disclosure {
    collapsible_with(cx, rect, st, title, &DiscloseOpts::default(), size, body)
}

/// [`collapsible`], with the options spelled out.
#[track_caller]
pub fn collapsible_with(
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut Collapse,
    title: &str,
    opts: &DiscloseOpts,
    size: &mut dyn FnMut(u16) -> u16,
    body: &mut dyn FnMut(&mut Ctx<'_, '_>),
) -> Disclosure {
    collapsible_into(&mut Direct, cx, rect, st, title, opts, size, |_ink, cx| {
        body(cx);
    })
}

/// **[`collapsible`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`collapsible`] is this with [`Direct`]. See [`crate::ink`] for why
/// the seam exists rather than a second implementation written against a `Tally`.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "the component's own six — a context, a rectangle, the state, the title, the options \
              and the sizing function — plus the body and the `Ink` seam's writer. The sizing \
              function is a closure over the caller's data and cannot live on the options, and \
              folding the first six into a parameter struct would invent a type that exists only \
              to satisfy a lint"
)]
pub fn collapsible_into<I, S, B>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut Collapse,
    title: &str,
    opts: &DiscloseOpts,
    size: S,
    body: B,
) -> Disclosure
where
    I: Ink,
    S: FnMut(u16) -> u16,
    B: FnMut(&mut I, &mut Ctx<'_, '_>),
{
    draw_with(ink, cx, rect, st, title, opts, size, body, Shape::default())
}

/// **The one way `collapsible` can be false that is not on [`DiscloseOpts`]**, as one value.
///
/// One field and not a boolean in a signature, so a reviewer's diff between the shipped build and
/// the refused one is a single line — [`crate::collect`]'s `TreeShape` arrangement one family over.
/// The other two — [`Height::Watermark`] and [`Focus::Drop`] — are *on* the options, because §8
/// states each of them as a spelling a caller may ask for and be priced for rather than as a defect
/// only a gate may build.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Shape {
    /// Whether a closed body is called with an `h = 0` rectangle instead of skipped.
    closed: Closed,
}

/// What a closed section does with its body.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Closed {
    /// **The rule.** The closure is not called.
    #[default]
    Skip,
    /// **The defect.** Called with a zero-height rectangle, every write rejected by the clip, and
    /// every declaration kept.
    ZeroRect,
}

/// **`#[track_caller]` all the way down.** `Ctx::id` mints from `Location::caller()`, so an
/// attribute that stops one frame short of the call makes every collapsible in an application one
/// collapsible — [`crate::collect`]'s own `draw_with` found that with **9 merges on the correct
/// arm**.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`collapsible_into`'s eight plus the shape that separates the shipped build from the \
              refused one. Splitting it would put the defect in a second function where a \
              reviewer's diff could not be one line"
)]
fn draw_with<I, S, B>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut Collapse,
    title: &str,
    opts: &DiscloseOpts,
    mut size: S,
    mut body: B,
    shape: Shape,
) -> Disclosure
where
    I: Ink,
    S: FnMut(u16) -> u16,
    B: FnMut(&mut I, &mut Ctx<'_, '_>),
{
    // **The id, taken outside every closure** (ADR 0027).
    let id = cx.id();
    let now = cx.now();
    let (head, below) = split_head(rect);

    // **The header's region, declared before the body is drawn.** The hit index is in draw order and
    // the press award is a *reverse* scan of it, so a container declared after the widgets it
    // contains is in front of them — `structure::panel_into`'s finding, and the same order here.
    //
    // **An empty header declares nothing at all**, which is this module's own rule applied to itself:
    // `Ctx::interact` appends to the hit index and to the ring *before* either looks at the
    // rectangle, so a section handed a zero-height rectangle — a stack that has run out of room — is
    // the `Closed::ZeroRect` defect one row up, at an amplitude of one. `Response::inert` is what
    // `crate::text::text_into` returns for the same reason.
    let mut resp = if head.is_empty() {
        Response::inert(id, head)
    } else {
        cx.interact(id, head, opts.interest)
    };
    let key = header_key(cx, id);

    let mut gesture = None;
    if resp.clicked || key {
        let want = !st.open();
        // **The focus goes to the header on the gesture**, which is what a focusable widget does on
        // a click anyway — and it is what keeps R08's vanish rule from being reached at all when the
        // body it was inside stops drawing. See [`Focus`].
        if opts.focus == Focus::Header {
            cx.focus(id);
        }
        if opts.collapses.applies() {
            let to = if want {
                open_height(
                    ink,
                    cx,
                    below,
                    st.height(),
                    opts.height,
                    &mut size,
                    &mut body,
                )
            } else {
                0
            };
            // **A fold steps.** `Collapses::OnRequest` never animates, and `Both` animates only the
            // region half — the index half is the caller's splice, which is an edit and not a tween.
            let dur = if opts.collapses.animates() {
                opts.dur
            } else {
                Duration::ZERO
            };
            st.set(now, want, to, dur);
        }
        gesture = Some(Toggle {
            open: want,
            applied: opts.collapses.applies(),
        });
        // §1's rule 4 and the runtime's own convention: *a component sets it to say its own value
        // changed; nothing here can know that*.
        resp.changed = true;
    }

    // **The tween, moved once a frame, and the frame it still wants.** A component that asks for a
    // wake without asking whether the tween is done is a spin ([`Tween::done`]).
    let wake = st.advance(now);
    // **An open section tracks its sizing function.** The answer moves with the width, so a section
    // that kept last frame's height would draw a stale rectangle for one frame after every resize —
    // and under [`Height::Watermark`] this is the dry run that runs every frame and is what the
    // spelling costs.
    if st.open() && !st.animating() {
        let to = open_height(
            ink,
            cx,
            below,
            st.height(),
            opts.height,
            &mut size,
            &mut body,
        );
        if to != st.height() {
            st.set(now, true, to, Duration::ZERO);
        }
    }
    if let Some(at) = wake {
        cx.deadline_for(id, at);
    }

    // **The header is a partition of its row**: one cell of chevron, the title in the rest. The face
    // is resolved *before* a cell is written and never restyled after (ADR 0026).
    let theme = cx.theme();
    let chevron = theme.glyph(if st.open() {
        Glyph::ArrowDown
    } else {
        Glyph::ArrowRight
    });
    let paint = face_paint(
        theme,
        Face {
            active: st.open(),
            hovered: resp.hovered,
            cursor: resp.focused,
            ..Face::REST
        },
    );
    if !head.is_empty() {
        let _ = ink.text(cx, head.x, head.y, chevron, paint);
        let label = Rect::new(head.x + 1, head.y, head.w.saturating_sub(1), 1);
        let _ = fit_into(
            ink,
            cx,
            label,
            title,
            &FitOpts {
                justify: Justify::Start,
                role: opts.head,
                pad: opts.pad,
            },
        );
    }

    // **The body**, at the height the machine says and never past the rectangle it was handed.
    let h = st.height().min(below.h);
    let cells = Rect::new(below.x, below.y, below.w, h);
    if h > 0 {
        let mut child = cx.child(cells);
        body(ink, &mut child);
    } else if shape.closed == Closed::ZeroRect {
        // **The defect, and it is one branch.** See [`defective::zero_rect`].
        let mut child = cx.child(Rect::new(below.x, below.y, below.w, 0));
        body(ink, &mut child);
    }

    resp.rect = head;
    Disclosure {
        response: resp,
        body: cells,
        used: head.h + h,
        gesture,
        wake,
    }
}

/// The header row and everything under it. One place, so the two cannot drift into disagreeing about
/// where a section's body starts.
fn split_head(rect: Rect) -> (Rect, Rect) {
    if rect.h == 0 {
        let empty = Rect::new(rect.x, rect.y, rect.w, 0);
        return (empty, empty);
    }
    (
        Rect::new(rect.x, rect.y, rect.w, 1),
        Rect::new(rect.x, rect.y + 1, rect.w, rect.h - 1),
    )
}

/// **What height an open body should be drawn at**, by whichever of §8's two spellings the options
/// name.
///
/// [`Height::Sized`] asks the sizing function and is done. [`Height::Watermark`] runs the body
/// through [`Ctx::measured`](vitui_runtime::Ctx::measured) **inside the rectangle the last decision
/// produced** — which is the whole defect, stated as arithmetic: the probe is `current`, and on the
/// frame a section opens there is no answer at all, so it is the rectangle the section was handed.
///
/// # The dry run draws through the caller's ink, and that is deliberate
///
/// A discard surface is its own world, so nothing the dry run writes reaches the screen — but a
/// [`Tally`](crate::counters::Tally) counts the *call*, so the watermark arm's verb and write counts
/// carry the probe. That is exactly the price §8 puts at **+11.5% of the frame**, in this crate's own
/// currency rather than in a clock's, and it is what makes the two spellings separable by a counter
/// instead of by a stopwatch.
fn open_height<I, S, B>(
    ink: &mut I,
    cx: &Ctx<'_, '_>,
    below: Rect,
    current: u16,
    height: Height,
    size: &mut S,
    body: &mut B,
) -> u16
where
    I: Ink,
    S: FnMut(u16) -> u16,
    B: FnMut(&mut I, &mut Ctx<'_, '_>),
{
    match height {
        Height::Sized => size(below.w),
        Height::Watermark => {
            let probe = if current == 0 { below.h } else { current };
            cx.measured(below.w, probe, |inner| body(ink, inner))
                .extent
                .h
        }
    }
}

/// **`Enter` or `Space` on the focused header.**
///
/// One drain loop, and a `Ctx::decline` for everything else — which **ends this level's turn at the
/// queue**, so the loop breaks rather than continuing: `next_key` answers `None` afterwards however
/// many keys are left. [`crate::collect`]'s `Refusal` is the same fact one component over, and the
/// reason it had to become a parameter of one loop there rather than a second loop here.
fn header_key(cx: &mut Ctx<'_, '_>, id: Id) -> bool {
    let mut hit = false;
    while let Some(k) = cx.next_key(id) {
        if k.kind == Edge::Release || k.mods.ctrl() || k.mods.alt() {
            cx.decline(k);
            break;
        }
        match k.code {
            Code::Enter | Code::Char(' ') => hit = true,
            _ => {
                cx.decline(k);
                break;
            }
        }
    }
    hit
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the spellings that are refused, kept because a gate nobody has watched fail is not a gate
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **§8's `h = 0` spelling, kept where the register can point at it.**
///
/// `pub` for [`crate::frame::defective`]'s reason: the correct arm and this one are **one function
/// with one field between them**, so the diff a reviewer would have to catch is the diff the
/// register names.
pub mod defective {
    use super::{Closed, Collapse, Ctx, DiscloseOpts, Disclosure, Ink, Rect, Shape, draw_with};

    /// **A closed section whose body is called with an `h = 0` rectangle instead of skipped.**
    ///
    /// Every write is rejected by the clip, so the surface is **identical** — 0 cells over 0 rows at
    /// 300×80 — and every declaration the body makes is kept: **408 hit entries and 408 ring entries
    /// nobody can reach** on §8's own accordion. `Ctx::interact` appends to both indexes in one call,
    /// before either looks at the rectangle, which is why the subtraction is the same number twice.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature, so the two arms are one call apart"
    )]
    pub fn zero_rect<I, S, B>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        rect: Rect,
        st: &mut Collapse,
        title: &str,
        opts: &DiscloseOpts,
        size: S,
        body: B,
    ) -> Disclosure
    where
        I: Ink,
        S: FnMut(u16) -> u16,
        B: FnMut(&mut I, &mut Ctx<'_, '_>),
    {
        draw_with(
            ink,
            cx,
            rect,
            st,
            title,
            opts,
            size,
            body,
            Shape {
                closed: Closed::ZeroRect,
            },
        )
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// inplace: the map from a content row to a data row, and it needs no prefix sum
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **One open detail row, and the map between a content row and a data row.**
///
/// §8's *inplace needs no prefix sum*: with **one** row open the map is two comparisons and a
/// subtraction, and C05's `ytop` — the prefix sum a variable-height collection needs — is the answer
/// when *many* rows are tall rather than when one is. [`Inplace::many`] is the many case and it is
/// `O(log k)` over a sorted store with a running sum, still flat in the million.
///
/// ```
/// use vitui_components::disclose::Inplace;
///
/// // Data row 3 has a four-row detail region open under it.
/// let map = Inplace::one(3, 4);
/// assert_eq!(map.data_row(3), Some(3), "the row itself");
/// assert_eq!(map.data_row(5), None, "inside the detail region, which is no data row");
/// assert_eq!(map.data_row(8), Some(4), "and everything after it is shifted by the height");
/// assert_eq!(map.content_row(4), 8, "the round trip");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Inplace {
    /// `(data row, rows of detail)`, sorted by row, with the running sum of the heights before each.
    open: Vec<(usize, u16, u32)>,
}

impl Inplace {
    /// **One open detail row.** Two comparisons and a subtraction, and no allocation past the one
    /// entry.
    pub fn one(row: usize, rows: u16) -> Inplace {
        Inplace {
            open: vec![(row, rows, 0)],
        }
    }

    /// **Many open detail rows**, as a sorted store with a running sum.
    ///
    /// `rows` need not be sorted; it is sorted here, once, because the whole point of the store is
    /// that every lookup afterwards is a `partition_point`.
    pub fn many(rows: impl IntoIterator<Item = (usize, u16)>) -> Inplace {
        let mut open: Vec<(usize, u16, u32)> = rows.into_iter().map(|(r, h)| (r, h, 0)).collect();
        open.sort_unstable_by_key(|e| e.0);
        let mut before = 0u32;
        for e in &mut open {
            e.2 = before;
            before += u32::from(e.1);
        }
        Inplace { open }
    }

    /// How many rows are open.
    pub fn len(&self) -> usize {
        self.open.len()
    }

    /// Whether none is.
    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }

    /// **The content row a data row is drawn at.** The data row plus every open height above it.
    pub fn content_row(&self, data: usize) -> usize {
        let at = self.open.partition_point(|e| e.0 < data);
        let before = self.open.get(at).map_or_else(
            || self.open.last().map_or(0, |e| e.2 + u32::from(e.1)),
            |e| e.2,
        );
        data + before as usize
    }

    /// **The data row a content row belongs to, or `None` because it is inside a detail region.**
    ///
    /// The two comparisons and the subtraction. With one entry the `partition_point` is one step,
    /// which is why §8 can call it two comparisons without a store being a different answer.
    pub fn data_row(&self, content: usize) -> Option<usize> {
        // The last entry whose own content row is at or before `content`.
        let at = self.open.partition_point(|e| e.0 + e.2 as usize <= content);
        if at == 0 {
            return Some(content);
        }
        let (row, rows, before) = self.open[at - 1];
        let head = row + before as usize;
        if content <= head {
            return Some(content - before as usize);
        }
        if content <= head + usize::from(rows) {
            // Inside the detail region: no data row is drawn here.
            return None;
        }
        Some(content - before as usize - usize::from(rows))
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the ledger
// ═════════════════════════════════════════════════════════════════════════════════════════════════
//
// **Every number this module is gated or reported on has one home and it is here** — the runtime's
// ledger rule (`crates/vitui-runtime/src/ledger.rs`), inherited by `crate::scroll` and
// `crate::state`. §8's own screen is `crate::accordion`'s; what is priced here is the machine.

/// The fixture section's width. Sixty columns, wide enough that a title is not the subject.
pub const W: u16 = 60;
/// The fixture section's height: a one-row header and nineteen rows of room under it.
pub const H: u16 = 20;
/// **How many rows of content the fixture's body has. Four**, and the sizing function answers it.
pub const CONTENT_ROWS: u16 = 4;
/// **What [`Height::Watermark`] latches at over a body that fills what it is handed. Nineteen** —
/// [`H`] less the header row, which is the rectangle the section was handed on the frame it opened
/// and therefore the rectangle every measurement after it is taken inside.
///
/// §9's own sentence on the other axis, and the number is the whole of it: nineteen rows where
/// [`CONTENT_ROWS`] are right, permanently, on a screen that looks correct.
pub const WATERMARK_LATCHED_ROWS: u16 = H - 1;
/// §8's *cells wrong* for the sizing function. A figure of a prototype's body — see this module's
/// header.
pub const SPEC_SIZED_CELLS: u32 = 22;
/// §8's *cells wrong* for the drawn extent. A prototype's body.
pub const SPEC_WATERMARK_CELLS: u32 = 83;
/// §8's *rows wrong* for the drawn extent. A prototype's body.
pub const SPEC_WATERMARK_ROWS_WRONG: u16 = 8;
/// §8's price for the watermark, as a fraction of the frame. A **report**, and this crate reads it as
/// a verb count instead — see
/// `tests::the_watermark_pays_for_a_second_pass_over_the_body_and_the_sizing_function_pays_for_none`.
pub const SPEC_WATERMARK_OVERHEAD: f64 = 0.115;
/// §8's ring probes for the arm that lets the vanish rule fire. A figure of a prototype's ring; what
/// is gated is the zero beside it.
pub const SPEC_VANISH_PROBES: u64 = 405;
/// §8's frames to quiet for a two-hundred-millisecond collapse, at a cadence it does not state. See
/// `tests::open_is_never_ambiguous_over_every_frame_of_a_two_hundred_millisecond_collapse`.
pub const SPEC_FRAMES_TO_QUIET: u32 = 14;

/// **What a caller that does not write the tail leaves on the screen. Two hundred and forty cells.**
///
/// Exactly the rows a closed section vacated — `W × CONTENT_ROWS` — and **the header is not among
/// them**, which is what makes the figure a residue rather than a picture of two different frames:
/// both arms are compared with the focus seated on the header and no pointer anywhere, so the chevron
/// and the face are the same on each. Spec §9's seam sentence on the height axis — *the component
/// that owns the rectangle must write it* — and this is what it costs when nobody does.
pub const RESIDUE_CELLS: usize = W as usize * CONTENT_ROWS as usize;
/// How many rows the residue spans: exactly the [`CONTENT_ROWS`] the body had.
///
/// **The header is not among them**, and that is what makes the figure a residue rather than a
/// picture of two different frames: both arms are compared with the focus seated on the header and
/// no pointer anywhere, so the only cells that differ are the ones the vacated body left behind.
pub const RESIDUE_ROWS: usize = CONTENT_ROWS as usize;

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// there is no third state, and the gate is a scan
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// The words a stored transition state would be spelled with. §8: *no third state*, and the two
/// names it forbids.
///
/// **They are assembled rather than written**, and that is not decoration: the scan below reads
/// *this file*, so a constant that spelled either word out would be a declaration of it and the scan
/// would report the module it is defending. `crate::frame`'s own source scan reported **itself** on
/// its first run for exactly this reason, and the fix is the same shape one family over.
pub const THIRD_STATE_WORDS: [&str; 2] = [concat!("Collaps", "ing"), concat!("Expand", "ing")];

/// **Every line of this module that declares a third state. Empty, and it is a scan rather than an
/// assertion.**
///
/// A source scan and not a `compile_fail`, for [`crate::dense::subjects_declared`]'s reason: *the
/// item does not exist* has no expression, and a fence naming a variant that was never built passes
/// today and passes again the day somebody adds one under a different name. The predicate is
/// `crate::dense::declares` and it is shared rather than copied — one definition of *a line that
/// is not a comment*, which is the only thing that makes *fires in both directions* mean anything.
///
/// The two words appear in this file only inside comments, which is exactly the case the shared
/// predicate exists to discount — `crate::frame`'s own scan reported **itself** on its first run for
/// want of it.
pub fn third_state_declarations() -> Vec<&'static str> {
    let source = std::fs::read_to_string(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/disclose.rs"
    )))
    .unwrap_or_default();
    THIRD_STATE_WORDS
        .into_iter()
        .filter(|w| crate::dense::declares(&source, w))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use vitui_runtime::ctx::Driver;
    use vitui_runtime::{Button, Buttons, Density, Mods, Mouse, MouseKind};

    use crate::counters::Tally;
    use crate::input::button_into;
    use crate::runner::{Canvas, Pen};

    /// How many rows of content the body has. [`CONTENT_ROWS`], read from the ledger.
    const CONTENT: u16 = CONTENT_ROWS;
    /// How long the region's collapse takes. §8's own two hundred milliseconds.
    const DUR: Duration = Duration::from_millis(200);
    /// The cadence the collapse is driven at. **Sixty hertz**, which is the steady-state budget's own
    /// rate (`scripts/steady-report.sh`) and the only cadence on this map that is not this test's
    /// opinion.
    const STEP: Duration = Duration::from_micros(16_667);

    /// **A section, and whether its body fills the rectangle it is handed.**
    ///
    /// `fills` is the precondition [`Height::Watermark`] stands or falls on, and it is what a padding
    /// ring *is*: a body that writes a background over every row of `cx.area()` reaches the bottom of
    /// whatever it was given, so a measurement taken inside the rectangle the last decision produced
    /// can never come back down. §9 states the identical precondition for a hideable reserved bar —
    /// *a smaller viewport may not produce a smaller extent*.
    struct Fixture {
        driver: Driver,
        st: Collapse,
        opts: DiscloseOpts,
        fills: bool,
        /// How many times the body closure was called.
        calls: u32,
        /// The height the body was handed on the last frame.
        drawn: u16,
        /// The button inside the body, so a test can put the focus in there.
        inner: Option<Id>,
        /// An id to seat the focus on before the component draws.
        seat: Option<Id>,
        /// **How the rectangle's owner writes the tail** — the rows of `rect` below
        /// `Disclosure::used`, which the component names and does not write. See criterion 14.
        tail: Tail,
        /// The `used` the previous frame reported, for [`Tail::Stale`].
        last_used: u16,
        gesture: Option<Toggle>,
        wake: Option<Instant>,
    }

    impl Fixture {
        fn new(st: Collapse, opts: DiscloseOpts) -> Fixture {
            Fixture {
                driver: crate::runner::driver_at(W, H, Density::default()),
                st,
                opts,
                fills: false,
                calls: 0,
                drawn: 0,
                inner: None,
                seat: None,
                tail: Tail::Fresh,
                last_used: 0,
                gesture: None,
                wake: None,
            }
        }

        /// The same fixture over a body that fills what it is handed.
        fn filling(mut self) -> Fixture {
            self.fills = true;
            self
        }

        /// One frame, through the shipped component and an [`Ink`] of the caller's choosing.
        fn play<I: Ink>(&mut self, ink: &mut I) {
            let Fixture {
                driver,
                st,
                opts,
                fills,
                calls,
                drawn,
                inner,
                seat,
                tail,
                last_used,
                gesture,
                wake,
            } = self;
            let fills = *fills;
            let seat = *seat;
            let tail = *tail;
            driver.frame(|cx| {
                if let Some(id) = seat {
                    cx.focus(id);
                }
                let d = collapsible_into(
                    ink,
                    cx,
                    Rect::new(0, 0, W, H),
                    st,
                    "General",
                    opts,
                    |_w| CONTENT,
                    |ink, cx| {
                        *calls += 1;
                        *drawn = cx.area().h;
                        body(ink, cx, fills, inner);
                    },
                );
                *gesture = d.gesture;
                *wake = d.wake;
                // **The tail, and it is the caller's** — every row of the rectangle below the
                // section is inside it and no section owns it (§2). Spec §9 assigns that line by
                // name: *the component that owns the rectangle must write it.*
                let from = match tail {
                    Tail::Fresh => d.used,
                    Tail::Stale => (*last_used).max(d.used),
                };
                *last_used = d.used;
                let paint = cx.theme().paint(Role::Body);
                for y in i32::from(from)..i32::from(H) {
                    let _ = ink.run(cx, 0, y, " ", W, paint);
                }
            });
        }

        /// One frame through a [`Tally`], and the frame's own declarations beside it.
        fn tallied(&mut self) -> (Tally, usize, usize) {
            let mut tally = Tally::new();
            self.play(&mut tally);
            let frame = self.driver.inspect();
            (tally, frame.hits().len(), frame.stop_count())
        }

        /// Point at the header's row, so a press can land on it.
        fn point(&mut self) {
            self.driver.post_mouse(at(MouseKind::Move));
        }

        /// **The runtime's five-frame click cadence**, and every frame of it is the runtime's
        /// rather than this test's.
        ///
        /// The grab is awarded at `end` from the index that has just drawn, so `Response::pressed`
        /// is false on the frame that *processes* the `Down` and true on the one after; a `Down` and
        /// an `Up` in one batch open and close the grab before anything draws, so the release needs
        /// a frame of its own; and **`clicked` lands on the frame after that** — measured rather
        /// than assumed, because the first spelling of this helper stopped one frame short and read
        /// `gesture == None` on a click that had happened.
        fn click<I: Ink>(&mut self, ink: &mut I) {
            self.point();
            self.play(ink);
            self.driver.post_mouse(at(MouseKind::Down(Button::Left)));
            self.play(ink);
            self.play(ink);
            self.driver.post_mouse(at(MouseKind::Up(Button::Left)));
            self.play(ink);
            self.play(ink);
        }

        /// **Seat the focus on the header and close the section with `Enter`.**
        ///
        /// The gesture a surface pair needs: a click leaves the pointer *on* the header, so the two
        /// arms would differ by the header's own hover paint and the diff would be one cell of
        /// furniture rather than a residue. The keyboard moves nothing but the focus, and the
        /// comparison arm can be seated the same way.
        fn press_enter<I: Ink>(&mut self, ink: &mut I) {
            self.play(ink);
            let header = self.driver.inspect().hits()[0].id;
            self.seat = Some(header);
            self.play(ink);
            self.driver
                .post_key(crate::keys::press_with(Code::Enter, Mods::NONE));
            self.play(ink);
        }

        /// **One frame with the focus seated on the header and nothing else happening.**
        fn resting<I: Ink>(&mut self, ink: &mut I) {
            self.play(ink);
            let header = self.driver.inspect().hits()[0].id;
            self.seat = Some(header);
            self.play(ink);
        }

        /// How many slots the vanish rule touched on the last frame.
        fn probes(&self) -> u64 {
            self.driver.inspect().vanish_probes()
        }

        /// [`Fixture::click`], reporting **the worst** probe count over its frames.
        ///
        /// The worst rather than the last, because the vanish rule fires on exactly one frame — the
        /// one the focused widget stops drawing on — and a reading taken after it is zero on both
        /// arms.
        fn click_probing<I: Ink>(&mut self, ink: &mut I) -> u64 {
            self.point();
            self.play(ink);
            let mut worst = self.probes();
            self.driver.post_mouse(at(MouseKind::Down(Button::Left)));
            for step in 0..4 {
                if step == 2 {
                    self.driver.post_mouse(at(MouseKind::Up(Button::Left)));
                }
                self.play(ink);
                worst = worst.max(self.probes());
            }
            worst
        }
    }

    /// **Which `used` the tail is written from.**
    ///
    /// `Fresh` is this frame's, which is the rule. `Stale` is the previous frame's — which is what a
    /// caller writes when it computed the height before the collapse, is the realistic mistake rather
    /// than an omission nobody would write, and is the whole of the residue.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Tail {
        Fresh,
        Stale,
    }

    /// A pointer event over the header's first cell.
    fn at(kind: MouseKind) -> Mouse {
        Mouse {
            x: 4,
            y: 0,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        }
    }

    /// **The body: one focusable row and [`CONTENT`] rows of content**, and a padding ring when
    /// `fills` says so.
    ///
    /// **The button is drawn unconditionally and the content rows cull**, which is what makes the
    /// `h = 0` spelling measurable at all: a body that culled *everything* declares nothing at
    /// `h = 0` and the defect has no amplitude. `crate::accordion` is where the two culling
    /// spellings are separated properly, at §8's own scale.
    fn body<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, fills: bool, inner: &mut Option<Id>) {
        let area = cx.area();
        let paint = cx.theme().paint(Role::Body);
        if fills {
            for y in 0..i32::from(area.h) {
                let _ = ink.run(cx, 0, y, " ", area.w, paint);
            }
        }
        let resp = button_into(
            ink,
            cx,
            Rect::new(0, 0, area.w, 1),
            "apply",
            &crate::input::ButtonOpts::default(),
        );
        *inner = Some(resp.id);
        let rows = if fills { area.h } else { CONTENT.min(area.h) };
        for y in 1..i32::from(rows) {
            let _ = ink.run(cx, 0, y, "·", area.w, paint);
        }
    }

    // ── criterion 1: one machine, and the three-row table is its documented split ────────────────

    /// **§8's split is three rows, and each row is exactly one configuration of one machine.**
    ///
    /// Criterion 1. The table is a value rather than a paragraph for §17's reason — every obligation
    /// this map has stated as a sentence has been broken by someone who had read it — and what makes
    /// it a *split* rather than a list is the bijection: three rows, three arms, and no fourth on
    /// either side.
    #[test]
    fn the_split_is_three_rows_and_each_one_is_one_configuration_of_one_machine() {
        assert_eq!(SPLIT.len(), 3, "§8's own three");
        let mut seen: Vec<Collapses> = SPLIT.iter().map(|r| r.who.collapses()).collect();
        seen.sort_by_key(|c| c.word());
        let mut all = Collapses::ALL.to_vec();
        all.sort_by_key(|c| c.word());
        assert_eq!(
            seen, all,
            "three rows, three arms, and no fourth on either side"
        );

        // Every example on the freeze's own disclosure family, or the row is describing something
        // this library does not have.
        assert_eq!(
            MEMBERS,
            &["collapsible"],
            "one component expresses all of it"
        );
        assert_eq!(
            SPLIT.iter().map(|r| r.examples.len()).sum::<usize>(),
            5,
            "tree node, code folding, accordion, panel minimise, inplace edit"
        );

        // **A fold steps and a region animates**, as a property of the configuration rather than of
        // a caller's duration.
        assert!(
            !Collapses::OnRequest.animates(),
            "an animated fold is refused"
        );
        assert!(Collapses::OnTheFrame.animates());
        assert!(
            Collapses::Both.animates(),
            "the region half of the inplace row"
        );
        for c in Collapses::ALL {
            assert_eq!(
                c.animates(),
                c.applies(),
                "only what it applies can it animate"
            );
        }
    }

    // ── criterion 2: what it costs, and neither of §8's two numbers ──────────────────────────────

    /// **A `Collapse` is four bytes live and forty-eight with the slot, and §8's `5 / 72` is
    /// neither.**
    ///
    /// Criterion 2, asserted as a **disagreement** rather than bent to fit — components ticket 17's
    /// standing one family over, where §7's stated widths were the number that was wrong. The live
    /// half is `open: bool` and a `u16`; five is what a third `u16` would cost and there is no third,
    /// because `Tween::to` is the target while a tween runs and the height is it afterwards.
    ///
    /// **And the pair cannot both be a `size_of` of one type**, which is the part worth keeping:
    /// `Option<Tween<u16>>` is a field, so it costs its forty bytes empty. A record that is 5 B
    /// without a tween and 72 B with one is a record whose tween lives somewhere else, and nothing on
    /// this map has anywhere else to put one.
    #[test]
    fn a_collapse_is_four_bytes_live_and_the_slot_costs_forty_whether_or_not_it_holds_one() {
        assert_eq!(LIVE_BYTES, 4, "`open: bool` and a `u16`");
        assert_eq!(TWEEN_BYTES, 40, "two moments, two `u16`s and an easing");
        assert_eq!(
            PADDING_BYTES, 4,
            "`Tween` carries two `Instant`s, so the alignment is eight"
        );
        assert_eq!(COLLAPSE_BYTES, LIVE_BYTES + TWEEN_BYTES + PADDING_BYTES);
        assert_eq!(COLLAPSE_BYTES, 48);

        // §8's pair, and both halves of the disagreement.
        assert_ne!(LIVE_BYTES, SPEC_LIVE_BYTES);
        assert_ne!(COLLAPSE_BYTES, SPEC_TWEEN_BYTES);

        // **The slot is not free when it is empty**, which is why the two numbers cannot both be
        // sizes of this type.
        let shut = Collapse::shut();
        assert!(!shut.animating());
        assert_eq!(size_of_val(&shut), COLLAPSE_BYTES, "empty costs the same");

        // **Never per row of content.** A fold set is line numbers beside the caller's document; the
        // `Collapse` is per section. Four thousand one hundred and sixty-seven folds cost the fold
        // set's four bytes each and not this record's forty-four.
        assert_eq!(size_of::<u32>() * crate::accordion::FOLDS, 16_668);
        assert!(
            COLLAPSE_BYTES * crate::accordion::SECTIONS
                < size_of::<u32>() * crate::accordion::FOLDS,
            "twelve sections of state is cheaper than the fold set it is not"
        );
    }

    // ── criterion 3: there is no transition state ────────────────────────────────────────────────

    /// **No stored third state, and the scan is watched finding one.**
    ///
    /// Criterion 3's first half. A source scan rather than an assertion, because *the item does not
    /// exist* has no expression — and both directions, because a scanner that has quietly stopped
    /// scanning returns the same empty list a clean file does.
    #[test]
    fn there_is_no_stored_third_state_and_the_scan_finds_one_when_there_is() {
        assert_eq!(
            third_state_declarations(),
            Vec::<&str>::new(),
            "a stored third state is a machine that can be found halfway between two states with \
             no clock running: {THIRD_STATE_WORDS:?}"
        );
        for word in THIRD_STATE_WORDS {
            assert!(
                crate::dense::declares(&format!("    {word},"), word),
                "the scan finds a declaration when there is one"
            );
            assert!(
                !crate::dense::declares(&format!("//! no `{word}` and no other"), word),
                "a mention in a comment is not a declaration — this file is full of them"
            );
        }
    }

    /// **`open` is never ambiguous, over every frame of a two-hundred-millisecond collapse.**
    ///
    /// Criterion 3's second half and criterion 4's shape. `set` writes `open` at the instant the
    /// gesture lands, so there is no frame on which the machine is *between* two states — and the
    /// only thing that moves is the height, monotonically, ending exactly at zero.
    ///
    /// **The frame count is arithmetic and not a measurement**: sixty hertz over two hundred
    /// milliseconds is `⌈200 / 16.667⌉`, which is **12**, and the twelfth frame is the one that
    /// lands. §8 states [`SPEC_FRAMES_TO_QUIET`] at a cadence it does not state; the relation — *the
    /// tween is quiet exactly when `now` reaches `start + dur`, and the screen sleeps after* — is
    /// the gate, because a frame count is a cadence wearing a count's clothes.
    #[test]
    fn open_is_never_ambiguous_over_every_frame_of_a_two_hundred_millisecond_collapse() {
        let now = Instant::now();
        let mut c = Collapse::open_at(10);
        c.set(now, false, 0, DUR);
        assert!(!c.open(), "flipped at the instant the gesture landed");
        assert_eq!(c.height(), 10, "and only the height has yet to move");

        let mut heights = vec![c.height()];
        let mut frames = 0u32;
        let mut at = now;
        loop {
            at += STEP;
            let wake = c.advance(at);
            frames += 1;
            heights.push(c.height());
            assert!(!c.open(), "never ambiguous, at frame {frames}");
            if wake.is_none() {
                break;
            }
            assert!(frames < 1_000, "a tween that never finishes is a spin");
        }

        assert_eq!(
            frames, 12,
            "⌈200 / 16.667⌉, and the twelfth is the one that lands"
        );
        assert_eq!(*heights.last().expect("a frame ran"), 0);
        assert!(!c.animating(), "and the screen may sleep");
        assert_eq!(c.advance(at + STEP), None, "asking again asks for nothing");
        assert!(
            heights.windows(2).all(|w| w[1] <= w[0]),
            "monotone, and it never overshoots: {heights:?}"
        );
        assert_eq!(heights.iter().copied().max(), Some(10));

        // **§8's own fourteen is a cadence and not a count**, and the relation under it is what is
        // gated: the tween is quiet exactly at `start + dur`, whatever the cadence.
        let mut c = Collapse::open_at(10);
        c.set(now, false, 0, DUR);
        assert_eq!(
            c.advance(now + DUR - Duration::from_millis(1)),
            Some(now + DUR - Duration::from_millis(1))
        );
        assert_eq!(c.advance(now + DUR), None);
        assert_eq!(c.height(), 0);
    }

    // ── criterion 5 and 6: a fold steps, a region animates, and both on the frame it lands ───────

    /// **A region collapses on the frame the gesture lands; an index collapse is a request the caller
    /// drains.**
    ///
    /// Criteria 5 and 6, and the difference between them is the one field [`Collapses`] is. The
    /// region arm's state is already collapsed by the time the frame ends, and it never animates
    /// under a zero duration — which is what a fold does and what a theme asking for no motion does.
    /// The index arm's state is **untouched**, and the gesture is the whole of what happened.
    #[test]
    fn a_region_collapses_on_the_frame_and_an_index_collapse_is_a_request_the_caller_drains() {
        // The region: one click, and the section is shut before the frame ends.
        let mut f = Fixture::new(Collapse::open_at(CONTENT), DiscloseOpts::default());
        let mut ink = Direct;
        f.click(&mut ink);
        assert_eq!(
            f.gesture,
            Some(Toggle {
                open: false,
                applied: true
            })
        );
        assert!(!f.st.open(), "on the frame, not on the next one");
        assert_eq!(f.st.height(), 0);
        assert!(!f.st.animating(), "a zero duration steps");

        // The index: the state is untouched and the request is the answer.
        let mut f = Fixture::new(
            Collapse::open_at(CONTENT),
            DiscloseOpts {
                collapses: Collapses::OnRequest,
                dur: DUR,
                ..DiscloseOpts::default()
            },
        );
        f.click(&mut ink);
        assert_eq!(
            f.gesture,
            Some(Toggle {
                open: false,
                applied: false
            })
        );
        assert!(f.st.open(), "a component may not perform the edit");
        assert!(
            !f.st.animating(),
            "an animated fold is refused: the removed rows would have to still be in the index \
             while they shrink"
        );
        assert_eq!(f.wake, None, "and a fold asks for no further frame");
    }

    /// **A region with a duration animates, and it asks for its own frames.**
    ///
    /// Criterion 4's second half: the tween asks through `Ctx::deadline_for`, so a component that
    /// asks for a wake without asking whether the tween is done would be a spin — and the frame
    /// after it lands asks for nothing.
    #[test]
    fn a_region_with_a_duration_animates_and_asks_for_its_own_frames_until_it_lands() {
        let mut f = Fixture::new(
            Collapse::open_at(CONTENT),
            DiscloseOpts {
                dur: DUR,
                ..DiscloseOpts::default()
            },
        );
        let mut ink = Direct;
        f.click(&mut ink);
        assert!(f.st.animating(), "the height is moving");
        assert!(f.wake.is_some(), "and it asked for the frame that moves it");
        assert!(!f.st.open());

        // Drive it to quiet, and the last frame asks for nothing.
        let before = f.calls;
        let mut frames = 0u32;
        let mut drawn = 0u32;
        while f.st.animating() {
            f.driver.advance(STEP);
            f.play(&mut ink);
            frames += 1;
            // **The height the frame drew at**, which is the height *after* the tween moved — the
            // component advances before it draws, which is what makes `open` unambiguous. So *the
            // body was called* is a prediction over this and not a recording: the rounded height
            // reaches zero before the tween does, and those last frames draw no body at all.
            if f.st.height() > 0 {
                drawn += 1;
            }
            assert!(frames < 1_000, "a tween that never finishes is a spin");
        }
        assert_eq!(f.st.height(), 0);
        assert_eq!(f.wake, None, "the screen may sleep");
        // **A collapsing body is still drawn, and that is the point of the mid-transition case**:
        // §8's *halfway down a collapse, a body that does not cull declares 273 ring entries against
        // 247*. What is never called is a body whose height has reached zero, and every one of the
        // shrinking frames called it exactly once.
        assert_eq!(
            f.calls - before,
            drawn,
            "one call a frame that had a body, and {} of the {frames} moving frames had none — the \
             rounded height reaches zero before the tween does",
            frames - drawn
        );
        assert!(drawn < frames, "or the prediction is the recording");
        let moving = f.calls;
        f.play(&mut ink);
        assert_eq!(f.calls, moving, "settled shut, the closure is not called");
    }

    // ── criterion 7: the height is an argument ───────────────────────────────────────────────────

    /// **The sizing function is right on every frame; the watermark latches at the first rectangle it
    /// ever saw.**
    ///
    /// Criterion 7, and §8's three figures for it — *22 cells, 0 rows wrong, 2 frames* against *83,
    /// 8, 3* — are a prototype's body and are not reproduced. What is reproduced is stronger and it
    /// is §9's own sentence one axis over: **a measured extent is taken inside the rectangle the
    /// decision produced.** Over a body that fills what it is handed the watermark never comes down —
    /// [`WATERMARK_LATCHED_ROWS`] where [`CONTENT`] are right, permanently, on a screen that looks
    /// correct.
    ///
    /// Over a body that draws only its content the two arms are **indistinguishable**, which is what
    /// makes the rule unconditional rather than a preference: *fills its rectangle* is not a property
    /// any component can guarantee of its body, which is §9's word for word.
    #[test]
    fn the_sizing_function_is_right_on_every_frame_and_a_measured_extent_latches() {
        let mut ink = Direct;

        // The rule, over both bodies: right on the frame it opens and every frame after.
        for fills in [false, true] {
            let mut f = Fixture::new(Collapse::shut(), DiscloseOpts::default());
            if fills {
                f = f.filling();
            }
            f.click(&mut ink);
            assert!(f.st.open());
            assert_eq!(f.st.height(), CONTENT, "0 rows wrong, on the opening frame");
            f.play(&mut ink);
            assert_eq!(f.st.height(), CONTENT, "and it stays right");
        }

        // The negative case over a body that draws only its content: indistinguishable.
        let mut f = Fixture::new(
            Collapse::shut(),
            DiscloseOpts {
                height: Height::Watermark,
                ..DiscloseOpts::default()
            },
        );
        f.click(&mut ink);
        assert_eq!(
            f.st.height(),
            CONTENT,
            "a body that draws only its content answers the same either way"
        );

        // And over a body that fills what it is handed, it latches and never comes down.
        let mut f = Fixture::new(
            Collapse::shut(),
            DiscloseOpts {
                height: Height::Watermark,
                ..DiscloseOpts::default()
            },
        )
        .filling();
        f.click(&mut ink);
        assert_eq!(
            f.st.height(),
            WATERMARK_LATCHED_ROWS,
            "the probe is the rectangle the section was handed, because there is no answer yet"
        );
        for _ in 0..8 {
            f.play(&mut ink);
            assert_eq!(
                f.st.height(),
                WATERMARK_LATCHED_ROWS,
                "and the next measurement is taken inside the rectangle this one produced"
            );
        }
        // The two constants are compile-time, so the relation between them belongs in a `const`
        // assertion rather than in a runtime one clippy is right to call out.
        const _: () = assert!(
            WATERMARK_LATCHED_ROWS > CONTENT_ROWS,
            "the latched height must exceed the content, or the arms are indistinguishable"
        );
    }

    /// **What the watermark costs, in this crate's own currency rather than a clock's.**
    ///
    /// §8 prices it at **+11.5% of the frame** and a timing is a report (§21, R15). The dry run
    /// happens through the caller's ink, so a `Tally` carries it: the watermark arm makes the body's
    /// verbs **twice** and the sizing arm makes them once. That is a count, it is the same fact, and
    /// it is why the spelling is off by default.
    #[test]
    fn the_watermark_pays_for_a_second_pass_over_the_body_and_the_sizing_function_pays_for_none() {
        let sized = {
            let mut f = Fixture::new(Collapse::open_at(CONTENT), DiscloseOpts::default());
            let (tally, _, _) = f.tallied();
            (tally.verbs(), f.calls)
        };
        let watermarked = {
            let mut f = Fixture::new(
                Collapse::open_at(CONTENT),
                DiscloseOpts {
                    height: Height::Watermark,
                    ..DiscloseOpts::default()
                },
            );
            let (tally, _, _) = f.tallied();
            (tally.verbs(), f.calls)
        };
        assert_eq!(sized.1, 1, "the body is drawn once");
        assert_eq!(watermarked.1, 2, "drawn once and measured once");
        assert!(
            watermarked.0 > sized.0,
            "the dry run is a cost a counter can read: {} against {}",
            watermarked.0,
            sized.0
        );
    }

    // ── criterion 8: closed content is not drawn ─────────────────────────────────────────────────

    /// **A closed body is not called, and the `h = 0` spelling keeps every declaration it would have
    /// made.**
    ///
    /// Criterion 8 at one section; [`crate::accordion`] is the same pair at §8's own scale, where the
    /// excess is 408 on the hit index and on the ring at once. Here the amplitude is one button and
    /// the point is that the **cells** are equal and the **declarations** are not — which is §8's *no
    /// golden-cell gate can see it* at the smallest amplitude it has.
    #[test]
    fn a_closed_body_is_not_called_and_the_zero_rect_spelling_declares_what_it_would_have() {
        let correct = shut_section(Closed::Skip);
        let defective = shut_section(Closed::ZeroRect);

        assert_eq!(
            correct.calls, 0,
            "a closure that is not called costs nothing"
        );
        assert_eq!(correct.entries, 1, "the header, and nothing else");
        assert_eq!(correct.stops, 1);

        assert_eq!(
            defective.calls, 1,
            "called, and every write rejected by the clip"
        );
        assert_eq!(defective.entries, 2, "one button nobody can reach");
        assert_eq!(defective.stops, 2);
        assert_eq!(
            defective.entries - correct.entries,
            defective.stops - correct.stops,
            "`Ctx::interact` appends to the hit index and to the ring in one call, before either \
             looks at the rectangle, so the two excesses are one subtraction"
        );

        // **And no counter that reads a cell moves.** The writes are equal, the distinct cells are
        // equal, and the one figure that does move is what the clip discarded.
        assert_eq!(defective.writes, correct.writes, "the write count is blind");
        assert_eq!(defective.distinct, correct.distinct, "and so is the union");
        assert_eq!(correct.asked, correct.writes, "nothing was clipped");
        assert!(
            defective.asked > defective.writes,
            "the body asked for {} and {} landed",
            defective.asked,
            defective.writes
        );
    }

    /// One shut section drawn through a [`Tally`], as counts. Both arms of the pair above, one
    /// argument apart.
    struct Shut {
        calls: u32,
        entries: usize,
        stops: usize,
        writes: u64,
        distinct: u64,
        asked: u64,
    }

    #[track_caller]
    fn shut_section(closed: Closed) -> Shut {
        let mut driver = crate::runner::driver_at(W, H, Density::default());
        let mut st = Collapse::shut();
        let opts = DiscloseOpts::default();
        let mut calls = 0u32;
        let mut tally = Tally::new();
        let mut inner = None;
        driver.frame(|cx| {
            let draw = |ink: &mut Tally, cx: &mut Ctx<'_, '_>| {
                calls += 1;
                body(ink, cx, false, &mut inner);
            };
            match closed {
                Closed::Skip => {
                    collapsible_into(
                        &mut tally,
                        cx,
                        Rect::new(0, 0, W, H),
                        &mut st,
                        "General",
                        &opts,
                        |_w| CONTENT,
                        draw,
                    );
                }
                Closed::ZeroRect => {
                    defective::zero_rect(
                        &mut tally,
                        cx,
                        Rect::new(0, 0, W, H),
                        &mut st,
                        "General",
                        &opts,
                        |_w| CONTENT,
                        draw,
                    );
                }
            }
        });
        let frame = driver.inspect();
        Shut {
            calls,
            entries: frame.hits().len(),
            stops: frame.stop_count(),
            writes: tally.writes(),
            distinct: tally.distinct(),
            asked: tally.asked(),
        }
    }

    // ── criterion 9: the focus, and the vanish rule that must not be reached ─────────────────────

    /// **No gesture that closes a section from its own header can reach the vanish rule at all**, and
    /// that is the finding rather than criterion 9 restated.
    ///
    /// §8 states the pair as *0 ring probes against 405* and says out loud why the left half is free:
    /// *which is what a focusable widget does on a click anyway.* Following that sentence to the end,
    /// **all three self-close gestures leave the focus off the body before the vanish rule looks**,
    /// each for a different reason, and every one of them is measured here:
    ///
    /// | the gesture | why the vanish rule is not reached |
    /// |---|---|
    /// | a click on a focusable header | the press award focuses it. Architecture issue 25's *the click is an accidental repair* |
    /// | a click on a header that is not a tab stop | the press award **defocuses**: *a press landed on nothing interested and the user meant to defocus, so there is no id left to have vanished* |
    /// | `Enter` | the header already holds the focus, so nothing inside the body did |
    ///
    /// So the zero is the gate and the 405 is **not reachable from a header gesture**. The arm that
    /// pays it is a section closed by something that is *not* one — exclusive mode, or a collapse-all
    /// — and [`Focus::Header`] is no answer there either, because there was no header to have been
    /// acted on. `crate::accordion` runs that arm at §8's own scale, which is where its number
    /// belongs.
    ///
    /// # What [`Focus::Header`] *does* buy, and it is measured rather than argued
    ///
    /// On a header that is not a tab stop the runtime's answer is `None`: the click **loses the
    /// keyboard entirely**, which is architecture issue 25's own finding one component over. The rule
    /// is what puts it somewhere, and that is the difference the two arms actually have.
    #[test]
    fn no_self_close_gesture_reaches_the_vanish_rule_and_the_rule_is_what_keeps_the_keyboard() {
        // Every self-close gesture, on both arms, pays zero.
        for focus in [Focus::Header, Focus::Drop] {
            for interest in [Interest::CLICK, Interest::CLICK.with(Interest::FOCUS)] {
                let (probes, _) = closed_by_a_click(focus, interest);
                assert_eq!(
                    probes,
                    0,
                    "{} on {interest:?}: the press award got there first",
                    focus.word()
                );
            }
        }

        // **And the difference the two arms do have.** On a header that is not a tab stop the click
        // defocuses, so the rule is the only thing that leaves the keyboard anywhere at all.
        let (_, kept) = closed_by_a_click(Focus::Header, Interest::CLICK);
        let (_, dropped) = closed_by_a_click(Focus::Drop, Interest::CLICK);
        assert!(kept.is_some(), "the header took it");
        assert_eq!(
            dropped, None,
            "and without the rule a click on a header that is not a tab stop loses the keyboard"
        );

        // On a focusable header the runtime does it, so the two arms are one program — which is why
        // the rule cannot be watched failing on the configuration a reader would reach for.
        let both = Interest::CLICK.with(Interest::FOCUS);
        assert_eq!(
            closed_by_a_click(Focus::Header, both).1,
            closed_by_a_click(Focus::Drop, both).1,
            "the click is an accidental repair"
        );
    }

    /// **Put the focus inside the body, close the section by clicking its header, and report `(the
    /// worst probe count over the click's frames, where the focus ended)`.**
    ///
    /// The worst probe count rather than the last, because the vanish rule fires on exactly one frame
    /// — the one the focused widget stops drawing on — and a frame in which nothing vanished pays
    /// zero.
    fn closed_by_a_click(focus: Focus, interest: Interest) -> (u64, Option<Id>) {
        let mut ink = Direct;
        let mut f = Fixture::new(
            Collapse::open_at(CONTENT),
            DiscloseOpts {
                focus,
                interest,
                ..DiscloseOpts::default()
            },
        );
        // One frame to learn the button's id, one to seat the focus on it.
        f.play(&mut ink);
        f.seat = f.inner;
        assert!(f.seat.is_some(), "the body drew a focusable");
        f.play(&mut ink);
        f.seat = None;
        assert_eq!(f.probes(), 0, "nothing has vanished yet");

        let probes = f.click_probing(&mut ink);
        assert!(!f.st.open(), "the section is shut");
        (probes, f.driver.inspect().focused())
    }

    // ── criterion 13: the surface after a collapse equals a freshly built one ────────────────────

    /// **The surface after a collapse equals a freshly built one, cell for cell.**
    ///
    /// Criterion 13. The instrument is a [`Pen`] carried **across** frames — `Pen::over`, which is
    /// what [`crate::area`] uses for re-damage — because residue is a relation between two frames and
    /// a pen built per frame has nothing to relate to. A collapse that left the vacated rows behind
    /// would be a screen that looks correct and a diff that is not clean.
    ///
    /// **This is not register row 21.** That row asks for the equality between two *composited*
    /// surfaces and ADR 0023 hands over no cell; this is the crate's own model of what it drew, which
    /// is row 41's standing to row 2's. It sees residue in the cells **this crate wrote** and nothing
    /// else, and that is stated rather than implied.
    #[test]
    fn the_surface_after_a_collapse_equals_a_freshly_built_one() {
        let after = collapsed_canvas(Tail::Fresh);
        let fresh = resting_canvas(Collapse::shut());
        after
            .diff(&fresh)
            .assert_clean("after a collapse against a fresh shut section");

        // And the other direction, so the equality is not between two blank screens: an open section
        // is **not** equal to a shut one.
        let open = resting_canvas(Collapse::open_at(CONTENT));
        assert!(
            !after.diff(&open).clean(),
            "a collapse that changed nothing would pass the equality above"
        );
    }

    /// **The equality holds exactly when the rectangle's owner writes the tail, and
    /// [`RESIDUE_CELLS`] over [`RESIDUE_ROWS`] is what it costs when it does not.**
    ///
    /// Criterion 14, and it is the *same* instrument as the equality above with one field flipped.
    /// Spec §9 assigns the line by name — *`[extent, offset + viewport)` is inside the rectangle, and
    /// the component that owns the rectangle must write it* — and this is that sentence on the height
    /// axis: the rows a closed section vacated are inside the caller's rectangle and no section owns
    /// them, so a caller writing its tail from the height it had **before** the collapse keeps the old
    /// body on the screen under a correct header. Nothing throws and the screen is plausible.
    ///
    /// **What is *not* decided here is whether a collapsible inside a scroll area may learn its
    /// content height one frame late under the extent shape.** That is §22's fog, it is not this
    /// ticket's, and this test does not answer it: the rectangle here is the section's own and no
    /// scroll area is involved.
    #[test]
    fn the_tail_is_the_rectangles_owners_and_the_residue_is_what_it_costs() {
        let diff = collapsed_canvas(Tail::Stale).diff(&resting_canvas(Collapse::shut()));
        assert!(!diff.clean(), "the vacated rows are still on the screen");
        assert_eq!(
            (diff.cells, diff.rows),
            (RESIDUE_CELLS, RESIDUE_ROWS),
            "the old body under a correct header"
        );
    }

    /// The surface a section collapsed by `Enter` leaves behind, with the tail written `tail`'s way.
    fn collapsed_canvas(tail: Tail) -> Canvas {
        let mut ink = Pen::new(W, H);
        let mut f = Fixture::new(Collapse::open_at(CONTENT), DiscloseOpts::default());
        f.tail = tail;
        f.press_enter(&mut ink);
        ink.end_frame();
        assert!(!f.st.open(), "it collapsed");
        ink.into_canvas()
    }

    /// The surface a section at `st` draws at rest, with the focus seated on its header — which is
    /// where the collapse leaves it, so the two arms differ by the collapse and by nothing else.
    fn resting_canvas(st: Collapse) -> Canvas {
        let mut ink = Pen::new(W, H);
        let mut f = Fixture::new(st, DiscloseOpts::default());
        f.resting(&mut ink);
        ink.end_frame();
        ink.into_canvas()
    }

    // ── the inplace map ──────────────────────────────────────────────────────────────────────────

    /// **The inplace map round-trips, and one open row needs no prefix sum.**
    ///
    /// Criterion 12's neighbour and register row 24. §8: *one open detail row maps a content row to a
    /// data row in two comparisons and a subtraction; C05's `ytop` is the answer when many rows are
    /// tall, not when one is.* The round trip is the equality — every data row maps to a content row
    /// and back — and the **many** arm is the same equality over a sorted store with a running sum,
    /// so the two spellings are checked against each other rather than each against itself.
    #[test]
    fn the_inplace_map_round_trips_and_one_open_row_needs_no_prefix_sum() {
        const LEN: usize = 1_000;

        // One open row, and the map is exact in both directions over the whole length.
        let one = Inplace::one(3, 4);
        for data in 0..LEN {
            let content = one.content_row(data);
            assert_eq!(
                one.data_row(content),
                Some(data),
                "the round trip at data row {data}"
            );
        }
        // The detail region maps to no data row at all, which is the whole reason the map is not a
        // shift.
        for content in 4..=7 {
            assert_eq!(one.data_row(content), None, "content row {content}");
        }
        assert_eq!(one.len(), 1);

        // **And the same equality over many open rows**, through a sorted store with a running sum.
        let many = Inplace::many((0..LEN).step_by(7).map(|r| (r, 3)));
        assert_eq!(many.len(), LEN.div_ceil(7));
        for data in 0..LEN {
            let content = many.content_row(data);
            assert_eq!(many.data_row(content), Some(data), "at data row {data}");
        }

        // **The two spellings agree on the one-open case**, so *needs no prefix sum* is a statement
        // about cost and not about a second answer.
        let store = Inplace::many([(3, 4)]);
        for data in 0..LEN {
            assert_eq!(one.content_row(data), store.content_row(data));
        }
        for content in 0..LEN {
            assert_eq!(one.data_row(content), store.data_row(content));
        }

        // **Flat in the million**: the store's lookup is a `partition_point` and the length is not an
        // input. The count is the assertion — a timing here would be a report about this machine.
        let huge = Inplace::many((0..1_000_000).step_by(1_000).map(|r| (r, 3)));
        assert_eq!(huge.len(), 1_000);
        assert_eq!(huge.data_row(huge.content_row(999_999)), Some(999_999));
        assert_eq!(huge.data_row(huge.content_row(0)), Some(0));
    }

    // ── the keyboard ─────────────────────────────────────────────────────────────────────────────

    /// **`Enter` and `Space` toggle the focused header, and everything else is declined.**
    ///
    /// The decline is what hands an unclaimed key to the level above, and it **ends this level's turn
    /// at the queue** — which is why the loop breaks rather than continuing. A gate that posted a key
    /// without seating the focus would measure nothing at all: `next_key` answers only the routing
    /// target, resolved from the *previous* frame's focus.
    #[test]
    fn enter_and_space_toggle_the_focused_header_and_every_other_key_is_declined() {
        use vitui_runtime::Mods;

        // **`Code::Enter` and not `Chord::key('\r')`**, which is `Code::Char('\r')` — a terminal
        // delivers the return key as its own code, and a gate that posted the carriage return would
        // be measuring a character this component deliberately does not own.
        for (key, toggles) in [
            (crate::keys::press_with(Code::Enter, Mods::NONE), true),
            (crate::keys::press_with(Code::Char(' '), Mods::NONE), true),
            (crate::keys::press_with(Code::Char('x'), Mods::NONE), false),
            (crate::keys::press_with(Code::Char(' '), Mods::CTRL), false),
        ] {
            let mut ink = Direct;
            let mut f = Fixture::new(Collapse::open_at(CONTENT), DiscloseOpts::default());
            // The focus is seated from the previous frame, so the key needs a frame ahead of it.
            f.play(&mut ink);
            f.seat = Some(f.driver.inspect().hits()[0].id);
            f.play(&mut ink);
            f.seat = None;
            f.driver.post_key(key);
            f.play(&mut ink);
            assert_eq!(
                f.gesture.is_some(),
                toggles,
                "{:?} on a focused header",
                key.code
            );
            assert_eq!(f.st.open(), !toggles);
        }
    }
}
