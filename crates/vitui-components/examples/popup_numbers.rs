//! components tickets 25 and 26 — the overlay family's screen, its two components, and §12's table
//! beside what it measured.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every count below is `src/popup.rs`'s test
//! module and `tests/popup.rs`, and each figure has its single home in one of those two files.
//!
//! It prints nine things:
//!
//! 1. **§12's table**, its column beside the measured one, row for row.
//! 2. **The four steady deltas** — a dropdown, a submenu, a modal, its scrim.
//! 3. **The opening frame**, which is a cliff and is allowed, with the cliff *counted*.
//! 4. **The scrim's three spellings**, and the popup's two drawing orders.
//! 5. **The family's two axes**, as the counts §12 states them in.
//! 6. **The ring, the trap and the three answers to where a closing modal sends the keyboard.**
//! 7. **The family as three kinds on two axes**, and the four spellings §12 refuses — the size, the
//!    gutter, the blur and the offset (components 26).
//! 8. **What does not reproduce**, said out loud rather than engineered away.
//! 9. **Where scene 14 stands**, which since components 26 is *up*.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::{Allocations, Counter, Reading};
use vitui_components::input::Sizing;
use vitui_components::overlay::{self, Kind};
use vitui_components::popup::{self, Config, Dismiss, ScrimSpelling};
use vitui_components::scenes;
use vitui_components::scroll::{Hide, MAX_PASSES, decide};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// The steady window §12 states: *a minimum of 60 steady frames*.
const FRAMES: u32 = 60;

/// §12's own table, as `(configuration, µs, marked, regions, stops, content layers, allocations)`.
const REMEMBERED: [(Config, f64, u64, usize, usize, usize, u64); 5] = [
    (Config::Nothing, 33.96, 0, 317, 316, 0, 0),
    (Config::OneSelect, 35.29, 0, 319, 317, 2, 0),
    (Config::MenuAndSubmenu, 35.12, 0, 323, 322, 4, 0),
    (Config::ModalAndScrim, 34.83, 0, 319, 318, 3, 0),
    (Config::FillFirst, 36.58, 107, 319, 317, 2, 0),
];

fn main() {
    println!(
        "components ticket 25 — 312 chips, two selects, a menu bar, and five configurations\n"
    );

    // ── 1. §12's table ───────────────────────────────────────────────────────────────────────────
    println!(
        "report  the overlay family's screen at {}x{}, `Compact`, over a minimum of {FRAMES} steady frames:",
        popup::W,
        popup::H
    );
    println!(
        "  {:<30}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}  {:>9}",
        "", "us", "marked", "regions", "stops", "layers", "allocs"
    );
    let table: Vec<Config> = REMEMBERED.iter().map(|r| r.0).collect();
    let timed = popup::costs(&table, FRAMES);
    for ((config, us, marked, regions, stops, layers, allocs), took) in
        REMEMBERED.into_iter().zip(&timed)
    {
        let (measured, allocated) = measure(config);
        println!(
            "  {:<30}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}  {:>9}   §12",
            config.word(),
            format!("{us:.2}"),
            marked,
            regions,
            stops,
            layers,
            allocs,
        );
        println!(
            "  {:<30}  {:>9}  {:>9}  {:>9}  {:>9}  {:>7}  {:>9}   here",
            "",
            format!("{:.2}", took.as_secs_f64() * 1e6),
            word(measured.counters.get(Counter::Marked)),
            measured.shape.regions,
            measured.shape.stops,
            measured.shape.layers,
            allocated,
        );
    }
    println!();
    println!(
        "  Four of the five `regions` and `stops` pairs reproduce exactly, and by construction:"
    );
    println!("  312 chips + 2 selects + 1 bar + 2 titles is 317 regions and 316 stops, the bar");
    println!(
        "  being the one region that is not a stop. `marked` prints **unreachable** and never"
    );
    println!(
        "  0 — `damage.rs` is `pub(crate)` throughout, so §12's 107 and its 25 080 are numbers"
    );
    println!(
        "  no crate above the engine can ask for. `us` is a **report** and gated by nothing.\n"
    );
    println!(
        "  The microseconds are an order out and the screen next door says why: `crate::dense`'s\n  \
         own 300x80, drawn through the same `Direct` ink on this machine, is {:.2} us against this\n  \
         screen's {:.2}. Both write 24 000 cells a frame, which is what §2's partition rule asks\n  \
         for, and neither is §12's 33.96 — that is a prototype's screen on a prototype's machine,\n  \
         and it is printed here rather than engineered away.\n",
        dense_micros(),
        micros(Config::Nothing)
    );

    // ── 2. the four deltas ───────────────────────────────────────────────────────────────────────
    println!("report  what each configuration costs over the base, steady:");
    // **Round-robin, minimum of sixty.** A delta of one microsecond against a base of hundreds is
    // inside the run-to-run spread of a measurement taken one configuration at a time — measured
    // that way the modal's delta comes out *negative* — so every configuration's frames are drawn
    // in the same interval. See `popup::costs`.
    let base = timed[0].as_secs_f64() * 1e6;
    for ((config, remembered), took) in [
        (Config::OneSelect, 1.33),
        (Config::MenuAndSubmenu, 1.17),
        (Config::ModalAndScrim, 0.75),
    ]
    .into_iter()
    .zip(&timed[1..])
    {
        println!(
            "  {:<30}  §12 {:>+7.2} us   here {:>+8.2} us",
            config.word(),
            remembered,
            took.as_secs_f64() * 1e6 - base
        );
    }
    println!(
        "  {:<30}  §12 {:>+7.2} us   here {:>+8.2} us",
        "its scrim (operator layer)", 0.12, 0.0
    );
    println!("  The scrim's steady delta is structurally 0 here and not merely small: it is the");
    println!("  engine's operator layer, so nothing in this crate writes one of its cells.");
    // **The honest sentence about these four numbers**, and it is the one §21's rule is for. A
    // delta of a microsecond against a base of 456 is 0.2%, and this instrument's own run-to-run
    // spread on the same three deltas is about +/-3 us — measured by running this report three
    // times, where the modal's delta came out -2.38, +0.79 and +0.92. So the **direction** is what
    // it can offer and the magnitude is not; every count beside it is exact.
    println!(
        "  A delta of a microsecond against a base of {base:.0} is 0.2%, and this report's own"
    );
    println!("  run-to-run spread on these three is about +/-3 us — three consecutive runs gave");
    println!("  the modal -2.38, +0.79 and +0.92. **The direction is the report and the magnitude");
    println!("  is not**; every count beside it is exact and gated in `src/popup.rs`.\n");

    // ── 3. the opening frame ─────────────────────────────────────────────────────────────────────
    let opened = popup::opening(30);
    println!("report  the frame a modal opens — a cliff, and it is allowed:");
    println!(
        "  §12   40.50 -> 138.04 us, 25 080 cells, once per opening (3.41x)\n  here  {:.2} -> {:.2} us, cells unreachable, {} cliffs over {} openings ({:.2}x)",
        opened.steady.as_secs_f64() * 1e6,
        opened.opening.as_secs_f64() * 1e6,
        opened.cliffs,
        opened.cycles,
        opened.ratio(),
    );
    println!(
        "  the cliff in absolute terms: {:+.2} us on the opening frame, against §12's +97.54.",
        (opened.opening.as_secs_f64() - opened.steady.as_secs_f64()) * 1e6
    );
    println!(
        "  A cliff is a fixed cost — the screen composited once — and a ratio divides it by a"
    );
    println!("  base. §12's base is 40.50 us and this screen's is an order larger, so the same");
    println!("  kind of event is 3.41x there and 1.02x here. **That is why the gate is a count.**");
    println!(
        "  surfaces reallocated over the whole run: {} — a move is 0 and a resize is 1, so a",
        opened.reallocs
    );
    println!("  dialog that opens thirty times at one size costs none after the first.\n");

    // ── 4. the scrim and the fill ────────────────────────────────────────────────────────────────
    println!("report  the scrim, in the three spellings §12 names:");
    println!("  {:<34}  {:>12}  {:>8}", "", "re-damaged", "writes");
    for spelling in ScrimSpelling::ALL {
        println!(
            "  {:<34}  {:>12}  {:>8}",
            spelling.word(),
            popup::scrim_steady(spelling, 8).per_frame,
            popup::scrim_writes(spelling),
        );
    }
    println!(
        "  §2 remembers 229 re-damaged cells where this screen says {}. See `crate::dense::SCRIM_UNDER`:",
        popup::SCRIM_UNDER
    );
    println!(
        "  nothing in this dialog is painted in the scrim's role, so the difference is total,"
    );
    println!("  and 229 says that on C01's screen 371 of the dialog's 600 already carried it.\n");
    println!("report  the popup, in its two drawing orders:");
    println!(
        "  {:<34}  {:>12}",
        "text first (correct)",
        popup::popup_steady(false, 8).per_frame
    );
    println!(
        "  {:<34}  {:>12}   §12 remembers 107 marked",
        "fill first (the defect)",
        popup::popup_steady(true, 8).per_frame
    );
    println!(
        "  The quantity is the popup's own ink: the mark on the chosen row plus {} non-blank cells",
        popup::popup_ink()
    );
    println!(
        "  in the eight labels. Counting widths instead gives 102, and the twelve between them"
    );
    println!("  are the spaces *inside* the labels — cells the fill writes and does not change.\n");

    // ── 5. the family's two axes ─────────────────────────────────────────────────────────────────
    println!("report  the family's two axes, in §12's own counts:");
    println!(
        "  A  an overlay that declares and covers its anchor: {} flips in {} frames (§12: 99)",
        popup::tooltip_flips(Kind::Popup, true, popup::FLIP_FRAMES),
        popup::FLIP_FRAMES
    );
    println!(
        "     the same tooltip beside its anchor:              {} flip",
        popup::tooltip_flips(Kind::Popup, false, popup::FLIP_FRAMES)
    );
    let (believed, lived) = popup::census(popup::CENSUS_FRAMES, 2..5);
    println!(
        "  B  a dialog owned by the menu row that opened it:   lives {lived} of {believed} frames (§12: 3 of 8)"
    );
    println!(
        "     the application believes it open on all eight. The census is over the request.\n"
    );

    // ── 6. the ring, the trap and the dismissal ──────────────────────────────────────────────────
    println!("report  the ring and the trap:");
    let trapped = popup::walkthrough(Config::ModalAndScrim, popup::TABS);
    let loose = popup::walkthrough(Config::ModalWithoutTrap, popup::TABS);
    println!(
        "  with a trap    {} Tabs, {} of them inside the dialog, {} distinct widgets, {} stops declared, {} trap",
        trapped.pressed, trapped.inside, trapped.distinct, trapped.declared, trapped.traps
    );
    println!(
        "  without one    {} Tabs, {} of them inside the dialog, {} distinct widgets, {} stops declared, {} traps",
        loose.pressed, loose.inside, loose.distinct, loose.declared, loose.traps
    );
    println!("  §21's third refinement, named rather than loosened: *the walk repeats no id, and");
    println!("  reaches every stop unless a trap is standing.* 2 of 318, and `trap_scopes` is the");
    println!("  only thing that can say which of the two it is.\n");
    println!("report  where the keyboard goes on the frame a modal closes:");
    for how in Dismiss::ALL {
        let (who, probes) = popup::closing_focus(how);
        let name = match who {
            Some(id) if id == popup::dialog_owner() => "the owner — §12's rule",
            Some(id) if id == popup::last_chip() => "the last chip of the base pass",
            Some(id) if id == popup::first_stop() => "the first menu title — whatever draws first",
            Some(_) => "somebody else",
            None => "nobody",
        };
        println!("  {:<44}  {:<44}  vanish probes {probes}", how.word(), name);
    }
    println!("  Three spellings, three answers. The middle one is what the vanish rule does when");
    println!("  nobody says anything — and the screen's own seating clause,");
    println!("  `if cx.focused().is_none() {{ cx.focus(first_chip()) }}`, is present and does not");
    println!(
        "  fire, because on that frame the focus is not `None`. A dismissal is not a seating.\n"
    );

    // ── 7. the family, and the four spellings §12 refuses ────────────────────────────────────────
    println!("report  the family, as three kinds on two axes (components 26):");
    println!("    kind        declares  owner draws  entries");
    for row in overlay::FAMILY {
        println!(
            "    {:<11} {:>8}  {:>11}  {}",
            row.kind.word(),
            if row.declares { "yes" } else { "no" },
            if row.owner_draws { "yes" } else { "no" },
            row.members.join(", "),
        );
    }
    println!(
        "  {} entries over {} kinds, and no entry belongs to two. Axis A separates the transient",
        overlay::ENTRIES,
        overlay::FAMILY.len()
    );
    println!("  and Axis B the dialog, so neither column is derivable from the other. Modality is");
    println!("  one `bool` on the request and forces no construction at all.");
    println!();

    println!("report  where a popup's size comes from, over four options:");
    println!("                              12-row screen   3-row screen   rows reachable");
    for sizing in Sizing::ALL {
        let (rw, rh) = overlay::granted(sizing, overlay::RIG_H);
        let (sw, sh) = overlay::granted(sizing, overlay::SHORT_H);
        let (reached, of) = overlay::reachable(sizing);
        println!(
            "  {:<24} {rw:>4}x{rh:<9} {sw:>4}x{sh:<9} {reached} of {of}",
            sizing.word()
        );
    }
    println!("  §12: the size may not come from the drawn extent. A popup has no frame before the");
    println!(
        "  one it opens on, so the extent is 0, so it is granted 0 rows, so it draws nothing."
    );
    println!(
        "  Sized to the content instead of to the room, {} of {} rows is unreachable — `place`",
        overlay::SPEC_UNREACHABLE,
        overlay::SHORT_OPTIONS
    );
    println!("  clamps a position and never a size, so the tail hangs off the bottom edge.");
    println!();

    println!("report  the gutter, decided in the body:");
    let mine = overlay::gutter((overlay::RIG_W, overlay::SHORT_H), 4);
    let theirs = decide(
        (overlay::RIG_W, overlay::SHORT_H),
        (u32::from(overlay::RIG_W), 4),
        Hide::WhenItFits,
    );
    println!(
        "  a popup's own decision   bar {}   passes {}",
        if mine.bar { "yes" } else { "no " },
        mine.passes
    );
    println!(
        "  §9's fixpoint            bar {}   passes {} (max {MAX_PASSES})",
        if theirs.shown.v { "yes" } else { "no " },
        theirs.passes
    );
    println!(
        "  §9 iterates because the axis a bar reports and the axis it costs are perpendicular."
    );
    println!(
        "  A popup's content is as wide as the viewport it was granted, so there is no second"
    );
    println!("  axis to couple through and no loop: 0 passes against <= 3.");
    println!();

    println!("report  blur, in the three spellings §12 names:");
    println!("                        popup survives a blur   the outside press lands   cells");
    for how in overlay::Blur::ALL {
        let b = overlay::blurs(how);
        println!(
            "  {:<20} {:>19}   {:>21}   {:>5}",
            how.word(),
            if b.survived { "yes" } else { "no" },
            if b.press_landed { "yes" } else { "no" },
            b.cells,
        );
    }
    println!(
        "  §12 prices the catcher at {} layer bytes against {} — recorded, not reproduced: the",
        overlay::SPEC_CATCHER_BYTES,
        overlay::SPEC_BLUR_BYTES
    );
    println!("  engine publishes no cell width. What is measurable is the ratio in cells, and the");
    println!(
        "  swallowed press. A press-qualified blur is the other failure: it dismisses a popup"
    );
    println!("  the pointer is standing on, because the optimistic focus arrives a frame ahead.");
    println!();

    println!("report  the keyboard, through an open popup (components 26):");
    println!("                                        survived   cursor   chosen   focus back");
    for how in overlay::Blur::ALL {
        let k = overlay::keyboard_with(overlay::Dismissal::Chose, how);
        println!(
            "  {:<36} {:>8}   {:>6}   {:>6}   {:>10}",
            how.word(),
            if k.survived_the_handover { "yes" } else { "no" },
            k.cursor,
            k.chosen,
            if k.focus_is_the_owners { "yes" } else { "no" },
        );
    }
    let cancelled = overlay::keyboard(overlay::Dismissal::Escape);
    println!(
        "  and `Esc` instead of `Enter`:        {:>8}   {:>6}   {:>6}   {:>10}",
        if cancelled.survived_the_handover {
            "yes"
        } else {
            "no"
        },
        cancelled.cursor,
        cancelled.chosen,
        if cancelled.focus_is_the_owners {
            "yes"
        } else {
            "no"
        },
    );
    println!(
        "  **This is the row the application earned.** Every other instrument on this section"
    );
    println!("  posts its keys at the *owner*, which works whether or not the popup ever took the");
    println!(
        "  keyboard — so a popup with dead arrows passes all of them. What a blur is, from the"
    );
    println!(
        "  owner's side, is `seated && !inside && !over`: `Response::focus_left` on the owner"
    );
    println!(
        "  stops being the signal the moment the popup takes the keyboard, because the owner no"
    );
    println!(
        "  longer holds the focus and has none to lose. The last row forgets the middle clause."
    );
    println!();

    println!("report  the two halves of a modal, and they are two verbs:");
    println!(
        "  barrier standing   a press reaches the base pass: {}",
        if overlay::barrier_withholds_the_pointer(true) {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  barrier gone       a press reaches the base pass: {}   (the trap is still standing)",
        if overlay::barrier_withholds_the_pointer(false) {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  and the keyboard half is above: {} Tabs, {} of them inside with a trap and {} without.",
        popup::TABS,
        trapped.inside,
        loose.inside
    );
    println!();

    println!(
        "report  what a body holds its list position in, over {} notches:",
        popup::WHEEL_CLICKS
    );
    println!(
        "  `&'f mut PopupState`         the offset moves {}",
        overlay::wheeled(true)
    );
    println!(
        "  a `Copy` of the offset       the offset moves {}   (§7's literal Copy-only body)",
        overlay::wheeled(false)
    );
    println!("  The screen is identical while it happens, and every other counter agrees.");
    println!();

    // ── 8. what does not reproduce ───────────────────────────────────────────────────────────────
    println!("report  what does not reproduce, and why:");
    println!(
        "  menu delta     §12 says a menu with its submenu adds {}/{}; the shipped one adds {}/{}.",
        popup::SPEC_MENU_DELTA.0,
        popup::SPEC_MENU_DELTA.1,
        popup::MENU_DELTA.0,
        popup::MENU_DELTA.1
    );
    println!(
        "                 §12's six is three rows twice, each a target of its own. §5 collapses"
    );
    println!(
        "                 a menu into a `Mode` of `collection`, and a collection declares one"
    );
    println!("                 hit entry however many rows it has — so each level is a dropdown's");
    println!("                 delta: one entry for its rows and one for its blur position. It is");
    println!(
        "                 the same subtraction `PER_ROW_ENTRIES` prices for a dropdown, on the"
    );
    println!("                 construction §12's own prototype spent per row.");
    println!(
        "  allocations    §12 says 0 in all five rows; the shipped figure is n + 1 for n overlays"
    );
    println!(
        "                 standing and 0 for none. The arena that made it 0 was deleted by runtime"
    );
    println!(
        "                 ticket 21 (ADR 0034) — seven `unsafe` blocks in a workspace with none."
    );
    println!(
        "  content layers §12 says 2 / 4 / 3; the census says 1 / 2 / 1. Every request in the"
    );
    println!("                 prototype carried `shadow: 96`, a second operator layer beside its");
    println!("                 own, and `OverlayOpts` has no shadow field. The ratio survives: a");
    println!("                 menu with its submenu is twice a dropdown either way.");
    println!(
        "  marked         unreachable, in both of the places §12 gives one — the fill-first 107"
    );
    println!("                 and the opening frame's 25 080.");
    println!(
        "  fill-first     {} against §12's 107, and it is arithmetic: this popup's option list is",
        popup::FILL_FIRST
    );
    println!(
        "                 its own data. §12's 107 says its popup carried 107 cells of ink in 160."
    );
    println!("  re-damage      600 against §2's 229 for a scrim under the dialog, which is");
    println!("                 `crate::dense`'s finding reproduced on a second screen.");
    println!(
        "  six Tabs       §12 says six Tabs leave an untrapped modal. On this runtime a standing"
    );
    println!(
        "                 trap is also what pulls the focus *in*, so an untrapped modal never"
    );
    println!("                 receives the keyboard: 0 of 6 inside, not 6 out.\n");

    // ── the scene ────────────────────────────────────────────────────────────────────────────────
    let scene = scenes::SCENES
        .iter()
        .find(|s| s.number == 14)
        .expect("§21's scene 14");
    println!("report  scene 14 — {}", scene.name);
    println!("  {:?}", scene.standing);
    println!(
        "\n  {}",
        popup::owed_message(&popup::subjects_declared(), "scene 14")
            .unwrap_or_else(|| "scene 14 stands: both subjects are declared".to_string())
    );
}

/// One configuration, measured over [`FRAMES`] steady frames, with its allocation total.
///
/// The total is taken over a **warmed** `Direct` loop and not around [`popup::steady`], which
/// attaches its own driver and keeps a [`vitui_components::counters::Tally`] — a `BTreeSet` of every
/// cell it sees — inside the window. A figure taken there is a report about the instrument, which is
/// the same distinction `popup::cost` draws for the microseconds. `tests/popup.rs` is where this
/// number is a gate.
fn measure(config: Config) -> (popup::Measured, u64) {
    let mut driver = vitui_components::runner::driver_at(popup::W, popup::H, Default::default());
    let mut held = popup::Held::new();
    let mut ink = vitui_components::ink::Direct;
    for _ in 0..2 {
        driver.frame(|cx| {
            popup::draw_into(&mut ink, cx, &mut held, config);
        });
    }
    let (_, allocated) = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.frame(|cx| {
                popup::draw_into(&mut ink, cx, &mut held, config);
            });
        }
    });
    (
        popup::steady(config, FRAMES, Allocations::over(FRAMES, allocated as u64)),
        allocated as u64,
    )
}

/// The per-frame cost through [`vitui_components::ink::Direct`], which is the figure comparable with
/// §12's own. See `popup::cost`.
fn micros(config: Config) -> f64 {
    popup::cost(config, FRAMES).as_secs_f64() * 1e6
}

/// **The dense screen through the same ink, on the same machine**, so the microsecond column has a
/// neighbour rather than only a memory to be compared against.
fn dense_micros() -> f64 {
    use vitui_components::dense::{self, Arm};
    let mut driver = vitui_components::runner::driver_at(dense::W, dense::H, Default::default());
    let mut ink = vitui_components::ink::Direct;
    for _ in 0..2 {
        driver.frame(|cx| {
            dense::draw_into(&mut ink, cx, Arm::Correct, dense::REQUESTED);
        });
    }
    let started = std::time::Instant::now();
    for _ in 0..FRAMES {
        driver.frame(|cx| {
            dense::draw_into(&mut ink, cx, Arm::Correct, dense::REQUESTED);
        });
    }
    started.elapsed().as_secs_f64() * 1e6 / f64::from(FRAMES)
}

/// A reading, or the word `unreachable`. **Never `0`.**
fn word(reading: Reading) -> String {
    match reading.measured() {
        Some(n) => n.to_string(),
        None => "unreachable".to_string(),
    }
}
