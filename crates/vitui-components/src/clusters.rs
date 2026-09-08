//! **The cluster corpus, and the forward cluster step this crate had to reconstruct to have one.**
//!
//! The cluster corpus.
//!
//! > Every number and every gate runs on clusters that are not one code point: a combining acute,
//! > two marks on one base, a ZWJ family, a skin-tone modifier, `U+FE0F`, a regional-indicator pair,
//! > CJK.
//!
//! That sentence is the reason this module is a value rather than a `const CORPUS: &str` at the top
//! of [`crate::document`]. A corpus written as a string literal is a corpus that can be **sampled**
//! — a later ticket measuring "the same thing on ASCII because it was quicker" leaves nothing
//! behind that says a kind stopped being run. Here the kinds are [`Kind::ALL`], the rota is
//! [`ROTA`], and `tests::the_corpus_carries_every_kind_section_eleven_names` is what refuses a
//! corpus that has quietly lost one.
//!
//! # `graphemes()` is unreachable from this crate, and that is a finding rather than an obstacle
//!
//! The mechanism is a caret *moved in cluster steps*, and a cluster step needs segmentation.
//! `vitui_engine::graphemes` is the engine's own forward iterator, and what
//! `vitui-runtime` re-exports of it is **nothing**: `vitui_runtime::layout::text` ships [`width`],
//! `wrap`, `wrap_height` and [`truncate`], and `crate::line::ENGINE_NAMES` is a list of *types* —
//! `graphemes` is a free function and is on no re-export line in the crate. C6 says this crate's
//! dependency table is `vitui-runtime` and nothing else, so there is no second spelling.
//!
//! **A forward cluster step is expressible anyway, and it is two calls rather than one.**
//! [`next_cluster`] asks `truncate(rest, 1)`; a cluster one column wide comes back, and a cluster
//! *two* columns wide comes back as the empty string because it does not fit — so the second probe
//! is `truncate(rest, 2)`. The engine's tables answer 0, 1 or 2 and nothing else, which is what
//! makes two probes exhaustive rather than a guess.
//!
//! **The exception is width 0, and it is asserted rather than assumed.** `truncate` is greedy, so a
//! zero-width cluster always fits and arrives welded to whatever follows it:
//! `truncate("\u{200B}x", 1)` is `"\u{200B}x"`, which is two clusters in one answer. Every entry of
//! [`ROTA`] is therefore checked to be at least one column wide by
//! `tests::no_cluster_in_the_corpus_is_zero_columns_wide`, and the day one is not, the probe is
//! wrong and that test is what says so rather than a caret landing in the wrong column six months
//! later.
//!
//! What this costs `field` is a real number and it belongs to the field: a step is two
//! `truncate` calls where the engine would answer in one `Graphemes::next`, and the O(prefix)
//! backward question is unchanged. It is recorded here as
//! [`crate::gates::Instrument::Barrier`] rather than argued, because the map is
//! closed and *a ticket that appears to require reopening a decision has found something, not
//! decided something*.
//!
//! # The boundary set is derived three ways, and that is the point
//!
//! A caret gate whose oracle is the same code the caret uses is a gate testing a copy. So:
//!
//! 1. **By construction.** [`ROTA`] is a list of *clusters*, so [`Corpus::boundaries`] is the
//!    prefix sum of their byte lengths and no segmentation was involved in producing it.
//! 2. **By the forward probe.** [`boundaries_by_probe`] walks [`next_cluster`] from byte 0.
//! 3. **By the width sweep.** [`boundaries_by_truncate`] asks `truncate(s, w)` for every `w` from 0
//!    to the string's width and keeps the byte lengths. This is exact **only** because no cluster
//!    here is zero columns wide — two zero-width clusters sit at the same column and a sweep over
//!    columns cannot separate them — which is the same precondition [`next_cluster`] carries, from
//!    the other side.
//!
//! `tests::the_three_derivations_of_the_boundary_set_agree` is the equality. Two of the three go
//! through the engine's tables and one does not, so the day the engine's Unicode version moves under
//! this crate, derivation 1 is the one that disagrees and the failure names a cluster.
//!
//! # The count is a hundred and sixty-seven and the recorded figure is a hundred and sixty-six
//!
//! [`CYCLES`] whole turns of an eight-word rota is 167 clusters, and there is no arrangement of
//! whole cycles that is 166. **Padding to 166 was refused**: a corpus with a hand-added cluster on
//! the end is a corpus tuned to a remembered number, which is the move this repository forbids by
//! name. See [`crate::document::CORPUS_CLUSTERS`] for the rest of the arithmetic, including the one
//! part of the figure that reproduces exactly and is not a measurement at all.

use vitui_runtime::layout::text::{truncate, width};

/// One of the seven named kinds, plus the plain ASCII the wrap needs.
///
/// **`Kind::Ascii` is an eighth arm and not a smuggled ninth kind.** Seven are listed, and a corpus
/// of nothing but them has no spaces in it — which would make the wrap scene next door a scene about
/// a string with no break opportunities. It is named so a report can say how much of the corpus is
/// the easy case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// One code point, one column, one cluster. The easy case, present so that there is prose.
    Ascii,
    /// A base and a combining acute: `e` + `U+0301`.
    Acute,
    /// **Two marks on one base**: `o` + `U+0323` + `U+0302`, which is how Vietnamese spells `ộ`.
    TwoMarks,
    /// A ZWJ family: four emoji and three `U+200D`, seven code points in one cluster.
    ZwjFamily,
    /// A skin-tone modifier: an emoji and a `U+1F3FB`–`U+1F3FF` selector.
    SkinTone,
    /// `U+FE0F`, the emoji presentation selector, which changes the *width* of what precedes it.
    Presentation,
    /// A regional-indicator pair, which is a flag exactly when there are two of them.
    RegionalPair,
    /// CJK, the one kind that is one code point and **two columns**.
    Cjk,
}

impl Kind {
    /// Every kind, in [`ROTA`]'s order.
    pub const ALL: [Kind; 8] = [
        Kind::Ascii,
        Kind::Acute,
        Kind::TwoMarks,
        Kind::ZwjFamily,
        Kind::SkinTone,
        Kind::Presentation,
        Kind::RegionalPair,
        Kind::Cjk,
    ];

    /// The word a report prints.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Ascii => "ascii",
            Kind::Acute => "combining acute",
            Kind::TwoMarks => "two marks on one base",
            Kind::ZwjFamily => "zwj family",
            Kind::SkinTone => "skin-tone modifier",
            Kind::Presentation => "u+fe0f",
            Kind::RegionalPair => "regional-indicator pair",
            Kind::Cjk => "cjk",
        }
    }

    /// Whether this kind is one of the named ones. **Seven of the eight.** See [`Kind::Ascii`].
    pub fn named_by_spec(self) -> bool {
        self != Kind::Ascii
    }

    /// **Whether `cluster` really is one of these**, checked against the code points it is made of
    /// rather than against the label somebody put on it.
    ///
    /// The label alone is a vacuous gate and it was one: the first version of
    /// `tests::the_corpus_carries_every_kind_section_eleven_names` counted the `kinds` vector, so
    /// the ZWJ family's word could be replaced with `"f", "a", "m"` and the corpus still reported
    /// seven ZWJ clusters. That is exactly the accident
    /// [`crate::obligations::Verdict::of`] refuses one file over — *an equality between two things
    /// that do not exist holds* — arriving on the one value the whole ticket runs over.
    ///
    /// The joining spaces are `Kind::Ascii` and satisfy it, which is why the ASCII arm is the
    /// permissive one.
    pub fn describes(self, cluster: &str) -> bool {
        let points: Vec<char> = cluster.chars().collect();
        let columns = width(cluster);
        match self {
            Kind::Ascii => points.len() == 1 && points[0].is_ascii() && columns == 1,
            // A base and one combining mark, one column wide.
            Kind::Acute => points.len() == 2 && points[1] == '\u{301}' && columns == 1,
            // A base and two, still one column wide — which is the whole point of the kind.
            Kind::TwoMarks => {
                points.len() == 3
                    && points[1] == '\u{323}'
                    && points[2] == '\u{302}'
                    && columns == 1
            }
            // Four emoji and three joiners: seven code points, two columns.
            Kind::ZwjFamily => {
                points.len() >= 5
                    && points.iter().filter(|c| **c == '\u{200D}').count() >= 2
                    && columns == 2
            }
            // An emoji and a `U+1F3FB`-`U+1F3FF` selector.
            Kind::SkinTone => {
                points.len() == 2
                    && ('\u{1F3FB}'..='\u{1F3FF}').contains(&points[1])
                    && columns == 2
            }
            // The presentation selector, which is what makes the cluster two columns rather than
            // one — so the width is not decoration here, it is the kind.
            Kind::Presentation => points.len() == 2 && points[1] == '\u{FE0F}' && columns == 2,
            // Two regional indicators, which are a flag exactly when there are two of them.
            Kind::RegionalPair => {
                points.len() == 2
                    && points
                        .iter()
                        .all(|c| ('\u{1F1E6}'..='\u{1F1FF}').contains(c))
                    && columns == 2
            }
            // **One code point and two columns**, which is the one kind that is wide without being
            // long.
            Kind::Cjk => points.len() == 1 && !points[0].is_ascii() && columns == 2,
        }
    }
}

/// One word of the corpus, **as a list of clusters rather than as a string**.
///
/// The list is what makes [`Corpus::boundaries`] a construction instead of a measurement: a word
/// that arrived as `&str` would have to be segmented before its boundaries were known, and the
/// thing under test is the segmentation.
#[derive(Clone, Copy, Debug)]
pub struct Word {
    /// Which kind the word is here to carry.
    pub kind: Kind,
    /// Its clusters, in order.
    pub clusters: &'static [&'static str],
}

/// **The rota: one word per kind, in [`Kind::ALL`]'s order.**
///
/// Every word is a real word rather than a lone cluster wherever the kind admits one — `café` and
/// `Hội` are the two marks in the places a document actually puts them, welded into ASCII, which is
/// what makes the wrap scene next door break *inside* prose rather than between isolated symbols.
pub const ROTA: [Word; 8] = [
    Word {
        kind: Kind::Ascii,
        clusters: &["t", "h", "e"],
    },
    Word {
        kind: Kind::Acute,
        clusters: &["c", "a", "f", "e\u{301}"],
    },
    Word {
        kind: Kind::TwoMarks,
        clusters: &["H", "o\u{323}\u{302}", "i"],
    },
    Word {
        kind: Kind::ZwjFamily,
        clusters: &["\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}"],
    },
    Word {
        kind: Kind::SkinTone,
        clusters: &["\u{1F44D}\u{1F3FD}"],
    },
    Word {
        kind: Kind::Presentation,
        clusters: &["\u{2764}\u{FE0F}"],
    },
    Word {
        kind: Kind::RegionalPair,
        clusters: &["\u{1F1EF}\u{1F1F5}"],
    },
    Word {
        kind: Kind::Cjk,
        clusters: &["\u{6F22}", "\u{5B57}"],
    },
];

/// **How many whole turns of [`ROTA`] the corpus is. Seven.**
///
/// Chosen because seven turns is the smallest whole number of them that puts the cluster count
/// within two of the 166, and a *partial* turn is a sampled corpus — the one thing the ticket says
/// the corpus may not be. The exact count is [`Corpus::len`] and the difference is written down
/// rather than closed.
pub const CYCLES: usize = 7;

/// The cluster the rota's words are joined by. A space, and it is a cluster like any other.
pub const JOINER: &str = " ";

/// **The corpus: a string, its clusters, and where every boundary is.**
///
/// Built once by [`corpus`]. Nothing here is `pub` by field, because a caller that could edit
/// `boundaries` without editing `text` would hold a corpus whose oracle disagrees with its subject
/// and no test would say so.
#[derive(Clone, Debug)]
pub struct Corpus {
    clusters: Vec<&'static str>,
    kinds: Vec<Kind>,
    text: String,
    boundaries: Vec<usize>,
    columns: Vec<u16>,
}

impl Corpus {
    /// The corpus as a string.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Its clusters, in order.
    pub fn clusters(&self) -> &[&'static str] {
        &self.clusters
    }

    /// The kind of every cluster, in order. `Kind::Ascii` for the joining spaces.
    pub fn kinds(&self) -> &[Kind] {
        &self.kinds
    }

    /// **Every cluster boundary, by construction.** `len() + 1` entries, `0` first and
    /// `text().len()` last.
    pub fn boundaries(&self) -> &[usize] {
        &self.boundaries
    }

    /// The column each boundary sits at, in the same order. `len() + 1` entries.
    pub fn columns(&self) -> &[u16] {
        &self.columns
    }

    /// How many clusters. **167**, where 166 was recorded — see this module's header.
    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    /// Whether it holds nothing. It never does; the method is here because clippy asks for it
    /// beside [`Corpus::len`] and a corpus that could be empty is a corpus a gate can pass over.
    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }

    /// How many code points. **A char caret takes exactly this many steps to cross the corpus.**
    pub fn chars(&self) -> usize {
        self.text.chars().count()
    }

    /// **How many of those steps land inside a cluster**, which is [`Corpus::chars`] minus
    /// [`Corpus::len`].
    ///
    /// It was recorded as a third figure — *51 of 217*, beside *166* — and it is not a third
    /// measurement: a cluster of `k` code points is crossed in `k` char steps, `k − 1` of which land
    /// inside it, so `inside == chars − clusters` for **any** string at all. The relation reproduces
    /// exactly; the magnitudes are this corpus's and are reported as such.
    pub fn inside(&self) -> usize {
        self.chars() - self.len()
    }

    /// The corpus's width in columns.
    pub fn width(&self) -> u16 {
        self.columns[self.columns.len() - 1]
    }

    /// **Whether `byte` is a cluster boundary**, answered from the construction and not from the
    /// tables. The first gate, as a question.
    pub fn is_boundary(&self, byte: usize) -> bool {
        self.boundaries.binary_search(&byte).is_ok()
    }

    /// The column of a boundary, or `None` when `byte` is not one.
    pub fn column_of(&self, byte: usize) -> Option<u16> {
        self.boundaries
            .binary_search(&byte)
            .ok()
            .map(|i| self.columns[i])
    }

    /// How many clusters of `kind` the corpus holds.
    pub fn count_of(&self, kind: Kind) -> usize {
        self.kinds.iter().filter(|k| **k == kind).count()
    }
}

/// **Build the corpus.** [`CYCLES`] turns of [`ROTA`], every word joined to the next by [`JOINER`].
///
/// Cheap enough to call from a test and deliberately not a `static`: a corpus behind a lock is a
/// corpus two tests can observe in two states if one of them ever grows a `&mut`.
pub fn corpus() -> Corpus {
    let mut clusters: Vec<&'static str> = Vec::new();
    let mut kinds: Vec<Kind> = Vec::new();
    for cycle in 0..CYCLES {
        for (index, word) in ROTA.iter().enumerate() {
            if !(cycle == 0 && index == 0) {
                clusters.push(JOINER);
                // A joining space is ASCII whatever it separates, and saying so keeps
                // `count_of(Kind::Ascii)` a number about the corpus rather than about the rota.
                kinds.push(Kind::Ascii);
            }
            for cluster in word.clusters {
                clusters.push(cluster);
                kinds.push(word.kind);
            }
        }
    }
    from_clusters(clusters, kinds)
}

/// **A corpus of nothing but ASCII**, which is what every gate here has to be watched reporting
/// nothing over.
///
/// The ticket's sentence is *every gate runs on clusters that are not one code point, not on ASCII*,
/// and a gate that fires on the real corpus has said nothing until the same gate has been seen
/// **not** firing on this one. Same shape, same length in words, one kind.
pub fn ascii_corpus() -> Corpus {
    let mut clusters: Vec<&'static str> = Vec::new();
    let mut kinds: Vec<Kind> = Vec::new();
    for cycle in 0..CYCLES {
        for index in 0..ROTA.len() {
            if !(cycle == 0 && index == 0) {
                clusters.push(JOINER);
                kinds.push(Kind::Ascii);
            }
            for cluster in ROTA[0].clusters {
                clusters.push(cluster);
                kinds.push(Kind::Ascii);
            }
        }
    }
    from_clusters(clusters, kinds)
}

/// A corpus over clusters a caller assembled. The three vectors below are derived, never given.
fn from_clusters(clusters: Vec<&'static str>, kinds: Vec<Kind>) -> Corpus {
    let mut text = String::new();
    let mut boundaries = vec![0usize];
    let mut columns = vec![0u16];
    for cluster in &clusters {
        text.push_str(cluster);
        boundaries.push(text.len());
        columns.push(width(&text));
    }
    Corpus {
        clusters,
        kinds,
        text,
        boundaries,
        columns,
    }
}

/// **The forward cluster step, in two probes.** `None` at the end of the string.
///
/// This is what `vitui_engine::graphemes` would answer in one call and what this crate cannot
/// reach — see the module header. The second probe is not a fallback for an unusual case: **every
/// cluster two columns wide takes it**, because a two-column cluster does not fit in one column and
/// `truncate` correctly returns nothing rather than half of it.
///
/// # It is wrong on a zero-width cluster, and that is a precondition rather than a bug
///
/// `truncate` is greedy and a zero-width cluster always fits, so the answer for `"\u{200B}x"` is
/// `"\u{200B}x"` — two clusters. Nothing in [`ROTA`] is zero columns wide and
/// `tests::no_cluster_in_the_corpus_is_zero_columns_wide` is what keeps it that way.
pub fn next_cluster(rest: &str) -> Option<&str> {
    if rest.is_empty() {
        return None;
    }
    let one = truncate(rest, 1);
    if one.is_empty() {
        // Two columns, which is the widest answer the engine's tables give.
        Some(truncate(rest, 2))
    } else {
        Some(one)
    }
}

/// **Every boundary of `s`, walked forward from byte 0 with [`next_cluster`].** Derivation 2.
///
/// O(clusters) probes and O(bytes) work, which is what *segmenting forward from one already
/// known* costs when the one already known is the start of the string. That it has to start
/// somewhere is the whole of the index argument.
pub fn boundaries_by_probe(s: &str) -> Vec<usize> {
    let mut out = vec![0usize];
    let mut at = 0usize;
    while let Some(cluster) = next_cluster(&s[at..]) {
        at += cluster.len();
        out.push(at);
    }
    out
}

/// **Every boundary of `s`, swept by column with `truncate`.** Derivation 3.
///
/// Exact only where no cluster is zero columns wide: two of those share a column and this sweep
/// keeps the later one. See the module header.
pub fn boundaries_by_truncate(s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for w in 0..=width(s) {
        let len = truncate(s, w).len();
        if out.last() != Some(&len) {
            out.push(len);
        }
    }
    if out.last() != Some(&s.len()) {
        out.push(s.len());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every named kind is in the corpus, and none of them is there once by accident.**
    ///
    /// The ticket's own sentence is *the corpus is not optional and is not sampled*, and this is the
    /// form that survives a later ticket deciding one kind is inconvenient.
    #[test]
    fn the_corpus_carries_every_kind_section_eleven_names() {
        let corpus = corpus();
        for kind in Kind::ALL {
            let n = corpus.count_of(kind);
            assert!(
                n >= CYCLES,
                "the corpus carries {n} clusters of kind `{}` and the rota puts one word of it in \
                 each of {CYCLES} cycles",
                kind.word()
            );
        }
        assert_eq!(
            Kind::ALL.iter().filter(|k| k.named_by_spec()).count(),
            7,
            "spec §11 names seven kinds; the eighth is the ASCII the wrap needs"
        );

        // **And every kind is checked against its code points rather than against its label.**
        // Counting the `kinds` vector alone is a vacuous gate and it *was* one: the ZWJ family's
        // word could be replaced with three ASCII letters and the corpus went on reporting seven
        // ZWJ clusters. See [`Kind::describes`].
        for word in ROTA {
            let carrier = word
                .clusters
                .iter()
                .filter(|c| word.kind.describes(c))
                .count();
            assert!(
                carrier >= 1,
                "no cluster of the `{}` word is actually a `{}` — the label is the only thing \
                 saying so",
                word.clusters.concat(),
                word.kind.word()
            );
        }

        // The other direction, on the substitution that used to pass: three ASCII letters are not
        // a ZWJ family, whatever the row beside them says.
        assert!(!Kind::ZwjFamily.describes("f"));
        assert!(!Kind::Cjk.describes("a"));
        assert!(!Kind::Acute.describes("e"));
        assert!(Kind::Acute.describes("e\u{301}"));
    }

    /// **No cluster in the corpus is zero columns wide**, which is [`next_cluster`]'s precondition
    /// and [`boundaries_by_truncate`]'s at the same time.
    ///
    /// Watched failing on the case that breaks it: a zero-width space comes back welded to the
    /// cluster after it, which is two clusters in one answer.
    #[test]
    fn no_cluster_in_the_corpus_is_zero_columns_wide() {
        let corpus = corpus();
        for cluster in corpus.clusters() {
            assert!(
                width(cluster) >= 1,
                "{cluster:?} is zero columns wide, so the two-probe cluster step is wrong on it"
            );
            assert!(
                width(cluster) <= 2,
                "{cluster:?} is {} columns wide and two probes are no longer exhaustive",
                width(cluster)
            );
        }

        // The other direction, on a cluster that is not in the corpus: the probe over-reads.
        let hostile = "\u{200B}x";
        assert_eq!(width("\u{200B}"), 0);
        assert_eq!(
            next_cluster(hostile),
            Some(hostile),
            "a zero-width cluster arrives welded to what follows it, which is the precondition"
        );
    }

    /// **The boundary set, three ways: by construction, by the forward probe, by the width sweep.**
    ///
    /// Two of the three go through the engine's tables and one does not, so a disagreement names
    /// which side moved.
    #[test]
    fn the_three_derivations_of_the_boundary_set_agree() {
        let corpus = corpus();
        let built = corpus.boundaries().to_vec();
        let probed = boundaries_by_probe(corpus.text());
        let swept = boundaries_by_truncate(corpus.text());
        assert_eq!(
            built, probed,
            "the forward probe and the construction disagree"
        );
        assert_eq!(
            built, swept,
            "the width sweep and the construction disagree"
        );
        assert_eq!(built.len(), corpus.len() + 1);
        assert_eq!(built[0], 0);
        assert_eq!(built[built.len() - 1], corpus.text().len());
    }

    /// **The column of every boundary is the engine's tables over the prefix.** The second gate,
    /// checked over the corpus that is the corpus rather than over a sample.
    #[test]
    fn every_boundary_column_is_the_engines_tables_over_the_prefix() {
        let corpus = corpus();
        for (i, byte) in corpus.boundaries().iter().enumerate() {
            assert_eq!(
                corpus.columns()[i],
                width(&corpus.text()[..*byte]),
                "boundary {i} at byte {byte}"
            );
        }
    }

    /// **`inside == chars − clusters` is arithmetic, and it reproduces exactly.**
    ///
    /// *51 of 217* reads as a third measurement beside *166*. It is not one: it holds for
    /// every string, and the case below is a two-cluster string with the answer worked out by hand.
    #[test]
    fn the_inside_landings_are_arithmetic_over_the_other_two_figures() {
        let corpus = corpus();
        assert_eq!(corpus.inside(), corpus.chars() - corpus.len());

        // `e` + `U+0301` is one cluster of two code points; `a` is one of one. Three char steps
        // cross two clusters and one of the three lands inside.
        let small = from_clusters(vec!["e\u{301}", "a"], vec![Kind::Acute, Kind::Ascii]);
        assert_eq!(small.len(), 2);
        assert_eq!(small.chars(), 3);
        assert_eq!(small.inside(), 1);
    }
}
