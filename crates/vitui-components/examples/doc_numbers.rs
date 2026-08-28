//! **Components ticket 36's report**: O1's two halves, the axis line joined against the freeze, and
//! the numbers the register's rows compress into a sentence.
//!
//! ```text
//! cargo run --release --example doc_numbers -p vitui-components
//! ```
//!
//! # What is here and what is not
//!
//! Nothing below is a gate. The gates are `crate::gates::REGISTER` rows 30 and 210–212, and this
//! file is cited beside them as a [`Report`](vitui_components::gates::Instrument::Report) and never
//! instead of one — R15's refinement 2, which components ticket 20 found a whole crate had been
//! breaking: `cargo test` does not run an example, so an `assert!` here is compiled by
//! `cargo clippy --all-targets` and evaluated by nobody.
//!
//! What the report is *for* is the three things those rows compress:
//!
//! - **the page table**, one row per built component, so that *every one of them carries an
//!   example* is twenty-eight lines rather than a count;
//! - **the axis join**, printed with the freeze's answer beside the page's, because the equality is
//!   the load-bearing half and thirteen of the twenty-eight state `none`;
//! - **the fence census**, which is where a `compile_fail` body stops being evidence: this crate
//!   carries fifteen of them and O1 counts none.

use vitui_components::doc::{
    Found, REFUSED, doc_tested, pages, refusals_missing, running_examples, survey,
};
use vitui_components::obligations::{DOC_TESTED, o1};
use vitui_components::{INVENTORY, Tier};

fn read(relative: &str) -> String {
    let path =
        std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every fence on a page, running or not, so the two counts can be printed side by side.
fn fences(doc: &str) -> usize {
    doc.lines().filter(|l| l.trim().starts_with("```")).count() / 2
}

fn main() {
    let pages = pages();
    let found: Vec<Found> = survey(read);

    println!("O1 — a rustdoc page with a compiled example, for every component");
    println!("================================================================\n");
    println!(
        "population       {} built of {} frozen  (the unbuilt one is `spinner`)",
        pages.len(),
        INVENTORY.len()
    );
    println!("verdict          {:?}\n", o1(DOC_TESTED));

    println!(
        "{:<20} {:<4} {:<5} {:<6} {:<34} freeze says",
        "component", "tier", "runs", "fences", "page says"
    );
    println!("{}", "-".repeat(104));
    let mut total_running = 0usize;
    let mut total_fences = 0usize;
    for (page, f) in pages.iter().zip(&found) {
        let source = read(&page.file);
        let doc = vitui_components::doc::doc_comment(&source, page.id).unwrap_or_default();
        let tier = INVENTORY
            .iter()
            .find(|c| c.id == page.id)
            .map_or(Tier::Three, |c| c.tier);
        let says = match f.stated.as_deref() {
            None => "— no line —".to_string(),
            Some([]) => "none".to_string(),
            Some(words) => words.join(", "),
        };
        let freeze = if page.axes.is_empty() {
            "none".to_string()
        } else {
            page.axis_words().join(", ")
        };
        total_running += running_examples(&doc);
        total_fences += fences(&doc);
        println!(
            "{:<20} {:<4} {:<5} {:<6} {:<34} {}",
            page.id,
            match tier {
                Tier::One => "1",
                Tier::Two => "2",
                Tier::Three => "3",
            },
            running_examples(&doc),
            fences(&doc),
            says,
            freeze
        );
    }
    println!("{}", "-".repeat(104));
    println!(
        "{:<20} {:<4} {:<5} {}\n",
        "total", "", total_running, total_fences
    );

    // **The census the count is blind to on purpose**, and the finding inside it. A `compile_fail`
    // body has proved that a spelling does **not** compile, which is exactly not the claim O1
    // makes — and on this crate the exclusion costs nothing, because not one of the fifteen sits on
    // a component's page. They live on `WhyThereIsNo…` items of their own, which is this crate's
    // shape for a refusal, so the rule and the population never meet. That is worth printing rather
    // than concluding from a zero: the rule is right and this crate is not the evidence for it.
    let hostile_on_pages = total_fences - total_running;
    let files: std::collections::BTreeSet<String> = pages.iter().map(|p| p.file.clone()).collect();
    let per_file: Vec<(String, usize)> = files
        .iter()
        .map(|f| (f.clone(), read(f).matches("```compile_fail").count()))
        .filter(|(_, n)| *n > 0)
        .collect();
    let hostile_in_those_files: usize = per_file.iter().map(|(_, n)| n).sum();
    println!("fences on the twenty-eight pages   {total_fences}");
    println!("of them, doctests rustdoc runs     {total_running}");
    println!("of them, hostile or unrun          {hostile_on_pages}");
    println!(
        "the pages live in                  {} files, {} of which carry a hostile fence",
        files.len(),
        per_file.len()
    );
    println!(
        "hostile fences in those files      {hostile_in_those_files}   (not one of them on a page)"
    );
    for (file, n) in &per_file {
        println!("  {n}  {file}");
    }
    println!();

    println!(
        "axes stated `none`                 {}",
        pages.iter().filter(|p| p.axes.is_empty()).count()
    );
    println!(
        "axes stated, at least one          {}",
        pages.iter().filter(|p| !p.axes.is_empty()).count()
    );
    println!(
        "pages and the freeze disagreeing   {}\n",
        pages
            .iter()
            .zip(&found)
            .filter(|(p, f)| !f.agrees_about_axes(&p.axis_words()))
            .count()
    );

    println!("the written list and the scan");
    println!("  DOC_TESTED       {} ids", DOC_TESTED.len());
    println!("  crate::doc scan  {} ids", doc_tested(read).len());
    println!(
        "  agree            {}\n",
        DOC_TESTED.to_vec() == doc_tested(read)
    );

    println!("refusals owed a doc line           {}", REFUSED.len());
    for r in REFUSED {
        println!("  {:<28} {}", r.what, r.file);
    }
    println!(
        "  missing                          {:?}",
        refusals_missing(read)
    );
}
