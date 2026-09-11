# commander — GNU Midnight Commander on the vitui component surface.
#
# Run from the repository root, after:
#     cargo build --release -p vitui-apps --example commander
#
#     vhs docs/tapes/commander.tape
#
# Everything here is pinned so the same tape produces the same picture on another machine: the
# shell, the font size, the grid, the theme and the typing speed. `Hide`/`Show` keeps the invoking
# command out of frame, so the recording opens on the application rather than on a prompt.
#
# **No function key is pressed, and that is a constraint rather than a choice.** VHS can send
# arrows, Tab, Enter, Insert, Escape, Ctrl chords and text, and has no command for F1-F10 — so the
# sequence below is built from what `commander` binds *besides* its function bar: the cursor, the
# panel swap, a mark, a descent, and a letter reaching the shell prompt past a focused collection.

Output docs/img/commander.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 450
Set Padding 0
Set Margin 0
Set Theme "Catppuccin Mocha"
Set TypingSpeed 90ms
Set Framerate 24

Hide
Type "target/release/examples/commander"
Enter
Sleep 2s
Show

# The screen, before anything is touched.
Sleep 1600ms

# The cursor down the left panel. A row under the cursor takes the cursor face, so this is the
# component's own selection drawing rather than a repaint of the panel.
Down@300ms 5
Sleep 800ms

# Mark two entries. `Insert` is `mc`'s own Mark key, and a mark *replaces* the type colour rather
# than riding over it — the face ladder resolves before a cell is written.
Insert
Sleep 400ms
Down@300ms 1
Insert
Sleep 1000ms

# The other panel. Both panels are one call site drawing twice, so this is the keyed-identity path:
# without a key the second panel would take the first one's focus, cursor and hover.
Tab
Sleep 1000ms
Down@300ms 3
Sleep 1000ms
Tab
Sleep 800ms

# A letter typed over a focused collection reaches the shell prompt, which is the finding this
# application was written to expose: the collection takes the key into its type-ahead, answers
# `None` for every buffer, declines it, and it arrives in the unhandled window a frame later.
Type "cargo build --release"
Sleep 2200ms

Hide
Ctrl+Q
Sleep 500ms
