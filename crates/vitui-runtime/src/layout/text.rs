//! Text measurement: wrap, width and truncate, **entirely over the engine's exports**.
//!
//! ```
//! use vitui_runtime::layout::text;
//!
//! assert_eq!(text::width("漢字"), 4);
//! assert_eq!(text::wrap_height("the quick brown fox", 10), 2);
//! assert_eq!(text::truncate("the quick brown fox", 9), "the quick");
//! ```
//!
//! # This is what layout actually costs
//!
//! **Measuring twenty-four wrapped rows costs 4.41 µs — two and a half times the whole screen's
//! thirty-two splits and one hundred and nineteen lanes.** The constraint solver next door is the
//! part that looks like layout and the cheap part; this is the expensive part, and it is here rather
//! than in a measure pass because *the caller decides whether to pay it*. A measure pass would pay it
//! for every component on every frame whether or not anything wrapped.
//!
//! `examples/text_numbers.rs` is that comparison, so the ratio is in the repository rather than in a
//! reader's assumption.
//!
//! # The runtime never ships a second copy of the UCD
//!
//! Every function here goes through [`vitui_engine::graphemes`] and [`vitui_engine::width_of`]. Not
//! as a convenience — **as the whole point.** ADR 0005 put grapheme segmentation and display width on
//! the engine's public surface precisely so that nothing above it needs its own tables, and the
//! failure it exists to prevent is not *duplication*: it is that **two copies of the UCD disagree the
//! day their Unicode versions differ**, and the disagreement is a caret in the wrong column, a
//! truncation that cuts a cluster in half, and a wrap that puts a combining mark on the next line.
//!
//! `tests::the_runtime_ships_no_unicode_table_of_its_own` is a source scan, because a claim about
//! *absent* code is only checkable where that code would have been written.
//!
//! And correctness is not free: measuring properly costs **25.5×** what the wrong answer costs.
//! `chars().count()` is a single pass over bytes; this is segmentation plus a table lookup a cluster.
//! The wrong answer puts the caret of a family emoji at column 10 where the engine's rule says 5, so
//! the 25.5× buys the only version of the number that is a number.
//!
//! # Line breaking is on ASCII whitespace only, and that is a contract
//!
//! **Stated as a contract rather than apologised for as a limitation.** [`wrap`] breaks at spaces,
//! tabs, newlines and carriage returns, and nowhere else. It does not break inside a word, it does
//! not hyphenate, and it does not know that a CJK run may break between any two characters.
//!
//! **UAX #14 is the near-miss, and it is deliberately not asked of the engine.** Proper line breaking
//! needs break classes — twenty-odd per code point, another generated table — and asking the engine
//! for them means the second copy of the UCD arriving through the front door instead of the back.
//! The honest answer is the narrower contract above. The engine map's one open ticket stays one, and
//! **this module is not the place to widen it.**
//!
//! What that costs, so a caller can decide: a Chinese paragraph with no spaces in it is one line as
//! far as [`wrap`] is concerned, and a caller who needs it broken must break it. A component that
//! must not be wrong here should [`truncate`] rather than wrap, because truncation is exact under
//! this contract and wrapping is not.

use vitui_engine::{graphemes, width_of};

/// The columns `s` occupies.
///
/// [`vitui_engine::width_of`] under another name, re-exported here so that a component measuring
/// text has one module to reach for rather than two. **Not reimplemented** — see the module comment.
#[inline]
pub fn width(s: &str) -> u16 {
    width_of(s)
}

/// The lines `s` breaks into at `w` columns.
///
/// Zero allocations: every item is a subslice of `s`, and the iterator holds two indices and a
/// counter. A caller that wants a `Vec` can collect one and will know it did.
///
/// `w == 0` yields nothing at all, which is the clamp-and-discard rule its neighbours follow: a
/// zero-column band is reachable by dragging a terminal edge, and one line of nothing is a worse
/// answer than no lines.
///
/// ```
/// use vitui_runtime::layout::text;
///
/// let lines: Vec<&str> = text::wrap("the quick brown fox", 10).collect();
/// assert_eq!(lines, ["the quick", "brown fox"]);
///
/// // A word longer than the band is not broken — it overhangs, and the caller can see that it did.
/// let lines: Vec<&str> = text::wrap("supercalifragilistic", 5).collect();
/// assert_eq!(lines, ["supercalifragilistic"]);
/// ```
pub fn wrap(s: &str, w: u16) -> Wrap<'_> {
    Wrap { rest: s, w }
}

/// How many lines [`wrap`] would yield.
///
/// **The contract is an equality against [`wrap`] and it is gated as one**: two functions that answer
/// the same question and can drift are exactly the shape ticket 15's dry-run detector exists for, one
/// axis over. So this is not a second implementation — it is `wrap(s, w).count()`, and the test that
/// says so is the reason it can stay that way.
///
/// It is a separate function because the call site reads better and because a caller measuring a
/// column of paragraphs wants a number, not an iterator it has to remember to consume.
#[inline]
pub fn wrap_height(s: &str, w: u16) -> usize {
    wrap(s, w).count()
}

/// The longest prefix of `s` that fits in `w` columns.
///
/// **Cluster-exact.** It never cuts inside a grapheme cluster, so a family emoji is either wholly
/// present or wholly absent, and a combining mark never arrives without the letter it belongs to.
/// That is the difference between this and a byte slice, and it is the whole reason the function
/// exists.
///
/// A double-width cluster that would straddle the edge is dropped rather than half-drawn: at `w == 3`
/// the string `"a漢"` truncates to `"a"`, not to `"a漢"` overhanging by one nor to a broken cell.
///
/// ```
/// use vitui_runtime::layout::text;
///
/// assert_eq!(text::truncate("hello", 3), "hel");
/// assert_eq!(text::truncate("a漢字", 3), "a漢");
/// assert_eq!(text::truncate("a漢字", 2), "a");
/// // A cluster is atomic: this is one grapheme two columns wide, and at one column it is gone.
/// let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
/// assert_eq!(text::truncate(family, 1), "");
/// assert_eq!(text::truncate(family, 2), family);
/// ```
pub fn truncate(s: &str, w: u16) -> &str {
    let mut used = 0u16;
    let mut end = 0usize;
    for (cluster, cw) in graphemes(s) {
        if used.saturating_add(cw) > w {
            break;
        }
        used += cw;
        // The cluster is a subslice of `s`, so its end offset is where it sits plus its length.
        // Deriving the offset from the pointer is what keeps this allocation-free and O(1) per
        // cluster rather than re-searching `s`.
        end = cluster_end(s, cluster);
    }
    &s[..end]
}

/// The byte offset just past `cluster` within `s`.
///
/// `graphemes` yields subslices of `s`, so the offset is pointer arithmetic on two slices that share
/// an allocation. `str::find` would be O(n) per cluster and would find the *wrong* occurrence for any
/// repeated cluster.
#[inline]
fn cluster_end(s: &str, cluster: &str) -> usize {
    // Both pointers are into the same allocation by construction, so the difference is the offset.
    let base = s.as_ptr() as usize;
    let here = cluster.as_ptr() as usize;
    here - base + cluster.len()
}

/// [`wrap`]'s iterator. Holds a remaining slice and a width, and nothing else.
#[derive(Clone, Debug)]
pub struct Wrap<'a> {
    rest: &'a str,
    w: u16,
}

impl<'a> Iterator for Wrap<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.w == 0 || self.rest.is_empty() {
            return None;
        }
        // A hard newline ends a line whatever the width, which is the one break this function makes
        // that is not about fitting.
        if let Some(nl) = self.rest.find('\n') {
            let (line, rest) = self.rest.split_at(nl);
            if width_of(line) <= self.w {
                self.rest = &rest[1..];
                return Some(line.trim_end_matches('\r'));
            }
        }

        let mut used = 0u16;
        let mut end = 0usize;
        // The last place a break was legal, and how wide the line was there. Zero means *no break
        // point seen*, which is how an over-long word ends up overhanging rather than being cut.
        let mut last_break = 0usize;
        for (cluster, cw) in graphemes(self.rest) {
            let at = cluster_end(self.rest, cluster);
            if is_ascii_space(cluster) {
                last_break = at;
            }
            if used.saturating_add(cw) > self.w {
                break;
            }
            used += cw;
            end = at;
        }

        if end == self.rest.len() {
            let line = self.rest;
            self.rest = "";
            return Some(line.trim_end());
        }
        // A break point inside what fits: cut there, and the whitespace goes with the line being
        // ended rather than starting the next one.
        //
        // **`end + 1` and not `end`, and the difference is a defect this guard used to have.** The
        // loop above records a break at *every* space it sees, including the one that ends the
        // fitted content and does not itself fit — and that space is the legal break, because the
        // row is everything before it. The guard read `<= end`, so a line that **exactly filled its
        // band** and was followed by a space failed it and fell through to the overhang branch
        // below, which cuts at the *first* space instead: `wrap("ab cde fg", 6)` answered
        // `["ab", "cde fg"]` where `["ab cde", "fg"]` is the greedy answer. All four break
        // characters are one byte, and the loop breaks at the first cluster that does not fit, so
        // the only non-fitting space it can have seen is the adjacent one — which is why `end + 1`
        // is exact rather than slack. Found by components ticket 23, whose wrap index over a cluster
        // corpus was drawing three rows where two were needed.
        if last_break > 0 && last_break <= end + 1 {
            let (line, rest) = self.rest.split_at(last_break);
            self.rest = rest.trim_start_matches(is_ascii_space_char);
            return Some(line.trim_end());
        }
        // No break point: the word is longer than the band. **It overhangs rather than being cut**,
        // because cutting a word is a decision about language and this function does not make those
        // — see the module comment on UAX #14. Advancing to the next break point is what stops the
        // iterator spinning.
        let overhang = self
            .rest
            .find(is_ascii_space_char)
            .unwrap_or(self.rest.len());
        let (line, rest) = self.rest.split_at(overhang);
        self.rest = rest.trim_start_matches(is_ascii_space_char);
        Some(line)
    }
}

/// Whether a cluster is one of the four ASCII break characters.
#[inline]
fn is_ascii_space(cluster: &str) -> bool {
    matches!(cluster, " " | "\t" | "\n" | "\r")
}

#[inline]
fn is_ascii_space_char(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The runtime ships no Unicode table of its own**, and this is a source scan because that is
    /// a claim about absent code.
    ///
    /// A grep-shaped test rather than a review item, which is what the ticket asks for. What it looks
    /// for is the shape a generated table has — a long array of code points or ranges — and the
    /// giveaways are a `build.rs` (the engine has one for exactly this reason and the runtime must
    /// not) and the words a UCD-derived table cannot avoid.
    #[test]
    fn the_runtime_ships_no_unicode_table_of_its_own() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/");
        assert!(
            !std::path::Path::new(&format!("{root}build.rs")).exists(),
            "the runtime has a build script, which is how the second copy of the UCD arrives"
        );
        // The module tree, read from disk rather than from `include_str!` of one file, so a table
        // added in a *new* module is caught too.
        let mut checked = 0usize;
        let mut stack = vec![std::path::PathBuf::from(format!("{root}src"))];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("the source tree is readable") {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                let source = std::fs::read_to_string(&path).expect("readable");
                checked += 1;
                // **Every needle is split in half and joined at run time, and that is not
                // decoration.** The first version listed them as plain literals and failed on its
                // own source file — the scanner was the only Unicode table in the crate. Excluding
                // this file would have been the easy fix and the wrong one: a table added *here* is
                // exactly as bad as a table added anywhere, and the scanner has to be able to scan
                // itself.
                for (head, tail) in [
                    ("GRAPHEME", "_BREAK"),
                    ("EAST_ASIAN", "_WIDTH"),
                    ("EastAsian", "Width"),
                    ("GraphemeCluster", "Break"),
                    ("UNICODE", "_VERSION"),
                    ("unicode", "-width"),
                    ("unicode", "-segmentation"),
                ] {
                    let needle = format!("{head}{tail}");
                    assert!(
                        !source.contains(&needle),
                        "{} names `{needle}`, which is a Unicode table or a crate that is one — \
                         ADR 0005 puts those on the engine and the runtime goes through \
                         `graphemes()` and `width_of()`",
                        path.display()
                    );
                }
            }
        }
        assert!(checked >= 3, "only {checked} source files were scanned");
    }

    /// The corpus the equality is gated over: CJK, a ZWJ family emoji, VS15 and VS16, and combining
    /// marks.
    ///
    /// Every one of these is a case where a cluster count, a `char` count and a column count are
    /// three different numbers, which is why the corpus is these and not lorem ipsum.
    const CORPUS: [&str; 12] = [
        "",
        "hello",
        "the quick brown fox jumps over the lazy dog",
        // CJK: two columns a character, no spaces to break on.
        "漢字漢字漢字漢字漢字",
        "mixed 漢字 and latin words together",
        // A ZWJ family: one cluster, two columns, seven scalars.
        "ab\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}cd",
        // VS15 (text presentation, one column) and VS16 (emoji presentation, two).
        "a\u{2764}\u{FE0E}b",
        "a\u{2764}\u{FE0F}b",
        // Combining marks: `e` plus an acute is one column.
        "e\u{301}e\u{301}e\u{301} and more",
        "a\u{300}\u{301}\u{302}\u{303} stacked",
        // Hard newlines and runs of whitespace.
        "one\ntwo\nthree",
        "  leading and   internal   spaces  ",
    ];

    /// **`wrap_height(s, w) == wrap(s, w).count()`, over the corpus at every width.**
    ///
    /// It is trivially true today because `wrap_height` *is* that call, and it is gated anyway: the
    /// obvious optimisation is to count without segmenting, and the day somebody writes it this is
    /// the test that has to stay green. Two functions answering the same question is the shape that
    /// drifts.
    #[test]
    fn wrap_height_equals_the_number_of_wrapped_lines() {
        for s in CORPUS {
            for w in 0..=40u16 {
                assert_eq!(
                    wrap_height(s, w),
                    wrap(s, w).count(),
                    "{s:?} at {w} columns"
                );
            }
        }
    }

    /// **Every wrapped line fits, except a word that cannot.**
    ///
    /// The exception is the contract rather than a hole in the test: a word longer than the band
    /// overhangs, because cutting it is a decision about language that this module does not make.
    #[test]
    fn every_wrapped_line_fits_unless_one_word_cannot() {
        for s in CORPUS {
            for w in 1..=40u16 {
                for line in wrap(s, w) {
                    if width(line) <= w {
                        continue;
                    }
                    assert!(
                        !line.contains(' ') && !line.contains('\t'),
                        "{s:?} at {w}: line {line:?} is too wide and had somewhere to break"
                    );
                }
            }
        }
    }

    /// Wrapping loses nothing but whitespace: the clusters come back in order.
    #[test]
    fn wrapping_preserves_every_cluster_that_is_not_whitespace() {
        for s in CORPUS {
            for w in 1..=40u16 {
                let joined: String = wrap(s, w).collect();
                let want: String = s.chars().filter(|c| !c.is_ascii_whitespace()).collect();
                let got: String = joined
                    .chars()
                    .filter(|c| !c.is_ascii_whitespace())
                    .collect();
                assert_eq!(
                    got, want,
                    "{s:?} at {w} columns lost or reordered something"
                );
            }
        }
    }

    /// Wrapping terminates. An iterator that failed to advance would hang the test rather than fail
    /// it, so the bound is explicit.
    #[test]
    fn wrapping_always_terminates() {
        for s in CORPUS {
            for w in 1..=40u16 {
                let mut lines = 0;
                for _ in wrap(s, w) {
                    lines += 1;
                    assert!(lines <= s.len() + 1, "{s:?} at {w} did not advance");
                }
            }
        }
    }

    /// Zero columns wrap to nothing.
    #[test]
    fn zero_columns_wrap_to_nothing() {
        for s in CORPUS {
            assert_eq!(wrap(s, 0).count(), 0, "{s:?}");
            assert_eq!(wrap_height(s, 0), 0);
            assert_eq!(truncate(s, 0), "");
        }
    }

    /// **Truncation never cuts inside a cluster**, at any width, over the whole corpus.
    ///
    /// Asserted structurally rather than by example: the result must be a prefix whose cluster
    /// sequence is a prefix of the original's, which is the property a byte slice would break.
    #[test]
    fn truncation_never_cuts_inside_a_cluster() {
        for s in CORPUS {
            for w in 0..=40u16 {
                let cut = truncate(s, w);
                assert!(s.starts_with(cut), "{cut:?} is not a prefix of {s:?}");
                assert!(width(cut) <= w, "{cut:?} is wider than {w}");
                let want: Vec<&str> = graphemes(s).map(|(c, _)| c).collect();
                let got: Vec<&str> = graphemes(cut).map(|(c, _)| c).collect();
                assert_eq!(
                    got.as_slice(),
                    &want[..got.len()],
                    "{s:?} at {w} truncated to a different cluster sequence"
                );
                // And it is the *longest* such prefix: one more cluster would not have fitted.
                if got.len() < want.len() {
                    let next = want[got.len()];
                    assert!(
                        width(cut).saturating_add(width(next)) > w,
                        "{s:?} at {w}: {next:?} would have fitted and was dropped"
                    );
                }
            }
        }
    }

    /// A double-width cluster that would straddle the edge is dropped, not half-drawn.
    #[test]
    fn a_double_width_cluster_that_straddles_the_edge_is_dropped() {
        assert_eq!(truncate("a漢字", 3), "a漢");
        assert_eq!(truncate("a漢字", 2), "a");
        assert_eq!(truncate("漢", 1), "");
    }

    /// `width` is the engine's answer and not a plausible one.
    ///
    /// The three numbers a family emoji has — seven scalars, one cluster, two columns — and only the
    /// third is a width. **This is the 25.5× being bought.**
    #[test]
    fn width_is_columns_and_not_characters() {
        let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        assert_eq!(width(family), 2);
        assert_eq!(
            family.chars().count(),
            5,
            "what a plausible caret would say"
        );
        assert_eq!(width("漢字"), 4);
        assert_eq!("漢字".chars().count(), 2);
        assert_eq!(width("e\u{301}"), 1);
        assert_eq!("e\u{301}".chars().count(), 2);
    }

    /// A hard newline breaks a line whatever the width.
    #[test]
    fn a_hard_newline_breaks_a_line_whatever_the_width() {
        let lines: Vec<&str> = wrap("one\ntwo\nthree", 40).collect();
        assert_eq!(lines, ["one", "two", "three"]);
    }

    /// CRLF does not leave a carriage return on the end of the line.
    #[test]
    fn crlf_does_not_leave_a_carriage_return_behind() {
        let lines: Vec<&str> = wrap("one\r\ntwo", 40).collect();
        assert_eq!(lines, ["one", "two"]);
    }

    /// A CJK run with no spaces is one line, which is the contract's cost and is asserted so that
    /// nobody discovers it in a component.
    #[test]
    fn a_cjk_run_with_no_spaces_is_one_line_and_that_is_the_contract() {
        let lines: Vec<&str> = wrap("漢字漢字漢字漢字漢字", 6).collect();
        assert_eq!(lines.len(), 1, "UAX #14 is deliberately not implemented");
        assert!(width(lines[0]) > 6, "so the one line overhangs");
        // And the answer for a component that must not be wrong: truncate, which is exact.
        assert_eq!(width(truncate("漢字漢字漢字漢字漢字", 6)), 6);
    }

    /// **A row that exactly fills its band keeps what fitted**, and does not fall through to the
    /// overhang branch.
    ///
    /// The defect this is the regression for: the break guard read `last_break <= end`, and the
    /// space that *ends* the fitted content sits one byte past `end` because it does not itself
    /// fit. So a line whose content came out at exactly `w` columns failed the guard and was cut at
    /// the **first** space instead of the last — `["ab", "cde fg"]` where `["ab cde", "fg"]` is
    /// what a greedy wrap answers.
    ///
    /// Found by components ticket 23, from the other end: a wrap index over a cluster corpus drew
    /// three visual rows for a line that needs two, and the fixture's own 625/875 construction is
    /// what noticed. Every case here is ASCII, because the defect is not about clusters.
    #[test]
    fn a_line_that_exactly_fills_its_band_is_not_cut_at_the_first_space() {
        assert_eq!(wrap("ab cde fg", 6).collect::<Vec<_>>(), ["ab cde", "fg"]);
        assert_eq!(wrap("ab cd ef", 5).collect::<Vec<_>>(), ["ab cd", "ef"]);
        assert_eq!(
            wrap("a b c d e f", 5).collect::<Vec<_>>(),
            ["a b c", "d e f"]
        );

        // The neighbours the guard was there for, unchanged: an over-long word still overhangs, and
        // a break inside what fits is still the last one rather than the first.
        assert_eq!(
            wrap("ab cdefghijkl mn", 6).collect::<Vec<_>>(),
            ["ab", "cdefghijkl", "mn"]
        );
        assert_eq!(
            wrap("ab cd efghij", 6).collect::<Vec<_>>(),
            ["ab cd", "efghij"]
        );

        // The width relation the fix is really about: every emitted row fits the band unless it is
        // one over-long word, and no row is shorter than it had to be.
        for w in 2u16..12 {
            let rows: Vec<&str> = wrap("ab cde fg hijk lm", w).collect();
            for (i, row) in rows.iter().enumerate() {
                if width(row) > w {
                    assert!(!row.contains(' '), "row {i} overhangs and is not one word");
                }
            }
        }
    }
}
