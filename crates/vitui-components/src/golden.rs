//! **O3's screens: one golden per construction, in engine 05's format and not a second one.**
//!
//! > O3 — one golden screen **per construction**, not per matrix cell: count `goldens ==
//! > constructions`, **and** an equality: screens declared identical must be identical.
//!
//! A golden is the one instrument on this map that catches a **wrong cell**. O1 catches an API that
//! cannot be called from outside the crate and O2 catches an inventory that has drifted from what
//! ships; neither of them looks at a picture. What this module holds is the picture, rendered from
//! [`crate::runner::Canvas`] — the surface a [`crate::runner::Pen`] recorded — in the format
//! `crates/vitui-engine/src/golden.rs` owns.
//!
//! # There is no second format, and [`FORMAT_OWNER`] is how that stays true
//!
//! The engine's `golden.rs` is `pub(crate)` throughout, so this crate cannot call it: a cell, a
//! handle and a style bit are all unreadable from outside the engine, which is the same
//! barrier that makes `marked` [`crate::counters::Reading::Unreachable`]. **What cannot be shared
//! is the code; what must not be forked is the format**, so the four load-bearing properties are
//! [`FORMAT_PROPERTIES`] — read out of the owner's own header by
//! `tests/golden.rs`'s `the_format_is_the_engines_and_the_owner_is_named_by_path`, which opens
//! [`FORMAT_OWNER`] and fails when a sentence has moved. A second format cannot appear unnoticed
//! because the day one does, the sentence it was supposed to inherit is no longer in the file this
//! module points at.
//!
//! The properties, in the owner's words:
//!
//! - **Two planes, not one** — a cell carries a cluster and a style word and they change
//!   independently.
//! - **One file column per terminal column, always.**
//! - **ASCII renders as itself; everything else gets a legend character.**
//! - **The header line is part of the assertion** — a golden taken at the wrong tier is a golden
//!   that passes for the wrong reason.
//!
//! And the two the engine added after diffing the format in anger, which are inherited whole:
//! every plane row carries its own number, so a line-based diff cannot pair two identical rows the
//! wrong way round; and every legend entry carries its cell count, so a one-cell style change and a
//! whole-screen one are not the same single line.
//!
//! # Three things this side has that the engine's does not, each because the surface is different
//!
//! - **A cell nobody wrote is [`UNWRITTEN`] and it is not a blank.** The engine composites over an
//!   opaque base, so an unpainted cell is a finding; here it is the *subject* — the partition
//!   rule is exactly the claim that a component leaves none of its own rectangle untouched, and
//!   [`crate::counters::sentinel`] exists because *what was already there is almost always right*.
//!   Rendering it as a space would make a golden agree with itself about the cells the shrink axis
//!   is about.
//! - **The style plane names a [`Role`], because that is all a component ever said.** A
//!   component names a role and never a colour, and a [`vitui_runtime::Paint`] is opaque here. So
//!   the legend resolves each paint against the theme's thirteen roles and prints the name; a paint
//!   that is none of them came from `Theme::custom` or `Theme::mix` and prints as such. That is
//!   strictly *more* legible than a colour triple would be: the thing a reviewer wants to know
//!   about a component's cell is which role it asked for.
//! - **The failure report carries cells and rows**, beside the engine's first-differing-line. That
//!   is not a second format — it is the same file compared twice — and it is the form every defect
//!   on this map was legible in: *6 662 cells over 80 rows* is a whole screen and *6 662 over 9* is
//!   a band. See [`Divergence`].
//!
//! # The rung is not nameable here, so a screen is taken at a theme somebody else built
//!
//! The replacement for the type `Paint` was able to be is a count — `GlyphSet::` occurrences in
//! `vitui-components/src` == 0 — so this module cannot say which rung a screen is at. It takes a
//! [`Theme`] and reads the rung *off* it, which is what puts the rung in the header without naming
//! it. The three-rung sweep is `crates/vitui-components/tests/golden.rs`, which is a test rather
//! than a component and is named in that file's own exception list.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use vitui_runtime::layout::text;
use vitui_runtime::{Paint, Role, Theme};

use crate::runner::{Canvas, Cell, Pen};
use vitui_runtime::ctx::Driver;

/// **The file the format belongs to, by path.**
///
/// Named rather than described, because a description drifts silently and a path does not: the gate
/// opens this file and looks for [`FORMAT_PROPERTIES`] inside it.
pub const FORMAT_OWNER: &str = "crates/vitui-engine/src/golden.rs";

/// The four sentences [`FORMAT_OWNER`] states the format in, verbatim.
///
/// Fragments rather than whole sentences, each long enough to be unique and short enough to survive
/// the formatter breaking a line — the shape `crate::overlay::OWED_SENTENCE` already uses, and for
/// its reason: the scan runs over a **flattened** source, because a doc comment is not an item and
/// in the shipped file one phrase is broken across a line by `rustfmt`.
pub const FORMAT_PROPERTIES: [&str; 4] = [
    "Two planes, not one",
    "One file column per terminal column, always",
    "ASCII renders as itself; everything else gets a legend character",
    "The header line is part of the assertion",
];

/// **The one cluster a plane renders as itself**, or `None` when it needs a legend key.
///
/// One predicate for the renderer and for [`alphabet`], because two copies of a classification that
/// must agree is a gate that lies the day they stop agreeing — and `alphabet` is what decides how
/// big a `plot`'s screen may be. A wide cluster is never ASCII, so this cannot swallow one: every
/// printable ASCII scalar is one column.
fn printable_ascii(cluster: &str) -> Option<char> {
    let mut chars = cluster.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_graphic() => Some(c),
        _ => None,
    }
}

/// A blank cell — a space, U+0020.
///
/// Non-ASCII on purpose, and it is the owner's own correction to the example: a space rendered as
/// a space is trailing whitespace, which editors strip and `git diff` paints red.
pub const BLANK: char = '·';

/// The second column of a double-width pair. **Reserved**, so a width bug shifts a row.
pub const CONTINUATION: char = '▸';

/// **A cell no verb of this frame or any frame before it wrote.**
///
/// The engine's marker at this position means *no layer painted*, which cannot survive compositing;
/// here it is the partition rule as one character. See the module header.
pub const UNWRITTEN: char = '▪';

/// The glyph plane's legend characters, for every cluster that is not printable ASCII.
///
/// The owner's list, unchanged: all non-ASCII, single-column, and chosen against their ASCII
/// look-alikes, because the whole point of the plane is that a reviewer reads it as a picture.
const GLYPH_KEYS: &[char] = &[
    'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'λ', 'μ', 'ξ', 'π', 'σ', 'τ', 'φ', 'ψ', 'ω', 'Γ', 'Δ',
    'Θ', 'Λ', 'Ξ', 'Σ', 'Φ', 'Ψ', 'Ω',
];

/// The style plane's legend characters. ASCII is free here — the style plane has no
/// renders-as-itself rule.
const STYLE_KEYS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// The width of the row-number gutter every plane row carries: three digits and a space.
const GUTTER: usize = 4;

/// **The tier a golden's header records**, read off the theme rather than handed in.
///
/// Two axes and one string, the owner's `colours/glyphs`. It is derived and not a parameter for the
/// reason the header exists at all: a screen taken at the wrong rung is a screen that passes for the
/// wrong reason, and a caller that can spell the word can spell the wrong one.
///
/// The words come from `Debug` lowercased rather than from the engine's own `word()`, which is
/// `pub(crate)`. `TrueColor` is `truecolor`, `Extended` is `extended`, and the two ladders have no
/// other spelling — `tests::the_tier_reads_the_themes_own_two_axes` pins them.
pub fn tier(theme: &Theme) -> String {
    format!("{:?}/{:?}", theme.tier(), theme.glyphs()).to_lowercase()
}

/// What one golden file holds: the header, two planes, and the legend.
///
/// Rendered from a recorded surface and the theme its paints were asked of, never from a `Driver`,
/// so the format's own properties can be asserted over a four-cell canvas instead of over 24 000.
///
/// # Panics
///
/// Panics on a canvas 1 000 rows tall or more — the row gutter is three digits, and a fourth would
/// widen every plane row by one, which is *one file column per terminal column* becoming false
/// rather than offset. The owner's guard, for the owner's reason.
pub fn render(scene: &str, frame: u32, theme: &Theme, canvas: &Canvas) -> String {
    let (w, h) = (canvas.w(), canvas.h());
    let label = format!("{scene}, frame {frame}");
    assert!(
        h < 1000,
        "the row gutter is three digits wide; a frame {h} rows tall would widen it"
    );
    let mut legend = Legend::new(label, theme);
    let mut glyph = String::with_capacity((w as usize + GUTTER + 1) * h as usize);
    let mut style = String::with_capacity((w as usize + GUTTER + 1) * h as usize);

    for y in 0..h {
        let _ = write!(glyph, "{y:3} ");
        let _ = write!(style, "{y:3} ");
        for x in 0..w {
            let cell = canvas.get(x, y);
            glyph.push(legend.glyph_key(cell));
            style.push(legend.style_key(cell));
        }
        glyph.push('\n');
        style.push('\n');
    }

    let mut out = String::with_capacity(glyph.len() + style.len() + 256);
    let _ = writeln!(
        out,
        "scene: {scene}  frame: {frame}  size: {w}x{h}  tier: {}",
        tier(theme)
    );
    out.push_str("\nglyph\n");
    out.push_str(&glyph);
    out.push_str("\nstyle\n");
    out.push_str(&style);
    out.push_str("\nlegend\n");
    legend.write(&mut out);
    out
}

/// Which key stands for which paint and which cluster, in the order they were first seen.
struct Legend<'t> {
    /// The scene and frame, so a format the picture has outgrown says which picture.
    label: String,
    /// The theme the paints were asked of — the only thing that can turn one back into a role name.
    theme: &'t Theme,
    /// The paints seen, in first-appearance order, which is also the order of [`STYLE_KEYS`].
    ///
    /// **A `Vec` and a linear scan rather than a map**, because [`Paint`] is `Eq` and not `Hash` —
    /// the runtime derives what an opaque value needs and nothing more — and the only identity a
    /// paint offers is its own equality. A screen carries at most sixty-two of these by
    /// construction, since a sixty-third is refused as unreviewable.
    styles: Vec<(Paint, Option<Role>)>,
    /// How many cells carry each — **the blast radius**, without which a style word that changed
    /// under one cell and one that changed under the whole screen are the same one-line diff.
    style_cells: Vec<u32>,
    clusters: Vec<String>,
    cluster_cells: Vec<u32>,
    /// The reserved markers this frame actually used, each with its own explanation.
    markers: Vec<(char, &'static str)>,
}

impl<'t> Legend<'t> {
    fn new(label: String, theme: &'t Theme) -> Legend<'t> {
        Legend {
            label,
            theme,
            styles: Vec::new(),
            style_cells: Vec::new(),
            clusters: Vec::new(),
            cluster_cells: Vec::new(),
            markers: Vec::new(),
        }
    }

    /// The style plane's character for one cell, minting one on first sight.
    fn style_key(&mut self, cell: Option<&Cell>) -> char {
        let Some(cell) = cell else {
            return self.marker(UNWRITTEN, "a cell no verb wrote");
        };
        let key = (cell.paint, cell.hover);
        if let Some(i) = self.styles.iter().position(|s| *s == key) {
            self.style_cells[i] += 1;
            return STYLE_KEYS[i] as char;
        }
        let i = self.styles.len();
        assert!(
            i < STYLE_KEYS.len(),
            "{}: more than {} distinct paints on screen is not reviewable by eye",
            self.label,
            STYLE_KEYS.len()
        );
        self.styles.push(key);
        self.style_cells.push(1);
        STYLE_KEYS[i] as char
    }

    /// The glyph plane's character for one cell.
    ///
    /// Printable ASCII is itself; the three reserved markers are themselves; everything else takes a
    /// key from [`GLYPH_KEYS`] on first sight. A wide head takes a key and its [`CONTINUATION`]
    /// takes a column of its own, which is the whole of *one file column per terminal column*.
    fn glyph_key(&mut self, cell: Option<&Cell>) -> char {
        let Some(cell) = cell else {
            return self.marker(UNWRITTEN, "a cell no verb wrote");
        };
        if cell.cluster.is_empty() {
            return self.marker(
                CONTINUATION,
                "the second column of the wide cluster to its left",
            );
        }
        if cell.cluster == " " {
            return self.marker(BLANK, "a blank cell, U+0020");
        }
        if let Some(c) = printable_ascii(&cell.cluster) {
            return c;
        }
        if let Some(i) = self.clusters.iter().position(|c| *c == cell.cluster) {
            self.cluster_cells[i] += 1;
            return GLYPH_KEYS[i];
        }
        let i = self.clusters.len();
        assert!(
            i < GLYPH_KEYS.len(),
            "{}: more than {} distinct non-ASCII clusters on screen is not reviewable by eye",
            self.label,
            GLYPH_KEYS.len()
        );
        self.clusters.push(cell.cluster.clone());
        self.cluster_cells.push(1);
        GLYPH_KEYS[i]
    }

    /// Record that a reserved marker was used, so the legend explains only the ones that appear.
    ///
    /// Markers carry no count: their blast radius is the whole point of the plane they appear in.
    fn marker(&mut self, m: char, what: &'static str) -> char {
        if !self.markers.iter().any(|(c, _)| *c == m) {
            self.markers.push((m, what));
        }
        m
    }

    fn write(&self, out: &mut String) {
        // **The customs are numbered, and that is not decoration.** A picture's cells and a chart's
        // palette are all outside the thirteen roles, so without an ordinal a screen
        // with two of them carries two legend lines a reviewer cannot tell apart — which is the
        // one thing a legend exists to prevent. The number is first-appearance order, the same
        // order the keys are in, and it names no colour.
        let mut customs = 0u32;
        for (i, (paint, hover)) in self.styles.iter().enumerate() {
            let described = describe_paint(self.theme, *paint, *hover, &mut customs);
            let _ = writeln!(
                out,
                "  {}  {}  ({})",
                STYLE_KEYS[i] as char,
                described,
                cells(self.style_cells[i])
            );
        }
        for (m, what) in &self.markers {
            let _ = writeln!(out, "  {m}  {what}");
        }
        for (i, g) in self.clusters.iter().enumerate() {
            let _ = writeln!(
                out,
                "  {}  {}  ({})",
                GLYPH_KEYS[i],
                describe_cluster(g),
                cells(self.cluster_cells[i])
            );
        }
    }
}

/// The code points a cluster names, and how many columns it takes.
///
/// Code points rather than the cluster itself, because a reviewer needs to see a ZWJ and a variation
/// selector — the two things a rendered cluster hides and the two things a width bug turns on.
fn describe_cluster(cluster: &str) -> String {
    let mut out = String::new();
    for c in cluster.chars() {
        if !out.is_empty() {
            out.push(' ');
        }
        let _ = write!(out, "U+{:04X}", c as u32);
    }
    match text::width(cluster) {
        2 => out.push_str(" (width 2, continuation follows)"),
        n => {
            let _ = write!(out, " (width {n})");
        }
    }
    out
}

/// **How a paint reads in the legend: the role a component asked for.**
///
/// This is a search and not a field read because a component names a role and never a
/// colour, and a `Paint` is opaque here — and it is also why the answer is the *useful* one. A paint
/// that is none of the thirteen came from `Theme::custom` or `Theme::mix`, which is the stated
/// exception for a picture and one for a chart's palette, and it says so rather than guessing.
fn describe_paint(theme: &Theme, paint: Paint, hover: Option<Role>, customs: &mut u32) -> String {
    let named = match Role::ALL.iter().find(|r| theme.paint(**r) == paint) {
        Some(r) => format!("{r:?}"),
        None => {
            *customs += 1;
            format!("custom #{customs} (Theme::custom or Theme::mix, outside the thirteen roles)")
        }
    };
    match hover {
        None => named,
        Some(r) => format!("{named}, hovered as {r:?}"),
    }
}

/// `1 cell` or `n cells`, so that a legend line reads as a sentence.
fn cells(n: u32) -> String {
    if n == 1 {
        "1 cell".to_string()
    } else {
        format!("{n} cells")
    }
}

// -------------------------------------------------------------------------------------------------
// The file, and `VITUI_BLESS=1`
// -------------------------------------------------------------------------------------------------

/// The directory the screens live in, relative to this crate's manifest.
///
/// The owner's layout — one file per frame under `tests/golden/<scene>/<nn>.txt` — so that a scene
/// played over several frames is several files a reviewer reads in order.
pub const DIR: &str = "tests/golden";

fn golden_path(scene: &str, frame: u32) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DIR)
        .join(scene)
        .join(format!("{frame:02}.txt"))
}

/// **Assert that this canvas is the picture on file** — or, under `VITUI_BLESS=1`, make the file say
/// what it is.
///
/// The review is the git diff, which is why blessing rewrites in place and takes no argument beyond
/// the variable: a golden nobody can regenerate in one command will be deleted. **`VITUI_BLESS=1`
/// is the owner's command and there is no second one**; the only difference here is the crate it is
/// spelled against.
///
/// # Panics
///
/// Panics when the file is missing, when it differs, and when `VITUI_BLESS=1` is set in CI.
#[track_caller]
pub fn assert_golden(scene: &str, frame: u32, theme: &Theme, canvas: &Canvas) {
    let rendered = render(scene, frame, theme, canvas);
    let path = golden_path(scene, frame);
    let shown = path
        .strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or(&path)
        .display()
        .to_string();

    if blessing() {
        let dir = path.parent().expect("a golden path always has a directory");
        std::fs::create_dir_all(dir)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
        std::fs::write(&path, &rendered).unwrap_or_else(|e| panic!("cannot write {shown}: {e}"));
        return;
    }

    let expected =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{shown}: {e}\n{}", BLESS_LINE));
    if let Some(report) = difference(&expected, &rendered, &shown, canvas.h()) {
        panic!("{report}");
    }
}

/// The one line a failing golden ends with, so that the command a reader needs is never two files
/// away from the failure.
const BLESS_LINE: &str = "run `VITUI_BLESS=1 cargo test -p vitui-components golden` and review the \
                          diff. A stale golden is a failure, not a warning";

/// Whether this run rewrites the files instead of comparing against them.
///
/// **It refuses to do that in CI**, and that is a gate rather than a convention: a job that blessed
/// would rewrite the very file it exists to compare against and report green. The owner's rule,
/// inherited whole. Both GitLab and GitHub Actions set `CI`.
///
/// # Panics
///
/// Panics when `VITUI_BLESS=1` and `CI` are both set.
fn blessing() -> bool {
    let on = std::env::var_os("VITUI_BLESS").is_some_and(|v| v == "1");
    assert!(
        !(on && std::env::var_os("CI").is_some()),
        "VITUI_BLESS=1 in CI would rewrite the file the job exists to compare against"
    );
    on
}

/// **How far apart two goldens are, in the unit every defect on this map was legible in.**
///
/// The engine's report is *the first line that differs*, which is right for a 300x80 screen where
/// printing both copies would bury the one character that moved — and it is the whole of what a
/// reviewer gets. It cannot tell a one-cell defect from a whole-screen one, and this crate's own
/// [`crate::runner::Diff`] has always said `n cells over m rows` because *6 662 cells over
/// 80 rows is a whole screen wrong and 6 662 over 9 is a band*. So a failing golden reports both.
///
/// **A cell counts once**, however many of the two planes disagree about it: the planes are two
/// renderings of one surface, and adding them would report a cell whose cluster and paint both
/// changed as two.
///
/// # The one thing it over-reports, and it is the owner's stated trade
///
/// The legend is assigned in first-appearance order, so a change that introduces a *new* paint
/// before an existing one shifts every letter after it and the whole style plane diffs. That is the
/// owner's documented cost of first-appearance order, and the case a golden exists to catch — a
/// defect that changes cells without changing the set of paints — leaves the legend alone.
///
/// # And the one thing it is **blind** to, which is why no equality on this map rests on it
///
/// A plane encodes the *pattern* of distinct clusters and not the clusters themselves: a key is
/// `GLYPH_KEYS[i]` where `i` is first-appearance order, so **two screens whose non-ASCII clusters
/// are swapped one for one in the same order have identical planes**. A screen drawing `U+2588`
/// everywhere and one drawing `U+2592` everywhere are `(0, 0)` apart here, and that is exactly the
/// regression *a bar chart started spelling itself differently at `Extended`* — which is what the
/// equality half of O3 exists to catch.
///
/// So **`divergence` is a report and never an equality.** The cell truth is
/// [`Canvas::diff`](crate::runner::Canvas::diff), which compares clusters and paints rather than
/// keys, and every gate in `crates/vitui-components/tests/golden.rs` that asks *are these two
/// screens the same* or *how far apart are they* runs over that. What this is for is the **file**:
/// two goldens on disk with no canvas behind them, where the whole-file line comparison in
/// [`difference`] has already caught the legend and what is left to say is the blast radius.
/// `tests::divergence_is_blind_to_a_rename_and_the_canvas_is_not` is that fact watched.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Divergence {
    /// How many cells the two goldens disagree about.
    pub cells: usize,
    /// How many rows carry at least one of them.
    pub rows: usize,
}

/// One golden's two planes, gutter stripped: `h` rows of `w` characters, glyph then style.
type Planes = (Vec<Vec<char>>, Vec<Vec<char>>);

/// The two planes of one golden, gutter stripped, as `h` rows of `w` characters each.
///
/// `None` when the file is not this format at all — a truncated write, or a golden from a screen of
/// a different height — which is reported as such rather than being read as *every cell differs*.
fn planes(golden: &str, h: u16) -> Option<Planes> {
    let lines: Vec<&str> = golden.lines().collect();
    let plane = |label: &str| -> Option<Vec<Vec<char>>> {
        let at = lines.iter().position(|l| *l == label)? + 1;
        let rows: Vec<Vec<char>> = lines
            .get(at..at + usize::from(h))?
            .iter()
            .map(|l| l.chars().skip(GUTTER).collect())
            .collect();
        (rows.len() == usize::from(h)).then_some(rows)
    };
    Some((plane("glyph")?, plane("style")?))
}

/// **How far apart two rendered goldens are**, as cells and rows.
///
/// Over the planes and not over the file, because the header and the legend are one line each and a
/// reviewer counting cells does not want them in the total. Two goldens of different heights have no
/// cell-for-cell comparison to make, which is [`Canvas::diff`]'s own refusal one layer up.
///
/// # Panics
///
/// Panics when either side is not in this format at `h` rows.
pub fn divergence(a: &str, b: &str, h: u16) -> Divergence {
    let (ag, ast) = planes(a, h).expect("the left golden is this format");
    let (bg, bst) = planes(b, h).expect("the right golden is this format");
    let mut cells = 0usize;
    let mut rows = 0usize;
    for y in 0..usize::from(h) {
        let mut row_differs = false;
        // Over all four rows and not the two glyph ones: a style plane longer than both glyph
        // planes would otherwise have its trailing columns excluded from the totals, which is a
        // number that is silently wrong rather than a number that is missing.
        let width = ag[y]
            .len()
            .max(bg[y].len())
            .max(ast[y].len())
            .max(bst[y].len());
        for x in 0..width {
            let glyph = ag[y].get(x) != bg[y].get(x);
            let style = ast[y].get(x) != bst[y].get(x);
            if glyph || style {
                cells += 1;
                row_differs = true;
            }
        }
        if row_differs {
            rows += 1;
        }
    }
    Divergence { cells, rows }
}

/// **The report a failing golden gives a human** — or `None` when the two agree.
///
/// It returns the report rather than panicking so that **the report itself is testable**: it is the
/// only thing a failing golden gives anybody, and a `#[should_panic(expected = …)]` can pin one
/// contiguous substring, so the section, the column and the totals could not all be asserted at
/// once.
///
/// **Line-wise rather than byte-wise, and that is load-bearing on one of the three CI runners.**
/// `str::lines` drops a trailing `\r`, so a Windows checkout with `core.autocrlf` on compares equal
/// — and the GitHub workflow's `test` job is a three-OS matrix.
pub fn difference(expected: &str, rendered: &str, shown: &str, h: u16) -> Option<String> {
    let (e, r): (Vec<&str>, Vec<&str>) = (expected.lines().collect(), rendered.lines().collect());
    let first = (0..e.len().max(r.len())).find(|&i| e.get(i) != r.get(i))?;
    let (a, b) = (e.get(first), r.get(first));
    let at = Section::of(first, h);
    let mut msg = format!(
        "{shown}: {at} differs\n  on file:  {}\n  rendered: {}",
        a.copied().unwrap_or("<the file ends here>"),
        b.copied().unwrap_or("<the frame ends here>")
    );
    if let (Some(a), Some(b)) = (a, b)
        && let Some(c) = a.chars().zip(b.chars()).position(|(x, y)| x != y)
    {
        let _ = write!(msg, "\n  first at {}", position(at, c));
    }
    // **Cells and rows, beside the first line.** See `Divergence`. A file that is not this format
    // has no cell comparison to make and says so rather than reporting a number that means nothing.
    match (planes(expected, h), planes(rendered, h)) {
        (Some(_), Some(_)) => {
            let d = divergence(expected, rendered, h);
            let _ = write!(msg, "\n  {} cells over {} rows", d.cells, d.rows);
        }
        _ => msg.push_str("\n  the file on disk is not this format at this height"),
    }
    let _ = write!(msg, "\n{BLESS_LINE}");
    Some(msg)
}

/// Where in a line the first difference is, in whatever unit that line is measured in.
///
/// A plane's columns are **cells**, and the row-number gutter is not one of them, so the gutter is
/// subtracted. A difference *inside* the gutter says so rather than being clamped to `column 0`.
fn position(at: Section, c: usize) -> String {
    match at {
        Section::Glyph(_) | Section::Style(_) if c >= GUTTER => format!("column {}", c - GUTTER),
        Section::Glyph(_) | Section::Style(_) => {
            format!("character {c}, inside the row-number gutter")
        }
        _ => format!("character {c}"),
    }
}

/// Which part of the file a line belongs to. An enum rather than a string because the column a plane
/// reports is a cell and the column anything else reports is a character.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Section {
    Header,
    Marker,
    Glyph(usize),
    Style(usize),
    Legend,
}

impl Section {
    /// Line `i` of a golden over a canvas `h` rows tall.
    fn of(i: usize, h: u16) -> Section {
        let h = h as usize;
        let glyph = 3..3 + h;
        let style = 3 + h + 2..3 + h + 2 + h;
        match i {
            0 => Section::Header,
            _ if glyph.contains(&i) => Section::Glyph(i - glyph.start),
            _ if style.contains(&i) => Section::Style(i - style.start),
            _ if i < style.end => Section::Marker,
            _ => Section::Legend,
        }
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Section::Header => f.write_str("the header line"),
            Section::Marker => f.write_str("a section marker"),
            Section::Glyph(y) => write!(f, "the glyph plane, row {y}"),
            Section::Style(y) => write!(f, "the style plane, row {y}"),
            Section::Legend => f.write_str("the legend"),
        }
    }
}

// -------------------------------------------------------------------------------------------------
// The screens, one per construction
// -------------------------------------------------------------------------------------------------

/// **A construction, as this crate is allowed to spell it.**
///
/// The replacement for the type `Paint` was able to be is a count — `GlyphSet::` occurrences in
/// `vitui-components/src` == 0 — so the repertoire cannot be named here, and a screen table that
/// could not say which rung it was at would be a table with nine of its thirty-three screens
/// missing. Three arms and no fourth, joined to the runtime's repertoire by **one match in one
/// file**: `crates/vitui-components/tests/golden.rs`, which is a test rather than a component and
/// says so in the exception list `tests/glyph_matrix.rs` asserts.
///
/// It is not a second [`vitui_runtime::theme::Glyph`] set and it is not a rename: it carries no
/// spelling, no table and no ordering claim. It is the freeze's `constructions` column with names.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rung {
    /// Printable ASCII, and nothing else.
    Ascii,
    /// Unicode a normal text font covers.
    Unicode,
    /// Braille, block elements, emoji — whatever the font actually has. **The default**, and the
    /// rung a component gets when nobody has declared one.
    Extended,
}

/// One golden screen: which freeze row it is evidence for, at which rung, over what rectangle.
///
/// `shoot` owns its own state and drives its own frames, because three of the twenty-eight cannot
/// be written any other way: `select` and `file_picker` borrow their popup state **for the frame**
/// (the sentence about the fifth component), and `file_preview_pane`'s answer arrives on a
/// frame after the one that asked for it.
#[derive(Clone, Copy)]
pub struct Screen {
    /// The [`crate::INVENTORY`] row this screen is a construction of.
    pub id: &'static str,
    /// The directory under [`DIR`]. `<id>` where a row has one construction, `<id>-<rung>` where it
    /// has more — so the join back to the freeze is the prefix and needs no second table.
    pub scene: &'static str,
    /// Which construction.
    pub rung: Rung,
    /// The rectangle the screen is played on.
    pub size: (u16, u16),
    /// What it draws.
    ///
    /// **A screen is one picture and one file**, whatever it takes to reach it. **One** of the
    /// twenty-eight plays more than one frame — `file_preview_pane`, because the answer arrives
    /// after the question — and its golden is one file, because *a construction* is what
    /// O3 counts and a cadence is not one.
    ///
    /// **The surface is not cleared between those frames**, and that is
    /// [`crate::runner::play`]'s mechanism rather than an oversight: a stale tail is a defect *of
    /// the second frame given the first*, and a runner that started each step blank would score it
    /// clean. What it costs is stated rather than hidden — a cell the last frame stops writing
    /// keeps what an earlier one put there, so [`UNWRITTEN`] answers about the **screen** and not
    /// about the last frame. A multi-frame shot ends each of its own frames, so an award declared
    /// on one frame lands on that frame's cells.
    pub shoot: fn(&mut Pen, &mut Driver),
}

/// **The screens, one per construction of every built row of the freeze.**
///
/// Thirty-three, which is [`crate::INVENTORY`]'s construction sum over the twenty-eight built rows.
/// `spinner` is the twenty-ninth row and has no screen, for the reason
/// [`crate::obligations::o3`] gives: *a golden for a function that does not exist is not a screen
/// anybody can draw*, and asking for one would put a permanent row in the failing set that no
/// ticket on this backlog can invert.
///
/// **Small rectangles on purpose.** A golden is reviewed as a git diff, and a 300x80 screen is
/// a hundred and sixty plane lines of three hundred columns. What a construction needs to be legible
/// is the component's own drawing, which fits in twenty-four columns for most of the freeze.
pub const SCREENS: &[Screen] = &[
    Screen {
        id: "text",
        scene: "text",
        rung: Rung::Extended,
        size: (20, 2),
        shoot: shots::text,
    },
    Screen {
        id: "panel",
        scene: "panel",
        rung: Rung::Extended,
        size: (24, 5),
        shoot: shots::panel,
    },
    Screen {
        id: "chip",
        scene: "chip",
        rung: Rung::Extended,
        size: (14, 1),
        shoot: shots::chip,
    },
    Screen {
        id: "button",
        scene: "button",
        rung: Rung::Extended,
        size: (14, 1),
        shoot: shots::button,
    },
    Screen {
        id: "field",
        scene: "field",
        rung: Rung::Extended,
        size: (20, 2),
        shoot: shots::field,
    },
    Screen {
        id: "collection",
        scene: "collection",
        rung: Rung::Extended,
        size: (20, 4),
        shoot: shots::collection,
    },
    Screen {
        id: "table",
        scene: "table",
        rung: Rung::Extended,
        size: (24, 4),
        shoot: shots::table,
    },
    Screen {
        id: "tree",
        scene: "tree",
        rung: Rung::Extended,
        size: (24, 4),
        shoot: shots::tree,
    },
    Screen {
        id: "select",
        scene: "select",
        rung: Rung::Extended,
        size: (20, 1),
        shoot: shots::select,
    },
    Screen {
        id: "overlay",
        scene: "overlay",
        rung: Rung::Extended,
        size: (20, 6),
        shoot: shots::overlay,
    },
    Screen {
        id: "scroll_area",
        scene: "scroll_area",
        rung: Rung::Extended,
        size: (20, 6),
        shoot: shots::scroll_area,
    },
    Screen {
        id: "scrollbar",
        scene: "scrollbar",
        rung: Rung::Extended,
        size: (3, 6),
        shoot: shots::scrollbar,
    },
    Screen {
        id: "sticky",
        scene: "sticky",
        rung: Rung::Extended,
        size: (20, 2),
        shoot: shots::sticky,
    },
    Screen {
        id: "collapsible",
        scene: "collapsible",
        rung: Rung::Extended,
        size: (24, 4),
        shoot: shots::collapsible,
    },
    Screen {
        id: "chart",
        scene: "chart-ascii",
        rung: Rung::Ascii,
        size: (24, 6),
        shoot: shots::chart,
    },
    Screen {
        id: "chart",
        scene: "chart-unicode",
        rung: Rung::Unicode,
        size: (24, 6),
        shoot: shots::chart,
    },
    Screen {
        id: "plot",
        scene: "plot-ascii",
        rung: Rung::Ascii,
        size: (12, 4),
        shoot: shots::plot,
    },
    Screen {
        id: "plot",
        scene: "plot-unicode",
        rung: Rung::Unicode,
        size: (12, 4),
        shoot: shots::plot,
    },
    Screen {
        id: "plot",
        scene: "plot-extended",
        rung: Rung::Extended,
        size: (12, 4),
        shoot: shots::plot,
    },
    Screen {
        id: "checkbox",
        scene: "checkbox",
        rung: Rung::Extended,
        size: (16, 1),
        shoot: shots::checkbox,
    },
    Screen {
        id: "radio",
        scene: "radio",
        rung: Rung::Extended,
        size: (16, 1),
        shoot: shots::radio,
    },
    Screen {
        id: "switch",
        scene: "switch",
        rung: Rung::Extended,
        size: (16, 1),
        shoot: shots::switch,
    },
    Screen {
        id: "meter",
        scene: "meter-ascii",
        rung: Rung::Ascii,
        size: (16, 1),
        shoot: shots::meter,
    },
    Screen {
        id: "meter",
        scene: "meter-unicode",
        rung: Rung::Unicode,
        size: (16, 1),
        shoot: shots::meter,
    },
    Screen {
        id: "sparkline",
        scene: "sparkline-ascii",
        rung: Rung::Ascii,
        size: (20, 2),
        shoot: shots::sparkline,
    },
    Screen {
        id: "sparkline",
        scene: "sparkline-unicode",
        rung: Rung::Unicode,
        size: (20, 2),
        shoot: shots::sparkline,
    },
    Screen {
        id: "rule",
        scene: "rule",
        rung: Rung::Extended,
        size: (20, 1),
        shoot: shots::rule,
    },
    Screen {
        id: "status_bar",
        scene: "status_bar",
        rung: Rung::Extended,
        size: (24, 1),
        shoot: shots::status_bar,
    },
    Screen {
        id: "pagination",
        scene: "pagination",
        rung: Rung::Extended,
        size: (24, 1),
        shoot: shots::pagination,
    },
    Screen {
        id: "form",
        scene: "form",
        rung: Rung::Extended,
        size: (24, 3),
        shoot: shots::form,
    },
    Screen {
        id: "slider",
        scene: "slider",
        rung: Rung::Extended,
        size: (20, 1),
        shoot: shots::slider,
    },
    // **Three, and it is the only row of the twenty-nine whose every rung is a different screen.**
    // `plot`'s three are three rasters; `spinner`'s are three ladders, and the middle one is
    // The correction to the prototype — braille is `Extended` by the engine's own
    // `GlyphSet`, so the `Unicode` rung is the quadrant blocks.
    Screen {
        id: "spinner",
        scene: "spinner-ascii",
        rung: Rung::Ascii,
        size: (20, 1),
        shoot: shots::spinner,
    },
    Screen {
        id: "spinner",
        scene: "spinner-unicode",
        rung: Rung::Unicode,
        size: (20, 1),
        shoot: shots::spinner,
    },
    Screen {
        id: "spinner",
        scene: "spinner-extended",
        rung: Rung::Extended,
        size: (20, 1),
        shoot: shots::spinner,
    },
    Screen {
        id: "file_picker",
        scene: "file_picker",
        rung: Rung::Extended,
        size: (40, 1),
        shoot: shots::file_picker,
    },
    Screen {
        id: "file_preview_pane",
        scene: "file_preview_pane",
        rung: Rung::Extended,
        size: (24, 5),
        shoot: shots::file_preview_pane,
    },
];

/// **How many legend keys a screen's glyph plane needs** — its alphabet.
///
/// The format has twenty-six, and a twenty-seventh is refused: *more than that on one screen is not
/// reviewable by eye* is the owner's rule and the reason the plane is a picture rather than a table
/// of handles. Everything that is printable ASCII, blank, a continuation or an unwritten cell costs
/// no key, which is why most of this crate's screens need one or two.
///
/// **One construction on this map can outgrow it, and that is a fact about braille.** A `plot` at
/// `Extended` draws 256 states a cell, so its alphabet grows with its rectangle until it saturates:
/// 41 keys at 24x6 and 20 at 12x4. Both numbers are pinned by
/// `crates/vitui-components/tests/golden.rs`, which is why the plot's screen is the size it is.
pub fn alphabet(canvas: &Canvas) -> usize {
    let mut seen: Vec<&str> = Vec::new();
    for y in 0..canvas.h() {
        for x in 0..canvas.w() {
            let Some(cell) = canvas.get(x, y) else {
                continue;
            };
            if cell.cluster.is_empty()
                || cell.cluster == " "
                || printable_ascii(&cell.cluster).is_some()
            {
                continue;
            }
            if !seen.contains(&cell.cluster.as_str()) {
                seen.push(&cell.cluster);
            }
        }
    }
    seen.len()
}

/// How many keys the glyph plane has before it refuses. The owner's list length, named so that a
/// gate can compare against it rather than against a literal.
pub const GLYPH_KEY_COUNT: usize = GLYPH_KEYS.len();

/// **Play one screen and hand back what it drew**, with the theme it was drawn at.
///
/// `at_rung` is how the repertoire gets in without being named here: it is handed the theme the
/// headless driver built — this crate's palette at `Density::default()`, which is
/// [`crate::runner::driver_at`]'s own answer — and returns the theme at the screen's own
/// [`Screen::rung`]. The one caller that can write that match is
/// `crates/vitui-components/tests/golden.rs`.
///
/// The density is not a parameter and that is deliberate. Density changes rectangles, so
/// a golden taken at another one is a golden of another screen — and the header line carries the
/// tier and the rung but has no field for it, because there is no second format to add one to.
/// `tests/golden.rs`'s `every_screen_is_taken_at_the_default_density` is what says so out loud.
pub fn shot(screen: &Screen, at_rung: impl FnOnce(Theme) -> Theme) -> (Theme, Canvas) {
    let (w, h) = screen.size;
    let mut driver = crate::runner::driver_at(w, h, vitui_runtime::Density::default());
    let theme = at_rung(*driver.env().theme());
    driver.set_theme(theme);
    let mut pen = Pen::new(w, h);
    (screen.shoot)(&mut pen, &mut driver);
    pen.end_frame();
    (theme, pen.into_canvas())
}

/// [`shot`], compared against the file — or blessed into it under `VITUI_BLESS=1`.
///
/// # Panics
///
/// [`assert_golden`]'s three.
#[track_caller]
pub fn assert_screen(screen: &Screen, at_rung: impl FnOnce(Theme) -> Theme) {
    let (theme, canvas) = shot(screen, at_rung);
    assert_golden(screen.scene, 0, &theme, &canvas);
}

/// **How many screens each freeze row has**, derived from [`SCREENS`].
///
/// [`crate::obligations::GOLDENS`] is the written-out version of this, and
/// `tests/golden.rs`'s `the_three_sources_agree_about_how_many_screens_there_are` compares the two — the arrangement
/// [`crate::obligations::DOC_TESTED`] and `crate::doc::survey` already use, and for its reason: a
/// `const fn` over the freeze would make the population and the evidence one expression, and an
/// equality between two things derived from each other holds.
pub fn counted() -> Vec<(&'static str, u8)> {
    let mut out: Vec<(&'static str, u8)> = Vec::new();
    for s in SCREENS {
        match out.iter_mut().find(|(id, _)| *id == s.id) {
            Some((_, n)) => *n += 1,
            None => out.push((s.id, 1)),
        }
    }
    out
}

/// **The scenes that are on disk**, as `(scene, files)`, read from [`DIR`].
///
/// The third source. [`SCREENS`] is a claim about what is drawn and
/// [`crate::obligations::GOLDENS`] is a claim about what that adds up to; neither of them opens a
/// file, so neither would notice a screen whose golden was deleted. `tests/golden.rs`'s
/// `the_three_sources_agree_about_how_many_screens_there_are` is the join, in both directions — a directory with no screen behind it fails as loudly as a
/// screen with no directory.
pub fn on_disk() -> Vec<(String, usize)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, usize)> = Vec::new();
    for entry in entries {
        let path = entry.expect("a readable entry").path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a scene directory has a name")
            .to_string();
        let files = std::fs::read_dir(&path)
            .expect("a readable scene directory")
            .filter(|e| {
                e.as_ref()
                    .is_ok_and(|e| e.path().extension().is_some_and(|x| x == "txt"))
            })
            .count();
        out.push((name, files));
    }
    out.sort();
    out
}

/// **The twenty-eight painters [`SCREENS`] points at**, one per built row of the freeze.
///
/// Twenty-eight painters and thirty-three screens: `chart`, `meter` and `sparkline` are each played
/// at two rungs and `plot` at three, which is the whole of what `constructions` counts.
///
/// Every one of them draws through [`Pen`] and therefore through a component's `_into` spelling,
/// because that is the only way a cell this crate wrote can be read back: nothing above the engine
/// can look at a composited frame, and `crate::composed`'s own gate is that the three
/// spellings route into one body — so a screen drawn through the ink is a screen of the shipped
/// drawing rather than of a copy.
///
/// **`sticky` is the one that draws nothing of its own**, and its screen is the caller's two rows
/// clipped by the band — which is the construction, since the sticky is *one band construction*
/// and the band is all of it.
///
/// **Private, and the reason is a collision rather than tidiness.** Every one of these is named
/// after a component, and this crate already carries three homonyms that are not components — a
/// `pub fn text(` in `document.rs`, a `pub fn chip(` in `state.rs` and a `pub fn table(` in
/// `gates.rs`, which is why `crate::doc` joins on [`crate::inventory::Component::module`] rather
/// than on the name. Twenty-eight more on the public surface would be twenty-eight more chances for
/// a name-based scan to find the wrong one. Nothing outside needs them: [`SCREENS`] carries the
/// pointers and [`shot`] is what plays one.
mod shots {
    use super::*;

    use vitui_runtime::layout::Constraint;
    use vitui_runtime::work::{Cancel, Task, Worker};
    use vitui_runtime::{Rect, Role};

    use crate::chart::raster::PlotState;
    use crate::chart::{Opts as PlotOpts, Series, chart_into, plot_into};
    use crate::collect::{
        CollOpts, CollState, Column, PageOpts, TableOpts, TableState, TreeOpts, TreeState,
        collection_into, pagination_into, table_into, tree_into,
    };
    use crate::disclose::{Collapse, DiscloseOpts, collapsible_into};
    use crate::edit::Text;
    use crate::files::{
        Entry, PaneOpts, PaneState, PickerBody, PickerOpts, PickerState, Preview, asking,
        file_picker_into, file_preview_pane_into,
    };
    use crate::frame::face_paint;
    use crate::gallery::SPIN_PER;
    use crate::indicate::{
        MeterOpts, SparkOpts, SpinOpts, SpinState, meter_into, sparkline_into, spinner_into,
    };
    use crate::ink::Ink;
    use crate::input::{
        ButtonOpts, FieldOpts, FormOpts, FormState, SelectOpts, SelectState, SliderOpts, Toggle,
        ToggleOpts, button_into, field_into, form_into, select_into, slider_into, toggle_into,
    };
    use crate::order::{Entry as Node, Order, Rows};
    use crate::overlay::{PopupState, ShellOpts, overlay_into};
    use crate::scroll::{
        AreaOpts, AreaState, ScrollbarOpts, Shares, Span, scroll_area_into, scrollbar_into,
        sticky as sticky_band,
    };
    use crate::structure::{
        PanelOpts, RuleOpts, StatusOpts, panel_into, rule_into, status_bar_into,
    };
    use crate::text::{ChipOpts, TextOpts, chip_into, text_into};

    /// The one preview a screen needs: an identity and an extent, and nothing else.
    struct Doc(u64);

    impl Preview for Doc {
        fn shows(&self) -> u64 {
            self.0
        }
        fn extent(&self) -> (u32, u32) {
            (8, 3)
        }
    }

    /// A decode is a free function over an identity — never a closure.
    fn decode(id: u64, _cancel: &Cancel) -> Doc {
        Doc(id)
    }

    fn line(cx: &mut vitui_runtime::Ctx<'_, '_>, row: Rect, _doc: &Doc, i: u32) {
        let paint = cx.theme().paint(Role::Body);
        let _ = cx.text(row.x, row.y, &format!("row {i:02}"), paint);
    }

    /// Every screen's data, so that two rows of the freeze drawn from the same shape differ because
    /// the component differs and not because the fixture does.
    const NAMES: [&str; 4] = ["alpha", "beta", "gamma", "delta"];

    pub fn text(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = TextOpts::default();
            let _ = text_into(
                pen,
                cx,
                Rect::new(0, 0, 20, 2),
                "the quick brown fox",
                &opts,
            );
        });
    }

    pub fn panel(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = PanelOpts::default();
            let _ = panel_into(pen, cx, Rect::new(0, 0, 24, 5), "General", &opts);
        });
    }

    pub fn chip(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = ChipOpts::default();
            let _ = chip_into(pen, cx, Rect::new(0, 0, 14, 1), "draft", &opts);
        });
    }

    pub fn button(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = ButtonOpts::default();
            let _ = button_into(pen, cx, Rect::new(0, 0, 14, 1), "Save", &opts);
        });
    }

    pub fn field(pen: &mut Pen, driver: &mut Driver) {
        let mut st = Text::input();
        st.insert(20, "hello");
        driver.frame(|cx| {
            let opts = FieldOpts::default();
            let _ = field_into(pen, cx, Rect::new(0, 0, 20, 2), &mut st, &opts);
        });
    }

    pub fn collection(pen: &mut Pen, driver: &mut Driver) {
        let mut st = CollState::new();
        driver.frame(|cx| {
            let opts = CollOpts::default();
            let _ = collection_into(
                pen,
                cx,
                Rect::new(0, 0, 20, 4),
                &mut st,
                &opts,
                Rows::of(NAMES.len()),
                |_buf, _range| None,
                |ink, cx, r, i, face| {
                    let paint = face_paint(cx.theme(), face);
                    ink.pad_to(cx, r.x, r.y, NAMES[i], r.w, paint);
                },
            );
        });
    }

    pub fn table(pen: &mut Pen, driver: &mut Driver) {
        let mut st = TableState::new();
        let cols = [
            Column::new(0, "id", Constraint::Fixed(4)),
            Column::new(1, "name", Constraint::Weight(1)),
        ];
        driver.frame(|cx| {
            let opts = TableOpts::default();
            let _ = table_into(
                pen,
                cx,
                Rect::new(0, 0, 24, 4),
                &mut st,
                &opts,
                &cols,
                Rows::of(NAMES.len()),
                |_buf, _range| None,
                |ink, cx, r, c, face| {
                    let paint = face_paint(cx.theme(), face);
                    let text = if c.key == 0 {
                        NAMES[c.row].get(..2).unwrap_or("--").to_string()
                    } else {
                        NAMES[c.row].to_string()
                    };
                    ink.pad_to(cx, r.x, r.y, &text, r.w, paint);
                },
            );
        });
    }

    pub fn tree(pen: &mut Pen, driver: &mut Driver) {
        let index = Order::built(vec![
            Node::of(0),
            Node::of(1).at_depth(1),
            Node::of(2).at_depth(1),
            Node::of(3),
        ]);
        let mut st = TreeState::new();
        driver.frame(|cx| {
            let opts = TreeOpts::default();
            let _ = tree_into(
                pen,
                cx,
                Rect::new(0, 0, 24, 4),
                &mut st,
                &opts,
                &index,
                |_buf, _range| None,
                |ink, cx, r, n, face| {
                    let paint = face_paint(cx.theme(), face);
                    ink.pad_to(cx, r.x, r.y, NAMES[n.node as usize], r.w, paint);
                },
            );
        });
    }

    /// **Shut, one row, and the popup is not in the picture** — which is a fact about the overlay
    /// family rather than a choice about this screen.
    ///
    /// `select_into` places its popup with `Ctx::overlay`, whose body is `move |cx| …` and takes no
    /// ink: a body outlives the base pass, so everything it touches is borrowed for the frame (spec
    /// the sentence about the fifth component, and components 26's `'f`), and a `&mut I` is not.
    /// So a popup's interior cannot reach a [`Pen`] at all, and a golden of an **open** `select` is
    /// four rows of *a cell no verb wrote* under one row of face — a picture that would pass, and
    /// would say nothing about the popup.
    ///
    /// What the shut face *is* evidence for is `select`'s own gate: a partition of its rectangle,
    /// with the chevron, the space and the elided label meeting exactly once. The interiors are
    /// measured by `crate::popup` and by `overlay`'s own screen, which draws through
    /// `overlay_into` — whose body **does** take the ink, because it is not a layer.
    pub fn select(pen: &mut Pen, driver: &mut Driver) {
        static OPTIONS: [&str; 3] = ["name", "date modified", "size"];
        let mut st = SelectState::at(1);
        let mut popup = PopupState::new();
        driver.frame(|cx| {
            let id = cx.id();
            let opts = SelectOpts::default();
            let _ = select_into(
                pen,
                cx,
                id,
                Rect::new(0, 0, 20, 1),
                &mut st,
                &mut popup,
                &OPTIONS,
                &opts,
            );
        });
    }

    pub fn overlay(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let id = cx.id();
            let opts = ShellOpts::default();
            let _ = overlay_into(
                pen,
                cx,
                id,
                Rect::new(0, 0, 20, 6),
                12,
                &opts,
                |ink: &mut Pen, cx: &mut vitui_runtime::Ctx<'_, '_>, r: Rect| {
                    let paint = cx.theme().paint(Role::Body);
                    for y in 0..r.h {
                        ink.pad_to(
                            cx,
                            r.x,
                            r.y + i32::from(y),
                            NAMES[usize::from(y) % 4],
                            r.w,
                            paint,
                        );
                    }
                },
            );
        });
    }

    pub fn scroll_area(pen: &mut Pen, driver: &mut Driver) {
        let mut st = AreaState { offset: (0, 2) };
        driver.frame(|cx| {
            let id = cx.id();
            let opts = AreaOpts::default();
            let _ = scroll_area_into(
                pen,
                cx,
                id,
                Rect::new(0, 0, 20, 6),
                &mut st,
                &opts,
                (20, 40),
                |_ink: &mut Pen, _cx: &mut vitui_runtime::Ctx<'_, '_>, _band| {},
                |ink: &mut Pen, cx: &mut vitui_runtime::Ctx<'_, '_>| {
                    let paint = cx.theme().paint(Role::Body);
                    let rows = cx.visible_rows();
                    for y in rows {
                        ink.pad_to(cx, 0, y, NAMES[(y as usize) % 4], 19, paint);
                    }
                },
            );
        });
    }

    pub fn scrollbar(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let id = cx.id();
            let opts = ScrollbarOpts::default();
            let span = Span {
                viewport: 6,
                extent: 24,
                offset: 6,
            };
            let _ = scrollbar_into(pen, cx, id, Rect::new(0, 0, 3, 6), span, &opts);
        });
    }

    pub fn sticky(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let paint = cx.theme().paint(Role::Title);
            let body = cx.theme().paint(Role::Body);
            // The band shares `x` with a body scrolled four columns right: the header's first
            // visible column is content column 4, and the clip is what keeps it inside the band.
            let _ = sticky_band(cx, Rect::new(0, 0, 20, 1), Shares::X, (4, 0), |cx| {
                pen.pad_to(cx, 4, 0, "name          size", 20, paint);
            });
            pen.pad_to(cx, 0, 1, "alpha", 20, body);
        });
    }

    pub fn collapsible(pen: &mut Pen, driver: &mut Driver) {
        let mut st = Collapse::open_at(3);
        driver.frame(|cx| {
            let opts = DiscloseOpts::default();
            let _ = collapsible_into(
                pen,
                cx,
                Rect::new(0, 0, 24, 4),
                &mut st,
                "General",
                &opts,
                |_w| 3,
                |ink: &mut Pen, cx: &mut vitui_runtime::Ctx<'_, '_>| {
                    let paint = cx.theme().paint(Role::Body);
                    for y in 0..3u16 {
                        ink.pad_to(cx, 0, i32::from(y), NAMES[usize::from(y)], 24, paint);
                    }
                },
            );
        });
    }

    pub fn chart(pen: &mut Pen, driver: &mut Driver) {
        let data = Series::build(1_000, 2);
        let mut st = PlotState::new();
        driver.frame(|cx| {
            let opts = PlotOpts::chart();
            let _ = chart_into(pen, cx, Rect::new(0, 0, 24, 6), &data, &mut st, &opts);
        });
    }

    /// **Twelve columns by four, and the size is a measurement rather than a taste.**
    ///
    /// A plot at `Extended` is braille, which is *256 states per cell* (the runtime's own sentence
    /// about why the three rungs are three rungs), and the format's legend has twenty-six keys —
    /// because more than that on one screen *is not reviewable by eye*, which is the owner's rule
    /// and not a limit to route around. It is the one construction on this map whose alphabet can
    /// outgrow the format: the same plot at 24x6 needs **41** keys. See
    /// `crate::golden::alphabet` and the gate that pins both numbers.
    pub fn plot(pen: &mut Pen, driver: &mut Driver) {
        let data = Series::build(1_000, 2);
        let mut st = PlotState::new();
        driver.frame(|cx| {
            let opts = PlotOpts::plot();
            // `cx.area()` and not a literal, so that the gate which measures the alphabet at 24x6
            // measures **this** plot at 24x6 rather than a 12x4 one on a wider surface.
            let area = cx.area();
            let _ = plot_into(pen, cx, area, &data, &mut st, &opts);
        });
    }

    pub fn checkbox(pen: &mut Pen, driver: &mut Driver) {
        toggle(pen, driver, Toggle::Check);
    }

    pub fn radio(pen: &mut Pen, driver: &mut Driver) {
        toggle(pen, driver, Toggle::Radio);
    }

    pub fn switch(pen: &mut Pen, driver: &mut Driver) {
        toggle(pen, driver, Toggle::Switch);
    }

    /// The three toggles are **one machine and three configurations**, so the
    /// three screens are one call with one field changed — and what a reviewer compares is three
    /// files that differ where the machine differs.
    fn toggle(pen: &mut Pen, driver: &mut Driver, kind: Toggle) {
        let mut on = true;
        driver.frame(|cx| {
            let opts = ToggleOpts {
                kind,
                ..ToggleOpts::default()
            };
            let _ = toggle_into(pen, cx, Rect::new(0, 0, 16, 1), "enabled", &mut on, &opts);
        });
    }

    pub fn meter(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = MeterOpts::default();
            let _ = meter_into(pen, cx, Rect::new(0, 0, 16, 1), 0.625, &opts);
        });
    }

    pub fn sparkline(pen: &mut Pen, driver: &mut Driver) {
        let data = Series::build(1_000, 1);
        let mut st = PlotState::new();
        driver.frame(|cx| {
            let opts = SparkOpts::default();
            let _ = sparkline_into(pen, cx, Rect::new(0, 0, 20, 2), &data, &mut st, &opts);
        });
    }

    pub fn rule(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = RuleOpts::default();
            let _ = rule_into(pen, cx, Rect::new(0, 0, 20, 1), " limits ", &opts);
        });
    }

    pub fn status_bar(pen: &mut Pen, driver: &mut Driver) {
        driver.frame(|cx| {
            let opts = StatusOpts::default();
            let _ = status_bar_into(
                pen,
                cx,
                Rect::new(0, 0, 24, 1),
                &["ready", "3 of 9"],
                (0, 0),
                &opts,
            );
        });
    }

    pub fn pagination(pen: &mut Pen, driver: &mut Driver) {
        let mut st = CollState::new();
        driver.frame(|cx| {
            let opts = PageOpts::default();
            let _ = pagination_into(pen, cx, Rect::new(0, 0, 24, 1), &mut st, 9, &opts);
        });
    }

    pub fn form(pen: &mut Pen, driver: &mut Driver) {
        let labels = ["name", "email", "role"];
        let mut texts = [Text::input(), Text::input(), Text::input()];
        let mut st = FormState::new();
        driver.frame(|cx| {
            let opts = FormOpts::default();
            let _ = form_into(
                pen,
                cx,
                Rect::new(0, 0, 24, 3),
                &mut st,
                &labels,
                &mut texts,
                &opts,
            );
        });
    }

    /// **The clock is pinned, or the ladder frame is whatever the run happened to land on.**
    ///
    /// A spinner's index is a function of `now`, so a golden of one is a golden of a *moment* — and
    /// the only screen in this table whose picture would otherwise change between two runs of the
    /// same binary. `Driver::pin_clock` is what makes it a picture; the anchor is the pinned instant
    /// and the frame is drawn three steps later, so the shot is `LADDERS[rung][3 % len]` at every
    /// rung and the three files differ because the ladders do.
    pub fn spinner(pen: &mut Pen, driver: &mut Driver) {
        let mut st = SpinState::new();
        let anchor = driver.env().now();
        driver.pin_clock(anchor);
        st.start(anchor, SPIN_PER);
        driver.advance(SPIN_PER * 3);
        driver.frame(|cx| {
            let opts = SpinOpts::default();
            let _ = spinner_into(pen, cx, Rect::new(0, 0, 20, 1), &st, "working", &opts);
        });
    }

    pub fn slider(pen: &mut Pen, driver: &mut Driver) {
        let mut value = 0.4f32;
        driver.frame(|cx| {
            let opts = SliderOpts::default();
            let _ = slider_into(pen, cx, Rect::new(0, 0, 20, 1), &mut value, &opts);
        });
    }

    /// Shut, one row — [`shots::select`]'s reason, one family over: a picker's list is an overlay
    /// and an overlay body takes no ink.
    pub fn file_picker(pen: &mut Pen, driver: &mut Driver) {
        let files: Vec<Entry<'_>> = NAMES
            .iter()
            .enumerate()
            .map(|(i, n)| Entry {
                id: i as u64,
                name: n,
            })
            .collect();
        let worker = Worker::queueing();
        let task: Task<Doc> = Task::new(&worker);
        let mut body: PickerBody<Doc> = PickerBody::new();
        let mut st = PickerState::new();
        // **Something chosen**, so the shut face is the label and its pad rather than thirty-nine
        // blanks. A picker with nothing chosen is a legitimate state and a poor picture: it agrees
        // with a picker that has forgotten how to draw one.
        st.choose(1);
        let opts = PickerOpts::default();
        driver.frame(|cx| {
            let id = cx.id();
            let _ = file_picker_into(
                pen,
                cx,
                id,
                Rect::new(0, 0, 40, 1),
                &mut st,
                &mut body,
                &files,
                &task,
                decode,
                line,
                &opts,
            );
        });
    }

    /// **Two frames, because the answer arrives after the question**: frame one asks,
    /// the worker answers, frame two lands it and draws it. A screen that played one frame would be
    /// a golden of a pane with nothing in it, which is a picture but not the construction.
    ///
    /// **It ends each frame itself.** `shot` calls [`Pen::end_frame`] once, after the shot returns;
    /// a shot that plays two frames and left it at that would apply frame one's hover awards to
    /// frame two's cells. `end_frame` takes the queue, so the outer call is a no-op after this.
    pub fn file_preview_pane(pen: &mut Pen, driver: &mut Driver) {
        let worker = Worker::queueing();
        let task: Task<Doc> = Task::new(&worker);
        let mut pane = PaneState::new();
        for frame in 0..2 {
            if frame == 1 {
                worker.run(0);
            }
            pane.land(&task);
            pen.end_frame();
            driver.frame(|cx| {
                let id = cx.id();
                let opts = PaneOpts::default();
                let _ = file_preview_pane_into(
                    pen,
                    cx,
                    id,
                    Rect::new(0, 0, 24, 5),
                    &mut pane,
                    &task,
                    asking(7, |_cancel| Doc(7)),
                    &opts,
                    |ink: &mut Pen,
                     cx: &mut vitui_runtime::Ctx<'_, '_>,
                     row: Rect,
                     _doc: &Doc,
                     i: u32| {
                        let paint = cx.theme().paint(Role::Body);
                        ink.pad_to(cx, row.x, row.y, &format!("row {i:02}"), row.w, paint);
                    },
                );
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use vitui_runtime::Density;
    use vitui_runtime::theme::Role;

    /// A four-cell canvas, so the format's own properties are asserted over four cells rather than
    /// over 24 000. The owner's arrangement, for the owner's reason.
    fn four_cells() -> (Theme, Canvas) {
        let screen = SCREENS
            .iter()
            .find(|s| s.scene == "chip")
            .expect("a one-row screen");
        shot(screen, |t| t)
    }

    /// **The header's tier is the theme's own two axes, lowercased**, and not a word a caller chose.
    ///
    /// The engine's `word()` is `pub(crate)`, so the spelling comes from `Debug` — and the two
    /// ladders have no other spelling, which is what this pins. A golden taken at the wrong tier is
    /// a golden that passes for the wrong reason (the format's fourth property), so the string has
    /// to be derived and it has to match the owner's.
    #[test]
    fn the_tier_reads_the_themes_own_two_axes() {
        let mut driver = crate::runner::driver_at(4, 1, Density::default());
        let theme = *driver.env().theme();
        assert_eq!(tier(&theme), "truecolor/extended");

        // The whole ladder, so that a fourth arm on either axis cannot arrive spelled `Some(..)`.
        // The tier this crate can hold is one — `ColorDepth` is `reachable_as: None` in
        // `crates/vitui-runtime/src/line.rs`, register row 45 — so the sweep is the rung's.
        let mut seen = Vec::new();
        driver.frame(|cx| seen.push(tier(cx.theme())));
        assert_eq!(seen, vec!["truecolor/extended".to_string()]);
    }

    /// **The failure report is the only thing a failing golden gives a human**, so it is returned
    /// rather than panicked and asserted here.
    ///
    /// Three things at once, which is why it could not be a `#[should_panic]`: a
    /// `should_panic(expected = …)` pins one contiguous substring, so the section, the column and
    /// the totals could not all be checked — and the column arithmetic was unasserted in the owner
    /// until it was split out for exactly this.
    #[test]
    fn a_failing_golden_reports_the_section_the_column_and_the_totals() {
        let (theme, canvas) = four_cells();
        let good = render("chip", 0, &theme, &canvas);
        assert!(
            difference(&good, &good, "x/00.txt", canvas.h()).is_none(),
            "a golden differs from itself"
        );

        // One cell of the glyph plane, changed where nothing else is: the case a golden exists to
        // catch, and the case that leaves the legend alone.
        let mut lines: Vec<String> = good.lines().map(str::to_string).collect();
        let row = &mut lines[3];
        let mut chars: Vec<char> = row.chars().collect();
        chars[GUTTER + 1] = '!';
        *row = chars.into_iter().collect();
        let broken = lines.join("\n");

        let report = difference(&broken, &good, "x/00.txt", canvas.h()).expect("it differs");
        assert!(report.contains("the glyph plane, row 0"), "{report}");
        assert!(report.contains("first at column 1"), "{report}");
        assert!(report.contains("1 cells over 1 rows"), "{report}");
        assert!(report.contains("VITUI_BLESS=1"), "{report}");

        // A difference inside the gutter says so rather than being clamped to column 0: that
        // happens when the two files disagree about the frame's height, which is not a defect at
        // cell zero.
        assert_eq!(
            position(Section::Glyph(0), 1),
            "character 1, inside the row-number gutter"
        );
        assert_eq!(position(Section::Header, 9), "character 9");

        // And a file that is not this format at this height is said to be that, rather than
        // reported as a number that means nothing.
        let truncated = good.lines().take(2).collect::<Vec<_>>().join("\n");
        let report = difference(&truncated, &good, "x/00.txt", canvas.h()).expect("it differs");
        assert!(
            report.contains("not this format at this height"),
            "{report}"
        );
    }

    /// **`divergence` is blind to a one-for-one rename and `Canvas::diff` is not**, which is why no
    /// equality on this map rests on the first.
    ///
    /// A plane's key is `GLYPH_KEYS[i]` in first-appearance order, so it encodes the *pattern* of
    /// distinct clusters and not the clusters. Two screens drawing different characters in the same
    /// arrangement therefore have **identical planes** — and *the day a bar chart started spelling
    /// itself differently at `Extended`* is exactly that shape. The whole-file comparison in
    /// [`difference`] does catch it, through the legend; a count over the planes cannot.
    #[test]
    fn divergence_is_blind_to_a_rename_and_the_canvas_is_not() {
        let one = one_cluster("\u{2588}");
        let other = one_cluster("\u{2592}");
        let theme = *crate::runner::driver_at(4, 1, Density::default())
            .env()
            .theme();

        let a = render("rename", 0, &theme, &one);
        let b = render("rename", 0, &theme, &other);
        assert_eq!(
            divergence(&a, &b, 1),
            Divergence { cells: 0, rows: 0 },
            "the planes are the pattern, so a rename moves nothing in them"
        );

        // The cell truth, and the legend, both see it.
        let d = one.diff(&other);
        assert_eq!((d.cells, d.rows), (4, 1));
        assert!(
            difference(&a, &b, "x/00.txt", 1).is_some(),
            "the legend differs"
        );
    }

    /// Four cells of one cluster, drawn through the shipped verb.
    fn one_cluster(cluster: &str) -> Canvas {
        let mut driver = crate::runner::driver_at(4, 1, Density::default());
        let mut pen = Pen::new(4, 1);
        driver.frame(|cx| {
            let paint = cx.theme().paint(Role::Body);
            for x in 0..4 {
                pen.set(cx, x, 0, cluster, paint);
            }
        });
        pen.into_canvas()
    }

    /// **The thirteen role paints are pairwise distinct at the palette the screens are taken at**,
    /// or the legend names a role the component never asked for.
    ///
    /// [`describe_paint`] resolves a paint by searching `Role::ALL` and taking the first match, and
    /// the legend is a reviewer's only handle on the style plane. Two roles sharing a paint is
    /// reachable — the runtime's `Roles::pick` exists because a stub palette once had `Dim` and
    /// `Border` both on `indexed(8)`, *two role names for one colour, which nine tickets drew
    /// through without noticing* — so this is the assertion that the ambiguity is not live here.
    #[test]
    fn the_thirteen_role_paints_are_distinct_so_the_legend_names_one_role() {
        let theme = *crate::runner::driver_at(4, 1, Density::default())
            .env()
            .theme();
        let mut clashes = Vec::new();
        for (i, a) in Role::ALL.iter().enumerate() {
            for b in &Role::ALL[i + 1..] {
                if theme.paint(*a) == theme.paint(*b) {
                    clashes.push((*a, *b));
                }
            }
        }
        assert_eq!(clashes, Vec::new(), "two roles share a paint");

        // And the other direction: a paint that is none of the thirteen is named as one.
        let mut customs = 0;
        let outside = theme.mix(Role::Body, Role::Danger, 0.5);
        assert!(
            describe_paint(&theme, outside, None, &mut customs).starts_with("custom #1"),
            "a mixed paint is named as a role"
        );
        assert_eq!(
            describe_paint(&theme, theme.paint(Role::Danger), None, &mut customs),
            "Danger"
        );
    }

    /// **A cell nobody wrote is not a blank**, which is the one place this side departs from the
    /// owner — and the departure is the partition rule.
    #[test]
    fn an_unwritten_cell_and_a_blank_are_two_characters() {
        let mut canvas = Canvas::new(2, 1);
        let theme = *crate::runner::driver_at(2, 1, Density::default())
            .env()
            .theme();
        canvas.award(0, 0, 0, 0, Role::Body, theme.paint(Role::Body));
        let out = render("two", 0, &theme, &canvas);
        let plane: String = out
            .lines()
            .nth(3)
            .expect("the glyph plane's first row")
            .chars()
            .skip(GUTTER)
            .collect();
        assert_eq!(plane, format!("{UNWRITTEN}{UNWRITTEN}"));
        assert!(out.contains("a cell no verb wrote"));
        assert_ne!(UNWRITTEN, BLANK);
    }

    /// **The alphabet counts what needs a key and nothing else**, in both directions.
    #[test]
    fn the_alphabet_counts_what_the_legend_would_have_to_name() {
        for s in SCREENS {
            let (_, canvas) = shot(s, |t| t);
            assert!(
                alphabet(&canvas) <= GLYPH_KEY_COUNT,
                "{}: {} keys",
                s.scene,
                alphabet(&canvas)
            );
        }
        // A screen of pure ASCII needs none, and one with a border needs one a corner.
        let (_, plain) = shot(
            SCREENS.iter().find(|s| s.scene == "text").expect("text"),
            |t| t,
        );
        assert_eq!(alphabet(&plain), 0);
        let (_, bordered) = shot(
            SCREENS.iter().find(|s| s.scene == "panel").expect("panel"),
            |t| t,
        );
        assert_eq!(alphabet(&bordered), 6, "four corners and two rules");
    }
}
