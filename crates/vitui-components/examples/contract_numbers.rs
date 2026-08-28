//! **O4's report: what every component declares, what the machine takes, and where they differ.**
//!
//! `crate::contract` is the value and `crate::obligations::o4` is the gate; this prints the table
//! neither of them can, because a count of disagreements does not say which chord.
//!
//! ```sh
//! cargo run --example contract_numbers -p vitui-components
//! ```

use std::collections::BTreeSet;
use vitui_components::contract::{
    ABSENT, CONTRACTS, CORPUS_SIZE, PICKER_IS_MISSING, RULER_REMOVES, RUNTIMES_SHARE, SINGLE_KEEPS,
    Trigger, at_ruler, at_single, control, corpus, help,
};
use vitui_runtime::Mods;

fn main() {
    println!("O4 — the declared keyboard contract\n");
    println!("the sweep asks about {CORPUS_SIZE} triggers per component\n");
    println!(
        "{:<14} {:>4} {:>4} {:>7} {:>9}  the difference",
        "component", "docd", "regd", "missing", "undecl"
    );
    let mut total_missing = 0usize;
    let mut total_undeclared = 0usize;
    for c in CONTRACTS {
        // **The same value the gate asserts on**, so the report cannot derive the difference its own
        // way and answer a different question.
        let d = c.disagreement();
        total_missing += d.dead.len();
        total_undeclared += d.undeclared.len();
        let mut note = String::new();
        if !d.dead.is_empty() {
            note.push_str(&format!("declared, dead: {:?} ", d.dead));
        }
        if !d.undeclared.is_empty() {
            note.push_str(&format!("taken, undeclared: {:?}", d.undeclared));
        }
        println!(
            "{:<14} {:>4} {:>4} {:>7} {:>9}  {note}",
            c.id,
            c.documented().len(),
            c.registered().len(),
            d.dead.len(),
            d.undeclared.len()
        );
    }
    println!("\n{total_missing} declared and dead, {total_undeclared} taken and undeclared");

    let free: Vec<String> = corpus()
        .into_iter()
        .filter(|t| control(*t))
        .map(Trigger::spell)
        .collect();
    println!(
        "\nwhat a focusable gets for free, subtracted from all thirteen: {} — {free:?}",
        RUNTIMES_SHARE
    );
    let ruler: BTreeSet<String> = corpus()
        .into_iter()
        .filter(|t| at_ruler(*t) && !control(*t))
        .map(Trigger::spell)
        .collect();
    let whole: BTreeSet<String> = CONTRACTS
        .iter()
        .find(|c| c.id == "field")
        .expect("`field` declares a contract")
        .registered()
        .into_iter()
        .collect();
    let gone: Vec<&String> = whole.difference(&ruler).collect();
    println!("what `WrapKind::Ruler` takes off a field: {RULER_REMOVES} spellings — {gone:?}");
    let kept: Vec<String> = [Mods::CTRL, Mods::SHIFT, Mods::CTRL.with(Mods::SHIFT)]
        .into_iter()
        .filter(|m| at_single(Trigger::Click(*m)))
        .map(|m| Trigger::Click(m).spell())
        .collect();
    println!("at `Mode::Single` the pointer half keeps {kept:?}");
    println!(
        "what a `file_picker`'s open popup does not answer that a `select`'s does: \
         {PICKER_IS_MISSING} — components architecture issue 23"
    );
    println!(
        "what survives a collection's pointer half at `Mode::Single`: {SINGLE_KEEPS} of 3, and the \
         one that disappears is the one that works"
    );

    println!("\nthe help, as it renders:\n");
    for c in CONTRACTS {
        println!("  {}", c.id);
        for line in help(c) {
            println!("    {line}");
        }
    }

    println!("\nabsent, and the fact that makes it absent:\n");
    for a in ABSENT {
        println!("  {} — {:?}", a.what, a.spellings);
        println!("    {}", a.because);
    }
}
