# 13. Shipping a component library

A component library is a crate that depends on `vitui-runtime` and exports functions. Everything
below is about keeping the crate and its documentation from drifting apart, which is the failure
mode a library has that an application does not.

## Make the inventory a value

The single most useful thing this codebase does is that its component list is **data**, not prose:

```rust
pub struct Component {
    pub id: &'static str,       // also the function a caller writes, and the join key
    pub tier: Tier,
    pub built: bool,
    pub families: &'static [Family],
    pub glyphs: &'static [Glyph],
    pub constructions: usize,
    pub can_shrink: bool,
    pub owns_offset: bool,
    pub scrolled: bool,
    pub narrow: bool,
}

pub const INVENTORY: &[Component] = &[ … ];
```

Every obligation is then a **query over it** rather than a sentence in a document, and a component
added to the list arrives already owing whatever the queries ask for. Seven such queries are worth
copying:

| # | The obligation | The query |
|---|---|---|
| O1 | every component has a documented page with a compiled example | components with zero doc-tests == 0 |
| O2 | every component appears in a gallery screen | every gallery panel joins the inventory |
| O3 | every construction has a golden screen | goldens == constructions, iterated over the **inventory** |
| O4 | every component that reads a key declares a contract | documented bindings == what the machine actually takes |
| O5 | every declared hostile axis has a scene | `(component, axis)` pairs with no scene == 0 |
| O6 | the frame is flat in n, and an edit is linear with a stated constant | two step counts, joined by an `iff` |
| O7 | every component has an application that imports it | the import paths in `examples/` join the inventory |

Two lessons about writing such a query:

**Iterate the obligation's population, not its evidence.** *Every panel in the gallery is in the
freeze* over an **empty gallery** is vacuously true. Written the other way round — over the
inventory — an unbuilt row reads red, which is the answer you want.

**Read the population out of the source where you can.** A `built: true` column said `true` for a
component that did not exist anywhere in the crate for several releases. O7's population is read by
opening the files.

## What a component owes a reader

- **A page that opens by saying what the item does** — not a heading, not a fence. Cheap, checkable,
  and 1 948 of 1 948 items here already did it, so the gate cost nothing to adopt.
- **An example the compiler sees.** `text`, `ignore` and `compile_fail` fences are not examples.
  Making two `ignore` fences run here found that the shipped example of *seat the focus on the first
  frame* had been asserting the wrong thing for as long as it was never compiled.
- **The hostile axes it declares**, by name, in its own documentation.
- **The keyboard it answers**, rendered from the binding table.

Do **not** gate section ordering or a heading vocabulary. Measured over 709 headings here, 68 were
`# Panics`, 2 `# Errors`, 0 `# Safety` and about 550 were narrative — a gate over section order would
push 550 headings towards a vocabulary chosen for a different kind of library, and the pages would
get worse to make it green.

## State the fact, not the pointer

Shipped documentation should not cite a document the reader has no copy of. Ticket numbers, internal
spec sections, backlog paths and register rows are all invisible to a stranger, and they were on
5 557 lines here before anyone counted.

The rule that survived the sweep is a boundary rather than an exemption list:

> **Provenance is data, and a pointer may live only where a reader meets it by asking** — which is
> `#[cfg(test)]`, or a module the crate has marked `#[doc(hidden)]`.

Note the second population: a citation written as a **string literal** — a panic message, an
`#[expect]` reason, a struct field holding a section number — is invisible to a scan of comments and
rustdoc, and reaches a user through a panic.

`#[doc(hidden)]` is also how you keep a front page readable: 24 of this crate's 53 public modules are
hidden, chosen by scanning what applications actually import, so a reader lands on fourteen families
and fifteen helpers rather than on fifty-three modules with the components buried among them.

## The manifest and the README must agree

Three places say what a crate is before a stranger reads a line of its code — the manifest's
`description`, the README's opening paragraph, and the rustdoc's first line. Nothing compares them
by default, and all three of this facade's said the library was *reactive*, which it is not and never
was.

The gate with teeth is the **equality**: a README must open with its manifest's `description`. A
vocabulary scan only catches a word coming back.

## Dependencies and lints

- **Decide the dependency policy and enforce it in the build**, not in review. `cargo deny`'s
  `[bans]` bans a crate's *presence in the graph*, with `wrappers` as the exception list — which is
  how *this crate may not name the engine* becomes a build error rather than a convention.
- **Deny warnings in the manifest**, not in `RUSTFLAGS`: an environment variable does not merge with
  `build.rustflags`, it replaces it, so anyone with an unrelated `RUSTFLAGS` set silently loses the
  deny. A `[lints]` table cannot be switched off from a shell.
- **`cargo doc` does not document an example**, so the intra-doc link lint never sees your
  `examples/`. `cargo rustdoc --example NAME`, once per example, is the sweep.
- **Ship a `clippy.toml` in a template rather than in the library.** Clippy reads it from the crate
  being linted, so a disallowed-method list inside your workspace applies to *your* crate, not to
  your users'.

## Versioning

A public struct with public fields cannot gain a field compatibly. So the options structs are the
part of your surface that is cheapest to get right **before** the first release, and a major version
afterwards. Two fields that three separate applications had asked for were nearly deferred past a
release here on the assumption that adding them later would be compatible; the compiler said
otherwise, and the deferral would have cost a major version.

Adding an *example* is compatible. Adding a *function* is compatible. Widening a signature, adding an
argument, or adding a field is not.
