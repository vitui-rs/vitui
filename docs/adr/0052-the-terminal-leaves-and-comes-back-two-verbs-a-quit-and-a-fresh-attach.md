---
status: accepted
date: 2026-08-29
---

# The terminal leaves and comes back: two verbs, a quit, and a fresh `attach`

Spec §15 filed *terminal lifecycle* as fog, and it was the one fog entry that is ordinary user
behaviour rather than an exotic case: **people press Ctrl-Z, and ssh connections drop.** It is three
cases wearing one name, and the decision is that they get **three different answers** rather than one
mechanism stretched over all of them.

1. **The application gives the terminal up on purpose.** `Screen::suspend` and `Screen::resume`.
2. **The terminal goes away underneath the process.** The input thread raises `Wake::Quit`.
3. **A different terminal on the same process.** Drop the `Screen` and `attach` again.

Decided and carried out by production ticket 07. Spec §7 gains *the terminal leaves and comes back*,
§10's immutability sentence gains the cross-reference, §15's entry is struck, and the register grows
its twenty-ninth entry — the second that §14 could not have stated, because §14 enumerates properties
of a *frame* and this is a property of the **session**.

## Why the engine handles no signal, and why that turns out to be the right answer anyway

**It cannot.** `sigaction` is not reachable from safe Rust, std has no signal API, and this crate's
dependency policy is crossterm plus generated UCD tables (ADR 0001) with `#![forbid(unsafe_code)]` in
the crate root. There is nowhere to put a handler and no crate permitted to supply one.

That would be an uncomfortable place to be if Ctrl-Z were a signal here, and **it is not**. Raw mode
is `cfmakeraw`, which clears `ISIG` — measured on this machine on 2026-08-29 through a pty with
crossterm 0.29, `ISIG` true before `enable_raw_mode` and false after — so for as long as a `Screen` is
attached the byte `0x1a` arrives at the parser as `Ctrl+z` and no `SIGTSTP` is generated at all. The
ordinary gesture is therefore already the application's, on the app thread, in a place where it knows
what it was doing: suspend, stop the process, resume, with the resume on the line *after* the stop
because a stopped process resumes exactly where it stopped. **No handler is needed for the case that
actually happens.**

What is left uncovered is an **external** stop — `kill -TSTP` from another terminal. The default
action stops the process where it stands with nothing to run, so the shell that takes the foreground
gets it inside the alternate screen. That is stated in §7 rather than mitigated, because the
mitigation is the handler that cannot exist.

## The suspension is not a stall, and the detector has to be told twice

§11's in-loop overrun detector aborts the process when the app thread's iteration stays open past the
threshold, and the iteration `wait` opened is open for as long as somebody else has the terminal. So a
suspend stops it. **And a resume has to start it again explicitly**, because `Watch`'s stop flag lives
behind the `Arc` every observer thread holds: a plain second `Perf::observe` spawns a thread that
reads the flag on its first poll and returns.

That failure is invisible from both ends — a detector that does nothing is what a healthy one looks
like, and the leaked thread exits by itself — which is why it is a gate with both halves watched
failing rather than a line of code with a comment on it.

## The reader does not stop, and that narrows what case 1 promises

The thread that reads the terminal is parked in a blocking `read` on standard input, and **nothing in
safe Rust cancels one**. So a suspension does not vacate stdin, and the two callers are not equally
served:

- **Ctrl-Z works completely**, because `SIGTSTP` stops every thread of the process and the reader
  with it.
- **An editor in the same window, with this process still running, does not.** Two readers on one
  descriptor, and the kernel gives each byte to whichever it schedules.

This is stated rather than solved because there is nothing here to solve it with. What is decided is
the half that is decidable: a resume **drops everything the user typed** during the suspension and
keeps everything the terminal *became*. Delivering it would put the editor's whole session into the
application as several hundred keystrokes, on a screen that has just repainted; and dropping the
*resize* with it would be worse than either, because `Screen::next_event` is the only thing that
applies one to the surfaces and `present` refuses to composite while the surfaces and the
authoritative size disagree — a frame owed for ever, which is a hang.

A child that needs the keyboard needs its own standard input, or this process needs to be stopped
while it runs.

## Why two verbs rather than one taking a closure

`screen.suspended(|| run_the_editor())` reads better and is not available: `no_public_verb_takes_a_closure_or_an_iterator`
is a gate over the whole public surface, and it is a gate for reasons that have nothing to do with
this pair. Two verbs it is, and they are idempotent — a key handler is where they are called from,
and *did I already do this* is genuinely hard to know there.

## What a resume does not do, and it is the load-bearing half

**It asks the terminal nothing.** `Capabilities` is sampled once and immutable for the life of a
`Screen` (§10), and a resume re-declares rather than re-detects. That is right for the case the pair
exists for — a process that suspends itself is `fg`'d back into the window it left, byte for byte the
same terminal — and it is *wrong* for a terminal that was replaced. The two are told apart by the
caller, because nothing on this side can tell them apart at all: a reconnected `ssh` session looks
like a resume from in here.

So case 3's answer is a fresh `attach` on a dropped `Screen`, which costs the whole session — the
layers, their cells, the interned clusters and the mirror — plus a detection round trip whose ceiling
is 250 ms. **That it works at all was a claim nobody had checked**, and it is not obvious: `shutdown`
installs a process-global panic hook and holds its site in a `static`, so a second session arms a
second site over the first. It is gated now.

**It does not re-sample the size either.** The authoritative size belongs to the input thread, and the
app thread agreeing with it is how a second resize gets the *older* size put back. A terminal that
changed size while somebody else had it arrives as an `Event::Resize` on the next read.

## Why the dropped connection is a `Wake::Quit` and not a new spelling

Every write fails and `write_frame` discards the error, because the frame path has no `Result` in it
by design (ADR 0022). So `present` answers `submitted: true` for ever and an application parked in
`Screen::wait` waits on a keyboard that cannot send another byte: **nothing anywhere says the terminal
is gone.** A hang is worse than a failure.

`Wake::Quit` rather than a fifth variant, and it is the same argument `WakeSource::renderer_gone`
makes one file over: an application that handles quit already does the right thing, and one that does
not was going to hang either way. The inference is sound because `Tty::open` hands a reader over only
when standard input *and* standard output are both a terminal — so an end-of-file there is the pty's
far side closing and not a redirect, an empty file or a supervisor holding stdin.

## The alternatives, and why each lost

- **Do nothing and document it.** Legitimate, and it was the ticket's own stated option. It loses on
  what it costs to say no: the epilogue is idempotent and already written, the negotiation is already
  written, and the repaint is a verb the sweep already has — so the whole of case 1 is *when* those
  run and not *what* they are. Three fields and forty lines against an application that cannot give
  its terminal to an editor.
- **Re-detect on resume.** It is the obvious way to make case 3 work without a fresh `attach`, and it
  would put a capability tier change between two frames — which §10 exists to prevent, and which every
  component above this crate is written against. It also spends a round trip on the case that never
  needs one.
- **A `Suspended` guard type whose `Drop` resumes.** Rejected for the same reason the closure form
  was, plus one of its own: a guard that resumes on unwind resumes into a terminal a panic is about to
  restore anyway, and the ordering between the two is a race nobody would ever see fail.
