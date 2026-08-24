//! **Runtime ticket 05's reports**: the shipped set, the swap frame, and the two silent mistakes.
//!
//! ```text
//! cargo run --release --example theme_set_numbers -p vitui-runtime
//! ```
//!
//! Six sections:
//!
//! 1. **The shipped set at three tiers** — collapsed role pairs, `Hover`, `Fade` and `Status`, per
//!    theme. The `fade_is_showable` split is in this table: every theme shows a hover at 256 colours
//!    and only some show the animation into it.
//! 2. **What a theme costs in `.rodata`** — the import is a `const fn`, so this is a `size_of` and
//!    not an allocation count.
//! 3. **Live switching** — building the registry, selecting, and re-detecting the tier.
//! 4. **The pair key** — a data key against a pair key, and what *every memo* costs instead.
//! 5. **The theme-dependent fold** — the O(visible) invariant broken on the one frame nobody
//!    measures.
//! 6. **The swap frame** — a full repaint against a steady one, in microseconds and in bytes on the
//!    wire.
//!
//! # What is quoted rather than measured, and why
//!
//! Four numbers on the map are properties of its **338-scheme corpus**: the mean collapsed pairs at
//! three tiers, the ten-profile C16 bound (10.87 → 16.36), the `fade_is_showable` split (165 of 338)
//! and the contrast finding (218 → 55). The corpus is deliberately **not vendored** — the map's own
//! research measured it from a clone and vendored none of it — so those four cannot be re-taken here.
//! Two of them could not be taken from an example even with the data, because they need a role's
//! colours and a wire index, and neither is on the public surface: ADR 0018 keeps a colour out of a
//! component's hands and ADR 0007 keeps a palette index out of everybody's.
//!
//! So each is quoted with its provenance, and the **mechanism** each one is evidence for is gated
//! in-tree as a relation instead:
//!
//! | corpus number | quoted from | the in-tree relation that holds it |
//! |---|---|---|
//! | 0.08 / 0.26 / 10.87 of 78 | research §3 | `narrowing_never_adds_a_distinction_for_any_shipped_theme` |
//! | 10.87 → 16.36 through ten profiles | research §3 | `a_sixteen_colour_count_is_a_lower_bound_and_an_operator_can_only_raise_it` |
//! | `fade_is_showable` 165 of 338 | research §3 | measured over the shipped fourteen, in §1 below |
//! | contrast 218 → 55 | research §2 | `pairing_by_luminance_never_costs_contrast_and_usually_buys_it` |
//!
//! # Provenance
//!
//! **R 05** took these numbers and **R 20** re-measured both of its `crate::ledger` rows against the
//! shipped runtime on 2026-08-24: Apple M1 Max, macOS 26.5.2, rustc 1.97.1, `--release`, unloaded,
//! minimum of 40 rounds. `theme()` — the one read a frame actually does — is **0.95 ns against the
//! prototype's 15**, sixteen times smaller; everything else the registry does is per swap, including
//! `set_tier` at 284.79 ns.
//!
//! **The swap frame's two microsecond columns are not comparable to the prototype's, and the report
//! says so rather than reconciling them.** The shipped fixture is a full-screen fill and the map's
//! was the dense IDE screen, so 309 µs against a 118 µs steady frame is a different measurement from
//! 41.92 against 33.20. What survived the fixture change is the *shape* of the cliff, and R 20
//! restated it in **bytes**, where the fixture cannot blur it: a steady frame is **0 bytes on the
//! wire** and a swap frame is **26 272**. That is not a ratio, it is a floor against a screen. Its
//! detector is the memo pair key,
//! `tests/swap.rs::a_pair_keyed_memo_misses_once_per_swap_and_a_data_keyed_one_never_does`.
//!
//! The frame budget the *quoted* figures lean on — the map's 221 / 399 µs read as **two and four
//! whole frame budgets** — is `crate::ledger`'s and not this file's. Spec §19 inherits it from the
//! engine map, and **a budget figure may not move without a new map decision**, which is what lets
//! that sentence be argued from rather than merely quoted.

use std::hint::black_box;
use std::sync::{Arc, Mutex};

use vitui_bench::Bench;
use vitui_engine::{Clock, ColorDepth, Config, Output, Overrides, Rect};
use vitui_runtime::Driver;
use vitui_runtime::data::{Memo, Versioned};
use vitui_runtime::theme::{Distinction, Role, Scheme, Theme, Themes};

/// Seventy-eight, which is thirteen roles taken two at a time.
const PAIRS: usize = 13 * 12 / 2;

/// The three tiers that carry colour. `None` is a tier and not a degenerate case, but it makes every
/// pair a colour-free pair and belongs in ticket 04's table rather than this one.
const TIERS: [ColorDepth; 3] = [
    ColorDepth::TrueColor,
    ColorDepth::Indexed256,
    ColorDepth::Ansi16,
];

/// The dense screen every budget number on this backlog is against.
const W: u16 = 300;
/// See [`W`].
const H: u16 = 80;

fn main() {
    println!("runtime ticket 05 — the standard set and the swap frame\n");
    the_shipped_set();
    what_a_theme_costs_in_rodata();
    live_switching();
    the_pair_key();
    the_theme_dependent_fold();
    the_swap_frame();
}

/// How many of the seventy-eight role pairs a theme cannot distinguish.
fn collapsed(theme: &Theme) -> usize {
    let mut n = 0;
    for (i, &a) in Role::ALL.iter().enumerate() {
        for &b in &Role::ALL[i + 1..] {
            if !theme.roles_differ_on_wire(a, b) {
                n += 1;
            }
        }
    }
    n
}

/// Report: every shipped theme at every tier that carries colour.
///
/// **The two universal gates are restated here as measurements** rather than repeated as assertions:
/// `Face`/`FaceHover` at 256 colours and a non-empty author are `cargo test`'s, and what this table
/// adds is the shape of what they are universal *over*.
fn the_shipped_set() {
    let mut set = Themes::standard();
    println!(
        "report  the shipped set, collapsed role pairs of {PAIRS} and the distinction bits\n\
         \x20       H = Hover, F = Fade, S = Status; a dash is the bit resolving to false\n"
    );
    println!(
        "          {:<24} {:>5} {:>5} {:>5}   {:<5} {:<5} {:<5}  variant",
        "scheme", "True", "C256", "C16", "True", "C256", "C16"
    );

    let mut fade = [0usize; 3];
    let mut hover = [0usize; 3];
    let mut totals = [0usize; 3];
    for i in 0..set.len() {
        assert!(set.select(i));
        let slug = set.scheme().slug();
        let dark = if set.scheme().is_dark() {
            "dark"
        } else {
            "light"
        };
        let mut counts = [0usize; 3];
        let mut bits = [String::new(), String::new(), String::new()];
        for (t, tier) in TIERS.iter().enumerate() {
            set.set_tier(*tier);
            counts[t] = collapsed(set.theme());
            totals[t] += counts[t];
            let mut spelling = String::new();
            for (d, letter) in [
                (Distinction::Hover, 'H'),
                (Distinction::Fade, 'F'),
                (Distinction::Status, 'S'),
            ] {
                spelling.push(if set.theme().shows(d) { letter } else { '-' });
            }
            if set.theme().shows(Distinction::Fade) {
                fade[t] += 1;
            }
            if set.theme().shows(Distinction::Hover) {
                hover[t] += 1;
            }
            bits[t] = spelling;
        }
        println!(
            "          {slug:<24} {:>5} {:>5} {:>5}   {:<5} {:<5} {:<5}  {dark}",
            counts[0], counts[1], counts[2], bits[0], bits[1], bits[2]
        );
    }

    let n = set.len();
    #[allow(
        clippy::cast_precision_loss,
        reason = "a mean of fourteen small counts"
    )]
    let mean = |t: usize| totals[t] as f64 / n as f64;
    println!(
        "\n          {:<24} {:>5.2} {:>5.2} {:>5.2}   mean of {n}\n",
        "",
        mean(0),
        mean(1),
        mean(2)
    );
    println!(
        "        **`hover_distinct` and `fade_is_showable` are two capabilities, and this is the\n\
        \x20       split.** hover {}/{n} / {}/{n} / {}/{n} against fade {}/{n} / {}/{n} / {}/{n} at\n\
        \x20       True / C256 / C16. A component that reads only the first and then cross-fades pays\n\
        \x20       nineteen wakeups for a picture the terminal will not draw.\n\
        \x20       The map's corpus figure is **165 of 338 at 256 colours**, quoted from research §3;\n\
        \x20       the corpus is not vendored here, and the shipped fourteen are what this row is.\n",
        hover[0], hover[1], hover[2], fade[0], fade[1], fade[2]
    );
    println!(
        "        Corpus, quoted with provenance (research/19-standard-theme-set.md §3): mean\n\
        \x20       **0.08 / 0.26 / 10.87** of 78 over 338 schemes, against the stub palette's\n\
        \x20       1 / 2 / 13, and **`Face`/`FaceHover` collapses at C256 in 0 of 338** against 73\n\
        \x20       under a mapping that renames slots instead of walking the ramp. Re-taken through\n\
        \x20       ten real terminal profiles the C16 mean goes **10.87 -> 16.36**, because five of\n\
        \x20       the ten spell an index twice — so **a C16 count is a lower bound, not a\n\
        \x20       measurement**, and nothing in this process may read the operator's palette.\n"
    );
}

/// Report: what the `const fn` import buys, which is a `size_of` rather than an allocation count.
fn what_a_theme_costs_in_rodata() {
    let scheme = size_of::<Scheme>();
    let roles = size_of::<vitui_runtime::Roles>();
    let theme = size_of::<Theme>();
    let shipped = Themes::standard().len();
    println!("report  what a theme costs, and where it lives\n");
    println!("          size_of::<Roles>()      {roles:>6} B   thirteen (fg, bg, attrs) triples");
    println!(
        "          size_of::<Scheme>()     {scheme:>6} B   + slug, name, author, the dark bit"
    );
    println!(
        "          size_of::<Theme>()      {theme:>6} B   + styles, wire keys, tier, revision"
    );
    println!(
        "          the shipped {shipped}          {:>6} B   in .rodata, built by a const fn",
        scheme * shipped
    );
    println!("          forty themes            {:>6} B", scheme * 40);
    println!(
        "\n        The map predicted **104 B** the roles and **160 B** with attribution, and forty\n\
        \x20       themes at **6.4 KB**. What makes any of it `.rodata` rather than a start-up cost\n\
        \x20       is that `Roles::from_palette` is a `const fn` — the pick, the derivation, the\n\
        \x20       luminance pairing and the 256-colour quantiser they all consult run at compile\n\
        \x20       time. There is no parser in this crate and no `Vec` on the path.\n\
        \x20       A `Theme` is larger than a `Scheme` and is **not** in `.rodata`: it carries the\n\
        \x20       thirteen built styles, the thirteen wire keys `resolve` precomputed, and a\n\
        \x20       revision from the process-global counter, which no `const` can have.\n"
    );
}

/// Report: the registry's own operations. **Only `select` and `set_tier` build a theme**, and both are
/// per switch rather than per frame.
fn live_switching() {
    // **One registry per arm, and it is not tidiness.** Four closures sharing one `&mut Themes`
    // does not compile, and the version that does — one registry threaded through a `RefCell` —
    // would put a borrow check inside the measurement.
    let (mut by_index, mut by_slug, mut retiered) =
        (Themes::standard(), Themes::standard(), Themes::standard());
    let mut read = Themes::standard();
    read.set_tier(ColorDepth::TrueColor);
    let report = Bench::new(40)
        .case("Themes::standard, once at start-up", 1_000, || {
            black_box(Themes::standard());
        })
        .case("select, one rebuild", 1_000, || {
            black_box(black_box(&mut by_index).select(black_box(3)));
        })
        .case("select_slug, a scan then a rebuild", 1_000, || {
            black_box(black_box(&mut by_slug).select_slug(black_box("kanagawa")));
        })
        .case("set_tier, a re-detection", 1_000, || {
            black_box(&mut retiered).set_tier(black_box(ColorDepth::Indexed256));
            black_box(&retiered);
        })
        .case("theme(), the read a frame does", 10_000, || {
            black_box(black_box(&read).theme());
        })
        .run();
    println!("report  live switching, minimum of 40 rounds:\n{report}");
    println!(
        "        The map measured a re-detected tier at **16.75 ns** and said the important part:\n\
        \x20       **it rebuilds one theme, not the set.** Only the current scheme is ever built into\n\
        \x20       a `Theme`, so only one thing can be stale. A registry that cached a resolved theme\n\
        \x20       per scheme would pay the whole set's import for a tier that arrives once.\n\
        \x20       `set_tier` here is `Theme::imported` plus `Theme::resolve`, and `resolve` is the\n\
        \x20       expensive half at ticket 04's measured 290 ns — thirteen quantisations paid once\n\
        \x20       so that every question a component asks afterwards is two array reads.\n\
        \x20       **The read a frame does is the cheap one**, which is the property that matters:\n\
        \x20       nothing here is on the per-frame path except `theme()`.\n"
    );
}

/// Report: a data key against a pair key, and what reading the rule as *every memo* costs instead.
fn the_pair_key() {
    let data = Versioned::new(vec![1u32, 2, 3]);
    let theme = Theme::default().resolve(ColorDepth::TrueColor);
    let report = Bench::new(40)
        .case("a data key, as ticket 01 ships it", 10_000, || {
            black_box(black_box(&data).revision());
        })
        .case("a pair key, data and theme", 10_000, || {
            black_box(black_box(&theme).memo_key(black_box(data.revision())));
        })
        .run();
    println!("report  the memo key, minimum of 40 rounds:\n{report}");
    let plain = report
        .get("a data key, as ticket 01 ships it")
        .expect("measured");
    let paired = report.get("a pair key, data and theme").expect("measured");
    println!(
        "        pair / data = {:.3}x, +{:.2} ns, where the map measured **+0.312 ns, 1.247x**.\n\
        \x20       **It is a rotate, a multiply and an xor, and not the crate\'s own FNV-1a.** FNV eats\n\
        \x20       one byte at a time, so folding two words is sixteen rounds — measured at 5.76 ns,\n\
        \x20       which does not fit the budget above for a value taken once per memo per frame.\n\
        \x20       What the cheaper form gives up is a hash\'s diffusion and what it buys is a property\n\
        \x20       that can be stated and gated: **injective in each argument with the other held\n\
        \x20       fixed**, so a swap always moves the key and an edit always moves the key.\n",
        paired / plain,
        paired - plain
    );

    // What "every memo" costs: the same fold, on the swap frame, for values that are bit-identical
    // afterwards. Sized at the dense screen's own visible window rather than at a round number.
    let rows: Vec<u32> = (0..H as u32 * 4).collect();
    let mut widths: Vec<Memo<usize>> = (0..4).map(|_| Memo::new()).collect();
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);

    for memos in [2usize, 4] {
        for m in &mut widths {
            *m = Memo::new();
        }
        let mut recomputes = 0;
        for slug in ["dracula", "nord", "solarized-light", "kanagawa"] {
            assert!(themes.select_slug(slug));
            let key = themes.theme().memo_key(data.revision());
            for m in widths.iter_mut().take(memos) {
                // A width is not made of paints, so this key is the defect and the fold is the cost.
                let _ = m.get(key, || {
                    rows.iter().map(|r| *r as usize % 17).max().unwrap_or(0)
                });
            }
        }
        for m in widths.iter().take(memos) {
            recomputes += m.recomputes;
        }
        println!(
            "        {memos} theme-keyed memos over four swaps: **{recomputes} recomputations**, and every one\n\
            \x20       of them produced a value bit-identical to the one it replaced."
        );
    }
    println!(
        "\n        The map priced that at **221 / 399 us** at two and four memos on the dense screen —\n\
        \x20       two and four whole frame budgets, on the frame that is already the most expensive\n\
        \x20       one. So the rule is narrow and the detector checks **both** directions:\n\
        \x20       *a memo carries the theme in its key exactly when its value is made of paints.*\n\
        \x20       `tests/swap.rs` is the count, over a corpus containing one of each defect.\n"
    );
}

/// Report: the second silent mistake, which is the O(visible) invariant broken on the swap frame.
fn the_theme_dependent_fold() {
    let theme = Theme::default().resolve(ColorDepth::TrueColor);
    // A chunked data source of the size the map's scene list uses.
    let rows: Vec<u32> = (0..200_000).collect();
    let visible = usize::from(H) - 4;

    let report = Bench::new(20)
        .case("theme-dependent fold, visible window", 200, || {
            let theme = black_box(&theme);
            let mut n = 0usize;
            for r in black_box(&rows).iter().take(black_box(visible)) {
                n += usize::from(theme.paint(paint_role(*r)) == theme.paint(Role::Body));
            }
            black_box(n);
        })
        .case("theme-dependent fold, whole dataset", 1, || {
            let theme = black_box(&theme);
            let mut n = 0usize;
            for r in black_box(&rows) {
                n += usize::from(theme.paint(paint_role(*r)) == theme.paint(Role::Body));
            }
            black_box(n);
        })
        .run();
    println!("report  a theme-dependent fold, minimum of 20 rounds:\n{report}");
    println!(
        "        The map measured **6.63 / 13.27 ms** for the whole-dataset form against **46.97 us**\n\
        \x20       bounded — the map's O(visible) invariant broken on the one frame nobody was\n\
        \x20       measuring, because on every other frame the memo hides it. **The engine never\n\
        \x20       iterates application data and neither does the runtime**; culling is the caller's,\n\
        \x20       and a swap is the frame that finds out whether it was done.\n"
    );
}

/// A role that depends on the datum, so the fold above cannot be hoisted out of the loop.
fn paint_role(r: u32) -> Role {
    Role::ALL[(r % 13) as usize]
}

/// A sink a report can read back, so the swap frame can be priced in bytes as well as in time.
struct Shared(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Shared {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("no panic holds this lock")
            .extend_from_slice(b);
        Ok(b.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Report: the swap frame against a steady one, in microseconds and in bytes.
///
/// **A swap frame needs no permission to exceed the budget, because it is a cliff by construction**:
/// it is a steady frame plus a full repaint, and a full repaint is what a swap *is*. The thing worth
/// being afraid of is not this frame; it is the two silent mistakes above, which exceed the budget on
/// frames nobody labelled.
fn the_swap_frame() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);
    let mut driver = Driver::attach(
        Config {
            clock: Clock::Manual,
            output: Output::Sink(Box::new(Shared(Arc::clone(&buf)))),
            size: (W, H),
            overrides: Overrides {
                colors: Some(ColorDepth::TrueColor),
                ..Default::default()
            },
            ..Default::default()
        },
        *themes.theme(),
    )
    .expect("attaching to a sink cannot fail");

    let mut painted = 0usize;
    let draw = |driver: &mut Driver| -> (u128, usize, usize) {
        buf.lock().expect("no panic").clear();
        let at = std::time::Instant::now();
        let mut cells = 0usize;
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let title = cx.theme().paint(Role::Title);
            let face = cx.theme().paint(Role::Face);
            for y in 0..H {
                let paint = match y % 3 {
                    0 => title,
                    1 => face,
                    _ => body,
                };
                cx.fill(Rect::new(0, i32::from(y), W, 1), "x", paint);
                cells += usize::from(W);
            }
        });
        let took = at.elapsed().as_micros();
        let bytes = buf.lock().expect("no panic").len();
        (took, bytes, cells)
    };

    let (first, first_bytes, cells) = draw(&mut driver);
    painted += cells;
    let (steady, steady_bytes, _) = draw(&mut driver);
    assert!(themes.select_slug("solarized-light"));
    driver.set_theme(*themes.theme());
    let (swap, swap_bytes, _) = draw(&mut driver);
    let (after, after_bytes, _) = draw(&mut driver);

    println!("report  the swap frame, one sample each (a timing, never a gate)\n");
    println!("          {:<22} {:>8} {:>12}", "frame", "us", "bytes out");
    println!(
        "          {:<22} {first:>8} {first_bytes:>12}",
        "first draw"
    );
    println!("          {:<22} {steady:>8} {steady_bytes:>12}", "steady");
    println!("          {:<22} {swap:>8} {swap_bytes:>12}", "the swap");
    println!(
        "          {:<22} {after:>8} {after_bytes:>12}",
        "the one after"
    );
    println!(
        "\n          cells the view painted per frame: {}\n\
        \x20       **This is a full-screen fill and not the map\'s dense IDE screen**, so the two\n\
        \x20       microsecond columns are not comparable and the shape is what to read: a steady\n\
        \x20       frame and a swap frame of the *same* drawing, one after the other.\n",
        painted
    );
    println!(
        "        The map measured **41.92 us against a 33.20 us steady frame, 20 060 damaged cells\n\
        \x20       against 0**. Two of those three are not readable from here, and that is the finding\n\
        \x20       rather than a gap in this report:\n\
        \x20       - **`Surface::damaged_cells` is `pub(crate)`**, so a damaged-cell count cannot be\n\
        \x20         taken from outside the engine at all. The cells the *view* painted are printed\n\
        \x20         above instead, and they are the application's writes rather than the engine's\n\
        \x20         damage.\n\
        \x20       - **A steady frame does not mark zero damage.** There is no write-time equality\n\
        \x20         filter on a surface — it was measured and refused, because `clear`-then-draw\n\
        \x20         defeats it — so every write marks damage and `Presented::submitted` is true for\n\
        \x20         every frame that drew. The swap becomes visible one layer further down, at the\n\
        \x20         serialiser's mirror, and **that is the bytes column above**: a steady frame\n\
        \x20         re-sends almost nothing and a swap frame re-sends the screen.\n\
        \x20       **The bytes are the cliff, and they are a ratio rather than a stopwatch** — which\n\
        \x20       is the one form of this number that is worth trusting across machines.\n"
    );
    println!(
        "        Measured here: a steady frame is **{steady_bytes} bytes** and a swap frame is\n\
        \x20       **{swap_bytes}**, so the ratio is not a ratio — it is a floor against a screen.\n\
        \x20       `tests/swap.rs` gates exactly that pair, which is an equality and a relation\n\
        \x20       rather than the stopwatch above."
    );
}
