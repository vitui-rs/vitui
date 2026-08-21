//! The golden frame: the residue the round trip cannot reach.
//!
//! The round trip ([`crate::testing::Harness`]) checks that the bytes replay to the frame. It
//! cannot check that the frame was the **right picture**, and that is a golden's whole job — which
//! is also why goldens are secondary here rather than primary: a golden byte string would pin the
//! encoding, and the encoding is exactly the part tickets 13, 14 and 15 are going to change.
//!
//! Four properties of the format are load-bearing (spec §14):
//!
//! - **Two planes, not one.** A cell carries a cluster and a style word and they change
//!   independently, so a single rendering can only make one of them diff legibly.
//! - **One file column per terminal column, always** — which is why the glyph plane cannot simply
//!   print clusters, and why the continuation cell of a double-width pair gets a column of its own.
//! - **ASCII renders as itself; everything else gets a legend character.** The picture survives for
//!   content that is mostly ASCII, and the cells that would break alignment are exactly the ones
//!   nobody can review by eye anyway.
//! - **The header line is part of the assertion.** A golden taken at the wrong tier is a golden
//!   that passes for the wrong reason.
//!
//! # The one place this format departs from §14's example
//!
//! §14's example prints a blank cell as `.` while also saying *ASCII renders as itself*, and the two
//! sentences cannot both hold: a `.` in the content would then be indistinguishable from a space.
//! **Every reserved marker here is non-ASCII**, so the rule holds literally — `.` is a full stop and
//! nothing else, and a blank is [`BLANK`]. That is not pedantry: a space rendered as a space puts
//! trailing whitespace in a file that editors strip and `git diff` paints red, and a golden a human
//! cannot read in a diff will never be updated correctly.
//!
//! # Two things the format did not have until it was diffed in anger
//!
//! §15's third owed measurement — *the format has never been diffed in anger* — was paid by
//! introducing one-cell defects into a blessed scene and reading the diff. Both halves of the format
//! below came out of that and neither was reasoned in advance:
//!
//! - **Every plane row carries its own number.** A one-cell glyph defect on `caret-blink` — whose
//!   glyph plane is eighty identical blank rows — was reported by `diff` as *a line inserted at row
//!   62 and a line deleted at row 79*, because a line-based diff over identical lines is free to
//!   pair them any way it likes. The change was visible and its **row number was wrong**. A gutter
//!   makes every row unique, so the pairing is forced and the number a reviewer reads is the number
//!   the cell is at. The gutter is fixed-width, which is what keeps *one file column per terminal
//!   column* true with a constant offset rather than false.
//! - **Every legend entry carries its cell count.** A one-cell style defect — the caret's `reverse`
//!   becoming `bold` — moved *nothing* in either plane: the cell still holds the same legend key, so
//!   the whole diff was one legend line. That is legible, but the diff for one cell and the diff for
//!   the whole screen are then the same single line, and a reviewer cannot tell them apart. The
//!   count is the blast radius, in the one place the change is visible.
//!
//! # The legend is assigned in first-appearance order, and that is a trade
//!
//! Row-major first appearance is what §14's own example shows (`a` is `default`, and `default` is
//! what the top-left cell holds). The cost is that a change which introduces a *new* style before an
//! existing one shifts every letter after it, and the whole plane diffs. The alternative — sorting
//! by style bits — has the same failure for the same reason and makes the letters mean nothing, so
//! it buys nothing. What matters is that the case a golden exists to catch, **a defect that changes
//! cells without changing the set of styles**, leaves the legend alone and diffs one character; that
//! is the measurement in this ticket's Progress.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::caps::Capabilities;
use crate::cell::GraphemeId;
use crate::engine::Screen;
use crate::exts::LinkId;
use crate::style::{Color, Style, TAG_INDEXED, TAG_RGB, Underline};
use crate::surface::Surface;
use crate::tables::Tables;

/// A blank cell — a space, U+0020.
///
/// Non-ASCII on purpose: see the module note. A space rendered as a space is trailing whitespace,
/// and a blank row rendered as eighty spaces is a row nobody can see has the right length.
const BLANK: char = '·';

/// The second column of a double-width pair.
///
/// **Reserved**, which is what makes a width bug show up as a shifted row rather than as nothing: a
/// cluster the engine measured as one column when it is two leaves this marker out, and everything
/// to its right in the file moves one column left.
const CONTINUATION: char = '▸';

/// `EMPTY` — a cell no layer painted. It cannot survive compositing over an opaque base, so seeing
/// one in a golden is the finding.
const SKIPPED: char = '▪';

/// The glyph plane's legend characters, for every cluster that is not printable ASCII.
///
/// All non-ASCII, single-column, and chosen against their ASCII look-alikes — no `ο` for `o`, no `ν`
/// for `v`, no `Α` for `A` — because the whole point of the plane is that a reviewer reads it as a
/// picture.
const GLYPH_KEYS: &[char] = &[
    'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'λ', 'μ', 'ξ', 'π', 'σ', 'τ', 'φ', 'ψ', 'ω', 'Γ', 'Δ',
    'Θ', 'Λ', 'Ξ', 'Σ', 'Φ', 'Ψ', 'Ω',
];

/// The width of the row-number gutter every plane row carries: three digits and a space.
///
/// Three digits because a 300x80 screen numbers rows to 79 and a taller terminal exists; the width
/// is fixed rather than sized to the frame so that two goldens of different heights still line up
/// against each other in an editor.
const GUTTER: usize = 4;

/// The style plane's legend characters. ASCII is free here — the style plane has no
/// renders-as-itself rule — which is why §14's example reads `aaaa…`.
const STYLE_KEYS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// The tier the header records: the two axes a frame's *content* can depend on.
pub(crate) fn tier(caps: &Capabilities) -> String {
    format!("{}/{}", caps.colors.word(), caps.glyphs.word())
}

/// What one golden file holds.
///
/// Rendered from a surface and the handle space its cells speak, never from a `Screen`, so that the
/// format's own properties can be asserted over a four-cell surface instead of over 24 000.
pub(crate) fn render(
    scene: &str,
    frame: u32,
    tier: &str,
    surface: &Surface,
    tables: &Tables,
) -> String {
    let (w, h) = surface.size();
    let label = format!("{scene}, frame {frame}");
    // The gutter is three digits, and a fourth would widen every plane row by one — which is *one
    // file column per terminal column* becoming false rather than offset. A frame that tall is not a
    // frame anybody reviews, so this is a guard rather than a width to compute.
    assert!(
        h < 1000,
        "the row gutter is three digits wide; a frame {h} rows tall would widen it"
    );
    let mut legend = Legend::new(label);
    let mut glyph = String::with_capacity((w as usize + GUTTER + 1) * h as usize);
    let mut style = String::with_capacity((w as usize + GUTTER + 1) * h as usize);

    for y in 0..h {
        let _ = write!(glyph, "{y:3} ");
        let _ = write!(style, "{y:3} ");
        for cell in surface.row(y) {
            glyph.push(legend.glyph_key(cell.grapheme));
            style.push(legend.style_key(cell.style));
        }
        glyph.push('\n');
        style.push('\n');
    }

    let mut out = String::with_capacity(glyph.len() + style.len() + 256);
    let _ = writeln!(
        out,
        "scene: {scene}  frame: {frame}  size: {w}x{h}  tier: {tier}"
    );
    out.push_str("\nglyph\n");
    out.push_str(&glyph);
    out.push_str("\nstyle\n");
    out.push_str(&style);
    out.push_str("\nlegend\n");
    legend.write(&mut out, tables);
    out
}

/// Which key stands for which style word and which grapheme handle, in the order they were first
/// seen.
#[derive(Default)]
struct Legend {
    /// The scene and frame, so that a format the picture has outgrown says which picture. `render` is
    /// also the blessing path, so this panic is what a caller sees when the golden cannot even be
    /// created — a message naming neither would send them looking.
    label: String,
    styles: Vec<Style>,
    /// Style word -> its index in `styles`, which is also its index into [`STYLE_KEYS`]. Indexed
    /// rather than keyed by the character, so that a cell costs one hash and no search.
    style_of: HashMap<u64, usize>,
    /// How many cells carry each style, by the same index as `styles`. **The blast radius**: without
    /// it a style word that changed under one cell and one that changed under the whole screen are
    /// the same one-line diff.
    style_cells: Vec<u32>,
    clusters: Vec<GraphemeId>,
    /// Handle -> its index in `clusters`, which is also its index into [`GLYPH_KEYS`].
    glyph_of: HashMap<u32, usize>,
    /// How many cells carry each non-ASCII cluster, by the same index as `clusters`.
    cluster_cells: Vec<u32>,
    /// The reserved markers this frame actually used, each with its own explanation, so the legend
    /// carries no boilerplate for a marker nobody will find in the plane.
    markers: Vec<(char, &'static str)>,
}

impl Legend {
    fn new(label: String) -> Legend {
        Legend {
            label,
            ..Legend::default()
        }
    }

    /// The style plane's character for one style word, minting one on first sight.
    fn style_key(&mut self, s: Style) -> char {
        if let Some(i) = self.style_of.get(&s.bits()).copied() {
            self.style_cells[i] += 1;
            return STYLE_KEYS[i] as char;
        }
        let i = self.styles.len();
        assert!(
            i < STYLE_KEYS.len(),
            "{}: more than {} distinct styles on screen is not reviewable by eye",
            self.label,
            STYLE_KEYS.len()
        );
        self.styles.push(s);
        self.style_cells.push(1);
        self.style_of.insert(s.bits(), i);
        STYLE_KEYS[i] as char
    }

    /// The glyph plane's character for one grapheme handle.
    ///
    /// Printable ASCII is itself; the two sentinels and the blank are reserved markers; everything
    /// else takes a key from [`GLYPH_KEYS`] on first sight. A wide head takes a key and its
    /// [`CONTINUATION`] takes a column of its own, which is the whole of *one file column per
    /// terminal column*.
    fn glyph_key(&mut self, g: GraphemeId) -> char {
        if g.is_continuation() {
            return self.marker(
                CONTINUATION,
                "the second column of the wide cluster to its left",
            );
        }
        if g.is_empty() {
            return self.marker(SKIPPED, "EMPTY - a cell no layer painted");
        }
        if g == GraphemeId::SPACE {
            return self.marker(BLANK, "a blank cell, U+0020");
        }
        if let Some(c) = g.as_scalar() {
            // A wide scalar is not ASCII, so this cannot swallow one: every printable ASCII scalar
            // is one column.
            if c.is_ascii_graphic() {
                return c;
            }
        }
        if let Some(i) = self.glyph_of.get(&g.bits()).copied() {
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
        self.clusters.push(g);
        self.cluster_cells.push(1);
        self.glyph_of.insert(g.bits(), i);
        GLYPH_KEYS[i]
    }

    /// Record that a reserved marker was used, so the legend explains only the ones that appear.
    ///
    /// Markers carry no count: their blast radius is the whole point of the plane they appear in —
    /// a blank is where nothing was drawn — and a count over 24 000 blanks that moves by one every
    /// time anything is drawn anywhere would churn the legend on every frame.
    fn marker(&mut self, m: char, what: &'static str) -> char {
        if !self.markers.iter().any(|(c, _)| *c == m) {
            self.markers.push((m, what));
        }
        m
    }

    fn write(&self, out: &mut String, tables: &Tables) {
        for (i, s) in self.styles.iter().enumerate() {
            let _ = writeln!(
                out,
                "  {}  {}  ({})",
                STYLE_KEYS[i] as char,
                describe_style(*s, tables),
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
                describe_cluster(*g, tables),
                cells(self.cluster_cells[i])
            );
        }
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

/// The code points a handle names, and how many columns it takes.
///
/// Code points rather than the cluster itself, because a reviewer needs to see a ZWJ and a variation
/// selector — the two things a rendered cluster hides and the two things a width bug turns on.
fn describe_cluster(g: GraphemeId, tables: &Tables) -> String {
    let mut scratch = [0u8; 4];
    let Some(text) = tables.interner.render(g, &mut scratch) else {
        return format!("an unresolvable handle, {:#010x}", g.bits());
    };
    let mut out = String::new();
    for c in text.chars() {
        if !out.is_empty() {
            out.push(' ');
        }
        let _ = write!(out, "U+{:04X}", c as u32);
    }
    if g.is_wide_head() {
        out.push_str(" (width 2, continuation follows)");
    } else {
        out.push_str(" (width 1)");
    }
    out
}

/// How a style word reads in the legend: `default`, or the tokens that are not default.
fn describe_style(s: Style, tables: &Tables) -> String {
    let (fg, bg, ul, link) = match s.ext_handle() {
        None => (s.foreground(), s.background(), Color::DEFAULT, LinkId::NONE),
        Some(handle) => match tables.exts.get(handle) {
            Some(e) => (e.fg, e.bg, e.ul, e.link),
            None => {
                return format!("an extended handle this table never minted, {handle}");
            }
        },
    };

    let mut out = String::new();
    let mut token = |t: &str| {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(t);
    };

    if fg != Color::DEFAULT {
        token(&format!("fg={}", describe_color(fg)));
    }
    if bg != Color::DEFAULT {
        token(&format!("bg={}", describe_color(bg)));
    }
    for (bit, name) in [
        (crate::style::BOLD, "bold"),
        (crate::style::DIM, "dim"),
        (crate::style::ITALIC, "italic"),
        (crate::style::REVERSE, "reverse"),
        (crate::style::BLINK, "blink"),
        (crate::style::STRIKETHROUGH, "strikethrough"),
        (crate::style::CONCEAL, "conceal"),
        (crate::style::OVERLINE, "overline"),
    ] {
        if s.bits() & bit != 0 {
            token(name);
        }
    }
    // The underline is one token however many of its two halves are set: a colour with no style is
    // a reachable state and a token that hid it would hide it in the one plane that could show it.
    let style_word = describe_underline(s.underline_style());
    if style_word != "none" || ul != Color::DEFAULT {
        let mut t = format!("underline={style_word}");
        if ul != Color::DEFAULT {
            let _ = write!(t, ":{}", describe_color(ul));
        }
        token(&t);
    }
    if !link.is_none() {
        token(&format!(
            "link={}",
            tables
                .links
                .uri(link)
                .unwrap_or("<an id this table never minted>")
        ));
    }

    if out.is_empty() {
        // An extended word with nothing extended in it should not exist — `ExtStyle::is_extended`
        // is what keeps `restyle` from minting one — so say so rather than print `default` twice.
        if s.is_extended() {
            return "extended, with no extended channel set".to_string();
        }
        return "default".to_string();
    }
    out
}

fn describe_underline(n: u8) -> &'static str {
    match n {
        _ if n == Underline::Single as u8 => "single",
        _ if n == Underline::Double as u8 => "double",
        _ if n == Underline::Curly as u8 => "curly",
        _ if n == Underline::Dotted as u8 => "dotted",
        _ if n == Underline::Dashed as u8 => "dashed",
        _ => "none",
    }
}

fn describe_color(c: Color) -> String {
    match c.tag() {
        TAG_INDEXED => format!("idx{}", c.payload()),
        TAG_RGB => format!("#{:06x}", c.payload()),
        // The two-bit tag has a fourth value spec §3 reserves. A golden is where an accident with
        // it should be visible rather than rendered as one of the other three.
        other => format!("<colour tag {other}, payload {:#08x}>", c.payload()),
    }
}

// -------------------------------------------------------------------------------------------------
// The file, and `VITUI_BLESS=1`
// -------------------------------------------------------------------------------------------------

fn golden_path(scene: &str, frame: u32) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(scene)
        .join(format!("{frame:02}.txt"))
}

/// Assert that this screen's composited frame is the picture on file — or, under `VITUI_BLESS=1`,
/// make the file say what it is.
///
/// The review is the git diff, which is why blessing rewrites in place and takes no argument beyond
/// the variable: a golden nobody can regenerate in one command will be deleted.
pub(crate) fn assert_golden(scene: &str, frame: u32, screen: &Screen) {
    let rendered = render(
        scene,
        frame,
        &tier(screen.capabilities()),
        screen.frame(),
        screen.tables(),
    );
    let path = golden_path(scene, frame);
    let shown = path
        .strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or(&path);

    if blessing() {
        let dir = path.parent().expect("a golden path always has a directory");
        std::fs::create_dir_all(dir)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
        std::fs::write(&path, &rendered)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", shown.display()));
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}\nrun `VITUI_BLESS=1 cargo test -p vitui-engine golden` and review the diff",
            shown.display()
        )
    });
    assert_same(&expected, &rendered, shown, screen.size().1);
}

/// Whether this run rewrites the files instead of comparing against them.
///
/// **It refuses to do that in CI**, and that is a gate rather than a convention: a job that blessed
/// would rewrite the very file it exists to compare against and report green, which is this
/// repository's own *a check that is weaker than the gate is not a check* arriving from a third
/// direction. Both GitLab and GitHub Actions set `CI`.
fn blessing() -> bool {
    let on = std::env::var_os("VITUI_BLESS").is_some_and(|v| v == "1");
    assert!(
        !(on && std::env::var_os("CI").is_some()),
        "VITUI_BLESS=1 in CI would rewrite the file the job exists to compare against"
    );
    on
}

/// Compare two goldens, and panic with [`difference`]'s report if they disagree.
fn assert_same(expected: &str, rendered: &str, shown: &Path, h: u16) {
    if let Some(report) = difference(expected, rendered, shown, h) {
        panic!("{report}");
    }
}

/// The **first** line that differs, by section — or `None` when the two agree.
///
/// Not a whole-file diff: a 300x80 golden is 160 plane lines of 300 columns, and an assertion that
/// printed both copies would bury the one character that changed. The header is reported on its own,
/// because a golden taken at the wrong tier is a golden that passes for the wrong reason and the
/// message has to say so rather than show two identical-looking pictures.
///
/// It returns the report rather than panicking so that **the report itself is testable**. It is the
/// only thing a failing golden gives a human, and a `#[should_panic(expected = …)]` can pin one
/// contiguous substring — so the section and the column could not both be asserted, and the column
/// arithmetic below was unasserted until this was split out.
///
/// **Line-wise rather than byte-wise, and that is load-bearing on one of the three CI runners.**
/// `str::lines` drops a trailing `\r`, so a Windows checkout with `core.autocrlf` on compares equal —
/// and the GitHub workflow's `test` job is a three-OS matrix. A byte comparison would fail every
/// golden there for a reason that has nothing to do with the picture.
fn difference(expected: &str, rendered: &str, shown: &Path, h: u16) -> Option<String> {
    let (e, r): (Vec<&str>, Vec<&str>) = (expected.lines().collect(), rendered.lines().collect());
    for i in 0..e.len().max(r.len()) {
        let (a, b) = (e.get(i), r.get(i));
        if let (Some(a), Some(b)) = (a, b) {
            if a == b {
                continue;
            }
        }
        let at = Section::of(i, h);
        let mut msg = format!(
            "{}: {at} differs\n  on file:  {}\n  rendered: {}",
            shown.display(),
            a.copied().unwrap_or("<the file ends here>"),
            b.copied().unwrap_or("<the frame ends here>")
        );
        if let (Some(a), Some(b)) = (a, b) {
            if let Some(c) = a.chars().zip(b.chars()).position(|(x, y)| x != y) {
                let _ = write!(msg, "\n  first at {}", position(at, c));
            }
        }
        msg.push_str("\nrun `VITUI_BLESS=1 cargo test -p vitui-engine golden` to update it");
        return Some(msg);
    }
    None
}

/// Where in a line the first difference is, in whatever unit that line is measured in.
///
/// A plane's columns are **cells**, and the row-number gutter is not one of them, so the gutter is
/// subtracted. A difference *inside* the gutter says so rather than being clamped to `column 0`: that
/// happens when the two files disagree about the frame's height or a row number was corrupted, and
/// neither is a defect at cell zero.
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
    /// Line `i` of a golden over a frame `h` rows tall.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::geom::Rect;
    use crate::restyle::Restyle;
    use crate::style::Color;

    /// The glyph plane's rows, without the header, the markers or the legend.
    fn plane(out: &str, which: &str, h: u16) -> Vec<String> {
        let start = out
            .lines()
            .position(|l| l == which)
            .expect("the plane is labelled")
            + 1;
        out.lines()
            .skip(start)
            .take(h as usize)
            .map(|l| l.chars().skip(GUTTER).collect())
            .collect()
    }

    fn legend(out: &str) -> Vec<String> {
        let start = out.lines().position(|l| l == "legend").expect("a legend") + 1;
        out.lines()
            .skip(start)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn drawn(w: u16, h: u16, f: impl FnOnce(&mut crate::view::View<'_>)) -> Surface {
        let mut s = Surface::new(w, h);
        {
            let mut v = s.root();
            f(&mut v);
        }
        s
    }

    fn rendered(s: &Surface) -> String {
        render("demo", 0, "truecolor/extended", s, s.tables())
    }

    #[test]
    fn the_header_names_the_scene_the_frame_the_size_and_the_tier() {
        let s = Surface::new(4, 2);
        let out = render("three-dialogs-apart", 7, "ansi16/ascii", &s, s.tables());
        assert_eq!(
            out.lines().next(),
            Some("scene: three-dialogs-apart  frame: 7  size: 4x2  tier: ansi16/ascii"),
            "the header is the assertion, not decoration"
        );
    }

    /// Property three: printable ASCII is itself, and a blank is a marker rather than a space.
    #[test]
    fn ascii_renders_as_itself_and_a_blank_is_visible() {
        let s = drawn(8, 1, |v| {
            v.text(1, 0, "a.Z~", Style::DEFAULT);
        });
        assert_eq!(plane(&rendered(&s), "glyph", 1), vec!["·a.Z~···"]);
        assert!(
            legend(&rendered(&s))
                .iter()
                .any(|l| l.contains("a blank cell")),
            "the marker that appears is explained"
        );
        assert!(
            legend(&rendered(&s))
                == vec![
                    "  a  default  (8 cells)".to_string(),
                    "  ·  a blank cell, U+0020".to_string(),
                ],
            "one style over all eight cells, and a marker with no count"
        );
    }

    /// Property one: two planes, because the two halves of a cell change independently.
    ///
    /// The same glyphs under a different style word must leave the glyph plane byte-identical and
    /// move only the style plane. A single-plane rendering can make one of the two diff legibly and
    /// this is the test that says which.
    #[test]
    fn two_planes_so_that_a_style_change_alone_is_legible() {
        let plain = drawn(6, 1, |v| {
            v.text(0, 0, "hello", Style::DEFAULT);
        });
        let bold = drawn(6, 1, |v| {
            v.text(0, 0, "hello", Style::new().bold());
        });
        let (a, b) = (rendered(&plain), rendered(&bold));
        assert_eq!(plane(&a, "glyph", 1), plane(&b, "glyph", 1));
        assert_ne!(plane(&a, "style", 1), plane(&b, "style", 1));
        assert_eq!(plane(&b, "style", 1), vec!["aaaaab"]);
        assert_eq!(
            legend(&b),
            vec![
                "  a  bold  (5 cells)".to_string(),
                "  b  default  (1 cell)".to_string(),
                "  ·  a blank cell, U+0020".to_string(),
            ],
        );
    }

    /// And the other direction: a glyph change alone leaves the style plane alone.
    #[test]
    fn and_a_glyph_change_alone_leaves_the_style_plane_alone() {
        let a = drawn(6, 1, |v| {
            v.text(0, 0, "hello", Style::DEFAULT);
        });
        let b = drawn(6, 1, |v| {
            v.text(0, 0, "hellp", Style::DEFAULT);
        });
        let (a, b) = (rendered(&a), rendered(&b));
        assert_eq!(plane(&a, "style", 1), plane(&b, "style", 1));
        assert_ne!(plane(&a, "glyph", 1), plane(&b, "glyph", 1));
    }

    /// Property two, the whole of it: a wide cluster takes two file columns, and the second one is
    /// the reserved marker.
    #[test]
    fn one_file_column_per_terminal_column_including_a_wide_pair() {
        let s = drawn(8, 1, |v| {
            v.text(0, 0, "漢字ab", Style::DEFAULT);
        });
        let out = rendered(&s);
        let rows = plane(&out, "glyph", 1);
        assert_eq!(rows, vec!["α▸β▸ab··"]);
        assert_eq!(
            rows[0].chars().count(),
            8,
            "one file column per terminal column, always"
        );
        let l = legend(&out);
        assert!(
            l.contains(&"  α  U+6F22 (width 2, continuation follows)  (1 cell)".to_string()),
            "{l:?}"
        );
        assert!(
            l.iter()
                .any(|e| e.contains("the second column of the wide cluster")),
            "{l:?}"
        );
    }

    /// The reason the continuation is a *reserved* marker rather than nothing: the same content with
    /// the wide cluster measured as one column shifts everything to its right one column left.
    ///
    /// The defective row is built by hand because no verb can produce it — which is the point. Spec
    /// §3's pairing invariant is asserted over both a surface and the composited frame
    /// ([`crate::testing::assert_pairing_holds`]); this is what the *format* contributes, and it is
    /// the half that survives the invariant being wrong.
    #[test]
    fn a_width_bug_shifts_a_row() {
        let good = drawn(6, 1, |v| {
            v.text(0, 0, "漢ab", Style::DEFAULT);
        });
        let mut bad = Surface::new(6, 1);
        {
            // The same three clusters, with the wide one occupying one column instead of two.
            let handle = bad
                .tables_mut()
                .interner
                .handle("漢")
                .expect("a single scalar always has a handle");
            assert!(handle.is_wide_head());
            let narrow = GraphemeId::scalar('漢');
            let row = bad.row_mut(0);
            row[0] = Cell::new(narrow, Style::DEFAULT);
            row[1] = Cell::new(GraphemeId::scalar('a'), Style::DEFAULT);
            row[2] = Cell::new(GraphemeId::scalar('b'), Style::DEFAULT);
        }
        let (g, b) = (rendered(&good), rendered(&bad));
        assert_eq!(plane(&g, "glyph", 1), vec!["α▸ab··"]);
        assert_eq!(plane(&b, "glyph", 1), vec!["αab···"]);
        assert_ne!(
            plane(&g, "glyph", 1),
            plane(&b, "glyph", 1),
            "a row that lost a column is a row that moved"
        );
    }

    /// Every style channel a cell can carry, spelled the way §14's example spells it.
    #[test]
    fn the_legend_spells_every_channel_a_cell_can_carry() {
        let mut s = Surface::new(5, 1);
        let link = s.tables_mut().link("https://example.invalid/1");
        {
            let mut v = s.root();
            v.text(
                0,
                0,
                "abcd",
                Style::new()
                    .fg(Color::rgb(0xff, 0xff, 0xff))
                    .bg(Color::indexed(236))
                    .bold()
                    .italic(),
            );
            v.restyle(
                Rect::new(0, 0, 4, 1),
                &Restyle {
                    set: Restyle::UNDERLINE_CURLY,
                    ul: Some(Color::rgb(0xff, 0, 0)),
                    link: Some(link),
                    ..Restyle::default()
                },
            );
        }
        let l = legend(&rendered(&s));
        assert_eq!(
            l[0],
            "  a  fg=#ffffff bg=idx236 bold italic underline=curly:#ff0000 \
             link=https://example.invalid/1  (4 cells)"
        );
    }

    #[test]
    fn a_default_style_says_so_rather_than_printing_nothing() {
        let s = Surface::new(2, 1);
        assert_eq!(legend(&rendered(&s))[0], "  a  default  (2 cells)");
    }

    /// `EMPTY` cannot survive compositing over an opaque base, so a golden that shows one is the
    /// finding. It gets a marker rather than a panic for exactly that reason.
    #[test]
    fn a_cell_no_layer_painted_is_visible_rather_than_blank() {
        let mut s = Surface::new(3, 1);
        s.row_mut(0)[1] = Cell::new(GraphemeId::EMPTY, Style::DEFAULT);
        let out = rendered(&s);
        assert_eq!(plane(&out, "glyph", 1), vec!["·▪·"]);
        assert!(legend(&out).iter().any(|l| l.contains("EMPTY")));
    }

    fn report(a: &Surface, b: &Surface, h: u16) -> String {
        difference(
            &rendered(a),
            &rendered(b),
            Path::new("tests/golden/demo/00.txt"),
            h,
        )
        .expect("the two goldens differ")
    }

    /// The header is reported on its own, because two 300x80 pictures that differ only in their tier
    /// look identical in an assertion that prints both.
    #[test]
    fn a_golden_taken_at_the_wrong_tier_is_reported_on_the_header() {
        let s = Surface::new(2, 1);
        let a = render("demo", 0, "truecolor/extended", &s, s.tables());
        let b = render("demo", 0, "ansi16/ascii", &s, s.tables());
        let msg = difference(&a, &b, Path::new("tests/golden/demo/00.txt"), 1)
            .expect("two tiers disagree");
        assert!(msg.contains("the header line differs"), "{msg}");
    }

    /// The row **and** the column, and the column is a cell rather than a byte of the line.
    ///
    /// Both halves are asserted because the gutter subtraction is arithmetic nothing else checks:
    /// drop it and every message points four cells to the right of the defect, with every other test
    /// still green.
    #[test]
    fn a_mismatched_plane_names_its_row_and_its_cell_column() {
        let a = drawn(8, 3, |v| {
            v.text(5, 1, "x", Style::DEFAULT);
        });
        let b = drawn(8, 3, |v| {
            v.text(5, 1, "y", Style::DEFAULT);
        });
        let msg = report(&a, &b, 3);
        assert!(msg.contains("the glyph plane, row 1 differs"), "{msg}");
        assert!(msg.contains("first at column 5"), "{msg}");
    }

    /// And the style plane reports in the same unit.
    #[test]
    fn a_mismatched_style_plane_names_its_row_and_its_cell_column() {
        let a = drawn(8, 3, |v| {
            v.text(5, 1, "x", Style::DEFAULT);
        });
        let b = drawn(8, 3, |v| {
            v.text(5, 1, "x", Style::new().bold());
        });
        let msg = report(&a, &b, 3);
        assert!(msg.contains("the style plane, row 1 differs"), "{msg}");
        assert!(msg.contains("first at column 5"), "{msg}");
    }

    /// A difference inside the gutter is not a defect at cell zero, and is not reported as one.
    ///
    /// It happens when the two files disagree about the frame's height — the header catches that
    /// first — or when a row number was corrupted, which is the case this pins: clamping to
    /// `column 0` would send a reviewer to the wrong end of the row.
    #[test]
    fn a_difference_inside_the_gutter_says_so_rather_than_naming_cell_zero() {
        let s = Surface::new(4, 3);
        let good = rendered(&s);
        let corrupt = good.replacen("  1 ", "  9 ", 1);
        let msg = difference(&good, &corrupt, Path::new("tests/golden/demo/00.txt"), 3)
            .expect("the row number was changed");
        assert!(msg.contains("the glyph plane, row 1 differs"), "{msg}");
        assert!(
            msg.contains("character 2, inside the row-number gutter"),
            "{msg}"
        );
    }

    /// The panic path itself still fires, so splitting the report out did not leave the assertion
    /// asserting nothing.
    #[test]
    #[should_panic(expected = "the glyph plane, row 0 differs")]
    fn assert_same_panics_with_the_report() {
        let a = drawn(4, 1, |v| {
            v.text(0, 0, "x", Style::DEFAULT);
        });
        let b = drawn(4, 1, |v| {
            v.text(0, 0, "y", Style::DEFAULT);
        });
        assert_same(
            &rendered(&a),
            &rendered(&b),
            Path::new("tests/golden/demo/00.txt"),
            1,
        );
    }

    /// The first finding of §15's owed measurement, as a test.
    ///
    /// A plane of identical rows lets a line-based diff pair any row with any other, and it does: a
    /// one-cell defect on `caret-blink`'s eighty identical blank glyph rows was reported as an insert
    /// at row 62 and a delete at row 79. The gutter is what forces the pairing, so the property is
    /// that **no two plane rows of a blank frame are the same line**.
    #[test]
    fn every_plane_row_carries_its_number_so_that_no_two_blank_rows_are_the_same_line() {
        let s = Surface::new(6, 4);
        let out = rendered(&s);
        let rows: Vec<&str> = out
            .lines()
            .skip(out.lines().position(|l| l == "glyph").expect("a plane") + 1)
            .take(4)
            .collect();
        assert_eq!(rows[0], "  0 ······");
        assert_eq!(rows[3], "  3 ······");
        let mut unique = rows.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            rows.len(),
            "four blank rows, four distinct lines"
        );
        assert_eq!(
            rows[0].chars().count(),
            GUTTER + 6,
            "the gutter is fixed-width, so one file column per terminal column holds with an offset"
        );
    }

    /// The second finding: a style *value* that changes under one cell and one that changes under the
    /// whole screen are the same one-line legend diff without the count.
    #[test]
    fn a_legend_entry_carries_its_blast_radius() {
        let one = drawn(4, 1, |v| {
            v.text(0, 0, "x", Style::new().bold());
        });
        let all = drawn(4, 1, |v| {
            v.text(0, 0, "xxxx", Style::new().bold());
        });
        assert_eq!(legend(&rendered(&one))[0], "  a  bold  (1 cell)");
        assert_eq!(legend(&rendered(&all))[0], "  a  bold  (4 cells)");
    }

    #[test]
    fn a_golden_that_matches_itself_asserts_nothing() {
        let s = drawn(4, 2, |v| {
            v.text(0, 0, "ok", Style::new().reverse());
        });
        let out = rendered(&s);
        assert_same(&out, &out, Path::new("tests/golden/demo/00.txt"), 2);
    }
}

/// The blessed scenes, driven through the round trip and compared against their files.
///
/// One test rather than three, because the list is the assertion: a scene dropped from [`BLESSED`]
/// is a scene whose picture nobody checks, and a `#[test]` per scene makes that a deletion nobody
/// notices.
#[cfg(test)]
mod scene_goldens {
    use super::*;
    use crate::scenes::{H, W, scenes};
    use crate::testing::{Harness, assert_pairing_holds, bisecting_cjk};

    /// **Every golden this crate has on disk**, with how many frames each keeps. One list, and both
    /// directions are closed against it: `every_golden_on_the_list_matches_its_file` drives each
    /// entry, and `the_directory_holds_exactly_the_goldens_on_the_list` fails on a file that no entry
    /// names. A golden nobody drives and a test whose file nobody registered are the same defect from
    /// two sides, and each of the two tests catches one of them.
    ///
    /// Three of spec §14's twelve, not all, and the choice is the ticket's: **caret-blink** is the
    /// forty-layer stack with one cell toggling, which is the shape a two-plane format exists for;
    /// **three-dialogs-apart** is the picture a per-row damage model got 2.53x wrong; and
    /// **sparse-chart-400-points** is the scene that has decided three tickets. The other nine are
    /// covered by the round trip and by [`crate::gates`], which is where a picture nobody would
    /// review belongs — `every-cell-a-distinct-style` has 24 000 distinct styles and no legend can
    /// hold them.
    ///
    /// The fourth is not one of §14's twelve, and is here because **none of the twelve draws a wide
    /// cluster**: without it the [`CONTINUATION`] marker is tested over a six-column surface and
    /// appears in no artefact anybody reviews. See [`the_cjk_frame`].
    ///
    /// Two frames for a scene, and the **birth frame is not one of them**: adding a layer damages its
    /// whole rectangle, so the frame before any scene verb has run is a picture of the stack rather
    /// than of the scene. Two is what it takes to make the caret's toggle visible, and the toggle is
    /// the case where the glyph plane must not move.
    const GOLDENS: [(&str, u32); 4] = [
        ("caret-blink", 2),
        ("three-dialogs-apart", 2),
        ("sparse-chart-400-points", 2),
        (CJK, 1),
    ];

    const CJK: &str = "twelve-bisecting-layers-over-cjk";

    /// The tier every golden is taken at, pinned rather than detected.
    ///
    /// **Both halves are needed and neither is enough.** Pinning the capabilities makes a frame's
    /// *content* deterministic; the deterministic clock — [`Clock::Manual`](crate::Clock), which
    /// [`Harness`] always uses — makes the *sequence* deterministic. A golden is reproducible only
    /// with both, and `colors: TrueColor` is what makes headless a **declared** tier rather than the
    /// lowest one.
    /// Architecture ticket 22 added three fields and this names all seven, because a golden that
    /// spread `..Default::default()` would let a new axis change every blessed picture silently. Two
    /// of the three are pinned to the value they already had: **silence is not declarable and does
    /// not need to be** — it is what a caller-supplied sink gets, and §5 leaves a default-coloured
    /// cell unmixed rather than mixing it against a guess, which is the behaviour every golden on
    /// the list was blessed under.
    fn pinned() -> crate::caps::Overrides {
        crate::caps::Overrides {
            colors: Some(crate::caps::ColorDepth::TrueColor),
            glyphs: Some(crate::caps::GlyphSet::Extended),
            default_fg: None,
            default_bg: None,
            hyperlinks: Some(false),
            legacy_sgr: Some(false),
            width: Some(crate::caps::WidthSource::Tables),
        }
    }

    /// Every entry of [`GOLDENS`], driven and compared.
    ///
    /// One test rather than one per golden, because the list is the assertion: an entry dropped from
    /// it is a picture nobody checks, and a `#[test]` per golden makes that a deletion nobody
    /// notices. A name that is neither one of §14's twelve nor [`CJK`] fails here rather than being
    /// skipped.
    #[test]
    fn every_golden_on_the_list_matches_its_file() {
        for (name, frames) in GOLDENS {
            if name == CJK {
                the_cjk_frame();
                continue;
            }
            let mut scene = scenes()
                .into_iter()
                .find(|s| s.name() == name)
                .unwrap_or_else(|| {
                    panic!("`{name}` is neither one of spec §14's twelve nor {CJK}")
                });
            let mut h = Harness::with_overrides(W, H, pinned()).labelled(name);
            scene.build(&mut h.screen);
            // The birth frame: presented and then left behind, exactly as `crate::gates` does.
            h.present();
            for t in 0..frames {
                scene.step(&mut h.screen, t);
                h.present();
                assert_golden(name, t, &h.screen);
            }
        }
    }

    /// The screen of mixed CJK under twelve bisecting layers, blessed.
    ///
    /// **The one picture in this crate that no other instrument shows a human.** The round trip
    /// cannot see a frame whose halves do not pair — the serializer emits nothing for a continuation
    /// and the terminal model consumes nothing for one, so the two are wrong in the same direction —
    /// and `crate::gates::the_pairing_invariant_survives_twelve_bisecting_layers_over_cjk` asserts
    /// the invariant and the equality against the reference compositor without producing anything
    /// anybody reads. This is the half that makes the repair reviewable, and it is the only golden
    /// here whose glyph plane carries a [`CONTINUATION`] marker.
    ///
    /// It is the gate's **own** fixture — [`bisecting_cjk`], rows and rects both — rather than a copy
    /// of it. A scene built to suit the instrument measuring it is a scene defined by what this
    /// framework does, which is what [`crate::scenes`] exists to forbid; and a copied one would go on
    /// passing against its own blessed file after the gate's rects changed underneath it.
    fn the_cjk_frame() {
        let mut h = Harness::with_overrides(W, H, pinned()).labelled(CJK);
        let base = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, W, H), true);
        for y in 0..H {
            let row = bisecting_cjk::row(y, W);
            h.screen.layers().view(base).expect("just added").text(
                0,
                y as i32,
                &row,
                Style::DEFAULT,
            );
        }
        for i in 0..bisecting_cjk::BISECTORS {
            let rect = bisecting_cjk::rect(i, W);
            let id = h.screen.layers().add_content(1 + i, rect, i % 2 == 0);
            for y in 0..rect.h {
                h.screen.layers().view(id).expect("just added").text(
                    0,
                    y as i32,
                    &bisecting_cjk::row(y, W),
                    Style::DEFAULT,
                );
            }
        }
        h.present();
        // The frame's halves pair, which is the gate's property; the golden is what says the repair
        // produced the picture somebody intended rather than merely a consistent one.
        assert_pairing_holds(h.screen.frame());
        assert_golden(CJK, 0, &h.screen);
    }

    /// The other direction: a file on disk that no entry of [`GOLDENS`] names.
    ///
    /// 500 KB of fixtures is worth a test that says which of them anything still looks at. A golden
    /// left behind by a renamed scene passes for a golden until somebody reads the diff it never
    /// produces.
    #[test]
    fn the_directory_holds_exactly_the_goldens_on_the_list() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
        let mut on_disk: Vec<String> = Vec::new();
        for dir in std::fs::read_dir(&root)
            .expect("tests/golden exists")
            .flatten()
        {
            let scene = dir.file_name().to_string_lossy().into_owned();
            for file in std::fs::read_dir(dir.path())
                .expect("a scene directory")
                .flatten()
            {
                on_disk.push(format!("{scene}/{}", file.file_name().to_string_lossy()));
            }
        }
        let mut expected: Vec<String> = GOLDENS
            .iter()
            .flat_map(|(name, frames)| (0..*frames).map(move |f| format!("{name}/{f:02}.txt")))
            .collect();
        on_disk.sort();
        expected.sort();
        assert_eq!(
            on_disk, expected,
            "tests/golden holds files no entry of GOLDENS names, or is missing ones it does"
        );
    }

    /// The header's tier is what a golden is taken at, and it has to be the tier that was pinned —
    /// otherwise the assertion that a golden fails at the wrong tier is asserting nothing.
    #[test]
    fn the_pinned_tier_is_what_the_header_records() {
        let h = Harness::with_overrides(4, 2, pinned());
        assert_eq!(tier(h.screen.capabilities()), "truecolor/extended");
    }
}
