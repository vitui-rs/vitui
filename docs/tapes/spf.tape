# spf — superfile, drawn on the vitui component surface.
#
#     cargo build --release -p vitui-apps --example spf
#     docs/tapes/render.sh docs/tapes/spf.tape
#
# Pinned the same way as every other tape here, and `Hide`/`Show` keeps the invoking command out of
# frame.

Output docs/img/spf.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Padding 0
Set Margin 0
Set Theme "Catppuccin Mocha"
Set TypingSpeed 110ms
Set Framerate 24

Hide
Type "target/release/examples/spf"
Enter
Sleep 2s
Show

Sleep 1500ms

# Down the panel, in superfile's own vi keys.
Type "jjjj"
Sleep 900ms

# Select mode, and extend over two rows.
Type "v"
Sleep 900ms
Type "JJ"
Sleep 1200ms

# Copy starts a process, which is what the process bar is for.
Ctrl+C
Sleep 1400ms

# Back to browser mode, then a second file panel: the ring walks over every pane.
Type "v"
Sleep 700ms
Type "n"
Sleep 1500ms
Tab
Sleep 1200ms

# The sidebar, which is a third pane and a third focus stop.
Type "s"
Sleep 1600ms

Hide
Ctrl+Q
Sleep 500ms
