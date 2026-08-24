# The vitui-runtime arm

Everything this arm decided that [`SCENES.md`](../../SCENES.md) and
[`ARM-CONTRACT.md`](../../ARM-CONTRACT.md) did not, and the three things it found about the layer it
measures. It is the fifth arm and the second of ours: [`arms/vitui`](../vitui) draws the same nine
pictures through `vitui-engine`, this one draws them through the facade, and the pair is the only
difference in [`REPORT.md`](../../REPORT.md) with exactly one known cause.

**Version: this workspace's `0.0.0`.** `vitui-runtime` at 20 of 21 tickets, through `crates/vitui`.

---

## The idiom: it redraws the whole picture, every frame, on all nine scenes

`arms/vitui` writes only the cells that changed. This one writes all of them. Neither is a choice
made for the benchmark — they are what ordinary code looks like at the two layers.

The engine's three verbs mark damage as they write, and a caller holding a `View` knows what it just
changed, so `status-line`'s thirty-nine static rows are written once on frame 0 and the status row
after that. The runtime has **no scene tree and no retained structure** (ADR 0012): the clip stack is
the call stack and the id path is the closure tree, so a widget that is on the screen on frame *n* is
a function that ran on frame *n*. There is nowhere for "already drawn" to live.

So the gap between the two columns is **what the equality filter costs to absorb an immediate-mode
redraw**, and `unchanged` is the scene that isolates it: 4 800 cells built, clipped, id-ed and
written 120 times, against a wire that carries nothing.

## `Paint` has no *leave it alone*, and four scenes ask for the terminal's default colours

A `Paint` comes from a `Theme` and cannot be constructed (ADR 0018 — a component names a role and can
never make a paint; the door is shut with a `compile_fail` pair). A `Theme` is thirteen concrete
colour pairs, and the one escape hatch, `Theme::custom(fg: Rgb, bg: Rgb)`, takes three channels and
three channels. **There is no argument that means *the terminal's own*.**

`SCENES.md` scenes 1, 2, 3 and 4 all say *in the terminal's default colours*. This arm therefore
names the two colours the suite has already declared for exactly this problem: scene 5 fixes them at
`rgb(192,192,192)` on `rgb(0,0,0)`, because no arm can read them from inside — that line exists
because our own arm's dim was a silent no-op until it was written down.

**That makes the picture the described one and it does not make the bytes the same**, and the second
half matters more than the first. A default-coloured cell costs `arms/vitui` no SGR at all and costs
this arm one, on every scene, at the truecolor tier. It is why `status-line` reads 22.4 against 52.4
there and **22.4 against 22.4** at `no-color`, where the serializer narrows the named colours back to
the default and the two arms become the same program.

**This is not a `cannot express`, and the reasoning is worth keeping.** It has the shape of one — a
scene asks for something the framework has no word for — and it was resolved as an encoding
difference because `SCENES.md` had already ruled: those two colours *are* the terminal's defaults for
measurement. An arm refusing after that ruling would take four rows off the table for a picture it
can draw, and **a missing row reads as a win in whichever direction it is missing**.

## Reverse video: the first version of this arm was a false win, and the numbers are how it was caught

A `Paint` being two colours makes `theme.custom(bg, fg)` look like reverse video, and at the
truecolor tier it *is* the described picture. At the `no-color` tier it is nothing at all: both
colours narrow to the terminal's default, the swap collapses, and the highlighted row comes out
identical to the rows around it.

What that produced was a number, not a visible defect:

| `no-color`, one frame | `arms/vitui` | this arm, swapping colours | this arm, `restyle` |
|---|---:|---:|---:|
| `status-line` | 22.4 | 18.4 | 22.4 |
| `list-scroll` | 572.3 | **40.0** | 572.3 |

**Forty bytes a frame against five hundred and seventy-two, between two arms running the same
compositor.** The scroll had stopped costing anything because the moving highlight had stopped
existing. ratatui and Textual keep theirs at that tier, since SGR 7 is an attribute and survives
having no colour, so the cell would have been read against three arms drawing a richer picture.

The fix is to ask for the attribute as an attribute: `Ctx::restyle` with
`Repaint { set: Repaint::REVERSE, .. }`, which is the engine's third verb surfaced through the
runtime and is a bit on the cell rather than a pair of colours. It costs one more verb over the same
rectangle and that cost is in the number.

**The general shape is the one this suite keeps meeting from the other side.** A missing row reads as
a win; so does a missing *element inside a row*, and that one has no blank cell to give it away. The
only thing that caught it was two arms of the same library disagreeing by 14× on a scene where they
should not have disagreed at all.

## The caret needs something focused, and the engine's does not

`--scene caret` written as `arms/vitui`'s with one call renamed emits **no caret on any frame**.
`Frame::settle_caret` drops it when nothing holds the keyboard, and the reason is a good one: *a
caret on a screen where no widget holds the keyboard is a lie about where typing goes.* The engine
has no such rule and could not have one — it does not know what focus is.

So the arm declares row 0 a tab stop and focuses it, which is what an application would do. It is the
framework's idiom rather than a workaround, and it costs this row an id, a hit-index entry, a focus
stop and a ring membership that `arms/vitui` does not pay. Both arms then read **9.5 bytes a frame**,
because what reaches the wire is one `?25h`/`?25l` pair either way.

## Scene 4 is the one place this arm pays per cell

`Theme::custom` builds a style rather than reading one, and its own documentation says a component
calling it per cell is paying per cell and should publish the census. This is the census: **4 800
calls a frame**, because a 24-bit ramp is not thirteen roles and no theme has a role for *the colour
at column c*.

It lands in the CPU column and not the byte column — the wire is the same wire, 96 408 against
96 420 — which is the honest place for it. It is the price of ADR 0018 on the one picture that is
nothing but colour.

## The modal is one call, and the anchor is arithmetic rather than an assertion

Scene 5's dim is `Scrim::shadow(Mix::FULL / 2)` on the overlay's own options, and the driver puts it
at `z - 1` over the whole screen. The dialog's rectangle is not written down: the anchor is a
full-width one-row rectangle at row 13 and the placement is *below, centred*, so `place` gives
`y = 14` and `x = (120 - 40) / 2 = 40` — exactly the rectangle `SCENES.md` fixes, arrived at through
the mechanism rather than asserted beside it.

`Scrim::shadow` and not `Scrim::for_theme`: the direction here is the *scene's* and not the theme's,
and a lift would be the described picture on no terminal at all.

## What this arm does not measure, and it is the same list as `arms/vitui`'s

The `latency` scene reads stdin on the app thread and calls `frame` inline, so the sample does not
cross the mailbox — same caveat, same reason (the input thread needs a tty and the harness gives
every arm a pipe), and the runtime adds nothing to it that a pipe can show.

And nothing here measures a **component**, because there are none: `vitui-components` is empty
scaffolding, and it comes into the graph through the facade without contributing a line. Every arm in
this suite is written directly against a rendering or layout layer, which is the only level at which
five of them are comparable at all, and it is not the level at which anybody writes an application.
