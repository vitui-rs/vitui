//! **`caps` — what this terminal answered, and nothing else.**
//!
//! ```text
//! cargo run -p vitui-apps --example caps
//! ```
//!
//! # Why an application and not a script
//!
//! Every gate in this workspace is headless. `Driver::headless` and `runner::driver_at` compare what
//! the drawing verbs *say*; `conform/` compares bytes against captures already committed. Neither
//! can answer the one question a person holding a broken screen has: **what did my terminal claim,
//! and what did the engine decide because of it.**
//!
//! Detection is levels 1–5 of engine spec §10, and level 5 — [`quirks`] — is *field work*: one
//! terminal, one version range, one observed misbehaviour, none of it establishable from a document.
//! This is the instrument that makes a report about somebody else's terminal possible at all, and it
//! is deliberately the smallest application here: attach, read `Capabilities::report`, detach, print.
//! **It draws no frame**, so nothing it prints can be a consequence of anything a component did.
//!
//! # What to look at first
//!
//! `legacy_sgr`. The engine writes truecolour as `SGR 38:2::r:g:b` — the ITU-T T.416 colon form —
//! and a terminal whose parser only handles the semicolon form does not merely lose the colour: a
//! parser that abandons the sequence mid-way emits the remainder **as text**, which puts runs of
//! `:` and digits on the screen and pushes everything after them sideways. Three terminals are in
//! the quirk table for exactly this, recognised by their environment rather than by a query, because
//! it is not a thing a terminal will admit to.
//!
//! `VITUI_FORCE_LEGACY_SGR=1` is the lever that settles it in one run. If a screen that was garbage
//! comes out clean under it, the answer is a sixth row in that table and not a change to any
//! component.
//!
//! [`quirks`]: https://docs.rs/vitui-engine

use std::env;

use vitui_runtime::ctx::Driver;

fn main() {
    // **Attach and detach without drawing.** `Driver::attach` is what performs the live capability
    // negotiation (ADR 0007), so the report has to come from a real attach — and a frame would put
    // component behaviour between the terminal and the answer.
    let report = match Driver::attach(Default::default(), Default::default()) {
        Ok(driver) => driver.env().caps().report(),
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            eprintln!();
            eprintln!("That is itself the answer: the terminal did not complete the negotiation.");
            return;
        }
    };
    // `driver` is dropped above, so the terminal is back before a word is printed — `Screen`'s own
    // `Drop` is the restoration and this is what it is for.
    print!("{report}");

    println!("\nthe environment detection reads, and nothing else:");
    for key in [
        "TERM",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "COLORTERM",
        "TERMINAL_EMULATOR",
        "TERMUX_VERSION",
        "TMUX",
        "NO_COLOR",
        "VITUI_FORCE_LEGACY_SGR",
        "VITUI_FORCE_COLOR",
        "VITUI_GLYPHS",
    ] {
        match env::var(key) {
            Ok(v) if !v.is_empty() => println!("  {key:<22} {v}"),
            _ => println!("  {key:<22} —"),
        }
    }
}
