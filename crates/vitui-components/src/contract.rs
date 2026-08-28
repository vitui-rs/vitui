//! **O4's evidence as a value: every component's key bindings as declared data, the help rendered
//! from that data, and a sweep that asks the machine what it actually takes.**
//!
//! > O4 — a declared keyboard contract, rendered as help | equality (documented == R12-registered),
//! > plus §21's walkthrough and *a chord types nothing*. (spec §17)
//!
//! Three checks, and this module carries the first. The other two were already green when it was
//! written and are cited here rather than rebuilt: the walkthrough is register rows 34 and 88 —
//! *the walk repeats no id, and reaches every stop unless a trap is standing*, with
//! `Frame::trap_scopes` naming the modal exception rather than the gate being loosened — and *a
//! chord pressed into every focusable types nothing* is row 5, over [`crate::keys::TEXT_BEARING`]'s
//! seven and over a real [`crate::input::form`].
//!
//! # The equality is between a declaration and a machine, and it could not have been anything else
//!
//! [`crate::obligations::o4`] is an equality — *documented == registered* — and its own header
//! records what it cost to find out that two empty lists satisfy it. The same trap is one level
//! down: a `registered` list derived from [`CONTRACTS`] and a `documented` list derived from
//! [`CONTRACTS`] agree about everything, for ever, whatever either says about the component.
//!
//! So **[`Contract::live`] runs the shipped component**. It posts a trigger at a headless driver
//! with the widget's own id holding the focus, draws a frame, and answers the one question the
//! runtime can answer about any key without knowing what it means: **did anything take it**, read
//! off `Driver::unhandled` — *the keys this frame's batch carried that nobody took*. There is no
//! per-component knowledge in the observable, and no second reading of the drain loop to drift
//! away from the first.
//!
//! [`corpus`] is what makes it an equality rather than a subset. It carries every trigger a
//! component on this map could plausibly bind — sixteen codes at five modifier states, the
//! twenty-six letters at the three chord states, the printable-text class, and the three modified
//! clicks — so a component that takes a key nobody declared is caught by the same sweep that
//! catches a declaration nothing honours. **One direction is a help bar that lies and the other is
//! a feature nobody can find**, and neither is visible from the other side.
//!
//! # The help is rendered and there is no second list
//!
//! [`help`] builds the runtime's [`KeyMap`] out of a contract's own binds and prints it through
//! [`vitui_runtime::keys::write_help`], which takes a [`Binding`] and writes its first chord and
//! its help text. The chord's *spelling* is the runtime's [`write_chord`] in both directions —
//! what the sweep reports and what a help bar prints are the same function of the same value, so a
//! help bar cannot print `Ctrl+A` for a binding the sweep knows as something else.
//!
//! # A pointer gesture is a binding like any other
//!
//! Spec §5 recorded Ctrl-click and Shift-click as inexpressible; runtime 10 put `mods: Mods` on
//! `Response` and [`crate::collect::from_click`] reads it. They are declared here with the keys,
//! and the sweep asks the same question of them in the only form a click has: **a modified click
//! and a plain one leave different pictures**. A component that ignores the modifier byte leaves
//! the same picture and is not registered, which is the answer.
//!
//! # What is absent, and the fact that makes it absent
//!
//! [`ABSENT`] is one row. **Word motion is not bound anywhere and must not appear in any help**,
//! because UAX #29's word-boundary half is not exported: the engine ships `graphemes()` and
//! `width_of()` and no word iterator. A binding that is missing and a binding that is *not yet
//! done* are the same thing to a reader, so the reason is carried beside the row and a gate reads
//! both — the help may not mention it, and the file that would have to change first is named.
//!
//! # What this module does not decide
//!
//! **Whether a help bar may show a binding as unavailable.** It is inherited from the runtime spec
//! as a components question and is still open; nothing here answers it, and [`Bind`] carries no
//! enabled flag for that reason.

use crate::ink::Ink;
use crate::runner::{Canvas, Pen};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Binding, Chord, Code, KeyMap, write_chord, write_help};
use vitui_runtime::{Ctx, Id, Mods};

// ── the vocabulary ───────────────────────────────────────────────────────────────────────────────

/// **What a trigger arrives as**, and there are three because the machine answers three different
/// questions about them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trigger {
    /// One key chord. Answered by *did anything take it*.
    Key(Chord),
    /// **A printable character with no chord modifier**, which is a class rather than a key.
    ///
    /// A component that reads text reads all of it — type-ahead takes any printable character and a
    /// field inserts any printable character — so twenty-six corpus entries would be twenty-six
    /// bindings on a contract that has one. [`TEXT_SAMPLES`] is what keeps it a class: the sweep
    /// asks the same question of three different letters and requires the same answer.
    Text,
    /// A pointer press with these modifiers held. Answered by *does the picture differ from a plain
    /// click*.
    Click(Mods),
}

impl Trigger {
    /// **The chord that goes into a [`KeyMap`]**, which is a key and never a class or a click.
    ///
    /// A key map routes chords. A printable-text class is not a chord — a map that bound one would
    /// claim the letter it was sampled at — and a pointer gesture is not a key at all.
    #[must_use]
    pub fn chord(self) -> Option<Chord> {
        match self {
            Trigger::Key(c) => Some(c),
            Trigger::Text | Trigger::Click(_) => None,
        }
    }

    /// **The chord the sweep posts**, which is the class's sample as well as the key.
    #[must_use]
    pub fn posted(self) -> Option<Chord> {
        match self {
            Trigger::Key(c) => Some(c),
            Trigger::Text => Some(Chord::key(TEXT_SAMPLES[0])),
            Trigger::Click(_) => None,
        }
    }

    /// **One spelling, and both halves of the equality read it.**
    ///
    /// A key is the runtime's [`write_chord`], which is the same function a help bar prints
    /// through — so a binding cannot be documented under one name and registered under another. A
    /// click borrows the same function for its modifier prefix and then drops the filler key, so
    /// `Ctrl+Click` is spelled by the runtime down to its last plus sign.
    #[must_use]
    pub fn spell(self) -> String {
        let mut out = String::new();
        match self {
            Trigger::Key(c) => write_chord(&mut out, c),
            Trigger::Text => out.push_str(TEXT),
            Trigger::Click(m) => {
                write_chord(
                    &mut out,
                    Chord {
                        mods: m,
                        ..Chord::key(CLICK_KEY)
                    },
                );
                // **What the filler renders as is asked rather than assumed.** `write_chord` writes
                // a `Char`'s uppercase, which for `NUL` is itself — one scalar today, and a
                // subtraction of `CLICK_KEY.len_utf8()` would corrupt the prefix silently the day
                // the renderer spelled it `NUL`. Rendering the bare chord is the same function
                // answering the same question, so the two cannot disagree.
                let mut filler = String::new();
                write_chord(&mut filler, Chord::key(CLICK_KEY));
                out.truncate(out.len() - filler.len());
                out.push_str(CLICK);
            }
        }
        out
    }
}

/// The key [`Trigger::Click`] borrows to reach [`write_chord`]'s modifier prefixes, and it is
/// filler — [`crate::keys::MASK_KEY`]'s arrangement and its reason.
pub const CLICK_KEY: char = '\0';

/// What a pointer gesture is called in a help bar, after the modifiers.
pub const CLICK: &str = "Click";

/// What the printable-text class is called in a help bar.
pub const TEXT: &str = "(text)";

/// **The three letters the text class is asked about**, because a class asked about one letter is a
/// key.
pub const TEXT_SAMPLES: [char; 3] = ['a', 'm', 'z'];

/// **One declared binding**: the trigger, what it fires, and the words a user reads.
///
/// The `action` is an [`ActionId`] rather than a name because that is what the runtime's
/// [`Binding`] carries, and a contract that invented its own would be a second registry.
#[derive(Clone, Copy, Debug)]
pub struct Bind {
    /// What the user does.
    pub trigger: Trigger,
    /// **The modifiers this binding is deaf to**, and it is `Shift` almost everywhere it is not
    /// `NONE`.
    ///
    /// Spec §3's rule is that [`crate::keys::SIGNIFICANT`] is `CTRL | ALT` and **Shift is
    /// deliberately not in it**, because a capital is what Shift is for. A component that filters a
    /// key through [`crate::keys::is_chord`] and then matches on `code` therefore answers `Shift+X`
    /// exactly as it answers `X` — by construction and not by accident — so the sweep finds two
    /// chords where a help bar should print one line. This field is that fact, declared: the
    /// equality expands a bind to its trigger *and* its trigger with these modifiers held, and
    /// [`help`] prints the one line.
    ///
    /// It is not a licence. `collection` declares `Up` and `Shift+Up` as **two** binds, because
    /// there the second is a different action; a row that ignored Shift there would be claiming the
    /// selection never extends.
    pub ignores: Mods,
    /// What it fires. Unique within a contract, and a test says so.
    pub action: ActionId,
    /// The help text. **The whole of what a help bar adds** — the chord half is rendered.
    pub help: &'static str,
}

impl Bind {
    /// The spellings this bind answers to: its own, and its own with [`Bind::ignores`] held.
    #[must_use]
    pub fn spellings(&self) -> Vec<String> {
        let mut out = vec![self.trigger.spell()];
        if !self.ignores.is_empty()
            && let Trigger::Key(c) = self.trigger
        {
            out.push(
                Trigger::Key(Chord {
                    mods: c.mods.with(self.ignores),
                    ..c
                })
                .spell(),
            );
        }
        out
    }
}

/// **One component's keyboard contract**, joined to [`crate::INVENTORY`] by `id`.
///
/// `live` is the machine half and is not derived from `binds`. See this module's header.
pub struct Contract {
    /// The freeze's id, which is also the function a caller writes.
    pub id: &'static str,
    /// What is declared, in the order a help bar prints it.
    pub binds: &'static [Bind],
    /// **What the shipped component actually takes.** Runs the widget; see [`probe`].
    pub live: fn(Trigger) -> bool,
}

impl Contract {
    /// The runtime's key map, built from the declaration. **O4's registry half.**
    ///
    /// Pointer gestures and the text class are not in it, for [`Trigger::chord`]'s reason — the
    /// help is where every half of a contract meets, and [`help`] prints all three.
    #[must_use]
    pub fn key_map(&self) -> KeyMap {
        let mut map = KeyMap::new();
        for b in self.binds {
            if let Some(c) = b.trigger.chord() {
                map = map.bind(&[c], b.action, b.help);
            }
        }
        map
    }

    /// What this contract declares, as spellings, in declaration order.
    ///
    /// One bind can be two spellings — see [`Bind::ignores`].
    #[must_use]
    pub fn documented(&self) -> Vec<String> {
        self.binds.iter().flat_map(Bind::spellings).collect()
    }

    /// **O4's equality for this component**, as the set difference in both directions.
    #[must_use]
    pub fn disagreement(&self) -> Disagreement {
        let documented: std::collections::BTreeSet<String> =
            self.documented().into_iter().collect();
        let registered: std::collections::BTreeSet<String> =
            self.registered().into_iter().collect();
        Disagreement {
            dead: documented.difference(&registered).cloned().collect(),
            undeclared: registered.difference(&documented).cloned().collect(),
        }
    }

    /// **What the machine takes, swept over [`corpus`], with the runtime subtracted.**
    ///
    /// See [`control`]. A trigger the control arm also takes is not this component's.
    #[must_use]
    pub fn registered(&self) -> Vec<String> {
        corpus()
            .into_iter()
            .filter(|t| (self.live)(*t) && !control(*t))
            .map(Trigger::spell)
            .collect()
    }
}

/// What a contract declares and does not answer, and what it answers and does not declare.
///
/// **The whole of O4's equality as one value**, so that the gate and the report ask the same
/// question of the same slices rather than each deriving the difference its own way.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Disagreement {
    /// Declared and dead. **A help bar that lies.**
    pub dead: Vec<String>,
    /// Answered and undeclared. **A feature nobody can find.**
    pub undeclared: Vec<String>,
}

impl Disagreement {
    /// Whether the two halves agree, which is what the gate asserts.
    #[must_use]
    pub fn clean(&self) -> bool {
        self.dead.is_empty() && self.undeclared.is_empty()
    }
}

/// **The help a user sees, rendered from the declaration and from nowhere else.**
///
/// One line per bind. A key goes through the runtime's [`write_help`], which is R12's own renderer
/// and the reason a [`Binding`] carries a help string at all; the class and the click go through
/// the same [`Trigger::spell`] the sweep reports in, and
/// `tests::the_runtimes_renderer_and_the_spelling_agree` is what keeps the two the same words.
#[must_use]
pub fn help(c: &Contract) -> Vec<String> {
    let map = c.key_map();
    let mut keyed = map.bindings.iter();
    let mut lines = Vec::with_capacity(c.binds.len());
    for b in c.binds {
        let mut line = String::new();
        if b.trigger.chord().is_some() {
            let binding: &Binding = keyed
                .next()
                .expect("every key bind put a binding in the map, in order");
            write_help(&mut line, binding);
        } else {
            line.push_str(&b.trigger.spell());
            line.push(' ');
            line.push_str(b.help);
        }
        lines.push(line);
    }
    lines
}

// ── the sweep ────────────────────────────────────────────────────────────────────────────────────

/// **Every code a component on this map could bind.**
///
/// Sixteen, and they are the codes rather than the keyboard: `F1`, the keypad, the media keys and
/// the lock keys are reachable spellings that nothing in this crate reads, and a corpus that
/// carried them would spend a thousand drives proving it. What it must carry is every code any
/// drain loop in this crate matches on, plus the ones a reader would expect a component to bind and
/// would not notice missing — which is what `Insert`, `Tab` and `BackTab` are doing here.
pub const CODES: [Code; 16] = [
    Code::Up,
    Code::Down,
    Code::Left,
    Code::Right,
    Code::Home,
    Code::End,
    Code::PageUp,
    Code::PageDown,
    Code::Enter,
    Code::Escape,
    Code::Tab,
    Code::BackTab,
    Code::Backspace,
    Code::Delete,
    Code::Insert,
    Code::Char(' '),
];

/// **The modifier states a code is swept at.**
///
/// `Ctrl+Shift` is in it because §5's `Mode::Multi` binds it and because a component that read
/// `Ctrl` without masking `Shift` would take it silently. `Alt` is in it for the opposite reason:
/// **nothing on this map binds it**, so an `Alt` that lands anywhere is a component reading a
/// modifier byte it does not own.
pub fn mod_states() -> [Mods; 5] {
    [
        Mods::NONE,
        Mods::CTRL,
        Mods::SHIFT,
        Mods::CTRL.with(Mods::SHIFT),
        Mods::ALT,
    ]
}

/// The chord states a **letter** is swept at. Bare and `Shift` are [`Trigger::Text`]'s.
pub fn letter_states() -> [Mods; 3] {
    [Mods::CTRL, Mods::ALT, Mods::CTRL.with(Mods::SHIFT)]
}

/// **The whole sweep**: 16 codes x 5 modifier states, 26 letters x 3 chord states, the text class,
/// and the three modified clicks.
///
/// A plain click is **not** in it, and that is what makes the click arm an answer rather than a
/// tautology: it is the reference the other three are compared against, so *every* clickable widget
/// would otherwise register `Click` and the column would say nothing.
#[must_use]
pub fn corpus() -> Vec<Trigger> {
    let mut out = Vec::with_capacity(CODES.len() * 5 + 26 * 3 + 4);
    for code in CODES {
        for m in mod_states() {
            out.push(Trigger::Key(Chord {
                mods: m,
                ..Chord::new(code)
            }));
        }
    }
    for letter in 'a'..='z' {
        for m in letter_states() {
            out.push(Trigger::Key(Chord {
                mods: m,
                ..Chord::key(letter)
            }));
        }
    }
    out.push(Trigger::Text);
    for m in [Mods::CTRL, Mods::SHIFT, Mods::CTRL.with(Mods::SHIFT)] {
        out.push(Trigger::Click(m));
    }
    out
}

// ── the machine half ─────────────────────────────────────────────────────────────────────────────

/// **The control arm: the same sweep against a component that reads no key at all.**
///
/// `Driver::unhandled` answers *the keys this frame's batch carried that nobody took*, and **the
/// runtime is one of the takers**: `Tab` and `BackTab` are the focus walk and never reach a
/// component, at any modifier. Read without a control, every one of the thirteen contracts
/// registers ten chords it has never heard of, and the equality would have been reconciled by
/// declaring them — thirteen components each claiming a binding on `Ctrl+Shift+Tab`.
///
/// The control is `button`, and choosing a **component** rather than a bare `Ctx::interact` sink is
/// the second half of it: `button` declares `Interest::FOCUS` and reads no key, so what the control
/// measures is exactly *what a focusable gets for free*. That it is a row of the freeze and not a
/// fixture is why `tests::the_control_is_a_component_of_the_freeze_that_reads_no_key` can say so.
#[must_use]
pub fn control(t: Trigger) -> bool {
    live::nothing(t)
}

/// **The same `field`, at `WrapKind::Ruler`** — the configuration most callers write.
///
/// A component whose contract moves with its options is a component whose help does, and this is
/// the one place on this map where that is true twice. See [`RULER_REMOVES`].
#[must_use]
pub fn at_ruler(t: Trigger) -> bool {
    live::field_at_ruler(t)
}

/// **The same `collection`, at `Mode::Single`** — the default. See [`SINGLE_KEEPS`].
#[must_use]
pub fn at_single(t: Trigger) -> bool {
    live::collection_at_single(t)
}

/// The sink every probe draws on. Wide enough for a twenty-four column widget and tall enough for
/// four rows of content, which is what every screen in [`crate::golden`] is drawn at.
pub const SINK: (u16, u16) = (24, 6);

/// Where the click arm seeds, and where it lands. **Two different rows**, because a plain click and
/// a `Ctrl`-click on the same row of an empty selection leave the same selection.
///
/// Column six and not column one, which is a `tree`'s doing: at column one a row of depth 1 is
/// indent and chevron, so the press toggled a fold instead of selecting a row and the three pointer
/// gestures read dead on the one component of the three whose rows are not all the same shape.
pub const SEED: (u16, u16) = (6, 0);
/// The cell the modified click lands on. See [`SEED`].
pub const HIT: (u16, u16) = (6, 2);

/// **Run one trigger at a component and answer whether the component took it.**
///
/// The two arms are two questions, because a key and a click are observed by different mechanisms:
///
/// - **A key** is posted at a headless driver with the widget's own id planted as the focus, and
///   the answer is `Driver::unhandled().is_empty()` — *the keys this frame's batch carried that
///   nobody took*. It is read **after** the frame, which is components 22's finding: the window is
///   onto the same queue and is valid until the next frame begins.
/// - **A click** has no such window — a pointer event is not handed back — so the observable is the
///   picture. The same drive is run twice, once with the modifiers and once without, and the answer
///   is whether the two differ. A component that never reads `Response::mods` draws the same cells
///   either way, which is the answer for every row that declares no pointer gesture.
///
/// **Two frames and not one**, both arms: the focus is seated from the previous frame's
/// declaration, and a press is awarded at `end`.
pub fn probe<S, D>(t: Trigger, mut fresh: impl FnMut() -> S, mut draw: D) -> bool
where
    D: FnMut(&mut Pen, &mut Ctx<'_, '_>, &mut S) -> Id,
{
    let (w, h) = SINK;
    match t {
        Trigger::Click(m) => {
            let plain = clicked(Mods::NONE, &mut fresh, &mut draw);
            let held = clicked(m, &mut fresh, &mut draw);
            !plain.diff(&held).clean()
        }
        _ => {
            let chord = t.posted().expect("a key trigger posts a chord");
            let mut st = fresh();
            let mut pen = Pen::new(w, h);
            let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
            let mut id = None;
            driver.frame(|cx| id = Some(draw(&mut pen, cx, &mut st)));
            pen.end_frame();
            driver.plant(None, id, None);
            driver.post_key(crate::keys::press(chord));
            driver.frame(|cx| {
                let _ = draw(&mut pen, cx, &mut st);
            });
            pen.end_frame();
            driver.unhandled().is_empty()
        }
    }
}

/// One click drive: seed with a plain press, then press again at [`HIT`] with `m` held, and answer
/// with the picture.
fn clicked<S, D>(m: Mods, fresh: &mut impl FnMut() -> S, draw: &mut D) -> Canvas
where
    D: FnMut(&mut Pen, &mut Ctx<'_, '_>, &mut S) -> Id,
{
    let (w, h) = SINK;
    let mut st = fresh();
    let mut pen = Pen::new(w, h);
    let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
    drive_clicks(&mut driver, m, &mut |d: &mut Driver| {
        d.frame(|cx| {
            let _ = draw(&mut pen, cx, &mut st);
        });
        pen.end_frame();
    });
    pen.into_canvas()
}

/// **The click drive, with the frames the caller's.**
///
/// A seed press at [`SEED`] and then the real one at [`HIT`], each two frames — *a press is two
/// frames, because the grab is awarded at `end`* (components 30) — and the seed is what makes the
/// arm an answer: a plain click and a `Ctrl`-click on the same row of an empty selection leave the
/// same selection, so a drive with one press would report every collection deaf to the modifier
/// byte.
///
/// `frame` is a parameter because the overlay family cannot be driven through a closure that takes
/// its state: a popup's state is borrowed for the frame. See `live::owner`.
pub fn drive_clicks(driver: &mut Driver, m: Mods, frame: &mut dyn FnMut(&mut Driver)) {
    frame(driver);
    for (at, mods) in [(SEED, Mods::NONE), (HIT, m)] {
        for kind in [
            vitui_runtime::MouseKind::Down(vitui_runtime::Button::Left),
            vitui_runtime::MouseKind::Up(vitui_runtime::Button::Left),
        ] {
            driver.post_mouse(vitui_runtime::Mouse {
                x: at.0,
                y: at.1,
                kind,
                buttons: vitui_runtime::Buttons::NONE,
                mods,
                at: std::time::Instant::now(),
            });
            frame(driver);
            frame(driver);
        }
    }
}

// ── the probes, one per contract ─────────────────────────────────────────────────────────────────

/// The shipped components, drawn as a probe draws them.
///
/// Every one of these is the **component**, reached through its `_into` spelling so that one
/// closure serves both arms of [`probe`] — the key arm throws the [`Pen`] away and the click arm
/// reads it. Nothing here re-implements a drain loop; that is the whole point of the module.
mod live {
    use super::{Canvas, Ink, Pen, Trigger, probe};
    use vitui_runtime::Mods;
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::layout::Constraint;
    use vitui_runtime::work::{Cancel, Task, Worker};
    use vitui_runtime::{Ctx, Rect, Role};

    use crate::collect::{
        CollOpts, CollState, Column, Mode, PageOpts, TableOpts, TableState, TreeOpts, TreeState,
        collection_into, pagination_into, table_into, tree_into,
    };
    use crate::disclose::{Collapse, DiscloseOpts, collapsible_into};
    use crate::edit::Text;
    use crate::files::{Entry, PickerBody, PickerOpts, PickerState, Preview, file_picker_into};
    use crate::frame::face_paint;
    use crate::input::{
        ButtonOpts, FieldOpts, FormOpts, FormState, SelectOpts, SelectState, SliderOpts, Toggle,
        ToggleOpts, button_into, field_into, form_into, select_into, slider_into, toggle_into,
    };
    use crate::order::{Entry as Node, Order, Rows};
    use crate::overlay::PopupState;

    /// Six rows, and every letter [`super::TEXT_SAMPLES`] asks about starts one of them.
    ///
    /// Six because [`super::SEED`] and [`super::HIT`] must land on two different ones, and those
    /// three initials because **a collection's type-ahead consumes a letter only when it matches**:
    /// `seek` answers `None` for a buffer no label starts with and the drain loop declines the key,
    /// so over a fixture that answered one sample of three the text *class* would come apart into
    /// three different answers. That is a fact about the component and it is worth a fixture rather
    /// than a special case — see
    /// `super::tests::a_collections_type_ahead_declines_a_letter_no_row_starts_with`.
    const NAMES: [&str; 6] = ["alpha", "beta", "mu", "delta", "zeta", "gamma"];

    /// The rectangle every list-shaped probe is drawn at.
    fn list() -> Rect {
        Rect::new(0, 0, 24, 4)
    }

    /// The rectangle every one-row probe is drawn at.
    fn strip() -> Rect {
        Rect::new(0, 0, 24, 1)
    }

    pub fn collection(t: Trigger) -> bool {
        probe(
            t,
            CollState::new,
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut CollState| {
                // **`Mode::Multi`, which is where §5's pointer half is whole.** The default is
                // `Mode::Single`, where `apply` answers `Plain` and `Toggle` with the same call —
                // so a ctrl-click and a plain click leave the same picture and the sweep reports a
                // component deaf to the modifier byte. `Mode` decides three of the twenty-three
                // binds; see `super::tests::the_mode_decides_three_of_a_collections_binds`.
                let opts = CollOpts {
                    mode: Mode::Multi,
                    ..CollOpts::default()
                };
                collection_into(
                    pen,
                    cx,
                    list(),
                    st,
                    &opts,
                    Rows::of(NAMES.len()),
                    |buf, range| range.clone().find(|i| NAMES[*i].starts_with(buf)),
                    |ink, cx, r, i, face| {
                        let paint = face_paint(cx.theme(), face);
                        ink.pad_to(cx, r.x, r.y, NAMES[i], r.w, paint);
                    },
                )
                .id
            },
        )
    }

    pub fn table(t: Trigger) -> bool {
        probe(
            t,
            TableState::new,
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut TableState| {
                let cols = [
                    Column::new(0, "id", Constraint::Fixed(4)),
                    Column::new(1, "name", Constraint::Weight(1)),
                ];
                let opts = TableOpts {
                    coll: CollOpts {
                        mode: Mode::Multi,
                        ..CollOpts::default()
                    },
                    ..TableOpts::default()
                };
                table_into(
                    pen,
                    cx,
                    list(),
                    st,
                    &opts,
                    &cols,
                    Rows::of(NAMES.len()),
                    |buf, range| range.clone().find(|i| NAMES[*i].starts_with(buf)),
                    |ink, cx, r, c, face| {
                        let paint = face_paint(cx.theme(), face);
                        ink.pad_to(cx, r.x, r.y, NAMES[c.row], r.w, paint);
                    },
                )
                .id
            },
        )
    }

    pub fn tree(t: Trigger) -> bool {
        probe(
            t,
            || {
                // **The index is state and not a local**, which is ADR 0031's own rule arriving in
                // a gate: `Order::built` stamps `Revision::fresh()`, and a collection handed a
                // revision it has not seen **clears the selection**, because every field of the
                // store is a position in an order that has been replaced. Rebuilt inside the draw
                // it is a new order every frame, so the three pointer gestures read dead — a
                // ctrl-click and a plain click leave the same picture, for the same reason a
                // caller who rebuilds its index every frame has no selection.
                (
                    TreeState::new(),
                    // **Six, like every other list-shaped probe**: the type-ahead's corpus is
                    // `NAMES`, and a four-row index leaves `zeta` unreachable, so the text *class*
                    // comes apart into two answers on one component.
                    Order::built(vec![
                        Node::of(0),
                        Node::of(1).at_depth(1),
                        Node::of(2).at_depth(1),
                        Node::of(3),
                        Node::of(4).at_depth(1),
                        Node::of(5),
                    ]),
                )
            },
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut (TreeState, Order)| {
                let (st, index) = (&mut st.0, &st.1);
                let opts = TreeOpts {
                    coll: CollOpts {
                        mode: Mode::Multi,
                        ..CollOpts::default()
                    },
                    ..TreeOpts::default()
                };
                tree_into(
                    pen,
                    cx,
                    list(),
                    st,
                    &opts,
                    index,
                    |buf, range| range.clone().find(|i| NAMES[*i].starts_with(buf)),
                    |ink, cx, r, n, face| {
                        let paint = face_paint(cx.theme(), face);
                        ink.pad_to(cx, r.x, r.y, NAMES[n.node as usize], r.w, paint);
                    },
                )
                .id
            },
        )
    }

    pub fn pagination(t: Trigger) -> bool {
        probe(
            t,
            CollState::new,
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut CollState| {
                let opts = PageOpts::default();
                pagination_into(pen, cx, strip(), st, 9, &opts).id
            },
        )
    }

    pub fn field(t: Trigger) -> bool {
        probe(
            t,
            || {
                // **A textarea, and the caret on a row with one above it and one below.** §11's
                // one flag is a break rule and it decides three of the eighteen binds: at
                // `WrapKind::Ruler` a one-row `input` has no row to step to, so `Up` and `Down` are
                // declined (ADR 0042), and `Enter` is the caller's submit rather than a cluster.
                // Probed there, three declared binds would read dead — and probed at the last row,
                // `Down` would too. See `super::tests::the_break_rule_decides_three_of_a_fields_binds`.
                let mut st = Text::textarea();
                st.insert(24, "aaa\nbbb\nccc");
                st.home(24, false);
                st.step_row(24, true, false);
                st
            },
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut Text| {
                let opts = FieldOpts::default();
                field_into(pen, cx, Rect::new(0, 0, 24, 4), st, &opts).id
            },
        )
    }

    pub fn form(t: Trigger) -> bool {
        probe(
            t,
            || {
                (
                    FormState::new(),
                    [Text::input(), Text::input(), Text::input()],
                )
            },
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut (FormState, [Text; 3])| {
                // **Labels that begin with the letters the class is asked about**, for
                // `NAMES`'s reason one line up: a form's type-ahead consumes a letter only when a
                // label matches it, so a fixture that answered one sample of three would report
                // the class as three different answers.
                let labels = ["alpha", "mu", "zeta"];
                let opts = FormOpts::default();
                // **The form's own id and not a row's.** A form is a container and its keyboard is
                // read after the body, when `Ctx::scope` resumes the level a field has declined
                // out of; planted at a row, every key a *field* takes is measured under the
                // container's name and the contract comes back as the union of two components.
                form_into(pen, cx, list(), &mut st.0, &labels, &mut st.1, &opts).id
            },
        )
    }

    /// **Where the keyboard is seated when the sweep posts.**
    ///
    /// The overlay family is the one place on this map where a component has **two** keyboards, and
    /// they are not the same widget: the owner's drain loop runs while it is shut, and open, *the
    /// popup takes the keyboard from its owner* — `if cx.is_focused(owner) { cx.focus(list) }`,
    /// once, which is components 26's repair. A contract is the union, because a help bar is about
    /// a component and not about one of its states.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum Seat {
        /// Shut, with the owner's own id holding the focus.
        Owner,
        /// Open, and **one frame further on**: the handover happens inside the frame after the one
        /// the focus was planted on, so a two-frame drive measures the owner and calls the popup
        /// deaf.
        Popup,
    }

    pub fn select(t: Trigger) -> bool {
        select_seated(t, Seat::Owner) || select_seated(t, Seat::Popup)
    }

    /// **`select`'s two frames — three when the popup is the subject — written out.**
    ///
    /// The generic [`probe`] cannot reach this family, and the reason is spec §1's own sentence
    /// about the fifth component: `Ctx::overlay`'s body outlives the base pass, so a popup's state
    /// and its options are `&'f mut` and `&'f`. A draw closure of the shape
    /// `FnMut(&mut Pen, &mut Ctx<'_, '_>, &mut S)` hands its state a lifetime **shorter** than the
    /// frame's — components 26's *a nested overlay's state has to arrive as an `Option` the body
    /// takes*, arriving in a gate rather than in a component. Written out, the popup is a local of
    /// the driving function and the borrow is the frame's.
    fn select_seated(t: Trigger, seat: Seat) -> bool {
        static OPTIONS: [&str; 6] = NAMES;
        let run = |mods: Option<Mods>| -> Option<Canvas> {
            let (w, h) = super::SINK;
            let opts = SelectOpts::default();
            let mut st = SelectState::at(1);
            if seat == Seat::Popup {
                st.open();
            }
            let mut popup = PopupState::new();
            let mut pen = Pen::new(w, h);
            let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
            let mut one = |d: &mut Driver, pen: &mut Pen| {
                let mut own = None;
                d.frame(|cx| {
                    let here = cx.id();
                    own = Some(here);
                    let _ =
                        select_into(pen, cx, here, strip(), &mut st, &mut popup, &OPTIONS, &opts);
                });
                pen.end_frame();
                own
            };
            match mods {
                Some(m) => {
                    super::drive_clicks(&mut driver, m, &mut |d: &mut Driver| {
                        let _ = one(d, &mut pen);
                    });
                    Some(pen.into_canvas())
                }
                None => {
                    let id = one(&mut driver, &mut pen);
                    driver.plant(None, id, None);
                    // **The handover frame**, and it is why `Seat` exists rather than a boolean on
                    // the fixture: the popup takes the keyboard *during* the frame after the one it
                    // was planted on, so a drive that posts on that frame posts at the owner.
                    if seat == Seat::Popup {
                        let _ = one(&mut driver, &mut pen);
                    }
                    if let Some(c) = t.posted() {
                        driver.post_key(crate::keys::press(c));
                    }
                    let _ = one(&mut driver, &mut pen);
                    if driver.unhandled().is_empty() {
                        Some(pen.into_canvas())
                    } else {
                        None
                    }
                }
            }
        };
        two_ways(t, seat, run)
    }

    /// **The two arms of [`probe`], for the family that cannot use it.**
    ///
    /// `run(None)` is the key drive and answers `Some` when the component took the key; `run(Some)`
    /// is a click drive and answers with the picture. A click is answered at the owner's seat and
    /// asking twice would double every pointer answer, so [`Seat::Popup`] declines it.
    fn two_ways(
        t: Trigger,
        seat: Seat,
        mut run: impl FnMut(Option<Mods>) -> Option<Canvas>,
    ) -> bool {
        match t {
            Trigger::Click(m) => {
                if seat == Seat::Popup {
                    return false;
                }
                match (run(Some(Mods::NONE)), run(Some(m))) {
                    (Some(a), Some(b)) => !a.diff(&b).clean(),
                    _ => false,
                }
            }
            _ => run(None).is_some(),
        }
    }

    pub fn file_picker(t: Trigger) -> bool {
        let files: Vec<Entry<'_>> = NAMES
            .iter()
            .enumerate()
            .map(|(i, n)| Entry {
                id: i as u64,
                name: n,
            })
            .collect();
        picker(t, &files, Seat::Owner) || picker(t, &files, Seat::Popup)
    }

    /// [`select_seated`]'s twin, one family over. `file_picker_into` borrows four things for `'f`.
    fn picker(t: Trigger, files: &[Entry<'_>], seat: Seat) -> bool {
        let run = |mods: Option<Mods>| -> Option<Canvas> {
            let (w, h) = super::SINK;
            let opts = PickerOpts::default();
            let worker = Worker::queueing();
            let task: Task<Doc> = Task::new(&worker);
            let mut st = PickerState::new();
            st.choose(1);
            if seat == Seat::Popup {
                st.open();
            }
            let mut body: PickerBody<Doc> = PickerBody::new();
            let mut pen = Pen::new(w, h);
            let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
            let mut one = |d: &mut Driver, pen: &mut Pen| {
                let mut own = None;
                d.frame(|cx| {
                    let here = cx.id();
                    own = Some(here);
                    let _ = file_picker_into(
                        pen,
                        cx,
                        here,
                        strip(),
                        &mut st,
                        &mut body,
                        files,
                        &task,
                        decode,
                        line,
                        &opts,
                    );
                });
                pen.end_frame();
                own
            };
            match mods {
                Some(m) => {
                    super::drive_clicks(&mut driver, m, &mut |d: &mut Driver| {
                        let _ = one(d, &mut pen);
                    });
                    Some(pen.into_canvas())
                }
                None => {
                    let id = one(&mut driver, &mut pen);
                    driver.plant(None, id, None);
                    if seat == Seat::Popup {
                        let _ = one(&mut driver, &mut pen);
                    }
                    if let Some(c) = t.posted() {
                        driver.post_key(crate::keys::press(c));
                    }
                    let _ = one(&mut driver, &mut pen);
                    if driver.unhandled().is_empty() {
                        Some(pen.into_canvas())
                    } else {
                        None
                    }
                }
            }
        };
        two_ways(t, seat, run)
    }

    /// The one preview a probe needs: an identity and an extent. [`crate::golden`]'s, for its
    /// reason — a decode is a free function over an identity and never a closure (spec §15).
    struct Doc(u64);

    impl Preview for Doc {
        fn shows(&self) -> u64 {
            self.0
        }
        fn extent(&self) -> (u32, u32) {
            (8, 3)
        }
    }

    fn decode(id: u64, _cancel: &Cancel) -> Doc {
        Doc(id)
    }

    fn line(cx: &mut Ctx<'_, '_>, row: Rect, _doc: &Doc, i: u32) {
        let paint = cx.theme().paint(Role::Body);
        let _ = cx.text(row.x, row.y, "row", paint);
        let _ = i;
    }

    pub fn collapsible(t: Trigger) -> bool {
        probe(
            t,
            || Collapse::open_at(3),
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut Collapse| {
                let opts = DiscloseOpts::default();
                collapsible_into(
                    pen,
                    cx,
                    list(),
                    st,
                    "General",
                    &opts,
                    |_w| 3,
                    |ink: &mut Pen, cx: &mut Ctx<'_, '_>| {
                        let paint = cx.theme().paint(Role::Body);
                        for y in 0..3u16 {
                            ink.pad_to(cx, 0, i32::from(y), NAMES[usize::from(y)], 24, paint);
                        }
                    },
                )
                .response
                .id
            },
        )
    }

    pub fn slider(t: Trigger) -> bool {
        probe(
            t,
            || 0.4f32,
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, v: &mut f32| {
                let opts = SliderOpts::default();
                slider_into(pen, cx, strip(), v, &opts).id
            },
        )
    }

    pub fn checkbox(t: Trigger) -> bool {
        toggle(t, Toggle::Check)
    }

    pub fn radio(t: Trigger) -> bool {
        toggle(t, Toggle::Radio)
    }

    pub fn switch(t: Trigger) -> bool {
        toggle(t, Toggle::Switch)
    }

    /// **The same field at `WrapKind::Ruler`**, which is what most callers write. See
    /// [`super::RULER_REMOVES`].
    pub fn field_at_ruler(t: Trigger) -> bool {
        probe(
            t,
            || {
                let mut st = Text::input();
                st.insert(24, "value");
                st
            },
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut Text| {
                let opts = FieldOpts::default();
                field_into(pen, cx, Rect::new(0, 0, 24, 4), st, &opts).id
            },
        )
    }

    /// **The same collection at `Mode::Single`**, which is the default. See
    /// [`super::SINGLE_KEEPS`].
    pub fn collection_at_single(t: Trigger) -> bool {
        probe(
            t,
            CollState::new,
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, st: &mut CollState| {
                let opts = CollOpts::default();
                collection_into(
                    pen,
                    cx,
                    list(),
                    st,
                    &opts,
                    Rows::of(NAMES.len()),
                    |buf, range| range.clone().find(|i| NAMES[*i].starts_with(buf)),
                    |ink, cx, r, i, face| {
                        let paint = face_paint(cx.theme(), face);
                        ink.pad_to(cx, r.x, r.y, NAMES[i], r.w, paint);
                    },
                )
                .id
            },
        )
    }

    /// **The control arm.** A `button` is a tab stop that reads no key, so whatever it takes is
    /// the runtime's. See [`super::control`].
    pub fn nothing(t: Trigger) -> bool {
        probe(
            t,
            || (),
            |pen: &mut Pen, cx: &mut Ctx<'_, '_>, (): &mut ()| {
                let opts = ButtonOpts::default();
                button_into(pen, cx, strip(), "Save", &opts).id
            },
        )
    }

    /// **One machine and three configurations** (ADR 0041), so the three probes are one call with
    /// one field changed — and three contracts that must agree is a gate rather than a repetition.
    fn toggle(t: Trigger, kind: Toggle) -> bool {
        probe(
            t,
            || true,
            move |pen: &mut Pen, cx: &mut Ctx<'_, '_>, on: &mut bool| {
                let opts = ToggleOpts {
                    kind,
                    ..ToggleOpts::default()
                };
                toggle_into(pen, cx, strip(), "enabled", on, &opts).id
            },
        )
    }
}

// ── the declaration ──────────────────────────────────────────────────────────────────────────────

/// A key bind that answers `Shift` too — §3's rule, and [`Bind::ignores`]'s whole subject.
const fn key(code: Code, action: ActionId, help: &'static str) -> Bind {
    Bind {
        trigger: Trigger::Key(Chord::new(code)),
        ignores: Mods::SHIFT,
        action,
        help,
    }
}

/// A chord that answers `Shift` as well, because the component reads `Ctrl` and matches on `code`.
const fn deaf(code: Code, mods: Mods, action: ActionId, help: &'static str) -> Bind {
    Bind {
        trigger: Trigger::Key(Chord {
            code,
            mods,
            on: vitui_runtime::keys::On::BaseLayout,
        }),
        ignores: Mods::SHIFT,
        action,
        help,
    }
}

/// A key bind at a modifier set the three builders cannot spell together, deaf to nothing.
const fn chord(code: Code, mods: Mods, action: ActionId, help: &'static str) -> Bind {
    Bind {
        trigger: Trigger::Key(Chord {
            code,
            mods,
            on: vitui_runtime::keys::On::BaseLayout,
        }),
        ignores: Mods::NONE,
        action,
        help,
    }
}

/// A key bind that answers exactly one chord, `Shift` included.
const fn only(code: Code, action: ActionId, help: &'static str) -> Bind {
    Bind {
        trigger: Trigger::Key(Chord::new(code)),
        ignores: Mods::NONE,
        action,
        help,
    }
}

/// The printable-text class.
const fn text(action: ActionId, help: &'static str) -> Bind {
    Bind {
        trigger: Trigger::Text,
        ignores: Mods::NONE,
        action,
        help,
    }
}

/// A pointer gesture.
const fn click(mods: Mods, action: ActionId, help: &'static str) -> Bind {
    Bind {
        trigger: Trigger::Click(mods),
        ignores: Mods::NONE,
        action,
        help,
    }
}

/// `Ctrl` and `Shift` together, which [`Chord`]'s builders can spell and [`chord`] cannot take from
/// them at compile time.
const CTRL_SHIFT: Mods = Mods::CTRL.with(Mods::SHIFT);

/// **§5's collection: the arrows, the two selection chords, and the type-ahead.**
///
/// Shared verbatim by `table` and `tree`, which *call* `collection_into` — spec §6's *a table is a
/// collection plus a column split* and §7's *a tree is a collection plus a flatten index* — so
/// three rows of the freeze declare one list and `tests::the_three_collections_declare_one_contract`
/// is what says they still do.
const COLLECTION_FULL: &[Bind] = &[
    only(Code::Up, 1, "Previous row"),
    only(Code::Down, 2, "Next row"),
    // **`←` and `→` are `↑` and `↓` here**, which is `crate::nav::step`'s own pairing and
    // components 17's finding: a list's index grows downward, so the two axes are one. A `tree`
    // takes them away again — its `Refusal` reads them as fold and unfold before the cursor sees
    // them — and the sweep is blind to that, because both readings take the key.
    only(Code::Left, 3, "Previous row"),
    only(Code::Right, 4, "Next row"),
    only(Code::Home, 5, "First row"),
    only(Code::End, 6, "Last row"),
    only(Code::PageUp, 7, "Page up"),
    only(Code::PageDown, 8, "Page down"),
    chord(Code::Up, Mods::SHIFT, 9, "Extend the selection up"),
    chord(Code::Down, Mods::SHIFT, 10, "Extend the selection down"),
    chord(Code::Left, Mods::SHIFT, 11, "Extend the selection up"),
    chord(Code::Right, Mods::SHIFT, 12, "Extend the selection down"),
    chord(Code::Home, Mods::SHIFT, 13, "Extend to the first row"),
    chord(Code::End, Mods::SHIFT, 14, "Extend to the last row"),
    chord(Code::PageUp, Mods::SHIFT, 15, "Extend a page up"),
    chord(Code::PageDown, Mods::SHIFT, 16, "Extend a page down"),
    deaf(Code::Up, Mods::CTRL, 17, "Move without selecting, up"),
    deaf(Code::Down, Mods::CTRL, 18, "Move without selecting, down"),
    deaf(
        Code::Home,
        Mods::CTRL,
        19,
        "Move without selecting, to the first row",
    ),
    deaf(
        Code::End,
        Mods::CTRL,
        20,
        "Move without selecting, to the last row",
    ),
    key(Code::Char(' '), 21, "Toggle the row under the cursor"),
    deaf(Code::Char('a'), Mods::CTRL, 22, "Select every row"),
    key(Code::Escape, 23, "Clear the selection"),
    // ── and the four a pager does not have ────────────────────────────────────────────────────────
    //
    // `pagination` reaches `CollState` and the thirteen arms of `apply` and reaches neither the
    // type-ahead nor `from_click`: it has no labels to seek and reads `Gesture::Plain` from its own
    // arithmetic, so a modified click on a page number is a plain one (components 35).
    // **The deadline is in the words**, because the ticket names the contract as *type-ahead with a
    // deadline* and a buffer that lapses is the half a user notices. [`TYPE_AHEAD_LAPSES`] holds
    // the sentence to `crate::nav::WINDOW`, so the prose cannot drift from the constant.
    text(
        24,
        "Jump to the row that starts with what you type; the buffer lapses after a second",
    ),
    click(Mods::CTRL, 25, "Add or remove one row"),
    click(Mods::SHIFT, 26, "Extend the selection to here"),
    click(CTRL_SHIFT, 27, "Add the range up to here"),
];

/// Where [`COLLECTION_FULL`] stops being a pager's contract and starts being a listing's.
const PAGER_BINDS: usize = 23;

/// **What a pager declares: [`COLLECTION_FULL`] without the listing's four.**
///
/// The other half — the type-ahead and the three pointer gestures — has no constant of its own,
/// because nothing declares it: it is what a row loop *adds*, and
/// `tests::the_three_collections_declare_the_pager_plus_the_listing` takes it off the same slice.
const COLLECTION: &[Bind] = COLLECTION_FULL.split_at(PAGER_BINDS).0;

/// **§12's overlay family, from the owner's side.** Open it, and close it while it is open.
///
/// This is `file_picker`'s whole contract and only the first four rows of [`SELECT`]'s, which is a
/// **defect** rather than a design and is stated as a number — see [`PICKER_IS_MISSING`].
const OVERLAY_OWNER: &[Bind] = &[
    key(Code::Enter, 1, "Open the list"),
    key(Code::Char(' '), 2, "Open the list"),
    // **`Shift+Down` opens it too**, where `select` reads the same chord as *extend the selection*
    // — the difference between `key` and `only` here is the whole of [`PICKER_IS_MISSING`] in one
    // line: a picker's popup has no selection to extend.
    key(Code::Down, 3, "Open the list"),
    key(Code::Escape, 4, "Close it"),
];

/// **`select`: the owner's four, and the popup's list underneath them.**
///
/// A component with two keyboards, and the contract is the **union** because a help bar is about a
/// component and not about one of its states: shut, the owner's drain loop runs; open, *the popup
/// takes the keyboard from its owner* (components 26), and what answers is §5's collection at
/// `Mode::Single` plus the popup's own `Enter` and `Esc`. Three lines therefore name both meanings,
/// because one spelling does two things across the two states.
const SELECT: &[Bind] = &[
    key(
        Code::Enter,
        1,
        "Open the list; inside it, choose the row under the cursor",
    ),
    key(
        Code::Char(' '),
        2,
        "Open the list; inside it, select the row under the cursor",
    ),
    only(Code::Down, 3, "Open the list; inside it, the next row"),
    key(Code::Escape, 4, "Close it"),
    only(Code::Up, 5, "Previous row"),
    only(Code::Left, 6, "Previous row"),
    only(Code::Right, 7, "Next row"),
    only(Code::Home, 8, "First row"),
    only(Code::End, 9, "Last row"),
    only(Code::PageUp, 10, "Page up"),
    only(Code::PageDown, 11, "Page down"),
    chord(Code::Up, Mods::SHIFT, 12, "Extend the selection up"),
    chord(Code::Down, Mods::SHIFT, 13, "Extend the selection down"),
    chord(Code::Left, Mods::SHIFT, 14, "Extend the selection up"),
    chord(Code::Right, Mods::SHIFT, 15, "Extend the selection down"),
    chord(Code::Home, Mods::SHIFT, 16, "Extend to the first row"),
    chord(Code::End, Mods::SHIFT, 17, "Extend to the last row"),
    chord(Code::PageUp, Mods::SHIFT, 18, "Extend a page up"),
    chord(Code::PageDown, Mods::SHIFT, 19, "Extend a page down"),
    deaf(Code::Up, Mods::CTRL, 20, "Move without selecting, up"),
    deaf(Code::Down, Mods::CTRL, 21, "Move without selecting, down"),
    deaf(
        Code::Home,
        Mods::CTRL,
        22,
        "Move without selecting, to the first row",
    ),
    deaf(
        Code::End,
        Mods::CTRL,
        23,
        "Move without selecting, to the last row",
    ),
    deaf(Code::Char('a'), Mods::CTRL, 24, "Select every row"),
    text(
        25,
        "Jump to the row that starts with what you type; the buffer lapses after a second",
    ),
    // **Two and not three**, and the missing one is `Ctrl+Click`: the popup's list is
    // `Mode::Single`, where `apply` answers `Plain` and `Toggle` with the same call — see
    // [`SINGLE_KEEPS`], which is the same fact one component over.
    click(Mods::SHIFT, 26, "Extend the selection to here"),
    click(CTRL_SHIFT, 27, "Add the range up to here"),
];

/// **§11's field: the cluster steps, the edits, and the two chords it owns.**
///
/// Read at [`crate::edit::WrapKind::Words`], which is the configuration where the contract is
/// whole — see `tests::the_break_rule_decides_two_of_the_fields_own_bindings`.
const FIELD: &[Bind] = &[
    only(Code::Left, 1, "One cluster left"),
    only(Code::Right, 2, "One cluster right"),
    only(Code::Home, 3, "To the start of the row"),
    only(Code::End, 4, "To the end of the row"),
    only(Code::Up, 5, "One row up"),
    only(Code::Down, 6, "One row down"),
    chord(Code::Left, Mods::SHIFT, 7, "Select one cluster left"),
    chord(Code::Right, Mods::SHIFT, 8, "Select one cluster right"),
    chord(Code::Home, Mods::SHIFT, 9, "Select to the start of the row"),
    chord(Code::End, Mods::SHIFT, 10, "Select to the end of the row"),
    chord(Code::Up, Mods::SHIFT, 11, "Select one row up"),
    chord(Code::Down, Mods::SHIFT, 12, "Select one row down"),
    key(Code::Backspace, 13, "Delete the cluster behind the caret"),
    key(Code::Delete, 14, "Delete the cluster ahead of it"),
    key(Code::Enter, 15, "A new row"),
    deaf(Code::Char('z'), Mods::CTRL, 16, "Undo"),
    deaf(Code::Char('a'), Mods::CTRL, 17, "Select everything"),
    text(18, "Type"),
    // **A space is a printable character and this is the one place it is also a code.** Every other
    // contract on this map binds `Space` to an action; a field types it, so [`CODES`] carries it as
    // a key and the field declares it as one line of its own.
    key(Code::Char(' '), 19, "Type a space"),
];

/// **§3's `nav::cursor` over a form's rows, and the arrows are what `Ctrl+G` takes away.**
const FORM: &[Bind] = &[
    key(Code::Up, 1, "Previous field"),
    key(Code::Down, 2, "Next field"),
    key(Code::Left, 3, "Previous field"),
    key(Code::Right, 4, "Next field"),
    key(Code::Home, 5, "First field"),
    key(Code::End, 6, "Last field"),
    key(Code::PageUp, 7, "First field"),
    key(Code::PageDown, 8, "Last field"),
    text(
        9,
        "Jump to the field whose label starts with what you type; the buffer lapses after a second",
    ),
];

/// **§8's collapsible: the header, activated.**
const COLLAPSIBLE: &[Bind] = &[
    key(Code::Enter, 1, "Open or close the section"),
    key(Code::Char(' '), 2, "Open or close the section"),
];

/// **§14's slider: eight codes, and `Up` pairs with `Right` because a value grows upward.**
const SLIDER: &[Bind] = &[
    key(Code::Right, 1, "Larger"),
    key(Code::Up, 2, "Larger"),
    key(Code::Left, 3, "Smaller"),
    key(Code::Down, 4, "Smaller"),
    key(Code::PageUp, 5, "A page larger"),
    key(Code::PageDown, 6, "A page smaller"),
    key(Code::Home, 7, "The minimum"),
    key(Code::End, 8, "The maximum"),
];

/// **The three toggles are one machine** (ADR 0041), so they are one contract and three rows.
const TOGGLE: &[Bind] = &[
    key(Code::Char(' '), 1, "Flip it"),
    key(Code::Enter, 2, "Flip it"),
];

/// **Every component that reads a key, and nothing else.**
///
/// Thirteen rows of the freeze. The sixteen that are not here read no key at all, and that is not
/// an omission — `text`, `panel` and `rule` are pure drawers, and the freeze's other pure drawers
/// are the reason [`crate::obligations::o4`]'s population is the union of the two lists rather than
/// all twenty-nine.
pub const CONTRACTS: &[Contract] = &[
    Contract {
        id: "collection",
        binds: COLLECTION_FULL,
        live: live::collection,
    },
    Contract {
        id: "table",
        binds: COLLECTION_FULL,
        live: live::table,
    },
    Contract {
        id: "tree",
        binds: COLLECTION_FULL,
        live: live::tree,
    },
    Contract {
        id: "pagination",
        binds: COLLECTION,
        live: live::pagination,
    },
    Contract {
        id: "select",
        binds: SELECT,
        live: live::select,
    },
    Contract {
        id: "file_picker",
        binds: OVERLAY_OWNER,
        live: live::file_picker,
    },
    Contract {
        id: "field",
        binds: FIELD,
        live: live::field,
    },
    Contract {
        id: "form",
        binds: FORM,
        live: live::form,
    },
    Contract {
        id: "collapsible",
        binds: COLLAPSIBLE,
        live: live::collapsible,
    },
    Contract {
        id: "slider",
        binds: SLIDER,
        live: live::slider,
    },
    Contract {
        id: "checkbox",
        binds: TOGGLE,
        live: live::checkbox,
    },
    Contract {
        id: "radio",
        binds: TOGGLE,
        live: live::radio,
    },
    Contract {
        id: "switch",
        binds: TOGGLE,
        live: live::switch,
    },
];

// ── what is absent, and the fact that makes it absent ────────────────────────────────────────────

/// A binding that is **not** bound, with the fact that keeps it unbound.
///
/// *Missing* and *not yet done* are the same thing to a reader, so the reason travels with the row
/// and `tests::the_absent_binding_is_absent_from_every_help_line` reads both halves.
#[derive(Clone, Copy, Debug)]
pub struct Absent {
    /// What a reader would look for.
    pub what: &'static str,
    /// The spellings no help line may contain.
    pub spellings: &'static [&'static str],
    /// The fact that makes it absent, and it is a fact about a file rather than a plan.
    pub because: &'static str,
}

/// **One row: word motion.**
///
/// It needs UAX #29's word-boundary half, which the engine's export does not offer — engine ticket
/// 06 ships `graphemes()` and `width_of()` and no word iterator, and nothing above the engine may
/// re-derive one, because a cell holds an interned grapheme-cluster handle and no cell, handle or
/// style bit is readable from outside the engine (ADR 0023).
pub const ABSENT: &[Absent] = &[Absent {
    what: "word motion",
    spellings: &["Ctrl+Left", "Ctrl+Right", "Ctrl+Backspace", "Ctrl+Delete"],
    because: "the engine exports `graphemes()` and `width_of()` and no word iterator, so there is \
              no word boundary above it to move to",
}];

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this module is gated or reported on has one home and it is here** — the runtime's
// ledger rule (`crates/vitui-runtime/src/ledger.rs`), inherited by `crate::keys` and
// `crate::state`. Every figure is a count over a deterministic sweep, so none carries a machine.

/// **How many triggers the sweep asks about. 162.**
///
/// 16 codes x 5 modifier states, 26 letters x 3 chord states, the text class, and three modified
/// clicks.
pub const CORPUS_SIZE: usize = CODES.len() * 5 + 26 * 3 + 1 + 3;

/// **What each contract answers to**, in [`CONTRACTS`]'s order, counted as spellings.
///
/// A spelling and not a bind: [`Bind::ignores`] makes twelve of `slider`'s sixteen and one of
/// `collection`'s thirty-four a second chord on one line of help.
pub const REGISTERED: [(&str, usize); 13] = [
    ("collection", 34),
    ("table", 34),
    ("tree", 34),
    // **Four fewer, and the four are the listing's**: a pager has no labels to seek and reads
    // `Gesture::Plain` from its own arithmetic, so the type-ahead and the three pointer gestures
    // are `collection`'s and not the store's.
    ("pagination", 30),
    // **Thirty-five, and the popup's twenty-seven are most of it**: open, the list takes the
    // keyboard from its owner and answers §5's collection at `Mode::Single`.
    ("select", 35),
    // **Eight, and the gap is [`PICKER_IS_MISSING`]** rather than a smaller component.
    ("file_picker", 8),
    ("field", 25),
    ("form", 17),
    ("collapsible", 4),
    ("slider", 16),
    ("checkbox", 4),
    ("radio", 4),
    ("switch", 4),
];

/// **What a focusable gets for free, measured on the control arm. 10.**
///
/// `Tab` and `BackTab` at all five modifier states, and nothing else: the focus walk takes them
/// before a component sees them, which is §21's *`Tab` inside a trap* answered from the other side
/// — it is the runtime's key and no component's. Subtracted from all thirteen; see [`control`].
pub const RUNTIMES_SHARE: usize = 10;

/// **What `WrapKind` takes away from a field's contract. 3.**
///
/// `Up`, `Down` and `Enter`, at `WrapKind::Ruler` — ADR 0042's *a field declines what it cannot act
/// on* as a number. Counted in spellings the three are six, because each ignores `Shift`.
pub const RULER_REMOVES: usize = 6;

/// **The words every type-ahead line ends in, and the constant they are held to.**
///
/// The ticket names the contract as *type-ahead with a deadline*, and a buffer that lapses is the
/// half a user notices — a letter typed after the window starts a fresh search rather than
/// continuing an abandoned one. Prose in a help string is exactly what goes stale, so
/// `tests::every_type_ahead_line_states_the_deadline_and_the_words_match_the_constant` reads it
/// back against [`crate::nav::WINDOW`].
pub const TYPE_AHEAD_LAPSES: &str = "the buffer lapses after a second";

/// **What a `file_picker`'s open popup does not answer that a `select`'s does. 27.**
///
/// The two are one family and, until this ticket, one declaration. They are not: `select`'s popup
/// **takes the keyboard from its owner** — `if cx.is_focused(owner) { cx.focus(list) }`, which is
/// components 26's repair after `console` found the arrows dead — and reads `Enter` and `Esc`
/// through `collect::Refusal`. `file_picker`'s popup does neither: it draws a `collection_into`,
/// never seats a focus and declares no refusal, so **an open picker can only be used with a
/// mouse**. Its list has no arrows, no `Home`, no type-ahead and no way to choose a file.
///
/// **35 against 8 over one family**, and the shape is components 32's own sentence from the other
/// side: that ticket made the shut face *one drawing* because two copies of a drawing that has
/// already been wrong once is one copy too many — and the keyboard was the half that stayed
/// transcribed. Filed as components architecture issue 23 rather than repaired here: seating a
/// focus and minting a refusal inside `picker_body` is a component's keyboard being designed, which
/// is `files.rs`'s ticket and not this one. `crate::contract::tests::the_two_overlay_owners_do_not_declare_one_contract`
/// is the number, so the day it is repaired the count fails rather than the gate quietly widening.
pub const PICKER_IS_MISSING: usize = 27;

/// **What survives a collection's pointer half at `Mode::Single`. 2 of 3.**
///
/// The default is `Mode::Single`, where `apply` answers `Plain` and `Toggle` with the **same call**
/// and refuses `Extend` and `ExtendAdd` outright. So the ctrl-click is *invisible* — it does
/// exactly what a plain click does — and the two extends are *differences*, because a plain click
/// moves the selection and they do nothing at all.
///
/// **The one that disappears is the one that works.** A sweep reading `Mode::Single` would report a
/// collection deaf to `Ctrl` and fluent in the two gestures its own mode refuses, which is why the
/// contract is read at `Mode::Multi` — §5's *click, ctrl-click, shift-click, `Space`, `Shift+↑/↓`,
/// `Ctrl+A`, `Escape`* is written about `Multi` in as many words.
pub const SINGLE_KEEPS: usize = 2;

// ── the spellings that are refused, kept because a gate nobody has watched fail is not a gate ────

/// **A contract that is wrong in both directions at once**, so the equality can be watched failing.
///
/// Thirteen contracts that all agree is a gate nobody has seen work. This one declares a chord no
/// component on this map reads and omits one its own component answers, and
/// `tests::a_contract_wrong_in_both_directions_is_caught_in_both_directions` runs the same sweep
/// over it.
pub mod defective {
    use super::{Bind, Code, Contract, Mods, Trigger, live};

    /// `F1`, which nothing here binds, declared; and `Space`, which the component takes, omitted.
    const HALF_TRUE: &[Bind] = &[
        Bind {
            trigger: Trigger::Key(vitui_runtime::keys::Chord::new(Code::F(1))),
            ignores: Mods::NONE,
            action: 1,
            help: "Open or close the section",
        },
        Bind {
            trigger: Trigger::Key(vitui_runtime::keys::Chord::new(Code::Enter)),
            ignores: Mods::SHIFT,
            action: 2,
            help: "Open or close the section",
        },
    ];

    /// The shipped `collapsible`, under a declaration that is wrong twice.
    pub const CONTRACT: Contract = Contract {
        id: "collapsible",
        binds: HALF_TRUE,
        live: live::collapsible,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::INVENTORY;
    use std::collections::BTreeSet;

    fn contract(id: &str) -> &'static Contract {
        CONTRACTS
            .iter()
            .find(|c| c.id == id)
            .expect("every id named by a test is a contract")
    }

    /// **O4's equality, chord for chord, over every component that reads a key.**
    ///
    /// The two sides come from two places and neither is derived from the other: `documented` is
    /// [`CONTRACTS`]'s declaration and `registered` is a sweep that **runs the shipped component**
    /// over [`corpus`] with [`control`] subtracted. One direction is a help bar that lies and the
    /// other is a feature nobody can find, and the day either happens this is what says which chord.
    #[test]
    fn documented_equals_registered_for_every_component() {
        for c in CONTRACTS {
            let d = c.disagreement();
            assert!(
                d.dead.is_empty(),
                "`{}` documents {:?} and the component does not answer it — a help bar that lies",
                c.id,
                d.dead
            );
            assert!(
                d.undeclared.is_empty(),
                "`{}` answers {:?} and documents none of it — a feature nobody can find",
                c.id,
                d.undeclared
            );
            assert!(d.clean());
        }
    }

    /// **The sweep's own size and each component's answer, written down.**
    ///
    /// An equality between two lists says nothing about how much either holds: thirteen components
    /// each declaring and answering exactly `Escape` would pass the gate above. This is the
    /// magnitude beside it.
    #[test]
    fn the_sweep_and_every_answer_are_the_size_they_are() {
        assert_eq!(corpus().len(), CORPUS_SIZE);
        assert_eq!(CORPUS_SIZE, 162);
        let counted: Vec<(&str, usize)> = CONTRACTS
            .iter()
            .map(|c| (c.id, c.registered().len()))
            .collect();
        assert_eq!(counted, REGISTERED.to_vec());
    }

    /// **The control arm is a component of the freeze that reads no key**, and what it takes is the
    /// runtime's.
    ///
    /// Ten of a hundred and sixty-two, all of them `Tab` or `BackTab`. Read without this
    /// subtraction, all thirteen contracts register a binding on `Ctrl+Shift+Tab` and the equality
    /// is reconciled by declaring one.
    #[test]
    fn the_control_is_a_component_of_the_freeze_that_reads_no_key() {
        let taken: Vec<String> = corpus()
            .into_iter()
            .filter(|t| control(*t))
            .map(Trigger::spell)
            .collect();
        assert_eq!(taken.len(), RUNTIMES_SHARE);
        assert_eq!(RUNTIMES_SHARE, 10);
        for t in &taken {
            assert!(
                t.ends_with("Tab"),
                "the control arm took {t:?}, which is not the focus walk"
            );
        }
        // The control is `button`, which is a row of the freeze — so *a tab stop that reads no key*
        // is a statement about a shipped component and not about a fixture.
        assert!(INVENTORY.iter().any(|c| c.id == "button" && c.built));
        assert!(!CONTRACTS.iter().any(|c| c.id == "button"));
    }

    /// **Every contract names a built row of the freeze, and the two written lists agree with it.**
    ///
    /// [`crate::obligations::KEYBOARD_DOCUMENTED`] and `KEYBOARD_REGISTERED` are written out for
    /// `AXIS_SCENES`'s reason — a `const fn` over this module would make the population and the
    /// evidence one expression — and this is the comparison that keeps them honest, which is
    /// `crate::doc`'s arrangement for O1.
    #[test]
    fn the_written_lists_and_the_sweep_agree() {
        let ids: Vec<&str> = CONTRACTS.iter().map(|c| c.id).collect();
        for id in &ids {
            let row = INVENTORY
                .iter()
                .find(|c| c.id == *id)
                .unwrap_or_else(|| panic!("`{id}` declares a contract and is not in the freeze"));
            assert!(row.built, "`{id}` declares a contract and is not built");
        }
        assert_eq!(ids, crate::obligations::KEYBOARD_DOCUMENTED.to_vec());
        assert_eq!(ids, crate::obligations::KEYBOARD_REGISTERED.to_vec());
        let mut seen = BTreeSet::new();
        for id in &ids {
            assert!(seen.insert(*id), "`{id}` declares two contracts");
        }
    }

    /// **The help is rendered, and the runtime's renderer and this module's spelling are the same
    /// words.**
    ///
    /// [`help`] prints a key through [`write_help`], which is R12's own renderer; the class and the
    /// click go through [`Trigger::spell`], which the sweep reports in. If the two disagreed, a
    /// help bar would print one name for a chord the equality knows under another — and the
    /// equality would go on holding.
    #[test]
    fn the_runtimes_renderer_and_the_spelling_agree() {
        for c in CONTRACTS {
            let lines = help(c);
            assert_eq!(lines.len(), c.binds.len(), "`{}`", c.id);
            for (line, b) in lines.iter().zip(c.binds) {
                let want = format!("{} {}", b.trigger.spell(), b.help);
                assert_eq!(line, &want, "`{}` renders {line:?}", c.id);
                assert!(line.ends_with(b.help));
            }
        }
    }

    /// **A click is spelled by the runtime down to its last plus sign.**
    #[test]
    fn a_pointer_gesture_is_spelled_by_the_same_function_a_chord_is() {
        assert_eq!(Trigger::Click(Mods::NONE).spell(), "Click");
        assert_eq!(Trigger::Click(Mods::CTRL).spell(), "Ctrl+Click");
        assert_eq!(Trigger::Click(Mods::SHIFT).spell(), "Shift+Click");
        assert_eq!(Trigger::Click(CTRL_SHIFT).spell(), "Ctrl+Shift+Click");
        assert_eq!(Trigger::Text.spell(), "(text)");
        assert_eq!(Trigger::Key(Chord::key('a').ctrl()).spell(), "Ctrl+A");
    }

    /// **An action fires one thing**, and a contract that reused an id would have two bindings the
    /// runtime's key map cannot tell apart.
    #[test]
    fn an_action_is_declared_once_within_a_contract() {
        for c in CONTRACTS {
            let mut seen = BTreeSet::new();
            for b in c.binds {
                assert!(
                    seen.insert(b.action),
                    "`{}` binds action {} twice",
                    c.id,
                    b.action
                );
            }
            assert_eq!(
                c.key_map().len(),
                c.binds
                    .iter()
                    .filter(|b| b.trigger.chord().is_some())
                    .count()
            );
        }
    }

    /// **The three collections declare one contract and the pager declares its keys alone.**
    ///
    /// §6's *a table is a collection plus a column split* and §7's *a tree is a collection plus a
    /// flatten index* as an equality between three declarations, and §5's store without its row
    /// loop as the difference — `pagination` reaches `CollState` and the thirteen arms of `apply`
    /// and reaches neither the type-ahead nor `from_click` (components 35).
    #[test]
    fn the_three_collections_declare_the_pager_plus_the_listing() {
        let full = contract("collection").documented();
        assert_eq!(contract("table").documented(), full);
        assert_eq!(contract("tree").documented(), full);
        let pager = contract("pagination").documented();
        let listing: Vec<String> = COLLECTION_FULL[PAGER_BINDS..]
            .iter()
            .flat_map(Bind::spellings)
            .collect();
        assert_eq!(listing.len(), 4);
        assert_eq!(
            full,
            pager
                .iter()
                .cloned()
                .chain(listing.iter().cloned())
                .collect::<Vec<String>>()
        );
    }

    /// **The three toggles are one machine** (ADR 0041), so they declare one contract.
    #[test]
    fn the_three_toggles_declare_one_contract() {
        let want = contract("checkbox").documented();
        assert_eq!(contract("radio").documented(), want);
        assert_eq!(contract("switch").documented(), want);
        assert_eq!(want.len(), 4);
    }

    /// **The equality is watched failing, in both directions, on one declaration.**
    ///
    /// A declared chord nothing answers is a help bar that lies; a chord answered and undeclared is
    /// a feature nobody can find. Neither is visible from the other side, which is why the gate is
    /// a set equality and not a subset — and why this is two assertions rather than one.
    #[test]
    fn a_contract_wrong_in_both_directions_is_caught_in_both_directions() {
        let d = defective::CONTRACT.disagreement();
        assert!(!d.clean());
        assert_eq!(d.dead, vec!["F1"]);
        assert_eq!(d.undeclared, vec!["Shift+Space", "Space"]);
    }

    /// **The text class is a class**, asked of three letters rather than of one.
    ///
    /// A component that read `a` and not `z` would be a component with twenty-six bindings, and
    /// [`Trigger::Text`] would be reporting the first of them as all of them.
    #[test]
    fn the_text_class_is_a_class() {
        for c in CONTRACTS {
            let answers: Vec<bool> = TEXT_SAMPLES
                .into_iter()
                .map(|ch| (c.live)(Trigger::Key(Chord::key(ch))))
                .collect();
            assert!(
                answers.iter().all(|a| *a == answers[0]),
                "`{}` answers {:?} to {TEXT_SAMPLES:?}, so its text class is three bindings",
                c.id,
                answers
            );
            assert_eq!(
                answers[0],
                (c.live)(Trigger::Text),
                "`{}` disagrees with its own class",
                c.id
            );
        }
    }

    /// **A collection's type-ahead consumes a letter only when a row starts with it.**
    ///
    /// The fixture's own rule, as a number: `seek` answers `None` for a buffer no label matches and
    /// the drain loop declines the key, so the letter reaches the application. It is why
    /// `live::NAMES` has an `alpha`, a `mu` and a `zeta` in it, and it is a fact about the
    /// component rather than about the fixture.
    #[test]
    fn a_collections_type_ahead_declines_a_letter_no_row_starts_with() {
        let coll = contract("collection");
        assert!((coll.live)(Trigger::Key(Chord::key('a'))));
        assert!(
            !(coll.live)(Trigger::Key(Chord::key('q'))),
            "a letter no row starts with was consumed, so the application never sees it"
        );
        // A field takes every letter, matching or not — which is what makes the two different
        // readings of one class.
        let field = contract("field");
        assert!((field.live)(Trigger::Key(Chord::key('q'))));
    }

    /// **§11's one flag takes three binds away, and the field declines them rather than eating
    /// them** (ADR 0042).
    ///
    /// `Up`, `Down` and `Enter`: a one-row `input` has no row to step to and its `Enter` is the
    /// caller's submit. Six spellings, because each ignores `Shift`. Probed at `Ruler` the contract
    /// would read three binds dead — which is why the declaration is read at `Words`, where it is
    /// whole, and why this number is written down instead.
    #[test]
    fn the_break_rule_decides_three_of_a_fields_binds() {
        let whole: BTreeSet<String> = contract("field").registered().into_iter().collect();
        let ruler: BTreeSet<String> = corpus()
            .into_iter()
            .filter(|t| at_ruler(*t) && !control(*t))
            .map(Trigger::spell)
            .collect();
        let gone: Vec<&String> = whole.difference(&ruler).collect();
        assert_eq!(gone.len(), RULER_REMOVES);
        assert_eq!(
            gone,
            vec![
                "Down",
                "Enter",
                "Shift+Down",
                "Shift+Enter",
                "Shift+Up",
                "Up"
            ]
        );
        assert!(ruler.difference(&whole).next().is_none());
    }

    /// **`Mode` decides the pointer half, and the gesture it hides is the one that works.**
    ///
    /// At `Mode::Single` — the default — a ctrl-click *is* a plain click, so it leaves the same
    /// picture and reads deaf; the two extends leave a different one, because a plain click moves
    /// the selection and a refused gesture does nothing. See [`SINGLE_KEEPS`].
    #[test]
    fn the_mode_decides_the_pointer_half_and_hides_the_gesture_that_works() {
        let kept: Vec<String> = [Mods::CTRL, Mods::SHIFT, CTRL_SHIFT]
            .into_iter()
            .filter(|m| at_single(Trigger::Click(*m)))
            .map(|m| Trigger::Click(m).spell())
            .collect();
        assert_eq!(kept.len(), SINGLE_KEEPS);
        assert_eq!(kept, vec!["Shift+Click", "Ctrl+Shift+Click"]);
        // At `Mode::Multi` all three answer, which is the configuration the contract is read at.
        for m in [Mods::CTRL, Mods::SHIFT, CTRL_SHIFT] {
            assert!((contract("collection").live)(Trigger::Click(m)));
        }
    }

    /// **The four chord leaks are shut, and the shipped answer is that the key reaches the
    /// application.**
    ///
    /// Every one of them was a component matching on `k.code` with no modifier guard, and every one
    /// of them was invisible to §21's *a chord pressed into every focusable types nothing* — moving
    /// a caret, opening a list and clearing a selection all type nothing. `Ctrl+Left` is the sharpest:
    /// it is what a user pressing for **word motion** means, and word motion is [`ABSENT`]'s one
    /// row, so the widget was swallowing the accelerator *and* answering it with a cluster.
    #[test]
    fn the_four_chord_leaks_are_shut() {
        let leaks: [(&str, Trigger); 7] = [
            ("field", Trigger::Key(Chord::new(Code::Left).ctrl())),
            ("field", Trigger::Key(Chord::new(Code::Backspace).ctrl())),
            // **Not `Ctrl+Down` for `select`**: open, its popup's `ctrl_step` owns that chord, and
            // a leak test that named it would be asserting the absence of a shipped binding. The
            // two below are the owner's own keys with a modifier on them, which nothing here binds.
            ("select", Trigger::Key(Chord::new(Code::Enter).ctrl())),
            ("select", Trigger::Key(Chord::key(' ').ctrl())),
            ("file_picker", Trigger::Key(Chord::new(Code::Down).ctrl())),
            ("collection", Trigger::Key(Chord::new(Code::Escape).ctrl())),
            ("collection", Trigger::Key(Chord::key(' ').alt())),
        ];
        for (id, t) in leaks {
            assert!(
                !(contract(id).live)(t),
                "`{id}` still takes {:?}, so an application's accelerator never arrives",
                t.spell()
            );
        }
        // **And the collection's is watched failing on the spelling it replaced**, which is the one
        // of the four small enough to keep runnable beside the shipped arm.
        let esc = crate::keys::press(Chord::new(Code::Escape).ctrl());
        let space = crate::keys::press(Chord::key(' ').alt());
        assert_eq!(crate::collect::from_key(&esc, 0, None), None);
        assert_eq!(crate::collect::from_key(&space, 0, None), None);
        assert_eq!(
            crate::collect::defective::from_key_on_code_alone(&esc, 0, None),
            Some(crate::collect::Gesture::Nothing)
        );
        assert_eq!(
            crate::collect::defective::from_key_on_code_alone(&space, 0, None),
            Some(crate::collect::Gesture::Toggle(0))
        );
    }

    /// **What is absent is absent from every help line, and the reason is a fact about a file.**
    ///
    /// *Missing* and *not yet done* are the same thing to a reader, so the row carries the fact
    /// that makes it absent — and this reads both halves: no help line anywhere mentions a word
    /// motion, and the engine still exports no word iterator. The day one ships, the second half
    /// fails and the row is revisited rather than quietly staying true.
    #[test]
    fn the_absent_binding_is_absent_from_every_help_line() {
        assert_eq!(ABSENT.len(), 1);
        for a in ABSENT {
            for c in CONTRACTS {
                for line in help(c) {
                    for spelling in a.spellings {
                        assert!(
                            !line.contains(spelling),
                            "`{}` offers {spelling:?}, which is {} — {}",
                            c.id,
                            a.what,
                            a.because
                        );
                    }
                }
            }
        }
        // **The fact, read out of the engine's own surface**, and the needle is the *rule* rather
        // than one name. `graphemes()` and `width_of()` are what engine ticket 06 exports; a word
        // iterator could arrive as `words`, `word_starts`, `next_word`, `iter_words` or
        // `word_boundaries`, and a scan for one of those five is the needle problem this map has
        // already met five times — `pub fn picture(` against `pub fn picture<P: Pixels>(`. So the
        // scan is *no exported declaration mentions a word at all*, which cannot be satisfied by
        // choosing a different name.
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../vitui-engine/src/text.rs"
        ))
        .expect("the engine's text module is two directories over");
        assert!(source.contains("pub fn graphemes"));
        let worded: Vec<&str> = source
            .lines()
            .map(str::trim_start)
            .filter(|l| l.starts_with("pub fn") && l.contains("word"))
            .collect();
        assert!(
            worded.is_empty(),
            "the engine exports {worded:?}, so `ABSENT`'s one row has stopped being true"
        );
    }

    /// **`select` and `file_picker` do *not* declare one contract, and the difference is a
    /// defect** — see [`PICKER_IS_MISSING`].
    ///
    /// They are one family and one drawing (components 32), and the keyboard is the half that
    /// stayed transcribed: `select`'s popup takes the keyboard and reads `Enter` and `Esc`;
    /// `file_picker`'s seats no focus and declares no refusal, so an open picker can only be used
    /// with a mouse. The equality is asserted on the **owner's four** — which really are one
    /// contract — and the rest is a count, so the day issue 23 is answered this fails rather than
    /// quietly widening.
    #[test]
    fn the_two_overlay_owners_do_not_declare_one_contract() {
        let picker = contract("file_picker").documented();
        let select = contract("select").documented();
        assert_eq!(picker.len(), 8, "the picker's owner is four binds");
        assert_eq!(select.len() - picker.len(), PICKER_IS_MISSING);
        assert_eq!(PICKER_IS_MISSING, 27);
        // The owner's half is shared, and *that* is the equality components 32 earned: the three
        // opening keys and the close, in the same order, with the same words.
        for (a, b) in OVERLAY_OWNER.iter().zip(SELECT) {
            assert_eq!(
                a.trigger.chord().map(|c| c.code),
                b.trigger.chord().map(|c| c.code)
            );
        }
        // And the one line where the two spellings differ is `Down`, because a picker's popup has
        // no selection for `Shift+Down` to extend.
        assert!(picker.contains(&"Shift+Down".to_string()));
        assert!(select.contains(&"Shift+Down".to_string()));
    }

    /// **One text class, two mechanisms — and only one of them has a deadline to state.**
    ///
    /// The ticket asks for *type-ahead with a deadline*, and a help line that named the mechanism
    /// and not the window would document the half a user cannot see. But `Trigger::Text` covers two
    /// different things: a `field` **types** a printable character into a buffer that never lapses,
    /// and a collection **seeks** with one that lapses after `crate::nav::WINDOW`. So the gate is a
    /// partition rather than a blanket, and the one row on the typing side is named — a scan that
    /// simply required the words everywhere would have been satisfied by writing them onto `field`,
    /// where they are false.
    #[test]
    fn every_seeking_line_states_its_deadline_and_the_typing_one_has_none() {
        assert_eq!(crate::nav::WINDOW, std::time::Duration::from_secs(1));
        let mut seeks: Vec<&str> = Vec::new();
        let mut types: Vec<&str> = Vec::new();
        for c in CONTRACTS {
            for b in c.binds {
                if b.trigger != Trigger::Text {
                    continue;
                }
                if b.help.ends_with(TYPE_AHEAD_LAPSES) {
                    seeks.push(c.id);
                } else {
                    types.push(c.id);
                }
            }
        }
        // The three collections share one declaration; `select`'s popup and `form` have their own.
        // `pagination` has no labels to seek and declares no text bind at all.
        assert_eq!(seeks, vec!["collection", "table", "tree", "select", "form"]);
        assert_eq!(types, vec!["field"]);
    }

    /// **The fifth chord leak, and it is the one with nowhere else to go.**
    ///
    /// `popup_body`'s `Refusal` read `k.code` alone, so `Ctrl+Enter` committed and `Alt+Esc`
    /// dismissed — and unlike the owner's four, this loop runs *inside* a trapless overlay, where
    /// the application has no other reader for the key it just lost.
    #[test]
    fn an_open_popup_does_not_eat_a_chord_built_on_its_own_two_keys() {
        for c in [
            Chord::new(Code::Enter).ctrl(),
            Chord::new(Code::Enter).alt(),
            Chord::new(Code::Escape).ctrl(),
            Chord::new(Code::Escape).alt(),
        ] {
            assert!(
                !(contract("select").live)(Trigger::Key(c)),
                "an open popup still takes {:?}",
                Trigger::Key(c).spell()
            );
        }
    }
}
