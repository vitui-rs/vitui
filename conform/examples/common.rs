//! The half of the live arm that is not the terminal: the scene, the handshake, and the comparison.
//!
//! **Not a target**, and `Cargo.toml` is where that is arranged: `autoexamples = false` with the
//! arms declared one by one, so this file is a module the examples include rather than an example of
//! its own that fails for having no `main`.
//!
//! # Why it exists, and it is a later pass that forced it
//!
//! Stage 1 had one arm and one file, and the scene lived in it. An earlier pass asks for the eleven
//! attribute facts *on at least two families*, which makes the scene and the comparison the parts
//! that must be **identical** across arms while the launching, capturing and closing are the parts
//! that cannot be. A second copy of `scene01` would make a disagreement between the arms
//! unattributable: it could be the emulators, or it could be the two copies having drifted.
//!
//! There are four arms now — Ghostty, Ghostty-via-tmux, tmux and kitty — and the third emulator was
//! what made the split pay for itself twice over: kitty disagrees on two rows and cannot be asked a
//! third, and none of those three sentences would be worth anything if its scene were its own copy.
//!
//! So the split here is not tidiness. It is the thing that lets a row say *tmux dropped this bit and
//! Ghostty did not* and mean it.
//!
//! # The arm-specific half, stated as an interface
//!
//! An arm brings four things and nothing else: a way to start [`scene_argv`] somewhere a terminal
//! can see it, a way to read that terminal's screen back as bytes, a way to shut it down, and an
//! [`Arm`] describing itself for the report — including [`Arm::not_compared`], the rows it will not
//! compare and why, which it must declare *before* the run. Everything else — what is drawn, when it
//! is safe to photograph, and what counts as agreement — is here.

use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use vitui_conform::{
    Attrs, Bracket, Colour, Dialect, Dump, FLUSH_CEILING_MS, FLUSH_FLOOR_MS, FLUSH_OPENS,
    FLUSH_RESOLUTION_MS, ModeReply, Probe, Reply, Style, Underline, cursor_reports,
    default_colours, flush_bracket, mode_reports, next_delay, parse,
};
use vitui_engine::{Clock, Config, Engine, Rect, Style as EngineStyle, Wake, width_of};

/// How long to wait for the scene to present a frame and then stand still.
pub const READY_TIMEOUT: Duration = Duration::from_secs(20);

/// How often scene 04's raw-byte probe repaints itself.
///
/// **Not a settle time and not tuned to one.** It is a repaint cadence: the picture is redrawn with
/// an erase and absolute addressing, so a window that has just changed size is correct again within
/// one cycle and an echoed keystroke is gone within one cycle. The driver still waits
/// [`QUIESCENT`] before it captures, which is five of these.
pub const REPAINT: Duration = Duration::from_millis(100);

/// How many consecutive empty reads scene 05 takes as *this is not a terminal* rather than as the
/// read timeout it configured.
///
/// **Not a second deadline.** See the use — on a real tty this many empty reads cannot happen
/// inside [`ANSWER_TIMEOUT`], so reaching it is evidence about the descriptor and never about how
/// long the terminal took.
const EMPTY_READS: usize = 64;

/// How long scene 05 waits for the terminal to answer its batch.
///
/// **It must be shorter than [`READY_TIMEOUT`], and that is an invariant rather than a preference**:
/// the scene writes its stamp *after* the answers, so a scene that waited longer than the driver
/// would be given up on while it was still listening — and the driver would report *the scene never
/// presented a frame*, which is the readiness timeout blaming the scene for a terminal that was
/// merely slow.
///
/// It is a ceiling and not a settle time. The read stops on the sentinel's reply, which is an
/// observed condition; this is only what happens when there is not going to be one.
pub const ANSWER_TIMEOUT: Duration = Duration::from_secs(5);

/// How long the scene's stamp must stay unchanged before the screen counts as settled.
///
/// **Not tuned to a measurement**, which would make it the flaky-test shape this repository refuses.
/// It is a floor: a window-server resize storm is tens of milliseconds, and this is an order of
/// magnitude above it. If it ever needs raising, the run says so — it fails with the reason rather
/// than photographing a moving screen.
pub const QUIESCENT: Duration = Duration::from_millis(500);

/// Every scene, in the order an arm runs them.
///
/// **An arm runs all of them or it is not a run**, and one report per arm holds a section for each.
/// The rule is `SCENES.md`'s and it is the same one that gives each arm its own file: a scene whose
/// section is missing reads as a win, and that is the single easiest way for this directory to
/// become dishonest. There is deliberately no flag to run one.
pub const SCENES: &[&str] = &["01", "04", "05", "06"];

/// The private mode scene 06 is about.
///
/// One constant rather than a literal in five places, and it is the number `serial.rs` writes into
/// `\x1b[?2026h` — spelled here rather than imported, because an instrument that took the mode
/// number from the engine would be asking the terminal about whatever the engine happened to say.
pub const SYNC_MODE: u16 = 2026;

/// How long scene 06 waits after closing a block before opening the next one.
///
/// The terminal is asked to reset the mode and then asked whether it did, so this is not a settle
/// time for the *reset* — that is observed. It is slack between two opens, so a timer armed by the
/// previous one cannot still be running when the next one starts.
const FLUSH_BETWEEN: Duration = Duration::from_millis(150);

/// One row of scene 06, part A: a question about the mode and the state DECRPM defines for it.
pub struct Mode {
    /// What has been done to the mode by the time this question is asked. Also the row's identity
    /// in the report.
    pub label: &'static str,
    /// What the row is evidence about, in the report's own words.
    pub asks: &'static str,
    /// The state the standard says the reply must carry.
    ///
    /// **Compared and not surveyed**, which is where this scene differs from 05: these are
    /// DECRPM's own values, so a terminal that answers otherwise is wrong by the definition of the
    /// reply it sent rather than by a table this repository chose. Scene 05's twelve are a survey
    /// precisely because `ucd.rs` makes the engine's tables authoritative and there is nothing for
    /// a terminal to be wrong *against*.
    pub want: vitui_conform::ModeState,
}

/// Scene 06, part A — does the terminal have mode 2026, and does its state machine track it.
///
/// # Five questions in one batch, because the answers are only meaningful in sequence
///
/// A terminal that reports the mode set is not interesting; a terminal that reports it set *after
/// it was asked to reset it* is. So the batch is one write — ask, set, ask, reset, ask, set, set,
/// ask, reset, ask — with `CSI c` behind it, and the five answers are read positionally. That is
/// what makes `mode_reports` refuse a short batch rather than pad it: a lost answer does not blank
/// a row, it reports every later row under the wrong question.
///
/// # The last two rows are a mode that is not a counter
///
/// DEC private modes are set and reset, not pushed and popped, so two `h` and one `l` leave the
/// mode **reset**. A terminal that counted them would hold a frame back past the `l` the engine
/// sent, and the engine's design's twenty bytes of framing are a balanced pair per frame — so a counting terminal
/// would be one where a frame that opened a block twice never appeared. Nothing in this repository
/// does that, which is exactly why nothing here would notice.
pub fn scene06() -> [Mode; 5] {
    use vitui_conform::ModeState::{Reset, Set};
    [
        Mode {
            label: "before",
            asks: "whether the terminal recognises the mode at all, and what it says before \
                   anything has been done to it. A terminal without synchronised output answers \
                   `not recognised (0)` here, which is a legitimate answer and not a defect",
            want: Reset,
        },
        Mode {
            label: "while-open",
            asks: "whether `CSI ? 2026 h` reached the state machine. This is the row that \
                   separates a terminal that *has* the mode from one that parses the sequence and \
                   throws it away",
            want: Set,
        },
        Mode {
            label: "after-close",
            asks: "whether `CSI ? 2026 l` reached it too. A terminal that opens and never closes \
                   is one where the engine's own frame framing leaves the mode set for ever",
            want: Reset,
        },
        Mode {
            label: "opened-twice",
            asks: "whether a second `h` over an already-set mode is still simply set",
            want: Set,
        },
        Mode {
            label: "closed-once",
            asks: "whether one `l` undoes two `h`. A DEC private mode is not a counter, and a \
                   terminal that made it one would hold a frame past the close the engine sent",
            want: Reset,
        },
    ]
}

/// One row of scene 01: an attribute, the label under it, and what the dump must say.
pub struct Case {
    /// The label written into the row. Also the row's identity in the report.
    pub label: &'static str,
    /// The bit this row lights, named the way the engine's public surface names it.
    pub verb: &'static str,
    /// The engine style the scene draws with.
    pub draw: fn(EngineStyle) -> EngineStyle,
    /// The flags the dump must report, **exactly** — an extra one is a disagreement too.
    pub attrs: Attrs,
    /// The underline the dump must report.
    pub underline: Underline,
}

/// Scene 01, one row per attribute bit of the engine's style word.
///
/// **Eleven rows because the style word spends eleven bits on attributes** — eight flags and a
/// three-bit underline field — and each row here lights exactly one of them. That is what makes the
/// assertion eleven independent booleans rather than eleven overlapping ones, and it is why the
/// three underline rows are single, double and dotted: `4:1`, `4:2` and `4:4` are the three values
/// with one bit set. Curly (`4:3`) and dashed (`4:5`) light two bits each and so cannot be a
/// per-bit row; they are covered by the committed-fixture tests instead.
///
/// `attrs_dropped` is the field this feeds, and it is the field with **no query**: the eleven facts
/// are in the capability set precisely because nothing can ask a terminal for them. A dump is the
/// only thing that can, which is why this is scene 01 and not scene 07.
pub fn scene01() -> [Case; 11] {
    [
        Case {
            label: "bold",
            verb: "Style::bold",
            draw: EngineStyle::bold,
            attrs: Attrs::BOLD,
            underline: Underline::None,
        },
        Case {
            label: "dim",
            verb: "Style::dim",
            draw: EngineStyle::dim,
            attrs: Attrs::DIM,
            underline: Underline::None,
        },
        Case {
            label: "italic",
            verb: "Style::italic",
            draw: EngineStyle::italic,
            attrs: Attrs::ITALIC,
            underline: Underline::None,
        },
        Case {
            label: "reverse",
            verb: "Style::reverse",
            draw: EngineStyle::reverse,
            attrs: Attrs::REVERSE,
            underline: Underline::None,
        },
        Case {
            label: "blink",
            verb: "Style::blink",
            draw: EngineStyle::blink,
            attrs: Attrs::BLINK,
            underline: Underline::None,
        },
        Case {
            label: "strikethru",
            verb: "Style::strikethrough",
            draw: EngineStyle::strikethrough,
            attrs: Attrs::STRIKE,
            underline: Underline::None,
        },
        Case {
            label: "conceal",
            verb: "Style::conceal",
            draw: EngineStyle::conceal,
            attrs: Attrs::HIDDEN,
            underline: Underline::None,
        },
        Case {
            label: "overline",
            verb: "Style::overline",
            draw: EngineStyle::overline,
            attrs: Attrs::OVERLINE,
            underline: Underline::None,
        },
        Case {
            label: "under-sgl",
            verb: "Style::underline",
            draw: EngineStyle::underline,
            attrs: Attrs::default(),
            underline: Underline::Single,
        },
        Case {
            label: "under-dbl",
            verb: "Style::underline_double",
            draw: EngineStyle::underline_double,
            attrs: Attrs::default(),
            underline: Underline::Double,
        },
        Case {
            label: "under-dot",
            verb: "Style::underline_dotted",
            draw: EngineStyle::underline_dotted,
            attrs: Attrs::default(),
            underline: Underline::Dotted,
        },
    ]
}

/// One row of scene 04: what is written into it, and what the terminal must show afterwards.
///
/// **The row is read as text, and that is the whole trick.** A grid-to-text dump emits a
/// double-width glyph with no padding cell and no continuation marker, so *what is at column 3* is
/// not a question this instrument can ask — a later pass predicted that and scene 02 is the
/// scene kept to prove it. ASCII sentinels convert it into one it can: put `A` and `B` to the left
/// and `C` and `D` to the right, and the row read as a string says which columns survived without
/// anyone deriving a width. The width tables are the thing under test, so they may not be in the
/// measuring loop.
pub struct Pair {
    /// The row's identity in the report.
    pub label: &'static str,
    /// Where the cursor goes within the row, and what is written there, in order.
    pub emit: &'static [(u16, &'static str)],
    /// What the row must read as, trailing padding ignored.
    pub want: &'static str,
    /// A cluster whose **style is reported and never compared**, by index into the row's text.
    ///
    /// Only rule 3's row uses it, and it is reported rather than compared because **the three
    /// families answer it differently** — see that row's `asks`. There is no single expectation to
    /// hold them to, and inventing one would make two of three arms carry a permanent `FAILED` for
    /// something that is not a defect. What gates it instead is `tests.rs`, over the committed
    /// captures, one assertion per terminal: the live arm reports and the fixture gates, which is
    /// the trade this whole directory is built on.
    pub report_style: Option<usize>,
    /// What the row is asking, in one clause, for the report.
    pub asks: &'static str,
}

/// Scene 04 — a pair bisected, and what the terminal does with the orphan.
///
/// # This is the scene that answers an architecture decision, and it cannot be drawn with the engine
///
/// Every other scene here drives the engine and compares what came back. This one must not, and the
/// reason is the answer itself: the engine's drawing verbs repair a bisected pair before the bytes
/// are ever serialised, so an engine-driven scene could only ever photograph the repair. **The
/// question is what the terminal does when it is handed the bytes anyway**, which is what the
/// engine's mirror would have believed had the repair stopped at the clip.
///
/// `SCENES.md` licenses this in as many words — *a scene is a described picture, and an arm may
/// reach it any way it likes* — and it is the same discipline the kitty arm's conceal row came out
/// of: run the control before the instrument, or *the terminal does not do it* and *the instrument
/// cannot see it* have no way to be told apart.
///
/// It is also why this scene keeps its value after the answer, where `--through-tmux`'s overline row
/// lost its. An arm that asked the engine what to expect would be checking the engine against
/// itself; these six rows ask nothing of the engine at all.
///
/// # The rows
///
/// Each is `AB漢CD` — `A` at 0, `B` at 1, the wide glyph across 2 and 3, `C` at 4, `D` at 5 — and
/// then one write over one half of it.
pub fn scene04() -> [Pair; 6] {
    [
        Pair {
            label: "control",
            emit: &[(0, "AB漢CD")],
            want: "AB漢CD",
            report_style: None,
            asks: "nothing is overwritten, so a disagreement here says the other five rows are \
                   about the capture rather than about the terminal",
        },
        Pair {
            label: "over-cont",
            emit: &[(0, "AB漢CD"), (3, "x")],
            want: "AB xCD",
            report_style: None,
            asks: "**the ticket's own case.** A narrow cluster lands on the continuation at column \
                   3. Does the terminal blank the head at column 2, or leave a wide head with \
                   nothing after it?",
        },
        Pair {
            label: "over-head",
            emit: &[(0, "AB漢CD"), (2, "x")],
            want: "ABx CD",
            report_style: None,
            asks: "the mirror image: a narrow cluster lands on the head at column 2. Does the \
                   terminal blank the continuation at column 3?",
        },
        Pair {
            label: "over-wide",
            emit: &[(0, "AB漢CD"), (3, "漢")],
            want: "AB 漢D",
            report_style: None,
            asks: "rule 5 — a wide cluster landing across an existing pair, which orphans a half at \
                   each end at once",
        },
        Pair {
            label: "keeps-style",
            emit: &[(0, "AB"), (2, "\u{1b}[41m漢\u{1b}[0m"), (4, "CD"), (3, "x")],
            want: "AB xCD",
            report_style: Some(2),
            asks: "**what does the blanked half wear, and the askable families split two-two.** The \
                   wide glyph carries a red background and the `x` does not. kitty 0.48.2 and \
                   WezTerm 20240203 keep the orphan's own background; Ghostty 1.3.1 and tmux 3.7c \
                   blank it to the SGR state in force; Terminal.app 2.15 cannot be asked. Reported, \
                   never compared — there is no single right answer to hold an arm to, and this \
                   row's value is that sentence rather than a tick",
        },
        Pair {
            label: "ruler",
            emit: &[(0, "0123456789")],
            want: "0123456789",
            report_style: None,
            asks: "the row that makes a mis-sized or reflowed capture loud. This scene reports no \
                   surface size — a raw-byte probe has none to report — so a ruler that is short, \
                   wrapped or absent is what stands in for the handshake",
        },
    ]
}

/// One row of scene 05: a cluster, and what is done with the number the terminal answers.
pub struct Glyph {
    /// The row's identity in the report.
    pub label: &'static str,
    /// The cluster the scene writes, once, at column 1 of one row.
    pub cluster: &'static str,
    /// The advance this row is **compared** against, hand-written here, or `None` to survey it.
    ///
    /// # Why three rows carry a number and twelve do not
    ///
    /// The engine's own tables are **authoritative by decision**: `ucd.rs` says so in as many words,
    /// and the engine's design's `CHA`-after-non-ASCII rule is what bounds the disagreement instead of following
    /// it. So a terminal that answers 6 for a ZWJ family emoji is not misbehaving in any sense this
    /// repository acts on — there is no mechanism that would read such a `quirks.rs` row — and a
    /// `FAILED` there would be this instrument inventing a defect.
    ///
    /// What is still a defect is the *instrument* not working, and that is what the three compared
    /// rows are for. They are hand-written and **not** asked of the engine: a row that took its
    /// expectation from `width_of` would be checking the engine against itself, which is the
    /// arrangement `conform/` exists to break.
    pub compared: Option<u16>,
    /// What the row is asking, in one clause, for the report.
    pub asks: &'static str,
}

/// Scene 05 — what does this emulator think this cluster is worth.
///
/// # The one measurement in this directory with none of our tables in the loop
///
/// Every other scene reads a *dump*, and scene 02 is kept for establishing what a dump cannot say: a
/// grid-to-text capture emits no padding cell for a double-width glyph, so *how many columns did
/// that take* is not a question it can answer. Deriving it would need a width table, and a width
/// table is the thing under test.
///
/// A cursor report can answer it. The scene homes the cursor to column 1, writes one cluster, and
/// asks `CSI 6n`; the column that comes back is the **emulator's** UAX #11 verdict, reached by the
/// emulator's tables and reported by the emulator.
///
/// # It is a survey and not a comparison, and that is a decision rather than a shortfall
///
/// `ucd.rs`'s module docs already say the engine does not follow the terminal here — *our tables are
/// authoritative*, with three named policies and the engine's design's `CHA`-after-non-ASCII rule bounding what
/// a disagreement can cost. What that paragraph cites for the disagreement is a **survey of 23
/// terminals in a research document**. This scene is the first thing in this repository to observe
/// any of it, on the families the engine's design puts in tier 1.
pub fn scene05() -> [Glyph; 15] {
    [
        Glyph {
            label: "ascii",
            cluster: "A",
            compared: Some(1),
            asks: "the control. A terminal that disagrees here is not answering about widths at \
                   all, and every other row of this table is about the instrument rather than \
                   about the emulator",
        },
        Glyph {
            label: "ascii-pair",
            cluster: "AB",
            compared: Some(2),
            asks: "**the control the control needs.** A probe that reported a constant would pass \
                   the row above; two columns is what proves the number moves with what was \
                   written",
        },
        Glyph {
            label: "cjk",
            cluster: "漢",
            compared: Some(2),
            asks: "UAX #11 `W`, unambiguous, and the one wide verdict no terminal in spec §10's \
                   tier 1 is known to differ on. Compared rather than surveyed because a terminal \
                   that answers 1 here is one this engine's `CHA` rule could not bound",
        },
        Glyph {
            label: "hangul",
            cluster: "가",
            compared: None,
            asks: "wide by the same class as the row above, reached through a different block",
        },
        Glyph {
            label: "fullwidth",
            cluster: "Ａ",
            compared: None,
            asks: "U+FF21, UAX #11 `F` — the class that is wide for being a fullwidth *form* rather \
                   than for being East Asian",
        },
        Glyph {
            label: "ambiguous",
            cluster: "☂",
            compared: None,
            asks: "**UAX #11 class `A`, and the engine's answer is a policy rather than a \
                   standard**: `ucd.rs` pins ambiguous width as *narrow* by name. This is the row \
                   where a terminal running with an East Asian locale is entitled to disagree",
        },
        Glyph {
            label: "combining",
            cluster: "e\u{301}",
            compared: None,
            asks: "*a cluster's width is its base's width, never the sum of its code points* — the \
                   second of `ucd.rs`'s three pinned policies. Windows Terminal is recorded in that \
                   file as drawing a combining mark at width 1 of its own",
        },
        Glyph {
            label: "zero-width",
            cluster: "\u{200B}",
            compared: None,
            asks: "a cluster the cursor may not move for at all. The one row whose interesting \
                   answer is that nothing happened, which is also the one an instrument reading a \
                   missing reply would report by accident",
        },
        Glyph {
            label: "emoji",
            cluster: "\u{1F44D}",
            compared: None,
            asks: "`Emoji_Presentation`, one scalar, no selector — the emoji case with nothing else \
                   in it",
        },
        Glyph {
            label: "vs16",
            cluster: "\u{2764}\u{FE0F}",
            compared: None,
            asks: "**the headline of `ucd.rs`'s survey**: *only 7 of 23 surveyed widen a VS16 emoji \
                   correctly*. A default-text scalar plus U+FE0F, which forces emoji presentation \
                   and with it two columns",
        },
        Glyph {
            label: "vs15",
            cluster: "\u{2764}\u{FE0E}",
            compared: None,
            asks: "the third pinned policy: **VS15 does not change a width**, only a presentation. \
                   The same base as the row above with the other selector, so the two rows read \
                   together are what say whether a terminal has selectors at all",
        },
        Glyph {
            label: "zwj-family",
            cluster: "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
            compared: None,
            asks: "the second named disagreement in `ucd.rs`: *kitty sums a ZWJ family emoji to 6 \
                   where the answer is 2*. Three emoji joined by two ZWJs, one cluster",
        },
        Glyph {
            label: "flag",
            cluster: "\u{1F1EF}\u{1F1F5}",
            compared: None,
            asks: "two regional indicators, which the engine counts as a pair rather than as two \
                   clusters — the one place `cluster_width` looks past the base",
        },
        Glyph {
            label: "skin-tone",
            cluster: "\u{1F44D}\u{1F3FD}",
            compared: None,
            asks: "a base and a modifier, where the modifier is itself an emoji scalar. The row \
                   above answers whether a terminal joins; this one answers whether it modifies",
        },
        Glyph {
            label: "keycap",
            cluster: "1\u{FE0F}\u{20E3}",
            compared: None,
            asks: "an ASCII base carried into emoji presentation by a selector and a combining \
                   enclosing keycap — the sequence whose base is one column on its own",
        },
    ]
}

/// How many rows the scene declares, which is what the parser refuses a short capture against.
pub fn rows_expected(which: &str) -> usize {
    match which {
        "01" => scene01().len(),
        "04" => scene04().len(),
        // **Not a missing arm.** Scene 05's capture is not a screen: the terminal answers in band,
        // and the refusal that stands where a short screen's does is `cursor_reports`' own. See
        // [`capture`].
        "05" => panic!("scene 05 is not photographed — see `capture`"),
        "06" => panic!("scene 06 is not photographed — see `capture`"),
        other => panic!("no such scene: {other}"),
    }
}

/// The scene's command line, as an **argv**.
///
/// Built from [`std::env::current_exe`] so the scene is this same build. That is not a trick to save
/// a file: it makes the two halves the same build by construction, where a sibling binary path can
/// silently be yesterday's.
///
/// **An argv and not a string, because the launchers disagree about which they take** and the
/// difference is not cosmetic. kitty execs what it is handed; given one string it looks for a file
/// whose name ends in `--scene 01`, finds none, and says nothing — which arrives twenty seconds
/// later as the readiness timeout blaming the scene for the launcher. AppleScript and tmux take a
/// command *line*, and both join this themselves rather than being handed a guess at a quoting rule
/// for a launcher this file cannot see.
///
/// # Errors
///
/// Whatever [`std::env::current_exe`] failed with.
pub fn scene_argv(which: &str) -> Result<Vec<String>, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    Ok(vec![
        exe.display().to_string(),
        "--scene".into(),
        which.into(),
    ])
}

// ── The scene half: inside the terminal ──────────────────────────────────────────────────────────

/// Draw the scene, say so, and keep it right until the arm shuts the terminal down.
///
/// **It redraws rather than drawing once**, and that is not defensive coding — it is the answer to a
/// defect the first live run produced. A Ghostty window settles its size *after* the process inside
/// it starts, so the first frame can be painted at one geometry and then reflowed at another; that
/// run photographed a screen whose top two rows had scrolled off, and every row of the report was
/// wrong by two. The readiness handshake proves a frame was presented. It cannot prove the window
/// has stopped moving, so the stamp below carries a frame counter and the driver waits for it to
/// stand still.
///
/// The tmux arm does not need that — `capture-pane` runs against a pane whose size the driver set
/// with `-x` and `-y` — and it runs the same code anyway. An arm that only *believes* it set the
/// size still gets told when it did not.
pub fn scene(which: Option<&str>) {
    let ready = std::env::var("CONFORM_READY").expect("the driver sets CONFORM_READY");
    match which {
        Some("01") => scene01_frames(&ready),
        Some("04") => scene04_bytes(&ready),
        Some("05") => scene05_cpr(&ready),
        Some("06") => scene06_decrqm(&ready),
        other => panic!("no such scene: {other:?} — see SCENES"),
    }
}

/// Scene 01: attach the engine and hold the eleven rows right until the arm shuts the terminal down.
fn scene01_frames(ready: &str) {
    let engine = Engine::new(Config {
        clock: Clock::System,
        max_frame_rate: 60.0,
        // The terminal this writes to is the one being photographed, so a warning printed here would
        // become content in the capture.
        overrun_report: Some(Box::new(std::io::sink())),
        ..Default::default()
    });
    let Ok((mut screen, _wake)) = engine.attach() else {
        // Nothing can be drawn and nothing can be said on this terminal. The driver's readiness
        // timeout is what turns this into a `FAILED` row.
        return;
    };

    let mut frame = 0u64;
    {
        // Same declaration the app template makes, and for the same reason: a debug build panics on
        // the first unexcused overrun, and the cold-start frame is legitimately slower than a budget.
        let permit = screen.permit_slow("the cold-start frame");
        draw(&mut screen);
        drop(permit);
    }
    stamp(ready, &mut frame, &screen);

    loop {
        if screen.wait() == Wake::Quit {
            return;
        }
        // Drained and ignored. A window steals focus when it opens, so whatever the user types lands
        // here; a scene that echoed it would photograph the typing. A resize arrives through the same
        // drain and is answered by the redraw below, which is why nothing inspects the event kind —
        // every wake redraws, and a redraw is what makes the stamp stand still.
        while screen.next_event().is_some() {}
        draw(&mut screen);
        stamp(ready, &mut frame, &screen);
    }
}

/// One frame of scene 01: eleven rows at the top left of whatever surface there is.
fn draw(screen: &mut vitui_engine::Screen) {
    let (w, h) = screen.size();
    let layer = match screen.layers().is_empty() {
        true => screen.layers().add_content(0, Rect::new(0, 0, w, h), true),
        false => screen
            .layers()
            .topmost_at(0, 0)
            .expect("the one layer covers the screen"),
    };
    if let Some(mut view) = screen.layers().view(layer) {
        for (row, case) in scene01().iter().enumerate() {
            // Column 0 deliberately: the dump has no column grid, so a left margin would only add
            // leading spaces the comparison then has to strip.
            view.text(0, row as i32, case.label, (case.draw)(EngineStyle::new()));
        }
    }
    screen.present();
}

/// Tell the driver what has been presented, and how many times.
///
/// **After the frame is on the wire, never before.** The counter is what lets the driver distinguish
/// *a frame exists* from *the terminal has stopped changing shape*.
fn stamp(ready: &str, frame: &mut u64, screen: &vitui_engine::Screen) {
    *frame += 1;
    let (w, h) = screen.size();
    let _ = std::fs::write(ready, format!("{frame} {w}x{h}\n"));
}

/// Scene 04: write the raw bytes, say so once, and keep rewriting them until the arm shuts down.
///
/// # Why this half does not use the engine, and what it costs
///
/// See [`scene04`]. The short version: the engine repairs a bisected pair before it serialises
/// anything, so the bytes this scene needs are bytes the engine will not emit.
///
/// # The handshake it cannot have, and what stands in for it
///
/// Scene 01's stamp carries a frame counter and the driver waits for it to stand still, because an
/// idle vitui application costs zero wakeups — so a stamp that stops moving is a screen that has
/// stopped moving. **That mechanism is the engine's, not the handshake's.** A raw-byte probe is told
/// nothing when the window resizes: `std` has no signal handling and this workspace has no
/// dependencies, so there is no wake to count.
///
/// So it repaints unconditionally, every [`REPAINT`], with an erase and absolute cursor addressing.
/// The picture *heals* rather than the handshake *detecting* — a window that settles into a new size
/// is repainted correctly within one cycle, and the erase takes any keystroke the window stole focus
/// for with it. What makes a bad capture loud rather than silent is the scene's own content: six
/// rows that each identify themselves, one of which is a column ruler, against a parser that refuses
/// a capture with fewer rows than the scene declared.
fn scene04_bytes(ready: &str) {
    use std::io::Write as _;

    let mut screen = String::new();
    // Hidden, so a block cursor parked on a compared row is not part of the picture.
    screen.push_str("\u{1b}[?25l");
    for (r, pair) in scene04().iter().enumerate() {
        // **`EL` per row, never `ED`.** The repaint has to erase what it is about to redraw, and
        // the obvious way to do that is `CSI 2 J`. Under the `--through-tmux` arm that is wrong in
        // a way no other arm could have shown: tmux pushes a cleared screen into the pane's
        // history, so ten repaints a second scrolled the picture up through Ghostty's scrollback
        // and the capture came back with the scene at row 38 and again at row 76. Six rows of
        // `FAILED` against a screen that had the right answer on it twice.
        //
        // Erasing one row at a time touches no history in any of the four arms, and it erases
        // exactly the cells this scene is about to write. Anything the terminal echoes lands below
        // them, where the cursor is parked and nothing is compared.
        let _ = write!(&mut screen, "\u{1b}[{};1H\u{1b}[2K", r + 1);
        for (col, text) in pair.emit {
            let _ = write!(&mut screen, "\u{1b}[{};{}H{text}", r + 1, col + 1);
        }
    }
    // Parked below the scene, erased with it, so an echoed keystroke has somewhere to go that is not
    // a compared row.
    let _ = write!(&mut screen, "\u{1b}[{};1H\u{1b}[2K", scene04().len() + 2);

    // The stamp is written once and never moves. There is no counter to move it: see above.
    let _ = std::fs::write(ready, "1 raw\n");

    loop {
        let mut out = std::io::stdout();
        let _ = out.write_all(screen.as_bytes());
        let _ = out.flush();
        std::thread::sleep(REPAINT);
    }
}

/// Scene 05: ask the terminal what each cluster is worth, write down what it said, and hold a
/// readable table up until the arm shuts the terminal down.
///
/// # This is the only scene whose answer does not come back through a photograph
///
/// The other two are pictures, and an arm's whole job is to take one. This one asks a question the
/// terminal answers **in band**, on the same tty the scene is writing to — so the arm launches it
/// and nothing else, and the capture surface, the window server and the automation grant are all
/// out of the path. That is worth saying twice, because it is the property that would let this
/// scene run on an arm whose capture surface carries no style at all.
///
/// # The batch, and why the sentinel is not a timeout
///
/// One write: for each cluster, home the cursor to column 1 of one row, erase, write the cluster,
/// ask `CSI 6n`. Then `CSI c` behind all of them. A terminal cannot answer the device-attributes
/// query before it has processed everything ahead of it, so the read stops on an **observed**
/// condition rather than on a delay tuned until it passed. `detect.rs` is where that shape comes
/// from; its code is `pub(crate)` and this is a different crate, so what is reused is the design.
///
/// # It writes what arrived and judges none of it
///
/// The bytes go to `<ready>.cpr` exactly as they came, and the driver is what refuses them. A scene
/// that decided whether its own answers were good enough would be the half with no gate over it
/// judging the half that has one — where `cursor_reports` is ordinary library code with ordinary
/// tests over committed fixtures.
fn scene05_cpr(ready: &str) {
    use std::io::Write as _;

    let glyphs = scene05();

    // **Raw and unechoed, or the answers are not readable and are also on the screen.** In cooked
    // mode the reply sits in the line discipline until a newline that will never come, and the
    // echo paints it into the picture. `min 0 time 5` is a half-second read timeout — VMIN 0, so a
    // read returns empty rather than blocking for ever on a terminal that answers nothing.
    //
    // **Never restored.** The terminal this runs in was created by the arm and is destroyed by it,
    // so there is no session to hand back; and the repaint below addresses every row absolutely, so
    // nothing depends on a newline meaning two things.
    if !stty(&["raw", "-echo", "min", "0", "time", "5"]) {
        // **Nothing below can work, and every observable would blame the terminal.** In cooked mode
        // the reply sits in the line discipline until a newline that never comes, so the read times
        // out, the batch has no sentinel, and the driver reports *the terminal answered nothing* —
        // about a terminal that was never asked in a way it could answer.
        //
        // So the answers file is deliberately **not created**, which is the one refusal the driver
        // can attribute: it names this line. Same shape as the Ghostty arm resolving `tmux`'s path
        // in the process that has the developer's `PATH` — a refusal belongs where the cause is
        // visible. The stamp is still written, or the driver would time out instead and say the
        // scene never started, which is the vaguer of the two.
        let _ = std::fs::write(ready, "1 cpr\n");
        loop {
            std::thread::sleep(REPAINT);
        }
    }

    let mut batch = String::new();
    // Hidden, so the cursor the report is about is not also a glyph in the picture.
    batch.push_str("\u{1b}[?25l");
    for glyph in &glyphs {
        // **Row 1, because it is the one row every terminal has.** Two live runs of scene 01 on this
        // machine were handed 156x45 and 72x24, and a scratch row chosen for looking tidy is a row
        // a short window does not have. `EL` and not `ED`, for the reason scene 04 records: tmux
        // pushes a cleared screen into the pane's history.
        let _ = write!(&mut batch, "\u{1b}[1;1H\u{1b}[2K{}\u{1b}[6n", glyph.cluster);
    }
    // The sentinel. Everything after its reply belongs to some other question.
    batch.push_str("\u{1b}[c");
    // Stop on the sentinel's own reply and on nothing else. `cursor_reports` is the one that knows
    // what a finished batch looks like, and it is the half with tests over it.
    let replies = ask(&batch, ANSWER_TIMEOUT, |seen| {
        !matches!(
            cursor_reports(seen, glyphs.len()),
            Err(vitui_conform::CprError::NoSentinel | vitui_conform::CprError::UnterminatedReply)
        )
    });
    let _ = std::fs::write(answers_at(Path::new(ready), "cpr"), &replies);

    // What a person looking at the window sees. Compared by nothing — the driver reads the file
    // above — and here because a window that goes blank after asking its questions is a window
    // nobody can tell from one that never asked them.
    let seen = cursor_reports(&replies, glyphs.len()).unwrap_or_default();
    let mut screen = String::new();
    screen.push_str("\u{1b}[?25l");
    let _ = write!(
        &mut screen,
        "\u{1b}[1;1H\u{1b}[2Kscene 05 — what this terminal says each cluster is worth"
    );
    for (r, glyph) in glyphs.iter().enumerate() {
        let advance = match seen.get(r) {
            Some(reply) => format!("{}", reply.column.saturating_sub(1)),
            None => "no reply".to_string(),
        };
        let _ = write!(
            &mut screen,
            "\u{1b}[{};1H\u{1b}[2K{:<12} {} -> {advance}",
            r + 3,
            glyph.label,
            glyph.cluster
        );
    }
    let _ = write!(&mut screen, "\u{1b}[{};1H\u{1b}[2K", glyphs.len() + 4);

    // The stamp is written after the answers are on disk, so a driver that saw quiescence is a
    // driver whose file exists. It never moves: the measurement happened once, and there is no
    // frame counter because there are no frames.
    let _ = std::fs::write(ready, "1 cpr\n");

    loop {
        let mut out = std::io::stdout();
        let _ = out.write_all(screen.as_bytes());
        let _ = out.flush();
        std::thread::sleep(REPAINT);
    }
}

/// Write one batch to the terminal and read until it says it has finished with it.
///
/// **The stop condition is the caller's, and it is an observed one.** A batch ends with `CSI c`,
/// whose reply the terminal cannot send before it has processed everything ahead of it, so
/// `finished` is asked after every read and the loop leaves the moment the batch's own reader says
/// the batch is whole. `after` is the ceiling for when there is not going to be one, and never a
/// settle time.
///
/// # Shared because both of its defects were expensive and neither is visible in a passing run
///
/// A signal arriving mid-read is not the terminal declining to answer, and treating it as one
/// surfaces downstream as *no sentinel* — the instrument blaming the emulator for its own
/// interruption. And an empty read is the configured `VTIME` timeout *and* what EOF looks like:
/// with `VMIN 0 VTIME 5` a real tty takes half a second to produce one, so a run of
/// [`EMPTY_READS`] of them cannot happen inside `after` and reaching that bound is evidence about
/// the descriptor rather than about how long the terminal took. Without it, a descriptor that is
/// not a terminal spins at 100% of a core until the deadline.
///
/// Scene 06 asks nine times where scene 05 asks once, so a second copy of that loop would be a
/// second copy of both.
fn ask(batch: &str, after: Duration, finished: impl Fn(&[u8]) -> bool) -> Vec<u8> {
    use std::io::{Read as _, Write as _};

    {
        let mut out = std::io::stdout();
        let _ = out.write_all(batch.as_bytes());
        let _ = out.flush();
    }

    let mut seen: Vec<u8> = Vec::new();
    let deadline = Instant::now() + after;
    let mut stdin = std::io::stdin();
    let mut chunk = [0u8; 256];
    let mut empty = 0usize;
    while Instant::now() < deadline {
        match stdin.read(&mut chunk) {
            Ok(0) => {
                empty += 1;
                if empty > EMPTY_READS {
                    break;
                }
            }
            Ok(n) => {
                empty = 0;
                seen.extend_from_slice(&chunk[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
        if finished(&seen) {
            break;
        }
    }
    seen
}

/// Scene 06: ask the terminal what it says about mode 2026, and time the flag it lets go of.
///
/// # It asks about a mode where every other scene photographs a screen
///
/// `quirks.rs`'s synchronised-output table has four rows and every one of them has the same
/// provenance — *the implementation, read* — because a force flush is a **rendering** event and
/// nothing a process inside a terminal can ask reports whether the terminal painted. Production
/// an earlier pass asked for Ghostty's row to be measured and it could not be: the only capture this
/// repository has is an AppleScript round trip four runs put between 136 ms and 623 ms, which is
/// the same order as Alacritty's entire 150 ms limit.
///
/// That sentence is still true and it is not the whole of mode 2026. Whether the terminal
/// **recognises** the mode, whether its state machine tracks the set and the reset, and when it
/// stops reporting the mode as set are three questions the terminal answers about *itself*, in
/// band, on this scene's own tty — with no capture surface, no window server and no automation
/// grant anywhere in the path. Scene 05 established that shape; this is the second scene to use it,
/// and the first for a question that is not a width.
///
/// # The flag is not the paint, and this scene may never be read as though it were
///
/// DECRQM reports a *mode*. What part B measures is when the terminal stopped reporting the mode as
/// set — the event Ghostty's own source calls *reset the synchronized output flag*. A terminal
/// could paint without clearing the flag or clear it without painting, and nothing here can tell
/// those apart. Every row says so, and [`vitui_conform::Bracket`] says it again where the number
/// lives.
///
/// # One probe per open, which the control probe had to teach
///
/// See [`vitui_conform::next_delay`]. The polling instrument was written first and it is wrong on
/// one of the three families: it brought Ghostty's reset forward from 1002 ms to under 517 ms while
/// leaving tmux and kitty at their documented figures.
///
/// # It writes what arrived and judges none of it
///
/// Part A's bytes go to `<ready>.decrqm` exactly as they came. Part B's probes go to
/// `<ready>.flush` as `<delay> <ps>` lines — the state read back through `mode_reports`, which is
/// ordinary library code with ordinary tests, and the elapsed millisecond the scene measured, which
/// is a fact about the run and not a judgement. The driver is what folds them into a bracket, and
/// [`vitui_conform::flush_bracket`] is what refuses a run that is not one.
///
/// **Part B has no fixture, and that is the rule rather than an omission.** It is a timing, a timing
/// is a report, and a report is not gated. Part A is a comparison and its bytes are committed.
fn scene06_decrqm(ready: &str) {
    let rows = scene06();

    // Raw and unechoed, for scene 05's reasons exactly — in cooked mode the reply sits in the line
    // discipline until a newline that will never come, and the echo paints it into the picture.
    if !stty(&["raw", "-echo", "min", "0", "time", "5"]) {
        // The answers files are deliberately **not** created, which is the one refusal the driver
        // can attribute: it names this line. The stamp is still written, or the driver would time
        // out and say the scene never started, which is the vaguer of the two.
        let _ = std::fs::write(ready, "1 decrqm\n");
        loop {
            std::thread::sleep(REPAINT);
        }
    }

    let batch = format!(
        "\u{1b}[?25l\
         \u{1b}[?{SYNC_MODE}$p\
         \u{1b}[?{SYNC_MODE}h\u{1b}[?{SYNC_MODE}$p\
         \u{1b}[?{SYNC_MODE}l\u{1b}[?{SYNC_MODE}$p\
         \u{1b}[?{SYNC_MODE}h\u{1b}[?{SYNC_MODE}h\u{1b}[?{SYNC_MODE}$p\
         \u{1b}[?{SYNC_MODE}l\u{1b}[?{SYNC_MODE}$p\
         \u{1b}[c"
    );
    let answers = ask(&batch, ANSWER_TIMEOUT, |seen| {
        !matches!(
            mode_reports(seen, SYNC_MODE, rows.len()),
            Err(vitui_conform::ModeError::NoSentinel | vitui_conform::ModeError::UnterminatedReply)
        )
    });
    let _ = std::fs::write(answers_at(Path::new(ready), "decrqm"), &answers);

    // ── Part B: one open, one wait, one question, and a fresh open for the next ──
    //
    // **Two asks a round, each consuming exactly the replies it asked for**, and that is not
    // tidiness. An `ask` that stops on a stop condition a *previous* round's leftovers already
    // satisfy returns immediately with somebody else's answer — the accident `clear_handshake`
    // exists to prevent one level up, arriving inside one process's own tty buffer. So the close is
    // verified by a DECRQM in the same batch rather than by a bare `CSI c`, and there is no third
    // write in the round with a reply nobody reads.
    let mut probes: Vec<Probe> = Vec::new();
    let mut log = String::new();
    let one_reply = |seen: &[u8]| {
        !matches!(
            mode_reports(seen, SYNC_MODE, 1),
            Err(vitui_conform::ModeError::NoSentinel | vitui_conform::ModeError::UnterminatedReply)
        )
    };
    while let Some(at_ms) = next_delay(
        &probes,
        FLUSH_FLOOR_MS,
        FLUSH_CEILING_MS,
        FLUSH_RESOLUTION_MS,
    ) {
        // Closed and asked about in one batch, so the round starts from a state that was observed
        // rather than assumed, and then left to settle so a timer armed by the previous open
        // cannot still be running when this one starts.
        //
        // **The reply is discarded on purpose and the evidence for the close is elsewhere.** Part
        // A's `after-close` row is a compared assertion that `CSI ? 2026 l` resets the mode on this
        // terminal, in this run, so a close that did not take is already a loud failure one section
        // up; and a mode stuck set here reports as `NeverReset`, which the report prints as what it
        // is. What this batch is for is the *ordering* — that the open below is the first thing to
        // touch the mode since it was observed reset.
        let _ = ask(
            &format!("\u{1b}[?{SYNC_MODE}l\u{1b}[?{SYNC_MODE}$p\u{1b}[c"),
            ANSWER_TIMEOUT,
            one_reply,
        );
        std::thread::sleep(FLUSH_BETWEEN);

        {
            use std::io::Write as _;
            let mut out = std::io::stdout();
            let _ = out.write_all(format!("\u{1b}[?{SYNC_MODE}h").as_bytes());
            let _ = out.flush();
        }
        let opened = Instant::now();
        while opened.elapsed() < Duration::from_millis(u64::from(at_ms)) {
            std::thread::sleep(Duration::from_millis(2));
        }
        let seen = ask(
            &format!("\u{1b}[?{SYNC_MODE}$p\u{1b}[c"),
            ANSWER_TIMEOUT,
            one_reply,
        );
        // **The reply's clock, and the request is the other one.** Which of the two a bracket may
        // use depends on the answer, and `Probe` is where that is written down.
        let answered_at_ms = u32::try_from(opened.elapsed().as_millis()).unwrap_or(u32::MAX);
        let state = mode_reports(&seen, SYNC_MODE, 1)
            .ok()
            .and_then(|r| r.first().map(|r| r.state));
        probes.push(Probe {
            at_ms,
            answered_at_ms,
            state,
        });
        let _ = writeln!(
            &mut log,
            "{at_ms} {answered_at_ms} {}",
            state.map_or_else(|| "-".to_string(), |s| s.ps().to_string())
        );
    }
    // The last round left the mode set. Nothing downstream depends on it and the terminal is about
    // to be destroyed by the arm, but a scene that hands back a terminal mid-block is a scene whose
    // window a person could be looking at.
    {
        use std::io::Write as _;
        let mut out = std::io::stdout();
        let _ = out.write_all(format!("\u{1b}[?{SYNC_MODE}l").as_bytes());
        let _ = out.flush();
    }
    let _ = std::fs::write(answers_at(Path::new(ready), "flush"), &log);

    // What a person looking at the window sees. Compared by nothing.
    let seen = mode_reports(&answers, SYNC_MODE, rows.len()).unwrap_or_default();
    let mut screen = String::new();
    screen.push_str("\u{1b}[?25l");
    let _ = write!(
        &mut screen,
        "\u{1b}[1;1H\u{1b}[2Kscene 06 — what this terminal says about mode {SYNC_MODE}"
    );
    for (r, row) in rows.iter().enumerate() {
        let said = seen
            .get(r)
            .map_or_else(|| "no reply".to_string(), |reply| reply.state.to_string());
        let _ = write!(
            &mut screen,
            "\u{1b}[{};1H\u{1b}[2K{:<14} {said}",
            r + 3,
            row.label
        );
    }
    for (r, line) in log.lines().enumerate() {
        let _ = write!(
            &mut screen,
            "\u{1b}[{};1H\u{1b}[2Kflush probe   {line}",
            rows.len() + 4 + r
        );
    }
    let _ = write!(
        &mut screen,
        "\u{1b}[{};1H\u{1b}[2K",
        rows.len() + 5 + log.lines().count()
    );

    // The stamp is written after both files are on disk, so a driver that saw quiescence is a
    // driver whose files exist. It never moves: the measurement happened once.
    let _ = std::fs::write(ready, "1 decrqm\n");

    loop {
        use std::io::Write as _;
        let mut out = std::io::stdout();
        let _ = out.write_all(screen.as_bytes());
        let _ = out.flush();
        std::thread::sleep(REPAINT);
    }
}

/// Put this process's controlling terminal into the mode the batch needs, and say whether it worked.
///
/// `/dev/tty` and not stdin: a scene launched by an arm has its tty as all three descriptors, and
/// naming the device says which one is meant rather than depending on that staying true.
///
/// **Two spellings, because the flag that names the device is not portable and the tmux arm is the
/// one that could run somewhere other than this machine.** BSD `stty` — macOS's — takes `-f`; GNU
/// coreutils takes `-F` and rejects `-f`. Tried in that order rather than detected, because the
/// answer is one process exit status and a `uname` would be a second thing to be wrong about.
#[must_use]
fn stty(args: &[&str]) -> bool {
    ["-f", "-F"].iter().any(|flag| {
        std::process::Command::new("stty")
            .args([flag, "/dev/tty"])
            .args(args)
            // The failing spelling prints a usage message, and this scene's stderr is the terminal
            // being measured.
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

/// How long a driver waits for one scene, which is not the same for all of them.
///
/// **Scene 06 stamps after it has measured, and its measurement is timed by construction.** Part B
/// opens [`FLUSH_OPENS`] blocks and each one waits up to [`FLUSH_CEILING_MS`], so a driver on
/// [`READY_TIMEOUT`] alone would give up while the scene was still doing what it was launched to
/// do — and would report *the scene never presented a frame*, which is the timeout blaming the
/// scene for the measurement it asked for. That is [`ANSWER_TIMEOUT`]'s own invariant one level up,
/// and it is derived here rather than typed, so the three parameters of the bisection cannot move
/// without this moving with them.
///
/// It is a ceiling for a run that is stuck. A scene that finishes early stamps early, and the
/// driver leaves as soon as the stamp has stood still.
#[must_use]
pub fn ready_timeout(which: &str) -> Duration {
    match which {
        "06" => {
            READY_TIMEOUT + Duration::from_millis(u64::from(FLUSH_CEILING_MS) * FLUSH_OPENS as u64)
        }
        _ => READY_TIMEOUT,
    }
}

/// Block until the scene has presented *and stopped changing*, or give up loudly.
///
/// **This replaces the fixed delay** — the mechanism by which a capture races the paint, and a raced
/// capture is exactly the empty one an earlier pass predicted would read as agreement. It waits for two
/// distinct things, because the first live run proved one was not enough:
///
/// 1. a stamp exists at all, so a frame has been presented;
/// 2. the stamp has stood still for [`QUIESCENT`], so the terminal has finished settling its size.
///
/// Quiescence is an *observed* condition and not a sleep: the scene redraws and re-stamps on every
/// wake, and an idle vitui application costs zero wakeups, so a stamp that stops moving is a screen
/// that has stopped moving.
///
/// Returns the surface size the last stamp reported.
///
/// # Errors
///
/// A message naming which of the two conditions was never met, because they have different causes:
/// no stamp at all is a scene that failed to attach, and a stamp that never stands still is a
/// terminal still changing shape.
pub fn wait_for_quiescence(ready: &Path, which: &str) -> Result<String, String> {
    let limit = ready_timeout(which);
    let deadline = Instant::now() + limit;
    let mut last: Option<(String, Instant)> = None;
    while Instant::now() < deadline {
        let now = std::fs::read_to_string(ready)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !now.is_empty() {
            match &last {
                Some((seen, since)) if *seen == now => {
                    if since.elapsed() >= QUIESCENT {
                        // "<frame> <w>x<h>" — the report wants the size, not the counter.
                        return Ok(now
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or("unknown")
                            .to_string());
                    }
                }
                _ => last = Some((now, Instant::now())),
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(match last {
        None => format!(
            "the scene never reported a presented frame within {limit:?} — it may have failed to \
             attach, or the terminal may have refused the command"
        ),
        Some(_) => format!(
            "the scene presented but never stood still for {QUIESCENT:?} within {limit:?} — \
             something is redrawing it, and a capture of a moving screen is not evidence"
        ),
    })
}

// ── The capture, which is not always a photograph ────────────────────────────────────────────────

/// What an arm brought back for one scene.
///
/// **Two variants because scene 05 is not a picture.** Scenes 01 and 04 are photographs, parsed as
/// a screen; scene 05's answers come back in band on the scene's own tty, so the arm's capture
/// surface is not in its path at all. Keeping that in one enum rather than in three arms is what
/// stops the third arm being the one that forgets.
pub enum Capture {
    /// A photograph of the terminal's screen, parsed with the arm's dialect.
    Screen(Dump),
    /// The terminal's own answers to `CSI 6n`, read by the scene and written where the driver can
    /// find them.
    Replies(Vec<Reply>),
    /// The terminal's own answers about mode 2026, and when it let go of the flag.
    Modes(ModeAnswers),
}

/// Scene 06's two halves, which arrive through two channels and are one capture.
///
/// **The bracket is a `Result` that is kept rather than unwrapped.** A run that cannot be
/// bracketed is not a failed run — a terminal without the mode, a terminal that never let go
/// inside the ceiling, and a probe that got no answer are three different facts, each worth
/// printing. Collapsing them into a missing row is the dishonesty `SCENES.md` opens by naming.
pub struct ModeAnswers {
    /// Part A: the five states, in the order the batch asked them.
    pub states: Vec<ModeReply>,
    /// Part B: the probes as the scene took them, `(elapsed ms, state)`.
    pub probes: Vec<Probe>,
    /// Part B folded, or why it is not a bracket.
    pub bracket: Result<Bracket, vitui_conform::BracketError>,
}

/// Where scene 05's answers land, derived from the readiness file rather than passed separately.
///
/// **Derived and not a second environment variable**, because the arms disagree about how they pass
/// one — tmux takes `-e`, kitty takes `.env`, AppleScript takes a list inside a string literal — and
/// a scene reached by three launchers that each had to be taught a second name is a scene one of
/// them would be launched without.
#[must_use]
pub fn answers_at(ready: &Path, ext: &str) -> std::path::PathBuf {
    // Not `with_extension`: the ready file's name ends in `-05`, and `with_extension` would replace
    // nothing there while replacing `-3.7c`-shaped tails elsewhere if the name ever changed.
    ready.with_file_name(format!(
        "{}.{ext}",
        ready.file_name().unwrap_or_default().to_string_lossy()
    ))
}

/// Every side channel a scene may leave beside its stamp, so [`clear_handshake`] cannot be taught
/// one and left ignorant of the next.
///
/// **A list and not three call sites.** The arms once deleted the readiness file and left the
/// answers beside it, and a run whose scene never got as far as asking could read a *previous*
/// run's answers as its own. Scene 06 leaves two files where scene 05 leaves one, which is exactly
/// the shape that would have reintroduced it: a second channel added to one of the three places
/// that knows about the first.
const SIDE_CHANNELS: &[&str] = &["cpr", "decrqm", "flush"];

/// Remove both halves of one scene's handshake, before a run and after it.
///
/// **Both, and that is the point of the function.** The arms deleted the readiness file and left the
/// answers beside it, and the two names share a process id — so a run whose scene never got as far
/// as asking could read a **previous** run's answers and report them as its own. That is the
/// accident this scene's four refusals exist to prevent, arriving underneath all four of them: the
/// batch would be well formed, the count right, the rows one, and the measurement somebody else's.
pub fn clear_handshake(ready: &Path) {
    let _ = std::fs::remove_file(ready);
    for ext in SIDE_CHANNELS {
        let _ = std::fs::remove_file(answers_at(ready, ext));
    }
}

/// Bring back whatever this scene's answer is, and refuse anything that is not one.
///
/// `photograph` is the arm's own capture mechanism and is **not called for scene 05** — see
/// [`Capture`]. Returns the capture and the bytes it was made of, because an arm still owes those
/// to [`save_if_asked`] and to [`header`]: a fixture is the evidence, whichever channel it came
/// through.
///
/// # Errors
///
/// The photograph failing, a screen with fewer rows than the scene declared, or a batch of cursor
/// reports that is not one — see [`vitui_conform::CprError`], whose four arms say four different
/// things about what went wrong.
pub fn capture(
    which: &str,
    ready: &Path,
    dialect: Dialect,
    photograph: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<(Capture, Vec<u8>), String> {
    if which == "05" {
        let at = answers_at(ready, "cpr").display().to_string();
        // A missing file is the scene never having got as far as writing one, and it is refused
        // here rather than read as an empty batch — which is `screen -X hardcopy`'s zero bytes
        // wearing this scene's clothes.
        let bytes = std::fs::read(&at).map_err(|e| {
            format!(
                "the scene wrote no answers to {at}: {e} — it writes that file even when the \
                     terminal answered nothing, so a missing one is the scene never having got as \
                     far as asking, and the way that happens is `stty` failing to put the tty into \
                     raw mode"
            )
        })?;
        let replies = cursor_reports(&bytes, scene05().len())
            .map_err(|e| format!("the answers are not a batch: {e}"))?;
        return Ok((Capture::Replies(replies), bytes));
    }
    if which == "06" {
        let at = answers_at(ready, "decrqm");
        let bytes = std::fs::read(&at).map_err(|e| {
            format!(
                "the scene wrote no answers to {}: {e} — it writes that file even when the \
                 terminal answered nothing, so a missing one is the scene never having got as far \
                 as asking, and the way that happens is `stty` failing to put the tty into raw mode",
                at.display()
            )
        })?;
        // **One of the refusals is an observation, and only one.** A terminal that processed the
        // batch and answered nothing has told us it has no synchronised-output report — the
        // sentinel is what makes that a fact rather than a timeout — so it is carried as an empty
        // set of states and printed row by row as `no reply`, against an arm that declared
        // `cannot express` in advance. Every other refusal stays a refusal: a *short* batch is a
        // lost answer, and reading one as the other would report a terminal without the mode on the
        // strength of a dropped reply.
        let states = match mode_reports(&bytes, SYNC_MODE, scene06().len()) {
            Ok(states) => states,
            Err(vitui_conform::ModeError::Unanswered { .. }) => Vec::new(),
            Err(e) => return Err(format!("the answers are not a batch: {e}")),
        };
        // The probe log is a **separate** refusal from the batch above, because the two channels
        // fail for different reasons: part A is a terminal that would not answer, part B is a
        // measurement that never ran. A driver that read one missing file as the other would
        // report the wrong cause.
        let flush = answers_at(ready, "flush");
        let text = std::fs::read_to_string(&flush).map_err(|e| {
            format!(
                "the scene wrote no flush probes to {}: {e}",
                flush.display()
            )
        })?;
        let probes = flush_probes(&text)?;
        let bracket = flush_bracket(&probes);
        return Ok((
            Capture::Modes(ModeAnswers {
                states,
                probes,
                bracket,
            }),
            bytes,
        ));
    }
    let bytes = photograph()?;
    let dump = parse(&bytes, rows_expected(which), dialect)
        .map_err(|e| format!("the capture is not a screen: {e}"))?;
    Ok((Capture::Screen(dump), bytes))
}

/// Read scene 06's probe log back: one `<elapsed ms> <ps or dash>` line per open.
///
/// **A parser and therefore a refusal.** An unreadable line is not a probe that got no answer —
/// `-` is what that looks like, and it is a real observation the bracket refuses on its own terms.
/// A line this cannot read is the channel being wrong, and reading it as a hole would hand
/// `flush_bracket` a fact nothing measured.
fn flush_probes(text: &str) -> Result<Vec<Probe>, String> {
    let mut probes = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let (Some(at), Some(answered), Some(state), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(format!(
                "the flush probe log has a line that is not one: {line:?}"
            ));
        };
        let at_ms = at
            .parse()
            .map_err(|_| format!("the flush probe log has a delay that is not one: {at:?}"))?;
        let answered_at_ms = answered.parse().map_err(|_| {
            format!("the flush probe log has a reply time that is not one: {answered:?}")
        })?;
        let state = match state {
            "-" => None,
            ps => Some(
                ps.parse()
                    .ok()
                    .and_then(vitui_conform::ModeState::from_ps)
                    .ok_or_else(|| {
                        format!("the flush probe log has a state DECRPM does not define: {ps:?}")
                    })?,
            ),
        };
        probes.push(Probe {
            at_ms,
            answered_at_ms,
            state,
        });
    }
    Ok(probes)
}

// ── The report ───────────────────────────────────────────────────────────────────────────────────

/// What one arm has to say about itself, so a row can say where it came from.
///
/// **A row that does not name its arm is not a result**, and with two arms that stopped being a
/// slogan: `capture-pane` re-serialises *tmux's* grid, so the tmux arm's rows are about tmux and
/// never about the emulator behind it. [`Arm::measures`] is where that is written, in the report,
/// rather than only in `SCENES.md` where a reader of the numbers may not go.
pub struct Arm {
    /// The heading, and the name in the file this writes.
    pub title: &'static str,
    /// What the software under test says its version is. Asked of it, never assumed.
    pub version: String,
    /// The capture mechanism, spelled as the reader would have to type it.
    pub mechanism: &'static str,
    /// What this arm's rows are evidence *about*.
    pub measures: &'static str,
    /// Which terminal answers `CSI 6n` for this arm, and it is **not always the one named above**.
    pub answers_in_band: AnswersInBand,
    /// Scene rows this arm does not compare, by label, each with why and the reason in words.
    ///
    /// **Declared in advance, never inferred from the observation**, or the instrument would be
    /// excusing its own disagreements after seeing them. Three things keep an exclusion honest: it
    /// is a hand-written constant in the arm rather than a question asked of the engine, the row is
    /// still printed with what was nonetheless observed, and a row excluded that **agrees anyway**
    /// is reported `STALE` and counted as a failure — so a declaration cannot outlive what earned
    /// it.
    ///
    /// See [`Excluded`] for the three kinds and why they are not one cell.
    pub not_compared: &'static [(&'static str, Excluded, &'static str)],
    /// Why this arm's capture surface carries **no style at all**, if it carries none.
    ///
    /// # One fact, declared once, reaching two scenes
    ///
    /// Terminal.app's AppleScript surface hands back `contents` as `type="text" access="r"` and has
    /// no styled variant anywhere on the `tab` class, so every cell of every capture from that arm
    /// is unstyled — whatever Terminal.app actually rendered. That is one property of the arm and it
    /// lands in two places: scene 01 is eleven rows *about* style, and scene 04 has one row whose
    /// value is a style it **reports**.
    ///
    /// Eleven near-identical entries in [`Arm::not_compared`] would have said the same thing eleven
    /// times, which is how one of them comes to be worded differently from the other ten. This says
    /// it once. **It is still a declaration and still carries the `STALE` rule**: scene 01 feeds it
    /// through [`Excluded::CannotAsk`] like any other exclusion, so a row that agrees anyway is
    /// counted as a failure and the declaration cannot outlive what earned it.
    ///
    /// **A fact about the instrument and never about the emulator.** Terminal.app renders bold; this
    /// arm cannot see that it did. Nothing here may be read as a claim about what Terminal.app draws.
    pub no_style: Option<&'static str>,
    /// Anything else this arm knows that the reader needs. One bullet per entry, already worded.
    pub notes: Vec<String>,
}

impl Arm {
    /// Why this arm does not compare `label`, if it said so before the run.
    fn excluded(&self, label: &str) -> Option<(Excluded, &'static str)> {
        self.not_compared
            .iter()
            .find(|(row, ..)| *row == label)
            .map(|(_, kind, why)| (*kind, *why))
    }
}

/// Who answers a cursor report for one arm, and why it is not simply that arm's title.
///
/// **Two fields because one of them is a table heading and the other is a paragraph.** Scene 05's
/// answers come back in band on the scene's own tty, so they come from the *innermost* terminal in
/// the path and a capture surface further out cannot change that. For the arm that runs the engine
/// inside tmux inside Ghostty, the photograph measures what tmux **forwards** and the cursor
/// reports measure **tmux** — the same subject as the plain tmux arm, and a column headed *Ghostty*
/// over those numbers would be the dishonesty `SCENES.md` opens by naming.
///
/// The rows are still printed, because they are real answers about a real terminal. What needed
/// fixing was the heading, and [`AnswersInBand::who`] is what heads them.
pub struct AnswersInBand {
    /// The terminal, short enough to head a column.
    pub who: &'static str,
    /// Why it is that terminal and not this arm's title, in one clause.
    pub why: &'static str,
}

/// Why a row of the scene is printed without being compared.
///
/// # Two more kinds of non-number, where `compare/` supplied three
///
/// `cannot express` is a fact about the emulator, `not run here` a fact about the run, and `FAILED`
/// a defect. Neither of these is any of the three, and they are **not each other** either — which is
/// the distinction worth the enum, because collapsing them would hide this repository's own code
/// behind a terminal's limitation.
///
/// **The `allow` is not a spare part.** This file is compiled once *per arm* — it is a module the
/// examples include, not a crate they link — so a kind no single arm happens to declare is dead code
/// in that arm's build and a build failure under `warnings = "deny"`. Every kind here is constructed
/// by some arm, and requiring each arm to construct all of them would be the tail wagging the dog.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Excluded {
    /// A fact about the **emulator**: it has no answer to give, and that is the answer.
    ///
    /// `compare/`'s first kind of non-number, inherited by `SCENES.md` on the day this directory was
    /// laid out and constructed by no arm for four of them — because the first three emulator
    /// families all had every capability the scenes ask about. **Terminal.app 2.15 is the first that
    /// does not**: it has no synchronised output, so `CSI ? 2026 $ p` is a question it cannot answer
    /// rather than one it answers wrongly, and a `FAILED` there would be this suite demanding a
    /// feature of a terminal that never claimed it.
    ///
    /// Not [`Excluded::CannotAsk`]: that one is the *instrument* failing to see something the
    /// emulator does. Here the emulator is not doing it, this suite can see that perfectly well, and
    /// what is missing is the capability rather than the view of it. Collapsing the two would hide
    /// the more interesting of the two facts behind the less.
    CannotExpress,
    /// A fact about the **instrument**: the emulator does the thing and this suite cannot see it.
    ///
    /// kitty 0.48.2 renders a dotted underline and writes it into a capture as `CSI 4 : m`, which is
    /// a single underline in ECMA-48. A row compared anyway would report a misbehaviour that is not
    /// happening, and would earn a `quirks.rs` entry the table exists to keep out.
    CannotAsk,
    /// A fact about **the engine**: it consulted `quirks.rs` and deliberately did not send this.
    ///
    /// The row is the quirk table working, not a terminal misbehaving, and a `FAILED` here would
    /// blame the emulator for this repository's own decision. It arrived the session after
    /// a later pass wired `attrs_dropped` on to the wire: the tmux arm had reported 11/11
    /// while the engine still sent SGR 53, and reported ten the first time it was run afterwards.
    ///
    /// **It costs the instrument the measurement that earned the entry**, and that is not a defect
    /// to be fixed here. An arm that consulted the engine to decide what to expect would be checking
    /// the engine against itself, which is the arrangement `conform/` exists to break. What preserves
    /// the evidence is the committed fixture, captured while the engine still sent the bit — which is
    /// why `fixtures/` says, in three places, that a capture is never regenerated to make something
    /// pass.
    ByDesign,
}

impl Excluded {
    /// The report's cell for this kind.
    fn cell(self) -> &'static str {
        match self {
            Self::CannotExpress => "`cannot express`",
            Self::CannotAsk => "`cannot ask`",
            Self::ByDesign => "`by design`",
        }
    }

    /// What a row of this kind agreeing anyway would mean.
    fn stale(self) -> &'static str {
        match self {
            Self::CannotExpress => {
                "the row agrees, so the terminal answered a question this arm \
                 declared it had no answer to — it has grown the capability, and `SCENES.md` and \
                 this arm no longer describe the same emulator"
            }
            Self::CannotAsk => {
                "the row agrees, so this arm's declaration that it cannot be asked \
                                is out of date and is now hiding whatever it is next wrong about"
            }
            Self::ByDesign => {
                "the row agrees, so the engine sent an attribute this arm was told it \
                               would withhold — the declaration and `quirks.rs` no longer describe \
                               the same terminal"
            }
        }
    }
}

/// What the dump said about one row of the scene.
enum Verdict {
    /// The label is there, wearing exactly the attribute the scene drew it with, and the rest of the
    /// row is unstyled. Carries what was seen rather than a tick, so an agreeing row still prints
    /// the evidence for its own agreement.
    Agreed(Style),
    /// The row is not in the capture at all.
    Missing,
    /// The label did not survive. Printed rather than summarised, because a wrong label usually
    /// means the whole screen is offset and every other row is about to lie in the same way.
    WrongText(String),
    /// The label survived and the style did not.
    WrongStyle(Style),
    /// The attribute did not stop where the label did.
    Leaked(usize),
}

impl Verdict {
    /// What this verdict says when the arm's capture surface carries **no style at all**.
    ///
    /// **Not "nothing", which is what [`Verdict::observed`] prints here and is the wrong sentence.**
    /// A styleless capture still carries the row's *text*, so `Missing` and `WrongText` are real
    /// observations that stay loud — a mis-sized or scrolled screen is exactly as visible on this
    /// arm as on any other. What is unreachable is the style, and only the style, so the one verdict
    /// that changes is the one that fell over on it.
    fn observed_without_style(&self) -> String {
        match self {
            Self::WrongStyle(_) => "the label survived".to_string(),
            other => other.observed(),
        }
    }

    fn observed(&self) -> String {
        match self {
            Self::Agreed(style) => describe(*style),
            Self::Missing => "no such row".into(),
            Self::WrongText(text) => format!("text was {text:?}"),
            Self::WrongStyle(style) => describe(*style),
            Self::Leaked(n) => format!(
                "the attribute continued past the label, over {n} of the row's padding cells"
            ),
        }
    }
}

/// The style the scene drew row `case` with, in the dump's own vocabulary.
fn expected(case: &Case) -> Style {
    Style {
        attrs: case.attrs,
        underline: case.underline,
        ..Style::default()
    }
}

/// Judge one row, checking three things rather than one.
///
/// **The padding check is not padding.** The engine paints the whole surface, so a row is the label
/// followed by spaces, and the label's style must stop where the label does. If it did not, the
/// terminal would be showing an underline or a reverse block running to the right edge — a real
/// defect, and one that a comparison of the first cluster alone would report as a clean pass.
///
/// The two arms differ in what they hand back here and the check survives both. Ghostty's `vt` dump
/// keeps the default-styled blanks the engine painted; tmux's `capture-pane` trims them, so the
/// padding count is zero for a reason that is about the *capture format* rather than about the
/// screen. What matters is that neither trims a blank that is **styled** — a leaked reverse block
/// is not default-styled, so it survives the trim and is still counted.
fn judge(dump: &Dump, i: usize, case: &Case) -> Verdict {
    let Some(row) = dump.rows.get(i) else {
        return Verdict::Missing;
    };
    let text = row.text();
    if text.trim_end() != case.label {
        return Verdict::WrongText(text.trim_end().to_string());
    }
    let want = expected(case);
    let label_len = case.label.chars().count();
    if let Some(wrong) = row
        .clusters
        .iter()
        .take(label_len)
        .find(|c| c.style != want)
    {
        return Verdict::WrongStyle(wrong.style);
    }
    let leaked = row
        .clusters
        .iter()
        .skip(label_len)
        .filter(|c| c.style != Style::default())
        .count();
    match leaked {
        0 => Verdict::Agreed(want),
        n => Verdict::Leaked(n),
    }
}

/// The preamble one arm's report opens with, written once however many scenes it ran.
///
/// `bytes` is the first scene's capture, which is where the OSC 10/11 default colours come from.
/// They are the terminal's and not the scene's, so any capture would do and the first is the one
/// that exists when this is called.
pub fn header(arm: &Arm, bytes: &[u8]) -> String {
    let (fg, bg) = default_colours(bytes);

    let mut out = String::new();
    let _ = writeln!(out, "# The conformance suite — {}\n", arm.title);
    let _ = writeln!(
        out,
        "**Generated. It reports; it does not block.** One file per arm, because two arms writing \
         one file means the rows of whichever ran first are gone — and *a missing row reads as a \
         win*. **Every scene of `SCENES` is in here** for the same reason, and there is no flag to \
         run one of them.\n"
    );
    let _ = writeln!(
        out,
        "The version of software this repository does not control cannot gate its pull requests. So \
         this file is committed and regenerated, and a disagreement arrives as a review-visible diff \
         rather than as a red build. The gate is `cargo test` over `fixtures/`.\n"
    );
    let _ = writeln!(out, "Read [`SCENES.md`](SCENES.md) first.\n");
    let _ = writeln!(
        out,
        "- **Arm:** {} {}, {}",
        arm.title, arm.version, arm.mechanism
    );
    let _ = writeln!(out, "- **These rows are evidence about:** {}", arm.measures);
    let _ = writeln!(
        out,
        "- **Default colours, from the dump's own OSC 10/11:** fg {}, bg {}",
        show(fg),
        show(bg)
    );
    for note in &arm.notes {
        let _ = writeln!(out, "- {note}");
    }
    let _ = writeln!(out);
    out
}

/// One scene's section of one arm's report: the text, how many rows were asked, and how many
/// disagreed.
pub fn section(arm: &Arm, which: &str, capture: &Capture, size: &str) -> (String, usize, usize) {
    match (which, capture) {
        ("01", Capture::Screen(dump)) => section01(arm, dump, size),
        ("04", Capture::Screen(dump)) => section04(arm, dump, size),
        ("05", Capture::Replies(replies)) => section05(arm, replies),
        ("06", Capture::Modes(answers)) => section06(arm, answers),
        (other, _) => panic!("no such scene, or the wrong kind of capture for it: {other}"),
    }
}

/// The sentence every report ends with, and it is about what is *not* in the tables above.
pub fn trailer() -> String {
    "A row that does not say which arm it came from is not a result, and a *missing* row reads as a \
     win — which is the single easiest way for this directory to become dishonest. Every row of \
     every scene is printed above whether it agreed or not, and a capture with fewer rows than the \
     scene declared never reaches a table: it is refused as `FAILED` by the parser.\n"
        .to_string()
}

/// Scene 01's section: the eleven attribute bits, one per row.
fn section01(arm: &Arm, dump: &Dump, size: &str) -> (String, usize, usize) {
    let cases = scene01();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "## Scene 01 — the eleven attribute bits, one per row\n"
    );
    let _ = writeln!(
        out,
        "Surface: **{size}** cells, as the scene reported it.\n"
    );
    let _ = writeln!(
        out,
        "Each row lights exactly one of the style word's eleven attribute bits. A row agrees only \
         when the dump reports that attribute **and nothing else** — an invented attribute is a \
         disagreement in the same way a missing one is.\n"
    );
    let _ = writeln!(out, "| bit | verb | expected | observed | |");
    let _ = writeln!(out, "|---|---|---|---|---|");

    let mut failures = 0;
    let mut unanswerable = 0;
    for (i, case) in cases.iter().enumerate() {
        let verdict = judge(dump, i, case);
        // **The arm's own declaration first, then the styleless fallback.** An arm that named this
        // row keeps its wording; one that only said its capture surface carries no style gets that
        // reason on all eleven. See [`Arm::no_style`].
        let excluded = arm
            .excluded(case.label)
            .or_else(|| arm.no_style.map(|why| (Excluded::CannotAsk, why)));
        let observed = match arm.no_style {
            Some(_) => verdict.observed_without_style(),
            None => verdict.observed(),
        };
        let (mark, observed) = mark_of(
            excluded,
            verdict.agreed(),
            &observed,
            &mut failures,
            &mut unanswerable,
        );
        let _ = writeln!(
            out,
            "| {} | `{}` | {} | {} | {} |",
            case.label,
            case.verb,
            describe(expected(case)),
            observed,
            mark
        );
    }

    let asked = cases.len() - unanswerable;
    let _ = writeln!(out, "\n**{}/{} agreed.**\n", asked - failures, asked);
    if unanswerable > 0 {
        let _ = writeln!(out, "{}", not_in_the_denominator(unanswerable, cases.len()));
    }
    (out, asked, failures)
}

/// Scene 04's section: a pair bisected, and what the terminal does with the orphan.
fn section04(arm: &Arm, dump: &Dump, size: &str) -> (String, usize, usize) {
    let pairs = scene04();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "## Scene 04 — a pair bisected, and what the terminal does with the orphan\n"
    );
    let _ = writeln!(
        out,
        "Surface: **{size}**. This scene is raw bytes rather than the engine, so it has no surface \
         size to report — see below.\n"
    );
    let _ = writeln!(
        out,
        "**The only scene here that does not drive the engine, and it cannot.** The engine's \
         drawing verbs repair a bisected pair before anything is serialised, so an engine-driven \
         scene could photograph only the repair. The question is what a terminal does when it is \
         handed the bytes anyway — which is what the engine's mirror would have believed had the \
         repair stopped at a `View::child` clip, and is architecture ticket 20's whole subject.\n"
    );
    let _ = writeln!(
        out,
        "Every row is `AB漢CD` — `A` at column 0, `B` at 1, the wide glyph across 2 and 3, `C` at \
         4, `D` at 5 — and then one write over one half of it. **The row is compared as text**: a \
         grid-to-text dump emits no padding cell for a double-width glyph, so *what is at column 3* \
         is not askable of it, and ASCII sentinels turn the question into one that is.\n"
    );
    let _ = writeln!(out, "| row | asks | expected | observed | |");
    let _ = writeln!(out, "|---|---|---|---|---|");

    let mut failures = 0;
    let mut unanswerable = 0;
    for (i, pair) in pairs.iter().enumerate() {
        let verdict = judge04(dump, i, pair, arm.no_style);
        let (mark, observed) = mark_of(
            arm.excluded(pair.label),
            verdict.agreed(),
            &verdict.observed(),
            &mut failures,
            &mut unanswerable,
        );
        let _ = writeln!(
            out,
            "| {} | {} | `{:?}` | {} | {} |",
            pair.label, pair.asks, pair.want, observed, mark
        );
    }

    let asked = pairs.len() - unanswerable;
    let _ = writeln!(out, "\n**{}/{} agreed.**\n", asked - failures, asked);
    if unanswerable > 0 {
        let _ = writeln!(out, "{}", not_in_the_denominator(unanswerable, pairs.len()));
    }
    (out, asked, failures)
}

/// Scene 05's section: three rows compared, twelve surveyed, and the difference is a decision.
fn section05(arm: &Arm, replies: &[Reply]) -> (String, usize, usize) {
    let glyphs = scene05();
    let advance = |i: usize| replies.get(i).map(|r| r.column.saturating_sub(1));

    let mut out = String::new();
    let _ = writeln!(
        out,
        "## Scene 05 — what does this emulator think this cluster is worth\n"
    );
    let _ = writeln!(
        out,
        "**Answered by: {}** — {}.\n\nThis is the only scene here whose answer does not come back \
         through a photograph. The scene homes the cursor to column 1, writes one cluster, and asks \
         `CSI 6n`; the column that comes back is the **emulator's own UAX #11 verdict**, reached by \
         the emulator's tables and reported by the emulator, with nothing of this repository's in \
         the path. A `CSI c` behind the batch is the sentinel, so the read stops on an observed \
         condition rather than on a delay.\n",
        arm.answers_in_band.who, arm.answers_in_band.why
    );
    let _ = writeln!(
        out,
        "Because the answers arrive in band on the scene's own tty, they come from the **innermost** \
         terminal in the path — which is why the line above names a terminal rather than repeating \
         this arm's title. A capture surface further out cannot change who answered.\n"
    );

    // ── The three the instrument is held to ──
    let _ = writeln!(out, "### The three rows that are compared\n");
    let _ = writeln!(
        out,
        "Hand-written expectations, and deliberately **not** asked of the engine: a row that took \
         its number from `width_of` would be checking the engine against itself, which is the \
         arrangement this directory exists to break. They are here to say the probe is measuring an \
         advance at all — a report that answered a constant would pass the first of them.\n"
    );
    let _ = writeln!(
        out,
        "| row | asks | declared here | {} | |",
        arm.answers_in_band.who
    );
    let _ = writeln!(out, "|---|---|---|---|---|");

    let (mut failures, mut unanswerable, mut compared) = (0usize, 0usize, 0usize);
    for (i, glyph) in glyphs.iter().enumerate() {
        let Some(want) = glyph.compared else { continue };
        compared += 1;
        let seen = advance(i);
        let (mark, observed) = mark_of(
            arm.excluded(glyph.label),
            seen == Some(want),
            &match seen {
                Some(n) => format!("{n}"),
                None => "no reply".to_string(),
            },
            &mut failures,
            &mut unanswerable,
        );
        let _ = writeln!(
            out,
            "| {} | {} | {want} | {observed} | {mark} |",
            glyph.label, glyph.asks
        );
    }
    let asked = compared - unanswerable;
    let _ = writeln!(out, "\n**{}/{asked} agreed.**\n", asked - failures);
    if unanswerable > 0 {
        let _ = writeln!(out, "{}", not_in_the_denominator(unanswerable, compared));
    }

    // ── The survey, which is the deliverable ──
    let _ = writeln!(out, "### The survey\n");
    let _ = writeln!(
        out,
        "**Reported, never failed, and no row here earns a `quirks.rs` entry.** The engine's \
         tables are authoritative *by decision*: `ucd.rs` says so in as many words, pins three \
         answers as policy rather than standard — ambiguous width is narrow, a cluster's width is \
         its base's and never the sum, VS15 changes a presentation and not a width — and spec §8's \
         `CHA`-after-non-ASCII rule is what **bounds** a disagreement instead of following it. So a \
         terminal that answers differently below is not misbehaving in any sense this repository \
         acts on, there is no mechanism that would read such a quirk row, and a `FAILED` here would \
         be the instrument inventing a defect.\n"
    );
    let _ = writeln!(
        out,
        "What the paragraph in `ucd.rs` cites for the disagreement is a **survey of 23 terminals in \
         a research document**. This table is the first thing in this repository to observe any of \
         it.\n"
    );
    let _ = writeln!(
        out,
        "| cluster | code points | asks | the engine | {} | |",
        arm.answers_in_band.who
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|");

    let (mut same, mut surveyed) = (0usize, 0usize);
    for (i, glyph) in glyphs.iter().enumerate() {
        if glyph.compared.is_some() {
            continue;
        }
        surveyed += 1;
        let ours = width_of(glyph.cluster);
        let seen = advance(i);
        let agrees = seen == Some(ours);
        same += usize::from(agrees);
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {ours} | {} | {} |",
            glyph.cluster,
            code_points(glyph.cluster),
            glyph.asks,
            match seen {
                Some(n) => format!("{n}"),
                None => "no reply".to_string(),
            },
            match agrees {
                true => "\u{2713}",
                false => "**differs**",
            }
        );
    }
    let _ = writeln!(
        out,
        "\n**{same} of {surveyed} agree with the engine's tables.** That number is a fact about \
         this terminal and about the disagreement's size; it is not a score and it is not a \
         denominator anything is held to.\n"
    );

    (out, asked, failures)
}

/// Scene 06's section: five rows compared, and one bracket that is a report.
fn section06(arm: &Arm, answers: &ModeAnswers) -> (String, usize, usize) {
    let rows = scene06();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "## Scene 06 — mode {SYNC_MODE}, asked of the terminal rather than of its documentation\n"
    );
    let _ = writeln!(
        out,
        "**Answered by: {}** — {}.\n\nThe second scene here whose answer does not come back through \
         a photograph, and the first for a question that is not a width. The scene asks \
         `CSI ? {SYNC_MODE} $ p` on its own tty and the terminal answers in band, so the capture \
         surface, the window server and the automation grant are all out of the path — and the \
         terminal that answers is the **innermost** one, which is why the line above names a \
         terminal rather than repeating this arm's title.\n",
        arm.answers_in_band.who, arm.answers_in_band.why
    );

    // ── Part A ──
    let _ = writeln!(out, "### The state machine, and these five are compared\n");
    let _ = writeln!(
        out,
        "One batch — ask, set, ask, reset, ask, set, set, ask, reset, ask — with `CSI c` behind it, \
         and the answers read positionally. **The expectations are DECRPM's own**, which is what \
         makes this a comparison where scene 05's twelve rows are a survey: a terminal that reports \
         the mode set after it was asked to reset it is wrong by the definition of the reply it \
         sent, not by a table this repository chose. A terminal with no synchronised output answers \
         `not recognised (0)` throughout, which is a legitimate answer — the arm then owes a \
         `cannot express` declaration, and until it has one the rows are loud. **A terminal may \
         also answer nothing at all**, which Terminal.app 2.15 does: its parser does not take `$` \
         as an intermediate, so this is not a query it declines but one it never finishes reading. \
         The sentinel is what makes that an observation rather than a timeout, and the rows below \
         then read `no reply` against a `cannot express` the arm declared in advance.\n"
    );
    let _ = writeln!(
        out,
        "| row | asks | declared here | {} | |",
        arm.answers_in_band.who
    );
    let _ = writeln!(out, "|---|---|---|---|---|");

    let (mut failures, mut unanswerable) = (0usize, 0usize);
    for (i, row) in rows.iter().enumerate() {
        let seen = answers.states.get(i).map(|r| r.state);
        let (mark, observed) = mark_of(
            arm.excluded(row.label),
            seen == Some(row.want),
            &seen.map_or_else(|| "no reply".to_string(), |s| s.to_string()),
            &mut failures,
            &mut unanswerable,
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {observed} | {mark} |",
            row.label, row.asks, row.want
        );
    }
    let asked = rows.len() - unanswerable;
    let _ = writeln!(out, "\n**{}/{asked} agreed.**\n", asked - failures);
    if unanswerable > 0 {
        let _ = writeln!(out, "{}", not_in_the_denominator(unanswerable, rows.len()));
    }

    // ── Part B ──
    let _ = writeln!(
        out,
        "### When the terminal let go of the flag — reported, never failed\n"
    );
    let _ = writeln!(
        out,
        "**This is the flag and it is not the paint.** A force flush is a *rendering* event and \
         nothing a process inside a terminal can ask reports whether the terminal painted; DECRQM \
         reports a **mode**. What is below is when the terminal stopped reporting the mode as set — \
         the event Ghostty's own source calls *reset the synchronized output flag*. A terminal \
         could paint without clearing the flag or clear it without painting, and nothing here can \
         tell those apart. It is a timing besides, and a timing is a report.\n"
    );
    let _ = writeln!(
        out,
        "**One probe per open, and the control probe is why.** The polling instrument was written \
         first: it opens one block and asks repeatedly, which costs one open where this costs seven. \
         Polling a Ghostty 1.3.1 every 250 ms brought the reset forward from 1002 ms to under \
         517 ms, while the same polling left tmux 3.7c at 1007 ms and kitty 0.48.2 at 2261 ms — \
         their documented figures. An instrument that polls is inside its own measurement, and the \
         two families it happens not to disturb are exactly what would have made that invisible. So \
         each row below is a fresh open, a wait, one question and a close, and the boundary between \
         them is halved for. **The two clocks are both printed** because only one of them is sound \
         for each answer: the terminal processed the question somewhere between them, a *set* is \
         evidence back to the request and a *reset* is evidence forward to the reply, so the \
         bracket takes one end from each column.\n"
    );
    let _ = writeln!(
        out,
        "| open | held open for | answered at | {} |",
        arm.answers_in_band.who
    );
    let _ = writeln!(out, "|---|---|---|---|");
    for (n, probe) in answers.probes.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | {} ms | {} ms | {} |",
            n + 1,
            probe.at_ms,
            probe.answered_at_ms,
            probe
                .state
                .map_or_else(|| "no reply".to_string(), |s| s.to_string())
        );
    }
    let _ = writeln!(out);
    let verdict = match &answers.bracket {
        Ok(Bracket::Between {
            still_set_at,
            reset_by,
        }) => format!(
            "**Still set at {still_set_at} ms, reset by {reset_by} ms.** The event is in that \
             interval; a bracket and never a point, because a probe is a sample. `quirks.rs`'s \
             row for this terminal is the number to read it against, and that row's provenance is \
             *the implementation, read*."
        ),
        Ok(Bracket::NeverReset { ceiling }) => format!(
            "**Still set at {ceiling} ms**, which is the largest delay this run opened a block \
             for. Either this terminal's limit is beyond that or it has none, and this run cannot \
             say which."
        ),
        Ok(Bracket::AlreadyReset { floor }) => format!(
            "**Already reset at {floor} ms**, and there are now **three** facts that look like \
             this. The limit may be under the floor; the open may never have taken; or the \
             terminal's DECRQM may never say *set* at all, in which case the bisection has nothing \
             to bisect and this number is not a limit. **The third is what Alacritty 0.17.0 does** \
             — see part A above, where the two rows asked inside an open block are `FAILED` — and \
             it was the arm that found it: the first two were what this sentence said until then. \
             What the figure measures on such a terminal is the **reply**, which is the only thing \
             about a block it can still observe: a question asked inside one comes back when the \
             block drains. See the arm's own notes."
        ),
        Err(why) => format!("**No bracket:** {why}"),
    };
    let _ = writeln!(out, "{verdict}\n");
    let _ = writeln!(
        out,
        "Nothing in this section moves the numerator or the denominator above it. A timing is a \
         report, and a gate tuned to one is the flaky test this repository refuses by name.\n"
    );

    (out, asked, failures)
}

/// A cluster's scalars, spelled the way a reader would search for them.
fn code_points(cluster: &str) -> String {
    cluster
        .chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The report cell for one row, and the two counters it moves.
///
/// One function because the three scenes must treat an exclusion identically: a row an arm declared
/// it would not compare leaves the denominator, and a row so declared that **agrees anyway** is
/// `STALE` and counts as a failure. Two copies of that rule is how one of them would come to be
/// missing it.
///
/// **The exclusion is handed in rather than looked up**, because scene 01 has a second source for
/// one: an arm whose capture surface carries no style at all excludes every row of that scene from
/// [`Arm::no_style`], and the rule above has to reach those rows identically. A lookup inside here
/// would have made that a fourth place the rule is written.
fn mark_of(
    excluded: Option<(Excluded, &'static str)>,
    agreed: bool,
    observed: &str,
    failures: &mut usize,
    unanswerable: &mut usize,
) -> (&'static str, String) {
    match (excluded, agreed) {
        (Some((kind, _)), true) => {
            *failures += 1;
            ("**STALE**", kind.stale().to_string())
        }
        (Some((kind, why)), false) => {
            *unanswerable += 1;
            (kind.cell(), format!("{why} — observed: {observed}"))
        }
        (None, true) => ("\u{2713}", observed.to_string()),
        (None, false) => {
            *failures += 1;
            ("**FAILED**", observed.to_string())
        }
    }
}

/// The paragraph under a table that has rows outside its denominator.
fn not_in_the_denominator(unanswerable: usize, total: usize) -> String {
    format!(
        "**Not in that denominator: {unanswerable} of the scene's {total}.** This arm declared \
         before the run that it would not compare them, with the reason printed in the row beside \
         what was nonetheless observed. `cannot express` is a fact about the **emulator** — it has \
         no answer to give, and that is the answer. `cannot ask` is a fact about the \
         **instrument** — the emulator does the thing and this suite cannot see it. `by design` \
         is a fact about **the engine** — it consulted `quirks.rs` and did not send it, so a \
         `FAILED` would blame the terminal for a decision of ours. Only the first of the three is \
         one `compare/` had a word for, which is why `SCENES.md` grew the other two. A row so \
         declared that agrees anyway is \
         reported `STALE` and counted as a failure, so a declaration cannot outlive what earned \
         it.\n"
    )
}

/// What the dump said about one row of scene 04.
enum Seen {
    /// The row reads exactly as the scene said it would.
    Agreed(String),
    /// The same, and the row also asked for a cluster's style to be **reported**. Carried rather
    /// than folded into `Agreed` so the report cannot print a tick without printing the observation
    /// that earns the row its place.
    Reported(String, usize, Style),
    /// The row is not in the capture at all.
    Missing,
    /// The row does not read as the scene said it would. **This is the finding**, whichever way it
    /// falls: a terminal that left the orphan standing shows it here.
    WrongText(String),
    /// The row asked to report a cluster the capture does not have that many of. A defect in the
    /// scene or a capture that lost cells, and either way not a silent blank cell in the report.
    NoSuchCluster(usize),
    /// The text agreed and the style this row exists to report is **not askable of this arm**.
    ///
    /// **The alternative is a sentence that is false.** A styleless capture surface hands every
    /// cluster back wearing nothing, so folding this into [`Seen::Reported`] would print *the
    /// blanked half wears plain* — a claim about what Terminal.app renders, made by an instrument
    /// that cannot see what Terminal.app renders, in the one row of the scene whose whole value is
    /// that the three families answer it differently.
    ///
    /// It agrees, because the row's **assertion** is its text and the text is comparable here. Only
    /// the reported half is out of reach, and this variant is what says which half.
    Unreportable(String, usize, &'static str),
}

impl Seen {
    fn agreed(&self) -> bool {
        matches!(
            self,
            Self::Agreed(_) | Self::Reported(..) | Self::Unreportable(..)
        )
    }

    fn observed(&self) -> String {
        match self {
            Self::Agreed(text) => format!("`{text:?}`"),
            Self::Reported(text, at, style) => format!(
                "`{text:?}` — and cluster {at}, the blanked half, wears **{}**",
                describe(*style)
            ),
            Self::Missing => "no such row".into(),
            Self::WrongText(text) => format!("`{text:?}`"),
            Self::NoSuchCluster(at) => format!("the row has no cluster {at}"),
            Self::Unreportable(text, at, why) => format!(
                "`{text:?}` — and what cluster {at}, the blanked half, wears is **not askable of \
                 this arm**: {why}"
            ),
        }
    }
}

impl Verdict {
    fn agreed(&self) -> bool {
        matches!(self, Self::Agreed(_))
    }
}

/// Judge one row of scene 04: the text is the assertion, and a style is carried out for the report.
fn judge04(dump: &Dump, i: usize, pair: &Pair, no_style: Option<&'static str>) -> Seen {
    let Some(row) = dump.rows.get(i) else {
        return Seen::Missing;
    };
    let text = row.text();
    let text = text.trim_end().to_string();
    if text != pair.want {
        return Seen::WrongText(text);
    }
    if let Some(at) = pair.report_style {
        let Some(cluster) = row.clusters.get(at) else {
            return Seen::NoSuchCluster(at);
        };
        // The cluster is looked up **before** this, so a styleless arm still refuses a capture that
        // lost cells. What the declaration removes is the style claim and nothing else.
        return match no_style {
            Some(why) => Seen::Unreportable(text, at, why),
            None => Seen::Reported(text, at, cluster.style),
        };
    }
    Seen::Agreed(text)
}

/// A style in words, for a table cell.
fn describe(s: Style) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for (attr, name) in [
        (Attrs::BOLD, "bold"),
        (Attrs::DIM, "dim"),
        (Attrs::ITALIC, "italic"),
        (Attrs::BLINK, "blink"),
        (Attrs::REVERSE, "reverse"),
        (Attrs::HIDDEN, "conceal"),
        (Attrs::STRIKE, "strike"),
        (Attrs::OVERLINE, "overline"),
    ] {
        if s.attrs.has(attr) {
            parts.push(name);
        }
    }
    let underline = match s.underline {
        Underline::None => None,
        Underline::Single => Some("underline".to_string()),
        Underline::Double => Some("underline:2".to_string()),
        Underline::Curly => Some("underline:3".to_string()),
        Underline::Dotted => Some("underline:4".to_string()),
        Underline::Dashed => Some("underline:5".to_string()),
        Underline::Other(n) => Some(format!("underline:{n}")),
    };
    let mut all: Vec<String> = parts.into_iter().map(str::to_string).collect();
    all.extend(underline);
    if s.fg != Colour::Default {
        all.push(format!("fg {:?}", s.fg));
    }
    if s.bg != Colour::Default {
        all.push(format!("bg {:?}", s.bg));
    }
    if s.underline_colour != Colour::Default {
        all.push(format!("underline colour {:?}", s.underline_colour));
    }
    match all.is_empty() {
        true => "nothing".into(),
        false => all.join(" + "),
    }
}

/// A default colour the capture may not have carried.
fn show(c: Option<Colour>) -> String {
    match c {
        Some(Colour::Rgb(r, g, b)) => format!("`#{r:02x}{g:02x}{b:02x}`"),
        Some(other) => format!("`{other:?}`"),
        // A fact about the run, in `compare/`'s vocabulary: the capture format carries no header.
        None => "`cannot express` — this capture format has no OSC 10/11 header".into(),
    }
}

/// Write one arm's report where the arm's own name puts it, and say how it went.
///
/// # Errors
///
/// The write failing, or a disagreement — which is a non-zero exit so a human running this notices,
/// and **not** a CI gate: nothing runs these on a pull request.
pub fn publish(arm: &Arm, report: &str, asked: usize, failures: usize) -> Result<(), String> {
    // **Slugified rather than lowercased.** An arm titled `Terminal.app` would otherwise write
    // `REPORT-terminal.app.md`, whose apparent double extension reads as an accident and whose name
    // is not the one a reader arrives with — `cargo run --example terminal`. Every title that
    // existed before this passes through unchanged.
    let slug: String = arm
        .title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let path = format!("REPORT-{slug}.md");
    std::fs::write(&path, report).map_err(|e| format!("writing {path}: {e}"))?;
    println!("{report}");
    // `asked` is summed over the sections and already excludes the rows this arm said it would not
    // compare, for the same reason they are not in a table's agreed count: a row nothing asked is
    // not a row anything answered.
    match failures {
        0 => {
            println!(
                "{path} written. {asked}/{asked} agreed, over {} scenes.",
                SCENES.len()
            );
            Ok(())
        }
        n => Err(format!("{n} of {asked} disagreed — {path} written")),
    }
}

/// The capture an arm produced, saved out only when the operator asked for it.
///
/// **Opt-in, and never automatic.** `fixtures/` is evidence: a test that fails against it is a
/// parser defect or a claim that stopped being true, never a reason to regenerate. A driver that
/// rewrote its own fixtures on every run would turn the gate into a mirror.
///
/// `CONFORM_SAVE_CAPTURE` is a **prefix**, not a file — there is a capture per scene now, and one
/// name would have kept whichever ran last. `fixtures/kitty-0.48.2` becomes
/// `fixtures/kitty-0.48.2-scene04-pairs.vt`.
///
/// **It will not overwrite one**, which turns *a capture is never regenerated to make something
/// pass* from a sentence in three files into something the code will not do. The finding that earned
/// that rule is in `SCENES.md`: a later pass wired `attrs_dropped` on to the wire and the
/// tmux arm stopped being able to ask the question its fixture had already answered. The fixture is
/// what preserved it.
///
/// An existing fixture is **left alone and said so on stderr**, rather than failing the run: adding
/// a scene means running an arm whose other scenes are already captured, and a refusal there would
/// make the new capture impossible to take without deleting the old evidence first. Saying nothing
/// is the other wrong answer — an operator who meant to regenerate would read silence as success.
///
/// # Errors
///
/// The write failing. A capture that cannot be saved when saving was asked for is a failed run, not
/// a run with a missing side effect.
pub fn save_if_asked(which: &str, dialect: Dialect, bytes: &[u8]) -> Result<(), String> {
    let Ok(prefix) = std::env::var("CONFORM_SAVE_CAPTURE") else {
        return Ok(());
    };
    let to = format!(
        "{prefix}-scene{which}-{}.{}",
        scene_tag(which),
        scene_ext(which, dialect)
    );
    if std::fs::exists(&to).unwrap_or(false) {
        eprintln!(
            "{to} already exists and was left alone — a capture is evidence and is never \
             regenerated to make something pass. Move it aside by hand if this run really is a new \
             claim"
        );
        return Ok(());
    }
    std::fs::write(&to, bytes).map_err(|e| format!("saving the capture: {e}"))?;
    eprintln!("capture saved to {to}");
    Ok(())
}

/// The word in a fixture's name that says which scene it is a capture of.
fn scene_tag(which: &str) -> &'static str {
    match which {
        "01" => "attrs",
        "04" => "pairs",
        "05" => "widths",
        "06" => "sync",
        other => panic!("no such scene: {other}"),
    }
}

/// The extension that says which channel a fixture came through.
///
/// **Not decoration.** A `.vt` is a screen and reads through [`vitui_conform::parse`]; a `.cpr` is
/// a terminal's own answers and reads through [`cursor_reports`]. Handing either to the other
/// produces a refusal rather than a wrong number, and naming them apart is what stops a reader
/// having to find that out.
///
/// A photograph's extension is the **arm's**, because the sixth arm's is a `grid.json` rather than
/// an escape stream — see [`Dialect::capture_ext`].
fn scene_ext(which: &str, dialect: Dialect) -> &'static str {
    match which {
        "05" => "cpr",
        "06" => "decrqm",
        // **The arm's serialisation and not a constant**, since the sixth arm's photographs are not
        // an escape stream at all. The rule is the one this doc comment already states: handing a
        // grid to the SGR parser produces a refusal rather than a wrong number, so the two are named
        // apart. [`Dialect::capture_ext`] is where that mapping lives, beside the enum it is about.
        _ => dialect.capture_ext(),
    }
}
