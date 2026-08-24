#!/usr/bin/env python3
"""The Textual arm of the vitui comparative suite.

Implements ../../ARM-CONTRACT.md for the scenes in ../../SCENES.md.

What the bytes on stdout are the bytes *of*: a real Textual `App`, run through
`App.run_async()` with Textual's real `LinuxDriver` — real alt screen prologue,
real stylesheet, real compositor, real `Strip`/`Segment` encoding, real writer
thread. The only thing swapped is the output sink: Textual's driver writes to
**stderr** (`sys.__stderr__`), and the contract wants the scene on stdout, so
the driver's file is stdout here. See NOTES.md.
"""

from __future__ import annotations

import os
import sys

_REEXEC_FLAG = "VITUI_ARM_TEXTUAL_REEXEC"

# Scene 4 is a 24-bit ramp.  Rich (and so Textual) picks 8-bit under
# TERM=xterm-256color unless COLORTERM says otherwise, which quantises the ramp
# and draws a *different picture* — adjacent columns collapse onto one colour.
# Textual's own override is used so the arm can draw the picture SCENES.md
# specifies; an explicit setting from the harness still wins.  Read at
# `import textual.constants`, so it has to be set before the import below.
os.environ.setdefault("TEXTUAL_COLOR_SYSTEM", "truecolor")


def _reexec_in_venv() -> None:
    """Run under this arm's pinned interpreter, whichever python3 invoked us."""
    try:
        import textual  # noqa: F401

        return
    except ImportError:
        pass
    if os.environ.get(_REEXEC_FLAG) == "1":
        sys.stderr.write(
            "arm=textual error=textual-not-importable "
            "hint=run-setup.sh-in-compare/arms/textual\n"
        )
        raise SystemExit(2)
    here = os.path.dirname(os.path.abspath(__file__))
    venv_python = os.path.join(here, ".venv", "bin", "python3")
    if os.path.exists(venv_python):
        os.environ[_REEXEC_FLAG] = "1"
        os.execv(venv_python, [venv_python, os.path.abspath(__file__), *sys.argv[1:]])
    sys.stderr.write(
        "arm=textual error=no-venv hint=run-setup.sh-in-compare/arms/textual\n"
    )
    raise SystemExit(2)


_reexec_in_venv()

import argparse  # noqa: E402
import asyncio  # noqa: E402
import selectors  # noqa: E402
from codecs import getincrementaldecoder  # noqa: E402
from time import perf_counter  # noqa: E402

from rich.cells import cell_len  # noqa: E402
from rich.segment import Segment  # noqa: E402
from rich.style import Style  # noqa: E402

from textual import events  # noqa: E402
from textual.app import App, ComposeResult  # noqa: E402
from textual.containers import Grid, Horizontal, Vertical  # noqa: E402
from textual.drivers.linux_driver import LinuxDriver  # noqa: E402
from textual.geometry import Region  # noqa: E402
from textual.screen import ModalScreen  # noqa: E402
from textual.strip import Strip  # noqa: E402
from textual.widget import Widget  # noqa: E402
from textual._xterm_parser import XTermParser  # noqa: E402

ARM_NAME = "textual"

# --------------------------------------------------------------------------
# Scene literals.  Every one of these is a *picture* decision; where SCENES.md
# left it open, the choice is recorded in NOTES.md so other arms can copy it.
# --------------------------------------------------------------------------

CARET_TEXT = "vitui compare — caret"
CARET_COLUMN = cell_len(CARET_TEXT)  # 21: the cell immediately after the final "t"
assert CARET_COLUMN == 21  # `#ctext { width: 21 }` in ArmApp.CSS depends on this

# 64 characters exactly, verbatim from SCENES.md.  This used to be the arm's
# own choice, because SCENES.md gave the string's length and not the string.
# The first run made that false -- under-specification 4 is this exact string,
# and SCENES.md now writes it out -- and the arm was not brought back into
# line, so it drew a different picture from the vitui arm on every scene with a
# body of text in it.  Found by replaying frame 0 of four arms into a grid.
LOREM_64 = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do ei"
assert len(LOREM_64) == 64

STATUS_FIELDS_TAIL = "    cpu 12%    3 tasks"

# Nineteen, which is what SCENES.md now says twice.  This was 20 on the reading
# that the prose beat the example; the first run settled it the other way
# (under-specification 5) and this arm kept the 20, so its list rows were one
# column wider than the vitui arm's on scenes 3, 5 and 9.
LIST_LABEL_WIDTH = 19
LIST_ROWS = 10_000
LIST_HIGHLIGHT_OFFSET = 12

SPINNER = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"

# Four rows of body text for the dialog; SCENES.md does not name them.
DIALOG_BODY = (
    "compositing a dialog over the list",
    "only the spinner cell moves per frame",
    "everything behind here is dimmed",
    "the list does not scroll",
)
assert all(len(line) <= 38 for line in DIALOG_BODY)  # 40 wide, minus the border

BLANK = Style()
REVERSE = Style(reverse=True)
DIM = Style(dim=True)

NAIVE_IDIOM = os.environ.get("VITUI_ARM_TEXTUAL_IDIOM", "tuned") == "naive"
"""When set, every frame refreshes the whole widget instead of the cells that
changed.  This is what typical Textual app code (a reactive plus
`Static.update()`) actually costs; the default is Textual hand-tuned.  NOTES.md
reports both."""


def terminal_size() -> tuple[int, int]:
    """120x40 from COLUMNS/LINES, per SCENES.md.  Never a silent 80x24."""
    try:
        columns = int(os.environ.get("COLUMNS", "") or 120)
    except ValueError:
        columns = 120
    try:
        lines = int(os.environ.get("LINES", "") or 40)
    except ValueError:
        lines = 40
    return columns, lines


def strip_of(segments: list[Segment], width: int) -> Strip:
    """Pad `segments` out to `width` cells in the terminal's default colours."""
    length = sum(cell_len(segment.text) for segment in segments)
    if length < width:
        segments = [*segments, Segment(" " * (width - length), BLANK)]
    return Strip(segments, max(length, width))


# --------------------------------------------------------------------------
# Drivers.  Textual's own LinuxDriver, with output moved to stdout.
# --------------------------------------------------------------------------


class _StdoutDriver(LinuxDriver):
    """Textual's real driver, writing the scene to stdout instead of stderr."""

    def __init__(self, app: App, **kwargs) -> None:
        super().__init__(app, **kwargs)
        # LinuxDriver.__init__ sets this to sys.__stderr__.  Captured before
        # Textual installs its own stdout redirection, so this is the real fd 1.
        self._file = sys.stdout

    def _get_terminal_size(self) -> tuple[int, int]:
        if self._size is not None:
            return self._size
        return terminal_size()


class FrameDriver(_StdoutDriver):
    """For the byte and CPU scenes: no input thread.

    Textual's own input thread busy-spins when stdin is a closed pipe (its
    selector reports the fd readable forever, `os.read` returns b"", the loop
    goes straight round again), which would burn a core and make the `cpu`
    scene meaningless.  stdin carries nothing on these scenes, so the thread
    just parks on the exit event.
    """

    def run_input_thread(self) -> None:
        self.exit_event.wait()


class LatencyDriver(_StdoutDriver):
    """For the latency scene: Textual's real stdin path, exiting at EOF.

    Same selector, same `XTermParser`, same `process_message` as
    `LinuxDriver.run_input_thread`; the one change is that an empty read is
    EOF and ends the app instead of spinning.
    """

    def run_input_thread(self) -> None:
        selector = selectors.SelectSelector()
        selector.register(self.fileno, selectors.EVENT_READ)
        parser = XTermParser(self._debug)
        decode = getincrementaldecoder("utf-8")().decode
        try:
            while not self.exit_event.is_set():
                for _key, mask in selector.select(0.1):
                    if mask & selectors.EVENT_READ:
                        data = os.read(self.fileno, 4096)
                        if not data:
                            self._loop.call_soon_threadsafe(self._app.exit)
                            return
                        for event in parser.feed(decode(data)):
                            self.process_message(event)
                for event in parser.tick():
                    self.process_message(event)
        finally:
            selector.close()


# --------------------------------------------------------------------------
# Widgets.  `render_line` is Textual's own idiom for cell-exact content
# (DataTable, OptionList, RichLog all use it) and is the only way to reach the
# pictures SCENES.md describes without a theme's colours leaking in.
# --------------------------------------------------------------------------


class Cells(Widget):
    """A widget whose lines come from a painter callback."""

    def __init__(self, painter, **kwargs) -> None:
        super().__init__(**kwargs)
        self._painter = painter

    def render_line(self, y: int) -> Strip:
        return self._painter(y, self.size.width)


# --------------------------------------------------------------------------
# Scenes
# --------------------------------------------------------------------------


class Scene:
    """A scene is a set of widgets plus `set_frame(n)`."""

    name = ""

    def __init__(self, width: int, height: int) -> None:
        self.width = width
        self.height = height
        self.frame = 0

    def compose(self) -> ComposeResult:  # pragma: no cover - overridden
        raise NotImplementedError

    async def on_ready(self, app: App) -> None:
        """Anything that has to happen after mount (pushing a screen)."""

    def set_frame(self, n: int) -> None:  # pragma: no cover - overridden
        raise NotImplementedError


def changed_span(before: str, after: str, width: int) -> Region | None:
    """The columns of row 0 in which two strings differ, as a 1-row Region."""
    a = before.ljust(width)[:width]
    b = after.ljust(width)[:width]
    if a == b:
        return None
    first = next(i for i in range(width) if a[i] != b[i])
    last = next(i for i in range(width - 1, -1, -1) if a[i] != b[i])
    return Region(first, 0, last - first + 1, 1)


class CaretScene(Scene):
    """Row 0 is two widgets: the label, and the caret cell.

    Textual's smallest possible update is one *chop*, and a chop runs from a
    widget's left edge to the right edge of the change (see
    `ChopsUpdate.render_segments`).  A one-cell change inside a 120-cell widget
    is therefore never a one-cell update; the only way to spend one cell is for
    the caret to be its own one-cell widget.  That is what this does, and it is
    Textual at its most favourable on this scene.  `VITUI_ARM_TEXTUAL_IDIOM=naive`
    selects the ordinary `Static.update()` shape instead.
    """

    name = "caret"

    def compose(self) -> ComposeResult:
        if NAIVE_IDIOM:
            self.widget = Cells(self._paint_row, id="row0-whole")
            yield self.widget
            return
        with Horizontal(id="row0"):
            yield Cells(self._paint_label, id="ctext")
            self.caret = Cells(self._paint_caret, id="ccaret")
            yield self.caret

    def _shown(self) -> bool:
        return self.frame % 2 == 0

    def _paint_label(self, y: int, width: int) -> Strip:
        return strip_of([Segment(CARET_TEXT, BLANK)], width)

    def _paint_caret(self, y: int, width: int) -> Strip:
        return Strip([Segment(" ", REVERSE if self._shown() else BLANK)], width)

    def _paint_row(self, y: int, width: int) -> Strip:
        if y:
            return strip_of([], width)
        caret = REVERSE if self._shown() else BLANK
        return strip_of([Segment(CARET_TEXT, BLANK), Segment(" ", caret)], width)

    def set_frame(self, n: int) -> None:
        self.frame = n
        if NAIVE_IDIOM:
            self.widget.refresh()
        else:
            self.caret.refresh()


class StatusLineScene(Scene):
    name = "status-line"

    def compose(self) -> ComposeResult:
        self.body = Cells(self._paint_body, id="body")
        self.status = Cells(self._paint_status, id="status")
        yield self.body
        yield self.status

    def _paint_body(self, y: int, width: int) -> Strip:
        if y >= self.height - 1:
            return strip_of([], width)
        return strip_of([Segment(f"line {y:02d}  {LOREM_64}", BLANK)], width)

    def status_text(self) -> str:
        return (
            f" frame {self.frame}"
            f"     elapsed {self.frame / 60:.2f}s"
            f"{STATUS_FIELDS_TAIL}"
        )

    def _paint_status(self, y: int, width: int) -> Strip:
        text = self.status_text()
        return Strip([Segment(text.ljust(width)[:width], REVERSE)], width)

    def _restatus(self, before: str) -> None:
        if NAIVE_IDIOM:
            self.status.refresh()
            return
        region = changed_span(before, self.status_text(), self.width)
        if region is not None:
            self.status.refresh(region)

    def set_frame(self, n: int) -> None:
        before = self.status_text()
        self.frame = n
        self._restatus(before)


class LatencyScene(StatusLineScene):
    """Scene 2's screen, with the first field driven by keystrokes."""

    name = "latency"

    def __init__(self, width: int, height: int) -> None:
        super().__init__(width, height)
        self.key_name: str | None = None

    def status_text(self) -> str:
        if self.key_name is None:
            return super().status_text()
        return (
            f" key {self.key_name}"
            f"     elapsed {self.frame / 60:.2f}s"
            f"{STATUS_FIELDS_TAIL}"
        )

    def set_key(self, key_name: str) -> None:
        before = self.status_text()
        self.key_name = key_name
        self._restatus(before)


class ListScrollScene(Scene):
    name = "list-scroll"

    def compose(self) -> ComposeResult:
        self.widget = Cells(self._paint)
        yield self.widget

    @staticmethod
    def row_text(index: int) -> str:
        label = f"item-{index:05d}".ljust(LIST_LABEL_WIDTH, "-")
        return f"{index:05d}  {label}"

    def _paint(self, y: int, width: int) -> Strip:
        index = self.frame + y
        if index >= LIST_ROWS:
            return strip_of([], width)
        style = REVERSE if y == LIST_HIGHLIGHT_OFFSET else BLANK
        return Strip([Segment(self.row_text(index).ljust(width), style)], width)

    def set_frame(self, n: int) -> None:
        self.frame = n
        self.widget.refresh()


class FullRepaintScene(Scene):
    name = "full-repaint"

    def __init__(self, width: int, height: int) -> None:
        super().__init__(width, height)
        # rgb(c*2, r*6, 128) for the ramp's frame-0 column c, cached per row.
        self._row_styles = [
            [
                Style.parse(f"rgb({(c * 2) % 256},{(r * 6) % 256},128)")
                for c in range(self.width)
            ]
            for r in range(self.height)
        ]

    def compose(self) -> ComposeResult:
        self.widget = Cells(self._paint)
        yield self.widget

    def _paint(self, y: int, width: int) -> Strip:
        styles = self._row_styles[y]
        shift = self.frame
        return Strip(
            [Segment("#", styles[(c + shift) % self.width]) for c in range(width)],
            width,
        )

    def set_frame(self, n: int) -> None:
        self.frame = n
        self.widget.refresh()


class DialogScreen(ModalScreen):
    """The undimmed 40x12 dialog, composited over the list screen.

    `align: center middle` in a 120x40 terminal puts a 40x12 box at column 40,
    row 14 — exactly where SCENES.md wants it.
    """

    def __init__(self, scene: "ModalOverListScene") -> None:
        super().__init__()
        self._scene = scene

    def compose(self) -> ComposeResult:
        scene = self._scene
        with Vertical(id="dialog"):
            scene.title = Cells(scene.paint_title, id="dtitle")
            yield scene.title
            yield Cells(scene.paint_body, id="dbody")


class ModalOverListScene(ListScrollScene):
    name = "modal-over-list"

    def __init__(self, width: int, height: int) -> None:
        super().__init__(width, height)
        self.spin = 0

    def _paint(self, y: int, width: int) -> Strip:
        index = y  # the list behind the dialog does not scroll
        style = REVERSE if y == LIST_HIGHLIGHT_OFFSET else BLANK
        return Strip([Segment(self.row_text(index).ljust(width), style + DIM)], width)

    async def on_ready(self, app: App) -> None:
        self.dialog_screen = DialogScreen(self)
        await app.push_screen(self.dialog_screen)

    def spinner_glyph(self) -> str:
        return SPINNER[self.spin % len(SPINNER)]

    def paint_title(self, y: int, width: int) -> Strip:
        return strip_of([Segment(f"{self.spinner_glyph()} working", BLANK)], width)

    def paint_body(self, y: int, width: int) -> Strip:
        return strip_of([Segment(DIALOG_BODY[y], BLANK)], width)

    def set_frame(self, n: int) -> None:
        self.frame = 0
        self.spin = n
        if NAIVE_IDIOM:
            self.title.refresh()
        else:
            self.title.refresh(Region(0, 0, 1, 1))


# --------------------------------------------------------------------------
# Scenes 6..9, from the runtime backlog.  SCENES.md explains why they are one
# list with the first five rather than a second suite.
# --------------------------------------------------------------------------


class UnchangedScene(StatusLineScene):
    """Scene 6.  Scene 2's screen, asked for again on every frame.

    `set_frame` deliberately does **not** return early.  The scene forbids
    skipping the draw, and it says so because the shortcut here is the one
    shortcut in the suite that leaves no trace in the output: the right answer
    is zero bytes either way.  So this asks Textual to repaint both widgets on
    every frame and lets its own machinery decide what reaches the wire.

    That makes this the naive idiom on purpose, and here the naive idiom is the
    honest one -- an application whose data has not changed still runs its
    render, because it does not know that.
    """

    name = "unchanged"

    def set_frame(self, n: int) -> None:
        self.frame = 0
        self.body.refresh()
        self.status.refresh()


class FadeScene(Scene):
    """Scene 7.  A 60x20 panel of `=` fading up from black.

    One widget covering the screen, and the tuned refresh is the panel's own
    region -- which is what a Textual author writing an animation would do, and
    is the same `refresh(Region(...))` idiom `CaretScene` and `StatusLineScene`
    already use.  `VITUI_ARM_TEXTUAL_IDIOM=naive` refreshes the whole widget.
    """

    name = "fade"
    LEFT = 30
    TOP = 10
    WIDTH = 60
    HEIGHT = 20

    def compose(self) -> ComposeResult:
        self.widget = Cells(self._paint)
        yield self.widget

    def _paint(self, y: int, width: int) -> Strip:
        if not (self.TOP <= y < self.TOP + self.HEIGHT):
            return strip_of([], width)
        level = min(self.frame * 2, 255)
        style = Style.parse(f"rgb({level},{level},{level}) on rgb(0,0,0)")
        return Strip(
            [
                Segment(" " * self.LEFT, BLANK),
                Segment("=" * self.WIDTH, style),
                Segment(" " * (width - self.LEFT - self.WIDTH), BLANK),
            ],
            width,
        )

    def set_frame(self, n: int) -> None:
        self.frame = n
        if NAIVE_IDIOM:
            self.widget.refresh()
        else:
            self.widget.refresh(Region(self.LEFT, self.TOP, self.WIDTH, self.HEIGHT))


class Panel(Widget):
    """One of scene 8's twelve panels: 30x13, two rows of text in it."""

    def __init__(self, index: int, **kwargs) -> None:
        super().__init__(**kwargs)
        self.index = index
        self.count = index

    def render_line(self, y: int) -> Strip:
        width = self.size.width
        if y == 0:
            return strip_of([Segment(f"panel {self.index:02d}", BLANK)], width)
        if y == 2:
            return strip_of([Segment(f"count {self.count:02d}", BLANK)], width)
        return strip_of([], width)

    def set_count(self, count: int) -> None:
        self.count = count
        if NAIVE_IDIOM:
            self.refresh()
        else:
            # A chop runs from the widget's left edge to the right edge of the
            # change, so the smallest update available is eight cells and not
            # two.  Twelve widgets rather than one full-width one is what buys
            # that: inside a 120-cell widget the same change is a 120-cell chop.
            self.refresh(Region(0, 2, 8, 1))


class ScatteredScene(Scene):
    """Scene 8.  Twelve panels, and one two-digit field in each of them moves."""

    name = "scattered"

    def compose(self) -> ComposeResult:
        self.panels = [Panel(k, classes="panel") for k in range(12)]
        with Grid(id="scatter"):
            yield from self.panels

    def set_frame(self, n: int) -> None:
        self.frame = n
        for k, panel in enumerate(self.panels):
            panel.set_count((k + n) % 100)


class FilterShrinkScene(Scene):
    """Scene 9.  A list that narrows and widens again, rows blanked as they go.

    The blank rows are painted as spaces rather than left out.  Textual composites
    from the widget's own strips, so a shorter strip is padded by
    `Strip.adjust_cell_length` and the picture is right either way -- but writing
    the spaces is what the other arms have to do and what makes this row about
    the wire rather than about four compositors' padding rules.
    """

    name = "filter-shrink"

    def compose(self) -> ComposeResult:
        self.body = Cells(self._paint_body, id="body")
        self.query = Cells(self._paint_query, id="status")
        yield self.body
        yield self.query

    def matches(self) -> int:
        return 39 - (self.frame % 39)

    def _paint_body(self, y: int, width: int) -> Strip:
        if y >= self.matches():
            return strip_of([], width)
        return Strip(
            [Segment(ListScrollScene.row_text(y).ljust(width), BLANK)], width
        )

    def _paint_query(self, y: int, width: int) -> Strip:
        text = f" search: {self.matches():02d} matches"
        return Strip([Segment(text.ljust(width)[:width], REVERSE)], width)

    def set_frame(self, n: int) -> None:
        before = self.matches()
        self.frame = n
        after = self.matches()
        if NAIVE_IDIOM or after > before:
            # Widening repaints rows that were blank, and they are anywhere from
            # one row to thirty-eight.  The whole body is the honest ask.
            self.body.refresh()
        else:
            # Narrowing: only the rows that stopped matching.
            self.body.refresh(Region(0, after, self.width, before - after))
        self.query.refresh(Region(0, 0, 20, 1))


SCENES = {
    scene.name: scene
    for scene in (
        CaretScene,
        StatusLineScene,
        ListScrollScene,
        FullRepaintScene,
        ModalOverListScene,
        UnchangedScene,
        FadeScene,
        ScatteredScene,
        FilterShrinkScene,
        LatencyScene,
    )
}


# --------------------------------------------------------------------------
# The app
# --------------------------------------------------------------------------


def key_label(key: str) -> str:
    """Textual's key name -> the scene's `key <name>` spelling."""
    if key.startswith("ctrl+"):
        rest = key[5:]
        return f"C-{rest if len(rest) == 1 else rest.capitalize()}"
    if len(key) == 1:
        return key
    return key.capitalize()


class ArmApp(App):
    """Nothing but the scene: no header, no footer, no bindings."""

    CSS = """
    Screen { background: transparent; }
    Cells { width: 100%; height: 100%; }
    #row0 { width: 100%; height: 1; }
    #row0-whole { width: 100%; height: 1; }
    #ctext { width: 21; height: 1; }
    #ccaret { width: 1; height: 1; }
    #body { width: 100%; height: 1fr; }
    #status { width: 100%; height: 1; dock: bottom; }

    /* scattered: four panels across, three down, each 30x13.  Twelve widgets
       and not one, because a chop runs from a widget's left edge -- inside one
       full-width widget a two-digit change is a 120-cell update. */
    #scatter {
        layout: grid; grid-size: 4 3; grid-columns: 30; grid-rows: 13;
        grid-gutter: 0; width: 120; height: 39; padding: 0;
    }
    .panel { width: 30; height: 13; }

    /* modal-over-list: `align: center middle` in 120x40 puts a 40x12 box at
       column 40, row 14 — exactly where SCENES.md wants it. */
    DialogScreen { align: center middle; background: transparent; }
    #dialog { width: 40; height: 12; border: solid ansi_default; padding: 0; }
    #dtitle { width: 100%; height: 1; }
    #dbody { width: 100%; height: 4; }
    """
    BINDINGS = []
    ENABLE_COMMAND_PALETTE = False
    AUTO_FOCUS = None

    def __init__(self, scene: Scene) -> None:
        super().__init__(driver_class=None, ansi_color=True)
        self.scene = scene
        self.displays = 0

    def compose(self) -> ComposeResult:
        yield from self.scene.compose()

    def post_display_hook(self) -> None:
        self.displays += 1

    async def on_key(self, event: events.Key) -> None:
        if isinstance(self.scene, LatencyScene):
            self.scene.set_key(key_label(event.key))


def build_app(scene: Scene, driver_class) -> ArmApp:
    app = ArmApp(scene)
    app.driver_class = driver_class
    # The terminal's default colours: the only Textual themes whose foreground
    # and background are `ansi_default`.
    app.theme = "ansi-dark"
    return app


# --------------------------------------------------------------------------
# Runners
# --------------------------------------------------------------------------


async def run_picture_scene(app: ArmApp, frames: int, size: tuple[int, int]) -> int:
    scene = app.scene

    async def autopilot(pilot) -> None:
        await scene.on_ready(app)
        await pilot.pause()  # frame 0: the initial screen
        for n in range(1, frames):
            scene.set_frame(n)
            await pilot.pause()  # forces exactly one composite
        app.exit()

    await app.run_async(headless=False, size=size, auto_pilot=autopilot)
    return app.displays


async def run_cpu_scene(app: ArmApp, seconds: float, size: tuple[int, int]) -> None:
    """Scene `caret` at 60 Hz against a wall-clock deadline, Textual's own timer."""
    scene = app.scene

    async def autopilot(pilot) -> None:
        deadline = perf_counter() + seconds
        frame = 0

        def tick() -> None:
            nonlocal frame
            if perf_counter() >= deadline:
                app.exit()
                return
            frame += 1
            scene.set_frame(frame)

        app.set_interval(1 / 60, tick)

    await app.run_async(headless=False, size=size, auto_pilot=autopilot)


async def run_latency_scene(app: ArmApp, size: tuple[int, int]) -> None:
    await app.run_async(headless=False, size=size)


# --------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("--scene", required=True, choices=[*SCENES, "cpu"])
    parser.add_argument("--frames", type=int, default=120)
    parser.add_argument("--seconds", type=float, default=10.0)
    args = parser.parse_args()

    size = terminal_size()
    no_color_declared = os.environ.get("NO_COLOR") is not None

    from textual import __version__ as textual_version

    if args.scene == "cpu":
        scene: Scene = CaretScene(*size)
        driver_class = FrameDriver
    else:
        scene = SCENES[args.scene](*size)
        driver_class = LatencyDriver if args.scene == "latency" else FrameDriver
    app = build_app(scene, driver_class)

    # Asked rather than assumed: `App.no_color` is Textual's own answer.
    if no_color_declared:
        no_color = "honoured" if app.no_color else "ignored"
    else:
        no_color = "honoured"

    # The one line the arm owes the harness.  sys.__stderr__, because Textual
    # replaces sys.stderr with its own capture for the duration of the run.
    sys.__stderr__.write(
        f"arm={ARM_NAME} version={textual_version} "
        f"no_color={no_color} alt_screen=yes\n"
    )
    sys.__stderr__.flush()

    if args.scene == "cpu":
        asyncio.run(run_cpu_scene(app, args.seconds, size))
    elif args.scene == "latency":
        asyncio.run(run_latency_scene(app, size))
    else:
        asyncio.run(run_picture_scene(app, args.frames, size))

    sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
