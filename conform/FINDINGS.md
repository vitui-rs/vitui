# Findings

Hand-written and dated, because a number and what it means are two different artefacts with two
different lifetimes. `REPORT.md` is generated; this is not.

## 2026-08-22 — stage 0, and the capture format is the finding

The instrument does not exist yet. What exists is the answer to *what can a dump-based capture see*,
and it is narrower than the design assumed — narrow enough that it changes how ticket 06's scene has
to be built.

### A grid-to-text dump cannot see a double-width glyph's second cell

Sent through tmux 3.7c, `capture-pane -p -e`, pane 20×6:

```
printf 'AB\346\274\242CD\r\n'      →  41 42 e6 bc a2 43 44 0a
                                       A  B  漢          C  D  \n
```

**No padding cell, no continuation marker, nothing.** On screen `漢` occupies columns 2 and 3 and `C`
is at column 4; in the dump `C` follows the glyph immediately. To know that, a reader has to know `漢`
is double-width — **which is the thing under test.**

Ghostty 1.3.1's `write_screen_file:…,vt` was already observed to do the same. Two emulators from two
lineages, one behaviour, and it is not a quirk of either: **it is inherent to re-serialising a cell
grid as text.** So the design's prediction — *ticket 20 will not be settled by the dump* — is
confirmed and generalises from one arm to the whole *category* of arm.

The consequence is a design constraint rather than a disappointment, and it is written into
[production ticket 06](../.scratch/vitui-engine-production/issues/06-the-clip-edge-pair-decided-on-what-was-observed.md):
the scene must put a **unique ASCII sentinel in every cell that should be a continuation**, which
converts *how wide was that glyph* into *is the sentinel still there* — a question a text dump can
answer — or it must use CPR, where the emulator reports the column itself.

### tmux 3.7c parses the colon form, which makes two of tier 1 observed rather than inferred

| sent | returned | |
|---|---|---|
| `ESC[38:2::255:0:0m` | `ESC[38;2;255;0;0m` | exact channels |
| `ESC[1;4;58:2::0:0:255m` | `ESC[1;4m` `ESC[58;2;0;0;255m` | exact channels, attributes split off |
| `ESC[38:5:200m` | `ESC[38;5;200m` | exact index |

Both emulators tested so far **normalise to semicolons on output**, whatever spelling they were sent.
That is what makes the dump a usable measuring channel at all: a terminal that *mis*-parsed the colon
form would return channel values off by one, which is precisely the failure
`serial::emit_color`'s own comment names. Agreement on the normalised output is therefore evidence
about the *parse*, not about the spelling.

With [arch 23](../.scratch/vitui-engine-architecture/issues/23-the-two-sgr-spellings-and-which-one-is-the-default.md)'s
Ghostty observation, **two of tier 1's seven are now observed.** Five remain inference from libvaxis's
three quirk entries. Neither of the two is one of the three terminals those entries name, so nothing
here contradicts them.

### The reset is not one sequence, and a parser that assumes it is will be wrong

tmux closes a colour-only run with `ESC[39m` — default foreground, leaving other attributes alone —
and an attribute-bearing run with `ESC[0m`. Both appear in
`fixtures/tmux-3.7c-attrs-and-colours.vt`. A parser must treat the SGR stream as *state*, not as
paired delimiters. This is the kind of thing that reads as a detail until it silently mis-attributes a
style to the following cell.

### What is not yet known

- **Whether a tmux arm is worth what it costs.** `capture-pane` re-serialises *tmux's* grid, so this
  arm measures tmux's conformance and never the emulator behind it. That is legitimate — tmux is in
  §10's own tier-1 list and `quirks.rs` already cites its 1 s sync-flush limit — but it must never be
  read as a proxy, and `SCENES.md` says so.
- **The empty-capture failure mode has not been exercised.** `screen -X hardcopy` returned rc=0 and a
  zero-byte file during the design investigation, and the refusal that must catch it is unwritten. It
  is the first thing the parser needs, not the last.
- **Everything about the engine.** No engine bytes have been through this path yet: the public API is
  being changed by
  [production ticket 03](../.scratch/vitui-engine-production/issues/03-the-uri-travels-at-the-verb.md)
  as this is written, and a driver built against today's `Restyle` would not compile tomorrow.
