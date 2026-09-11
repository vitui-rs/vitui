# cluster — k9s, drawn on the vitui component surface.
#
#     cargo build --release -p vitui-apps --example cluster
#     docs/tapes/render.sh docs/tapes/cluster.tape
#
# Pinned the same way as every other tape here: shell, font size, grid, theme and typing speed, so
# the same tape makes the same picture on another machine. `Hide`/`Show` keeps the invoking command
# out of frame.

Output docs/img/cluster.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 400
Set Padding 0
Set Margin 0
Set Theme "Catppuccin Mocha"
Set TypingSpeed 110ms
Set Framerate 24

Hide
Type "target/release/examples/cluster"
Enter
Sleep 2s
Show

Sleep 1500ms

# Every namespace, which is the mnemonic row down the middle of the header. Nine pods rather than
# six, so the table fills.
Type "0"
Sleep 1400ms

# Down the table. The row under the cursor takes the cursor face; a marked row takes the mark.
Down@280ms 4
Sleep 700ms
Space
Sleep 350ms
Down@280ms 1
Space
Sleep 1100ms

# Sort by status, which reorders the table without touching the columns.
Type "S"
Sleep 1500ms

# The two wide columns k9s hides by default come back, and every column is re-solved.
Ctrl+W
Sleep 1600ms
Ctrl+W
Sleep 900ms

# The command prompt switches the resource, which replaces the table, its columns and its title.
Type ":deploy"
Sleep 700ms
Enter
Sleep 2000ms

Hide
Ctrl+Q
Sleep 500ms
