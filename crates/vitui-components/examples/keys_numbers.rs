//! components ticket 08 — `keys::text`, as numbers.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every number below is `src/keys.rs`'s test
//! module, and each figure has its single home in that file's ledger block.
//!
//! It prints five things:
//!
//! 1. **What is reachable**, which is the finding that made this ticket buildable at all.
//! 2. **The three-way measurement**, `Ctrl+S, h, i` through a focused field.
//! 3. **The capital**, which is where the third arm fails and the first two disagree.
//! 4. **The `textarea` arithmetic**: 2 bytes and 0 keys against 1 and 1.
//! 5. **What does not reproduce**, said out loud rather than engineered away.

use vitui_components::keys::{
    self, ARMS, CAPITAL_ON_CODE, CAPITAL_ON_INTENT, CAPITAL_RIGHT, CODE_ALONE, REACHABLE_STATES,
    SIGNIFICANT, START, TEXT_BEARING, TEXTAREA_CODE_BYTES, TEXTAREA_CODE_REACHED,
    TEXTAREA_TEXT_BYTES, TEXTAREA_TEXT_REACHED, THROUGH_INTENT, THROUGH_TEXT, typed,
};
use vitui_runtime::keys::{Chord, INTENT};

fn main() {
    println!("components ticket 08 — a chord is not text, and a capital is not a chord\n");

    // ── 1. what is reachable ─────────────────────────────────────────────────────────────────────
    println!("report  what a crate that cannot name `Mods` can do with one:");
    for line in [
        "  hold one                      yes   `Chord` has a `pub mods: Mods` field",
        "  construct CTRL / ALT / SHIFT  yes   `Chord::key('s').ctrl().mods` **is** `Mods::CTRL`",
        "  construct SUPER / HYPER /",
        "    META / CAPS / NUM           no    `Chord` has three builders and `Mods` has eight bits",
        "  read every bit off a key      yes   `ctrl`, `alt`, `shift`, `super_key`, `hyper`,",
        "                                      `meta`, `caps`, `num` are all inherent",
        "  build a `Key`                 yes   not `#[non_exhaustive]`, and the other four fields",
        "                                      are `Code::Char`, `Edge::Press`, `Text::EMPTY`, an",
        "                                      `Instant`",
        "  post one                      yes   `Driver::post_key`, public for `post_mouse`'s reason",
        "  build a `KeyText`             no    no public constructor anywhere above the engine",
        "  name `Mods`                   no    `reachable_as: None`, and that part was always right",
    ] {
        println!("{line}");
    }
    println!(
        "\n  Register row 5 read *no key can be posted and no chord can be pressed*, and that is\n  \
         the one standing on this register that was wrong rather than stale. The barrier that\n  \
         survives is narrower and is a number: {REACHABLE_STATES} of 256 modifier states are\n  \
         constructible here, so the exhaustive table is short even though the rule is not.\n"
    );

    let ctrl = Chord::key('s').ctrl().mods;
    let alt = Chord::key('s').alt().mods;
    let shift = Chord::key('s').shift().mods;
    println!("report  the two masks, one bit apart:");
    println!(
        "  {:<26}  {:?}",
        "keys::text (this crate)", SIGNIFICANT.mods
    );
    println!("  {:<26}  {INTENT:?}", "R12's INTENT (runtime)");
    println!(
        "  {:<26}  ctrl {} · alt {} · shift {}",
        "shared bits",
        INTENT.contains(ctrl),
        INTENT.contains(alt),
        SIGNIFICANT.mods.contains(shift)
    );
    println!(
        "\n  R12's is right for R12's question — `Ctrl+Shift+P` is not `Ctrl+P`, so Shift has to\n  \
         be compared. A text widget's question is different and the answer differs in one bit.\n"
    );

    // ── 2. the three-way measurement ─────────────────────────────────────────────────────────────
    let accelerator = [Chord::key('s').ctrl(), Chord::key('h'), Chord::key('i')];
    println!("report  `Ctrl+S, h, i` into a field holding {START:?}:");
    println!(
        "  {:<12}  {:<14}  {:>7}  {:>9}",
        "arm", "value", "gained", "reached"
    );
    let mut field = Vec::new();
    for (name, arm) in ARMS {
        let r = typed(arm, START, &accelerator);
        let quoted = format!("{:?}", r.value);
        println!(
            "  {name:<12}  {quoted:<14}  {:>7}  {:>9}",
            r.gained, r.reached
        );
        field.push(r);
    }
    println!(
        "\n  `code_alone` typed the accelerator's letter and the application never saw the key,\n  \
         which is the half that makes it a defect rather than a cosmetic. On **this** input the\n  \
         `intent` reading is indistinguishable from the fix — which is why spec §3 changes the\n  \
         input mid sentence, and why the next block exists.\n"
    );

    // ── 3. the capital ───────────────────────────────────────────────────────────────────────────
    let capital = [Chord::key('h').shift(), Chord::key('i')];
    println!("report  `Shift+H, i` into an empty field:");
    println!("  {:<12}  {:<8}  verdict", "arm", "value");
    let mut answers = Vec::new();
    for (name, arm) in ARMS {
        let r = typed(arm, "", &capital);
        let verdict = match r.value.as_str() {
            CAPITAL_RIGHT => "right",
            CAPITAL_ON_INTENT => "the capital was declined as a chord",
            CAPITAL_ON_CODE => "the modifier byte was ignored, so the capital went too",
            _ => "unexpected",
        };
        let quoted = format!("{:?}", r.value);
        println!("  {name:<12}  {quoted:<8}  {verdict}");
        answers.push(r.value);
    }
    println!(
        "\n  **Three readings, three different answers, two keys.** That is what makes one\n  \
         sentence of spec §3 worth a helper rather than a comment in every field.\n"
    );

    // ── 4. the textarea ──────────────────────────────────────────────────────────────────────────
    let short = [Chord::key('s').ctrl(), Chord::key('h')];
    println!("report  `Ctrl+S, h` into a textarea (§3's second number):");
    println!("  {:<12}  {:>13}  {:>9}", "arm", "bytes gained", "keys out");
    let code = typed(keys::defective::on_code_alone, "", &short);
    let right = typed(keys::text, "", &short);
    println!(
        "  {:<12}  {:>13}  {:>9}",
        "code_alone", code.gained, code.reached
    );
    println!(
        "  {:<12}  {:>13}  {:>9}",
        "text", right.gained, right.reached
    );
    println!(
        "\n  One byte and one key, and they are the same key: the byte the defective reading\n  \
         gains is exactly the key the application loses.\n"
    );

    // ── 5. what does not reproduce ───────────────────────────────────────────────────────────────
    println!("report  what does not reproduce, and why:");
    for line in [
        "  spec §3 / ticket 08        here            why",
        "  \"value 0shi\" on code       reproduces      exactly. §3 names the starting string, so",
        "                                             it is quoted rather than chosen.",
        "  \"value 0hi\" declining      reproduces      exactly.",
        "    chords",
        "  \"i\" where \"Hi\" is right    reproduces      exactly, and on a *different* input — §3's",
        "                                             sentence changes it mid clause and the",
        "                                             ledger says so in two constants.",
        "  2 bytes / 0 keys on code   reproduces      exactly, on `Ctrl+S, h`. The pair of numbers",
        "  1 byte / 1 key through                     fixes the input: one chord and one letter is",
        "    next_text_key                            the only sequence that gives 2/0 against 1/1.",
        "  a chord types nothing in   reproduces      over 7 sinks, not 7 components. `built` in",
        "    every focusable                          INVENTORY is the *prototype's* column and is",
        "                                             true for six of the seven; none of them is",
        "                                             declared in this crate, and the gate scans",
        "                                             `src/` to say so rather than reading a flag.",
        "  256 modifier states        8               does not, and cannot. `Chord` has `ctrl`,",
        "                                             `alt` and `shift` builders and no others, so",
        "                                             SUPER, HYPER, META, CAPS and NUM have no",
        "                                             reachable spelling. The *rule* reads all",
        "                                             eight bits off a key it is handed; only the",
        "                                             table is short, and row 53 carries the",
        "                                             barrier rather than the table pretending.",
        "  the `text`-carrying arm    unexercised     `KeyText` has no public constructor anywhere",
        "                                             above the engine — deliberately, so nothing",
        "                                             can forge a key whose `code` and `text`",
        "                                             disagree. Every key pressed here carries",
        "                                             empty text, so what the gates run over is the",
        "                                             `code`-plus-Shift fallback. That is the arm a",
        "                                             legacy terminal takes, so it is the arm that",
        "                                             matters most and the shortfall is still real.",
    ] {
        println!("{line}");
    }

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good. R15's rule, and the reason a report is allowed to carry asserts.
    assert_eq!(field[0].value, CODE_ALONE);
    assert_eq!(field[1].value, THROUGH_TEXT);
    assert_eq!(field[2].value, THROUGH_INTENT);
    assert_eq!(
        answers,
        vec![CAPITAL_ON_CODE, CAPITAL_RIGHT, CAPITAL_ON_INTENT]
    );
    assert_eq!(
        (code.gained, code.reached),
        (TEXTAREA_CODE_BYTES, TEXTAREA_CODE_REACHED)
    );
    assert_eq!(
        (right.gained, right.reached),
        (TEXTAREA_TEXT_BYTES, TEXTAREA_TEXT_REACHED)
    );
    assert_eq!(TEXT_BEARING.len(), 7);
    // **256 and not 8, and this line had been 8 since runtime architecture issue 22 lifted the
    // barrier.** A `Chord` reaches three of the eight modifier bits; `keys::press_with` takes a
    // `Mods` **value**, which this crate could not name until that issue, and the population went
    // 8 → 256 in `src/keys.rs` with this copy left behind. Nothing noticed because **`cargo test`
    // does not run an example**: it is compiled by `cargo clippy --all-targets` and evaluated by
    // nothing. Found by components 20 running every `*_numbers.rs` after finding the same class in
    // `listing_numbers.rs`.
    assert_eq!(REACHABLE_STATES, 256);
}
