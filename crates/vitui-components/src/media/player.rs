//! **A video player's chrome, taken apart part by part — six of ten ship here.**
//!
//! The video player's chrome.
//!
//! The survey's verdict on a video player is *transport, seek bar, timeline, volume, subtitles,
//! playlist, chapters: all ✅ cells; the picture is the problem*, and the earlier proposal
//! repeats it as *the whole chrome of a video player … no engine change for any of them*. Both
//! sentences are about the engine, and **the engine is not what the chrome was waiting for**.
//!
//! [`PARTS`] is that verdict as a value with the column the survey did not have: what each part is
//! waiting for. Six wait for nothing and ship here. Two — the seek bar and the volume — are
//! `slider`, frozen in Tier 3 for an **unmeasured mechanism**; the mechanism is measured
//! here ([`scrub`]) and the component is `input`'s. One is the playhead, which owns a
//! clock. One is the picture, which nothing here draws.
//!
//! # The mechanism is one function and it reads one field
//!
//! [`scrub`] is the whole of drag capture: **the value is `Response::local` divided by
//! `Response::rect`**, with no press origin, no stored anchor and no cross-frame fact of any kind.
//! A press jumps to where it landed, a move carries it, and a release moves nothing — which is what
//! a seek bar does and what a delta-only API cannot express. `crate::picture`'s drag gate asserts
//! the three phases at `20/299`, `60/299` and unchanged.
//!
//! **`Response::rect` and not the rectangle the caller passed**: a slider that took both would have
//! two widths that can disagree, and the one the pointer was resolved against is the one on the
//! `Response`.
//!
//! # The allocation this module is not allowed to reintroduce
//!
//! A defect was found *here*: `chrome` collected a `Vec<f32>` of chapter positions on the
//! draw path — **40 allocations over 40 frames against a budget of zero**, and *found only when the
//! figure was computed as a total rather than an integer mean*. The marks are a field now,
//! [`Player::marks`], derived once at construction. [`defective::chrome_collecting_into`] is the
//! shape it replaced, kept so the total-versus-mean argument has something to fire on: over a run
//! where only some frames are tall enough to draw the chapter list, the total is the defect and the
//! integer mean is **0**.

use std::time::{Duration, Instant};

use vitui_runtime::layout::rect;
use vitui_runtime::{Ctx, Glyph, Id, Interest, Rect, Response, Role};

use super::Census;
use crate::ink::{Direct, Ink};
use crate::input::{ButtonOpts, button_into};
use crate::text::{Justify, TextOpts, text_into};

/// **What a part of the chrome is waiting for.** The column that makes the survey's list a table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Needs {
    /// Built on mechanisms a resolved ticket already measured. **These are the six that ship.**
    Nothing,
    /// The pointer grab: `Interest::DRAG` and `Response::local`. Measured by [`scrub`]; the
    /// component that wraps it — the thumb, the keyboard, the step, the orientation — is `slider`,
    /// the `slider` component.
    DragCapture,
    /// A component that owns a clock and re-registers a deadline while it runs.
    ///
    /// **Nothing is waiting on this any more.** It was prototyped and then
    /// built it, and the permission is narrower than the row's own wording: a component may own an
    /// **anchor** and may not own a **clock**. [`Playhead`] is the anchor and `Ctx::now` is the
    /// clock, so *playhead advance* is a [`Needs::Nothing`] row now and this variant
    /// records what it was waiting for rather than what still is.
    ///
    /// It is kept rather than struck for [`PARTS`]'s own reason: the column is the survey's verdict
    /// with the thing it was waiting for beside it, and a variant deleted the day its last row
    /// moved is a table that cannot say what it used to say. `tests::the_parts_table_is_the_surveys
    /// _verdict_with_a_column_added` asserts no row carries it.
    Clock,
    /// The engine's out-of-band graphics. Not built here at all.
    Passthrough,
}

/// One part of the chrome.
#[derive(Clone, Copy, Debug)]
pub struct Part {
    /// What it is.
    pub name: &'static str,
    /// What it is waiting for.
    pub needs: Needs,
}

/// **The chrome, enumerated.** Every entry the survey ticked, plus the picture it did not.
pub const PARTS: [Part; 10] = [
    Part {
        name: "transport buttons",
        needs: Needs::Nothing,
    },
    Part {
        name: "playlist",
        needs: Needs::Nothing,
    },
    Part {
        name: "chapter list",
        needs: Needs::Nothing,
    },
    Part {
        name: "subtitle line",
        needs: Needs::Nothing,
    },
    Part {
        name: "elapsed / total",
        needs: Needs::Nothing,
    },
    Part {
        name: "chapter marks on the bar",
        needs: Needs::Nothing,
    },
    Part {
        name: "seek bar",
        needs: Needs::DragCapture,
    },
    Part {
        name: "volume",
        needs: Needs::DragCapture,
    },
    Part {
        // **The seventh to ship, and an earlier pass is where it did.** `chrome_playing_into`
        // is the row: an anchor, `Ctx::now`, and an ask that fires when the drawn column moves.
        name: "playhead advance",
        needs: Needs::Nothing,
    },
    Part {
        name: "the picture",
        needs: Needs::Passthrough,
    },
];

/// **How many of [`PARTS`] ship here. Seven.** The headline as a count over the table above rather
/// than as a number in a sentence.
///
/// It was **six**, and the seventh is *playhead advance* — the one row of
/// the ten whose [`Needs`] was [`Needs::Clock`] and the only mechanism in this family that is not
/// drag capture. What is left is the two `slider` rows, which are `crate::input`'s and ship as a
/// component rather than as chrome, and the picture, which nothing here draws.
pub const SHIPPED: usize = 7;

// ── the mechanism ────────────────────────────────────────────────────────────────────────────────

/// **Where along a track the pointer is, while it is held — and nothing else.**
///
/// `None` when nothing is being dragged. `Some(0.0..=1.0)` on the press frame and on every frame the
/// button is held, which is what makes a press *jump* rather than start a delta.
///
/// # It is `Response::local` and `Response::rect` and there is no third fact
///
/// That sentence is the deliverable. `Response::local` is the pointer in the widget's own
/// coordinates — added to the runtime's proposal for exactly this and then called by nothing until
/// now — and `Response::rect` is the width it was resolved against. There is no press origin, no
/// stored anchor, no *was I dragging last frame*, and therefore nothing for a caller to keep in
/// sync. A component that owns a value calls this and assigns; that is the whole grab.
///
/// ```
/// use vitui_components::media::player::scrub;
/// use vitui_runtime::{Id, Rect, Response};
///
/// let mut resp = Response::inert(Id::from_raw(1), Rect::new(0, 0, 300, 1));
/// // Nothing held: no answer, which is not the same as an answer of zero.
/// assert_eq!(scrub(&resp), None);
///
/// resp.pressed = true;
/// resp.local = Some((20, 0));
/// // A press jumps to where it landed.
/// assert_eq!(scrub(&resp), Some(20.0 / 299.0));
/// ```
#[must_use]
pub fn scrub(resp: &Response) -> Option<f32> {
    if !resp.pressed {
        return None;
    }
    let (lx, _) = resp.local?;
    let span = i32::from(resp.rect.w).max(1) - 1;
    let span = span.max(1);
    Some(lx.clamp(0, span) as f32 / span as f32)
}

// ── the state ────────────────────────────────────────────────────────────────────────────────────

/// One chapter: where it starts, as a fraction of the whole, and what it is called.
#[derive(Clone, Debug)]
pub struct Chapter {
    /// Where it starts, `0.0..=1.0`.
    pub at: f32,
    /// What it is called.
    pub name: String,
}

/// **What the chrome draws, and the one derived field that keeps the draw path clean.**
///
/// [`Player::marks`] is [`Player::chapters`]'s first column, derived once by [`Player::new`] and
/// once by [`Player::set_chapters`]. See this module's header for why it is a field.
#[derive(Clone, Debug)]
pub struct Player {
    /// Where the playhead is, `0.0..=1.0`.
    pub position: f32,
    /// How long the whole thing is, in seconds.
    pub duration_s: f32,
    /// Whether it is playing. The chrome draws it; nothing here advances it, which is
    /// [`Needs::Clock`].
    pub playing: bool,
    /// The volume, `0.0..=1.0`.
    pub volume: f32,
    /// The subtitle line standing right now.
    pub subtitle: String,
    /// What is queued.
    pub playlist: Vec<String>,
    /// Which of [`Player::playlist`] is playing.
    pub track: usize,
    /// **Set while the pointer is held on the track**, so a caller can suppress the source's own
    /// updates — the *scrubbing* state every real player has and no static description of a seek
    /// bar does.
    pub scrubbing: bool,
    chapters: Vec<Chapter>,
    marks: Vec<f32>,
}

impl Player {
    /// A player over a playlist and a chapter list.
    pub fn new(duration_s: f32, playlist: Vec<String>, chapters: Vec<Chapter>) -> Player {
        let marks = chapters.iter().map(|c| c.at).collect();
        Player {
            position: 0.0,
            duration_s,
            playing: false,
            volume: 0.6,
            subtitle: String::new(),
            playlist,
            track: 0,
            scrubbing: false,
            chapters,
            marks,
        }
    }

    /// The chapters.
    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    /// **The chapter positions, derived once.** What the draw path reads instead of collecting one.
    pub fn marks(&self) -> &[f32] {
        &self.marks
    }

    /// Replace the chapters, re-deriving [`Player::marks`] with them.
    pub fn set_chapters(&mut self, chapters: Vec<Chapter>) {
        self.marks.clear();
        self.marks.extend(chapters.iter().map(|c| c.at));
        self.chapters = chapters;
    }
}

// ── the draw ─────────────────────────────────────────────────────────────────────────────────────

/// **How tall the band above the transport block has to be before the two lists are drawn.**
///
/// Below it the band is padding, which is the ordinary short-screen answer and also the thing that
/// makes [`defective::chrome_collecting_into`] allocate on some frames and not others.
pub const LISTS_MIN_H: u16 = 3;

/// How many rows the transport block takes: subtitle, track, times, buttons.
pub const TRANSPORT_H: u16 = 4;

/// The five transport buttons, in order. `Play` and `Pause` share a slot.
const TRANSPORT: [&str; 5] = ["|<", "<<", ">", ">>", ">|"];

/// The index of the play/pause button in [`TRANSPORT`].
const PLAY_PAUSE: usize = 2;

/// How wide one transport button is.
const BUTTON_W: u16 = 4;

/// **The whole chrome, as a partition of `area`.** Returns the **track**'s response, so a caller can
/// hand it to [`scrub`].
///
/// Every cell of `area` is written exactly once, so nothing is owed back and the return value is
/// free to be the one thing a caller needs.
///
/// ```
/// use vitui_components::media::player::{Chapter, Player, chrome, scrub};
/// use vitui_runtime::ctx::Driver;
///
/// let mut p = Player::new(600.0, vec!["one".into()], vec![Chapter { at: 0.5, name: "half".into() }]);
/// let mut driver = Driver::headless(40, 10).expect("a sink attaches");
/// driver.frame(|cx| {
///     let track = chrome(cx, cx.area(), &mut p);
///     // Nothing is held, so the grab has no answer.
///     assert_eq!(scrub(&track), None);
/// });
/// ```
#[track_caller]
pub fn chrome(cx: &mut Ctx<'_, '_>, area: Rect, p: &mut Player) -> Response {
    chrome_into(&mut Direct, cx, area, p, &mut Census::default())
}

/// **[`chrome`], drawing through an [`Ink`] and publishing its [`Census`] — which is zero customs.**
///
/// The chrome is theme-coloured throughout: it is the contrast a picture is drawn against.
#[track_caller]
pub fn chrome_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    p: &mut Player,
    census: &mut Census,
) -> Response {
    let id = cx.id();
    draw_chrome(ink, cx, area, id, p, census, Marks::Field)
}

// ── the playhead: the seventh part, and the one that owns an anchor ──────────────────────────────

/// **When a playhead asks for its next frame.**
///
/// The two arms are not two optimisations of one thing: **only an anchored playhead can compute the
/// first**, because the instant the drawn column next moves is a function of `(anchor, duration,
/// width)`. A component with no anchor has nothing to project from and reaches for the frame rate,
/// which is the second arm and is what it costs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Cadence {
    /// **The rule.** The instant the drawn column would move, computed from the anchor.
    #[default]
    NextCellChange,
    /// The frame rate, because that is what a component with no anchor can reach for.
    EveryFrame,
}

/// **What a playhead keeps across frames: an anchor and nothing else.**
///
/// `(when, where)` — the moment the run began and the position it began at. Every other fact is
/// recoverable from it: where it is now, whether it has run out, when the drawn column next moves,
/// and when the last frame of the run will be.
///
/// # It is the spinner's opposite on two axes, and both decide something
///
/// - **It lands.** [`crate::indicate::spinner`] cycles for ever and has no target, so it has no
///   transient at all and is quiet on the frame after it stops. A playhead runs out at `1.0`, and
///   the instant it does is [`Playhead::ends_at`] — arithmetic on the anchor, computable before it
///   gets there.
/// - **Its visible state changes far more slowly than a frame.** A spinner's step is chosen so a
///   person sees motion, so it is ~80 ms. A playhead's step is **the width of one track column**,
///   which over a two-hour film on a sixty-column bar is two minutes — and that is
///   [`Cadence`]'s whole subject.
///
/// # A scrub is a re-anchor and there is nowhere else for it to be
///
/// [`scrub`] answers `Some(0.0..=1.0)` from `Response::local` over `Response::rect` and nothing
/// else. It carries no time and must not, because it is a *position* — so the seam is
/// [`Playhead::seek`], which is one assignment: the anchor moves to `(now, where the pointer is)`
/// and everything downstream is recomputed from it. **No stored velocity, no was-I-scrubbing-last-
/// frame, and no second clock**, and [`Playhead::ends_at`] moves with it for free.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Playhead {
    /// `Some((t0, p0))` while it runs. The anchor, and the only cross-frame fact there is.
    anchor: Option<(Instant, f32)>,
    /// The cadence it asks on.
    pub cadence: Cadence,
}

/// What the anchor costs. Quoted against [`crate::indicate::ANCHOR_BYTES`], which is the spinner's.
pub const PLAYHEAD_BYTES: usize = size_of::<Option<(Instant, f32)>>();

impl Playhead {
    /// A paused playhead.
    #[must_use]
    pub const fn paused() -> Playhead {
        Playhead {
            anchor: None,
            cadence: Cadence::NextCellChange,
        }
    }

    /// **Start running from `from`, at `now`.**
    pub const fn play(&mut self, now: Instant, from: f32) {
        self.anchor = Some((now, from));
    }

    /// Whether it is running.
    #[must_use]
    pub const fn running(&self) -> bool {
        self.anchor.is_some()
    }

    /// **Pause**, resolving the position into the caller's [`Player`] on the way out.
    pub fn pause(&mut self, now: Instant, p: &mut Player) {
        p.position = self.position(now, p);
        self.anchor = None;
        p.playing = false;
    }

    /// **Where it is at `now`.** Clamped at `1.0`, which is where it lands.
    #[must_use]
    pub fn position(&self, now: Instant, p: &Player) -> f32 {
        let Some((t0, p0)) = self.anchor else {
            return p.position;
        };
        if p.duration_s <= 0.0 {
            return p0;
        }
        let elapsed = now.saturating_duration_since(t0).as_secs_f32();
        (p0 + elapsed / p.duration_s).clamp(0.0, 1.0)
    }

    /// **A scrub is a re-anchor.** See [`Playhead`]'s header: one assignment and no second fact.
    pub const fn seek(&mut self, now: Instant, to: f32) {
        if self.anchor.is_some() {
            self.anchor = Some((now, to));
        }
    }

    /// **When the last frame of this run is** — the instant the position reaches `1.0`.
    ///
    /// A spinner has no answer to this question, which is why *frames to quiet* is a count here and
    /// a cadence of its own.
    #[must_use]
    pub fn ends_at(&self, p: &Player) -> Option<Instant> {
        let (t0, p0) = self.anchor?;
        if p.duration_s <= 0.0 {
            return None;
        }
        let left = (1.0 - p0).max(0.0) * p.duration_s;
        Some(t0 + Duration::from_secs_f32(left))
    }

    /// **When the drawn column next moves**, on a track `w` columns wide.
    ///
    /// The whole of [`Cadence::NextCellChange`]: `1 / w` of the position is one column, so the next
    /// change is the anchor plus `(k + 1) / w` of the duration, where `k` is the column the position
    /// is in now. It never answers a moment in the past and never one past the end.
    #[must_use]
    pub fn next_cell_change(&self, now: Instant, p: &Player, w: u16) -> Option<Instant> {
        let (t0, p0) = self.anchor?;
        if p.duration_s <= 0.0 || w == 0 {
            return None;
        }
        let at = self.position(now, p);
        if at >= 1.0 {
            return None;
        }
        let cell = (at * f32::from(w)).floor();
        let next = ((cell + 1.0) / f32::from(w)).min(1.0);
        let secs = (next - p0) * p.duration_s;
        if secs <= 0.0 {
            return Some(now);
        }
        Some(t0 + Duration::from_secs_f32(secs))
    }

    /// **What it asks for at `now`**, `None` when it wants nothing.
    ///
    /// `frame_dt` is what [`Cadence::EveryFrame`] reaches for, and it is a **nominal** interval —
    /// the arm has nothing better, which is half of why it is the wrong arm: `vitui_runtime::anim`
    /// measures a nominal `dt` at 16.9% slow over three seconds, and the shortfall is a *rate*.
    #[must_use]
    pub fn wake(&self, now: Instant, p: &Player, w: u16, frame_dt: Duration) -> Option<Instant> {
        self.anchor?;
        if self.position(now, p) >= 1.0 {
            // It has landed. Nothing to ask for, and this is the frame that says so.
            return None;
        }
        match self.cadence {
            Cadence::NextCellChange => self.next_cell_change(now, p, w),
            Cadence::EveryFrame => Some(now + frame_dt),
        }
    }

    /// **How many wakes a whole run costs at this cadence.** The four-orders-of-magnitude figure.
    #[must_use]
    pub fn wakes_over_a_run(&self, p: &Player, w: u16, frame_dt: Duration) -> u64 {
        match self.cadence {
            Cadence::NextCellChange => u64::from(w),
            Cadence::EveryFrame => {
                let dt = frame_dt.as_secs_f64();
                if dt <= 0.0 {
                    0
                } else {
                    (f64::from(p.duration_s) / dt) as u64
                }
            }
        }
    }
}

/// **Advance the chrome's playhead, draw, take the scrub, and ask for the frame that will move the
/// drawn column.**
///
/// [`chrome`] draws what the [`Player`] says and moves nothing, which is what [`Needs::Clock`] was
/// waiting for. This is that row, and it is four lines around the same body:
///
/// 1. the position comes from the **anchor** before anything is drawn, so everything on the frame
///    agrees about where the playhead is;
/// 2. the chrome draws, and returns the **track's** response — which is the one place the track's
///    width lives, so the cadence and the grab read the same number [`scrub`] was resolved against;
/// 3. a held pointer is a **re-anchor** — [`Playhead::seek`] — and not a second stored fact;
/// 4. the ask goes through `Ctx::deadline_for` under the **chrome's** id, because a playhead is a
///    part of the chrome and the widget that wants the frame is the chrome.
///
/// It asks only when the drawn column will move, which is [`Cadence`]'s subject and the reason the
/// anchor is not merely tidy: over a two-hour film on a sixty-column bar it is **60 wakes against
/// 431 991**, and only an anchored playhead can compute the first.
///
/// The return value is [`chrome`]'s — the track's response — so a caller that wants the grab for
/// something else still has it.
#[track_caller]
pub fn chrome_playing_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    p: &mut Player,
    head: &mut Playhead,
    census: &mut Census,
    frame_dt: Duration,
) -> Response {
    let id = cx.id();
    let now = cx.now();
    if head.running() {
        p.position = head.position(now, p);
        p.playing = true;
    }
    let track = draw_chrome(ink, cx, area, id, p, census, Marks::Field);
    // **A scrub is a re-anchor**, applied on the frame that produced it: read after the draw,
    // because that is when the grab has an answer, and before the ask, because the ask projects
    // from the anchor.
    if let Some(to) = scrub(&track) {
        p.position = to;
        head.seek(now, to);
    }
    if head.running() && p.position >= 1.0 {
        // It landed, and this is the frame that says so: the anchor goes, so nothing asks. One
        // frame to quiet, and there is no transient to run out.
        head.pause(now, p);
    }
    if let Some(at) = head.wake(now, p, track.rect.w, frame_dt) {
        cx.deadline_for(id, at);
    }
    track
}

/// Where the chapter marks the track draws come from. See [`defective`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Marks {
    /// [`Player::marks`], derived at construction.
    Field,
    /// Collected on the draw path — one allocation every frame the chapter list is drawn.
    Collected,
}

/// The body both arms are.
///
/// # It roots every child inside its own id, and the review caught it not doing so
///
/// *A container roots its children inside its own id.* The chrome draws a track and five
/// keyed buttons and two keyed lists, and without the scope below **every chrome on a screen derives
/// the same ids** — `Ctx::id` mints from `Location::caller()`, and a line inside a private body is
/// one line however many call sites reach it, while `Ctx::with_key` roots at whatever the enclosing
/// stack happens to be.
///
/// What that costs is `Ctx::interact`'s: a merged claim is **inert**, so the second player's seek
/// bar takes no press, no drag and no hover and its transport buttons never click, on a screen that
/// renders perfectly. `collect::collection` carries the same call for the same reason and says so.
///
/// The `#[track_caller]` on the three public entry points is the other half and neither works alone:
/// the attribute makes the id the *caller's* line, and the scope is what puts the children under it.
#[allow(clippy::too_many_arguments)]
fn draw_chrome<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    id: Id,
    p: &mut Player,
    census: &mut Census,
    marks: Marks,
) -> Response {
    cx.with_id(id, |cx| draw_chrome_inner(ink, cx, area, p, census, marks))
}

/// The drawing half, inside the chrome's own id.
fn draw_chrome_inner<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    p: &mut Player,
    census: &mut Census,
    marks: Marks,
) -> Response {
    let block = TRANSPORT_H.min(area.h);
    let (lists, transport) = rect::split_at_v(area, area.h - block);
    let (subtitle, rest) = rect::split_at_v(transport, 1.min(transport.h));
    let (bar, rest) = rect::split_at_v(rest, 1.min(rest.h));
    let (times, buttons) = rect::split_at_v(rest, 1.min(rest.h));

    // Part 4: the subtitle line.
    census.roles += 1;
    text_into(
        ink,
        cx,
        subtitle,
        &p.subtitle,
        &TextOpts {
            justify: Justify::Middle,
            role: Role::Title,
            pad: Role::Body,
            interest: None,
        },
    );

    // Parts 2 and 3: the playlist and the chapter list, when there is room for them. Both are drawn
    // before the track, because the chapter list is what the collected arm allocates for.
    let collected = lists_band(ink, cx, lists, p, census, marks);

    // Part 6: the chapter marks, on the track the grab reads.
    let marks: &[f32] = match &collected {
        Some(v) => v,
        None => &p.marks,
    };
    let track = track_into(ink, cx, bar, p.position, marks, census);
    p.scrubbing = track.pressed;

    // Part 5: elapsed / total.
    let (left, right) = rect::split_at_h(times, times.w / 2);
    let elapsed = Stamp::of(p.position * p.duration_s);
    let total = Stamp::of(p.duration_s);
    census.roles += 2;
    text_into(
        ink,
        cx,
        left,
        elapsed.as_str(),
        &TextOpts {
            role: Role::Body,
            ..TextOpts::default()
        },
    );
    text_into(
        ink,
        cx,
        right,
        total.as_str(),
        &TextOpts {
            justify: Justify::End,
            role: Role::Dim,
            pad: Role::Body,
            interest: None,
        },
    );

    // Part 1: the transport.
    transport_row(ink, cx, buttons, p, census);
    track
}

/// The two lists, or the padding that stands where they would be. Returns the collected marks on
/// the arm that collects them, so [`defective`] is one value rather than a second body.
fn lists_band<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    band: Rect,
    p: &Player,
    census: &mut Census,
    marks: Marks,
) -> Option<Vec<f32>> {
    let pad = TextOpts {
        role: Role::Body,
        ..TextOpts::default()
    };
    if band.h < LISTS_MIN_H {
        census.roles += 1;
        text_into(ink, cx, band, "", &pad);
        return None;
    }
    let (left, right) = rect::split_at_h(band, band.w / 2);

    // Part 2: the playlist.
    for row in 0..left.h {
        let line = Rect::new(left.x, left.y + i32::from(row), left.w, 1);
        let entry = p.playlist.get(usize::from(row));
        census.roles += 1;
        cx.with_key(u64::from(row), |cx| {
            text_into(
                ink,
                cx,
                line,
                entry.map_or("", String::as_str),
                &TextOpts {
                    role: if Some(usize::from(row)) == Some(p.track) && entry.is_some() {
                        Role::Ok
                    } else {
                        Role::Body
                    },
                    ..pad
                },
            );
        });
    }

    // Part 3: the chapter list. **The one part the collected arm allocates for**, which is what
    // makes the defect rarer than one a frame and therefore invisible to an integer mean.
    let collected = match marks {
        Marks::Field => None,
        Marks::Collected => Some(p.chapters.iter().map(|c| c.at).collect::<Vec<f32>>()),
    };
    for row in 0..right.h {
        let line = Rect::new(right.x, right.y + i32::from(row), right.w, 1);
        let (name, at) = match p.chapters.get(usize::from(row)) {
            Some(c) => (c.name.as_str(), Some(c.at)),
            None => ("", None),
        };
        let (label, stamp) = rect::split_at_h(line, line.w.saturating_sub(STAMP_W));
        census.roles += 2;
        cx.with_key(u64::from(row) | CHAPTER_KEY, |cx| {
            text_into(ink, cx, label, name, &pad);
            let time = at.map(|v| Stamp::of(v * p.duration_s));
            text_into(
                ink,
                cx,
                stamp,
                time.as_ref().map_or("", Stamp::as_str),
                &TextOpts {
                    justify: Justify::End,
                    role: Role::Dim,
                    pad: Role::Body,
                    interest: None,
                },
            );
        });
    }
    collected
}

/// The high bit of a chapter row's key, so a chapter row and a playlist row of the same index are
/// two ids rather than one. `Ctx::with_key` takes the whole `u64`, and two containers under one
/// parent that both key by row index are exactly the merge `crate::scroll` was caught making.
const CHAPTER_KEY: u64 = 1 << 63;

/// **The track the marks sit on and the grab reads.** Declares `CLICK | DRAG | HOVER`, which is what
/// makes `Response::local` arrive at all.
fn track_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    position: f32,
    marks: &[f32],
    census: &mut Census,
) -> Response {
    let id = cx.id();
    let resp = cx.interact(
        id,
        area,
        Interest::CLICK.with(Interest::DRAG).with(Interest::HOVER),
    );
    if area.w == 0 || area.h == 0 {
        return resp;
    }
    let theme = cx.theme();
    census.roles += 3;
    let done = theme.paint(Role::Ok);
    let rest = theme.paint(Role::Dim);
    let thumb_paint = theme.paint(if resp.pressed {
        Role::FaceActive
    } else if resp.hovered {
        Role::FaceHover
    } else {
        Role::Face
    });
    let played = theme.glyph(Glyph::HLine);
    let unplayed = theme.glyph(Glyph::Track);
    let mark = theme.glyph(Glyph::VLine);
    let thumb = theme.glyph(Glyph::Thumb);

    let span = area.w - 1;
    let at = |v: f32| (v.clamp(0.0, 1.0) * f32::from(span)).round() as u16;
    let filled = at(position);
    for row in 0..area.h {
        let y = area.y + i32::from(row);
        // A partition of the row in two runs, and then the marks and the thumb over it — every one
        // of which is a cell the two runs already wrote, so `distinct` and `writes` disagree by
        // exactly the marks. See the module's own gate.
        ink.run(cx, area.x, y, played, filled, done);
        ink.run(
            cx,
            area.x + i32::from(filled),
            y,
            unplayed,
            area.w - filled,
            rest,
        );
    }
    for m in marks {
        ink.run(cx, area.x + i32::from(at(*m)), area.y, mark, 1, rest);
    }
    ink.run(
        cx,
        area.x + i32::from(filled),
        area.y,
        thumb,
        1,
        thumb_paint,
    );
    resp
}

/// Part 1: five buttons and the padding after them, as a partition of the row.
fn transport_row<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    row: Rect,
    p: &mut Player,
    census: &mut Census,
) {
    let mut x = row.x;
    for (i, glyph) in TRANSPORT.into_iter().enumerate() {
        if x + i32::from(BUTTON_W) > row.right() {
            break;
        }
        let label = if i == PLAY_PAUSE && p.playing {
            "||"
        } else {
            glyph
        };
        let at = Rect::new(x, row.y, BUTTON_W, row.h);
        let clicked = cx.with_key(i as u64, |cx| {
            button_into(ink, cx, at, label, &ButtonOpts::default()).clicked
        });
        if clicked && i == PLAY_PAUSE {
            p.playing = !p.playing;
        }
        x += i32::from(BUTTON_W);
    }
    let used = u16::try_from(x - row.x).unwrap_or(row.w).min(row.w);
    census.roles += 1;
    text_into(
        ink,
        cx,
        Rect::new(x, row.y, row.w - used, row.h),
        "",
        &TextOpts {
            role: Role::Body,
            ..TextOpts::default()
        },
    );
}

// ── the clock face ───────────────────────────────────────────────────────────────────────────────

/// How wide a [`Stamp`] is: `MM:SS`.
const STAMP_W: u16 = 5;

/// **`MM:SS`, formatted onto the stack.**
///
/// `format!` would allocate, and a chrome that allocates once a frame is the defect this module
/// exists downstream of. Eight bytes and a length, filled by division, which is what a zero
/// allocation budget costs a label that changes every frame.
struct Stamp {
    buf: [u8; 8],
    len: usize,
}

impl Stamp {
    /// The stamp for a number of seconds, saturating at `99:59`.
    fn of(seconds: f32) -> Stamp {
        // **Saturating on the pair and not on the minutes alone**, which the first version got
        // wrong: clamping only the left half turns 4 369 minutes into `99:00`, and a ceiling that
        // reads like a time is worse than one that reads like a ceiling.
        let total = (seconds.max(0.0) as u32).min(99 * 60 + 59);
        let minutes = total / 60;
        let seconds = total % 60;
        let mut buf = [b'0'; 8];
        buf[0] = b'0' + (minutes / 10) as u8;
        buf[1] = b'0' + (minutes % 10) as u8;
        buf[2] = b':';
        buf[3] = b'0' + (seconds / 10) as u8;
        buf[4] = b'0' + (seconds % 10) as u8;
        Stamp { buf, len: 5 }
    }

    /// It as a string. Always ASCII, so the slice is always valid UTF-8.
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("--:--")
    }
}

// ── the shape that is kept because it was wrong ──────────────────────────────────────────────────

/// **The chrome as it was before the marks became a field.**
///
/// It is recorded: `player::chrome` collected a `Vec<f32>` on the draw path, **40 allocations over
/// 40 frames against a budget of zero**, *found only when the figure was computed as a total rather
/// than an integer mean*. The ticket's own instruction is *do not fix it again; keep the shape it
/// replaced as a negative case*, and this is that case.
pub mod defective {
    use super::{Census, Ctx, Ink, Marks, Player, Rect, Response, draw_chrome};

    /// **The chapter marks collected on the draw path.**
    ///
    /// One `Vec<f32>` per frame **that draws the chapter list** — which is the half that matters: a
    /// defect that fires on every frame is a mean of one and a defect that fires on some frames is
    /// a mean of zero, and only the total tells them apart. See
    /// `crate::picture::tests::the_chrome_allocates_nothing_and_the_shape_it_replaced_allocates_a_
    /// total_no_mean_would_show`.
    #[track_caller]
    pub fn chrome_collecting_into<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        p: &mut Player,
        census: &mut Census,
    ) -> Response {
        let id = cx.id();
        draw_chrome(ink, cx, area, id, p, census, Marks::Collected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{Canvas, Pen};
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::{Button, Buttons, Id, Mods, Mouse, MouseKind, Rect};

    /// The screen the drag is played on: `300` wide, so the span is **299** and the two fractions
    /// are the fractions it wrote down.
    const W: u16 = 300;

    /// Four rows, which is exactly [`TRANSPORT_H`] — so the lists band is empty and the track is on
    /// a row this test can name.
    const H: u16 = 4;

    /// The row the track is on: subtitle, **track**, times, buttons.
    const TRACK_ROW: u16 = 1;

    /// A pointer event at `x` on the track's row.
    fn at(x: u16, kind: MouseKind) -> Mouse {
        Mouse {
            x,
            y: TRACK_ROW,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        }
    }

    /// A player with something in every part.
    fn player() -> Player {
        Player::new(
            600.0,
            vec!["one".to_owned(), "two".to_owned()],
            vec![
                Chapter {
                    at: 0.0,
                    name: "cold open".to_owned(),
                },
                Chapter {
                    at: 0.5,
                    name: "the seam".to_owned(),
                },
            ],
        )
    }

    /// **Seven of ten, counted over the table rather than stated.**
    #[test]
    fn seven_parts_ship_and_the_other_three_each_name_what_they_wait_for() {
        let shipped = PARTS.iter().filter(|p| p.needs == Needs::Nothing).count();
        assert_eq!(shipped, SHIPPED);
        assert_eq!(PARTS.len(), 10);

        let waiting: Vec<Needs> = PARTS
            .iter()
            .map(|p| p.needs)
            .filter(|n| *n != Needs::Nothing)
            .collect();
        assert_eq!(
            waiting,
            vec![Needs::DragCapture, Needs::DragCapture, Needs::Passthrough],
            "the three that do not ship, and the two drag-capture rows are `slider`'s"
        );
        // **`Needs::Clock` has no row and the variant stays**, which is the table saying what it
        // used to say. An earlier pass built the playhead, so the column moved and the
        // vocabulary did not: a variant deleted the day its last row moved is a table that cannot
        // record what a part was waiting for.
        assert!(
            !PARTS.iter().any(|p| p.needs == Needs::Clock),
            "a part is waiting on a clock again, and ticket 46 settled that a component may own an \
             anchor and may not own a clock"
        );
    }

    /// **It lands, and the landing frame is the frame that stops asking.**
    ///
    /// *Frames to quiet* is **1** for the playhead and 1 for the spinner, for opposite reasons: a
    /// spinner has nothing to land on, and a playhead's landing instant is arithmetic on the anchor.
    /// Neither is a cadence — *14 frames to quiet* is one wearing a count's clothes, 12 at
    /// sixty hertz and 24 at a hundred and twenty — because neither has a transient to run out.
    #[test]
    fn a_playhead_lands_and_the_landing_frame_asks_for_nothing() {
        let now = Instant::now();
        let p = player();
        let dt = Duration::from_millis(16);
        let mut head = Playhead::paused();
        assert_eq!(
            head.wake(now, &p, 60, dt),
            None,
            "a paused playhead asks nothing"
        );
        head.play(now, 0.0);
        assert!(head.wake(now, &p, 60, dt).is_some());

        let end = head.ends_at(&p).expect("a running playhead lands");
        assert_eq!(end, now + Duration::from_secs_f32(p.duration_s));
        assert_eq!(head.position(end, &p), 1.0);
        assert_eq!(
            head.wake(end, &p, 60, dt),
            None,
            "the landing frame asks for nothing"
        );
    }

    /// **The cadence figure: 60 wakes against 431 991 over one two-hour run.**
    ///
    /// The number the anchor buys, and it is a **consequence** of the anchor rather than an
    /// optimisation available to either other arm: the instant the drawn column next moves is a
    /// function of `(anchor, duration, width)`, so a component that samples its own clock has a
    /// `now` nobody else on the screen agrees with and one that accumulates has nothing to project
    /// from at all.
    #[test]
    fn only_an_anchored_playhead_can_ask_when_the_column_moves() {
        let two_hours = Player::new(7_200.0, vec!["one".into()], Vec::new());
        // **60 Hz to the nearest whole microsecond**, and the spelling is load-bearing on this arm
        // and on no other: 16 666 µs answers 432 017 and 16 667 answers 431 991, because the
        // every-frame arm's count is a property of the *nominal* interval rather than of the run.
        // The cell-change arm answers 60 whatever is written here, which is the finding from its
        // other side.
        let dt = Duration::from_micros(16_667);
        let cell = Playhead {
            anchor: None,
            cadence: Cadence::NextCellChange,
        };
        let every = Playhead {
            anchor: None,
            cadence: Cadence::EveryFrame,
        };
        let (a, b) = (
            cell.wakes_over_a_run(&two_hours, 60, dt),
            every.wakes_over_a_run(&two_hours, 60, dt),
        );
        assert_eq!(
            a, 60,
            "one wake a column, and the bar is sixty columns wide"
        );
        assert_eq!(b, 431_991);
        assert_eq!(b / a, 7_199, "the ratio, which is the whole of the finding");

        // **And the next change really is a column away**, on a run rather than in the arithmetic:
        // over a ten-second film on a ten-column bar every wake advances the drawn column by one.
        let short = Player::new(10.0, vec!["one".into()], Vec::new());
        let now = Instant::now();
        let mut head = Playhead::paused();
        head.play(now, 0.0);
        let column = |at: Instant| (head.position(at, &short) * 10.0).floor() as i32;
        let mut at = now;
        for expected in 1..10 {
            at = head
                .next_cell_change(at, &short, 10)
                .expect("it has not landed");
            assert_eq!(column(at), expected, "the wake that moves the column");
        }
    }

    /// **A scrub is a re-anchor and nothing else.**
    ///
    /// One assignment: no stored velocity, no *was I scrubbing last frame*, no second clock — and
    /// `ends_at` moves with it for free, which is asserted rather than claimed. It is [`scrub`]'s
    /// own deliverable one axis over: the grab carries a *position* and must not carry a time.
    #[test]
    fn a_scrub_is_a_re_anchor_and_the_landing_moves_with_it() {
        let now = Instant::now();
        let p = player();
        let mut head = Playhead::paused();
        head.play(now, 0.0);
        let ends = head.ends_at(&p).expect("running");

        // Halfway through, dragged back to the start: the landing is a whole duration away again.
        let half = now + Duration::from_secs_f32(p.duration_s / 2.0);
        assert!((head.position(half, &p) - 0.5).abs() < 1e-3);
        head.seek(half, 0.0);
        let after = head.ends_at(&p).expect("still running");
        assert!(after > ends, "the landing did not move with the anchor");
        assert!((head.position(half, &p) - 0.0).abs() < 1e-6);

        // **A paused playhead is not re-anchored by a scrub**, because there is no anchor to move:
        // seeking a paused player is the caller's `Player::position`, which is the field the chrome
        // draws from either way.
        let mut paused = Playhead::paused();
        paused.seek(now, 0.5);
        assert!(!paused.running());
    }

    /// **The chrome advances its own playhead, asks under its own id, and takes the scrub back into
    /// the anchor.**
    ///
    /// The seventh part of [`PARTS`] as a frame rather than as arithmetic. The ask carries the
    /// **chrome's** id and not one of the playhead's own, because a playhead is a part of the
    /// chrome and the widget that wants the frame is the chrome — the prototype asked through
    /// `Ctx::deadline` in its first cut, and the census then named two widgets on a screen with
    /// three.
    #[test]
    fn the_chrome_advances_the_playhead_and_asks_under_its_own_id() {
        let dt = Duration::from_millis(16);
        let mut p = player();
        let mut head = Playhead::paused();
        let mut driver = Driver::headless(60, 10).expect("a sink cannot fail to attach");
        let mut pen = Pen::over(Canvas::new(60, 10));

        // Frame one: paused. Nothing moves and nothing is asked.
        let mut id = None;
        driver.frame(|cx| {
            let track = chrome_playing_into(
                &mut pen,
                cx,
                cx.area(),
                &mut p,
                &mut head,
                &mut Census::default(),
                dt,
            );
            id = Some(track.id);
        });
        assert_eq!(driver.inspect().wakes().line_count(), 0);

        // **The reading is a delta and never the total**, which is the trap an earlier pass
        // recorded against itself: `WakeLedger::line_count` and `asked_by` are **cumulative and
        // never reset**, so by the second frame every call site on the screen has registered and a
        // total is saturated — a playhead that never lands would measure exactly what a landed one
        // does.
        // It is the **sum over the ledger's lines** rather than `asked_by`, and that is a fact
        // about this seam worth one sentence: `chrome` returns the **track's** response so a caller
        // can hand it to `scrub`, and the ask carries the **chrome's** id — so an application
        // cannot name the widget that is keeping its screen awake, and only the census can.
        // `asked_by(track.id)` is asserted at zero below for exactly that reason.
        let asks = |driver: &Driver| {
            driver
                .inspect()
                .wakes()
                .lines()
                .map(|l| u64::from(l.asks))
                .sum::<u64>()
        };

        // Frame two: playing, on a pinned clock so the advance is a number rather than a race.
        let t0 = driver.env().now();
        let before = asks(&driver);
        head.play(t0, 0.0);
        driver.frame(|cx| {
            chrome_playing_into(
                &mut pen,
                cx,
                cx.area(),
                &mut p,
                &mut head,
                &mut Census::default(),
                dt,
            );
        });
        assert_eq!(asks(&driver) - before, 1, "a running playhead asks once");
        let wakes = driver.inspect().wakes();
        assert_eq!(
            wakes.census_len(),
            1,
            "and the ask names the chrome, which is the widget that wants the frame"
        );
        assert_eq!(
            wakes.asked_by(id.expect("a response")),
            0,
            "the ask carries the track's id, so the census names a part rather than the widget"
        );

        // Frame three: a quarter of the way in, and the position is the anchor's.
        driver.advance(Duration::from_secs_f32(p.duration_s / 4.0));
        driver.frame(|cx| {
            chrome_playing_into(
                &mut pen,
                cx,
                cx.area(),
                &mut p,
                &mut head,
                &mut Census::default(),
                dt,
            );
        });
        assert!((p.position - 0.25).abs() < 1e-2, "position {}", p.position);
        assert!(p.playing);

        // And past the end it lands: the anchor goes, the position is exactly 1.0, and the frame
        // that says so asks for nothing.
        driver.advance(Duration::from_secs_f32(p.duration_s));
        let before = asks(&driver);
        driver.frame(|cx| {
            chrome_playing_into(
                &mut pen,
                cx,
                cx.area(),
                &mut p,
                &mut head,
                &mut Census::default(),
                dt,
            );
        });
        assert!(!head.running(), "it did not land");
        assert_eq!(p.position, 1.0);
        assert!(!p.playing);
        assert_eq!(
            asks(&driver) - before,
            0,
            "the landing frame asked for another"
        );
        // One frame to quiet, and the frame after it is quiet too.
        let before = asks(&driver);
        driver.frame(|cx| {
            chrome_playing_into(
                &mut pen,
                cx,
                cx.area(),
                &mut p,
                &mut head,
                &mut Census::default(),
                dt,
            );
        });
        assert_eq!(asks(&driver) - before, 0);
    }

    /// **The grab reads two fields and there is no third.**
    ///
    /// The unit half; `crate::picture` plays the three phases through a posted pointer.
    #[test]
    fn a_press_jumps_a_move_carries_and_nothing_else_is_read() {
        let mut resp = Response::inert(Id::from_raw(1), Rect::new(0, 0, 300, 1));
        assert_eq!(scrub(&resp), None, "nothing held");

        resp.pressed = true;
        resp.local = Some((20, 0));
        assert_eq!(scrub(&resp), Some(20.0 / 299.0));
        resp.local = Some((60, 0));
        assert_eq!(scrub(&resp), Some(60.0 / 299.0));

        // Off either end, clamped rather than wrapped or refused.
        resp.local = Some((-4, 0));
        assert_eq!(scrub(&resp), Some(0.0));
        resp.local = Some((4_000, 0));
        assert_eq!(scrub(&resp), Some(1.0));

        // And a one-cell track answers rather than dividing by zero.
        let mut narrow = Response::inert(Id::from_raw(2), Rect::new(0, 0, 1, 1));
        narrow.pressed = true;
        narrow.local = Some((0, 0));
        assert_eq!(scrub(&narrow), Some(0.0));
    }

    /// **The stamp formats without allocating, and it is the whole reason it exists.**
    #[test]
    fn a_stamp_is_minutes_and_seconds_on_the_stack() {
        assert_eq!(Stamp::of(0.0).as_str(), "00:00");
        assert_eq!(Stamp::of(61.4).as_str(), "01:01");
        assert_eq!(Stamp::of(3_599.0).as_str(), "59:59");
        assert_eq!(Stamp::of(-5.0).as_str(), "00:00");
        assert_eq!(Stamp::of(f32::from(u16::MAX) * 4.0).as_str(), "99:59");
        assert_eq!(Stamp::of(1e12).as_str().len(), usize::from(STAMP_W));
    }

    /// **The marks are a field, and replacing the chapters re-derives them.**
    #[test]
    fn the_marks_are_derived_at_construction_and_stay_in_step() {
        let mut p = Player::new(
            100.0,
            vec!["a".to_owned()],
            vec![
                Chapter {
                    at: 0.0,
                    name: "one".to_owned(),
                },
                Chapter {
                    at: 0.5,
                    name: "two".to_owned(),
                },
            ],
        );
        assert_eq!(p.marks(), &[0.0, 0.5]);
        p.set_chapters(vec![Chapter {
            at: 0.25,
            name: "only".to_owned(),
        }]);
        assert_eq!(p.marks(), &[0.25]);
        assert_eq!(p.chapters().len(), 1);
    }

    /// **Press jumps, move carries, release moves nothing — and every one of the three is
    /// `Response::local` over `Response::rect`.**
    ///
    /// The three phases, played through a **posted** pointer over the shipped [`chrome`] rather
    /// than over a `Response` written beside the gate. `20/299 = 0.0669`, `60/299 = 0.2007`, and
    /// then the value the release leaves standing, which is the move's.
    ///
    /// # The cadence is two frames a press, and the gate is written around it
    ///
    /// `Response::pressed` is `frame.grab == id` and the grab is awarded at `end` from the index
    /// that has just drawn — so the frame that *delivers* the `Down` reads `pressed == false` and
    /// the frame after it reads `true`. A gate that played one frame a phase would measure the
    /// cadence and call it the mechanism.
    #[test]
    fn a_posted_press_jumps_a_posted_move_carries_and_the_release_moves_nothing() {
        let mut driver = Driver::headless(W, H).expect("a sink attaches");
        let mut p = player();
        let mut seen: Vec<Option<f32>> = Vec::new();
        let frame = |driver: &mut Driver, p: &mut Player, seen: &mut Vec<Option<f32>>| {
            let mut answer = None;
            driver.frame(|cx| {
                let track = chrome(cx, cx.area(), p);
                answer = scrub(&track);
            });
            if let Some(v) = answer {
                p.position = v;
            }
            seen.push(answer);
        };

        // The pointer arrives, and nothing is held.
        driver.post_mouse(at(20, MouseKind::Move));
        frame(&mut driver, &mut p, &mut seen);
        assert_eq!(seen.last().copied().flatten(), None);

        // The press. The grab is awarded at the end of the frame that delivers it.
        driver.post_mouse(at(20, MouseKind::Down(Button::Left)));
        frame(&mut driver, &mut p, &mut seen);
        frame(&mut driver, &mut p, &mut seen);
        let pressed = seen.last().copied().flatten().expect("the grab is held");
        assert!(
            (pressed - 20.0 / 299.0).abs() < 1e-6,
            "a press jumped to {pressed} rather than to 20/299"
        );
        assert!(p.scrubbing, "the chrome did not report the scrub");

        // The move carries it, without a press origin anywhere.
        driver.post_mouse(at(60, MouseKind::Move));
        frame(&mut driver, &mut p, &mut seen);
        let moved = seen.last().copied().flatten().expect("still held");
        assert!(
            (moved - 60.0 / 299.0).abs() < 1e-6,
            "a move carried to {moved} rather than to 60/299"
        );

        // The release ends the scrub and moves nothing.
        driver.post_mouse(at(60, MouseKind::Up(Button::Left)));
        frame(&mut driver, &mut p, &mut seen);
        frame(&mut driver, &mut p, &mut seen);
        assert_eq!(
            seen.last().copied().flatten(),
            None,
            "the grab outlived the release"
        );
        assert!(!p.scrubbing);
        assert!(
            (p.position - moved).abs() < f32::EPSILON,
            "the release moved the value from {moved} to {}",
            p.position
        );
    }

    /// **The chrome is a partition of its rectangle**, at the height that draws the lists and at the
    /// height that does not.
    #[test]
    fn the_chrome_writes_every_cell_of_its_rectangle_exactly_once() {
        for h in [H, H + LISTS_MIN_H, H + 12] {
            let mut driver = Driver::headless(60, h).expect("a sink attaches");
            let mut p = player();
            let mut tally = crate::counters::Tally::new();
            driver.frame(|cx| {
                chrome_into(
                    &mut crate::counters::Tally::new(),
                    cx,
                    cx.area(),
                    &mut p,
                    &mut Census::default(),
                );
            });
            driver.frame(|cx| {
                chrome_into(&mut tally, cx, cx.area(), &mut p, &mut Census::default());
            });
            let cells = u64::from(h) * 60;
            assert_eq!(tally.distinct(), cells, "at {h} rows: a cell nobody wrote");
            // **The marks and the thumb are the stated exception**, and they are counted rather than
            // waved at: a mark is written over the track run it sits on, so `writes` exceeds
            // `distinct` by exactly the chapters plus the thumb.
            let over = tally.writes() - tally.distinct();
            assert_eq!(
                over,
                p.marks().len() as u64 + 1,
                "at {h} rows: the overwrite is the marks and the thumb and nothing else"
            );
        }
    }

    /// **Two chromes at two call sites are two players**, and both halves of the fix are needed.
    ///
    /// The rule — *a container roots its children inside its own id* — on the one
    /// construction in this family that interacts, and the review caught it broken. `Ctx::id` mints
    /// from `Location::caller()` and `#[track_caller]` does not propagate into a plain private body,
    /// so the track's id was the fixed line inside `draw_chrome`: one value for every chrome ever
    /// drawn. `Ctx::with_key` then roots at whatever the enclosing stack happens to be, so the five
    /// transport buttons and the two lists collided as well.
    ///
    /// **What that costs is `Ctx::interact`'s**, and it is silent: a merged claim is *inert*, so the
    /// second player's seek bar takes no press, no drag and no hover and its buttons never click —
    /// on a screen that renders perfectly. `media::tests::two_pictures_at_two_call_sites_are_two_widgets`
    /// is the same gate over the family's pure drawers, where the only casualty is `Response::id`.
    #[test]
    fn two_chromes_at_two_call_sites_are_two_players_and_nothing_merges() {
        let mut driver = Driver::headless(60, 20).expect("a sink attaches");
        let (mut top, mut bottom) = (player(), player());
        let mut ids = (None, None);
        driver.frame(|cx| {
            let (upper, lower) = vitui_runtime::layout::rect::split_at_v(cx.area(), 10);
            ids.0 = Some(chrome(cx, upper, &mut top).id);
            ids.1 = Some(chrome(cx, lower, &mut bottom).id);
        });
        assert_ne!(ids.0, ids.1, "two call sites collided on one track");
        assert_eq!(
            driver.inspect().ids().merges(),
            0,
            "the chrome's keyed children collided, so the second player is inert on a screen that \
             renders perfectly"
        );

        // The other direction, or the inequality above is one a fresh-value id function would also
        // pass: the **same** call site drawn twice is one player, and its children merge on purpose.
        let mut twice = Vec::new();
        driver.frame(|cx| {
            let area = cx.area();
            for _ in 0..2 {
                twice.push(chrome(cx, area, &mut top).id);
            }
        });
        assert_eq!(twice[0], twice[1], "one call site minted two tracks");
    }

    /// **Zero customs: the chrome is the contrast a picture is drawn against.**
    #[test]
    fn the_chrome_spends_no_theme_custom_at_all() {
        let mut driver = Driver::headless(60, 16).expect("a sink attaches");
        let mut p = player();
        let mut census = Census::default();
        driver.frame(|cx| {
            chrome_into(&mut crate::ink::Direct, cx, cx.area(), &mut p, &mut census);
        });
        assert_eq!(census.customs, 0);
        assert!(census.roles > 0, "and it counted the roles it did spend");
    }
}
