# Vendored Unicode Character Database — 17.0.0

`build.rs` generates the engine's three Unicode tables from these files. They are vendored rather
than fetched because a build must not need the network, and because the table a release ships has
to be reproducible from the tree it was built from.

`deny.toml` bans `unicode-width` and `unicode-segmentation`. This directory is what replaces them.

## What each file is read for

| file | property | why the engine needs it |
|---|---|---|
| `GraphemeBreakProperty.txt` | `Grapheme_Cluster_Break` | GB3–GB8, GB9–GB9b, GB12–GB13 |
| `DerivedCoreProperties.txt` | `Indic_Conjunct_Break` | GB9c, added in Unicode 15.1 — a separate property, not a `Grapheme_Cluster_Break` value |
| `emoji-data.txt` | `Extended_Pictographic`, `Emoji`, `Emoji_Presentation` | GB11, the VS16 rule, and default-emoji width |
| `EastAsianWidth.txt` | `East_Asian_Width` | the column count of a code point |
| `DerivedGeneralCategory.txt` | `General_Category` | `Mn`, `Me`, `Cf`, `Cc` — what occupies no column |
| `GraphemeBreakTest.txt` | — | the conformance suite. Test data, not table input |

Only `DerivedCoreProperties.txt` is read for a single property out of many; it is kept whole rather
than filtered so that what is in the tree is a UCD file and not an edit of one.

## The version is checked, not assumed

`build.rs` states `UCD_VERSION` once and fails the build unless every file's own header declares
it. A half-updated directory is a build failure rather than a table that silently mixes two
Unicode releases. `emoji-data.txt` is the one that stamps `# Version: 17.0` where the others use
`<Name>-17.0.0.txt`.

## Re-downloading

```sh
base=https://www.unicode.org/Public/17.0.0/ucd
curl -O "$base/auxiliary/GraphemeBreakProperty.txt" \
     -O "$base/auxiliary/GraphemeBreakTest.txt" \
     -O "$base/EastAsianWidth.txt" \
     -O "$base/emoji/emoji-data.txt" \
     -O "$base/DerivedCoreProperties.txt" \
     -O "$base/extracted/DerivedGeneralCategory.txt"
shasum -a 256 -c SHA256SUMS
```

`SHA256SUMS` is **for that command and nothing else** — it answers "is what I just downloaded what
this tree was built from". It is deliberately not a gate, and no build step or test verifies it:
these files are committed, so git already content-addresses them, and a sums file living in the
same tree cannot detect a tamper that edits both. Making it a gate would mean a hand-written
SHA-256 in `build.rs` — no crate may enter the graph — bought against a threat git already
covers.

Updating the UCD moves the table sizes, and the sizes are asserted in `src/ucd/tests.rs` so the
move lands in a diff. Update them in the same commit; if one of them jumps, that is the finding.

## Terms of use

These files are © Unicode, Inc. and are redistributed under the Unicode licence:
<https://www.unicode.org/terms_of_use.html>.
