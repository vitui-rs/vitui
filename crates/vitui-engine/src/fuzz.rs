//! The two fuzz targets, and the reason they are a library module rather than two files in `fuzz/`.
//!
//! # The polarity is inverted
//!
//! The shape of this file:
//!
//! > **Every crash is minimised and committed as an ordinary unit test, and the committed corpus
//! > replayed as an ordinary test is the gate. The fuzzer itself is a scheduled soak.**
//!
//! A fuzzer cannot be a pull-request gate. At sixty seconds it finds nothing and reports green,
//! which is a gate that cannot fail; at an hour it is a flaky test that fails on a commit unrelated
//! to whatever it finally reached. So the corpus is the gate: the tests at the bottom of this file
//! replay every committed input through the same two functions the fuzzer calls, under an ordinary
//! `cargo test`, on the stable toolchain, in every job that runs the test suite.
//!
//! # Why the harness is here and not in `fuzz/`
//!
//! Because the gate and the soak must be **the same code**. `fuzz/fuzz_targets/*.rs` is four lines
//! each: a `fuzz_target!` that calls one of these functions. If the decoding and the oracle lived
//! there, the replay test could only reimplement them, and *a check that is weaker than the gate is
//! not a check* — a corpus file would then be replayed through a second oracle that agrees with the
//! first for as long as somebody keeps them in step.
//!
//! It also means the fuzz workspace needs nothing from this crate that a caller could not have:
//! `fuzz` is a non-default feature, the module is `#[doc(hidden)]`, and
//! `crate::audit`'s `the_fuzz_door_is_behind_a_feature_and_hidden` is what keeps that from becoming
//! an accidental widening of the surface.
//!
//! # What each target's oracle is, and what it is not
//!
//! **Draw sequences** (the first target) are checked against `crate::reference`, the naive
//! compositor — one cell at a time, no damage, no runs. Two halves, and they are gate #1's own two:
//! every cell the frame's verbs changed lies inside a run the damage structure reported, and the
//! damage-tracked frame equals the reference **everywhere**, including outside every damaged run.
//! Nothing in either sentence mentions how damage is marked.
//!
//! **Byte streams** (the second) go into `crate::input::parse`, where the oracle is
//! weaker — *no panic, every byte consumed, no unbounded growth* — and where the surface is the one
//! thing in this crate that parses input the engine did not produce.
//!
//! The middle of those three needs saying precisely, because the obvious reading of it is vacuous.
//! `Parser::feed` is a `for` loop over the bytes it was given, so *every byte was consumed* is true
//! by construction, and a counter asserting it would be measuring the loop rather than the parser.
//! What is not true by construction is that it does not **matter** where the
//! reads were cut, and that is the property this file asserts instead: feeding a stream in chunks
//! produces exactly the events feeding it whole does. A byte lost, double-counted or mis-attributed
//! at a chunk boundary is precisely what that catches — and it is the property a future `feed` that
//! scans for `ESC` with `memchr` instead of stepping byte by byte would break.
//!
//! `end_of_read` is deliberately outside that equality. It is the one place a read boundary carries
//! meaning — a bare `ESC` is flushed as the Escape key — so an arm that ends every chunk is run
//! for the other two oracles and not for this one.

use std::time::Instant;

use crate::caps::{ColorDepth, Overrides};
use crate::damage::Run;
use crate::engine::{Clock, Config, Engine, Output, Screen};
use crate::geom::Rect;
use crate::input::parse::Parser;
use crate::input::{Event, InputConfig};
use crate::layer::LayerId;
use crate::mix::Mix;
use crate::restyle::Restyle;
use crate::style::{Color, Style};
use crate::surface::Surface;

/// How many layers a program may have at once.
///
/// The reference compositor visits every cell against every layer, twice a frame, so this is the
/// term that decides what a soak second buys. Eight is above every shape the twelve scenes reach at
/// this size and low enough that a frame is microseconds.
const MAX_LAYERS: usize = 8;
/// How many frames one program may present.
const MAX_FRAMES: u32 = 8;
/// How many verbs one program may run, whatever its input length.
///
/// A cap and not a budget: without it a megabyte of input is a megabyte of `fill`, which is a slow
/// unit test and not a better oracle.
const MAX_OPS: u32 = 96;

/// The clusters a program can write.
///
/// Chosen for the repair rules rather than for coverage of Unicode: an ASCII head, a wide CJK pair,
/// a cluster whose combining mark makes it two codepoints and one column, an emoji that is wide and
/// multi-scalar, a regional-indicator pair, and a zero-width space. Every one of the five repair
/// rules is reachable from this list.
const CLUSTERS: [&str; 8] = ["a", "#", " ", "漢", "e\u{301}", "👍", "🇺🇦", "\u{200b}"];

/// A byte string, read left to right, that never runs out.
///
/// Past the end every read is zero and [`done`](Bytes::done) is true — so a truncated program is a
/// shorter program rather than a different one, which is what makes libFuzzer's minimisation
/// converge on something a human can read.
struct Bytes<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Bytes<'a> {
    fn new(data: &'a [u8]) -> Bytes<'a> {
        Bytes { data, at: 0 }
    }

    fn done(&self) -> bool {
        self.at >= self.data.len()
    }

    fn byte(&mut self) -> u8 {
        let b = self.data.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        b
    }

    /// A number below `n`. `n` is never zero at a call site here, and a zero would be a division.
    fn below(&mut self, n: u8) -> u8 {
        self.byte() % n
    }

    /// A coordinate on an axis `span` long, reaching four columns outside it at each end.
    ///
    /// Outside is the point. The rule is clamp-and-discard, the wide-glyph hazard lives at an edge,
    /// and a generator that only ever produced coordinates inside the screen would be a generator
    /// that never tested either.
    fn coord(&mut self, span: u16) -> i32 {
        i32::from(self.byte() % (span as u8).saturating_add(8)) - 4
    }

    /// An extent on an axis `span` long, including zero — an empty rectangle is a case.
    fn extent(&mut self, span: u16) -> u16 {
        u16::from(self.byte() % (span as u8).saturating_add(3))
    }

    fn rect(&mut self, w: u16, h: u16) -> Rect {
        let (x, y) = (self.coord(w), self.coord(h));
        Rect::new(x, y, self.extent(w), self.extent(h))
    }

    /// A stacking order, small and signed, so ties in `(z, seq)` are common rather than rare.
    fn z(&mut self) -> i32 {
        i32::from(self.byte() % 7) - 3
    }

    fn cluster(&mut self) -> &'static str {
        CLUSTERS[usize::from(self.below(8))]
    }

    fn style(&mut self) -> Style {
        let (fg, bg, attrs) = (self.byte(), self.byte(), self.byte());
        let mut style = Style::new().fg(Color::indexed(fg)).bg(Color::indexed(bg));
        if attrs & 0b0001 != 0 {
            style = style.bold();
        }
        if attrs & 0b0010 != 0 {
            style = style.italic();
        }
        if attrs & 0b0100 != 0 {
            style = style.reverse();
        }
        if attrs & 0b1000 != 0 {
            style = style.underline_curly();
        }
        style
    }
}

/// Takes everything, keeps none of it. A recording sink would make a long program a growing `Vec`.
struct Discard;

impl std::io::Write for Discard {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// A headless screen of `w` by `h`, on a terminal that can express everything.
///
/// **Truecolor and OSC 8 are pinned, and neither is decoration.** A caller-supplied sink is asked
/// nothing, so it stays at `ColorDepth::None` — where every operator layer is skipped outright
/// and a hyperlink stops making a cell extended. A differential fuzz on an unpinned screen would be
/// a differential fuzz over the half of the compositor that survives the depth, and both oracles
/// would agree about the half that does not run.
fn screen(w: u16, h: u16) -> Screen {
    let (screen, _wake) = Engine::new(Config {
        size: (w, h),
        output: Output::Sink(Box::new(Discard)),
        clock: Clock::Manual,
        max_frame_rate: f32::INFINITY,
        overrides: Overrides {
            colors: Some(ColorDepth::TrueColor),
            hyperlinks: Some(true),
            ..Overrides::default()
        },
        overrun_threshold: None,
        overrun_report: None,
        input: InputConfig::default(),
    })
    .attach()
    .expect("attaching to a sink cannot fail");
    screen
}

/// Whether `(x, y)` lies inside one of the reported runs. `crate::gates`' `covered`, over here.
fn covered(runs: &[Run], x: u16, y: u16) -> bool {
    runs.iter().any(|r| r.y == y && r.lo <= x && x <= r.hi)
}

/// The input, as a line a failure message can carry and a human can paste back.
fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

/// **Target 1: a draw sequence, against the naive reference compositor.**
///
/// The input is a program: two bytes of screen size, then verbs. Every `present` is a checkpoint,
/// and at a checkpoint gate #1's two halves are asserted of the frame that was just built — which
/// is what *the damage gate is generated rather than hand-written* means. A
/// hand-written expectation about damage is written by whoever wrote the damage and agrees with it
/// for the same reason; this one is a composite of the same layers by a compositor that shares no
/// line with the fast path.
///
/// Panics are the report, and every one of them names the input in hex: that is what a crash
/// becomes once it is minimised, and the named unit tests at the bottom of this file are what it
/// becomes once it is committed.
pub fn draw_sequence(data: &[u8]) {
    let mut input = Bytes::new(data);
    let w = 4 + u16::from(input.below(13));
    let h = 2 + u16::from(input.below(5));
    let mut screen = screen(w, h);
    // One hyperlink for the whole program. It is how a cell becomes extended without an operator's
    // arithmetic also being in the picture — the same fixture choice `crate::testing::pinned_extended`
    // exists for, and the reason OSC 8 is pinned above. The URI travels at the verb (architecture
    // ticket 21), so there is nothing to mint here and the table is reached by the first `restyle`
    // that names it.
    const URI: &str = "https://example.invalid/fuzz";
    let mut ids: Vec<LayerId> = Vec::new();
    let mut before = screen.reference();
    let mut frames = 0;
    let mut ops = 0;

    while !input.done() && ops < MAX_OPS && frames < MAX_FRAMES {
        ops += 1;
        match input.below(12) {
            0 if ids.len() < MAX_LAYERS => {
                let (z, rect) = (input.z(), input.rect(w, h));
                let opaque = input.byte() & 1 == 0;
                ids.push(screen.layers().add_content(z, rect, opaque));
            }
            1 if ids.len() < MAX_LAYERS => {
                let (z, rect) = (input.z(), input.rect(w, h));
                let toward = match input.byte() & 1 {
                    0 => Color::rgb(0, 0, 0),
                    _ => Color::rgb(255, 255, 255),
                };
                let mix = Mix::new(toward, u16::from(input.byte()));
                ids.push(screen.layers().add_operator(z, rect, mix));
            }
            2 if !ids.is_empty() && ids.len() < MAX_LAYERS => {
                let under = ids[usize::from(input.byte()) % ids.len()];
                let offset = (
                    i32::from(input.byte() % 5) - 2,
                    i32::from(input.byte() % 5) - 2,
                );
                let shadow = screen
                    .layers()
                    .add_shadow(under, offset, u16::from(input.byte()));
                ids.extend(shadow);
            }
            3 if !ids.is_empty() => {
                let at = usize::from(input.byte()) % ids.len();
                let id = ids.remove(at);
                screen.layers().remove(id);
            }
            4 if !ids.is_empty() => {
                let id = ids[usize::from(input.byte()) % ids.len()];
                let z = input.z();
                screen.layers().set_z(id, z);
            }
            5 if !ids.is_empty() => {
                let id = ids[usize::from(input.byte()) % ids.len()];
                let rect = input.rect(w, h);
                screen.layers().set_rect(id, rect);
            }
            6..=10 if !ids.is_empty() => {
                let id = ids[usize::from(input.byte()) % ids.len()];
                let verb = input.byte() % 5;
                let Some(mut view) = screen.layers().view(id) else {
                    continue;
                };
                match verb {
                    0 => {
                        let (x, y) = (input.coord(w), input.coord(h));
                        let (a, b, c) = (input.cluster(), input.cluster(), input.cluster());
                        let style = input.style();
                        view.text(x, y, &format!("{a}{b}{c}"), style);
                    }
                    1 => {
                        let rect = input.rect(w, h);
                        let (cluster, style) = (input.cluster(), input.style());
                        view.fill(rect, cluster, style);
                    }
                    2 => {
                        let (x, y) = (input.coord(w), input.coord(h));
                        let (cluster, style) = (input.cluster(), input.style());
                        view.set(x, y, cluster, style);
                    }
                    3 => {
                        let rect = input.rect(w, h);
                        let flags = input.byte();
                        let restyle = Restyle {
                            fg: (flags & 1 != 0).then(|| Color::indexed(input.byte())),
                            bg: (flags & 2 != 0).then(|| Color::indexed(input.byte())),
                            set: u16::from(input.byte()) << 3,
                            clear: u16::from(input.byte()) << 3,
                            ul: (flags & 4 != 0).then(|| Color::indexed(input.byte())),
                            // The extended bit from both ends: a link put on, and — when the same
                            // byte says so — taken off again, which is §3's *extended is a cost,
                            // not a state* and the one path that puts a cell back inline.
                            link: (flags & 8 != 0).then_some(match flags & 16 {
                                0 => crate::restyle::Link::Uri(URI),
                                _ => crate::restyle::Link::None,
                            }),
                        };
                        view.restyle(rect, &restyle);
                    }
                    // A clipped, offset child, which is the case a whole-layer verb cannot reach:
                    // `child` may not widen a clip (§4), so a pair it bisects keeps the half inside
                    // it and the composite is handed a broken pair it did not break.
                    _ => {
                        let rect = input.rect(w, h);
                        let (dx, dy) = (input.coord(w), input.coord(h));
                        let (x, y) = (input.coord(w), input.coord(h));
                        let (a, b) = (input.cluster(), input.cluster());
                        let style = input.style();
                        let mut child = view.child(rect);
                        let mut scrolled = child.scrolled(dx, dy);
                        scrolled.text(x, y, &format!("{a}{b}"), style);
                    }
                }
            }
            11 => {
                before = checkpoint(&mut screen, &before, data, frames);
                frames += 1;
            }
            // A verb whose guard refused it — no layers to draw into, or the layer ceiling reached.
            // It costs an op and draws nothing, which is what keeps a program that never adds a
            // layer from spending its whole frame budget on empty frames.
            _ => {}
        }
    }

    // The last frame, whether or not the program asked for it. Without this a program whose bytes
    // ran out mid-verb draws and is never checked, and the shortest inputs are exactly those.
    checkpoint(&mut screen, &before, data, frames);
}

/// One frame: composite it the slow way, present it, and assert gate #1's two halves of it.
///
/// Returns the reference picture, which is the next frame's *before*. Taking it from here rather
/// than compositing again at the top of the next frame is not an optimisation — it is what makes
/// *before* mean **after the last present** rather than *after whatever ran since*.
fn checkpoint(screen: &mut Screen, before: &Surface, data: &[u8], frame: u32) -> Surface {
    let after = screen.reference();
    screen.present();

    // **There is no allowance here any more, and its absence is the assertion.** This used to skip
    // every column at which the oracle's own picture violated §3's pairing invariant, because
    // architecture ticket 20 was open and a frame whose layers handed the composite a broken pair
    // had no defined content there. Ticket 20 is answered: the drawing verbs' repair is bounded by
    // the surface rather than by the clip, so a layer surface cannot arrive at the composite
    // already violating §3 and the case the allowance covered cannot be constructed. Four corpus
    // entries were kept for exactly this moment — 18, 19, 20 and 21 — and they are the gate on the
    // answer rather than a record of the question.
    for (x, y) in crate::reference::differences(before, &after) {
        assert!(
            covered(screen.runs(), x, y),
            "frame {frame}: ({x}, {y}) changed and no run reported it\n    before: {}\n     after: \
             {}\n     input: {}",
            row_picture(before, y),
            row_picture(&after, y),
            hex(data)
        );
    }

    let (w, h) = after.size();
    let frame_surface = screen.frame();
    for y in 0..h {
        for x in 0..w {
            // **The allowance, turned the right way up.** Where this file used to skip a column at
            // which the oracle's own picture left a pair in halves, it now asserts that no such
            // column exists. That is architecture ticket 20's answer read as a gate: the only way
            // to build one was a `View::child` whose clip bisected a pair, and the drawing verbs'
            // repair is bounded by the surface rather than by the clip, so there is no longer a
            // way. Asserted of the **oracle** and not of the frame, because the oracle is the one
            // that reports what the layers handed it.
            assert!(
                !orphaned(&after, x, y),
                "frame {frame}: the reference compositor's own picture leaves a pair in halves at \
                 ({x}, {y}), which architecture ticket 20 decided cannot happen\n    oracle: \
                 {}\n     input: {}",
                row_picture(&after, y),
                hex(data)
            );
            let (got, want) = (
                frame_surface.row(y)[usize::from(x)],
                after.row(y)[usize::from(x)],
            );
            if got == want {
                continue;
            }
            assert_eq!(
                got,
                want,
                "frame {frame}: the damage-tracked frame and the reference compositor disagree at \
                 ({x}, {y})\n     frame: {}\n    oracle: {}\n     input: {}",
                row_picture(frame_surface, y),
                row_picture(&after, y),
                hex(data)
            );
        }
    }
    after
}

/// Whether the cell at `(x, y)` is half of a pair whose other half is not there.
///
/// The frame's own edges count as absent neighbours, which is what `crate::reference`'s `repair`
/// says the other way round: a continuation in column zero and a wide head in the last column are
/// both orphans, and neither has a neighbour to be measured against.
fn orphaned(surface: &Surface, x: u16, y: u16) -> bool {
    let row = surface.row(y);
    let at = |x: i64| {
        (0..row.len() as i64)
            .contains(&x)
            .then(|| row[x as usize].grapheme)
    };
    let (x, g) = (i64::from(x), row[usize::from(x)].grapheme);
    let head_left = at(x - 1).is_some_and(crate::cell::GraphemeId::is_wide_head);
    let cont_right = at(x + 1).is_some_and(crate::cell::GraphemeId::is_continuation);
    (g.is_wide_head() && !cont_right) || (g.is_continuation() && !head_left)
}

/// One row, as a line a failure message can carry.
///
/// **The whole row and not the one cell**, because a one-cell diff of two `Cell` debug prints is the
/// least diagnosable failure this file could produce: every defect the draw-sequence target has found
/// so far was about a *pair* — a wide head and its continuation, one of them somewhere the other is
/// not — and the two cells either side of the disagreement are the ones that say which. `>` marks a
/// wide head and `<` its continuation, so a broken pair is visible without decoding a handle.
fn row_picture(surface: &Surface, y: u16) -> String {
    surface
        .row(y)
        .iter()
        .map(|cell| {
            let g = cell.grapheme;
            match (g.is_wide_head(), g.is_continuation(), g.is_empty()) {
                (true, _, _) => '>',
                (_, true, _) => '<',
                (_, _, true) => '_',
                _ => g.as_scalar().unwrap_or('?'),
            }
        })
        .collect()
}

/// The paste ceilings a program can pick, and every one of them is a boundary.
///
/// Zero truncates the first byte, one truncates the second, six is one below the terminator's own
/// length, and the last is the shipped default. A ceiling the input chose freely would mostly be a
/// large number, and the interesting arithmetic is all at the bottom.
///
/// The retention bound is a function of the ceiling, so **on the last of the five it is not the
/// binding constraint** — a megabyte of slack is not a bound any input libFuzzer generates will reach.
/// That arm is here for the parser's *default* behaviour rather than for the growth oracle, and the
/// four below it are where growth is actually pinned.
const PASTE_LIMITS: [usize; 5] = [0, 1, 6, 64, 1 << 20];

/// **Target 2: a byte stream into the input parser.**
///
/// Three arms over the same bytes, and they are not three rolls of the same dice:
///
/// 1. **One read.** The stream whole, ended once.
/// 2. **Chunked, with no boundary meaning.** The same bytes in chunks, `end_of_read` only after the
///    last. The events must be **identical** to the first arm's, and that equality is this file's
///    reading of *every byte consumed* — see the module documentation for why the literal
///    reading is a statement about a `for` loop.
/// 3. **Chunked, every boundary ended.** `end_of_read` after every chunk, which legitimately turns
///    a trailing `ESC` into an Escape key and so is outside the equality. It is run for the other
///    two oracles: it may not panic, and it may not grow.
///
/// Every arm is checked for retention afterwards, against a bound that is a function of
/// `MAX_SEQUENCE` and the paste ceiling and **not** of the input's length.
/// That is *no unbounded growth* stated as something a fuzzer can fail: a `String` state that kept
/// appending, or a paste that stopped checking its ceiling, is a bound violated by a long input and
/// invisible to every short one.
pub fn input_bytes(data: &[u8]) {
    let mut input = Bytes::new(data);
    let limit = PASTE_LIMITS[usize::from(input.below(5))];
    let chunk = usize::from(input.below(16)) + 1;
    let stream = &data[input.at.min(data.len())..];
    // One instant for every arm. `at` is the real clock and is stamped onto every event, so two
    // parses of the same bytes are never equal unless the caller supplies the moment — which is
    // exactly why the parser takes it rather than reading one.
    let at = Instant::now();

    let whole = feed(limit, &[stream], at, false);
    let split = feed(limit, &chunks(stream, chunk), at, false);
    assert_eq!(
        whole.events,
        split.events,
        "the same bytes in {chunk}-byte reads parsed differently from the whole stream — {}",
        hex(data)
    );

    let ended = feed(limit, &chunks(stream, chunk), at, true);
    for arm in [&whole, &split, &ended] {
        assert!(
            arm.held <= arm.bound,
            "the parser is holding {} bytes against a bound of {} — {}",
            arm.held,
            arm.bound,
            hex(data)
        );
        // Capacity and not only length, because the buffers are reused across sequences: one that
        // was cleared rather than shrunk has a length of zero and every byte it ever saw still
        // reserved. Four times the bound is slack for a doubling `Vec`; what matters is that the
        // number does not depend on how long the input was.
        assert!(
            arm.reserved <= arm.bound * 4 + 64,
            "the parser has {} bytes reserved against a bound of {} — {}",
            arm.reserved,
            arm.bound,
            hex(data)
        );
    }
}

/// `stream` in `size`-byte pieces, and the last piece is whatever is left.
fn chunks(stream: &[u8], size: usize) -> Vec<&[u8]> {
    match stream.is_empty() {
        true => vec![&stream[..0]],
        false => stream.chunks(size).collect(),
    }
}

/// What one arm of [`input_bytes`] produced, and what the parser was left holding.
struct Arm {
    events: Vec<Event>,
    held: usize,
    reserved: usize,
    bound: usize,
}

/// Feed `reads` through one parser, ending each read when `end_of_read` says so.
fn feed(limit: usize, reads: &[&[u8]], at: Instant, end_of_read: bool) -> Arm {
    let mut parser = Parser::new(limit);
    let mut events = Vec::new();
    let last = reads.len().saturating_sub(1);
    for (i, read) in reads.iter().enumerate() {
        let mut sink = |event: Event| events.push(event);
        parser.feed(read, at, &mut sink);
        if end_of_read || i == last {
            parser.end_of_read(at, &mut sink);
        }
    }
    let (held, reserved) = parser.retained();
    Arm {
        events,
        held,
        reserved,
        bound: parser.retention_bound(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// One target: the name its corpus directory has, and the function the fuzzer calls.
    type Target = (&'static str, fn(&[u8]));

    /// The two targets, by the name their corpus directory has.
    const TARGETS: [Target; 2] = [
        ("draw_sequence", draw_sequence),
        ("input_bytes", input_bytes),
    ];

    /// The smallest a committed corpus may be before it stops being a gate.
    ///
    /// A floor and not a count: the corpus **grows** — that is the whole design, a crash becomes a
    /// case and stays one — so an equality here would be a number edited on every soak that found
    /// something. What a floor catches is the direction that matters: a corpus emptied by a
    /// `.gitignore`, a rename, or a `cargo fuzz cmin` that rewrote it, which would leave this test
    /// green over nothing at all. The last of those is not hypothetical: `cmin` renames everything it
    /// keeps to a hash, which is why `fuzz/README.md` states what the committed corpus is and why a
    /// soak's grown pool is an uploaded artifact rather than a commit.
    const CORPUS_FLOOR: usize = 12;

    fn corpus() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus")
    }

    /// Every file of one target's corpus, sorted, so a failure is reproducible in its order.
    fn inputs(target: &str) -> Vec<PathBuf> {
        let dir = corpus().join(target);
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|_| panic!("{} is a committed directory", dir.display()))
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect();
        files.sort();
        files
    }

    /// **The gate: every committed input replays, and the oracle holds on all of them.**
    ///
    /// This is what the inversion buys. It is an ordinary test on the stable toolchain, it runs in
    /// every job that runs the suite, it takes milliseconds, and it can only fail for a reason
    /// somebody can reproduce from a file in the repository.
    #[test]
    fn the_committed_corpus_replays_as_an_ordinary_test() {
        let mut replayed = 0;
        for (target, run) in TARGETS {
            let files = inputs(target);
            assert!(
                files.len() >= CORPUS_FLOOR,
                "{target} has {} committed inputs, below the floor of {CORPUS_FLOOR}",
                files.len()
            );
            for file in files {
                let data = std::fs::read(&file).expect("a committed corpus file reads");
                // The panic a target raises is the report, and it names the input in hex. This adds
                // the one thing hex cannot say: which file on disk to open.
                std::panic::catch_unwind(|| run(&data)).unwrap_or_else(|payload| {
                    let what = payload
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| payload.downcast_ref::<&str>().copied())
                        .unwrap_or("a panic with no message");
                    panic!("{} failed to replay: {what}", file.display());
                });
                replayed += 1;
            }
        }
        assert!(replayed >= CORPUS_FLOOR * TARGETS.len());
    }

    /// **The corpus directory holds exactly the two targets, and nothing else.**
    ///
    /// A third directory is an input nothing replays: `cargo fuzz run` writes into
    /// `corpus/<target>/`, so a target renamed without its corpus moving leaves the old one behind
    /// looking exactly like a corpus that is still a gate.
    #[test]
    fn the_corpus_directory_holds_exactly_the_two_targets() {
        let mut on_disk: Vec<String> = std::fs::read_dir(corpus())
            .expect("fuzz/corpus is committed")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        on_disk.sort();
        let mut expected: Vec<String> =
            TARGETS.iter().map(|(name, _)| (*name).to_owned()).collect();
        expected.sort();
        assert_eq!(on_disk, expected);
    }

    /// **The fuzz workspace declares a target per corpus directory, and no more.**
    ///
    /// The corpus is the gate, so a target with no corpus is a soak slot spent on inputs nothing
    /// replays — and a corpus with no target is a directory that stopped growing on the day the
    /// target was renamed. Both directions, read out of `fuzz/Cargo.toml` itself.
    #[test]
    fn every_corpus_has_a_fuzz_target_and_every_target_has_a_corpus() {
        let manifest = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/Cargo.toml"),
        )
        .expect("the fuzz workspace's manifest is committed");
        let declared: Vec<&str> = manifest
            .lines()
            .filter_map(|line| line.trim().strip_prefix("name = \""))
            .filter_map(|rest| rest.split('"').next())
            .filter(|name| *name != "vitui-engine-fuzz")
            .collect();
        let mut expected: Vec<&str> = TARGETS.iter().map(|(name, _)| *name).collect();
        let mut declared = declared;
        declared.sort_unstable();
        expected.sort_unstable();
        assert_eq!(
            declared, expected,
            "`fuzz/Cargo.toml`'s `[[bin]]` targets and this file's corpus directories disagree"
        );
    }

    // -----------------------------------------------------------------------------------------
    // Minimised crashes, committed as named unit tests.
    // -----------------------------------------------------------------------------------------
    //
    // A crash arrives here as a `#[test]` whose body is one call with the minimised input inline,
    // **named for the defect and not for the byte string** — `fuzz/README.md` states the procedure.
    // The corpus keeps a copy of the same input, because the two are not redundant: the test names
    // one input for ever and the corpus entry is what the next soak mutates.

    /// **An operator's first column, over a pair the frame's own clamp bisected.**
    ///
    /// Found by this target on its first run, at 8 619 executions, and minimised from 68 bytes to
    /// these 37 by `cargo fuzz tmin`. The defect is in `crate::layer`'s `is_continuation`, whose
    /// documentation carries the full account; the short version is that it asked the **layers**
    /// whether a `CONTINUATION` still had its head, and a layer extends past the screen while the
    /// frame does not. A head in column `-1` is a cell the layer has and the frame has blanked, so
    /// the operator declined a column that was, by then, an ordinary space — an unshaded hole in a
    /// scrim over a partially clipped layer of CJK.
    ///
    /// The program, decoded: a 14x4 screen; one opaque content layer at `(-1, 2, 4, 4)` filled with
    /// a wide cluster, so screen column 0 holds a continuation whose head is off-screen; a shadow
    /// raised above it; and a lifting operator over `(0, -1, 4, 5)`, whose first column is that one.
    ///
    /// Every gate on the register was green on this. What it takes to see it is damage **disjoint
    /// from the layer whose edge is in question**, which is the same finding recorded earlier and
    /// the reason this target exists rather than a thirteenth scene.
    #[test]
    fn an_operator_reaching_over_a_pair_the_frame_clamped_on_the_left() {
        draw_sequence(&[
            10, 2, 0, 0, 3, 114, 4, 4, 4, 14, 0, 0, 59, 0, 6, 0, 1, 4, 4, 14, 4, 3, 4, 4, 3, 4, 11,
            5, 1, 7, 4, 3, 4, 5, 1, 4, 11,
        ]);
    }

    /// **Architecture the case, and now the gate on its answer.**
    ///
    /// This input is why the ticket could be answered at all. It found the case from a direction the
    /// ticket said nothing would come from — *no scene produces the case, and every gate is green* —
    /// and it showed more than the ticket knew: the frame **kept** the orphan on frames one and two
    /// and **blanked** it on frame three, same stack, same surfaces, because `mend` runs at the
    /// edges of the damaged span and on that frame a span edge landed on it. One question answered
    /// both ways depending on what else changed, which is that same shape.
    ///
    /// The answer is that the orphan never reaches the composite: the
    /// drawing verbs' repair is bounded by the surface rather than by the clip, because three
    /// terminals were asked and all three blank the orphaned half themselves. So this program now
    /// produces a well-formed frame on every one of its frames, and [`checkpoint`] asserts that
    /// rather than excusing it — see the note there where the allowance used to be.
    ///
    /// **It keeps its old number and its bytes and loses only the outcome from its name.** Entries
    /// 19, 20 and 21 keep theirs entirely: each names what it was found doing, and a corpus entry is
    /// evidence with a date on it. What they were found doing is what makes them a gate now.
    ///
    /// The program, decoded: a 10x2 screen; one opaque layer; three wide clusters written across row
    /// zero; then a `child` at `(1, 0, 3, 2)` writing two more, so the clip bisects the pair at
    /// column 0; then three more frames, the last of which damages a run whose left edge is that
    /// column. The head at column 0 is blanked by the verb now, inside the layer's own surface,
    /// before the composite has an opinion about it.
    #[test]
    fn a_pair_a_child_clip_bisected_is_mended_by_the_verb_before_the_composite_sees_it() {
        draw_sequence(&[
            6, 0, 0, 3, 4, 4, 10, 2, 0, 6, 0, 0, 4, 4, 5, 6, 5, 7, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 11, 6, 0, 4, 5, 4, 3, 2, 4, 4, 4, 4, 5, 6, 7, 0, 0, 11, 10, 2, 6, 0, 7, 0, 0, 11,
            10, 2, 0, 11, 6, 0, 122, 4, 5, 4, 3, 2, 122, 4, 4, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 4, 4,
            5, 6, 5, 7, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 11, 6, 0, 4, 5, 1, 4, 4, 14, 5, 2,
            7, 0, 0, 0, 4, 11,
        ]);
    }

    /// **The mirror image: a rectangle that ends off-screen, reaching back onto it.**
    ///
    /// Found by the same target ten minutes later, **after the case above was fixed**, and that
    /// order is the finding rather than a detail: the first fix bounded one half of the pair and the
    /// differential came straight back with the other. `crate::layer`'s rule is one function now —
    /// `pair_survives` — asked of the column itself by `is_head` and of the column to its left by
    /// `is_continuation`, because two spellings of one sentence is how the second defect existed at
    /// all.
    ///
    /// The program, decoded: a 10x6 screen; a **non-opaque** content layer at `(-3, 2, 7, 2)` whose
    /// text is three wide clusters, so the pair straddling the frame's left edge has its head at
    /// column `-1`; and a lifting operator at `(-4, -3, 4, 8)`, whose rectangle ends at column `-1`
    /// and so covers no cell of the screen at all. It shaded column zero anyway, on the strength of
    /// a head the frame had already blanked.
    #[test]
    fn an_operator_reaching_back_onto_the_frame_from_off_its_left_edge() {
        draw_sequence(&[
            6, 4, 10, 255, 7, 0, 0, 1, 20, 7, 11, 5, 1, 0, 0, 1, 4, 8, 11, 4, 6, 0, 0, 4, 4, 5, 6,
            5, 8, 4, 8, 11,
        ]);
    }

    /// The decoder's own floor: an empty program still presents one frame and asserts it.
    ///
    /// It is the shortest input libFuzzer will ever hand either target, and both of them have to
    /// mean something for a corpus of short inputs to be worth replaying.
    #[test]
    fn an_empty_input_is_a_program_and_a_stream() {
        draw_sequence(&[]);
        input_bytes(&[]);
    }

    /// A program that runs out of bytes mid-verb is a shorter program, not a different one.
    #[test]
    fn a_truncated_program_still_asserts_its_last_frame() {
        for cut in 1..24 {
            let program: Vec<u8> = (0..cut).map(|b| b as u8).collect();
            draw_sequence(&program);
        }
    }

    /// The decoder is a pure function of its bytes: two runs of one input do the same thing.
    ///
    /// A generator that read a clock, a hash seed or an address would make every committed input a
    /// different program on the next run, and the corpus would be a gate over nothing.
    #[test]
    fn the_same_input_is_the_same_program_twice() {
        let program: Vec<u8> = (0..200u32).map(|b| (b * 37 % 251) as u8).collect();
        draw_sequence(&program);
        draw_sequence(&program);
        input_bytes(&program);
        input_bytes(&program);
    }
}
