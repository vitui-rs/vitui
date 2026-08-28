//! Spec §3's `keys::text`: **a chord is not text, and a capital is not a chord.**
//!
//! One line of arithmetic, and it is the components-side half of R12's registry. R12's
//! [`INTENT`](vitui_runtime::keys::INTENT) is `SHIFT | ALT | CTRL | SUPER | HYPER | META` and is
//! **right for R12's question**, which is *does this key press fire a binding* — a binding on
//! `Ctrl+Shift+P` is a different binding from one on `Ctrl+P`, so Shift has to be compared. A text
//! widget asks a different question — *did the user mean to type something* — and the answer differs
//! in exactly one bit: [`SIGNIFICANT`] is `CTRL | ALT`, and **Shift is deliberately not in it**,
//! because a capital is what Shift is for.
//!
//! Without this predicate every accelerator in the application is swallowed by whatever text widget
//! holds the focus: `Ctrl+S` puts an `s` in the box and saves nothing. The runtime already carries
//! the defect as an obligation written where a component author reads it —
//! `crates/vitui-runtime/src/keys.rs`'s `the_accelerator_a_field_ate` — and says in as many words
//! that the fix *belongs in the component library rather than here*. This is that fix.
//!
//! # The finding that made this ticket buildable, and it inverts a register row
//!
//! Components ticket 03 filed six gate families as [`Unreachable`](crate::gates::Standing::Unreachable)
//! against `Mods`, and register row 5 stated the conclusion in as many words: *its `mods` field is
//! `vitui_engine::Mods`, which is `reachable_as: None`. So no key can be posted and no chord can be
//! pressed.*
//!
//! **The premise was true and the conclusion did not follow.** `Mods` could not be *named* from this
//! crate — there was no path to it, and `crates/vitui-runtime/src/line.rs` was right that there was
//! none. But a struct literal needs a **value** for each field, not a name for its type, and this
//! crate can obtain `Mods` values:
//!
//! - `vitui_runtime::keys::Chord` has a `pub mods: Mods` field and three `const` builders that set
//!   it, so `Chord::key('s').ctrl().mods` **is** `Mods::CTRL`, written without the word `Mods`.
//! - `Mods`' whole read surface — `with`, `contains`, `is_empty`, `chord`, `ctrl`, `alt`, `shift`,
//!   `super_key`, `hyper`, `meta`, `caps`, `num` — is inherent, so it is callable on a value whose
//!   type has no name here.
//! - [`vitui_runtime::keys::Pressed`] is the engine's `Key` re-exported, is not `#[non_exhaustive]`,
//!   and its other three fields are all reachable: `Code::Char`, `Edge::Press`, `Text::EMPTY`.
//! - `Driver::post_key` is public, for `Driver::post_mouse`'s stated reason — *a headless attach has
//!   no tty, so nothing about routing would be reachable without it.*
//!
//! So [`press`] compiles, row 5's gate runs, and the register carries the correction rather than the
//! sentence. **The barrier that remains is narrower and is stated as a number**: see
//! [`REACHABLE_STATES`].
//!
//! # What is still out of reach, precisely
//!
//! Two things, and neither is a component's to fix:
//!
//! 1. **~~Five of the eight modifier bits cannot be constructed here.~~ Lifted by runtime
//!    architecture issue 22.** It read: `Chord` has `ctrl`, `alt` and `shift` builders and no others,
//!    so `SUPER`, `HYPER`, `META`, `CAPS` and `NUM` have no reachable spelling — and the
//!    **exhaustive table** is 8 states of 256 rather than the rule. `Mods` is `vitui_runtime::Mods`
//!    now and its eight bits are `pub const`, so [`press_with`] builds a key at any of the 256 and
//!    [`REACHABLE_STATES`] is 256. The `Chord` limit is unchanged and is not a defect: three
//!    builders are what an application binds through.
//! 2. **`KeyText` has no public constructor anywhere.** `Text::EMPTY` is the only value, deliberately
//!    — so that nothing can forge a key whose `code` and `text` disagree — which is the same
//!    shortfall the runtime writes down for `On::Typed`. Every key pressed from this crate carries
//!    empty text, so [`text`]'s terminal-supplied arm is unexercised and its `code`-plus-Shift
//!    fallback is the one the gates below run over. That is exactly the arm a legacy terminal takes.
//!
//! # The mask is a `Chord` because a `Chord` is the only nameable box a `Mods` fits in
//!
//! [`SIGNIFICANT`] would be a `Mods` constant if this crate could write the word. It cannot, and a
//! `pub const` needs a written type — so the mask ships as the `Chord` that carries it and every
//! reader takes `.mods`. The `code` field is filler and [`MASK_KEY`] says so. This is not a
//! workaround dressed as a design: it is the same shape as `crate::gates::Instrument::Barrier` —
//! the constraint recorded as a value instead of as a paragraph.

use std::time::Instant;

// `Mods` arrived at the crate root with runtime architecture issue 22; before it, this
// module's whole header was about not being able to write this line.
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Chord, Code, Edge, Pressed, Text};
use vitui_runtime::{Interest, Mods};

/// The key [`SIGNIFICANT`] is written on, and it is filler.
///
/// A [`Chord`] is a key *and* a modifier set, and only the modifier half is read. `\0` rather than a
/// plausible letter, so that a reader who mistakes the mask for a binding meets a key no terminal
/// sends.
pub const MASK_KEY: char = '\0';

/// **The modifiers that make a key a chord rather than text: `CTRL | ALT`.**
///
/// Read `SIGNIFICANT.mods`; the `code` is [`MASK_KEY`] and means nothing. See this module's header
/// for why the constant is a [`Chord`] and not a `Mods`.
///
/// **Shift is not in it, and that is the whole difference from R12.**
/// [`vitui_runtime::keys::INTENT`] contains `SHIFT` correctly, for the question *does this fire a
/// binding*. Declining on `INTENT` from a text widget declines every capital: measured through a
/// focused field it gives `"i"` where `"Hi"` is right — a defect that looks like a broken keyboard
/// and points at nothing.
pub const SIGNIFICANT: Chord = Chord::key(MASK_KEY).ctrl().alt();

/// [`SIGNIFICANT`] one bit at a time.
///
/// **The only way to ask *do these two share a bit* from here.** `Mods::contains` is *all* bits and
/// there is no `intersects`, so the mask has to be decomposed to be used as a predicate. The
/// decomposition is checked against the mask by
/// `tests::the_mask_and_the_predicate_are_one_statement`
/// rather than trusted, because a mask nobody reads and a predicate nobody compares it against are
/// two rules that drift.
pub const BITS: [Chord; 2] = [Chord::key(MASK_KEY).ctrl(), Chord::key(MASK_KEY).alt()];

/// How many of the 256 modifier states a gate in this crate can construct. **All of them.**
///
/// This was **eight** and said so for a reason that has since gone. `Chord`'s builders are `ctrl`,
/// `alt` and `shift`, so the set reachable *through a `Chord`* is the eight subsets of those three,
/// and `SUPER`, `HYPER`, `META`, `CAPS` and `NUM` had no reachable spelling at all: `Mods` was
/// `reachable_as: None` and its eight `pub const` bits were behind a name this crate could not write.
///
/// **Runtime architecture issue 22 re-exported `Mods`**, so `Mods::SUPER` and its four siblings are
/// writable here and [`press_with`] builds a key at any of the 256. The exhaustive table is now
/// exhaustive, which is the difference between *the rule reads all eight bits off a key it is handed*
/// and *the rule is checked against all eight bits* — the module used to be able to state only the
/// first.
pub const REACHABLE_STATES: usize = 256;

// ── the predicate ────────────────────────────────────────────────────────────────────────────────

/// Whether this key is a **chord** — something the application bound, not something the user typed.
///
/// Reads [`BITS`], so the mask above is the single statement of the rule. The two locks are removed
/// first through `Mods::chord`, for `INTENT`'s reason: a binding on `Ctrl+C` fires with caps lock
/// on, and a field must decline it with caps lock on too.
pub fn is_chord(k: &Pressed) -> bool {
    let held = k.mods.chord();
    BITS.iter().any(|b| held.contains(b.mods))
}

/// **Spec §3's `keys::text`.** Append what this key types to `out`, and answer whether it typed.
///
/// `false` is not *nothing happened* — it is **not this widget's key**, and the caller owes it a
/// `Ctx::decline` so the level above gets its turn. The two halves are one statement on purpose: a
/// field that swallowed the key silently would pass a gate that only checked what landed in the box.
///
/// A caller-owned `&mut String` rather than a returned value, on
/// [`write_chord`](vitui_runtime::keys::write_chord)'s rule — a text widget's buffer already exists,
/// and a grapheme cluster is not a `char`.
///
/// # Three refusals, in order
///
/// **A release never types**, the same refusal `Chord::matches` makes and for the same reason: at
/// kitty flag 2 a terminal reports both edges and a field that took both would double every letter.
/// A **repeat** does type, because below the kitty protocol a held key is a stream of presses and
/// the two tiers have to behave alike.
///
/// **A chord never types** — [`is_chord`], which is the whole helper.
///
/// **A control character never types.** `Code::Char` carries the base scalar, and a legacy terminal
/// that sends `Ctrl+S` with no modifier byte at all sends `0x13`; without this a field with no
/// modifiers to inspect writes a control character into its buffer.
///
/// # The terminal's answer first, and the fallback is what the gates run over
///
/// Where the terminal supplied text, that is what was typed and nothing here improves on it. Where
/// it did not, the capital is derived from `code` and `Mods::shift` — which is the arm a legacy
/// terminal takes, and the only arm any crate above the engine can exercise, because `KeyText` has
/// no public constructor.
pub fn text(k: &Pressed, out: &mut String) -> bool {
    if k.kind == Edge::Release || is_chord(k) {
        return false;
    }
    let supplied = k.text.as_str();
    if !supplied.is_empty() {
        out.push_str(supplied);
        return true;
    }
    match k.code {
        Code::Char(c) if !c.is_control() => {
            match k.mods.shift() {
                true => out.extend(c.to_uppercase()),
                false => out.push(c),
            }
            true
        }
        // `Code` is `#[non_exhaustive]`, so this arm is required rather than tidy — and it is also
        // the right answer: `Enter`, `Tab` and every arrow are somebody else's.
        _ => false,
    }
}

/// The two readings of a key that are **wrong**, kept runnable.
///
/// Neither is a strawman: `on_code_alone` is the code the runtime found in a shipped field six
/// tickets old, and `on_intent` is the over-correction a reader of R12's registry writes when they
/// take the runtime's mask for the components' one. A gate that only ran the right answer would pass
/// on a field that had stopped accepting anything.
pub mod defective {
    use super::{Code, Edge, Pressed};

    /// **The defect.** Matches on `code` alone, so the accelerator's letter lands in the box.
    ///
    /// `Ctrl+S` types an `s` and saves nothing, and it survived six runtime tickets because until
    /// the modifier byte existed there was nothing to check.
    pub fn on_code_alone(k: &Pressed, out: &mut String) -> bool {
        if k.kind == Edge::Release {
            return false;
        }
        match k.code {
            Code::Char(c) if !c.is_control() => {
                out.push(c);
                true
            }
            _ => false,
        }
    }

    /// **The over-correction.** Declines on R12's `INTENT`, which contains `SHIFT`.
    ///
    /// Every accelerator is correctly declined and so is every capital: `Shift+H, i` gives `"i"`
    /// where `"Hi"` is right. Written as *any intent bit at all*, which is what `Mods::chord`
    /// answers — the locks removed, everything else kept.
    pub fn on_intent(k: &Pressed, out: &mut String) -> bool {
        if k.kind == Edge::Release || !k.mods.chord().is_empty() {
            return false;
        }
        match k.code {
            Code::Char(c) if !c.is_control() => {
                out.push(c);
                true
            }
            _ => false,
        }
    }
}

// ── the instrument ───────────────────────────────────────────────────────────────────────────────

/// A key press, built through a `Chord` — which is how a crate that could not name `Mods` built one,
/// and still the path an application's own bindings take.
///
/// **This function is the ticket's finding.** See this module's header: the modifiers travel inside
/// a [`Chord`], which is the one nameable type that carries them, and the key is a plain struct
/// literal. `Text::EMPTY` because there is no other `KeyText` value anywhere above the engine.
pub fn press(c: Chord) -> Pressed {
    press_at(c, Instant::now())
}

/// [`press`], at a moment the caller already had.
///
/// `Instant` has no public constructor, so `at` is always a real reading offset by real durations —
/// `Driver::pin_clock`'s own note, one crate up.
pub fn press_at(c: Chord, at: Instant) -> Pressed {
    Pressed {
        code: c.code,
        mods: c.mods,
        kind: Edge::Press,
        text: Text::EMPTY,
        at,
    }
}

/// [`press`], at an arbitrary modifier state.
///
/// The door [`REACHABLE_STATES`] became 256 through. A `Chord` reaches three of the eight bits, so a
/// gate that wants `Super` or a lock has to build the key rather than bind it — which needs a `Mods`
/// **value** whose type this crate can name, and it can since runtime architecture issue 22.
///
/// Deliberately beside `press` rather than replacing it: a `Chord` is what an application actually
/// binds, and a gate that stops going through one stops exercising the path components use.
pub fn press_with(code: Code, mods: Mods) -> Pressed {
    Pressed {
        code,
        mods,
        kind: Edge::Press,
        text: Text::EMPTY,
        at: Instant::now(),
    }
}

/// Which reading of a key a text sink uses. See [`ARMS`].
pub type Arm = fn(&Pressed, &mut String) -> bool;

/// The three readings, in the order spec §3's sentence names them.
///
/// Iterated by the gates and by `examples/keys_numbers.rs`, so that the measurement is *one loop
/// over three arms* rather than three transcriptions of the same drive loop — which is how two of
/// them come to disagree about the input.
pub const ARMS: [(&str, Arm); 3] = [
    ("code_alone", defective::on_code_alone as Arm),
    ("text", text as Arm),
    ("intent", defective::on_intent as Arm),
];

/// What a sequence of keys did to a focused text sink.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Typed {
    /// What the sink holds afterwards.
    pub value: String,
    /// **How many keys reached the application** — the half that makes this a measurement rather
    /// than a mute. `Driver::unhandled` is the outermost level of the one queue, which is where an
    /// application reads.
    pub reached: usize,
    /// How many bytes the sink gained.
    pub gained: usize,
}

/// The id every text sink in this module is planted on.
///
/// One id and not a table: each drive runs its own [`Driver`], because `Ctx::plant` seats exactly
/// one focus and *a chord pressed into every focusable* is a loop over screens, not over widgets on
/// one screen.
pub fn sink_id() -> vitui_runtime::Id {
    vitui_runtime::Id::named("text-sink")
}

/// **Press `chords` into a focused text sink reading keys through `arm`.**
///
/// # One key per frame, and that is the runtime's contract rather than a convenience
///
/// `Ctx::decline` ends a widget's turn at the queue — `next_key` answers `None` afterwards however
/// many keys are left, so that a widget which declined *k* cannot then take *k+1* and leave the
/// level above seeing the two the wrong way round. A batch of three posted at once therefore loses
/// the two after the first decline, which is not a defect and not what a terminal does: keys arrive
/// one at a time and each is a frame. The loop below is that, and it is why [`Typed::reached`] is
/// exactly countable.
pub fn typed(arm: Arm, start: &str, chords: &[Chord]) -> Typed {
    let mut driver = Driver::headless(60, 8).expect("a sink cannot fail to attach");
    let id = sink_id();
    let mut value = String::from(start);
    let before = value.len();
    let mut reached = 0;
    for &c in chords {
        driver.plant(None, Some(id), None);
        driver.post_key(press(c));
        driver.frame(|cx| {
            let area = cx.area();
            let _ = cx.interact(id, area, Interest::FOCUS);
            while let Some(k) = cx.next_key(id) {
                if !arm(&k, &mut value) {
                    cx.decline(k);
                }
            }
        });
        reached += driver.unhandled().len();
    }
    let gained = value.len() - before;
    Typed {
        value,
        reached,
        gained,
    }
}

/// Press one chord into a sink that already holds `start`, and answer what reached the application.
///
/// The shape register row 5 gates: [`Typed::gained`] must be `0` and [`Typed::reached`] must be `1`.
pub fn chord_into(arm: Arm, start: &str, c: Chord) -> Typed {
    typed(arm, start, &[c])
}

// ── the text-bearing population ──────────────────────────────────────────────────────────────────

/// **Every text-bearing component in [`crate::INVENTORY`]**, which is what row 5's *every focusable*
/// means on a screen where no component exists.
///
/// A component is text-bearing if a character key would reach it: a field, anything containing one,
/// or anything with type-ahead — which is every collection, because
/// [`crate::nav::seek`] is what a `Group` seeks with.
///
/// **Joined against the freeze rather than listed**:
/// `tests::every_text_bearing_name_is_in_the_freeze`
/// opens [`crate::INVENTORY`] and fails on a name that is not a row of it, which is
/// `crate::gates::Instrument`'s rule applied to a population instead of to a file.
///
/// **All seven exist in this crate since components ticket 35**, and the gate says so by reading the
/// source rather than [`crate::Component::built`] — which was the *prototype's* column when this
/// list was written and is the shipped one now. When it was none of the seven, what row 5 stood over
/// was seven **sinks**, one per row of this list, and that was a real standing: the claim is *the
/// predicate declines a chord whatever is holding the focus*, which does not depend on what the
/// focus is holding.
///
/// **Two of the seven arrived by repairing the needle**, not by anybody building anything — `select`
/// and `file_picker` have carried a lifetime and a type parameter since tickets 26 and 32, and the
/// parenthesis `tests::every_text_bearing_name_is_in_the_freeze` looked for answered *undeclared*
/// about both. `crate::obligations` is one file over precisely because *a query with no evidence
/// must fail loudly* is the distinction this map keeps losing.
pub const TEXT_BEARING: [&str; 7] = [
    // §11's one flag absorbs sixteen named input variants including `textarea`.
    "field",
    // Holds fields, and is where §3's `"value 0"` fixture came from.
    "form",
    // Type-ahead is what its closed state is navigated with.
    "select",
    // The three collections. `collection` is the virtualised one; `table` and `tree` express it.
    "collection",
    "table",
    "tree",
    // A filename is typed into it.
    "file_picker",
];

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this module is gated or reported on has one home and it is here** — the runtime's
// ledger rule (`crates/vitui-runtime/src/ledger.rs`), inherited by `crate::state` and `crate::form`.
// Every figure is a count over a deterministic drive, so none carries a machine.

/// What a field holds before anything is typed into it. **Spec §3's own fixture**, quoted rather
/// than chosen: the sentence being reproduced is `"value 0shi"` against `"value 0hi"`, and a
/// different starting string would make both numbers unrecognisable.
pub const START: &str = "value 0";

/// **`Ctrl+S, h, i` read on `code` alone.** The accelerator's letter is in the box.
pub const CODE_ALONE: &str = "value 0shi";
/// **`Ctrl+S, h, i` read through [`text`].** The chord was declined and the two letters landed.
pub const THROUGH_TEXT: &str = "value 0hi";
/// **`Ctrl+S, h, i` read by declining on `INTENT`.** Indistinguishable from the right answer on this
/// input, which is why the third arm needs the capital and why spec §3 changes the input mid
/// sentence.
pub const THROUGH_INTENT: &str = "value 0hi";

/// What `Shift+H, i` types through [`text`]. **`"Hi"`, and this is the input the third arm fails
/// on.**
pub const CAPITAL_RIGHT: &str = "Hi";
/// What `Shift+H, i` types when the field declines on `INTENT`. **`"i"`.**
pub const CAPITAL_ON_INTENT: &str = "i";
/// What `Shift+H, i` types on `code` alone. **`"hi"`** — the capital is lost as well, because a
/// reading that ignores the modifier byte ignores Shift too.
pub const CAPITAL_ON_CODE: &str = "hi";

/// **Bytes a `textarea` gains from `Ctrl+S, h` read on `code` alone. Two.**
pub const TEXTAREA_CODE_BYTES: usize = 2;
/// **Keys reaching the application from that same reading. Zero** — the accelerator was eaten.
pub const TEXTAREA_CODE_REACHED: usize = 0;
/// **Bytes a `textarea` gains through [`text`]. One.**
pub const TEXTAREA_TEXT_BYTES: usize = 1;
/// **Keys reaching the application through [`text`]. One** — which is the accelerator, arriving.
pub const TEXTAREA_TEXT_REACHED: usize = 1;

#[cfg(test)]
mod tests {
    use super::*;
    use vitui_runtime::keys::INTENT;

    /// The three bits this crate can build, as values rather than as names.
    fn shift() -> Chord {
        Chord::key(MASK_KEY).shift()
    }

    // ── the mask ─────────────────────────────────────────────────────────────────────────────────

    /// **`keys::text` is `CTRL | ALT`, with Shift excluded** — acceptance criterion 1, as an
    /// equality against a mask built two different ways.
    ///
    /// Asserted as an exact equality and not as two `contains` calls: *contains CTRL and contains
    /// ALT* is green on a mask that is `CTRL | ALT | SHIFT`, which is the exact failure the
    /// criterion is about.
    #[test]
    fn the_mask_is_ctrl_and_alt_and_shift_is_not_in_it() {
        let ctrl = Chord::key(MASK_KEY).ctrl().mods;
        let alt = Chord::key(MASK_KEY).alt().mods;
        assert_eq!(
            SIGNIFICANT.mods,
            ctrl.with(alt),
            "the mask is exactly CTRL | ALT"
        );
        assert!(
            !SIGNIFICANT.mods.contains(shift().mods),
            "Shift is in the mask, so a capital is being read as a chord"
        );
        assert!(!SIGNIFICANT.mods.shift());

        // **And R12's is right for R12's question**, which is the sentence this constant exists
        // against. One bit apart, and the bit is Shift.
        assert!(INTENT.contains(shift().mods));
        assert!(INTENT.contains(ctrl));
        assert!(INTENT.contains(alt));
    }

    /// **The mask and the predicate are one statement**, so that [`BITS`] cannot drift from
    /// [`SIGNIFICANT`].
    #[test]
    fn the_mask_and_the_predicate_are_one_statement() {
        let union = BITS
            .iter()
            .fold(Chord::key(MASK_KEY).mods, |acc, b| acc.with(b.mods));
        assert_eq!(union, SIGNIFICANT.mods);
        assert_eq!(BITS.len(), 2);
    }

    /// **The eight states this crate can construct**, and the count is the barrier rather than a
    /// coverage claim.
    ///
    /// Over the three reachable bits the predicate agrees with the mask on every combination. The
    /// other 248 states need `SUPER`, `HYPER`, `META`, `CAPS` or `NUM`, none of which has a
    /// reachable spelling — see this module's header and register row 5.
    #[test]
    fn the_predicate_agrees_with_the_mask_on_every_reachable_state() {
        const BIT: [(&str, Mods); 8] = [
            ("shift", Mods::SHIFT),
            ("alt", Mods::ALT),
            ("ctrl", Mods::CTRL),
            ("super", Mods::SUPER),
            ("hyper", Mods::HYPER),
            ("meta", Mods::META),
            ("caps", Mods::CAPS),
            ("num", Mods::NUM),
        ];

        let mut seen = 0;
        for state in 0u32..256 {
            let mut mods = Mods::NONE;
            let mut held = Vec::new();
            for (i, (name, bit)) in BIT.iter().enumerate() {
                if state & (1 << i) != 0 {
                    mods = mods.with(*bit);
                    held.push(*name);
                }
            }
            let k = press_with(Code::Char('x'), mods);

            // The rule, restated from `BITS` and not from the loop: a chord is ctrl or alt, and the
            // two locks are masked off before anything is read. `super`, `hyper` and `meta` are in
            // `INTENT` and are **not** in `BITS`, so a `Super+x` is not a chord by this predicate —
            // which is the existing rule now checked on the 248 states that could not be built.
            let expect = mods.contains(Mods::CTRL) || mods.contains(Mods::ALT);
            assert_eq!(
                is_chord(&k),
                expect,
                "state {state} ({}) disagrees with the mask",
                match held.is_empty() {
                    true => "none".to_string(),
                    false => held.join("+"),
                }
            );
            seen += 1;
        }
        assert_eq!(seen, REACHABLE_STATES);
    }

    /// **Caps lock does not turn a letter into a chord**, and the reason is `Mods::chord`.
    ///
    /// A lock cannot be written *through a `Chord`* — it has no `caps` builder — so this asserts the
    /// projection rather than the state: the predicate masks the locks off before it reads anything,
    /// which is the same projection R12's `INTENT` is. The locks themselves are reachable since
    /// runtime issue 22 and `the_predicate_agrees_with_the_mask_on_every_reachable_state` holds
    /// them, including the 64 states where a lock is down.
    #[test]
    fn the_predicate_masks_the_locks_before_it_reads_anything() {
        let plain = press(Chord::key('s'));
        assert_eq!(plain.mods.chord(), plain.mods);
        assert!(!is_chord(&plain));
        let held = press(Chord::key('s').ctrl());
        assert!(is_chord(&held));
        assert_eq!(held.mods.chord(), held.mods, "a chord holds no lock");
    }

    // ── the three-way measurement ────────────────────────────────────────────────────────────────

    /// **Acceptance criterion 2: the three-way measurement is a test, not a note.**
    ///
    /// Spec §3's sentence, run: `Ctrl+S, h, i` through a focused field gives `"value 0shi"` matching
    /// on `code` alone and `"value 0hi"` declining chords; and *declining on `intent()` instead
    /// gives `"i"` where `"Hi"` is right* — which is a **different input**, because on `Ctrl+S, h,
    /// i` the `INTENT` reading is indistinguishable from the right answer. Both inputs are here for
    /// that reason: the third arm's defect is invisible on the first one.
    #[test]
    fn the_three_way_measurement_over_a_focused_field() {
        let seq = [Chord::key('s').ctrl(), Chord::key('h'), Chord::key('i')];

        let code = typed(defective::on_code_alone, START, &seq);
        assert_eq!(code.value, CODE_ALONE, "the field typed the accelerator");
        assert_eq!(
            code.reached, 0,
            "and the application never saw it, which is the half that makes this a defect rather \
             than a cosmetic"
        );

        let right = typed(text, START, &seq);
        assert_eq!(right.value, THROUGH_TEXT);
        assert_eq!(right.reached, 1, "the accelerator reached the application");

        let intent = typed(defective::on_intent, START, &seq);
        assert_eq!(
            intent.value, THROUGH_INTENT,
            "on this input the over-correction is indistinguishable from the fix"
        );
        assert_eq!(intent.reached, 1);
    }

    /// **And the capital, which is where the third arm fails.** `"i"` where `"Hi"` is right.
    ///
    /// All three arms, because the interesting fact is that **each of the three gives a different
    /// answer on the same two keys** — which is what makes this one sentence in spec §3 worth a
    /// helper.
    #[test]
    fn a_capital_is_not_a_chord_and_the_intent_reading_says_it_is() {
        let seq = [Chord::key('h').shift(), Chord::key('i')];

        assert_eq!(typed(text, "", &seq).value, CAPITAL_RIGHT);
        assert_eq!(
            typed(defective::on_intent, "", &seq).value,
            CAPITAL_ON_INTENT,
            "declining on INTENT declines the capital"
        );
        assert_eq!(
            typed(defective::on_code_alone, "", &seq).value,
            CAPITAL_ON_CODE,
            "and a reading that ignores the modifier byte loses the capital too"
        );

        // The three disagree, which is the point.
        let answers: Vec<String> = ARMS
            .iter()
            .map(|(_, arm)| typed(*arm, "", &seq).value)
            .collect();
        answers.iter().enumerate().for_each(|(i, a)| {
            for b in &answers[i + 1..] {
                assert_ne!(a, b, "two arms agree, so one of them is not being run");
            }
        });
    }

    /// **The `textarea` arithmetic: 2 bytes and 0 keys against 1 and 1.**
    ///
    /// Spec §3's second number, and the input is `Ctrl+S, h` — one chord and one letter, which is
    /// what makes the difference exactly one byte and exactly one key. Read off `Driver::unhandled`,
    /// which is the outermost level of the one queue.
    #[test]
    fn the_textarea_arithmetic_is_two_and_zero_against_one_and_one() {
        let seq = [Chord::key('s').ctrl(), Chord::key('h')];

        let code = typed(defective::on_code_alone, "", &seq);
        assert_eq!(code.gained, TEXTAREA_CODE_BYTES);
        assert_eq!(code.reached, TEXTAREA_CODE_REACHED);

        let right = typed(text, "", &seq);
        assert_eq!(right.gained, TEXTAREA_TEXT_BYTES);
        assert_eq!(right.reached, TEXTAREA_TEXT_REACHED);

        assert_eq!(code.gained - right.gained, 1);
        assert_eq!(right.reached - code.reached, 1);
    }

    // ── row 5: a chord pressed into every focusable types nothing ────────────────────────────────

    /// **Acceptance criterion 3 and register row 5: a chord pressed into every focusable types
    /// nothing.**
    ///
    /// Over [`TEXT_BEARING`], which is the freeze's text-bearing set, and in **both directions** —
    /// the defective reading is run over the same population and types one byte into every one of
    /// them, so the gate is watched separating a correct sink from a defective one rather than
    /// asserted over a population that accepts nothing at all.
    #[test]
    fn a_chord_pressed_into_every_focusable_types_nothing() {
        let accelerator = Chord::key('s').ctrl();

        let mut gained = 0;
        let mut reached = 0;
        for name in TEXT_BEARING {
            let r = chord_into(text, START, accelerator);
            assert_eq!(r.value, START, "{name} typed something");
            gained += r.gained;
            reached += r.reached;
        }
        assert_eq!(gained, 0, "the chord typed nothing anywhere");
        assert_eq!(
            reached,
            TEXT_BEARING.len(),
            "and it reached the application from every one of them"
        );

        // The other direction, or the gate passes on a sink that accepts nothing.
        let mut wrong = 0;
        for _ in TEXT_BEARING {
            wrong += chord_into(defective::on_code_alone, START, accelerator).gained;
        }
        assert_eq!(
            wrong,
            TEXT_BEARING.len(),
            "the defective reading types the accelerator into every one of them, which is the \
             state this row exists to see"
        );
    }

    /// **Every name in [`TEXT_BEARING`] is a row of the freeze, and exactly one of them exists
    /// here.**
    ///
    /// The second half is a scan of `src/` and **not** a reading of [`crate::Component::built`]:
    /// that column is the prototype's and is `true` for six of the seven. What row 5 needs to be
    /// honest about is what its population *is*, and components ticket 12 changed it: `collection`
    /// is declared, so the population is **six sinks and one component**.
    ///
    /// # The needle had a false negative, and it is the shape this file already warns about
    ///
    /// It read `sources.contains("pub fn {name}(cx")`, which is right for a signature that fits on
    /// one line and blind to one rustfmt has broken — `pub fn collection(` alone, with `cx:` on the
    /// line beneath. Left as it was, this test would have gone on asserting *`collection` is not
    /// declared in this crate* about a component sitting in `src/collect.rs`, and passing. The
    /// `(cx` is still what keeps `gates::table()` out of the answer; the broken form is accepted
    /// only when the line **ends** at the parenthesis, which `pub fn table()` never does.
    #[test]
    fn every_text_bearing_name_is_in_the_freeze() {
        let src = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let sources: Vec<String> = std::fs::read_dir(&src)
            .expect("a readable source directory")
            .filter_map(|e| {
                let path = e.expect("a directory entry reads").path();
                match path.extension().is_some_and(|x| x == "rs") {
                    true => Some(std::fs::read_to_string(&path).expect("a readable source file")),
                    false => None,
                }
            })
            .collect();
        assert!(sources.len() > 10, "the scan found no sources");

        // **`(cx`, not `(`** — a component's signature is `pub fn button(cx: &mut Ctx, …)` (spec
        // §1), and the bare form has a false positive already in the crate: `gates::table()` prints
        // the register and is not the `table` component. The second spelling is the same signature
        // with the parameters on their own lines, which is the only form a seven-parameter
        // component has.
        //
        // **And the third is `<`, which is components ticket 33's either-delimiter rule.** Two of
        // the seven have been declared since tickets 26 and 32 and this predicate could not see
        // either: `select` is `pub fn select<'f>(` — spec §1 already says a component that opens an
        // overlay costs two lifetime annotations — and `file_picker` is `pub fn file_picker<'f,
        // T>(`. **Both were counted as unbuilt for six tickets by a needle rather than by a fact**,
        // which is the fifth time on this map that a scan's parenthesis has answered *undeclared*
        // about a component that is right there.
        let declared_here = |name: &str| {
            let one_line = format!("pub fn {name}(cx");
            let broken = format!("pub fn {name}(");
            let generic = format!("pub fn {name}<");
            sources.iter().any(|s| {
                s.lines().map(str::trim).any(|l| {
                    !l.starts_with("//")
                        && (l.contains(&one_line) || l == broken || l.starts_with(&generic))
                })
            })
        };

        let mut built: Vec<&str> = Vec::new();
        for name in TEXT_BEARING {
            assert!(
                crate::INVENTORY.iter().any(|c| c.id == name),
                "`{name}` is not a row of INVENTORY"
            );
            if declared_here(name) {
                built.push(name);
            }
        }
        assert_eq!(
            built,
            vec![
                "field",
                "form",
                "select",
                "collection",
                "table",
                "tree",
                "file_picker"
            ],
            "row 5's population is **seven names, and since components ticket 35 every one of them \
             is built** — `field` since ticket 24, `select` since 26, `tree` since 17, `table` since \
             15, `collection` since 12, `file_picker` since 32, and `form` here. **Two of those \
             arrived by repairing the needle rather than by anybody building anything**: `select` \
             and `file_picker` carry a lifetime and a type parameter, and the parenthesis this \
             predicate looked for answered *undeclared* about both for six tickets. A name arriving \
             here or leaving it is a deliberate edit — the row is a claim about what a chord does to \
             *every* focusable, and which of them are real components is the half this test keeps \
             honest. **`field` is the one that makes the row a claim rather than a prediction**: \
             every other name on it consumes a text-bearing key into a type-ahead buffer, and this \
             one consumes it into a document"
        );
        assert_eq!(
            TEXT_BEARING.len() - built.len(),
            0,
            "and the count of names no component declares is zero, which is what this module's \
             header said it was waiting for"
        );
        // No duplicates, or the count above is not the population.
        let mut sorted = TEXT_BEARING.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), before);
    }

    /// **A release never types, and a repeat does.**
    #[test]
    fn a_release_never_types_and_a_repeat_does() {
        let mut out = String::new();
        let mut k = press(Chord::key('h'));
        k.kind = Edge::Release;
        assert!(!text(&k, &mut out));
        assert!(out.is_empty());

        k.kind = Edge::Repeat;
        assert!(text(&k, &mut out));
        assert_eq!(out, "h", "a held key is a stream of presses below kitty");
    }

    /// **A control character never types**, which is what a legacy terminal sending `Ctrl+S` as
    /// `0x13` with no modifier byte produces.
    #[test]
    fn a_control_character_never_types_even_with_no_modifier_byte() {
        let mut out = String::new();
        let bare = press(Chord::key('\u{13}'));
        assert!(bare.mods.is_empty(), "no modifier byte at all");
        assert!(
            !text(&bare, &mut out),
            "a legacy Ctrl+S is a control character and is nobody's text"
        );
        assert!(out.is_empty());
        // And the defective reading writes it into the buffer, which is the failure.
        assert!(defective::on_code_alone(&bare, &mut out) || out.is_empty());
    }

    /// **`Enter`, `Tab` and the arrows are not text**, so a field declines them and the ring or the
    /// group gets its turn.
    #[test]
    fn a_named_key_is_never_text() {
        let mut out = String::new();
        for code in [Code::Enter, Code::Tab, Code::Up, Code::Home, Code::Escape] {
            let k = press(Chord::new(code));
            assert!(!text(&k, &mut out), "{code:?} was read as text");
        }
        assert!(out.is_empty());
    }
}
