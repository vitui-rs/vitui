//! The public surface, as a value — and the audit spec §12 asked for.
//!
//! Ticket 24's first half is *assembled and audited*, and the audit is here rather than in a
//! document for the same reason register #19's negative cases are `compile_fail` doctests rather
//! than sentences: **a public surface described in prose is not checked by anything.** §12 states a
//! count and a listing, four implementation tickets moved both, and every one of those tickets said
//! so in its own answer — which is four places to look and no place that fails.
//!
//! So the surface is a value. [`SURFACE`] names every public item, the receiver every verb takes,
//! and — for the items §12's block does not list — the implementation ticket that added it and why.
//! The gates below are queries over it, in both directions: an item in the source and not in the
//! inventory is a failure, and an item in the inventory and not in the source is the same failure.
//! **A rename turns both red**, which is the property the negative corpus is built around and which
//! a one-directional check does not have.
//!
//! # What the audit found, and it is not what §12 says
//!
//! §12 opens with *twenty-one public types and about sixty-three functions*, and **its own block
//! does not agree with that sentence.** The block declares **thirty-nine** types with a `pub struct`
//! or `pub enum` keyword and names two more only inside a signature — `AttachError` in `attach`'s
//! `Result` and `Permit` in `permit_slow`'s return — so §12's listing is **forty-one types**. The
//! prose count is ticket 12's and was never true of the block beside it; the block is what ticket 24
//! means by *§12's listing*, and the block is what the inventory is checked against.
//!
//! The surface as built is **forty-nine types and one hundred and four functions**:
//!
//! | | types | functions |
//! |---|---|---|
//! | §12's block | 41 | 41 named, plus the 28 inside the four types it blesses wholesale |
//! | absent, and recorded rather than resurrected | −1 (`Resolver`) | — |
//! | added by implementation tickets, each naming one | +9 | +35 |
//! | **built** | **49** | **104** |
//!
//! **Architecture ticket 21 moved both columns and left the type count where it was.** §12's block
//! lost `LinkId` and gained `Link<'a>`, so forty-one is still forty-one; it lost `Screen::link`,
//! which is why the named functions are forty-one rather than forty-two. Both absent names are in
//! [`REFUSED_NAMES`], which is what makes the subtraction checkable rather than remembered.
//!
//! Not one of the nine types and not one of the thirty-five functions breaches a refusal: none is a
//! layout, a widget, a signal, a trait, an alpha, a blocking primitive, an executor, a clock, a
//! display query or a cell. Seven of the nine are input payload types §12's own delta list asked for
//! without naming (*the six `Event` variants and their payload types*), one is the type of a field
//! §12 does name (`Cursor { x, y, shape }`), and one is where the bytes go, which §12's `Config` has
//! no field for at all.
//!
//! # `Resolver` does not exist, and that is the finding
//!
//! §12's compositing line reads `pub struct LayerId; pub struct Mix { toward, amount }; pub struct
//! Resolver;`, and **`Resolver` appears nowhere else in the whole architecture** — not in a
//! signature, not in a sentence, not in the ticket the block came from beyond that one line. It has
//! no fields and no verbs, and the job the name suggests is done: impl 12 resolves an operator
//! layer's colour at composite time against what the terminal answered, which is [`Capabilities`]
//! and not a resolver the caller holds.
//!
//! Adding a public empty struct to make a count come out right is worse than recording that the
//! count is wrong, so the absence is **gated** — [`REFUSED_NAMES`] — with a positive twin naming
//! `Capabilities` by path. `Config::packets` and `Config::resolver` went the same way and for the
//! same kind of reason: the packet pool is fixed at two *provably* (spec §7), so a knob for it would
//! be a knob that may only hold one value.
//!
//! # Precedence rule 4, and the one place it reads differently than §12 wrote it
//!
//! [`Recv`] is on every verb, so *`&self` wherever a call only reads* is a query rather than a
//! paragraph. Three findings, all recorded and none of them a defect:
//!
//! - **`&mut self` appears on exactly the verbs where exclusivity is the guarantee**:
//!   [`Screen::layers`], [`Screen::present`], [`Screen::wait`], the write verbs, and the four
//!   `LayerStack` mutators. [`Screen::permit_slow`] takes `&self` — impl 23 found that `&mut self`
//!   there was `E0499` against drawing inside the permitted region.
//! - **The types §12 blesses wholesale read by value, not by reference.** `Rect`, `Style` and `Mix`
//!   are `Copy`, and so are `Mods` and `Buttons` — five of them, and `Color` is not among them
//!   because its only two verbs are constructors that take no receiver at all. On a `Copy` type
//!   `self` *is* the read, and `&self` would be a reference to something the size of a register.
//!   §12's precedence list says `&self`; the rule it states is *wherever a call only reads*, and
//!   forty verbs satisfy the rule while reading by value. Recorded, because the next reader will
//!   check.
//! - **Three moves, all of them ownership transfer**: [`Engine::attach`], `Slot::put` and
//!   `LayerStack::add_content_with` — and `Slot::put` moves its *argument*, not its receiver, which
//!   is `&self`.

use crate::{Engine, Slot, Surface, WakeHandle};

/// What a public item is. There is no third arm for a trait, and [`there_are_no_public_traits`] is
/// why that is a fact rather than a hope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A `pub struct`.
    Struct,
    /// A `pub enum`.
    Enum,
    /// A free function. There are two, and ADR 0005 is why.
    Function,
}

/// The receiver a verb takes: spec §12's precedence rule 4, made checkable.
///
/// **Syntax and not intent**, which is why there is no `Move` arm. `self` by value is a read on a
/// `Copy` type and an ownership transfer on everything else, and the two are the same three
/// characters in a signature — so [`MOVES`] names the transfers and
/// [`the_types_that_read_by_value_are_copy`] holds the other side of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recv {
    /// No receiver: a constructor, or one of the two free functions.
    None,
    /// `self`, by value.
    Value,
    /// `&self`. A call that only reads.
    Ref,
    /// `&mut self`. Exclusivity is the guarantee, not a concession.
    RefMut,
}

/// Where an item came from.
///
/// Two arms and no third, for the reason `crate::register`'s [`State`](crate::register::State) has
/// two: an item that is neither in §12's block nor attributable to a ticket is drift, and drift that
/// has a name for itself stops being visible.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Origin {
    /// Named in spec §12's block, or inside the four types its precedence list blesses wholesale.
    Spec12,
    /// Not in §12's block. The implementation ticket that added it, and why.
    Added {
        /// The implementation ticket, as `impl NN`.
        by: &'static str,
        /// Why it is there, in the terms the ticket used.
        why: &'static str,
    },
}

/// One public verb.
#[derive(Clone, Copy, Debug)]
pub struct Verb {
    /// Its name in the source.
    pub name: &'static str,
    /// Its receiver.
    pub recv: Recv,
    /// Where it came from.
    pub origin: Origin,
}

/// One public item, and every verb on it.
#[derive(Clone, Copy, Debug)]
pub struct Item {
    /// Its name, as `lib.rs` re-exports it.
    pub name: &'static str,
    /// What it is.
    pub kind: Kind,
    /// Where it came from.
    pub origin: Origin,
    /// Its inherent public verbs, sorted by name. Empty for a plain-data type and for the two free
    /// functions.
    pub verbs: &'static [Verb],
}

/// Names that may **not** be public, and the reason each one is not.
///
/// The first is §12's own, and the rest are the refusals. Each is checked against the crate root's
/// re-exports by [`the_refused_names_are_not_re_exported`], whose positive twin names a type that
/// *is* there — a list of absences passes for any reason at all, including the file having moved.
///
/// **A name spelled `Type::verb` is checked against that type's verbs instead**, and architecture
/// ticket 21 is why the second form exists: `Screen::link` is a method, so a check over the crate
/// root's re-exports would have passed for it on the day it was written and every day after,
/// whatever `Screen` grew back. A refusal nothing can fail is not a refusal.
pub const REFUSED_NAMES: &[(&str, &str)] = &[
    (
        "LinkId",
        "architecture ticket 21: the URI travels at the drawing verb (`Link`), so no handle is \
         public and refusal 11 holds with no exception beside it. A second mint is what made an \
         in-range collision between two handle spaces reachable",
    ),
    (
        "Screen::link",
        "architecture ticket 21: the only mint, deleted rather than joined by a second one. \
         `Restyle::link` names a URI and the `View` interns it into the handle space it draws into",
    ),
    (
        "Surface::link",
        "architecture ticket 21: the shape the question was asked in, and refused — a standalone \
         mint gives two handle spaces one opaque type and no way to tell them apart",
    ),
    (
        "View::link",
        "architecture ticket 21: strictly better than `Surface::link` and refused for the same \
         residue. Making the value unobtainable is what closes it",
    ),
    (
        "Resolver",
        "§12's compositing line names it once and nothing else in the architecture mentions it \
         again; impl 12 resolves an operator's colour against `Capabilities` at composite time",
    ),
    (
        "Cell",
        "refusal 11 and ADR 0023: a caller cannot read back what is on screen, so the cell is not a \
         public type",
    ),
    (
        "Painter",
        "refusal 5: the engine has nothing to call upward, so the dependency arrow is enforced by \
         there being no arrow. `dyn Painter` stays a negative result",
    ),
    (
        "Widget",
        "refusal 2: nothing is drawn for you and nothing can be registered to be drawn",
    ),
    ("Layout", "refusal 1 and ADR 0002: callers bring rectangles"),
    ("Constraint", "refusal 1 and ADR 0002"),
    (
        "Signal",
        "refusal 3: no signal, no observer, no subscription",
    ),
    (
        "Executor",
        "refusal 8: no executor and no thread pool. What is offered instead is `Slot`",
    ),
    (
        "ThreadPool",
        "refusal 8, and spec §11's numbers for a spawn per frame are beside `Slot` itself",
    ),
    (
        "Options",
        "priced rather than overlooked (§8): the serializer takes no options type, because every \
         axis it would have carried is a `Capabilities` fact",
    ),
    (
        "Waker",
        "half the `Future` contract in a crate that bans futures — the type is `WakeHandle`",
    ),
    (
        "Parker",
        "ADR 0003's split handles are internal, which makes `Perf::enter` uncallable rather than \
         merely unforgettable",
    ),
];

/// Exactly what `pub mod prelude` re-exports, as ticket 24 states it.
pub const PRELUDE: &[&str] = &[
    "Color", "Config", "Engine", "LayerId", "Rect", "Screen", "Style", "View", "Wake",
];

/// The modules that are compiled only under `cfg(test)`, and are therefore not on the public
/// surface however much `pub` is written inside them.
///
/// A list rather than a filter, so that a module arriving in `src/` is either parsed by the gates
/// below or named here — the same *nothing is silently absent* shape `crate::register` uses. It is
/// what lets `scenes.rs`'s `pub trait Scene` be the positive twin of the zero-trait gate instead of
/// a hole in it.
///
/// **Paths relative to `src/`, and the two subdirectory files are on it.** `src/input/` and
/// `src/ucd/` both hold code, and a flat `read_dir` over `src/` is the exact mistake `crate::gates`
/// records having already made once — a gate that would have passed for a blocking receive added to
/// the input parser, the one file where a reader would most expect to find one.
///
/// **The list is checked against the `#[cfg(test)]` attributes themselves**, in both directions, by
/// [`the_test_only_list_is_what_the_crate_root_declares_under_cfg_test`]. A name here that stops
/// being test-only would otherwise stay silently exempt from every scan below — and `lib.rs`
/// predicted exactly that happening: *ticket 25's fuzz targets are what will need `reference`
/// outside `cfg(test)`*. It happened, at ticket 25, and `reference.rs` moved to
/// [`SOAK_ONLY_MODULES`] rather than quietly staying here. Left unchecked, dropping `#[cfg(test)]`
/// from `mod scenes;` would ship `pub trait Scene` with the zero-trait gate still green, because
/// that gate's positive twin *requires* `Scene` to be found.
pub const TEST_ONLY_MODULES: &[&str] = &[
    "audit.rs",
    "gates.rs",
    "golden.rs",
    "input/tests.rs",
    // Impl 26's ledger. Test-only for the same reason `register.rs` is: it is the instrument §14
    // asks for rather than a part of the engine, `examples/budget.rs` reaches it with `#[path]`, and
    // nothing a caller can hold is in it.
    "ledger.rs",
    "register.rs",
    "roundtrip.rs",
    "scenes.rs",
    "term_model.rs",
    "testing.rs",
    "ucd/tests.rs",
];

/// The modules the crate compiles under `cfg(test)` **or** the `fuzz` feature, and never in an
/// ordinary build.
///
/// [`TEST_ONLY_MODULES`]'s sibling, and it exists because ticket 25 needed a third state that the
/// two-arm question *shipped or test-only?* cannot express. The reference compositor is §14's oracle
/// for gate #1 **and** for the first fuzz target, and a fuzz target is in another crate — `fuzz/` is
/// its own workspace, because `cargo-fuzz` needs nightly and `libfuzzer-sys`. So it is compiled by
/// `cargo test` and by `cargo build --features fuzz`, and by nothing a dependent of this crate
/// builds.
///
/// The distinction is not bookkeeping. These modules are **absent from the shipped surface** and so
/// are exempt from the scans over [`shipped_modules`](tests::shipped_modules) exactly as the
/// test-only ones are, and they are **absent from the doc build** and so may no more hold a
/// `compile_fail` case than a `cfg(test)` module may. Both halves are asserted below, and the list
/// itself is checked against the crate root's attributes by
/// [`the_soak_only_list_is_what_the_crate_root_declares_under_the_fuzz_feature`].
pub const SOAK_ONLY_MODULES: &[&str] = &["fuzz.rs", "reference.rs"];
/// Every public item, with the receiver of every verb and the provenance of everything §12's block
/// does not list.
///
/// Sorted by name, because the gates compare it against a parse of the source and a sorted list is
/// the one a human can diff. Forty-nine types, one hundred and five functions, and no traits.
pub const SURFACE: &[Item] = &[
    Item {
        name: "AttachError",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Button",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 20",
            why: "a payload type of one of §12's six `Event` variants: the delta list says *the six `Event` variants and their payload types* and names none of them",
        },
        verbs: &[],
    },
    Item {
        name: "Buttons",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "back",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "`Buttons` is §12's own type and these are the readers of its bits",
                },
            },
            Verb {
                name: "forward",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "`Buttons` is §12's own type and these are the readers of its bits",
                },
            },
            Verb {
                name: "is_empty",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "`Buttons` is §12's own type and these are the readers of its bits",
                },
            },
            Verb {
                name: "left",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "`Buttons` is §12's own type and these are the readers of its bits",
                },
            },
            Verb {
                name: "middle",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "`Buttons` is §12's own type and these are the readers of its bits",
                },
            },
            Verb {
                name: "right",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "`Buttons` is §12's own type and these are the readers of its bits",
                },
            },
        ],
    },
    Item {
        name: "Capabilities",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[Verb {
            name: "report",
            recv: Recv::Ref,
            origin: Origin::Added {
                by: "impl 16",
                why: "diagnostics, and the reason there is not a public field per fact: a bug report pastes it and nothing can branch on it",
            },
        }],
    },
    Item {
        name: "Clock",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Color",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "indexed",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
            Verb {
                name: "rgb",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "ColorDepth",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Config",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Cursor",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "CursorShape",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 21",
            why: "§12 writes `Cursor { x, y, shape }` and names the field rather than its type; `DECSCUSR` conflates shape with blink, so the enum carries the blinking spellings and `Terminal`",
        },
        verbs: &[],
    },
    Item {
        name: "Engine",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "attach",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "new",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Event",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[Verb {
            name: "at",
            recv: Recv::Ref,
            origin: Origin::Added {
                by: "impl 20",
                why: "the stamp every variant carries, which the app thread cannot recover once the frame clock has held it",
            },
        }],
    },
    Item {
        name: "GlyphSet",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "InputConfig",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "InputDiagnostics",
        kind: Kind::Struct,
        origin: Origin::Added {
            by: "impl 20",
            why: "what the parser could not recognise, so that a sequence nobody parses is visible rather than silently dropped",
        },
        verbs: &[
            Verb {
                name: "last_unrecognised",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "what the parser could not recognise",
                },
            },
            Verb {
                name: "unrecognised",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "what the parser could not recognise",
                },
            },
        ],
    },
    Item {
        name: "Key",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "KeyCode",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "KeyKind",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "KeyText",
        kind: Kind::Struct,
        origin: Origin::Added {
            by: "impl 20",
            why: "ADR 0007: what the key printed, an inline grapheme cluster and never a `char`",
        },
        verbs: &[
            Verb {
                name: "as_str",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "ADR 0007: what the key printed, read as a `&str` without allocating",
                },
            },
            Verb {
                name: "is_empty",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "ADR 0007: what the key printed, read as a `&str` without allocating",
                },
            },
        ],
    },
    Item {
        name: "Keypad",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 20",
            why: "a payload type of one of §12's six `Event` variants: the delta list says *the six `Event` variants and their payload types* and names none of them",
        },
        verbs: &[],
    },
    Item {
        name: "LayerId",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "LayerStack",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "add_content",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "add_content_with",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "add_operator",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "add_shadow",
                recv: Recv::RefMut,
                origin: Origin::Added {
                    by: "impl 12",
                    why: "mandated by spec §5 — *a constructor taking an offset and an intensity and emitting the operator at the right `z`* — and absent from §12's block",
                },
            },
            Verb {
                name: "is_empty",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "len",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "remove",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "set_rect",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "set_z",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "topmost_at",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "view",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Link",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Media",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 20",
            why: "a payload type of one of §12's six `Event` variants: the delta list says *the six `Event` variants and their payload types* and names none of them",
        },
        verbs: &[],
    },
    Item {
        name: "Mix",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "amount",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "darken",
                recv: Recv::None,
                origin: Origin::Added {
                    by: "impl 12",
                    why: "the spec's own vocabulary for the two directions, kept as a symmetric pair with `lift`",
                },
            },
            Verb {
                name: "is_identity",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "lift",
                recv: Recv::None,
                origin: Origin::Added {
                    by: "impl 12",
                    why: "the spec's own vocabulary for the two directions, kept as a symmetric pair with `darken`",
                },
            },
            Verb {
                name: "new",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
            Verb {
                name: "toward",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Modifier",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 20",
            why: "a payload type of one of §12's six `Event` variants: the delta list says *the six `Event` variants and their payload types* and names none of them",
        },
        verbs: &[],
    },
    Item {
        name: "Mods",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "alt",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "caps",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "chord",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "contains",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "ctrl",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "hyper",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "is_empty",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "meta",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "num",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "shift",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "super_key",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
            Verb {
                name: "with",
                recv: Recv::Value,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the intent half of eight modifier bits (ADR 0007), which §12 named as a type and left without verbs",
                },
            },
        ],
    },
    Item {
        name: "Mouse",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "MouseKind",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "MouseMode",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Output",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 03",
            why: "§12's `Config` has no field for where the bytes go, and §14's deterministic mode is written against a caller-supplied sink",
        },
        verbs: &[],
    },
    Item {
        name: "Overrides",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[Verb {
            name: "plain",
            recv: Recv::None,
            origin: Origin::Added {
                by: "impl 16",
                why: "the one named combination that survives, and it survives because it is a constructor rather than a tier",
            },
        }],
    },
    Item {
        name: "Paste",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "at",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "ADR 0007: a paste owns **bytes** and not a `String`, so reading one is three accessors and a stamp",
                },
            },
            Verb {
                name: "bytes",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "ADR 0007: a paste owns **bytes** and not a `String`, so reading one is three accessors and a stamp",
                },
            },
            Verb {
                name: "text",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "ADR 0007: a paste owns **bytes** and not a `String`, so reading one is three accessors and a stamp",
                },
            },
            Verb {
                name: "truncated",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "ADR 0007: a paste owns **bytes** and not a `String`, so reading one is three accessors and a stamp",
                },
            },
        ],
    },
    Item {
        name: "Permit",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Presented",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Rect",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "bottom",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "intersect",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "is_empty",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "new",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
            Verb {
                name: "right",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Restyle",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Rgb",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[Verb {
            name: "new",
            recv: Recv::None,
            origin: Origin::Added {
                by: "impl 16",
                why: "§12 lists the type and no verb; a colour with three private bytes needs one",
            },
        }],
    },
    Item {
        name: "Screen",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "capabilities",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "input_diagnostics",
                recv: Recv::Ref,
                origin: Origin::Added {
                    by: "impl 20",
                    why: "the unrecognised-sequence count, which is the only way an application learns its terminal spoke something this parser does not",
                },
            },
            Verb {
                name: "layers",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "next_event",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "permit_slow",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "present",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "request_wake_at",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "resume",
                recv: Recv::RefMut,
                origin: Origin::Added {
                    by: "production 07",
                    why: "§12 has no verb for the terminal leaving and coming back, and §15 filed it as fog; the epilogue and the negotiation both already existed and only the question of when they run was open",
                },
            },
            Verb {
                name: "set_cursor",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "set_max_frame_rate",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "set_mouse",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "size",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "suspend",
                recv: Recv::RefMut,
                origin: Origin::Added {
                    by: "production 07",
                    why: "the other half of `resume`, and two verbs rather than one taking a closure because `no_public_verb_takes_a_closure_or_an_iterator` is a gate",
                },
            },
            Verb {
                name: "wait",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Slot",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "new",
                recv: Recv::None,
                origin: Origin::Added {
                    by: "impl 18",
                    why: "§12 lists the two verbs; a `const` constructor is what lets an application hold one in a `static` without a `OnceLock` around it",
                },
            },
            Verb {
                name: "put",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "take",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Stop",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Style",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "bg",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "blink",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "bold",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "conceal",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "dim",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "fg",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "italic",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "new",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
            Verb {
                name: "no_underline",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "overline",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "reverse",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "strikethrough",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "underline",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "underline_curly",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "underline_dashed",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "underline_dotted",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
            Verb {
                name: "underline_double",
                recv: Recv::Value,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Surface",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "new",
                recv: Recv::None,
                origin: Origin::Spec12,
            },
            Verb {
                name: "root",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "size",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "View",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "child",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "fill",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "restyle",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "scrolled",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "set",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "size",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "text",
                recv: Recv::RefMut,
                origin: Origin::Spec12,
            },
            Verb {
                name: "visible_cols",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "visible_rows",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Wake",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "WakeHandle",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[
            Verb {
                name: "post",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
            Verb {
                name: "quit",
                recv: Recv::Ref,
                origin: Origin::Spec12,
            },
        ],
    },
    Item {
        name: "Wheel",
        kind: Kind::Enum,
        origin: Origin::Added {
            by: "impl 20",
            why: "a payload type of one of §12's six `Event` variants: the delta list says *the six `Event` variants and their payload types* and names none of them",
        },
        verbs: &[],
    },
    Item {
        name: "WidthSource",
        kind: Kind::Enum,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "Written",
        kind: Kind::Struct,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "graphemes",
        kind: Kind::Function,
        origin: Origin::Spec12,
        verbs: &[],
    },
    Item {
        name: "width_of",
        kind: Kind::Function,
        origin: Origin::Spec12,
        verbs: &[],
    },
];

/// The three moves on the whole surface, and what ownership transfer buys in each.
///
/// §12's precedence list names exactly these three. `Slot::put` and `LayerStack::add_content_with`
/// move their **argument** and take `&self` and `&mut self` respectively, which is why they are here
/// and their receivers are not [`Recv::Value`].
pub const MOVES: &[(&str, &str, &str)] = &[
    (
        "Engine",
        "attach",
        "the engine is consumed to produce the app thread's world: raw mode, the capability queries \
         and the prologue happen once, and a second `attach` from the same value would be a second \
         terminal",
    ),
    (
        "Slot",
        "put",
        "the worker gives the value up. A `&T` would need the app thread to copy it out under the \
         lock, which is the allocation the frame path does not have",
    ),
    (
        "LayerStack",
        "add_content_with",
        "a surface drawn on a worker is donated, and its handles are renumbered into the stack's \
         once, at donation (spec §5)",
    ),
];

/// The five `Copy` types whose verbs read by value.
///
/// Named here rather than derived, because *this type is `Copy`* is a compile-time fact and a list
/// is what lets [`the_types_that_read_by_value_are_copy`] check the two halves against each other.
pub const READ_BY_VALUE: &[&str] = &["Buttons", "Mix", "Mods", "Rect", "Style"];

/// How a refusal is held: the four shapes, and there is no fifth.
///
/// The ticket's acceptance line is *every refusal has either a negative doctest or an asserted
/// absence; none is documented only in prose*, so prose is exactly what this enum cannot express.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Evidence {
    /// A paired `compile_fail` doctest in this module, and a positive twin beside it.
    Pair {
        /// The file, under `src/`.
        module: &'static str,
        /// A fragment of the hostile line itself.
        ///
        /// Naming the module alone is not enough: `view.rs`, `engine.rs`, `layer.rs`, `mix.rs` and
        /// the crate root each carry several negative cases, so *this file has a `compile_fail` in
        /// it somewhere* stays true after the one this refusal is about has been deleted. The
        /// fragment is looked for **inside a `compile_fail` block**, so a mention in prose does not
        /// satisfy it.
        hostile: &'static str,
    },
    /// A gate that asserts the absence over the source, because there is no signature to write.
    Absence {
        /// The test function's name.
        at: &'static str,
    },
    /// A lint on the crate root. A lint is not a claim.
    Lint {
        /// The attribute, verbatim.
        attribute: &'static str,
    },
    /// Internal to the crate, and **out of the corpus' reach**: doctests compile as an external
    /// crate, so they reach only the public surface — which is exactly the set of misuses a *user*
    /// could commit. An internal misuse is caught by the engine's own compilation, and an internal
    /// negative case costs a `#[doc(hidden)] pub`, which is a design admission and should read as
    /// one.
    Internal {
        /// Why it is not reachable from outside.
        why: &'static str,
    },
}

/// One thing the engine refuses, and where the refusal is held.
#[derive(Clone, Copy, Debug)]
pub struct Refusal {
    /// §12's number, or `0` for one of the six priced-rather-than-overlooked absences.
    pub number: u8,
    /// What is refused, in §12's own words.
    pub what: &'static str,
    /// How it is held.
    pub evidence: &'static [Evidence],
}

/// §12's twelve refusals and its six priced absences, each with the place it is held.
///
/// **There is no type to look for — that is the test**, which is exactly why a list of them has to
/// exist somewhere that fails. Nine are gated on the type that would have carried the refused item,
/// three have no such type and are on the crate root, one is a lint, and one is internal and says so.
pub const REFUSALS: &[Refusal] = &[
    Refusal {
        number: 1,
        what: "No layout. No constraint, no flex, no measure, no auto-size (ADR 0002)",
        evidence: &[Evidence::Pair {
            module: "view.rs",
            hostile: ".layout(",
        }],
    },
    Refusal {
        number: 2,
        what: "No widget. Nothing is drawn for you and nothing can be registered to be drawn",
        evidence: &[Evidence::Pair {
            module: "view.rs",
            hostile: ".layout(",
        }],
    },
    Refusal {
        number: 3,
        what: "No reactivity. No signal, no observer, no subscription",
        evidence: &[Evidence::Pair {
            module: "engine.rs",
            hostile: ".subscribe(",
        }],
    },
    Refusal {
        number: 4,
        what: "No iteration of application data. Coordinates and `&str` go in; nothing of yours is \
               held",
        evidence: &[Evidence::Absence {
            at: "no_public_verb_takes_a_closure_or_an_iterator",
        }],
    },
    Refusal {
        number: 5,
        what: "No trait — zero of them. The dependency arrow is enforced by there being no arrow",
        evidence: &[
            Evidence::Pair {
                module: "lib.rs",
                hostile: "impl vitui_engine::Painter for Mine",
            },
            Evidence::Absence {
                at: "there_are_no_public_traits",
            },
        ],
    },
    Refusal {
        number: 6,
        what: "No alpha and no per-layer opacity",
        evidence: &[
            Evidence::Pair {
                module: "layer.rs",
                hostile: "add_content(0, Rect::new(0, 0, 4, 2), true, 0.5)",
            },
            Evidence::Pair {
                module: "mix.rs",
                hostile: "Mix::new",
            },
        ],
    },
    Refusal {
        number: 7,
        what: "No blocking primitive on the app thread and no completion anywhere",
        evidence: &[
            Evidence::Pair {
                module: "slot.rs",
                hostile: "slot.recv()",
            },
            Evidence::Absence {
                at: "every_blocking_receive_in_the_crate_is_outside_the_app_threads_loop",
            },
        ],
    },
    Refusal {
        number: 8,
        what: "No executor and no thread pool",
        evidence: &[Evidence::Pair {
            module: "lib.rs",
            hostile: "ThreadPool::new(4)",
        }],
    },
    Refusal {
        number: 9,
        what: "No clock and no scheduler. `request_wake_at` is a deadline sink — and this one was \
               nearly lost: `elapsed()` and `wake_in()` on `View` were proposed, worked, and were \
               refused",
        evidence: &[
            Evidence::Pair {
                module: "view.rs",
                hostile: "view.elapsed()",
            },
            Evidence::Pair {
                module: "view.rs",
                hostile: "view.wake_in(",
            },
        ],
    },
    Refusal {
        number: 10,
        what: "No display query. The refresh rate arrives as configuration or not at all",
        evidence: &[
            Evidence::Pair {
                module: "engine.rs",
                hostile: "screen.refresh_rate()",
            },
            Evidence::Absence {
                at: "nothing_anywhere_queries_a_display_for_a_refresh_rate",
            },
        ],
    },
    Refusal {
        number: 11,
        what: "No cells, no grapheme handles, no style bits (ADR 0023)",
        evidence: &[Evidence::Pair {
            module: "lib.rs",
            hostile: "Cell::default()",
        }],
    },
    Refusal {
        number: 12,
        what: "No `unsafe`",
        evidence: &[
            Evidence::Lint {
                attribute: "#![forbid(unsafe_code)]",
            },
            Evidence::Absence {
                at: "the_crate_root_forbids_unsafe_code",
            },
        ],
    },
    Refusal {
        number: 0,
        what: "`split_h` / `split_v` (§4)",
        evidence: &[Evidence::Pair {
            module: "view.rs",
            hostile: "view.split_h(4)",
        }],
    },
    Refusal {
        number: 0,
        what: "`fill_with` (§4)",
        evidence: &[Evidence::Pair {
            module: "view.rs",
            hostile: "view.fill_with(",
        }],
    },
    Refusal {
        number: 0,
        what: "a write-time equality filter (§4)",
        evidence: &[Evidence::Internal {
            why: "nothing public would have named it: the filter that does exist is the \
                  serializer's, against the mirror, and a write-time one is a decision about what \
                  `View::set` does to damage. `crate::surface` records the price and the engine's \
                  own compilation is what holds it",
        }],
    },
    Refusal {
        number: 0,
        what: "a flattened layer cache (§5, ADR 0024)",
        evidence: &[Evidence::Pair {
            module: "layer.rs",
            hostile: "layers().flattened()",
        }],
    },
    Refusal {
        number: 0,
        what: "a serializer `Options` type (§8)",
        evidence: &[Evidence::Pair {
            module: "lib.rs",
            hostile: "Options::default()",
        }],
    },
    Refusal {
        number: 0,
        what: "a worker pool (§11)",
        evidence: &[Evidence::Pair {
            module: "lib.rs",
            hostile: "ThreadPool::new(4)",
        }],
    },
];

/// How many `compile_fail` fences the crate carries.
///
/// An equality and not a floor, because the number is a property of the mechanism rather than of the
/// data: **the pair is the unit**, and a case deleted without its twin is precisely the edit this
/// number exists to catch. Impl 23 left sixteen; ticket 24 brought the corpus to thirty-six;
/// architecture ticket 21 made it thirty-seven.
///
/// **Its arithmetic is worth spelling out, because it went up while two public items went away.**
/// `LinkId`'s own `E0423` case — *you cannot build one from a number* — went with the type, and two
/// took its place on the crate root: the type does not exist, and neither does the mint that made
/// one. Their twin is the crate root's, which now names `Restyle::link` and `Link::Uri` by path
/// alongside the five it already named; `Screen::link`'s own runnable example went with the verb,
/// which is why [`RUNNABLE_EXAMPLES`] fell by two while this rose by one.
pub const NEGATIVE_CASES: usize = 37;

/// How many **runnable** doc examples the crate carries — the positive twins, and the ordinary
/// examples beside them.
///
/// Counted for the same reason as [`NEGATIVE_CASES`] and it is the half that was missing: a corpus
/// count over the `compile_fail` fences alone protects the hostile line and leaves the twin
/// unguarded, and *the twin is the half that catches a rename*. Deleting one on its own left all 747
/// lib tests and all 84 doc examples green, at which point a later rename would make the surviving
/// negative case fail for the wrong error and report `ok` — exactly the failure the pairing exists
/// to close.
///
/// Forty-six since architecture ticket 21: `LinkId`'s twin and `Screen::link`'s own example both
/// named items that no longer exist, and what replaced them is one more path inside the crate
/// root's existing twin rather than a fence of its own.
///
/// **Forty-seven since production ticket 07**, and it is the one place `Screen::suspend` and
/// `Screen::resume` are compiled as a caller would write them — three lines with somebody else
/// holding the terminal in between. It has no negative twin because the pair refuses nothing a
/// compile outcome can express: the closure form the two verbs replace is already held out by
/// `no_public_verb_takes_a_closure_or_an_iterator`, which is a gate over the whole surface rather
/// than a fence beside one item.
pub const RUNNABLE_EXAMPLES: usize = 47;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn src_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
    }

    fn root() -> String {
        std::fs::read_to_string(src_dir().join("lib.rs"))
            .expect("the crate root is beside this file")
    }

    /// Whole `pub use` and `pub mod` statements out of the crate root, not first lines.
    ///
    /// `lib.rs` has a re-export spanning five lines — `pub use input::{ … }` — so a name on a
    /// continuation line is invisible to a filter over line starts, which is the defect impl 23
    /// found in `the_split_handles_are_not_public_names` and fixed the same way.
    fn re_export_statements() -> Vec<String> {
        let root = root();
        let mut out = Vec::new();
        let mut open: Option<String> = None;
        for line in root.lines() {
            let statement = match open.take() {
                Some(mut held) => {
                    held.push(' ');
                    held.push_str(line.trim());
                    held
                }
                None if line.starts_with("pub use ") || line.starts_with("pub mod ") => {
                    line.trim().to_string()
                }
                None => continue,
            };
            match statement.ends_with(';') || statement.ends_with('}') {
                true => out.push(statement),
                false => open = Some(statement),
            }
        }
        assert!(
            open.is_none(),
            "a re-export in lib.rs never ended, so this parse stopped early"
        );
        out
    }

    /// Every name on the crate's public surface: what the crate root re-exports **and what it
    /// declares itself**.
    ///
    /// The second half is not decoration. A surface derived from `pub use` alone misses a `pub`
    /// item written directly into `lib.rs`, and that is the one edit no other gate here sees:
    /// `pub struct Painter;` appended to the crate root left all sixteen green, `REFUSED_NAMES`
    /// included, because the name it forbids was never in the set being searched. Measured, and
    /// closed by the union below.
    fn public_names() -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for statement in re_export_statements() {
            let Some(body) = statement.strip_prefix("pub use ") else {
                continue;
            };
            let body = body.trim_end_matches(';');
            // Two spellings this parse cannot read, and neither exists today. It fails loudly
            // rather than quietly, because a silent half-parse is what the whole file is against:
            // `pub use foo::{bar::{A, B}, C}` yields `bar::{A` and `B}`, and `Bar as Baz` yields
            // one name with a space in it.
            let inner = body.find('{').map(|open| &body[open + 1..]).unwrap_or("");
            assert!(
                !inner.contains('{'),
                "a nested-brace re-export needs a better parse than this one: {statement}"
            );
            assert!(
                !body.contains(" as "),
                "an `as` rename needs a better parse than this one: {statement}"
            );
            match body.find('{') {
                Some(open) => {
                    let close = body.rfind('}').expect("a braced re-export closes");
                    for name in body[open + 1..close].split(',') {
                        let name = name.trim();
                        if !name.is_empty() {
                            names.insert(name.to_string());
                        }
                    }
                }
                None => {
                    let last = body.rsplit("::").next().expect("a path has a last segment");
                    names.insert(last.trim().to_string());
                }
            }
        }
        for (module, _, name) in declarations() {
            if module == "lib.rs" {
                names.insert(name);
            }
        }
        names
    }

    /// The top-level modules that are part of the shipped crate rather than of its tests.
    /// Every `.rs` file under `src/`, as a path relative to `src/`, sorted.
    ///
    /// **Recursive, because `src/` has subdirectories and both of them hold code.** A flat
    /// `read_dir` is the mistake `crate::gates::every_blocking_receive_in_the_crate_is_outside_the_app_threads_loop`
    /// records having already made, and it would have made every gate below blind to
    /// `src/input/` — a new public verb, a `pub trait` or a deleted negative case in the input
    /// parser, seen by none of the sixteen.
    fn all_modules() -> Vec<String> {
        fn walk(dir: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).expect("a readable source directory") {
                let path = entry.expect("a directory entry reads").path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                let relative = match prefix.is_empty() {
                    true => name.to_string(),
                    false => format!("{prefix}/{name}"),
                };
                if path.is_dir() {
                    walk(&path, &relative, out);
                    continue;
                }
                if name.ends_with(".rs") {
                    out.push(relative);
                }
            }
        }
        let mut out = Vec::new();
        walk(&src_dir(), "", &mut out);
        out.sort();
        assert!(
            out.len() > 25,
            "only {} modules were found under src/, which is too few to be the whole crate",
            out.len()
        );
        assert!(
            out.iter().any(|m| m.contains('/')),
            "the walk found nothing in a subdirectory, so it is not recursing"
        );
        out
    }

    /// The modules that are part of the shipped crate rather than of its tests or its soak.
    ///
    /// Both lists come off, and for the same reason: a module an ordinary build does not compile is
    /// on nobody's surface, whichever `cfg` keeps it out. See [`SOAK_ONLY_MODULES`].
    fn shipped_modules() -> Vec<String> {
        let out: Vec<String> = all_modules()
            .into_iter()
            .filter(|m| {
                m != "lib.rs"
                    && !TEST_ONLY_MODULES.contains(&m.as_str())
                    && !SOAK_ONLY_MODULES.contains(&m.as_str())
            })
            .collect();
        assert!(
            out.len() > 20,
            "only {} shipped modules were found, which is too few to be the whole crate",
            out.len()
        );
        out
    }

    fn read(module: &str) -> String {
        std::fs::read_to_string(src_dir().join(module))
            .unwrap_or_else(|_| panic!("`{module}` reads"))
    }

    /// `impl <Target>` blocks, keyed by target, with the body's lines.
    ///
    /// **Line-anchored rather than brace-matched.** Doc comments in this crate carry doctests full
    /// of braces — a paired negative case is *literally* `Overrides { mouse: Some(true), .. }` —
    /// and a brace counter walks straight into them. `rustfmt` puts an `impl` at column zero and its
    /// closing brace alone on a line at column zero, so the block is exactly what lies between.
    fn inherent_impls(module: &str) -> Vec<(String, Vec<String>)> {
        let text = read(module);
        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        let mut open: Option<(String, Vec<String>)> = None;
        for line in text.lines() {
            if let Some((target, body)) = open.as_mut() {
                if line == "}" {
                    let done = (std::mem::take(target), std::mem::take(body));
                    open = None;
                    out.push(done);
                } else {
                    body.push(line.to_string());
                }
                continue;
            }
            let Some(rest) = line.strip_prefix("impl") else {
                continue;
            };
            // `impl<'a> View<'a> {` — step over the generics before reading the target, and step
            // over them by **depth** rather than by the first `>`: for `impl<T: Into<u8>> Foo<T>`
            // the first `>` lands inside the bound, the target parses empty, and the whole block
            // disappears from the surface with nothing failing. `SURFACE` is written to match what
            // this parser reports, so both directions would have agreed and stayed green.
            let rest = match rest.starts_with('<') {
                true => match balanced_angles(rest) {
                    Some(end) => &rest[end + 1..],
                    None => panic!("an impl in {module} opens generics that never close: {line}"),
                },
                false => rest,
            };
            let head = rest.trim();
            // A trait impl is somebody else's contract, not this crate's surface.
            if head.contains(" for ") {
                continue;
            }
            let target: String = head
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            assert!(
                !target.is_empty(),
                "an impl in {module} has a target this parse cannot read: {line}"
            );
            open = Some((target, Vec::new()));
        }
        assert!(
            open.is_none(),
            "an impl block in {module} never closed at column zero"
        );
        out
    }

    /// The index of the `>` that closes the `<…>` a string opens with, counting depth.
    fn balanced_angles(text: &str) -> Option<usize> {
        let mut depth = 0i32;
        for (index, ch) in text.char_indices() {
            match ch {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// One public verb as the source spells it: its name, its receiver, and its whole parameter
    /// list.
    /// One public verb as the source spells it: its name, its receiver, and **its whole signature**
    /// — generic bounds, parameters and `where` clause alike.
    ///
    /// The signature is captured whole rather than parameters-only, and that is a fix rather than a
    /// flourish. Capturing from the first `(` on the line reads
    /// `pub fn probe<F: Fn(Style) -> Style>(&mut self, r: Rect, f: F)` as the parameter list
    /// `Style` — so refusal 4's gate passes and the receiver is recorded as `Recv::None` instead of
    /// `RefMut`, wrongly on both counts. Measured, and it is the one shape that matters
    /// historically: `restyle` took `impl Fn(Style) -> Style` in ticket 12's skeleton. A
    /// `where F: Fn(…)` clause escapes the same way, one line further down, which is why the
    /// capture runs to the opening brace of the body.
    fn verbs_of(body: &[String]) -> Vec<(String, Recv, String)> {
        let mut out = Vec::new();
        for (index, line) in body.iter().enumerate() {
            let Some(rest) = line
                .strip_prefix("    pub fn ")
                .or_else(|| line.strip_prefix("    pub const fn "))
            else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            // The whole signature, joined forward to the brace that opens the body. rustfmt breaks
            // a long one over as many lines as it likes, and a `where` clause is one of them.
            let mut signature = String::new();
            let mut depth = 0i32;
            'join: for line in &body[index..] {
                for ch in line.chars() {
                    match ch {
                        '(' | '[' => depth += 1,
                        ')' | ']' => depth -= 1,
                        '{' if depth == 0 => break 'join,
                        _ => {}
                    }
                    signature.push(ch);
                }
                signature.push(' ');
            }
            // The receiver comes from the parameter list, which starts after the generics — so
            // step over a `<…>` by depth before looking for the `(`.
            let after_name =
                &signature[signature.find(&name).map(|at| at + name.len()).unwrap_or(0)..];
            let after_generics = match after_name.trim_start().starts_with('<') {
                true => {
                    let trimmed = after_name.trim_start();
                    match balanced_angles(trimmed) {
                        Some(at) => &trimmed[at + 1..],
                        None => panic!("`{name}` opens generics that never close: {signature}"),
                    }
                }
                false => after_name,
            };
            let params = match after_generics.find('(') {
                Some(at) => after_generics[at + 1..].trim_start(),
                None => panic!("`{name}` has no parameter list: {signature}"),
            };
            let recv = if params.starts_with("&mut self") {
                Recv::RefMut
            } else if params.starts_with("&self") {
                Recv::Ref
            } else if params.starts_with("self") {
                Recv::Value
            } else {
                Recv::None
            };
            out.push((name, recv, signature));
        }
        out
    }

    /// Every inherent public verb on a re-exported type, as the source has it.
    fn verbs_in_source() -> BTreeMap<String, BTreeMap<String, (Recv, String)>> {
        let exported = public_names();
        let mut out: BTreeMap<String, BTreeMap<String, (Recv, String)>> = BTreeMap::new();
        for module in shipped_modules() {
            for (target, body) in inherent_impls(&module) {
                if !exported.contains(&target) {
                    continue;
                }
                for (name, recv, params) in verbs_of(&body) {
                    out.entry(target.clone())
                        .or_default()
                        .insert(name, (recv, params));
                }
            }
        }
        out
    }

    /// Items declared `pub` at column zero, per module: `(module, keyword, name)`.
    ///
    /// Over the recursive walk, so a `pub trait` in `src/input/` is seen.
    fn declarations() -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        for module in all_modules() {
            for line in read(&module).lines() {
                for keyword in ["struct", "enum", "trait", "union", "type", "const fn", "fn"] {
                    let prefix = format!("pub {keyword} ");
                    if let Some(rest) = line.strip_prefix(&prefix) {
                        let name: String = rest
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();
                        out.push((module.clone(), keyword.to_string(), name));
                        break;
                    }
                }
            }
        }
        out
    }

    /// Every doc code block in a module, as `(attributes, body)`.
    ///
    /// One state machine for both halves of a pair: a fence with `compile_fail` among its attributes
    /// opens a negative case, and a fence with none opens a runnable doctest, which is what a
    /// positive twin is. Counting both is what makes the corpus size a gate over the **pair** rather
    /// than over half of it — the twin is precisely the half that catches a rename, and deleting one
    /// alone left every test in the workspace green.
    fn doc_blocks(module: &str) -> Vec<(String, Vec<String>)> {
        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        let mut open: Option<(String, Vec<String>)> = None;
        for line in read(module).lines() {
            let trimmed = line.trim_start();
            let Some(doc) = trimmed
                .strip_prefix("///")
                .or_else(|| trimmed.strip_prefix("//!"))
            else {
                continue;
            };
            let doc = doc.strip_prefix(' ').unwrap_or(doc);
            match doc.strip_prefix("```") {
                Some(attributes) => match open.take() {
                    Some(block) => out.push(block),
                    None => open = Some((attributes.trim().to_string(), Vec::new())),
                },
                None => {
                    if let Some((_, body)) = open.as_mut() {
                        body.push(doc.to_string());
                    }
                }
            }
        }
        assert!(
            open.is_none(),
            "a doc code block in {module} never closed its fence"
        );
        out
    }

    /// **The inventory and the crate root name the same types, in both directions.**
    ///
    /// One direction alone is half a gate: a name deleted from `lib.rs` is caught by the first, and
    /// a name added to it by the second. This is the same reason register #19's negative cases are
    /// pairs.
    #[test]
    fn the_inventory_names_exactly_what_the_crate_root_re_exports() {
        let exported = public_names();
        let inventory: BTreeSet<String> = SURFACE.iter().map(|i| i.name.to_string()).collect();
        let missing: Vec<&String> = exported.difference(&inventory).collect();
        let stale: Vec<&String> = inventory.difference(&exported).collect();
        assert!(
            missing.is_empty(),
            "on the public surface and not in the audit: {missing:?} — every public item is either \
             in §12's block or names the ticket that added it"
        );
        assert!(
            stale.is_empty(),
            "in the audit and not on the public surface: {stale:?} — a renamed item makes the \
             inventory a document again"
        );
    }

    /// **The inventory and the source name the same verbs, with the same receivers, in both
    /// directions.**
    ///
    /// The receiver is half of what is checked, which is what makes spec §12's precedence rule 4 a
    /// gate rather than a paragraph: a `&self` quietly widened to `&mut self` fails here.
    #[test]
    fn the_inventory_names_exactly_the_verbs_the_source_has() {
        let source = verbs_in_source();
        for item in SURFACE {
            if item.kind == Kind::Function {
                continue;
            }
            let empty = BTreeMap::new();
            let actual = source.get(item.name).unwrap_or(&empty);
            let recorded: BTreeSet<&str> = item.verbs.iter().map(|v| v.name).collect();
            let found: BTreeSet<&str> = actual.keys().map(String::as_str).collect();
            assert_eq!(
                recorded, found,
                "`{}`'s verbs disagree between the audit and the source",
                item.name
            );
            for verb in item.verbs {
                let (recv, _) = actual[verb.name];
                assert_eq!(
                    verb.recv, recv,
                    "`{}::{}` takes {recv:?} in the source and {:?} in the audit — precedence rule \
                     4 is that `&self` goes wherever a call only reads and `&mut self` only where \
                     exclusivity is the guarantee",
                    item.name, verb.name, verb.recv
                );
            }
        }
        let inventoried: BTreeSet<&str> = SURFACE
            .iter()
            .filter(|i| !i.verbs.is_empty())
            .map(|i| i.name)
            .collect();
        let with_verbs: BTreeSet<&str> = source.keys().map(String::as_str).collect();
        assert_eq!(
            inventoried, with_verbs,
            "a re-exported type grew or lost its whole set of verbs without the audit moving"
        );
    }

    /// **The two free functions are the two ADR 0005 exported, and there is no third.**
    #[test]
    fn the_only_free_functions_are_the_text_tables() {
        let free: Vec<&str> = SURFACE
            .iter()
            .filter(|i| i.kind == Kind::Function)
            .map(|i| i.name)
            .collect();
        assert_eq!(free, ["graphemes", "width_of"]);
    }

    /// **Every item §12's block does not list names the ticket that added it.**
    ///
    /// Nine types and thirty-seven functions, and the point of the count is that it is a count: a
    /// tenth type arriving without a ticket beside it fails, and a tenth type arriving *with* one is
    /// ordinary work that shows up in the diff of this file.
    ///
    /// # Two backlogs, and the second one is why this reads a prefix list rather than one prefix
    ///
    /// It said `impl NN` and nothing else until production ticket 07, which is the first item on
    /// this surface added after the implementation backlog closed. **The gate was widened rather
    /// than the citation bent**: `Screen::suspend` really does come from
    /// `.scratch/vitui-engine-production/issues/07`, and writing `impl 07` there would have sent a
    /// reader to a resolved ticket about something else. That is the same repair the runtime's
    /// register made for its entry 40, one crate over, for the same reason — a destination gate that
    /// admits only one backlog is a gate that asks the next ticket to lie about where it came from.
    #[test]
    fn everything_outside_spec_12s_block_names_the_ticket_that_added_it() {
        /// The backlogs an item may have come from, longest-lived first.
        const BACKLOGS: [&str; 2] = ["impl ", "production "];
        let cited = |by: &str| BACKLOGS.iter().any(|prefix| by.starts_with(prefix));
        let mut types = 0;
        let mut verbs = 0;
        for item in SURFACE {
            if let Origin::Added { by, why } = item.origin {
                types += 1;
                assert!(
                    cited(by),
                    "`{}` names `{by}`, which is neither an implementation ticket nor a \
                     production one",
                    item.name
                );
                assert!(
                    why.len() > 30,
                    "`{}`'s reason is too short to be one",
                    item.name
                );
            }
            for verb in item.verbs {
                if let Origin::Added { by, why } = verb.origin {
                    verbs += 1;
                    assert!(
                        cited(by),
                        "`{}::{}` names `{by}`, which is neither an implementation ticket nor a \
                         production one",
                        item.name,
                        verb.name
                    );
                    assert!(
                        why.len() > 30,
                        "`{}::{}`'s reason is too short to be one",
                        item.name,
                        verb.name
                    );
                }
            }
        }
        assert_eq!((types, verbs), (9, 37), "the audit's own numbers moved");
    }

    /// **The counts, as the audit recorded them.**
    ///
    /// Forty-nine types and one hundred and six functions, against §12's *twenty-one public types
    /// and about sixty-three functions* — a sentence its own block never agreed with. The
    /// arithmetic is stated so that a reader can check it rather than trust it: 41 − 1 + 9 = 49.
    ///
    /// It was one hundred and five until architecture ticket 21 deleted `Screen::link`; the type
    /// count did not move, because `LinkId` left the listing and `Link<'a>` joined it. **Production
    /// ticket 07 took it to one hundred and six** with `Screen::suspend` and `Screen::resume`, and
    /// the type count did not move there either: the pair is two verbs on a type that was already
    /// listed, and neither of them returns anything.
    #[test]
    fn the_counts_are_the_ones_the_audit_recorded() {
        let types = SURFACE.iter().filter(|i| i.kind != Kind::Function).count();
        let functions = SURFACE.iter().map(|i| i.verbs.len()).sum::<usize>()
            + SURFACE.iter().filter(|i| i.kind == Kind::Function).count();
        assert_eq!(types, 49, "the public type count moved");
        assert_eq!(functions, 106, "the public function count moved");
        let added = SURFACE
            .iter()
            .filter(|i| i.kind != Kind::Function)
            .filter(|i| matches!(i.origin, Origin::Added { .. }))
            .count();
        assert_eq!(
            41 - 1 + added,
            types,
            "§12's forty-one types, less `Resolver`, plus what the tickets added, is not what is here"
        );
    }

    /// **Zero public traits, and the scanner is known to work because it finds the one that is not
    /// public.**
    ///
    /// Refusal 5 is the load-bearing one — *the engine has nothing to call upward, so the dependency
    /// arrow is enforced by there being no arrow* — and it is the refusal a single `pub trait` in a
    /// shipped module would end. `crate::scenes` declares `pub trait Scene`, and it is
    /// `cfg(test)`-only inside a private module: it is on nobody's surface, and it is the positive
    /// twin that stops this gate from passing because the scan found nothing at all.
    #[test]
    fn there_are_no_public_traits() {
        let declared = declarations();
        let traits: Vec<&(String, String, String)> = declared
            .iter()
            .filter(|(_, kind, _)| kind == "trait")
            .collect();
        let shipped: Vec<&&(String, String, String)> = traits
            .iter()
            .filter(|(file, _, _)| !TEST_ONLY_MODULES.contains(&file.as_str()))
            .collect();
        assert!(
            shipped.is_empty(),
            "a public trait arrived in a shipped module: {shipped:?} — refusal 5 is that there are \
             zero of them"
        );
        assert!(
            traits
                .iter()
                .any(|(file, _, name)| file == "scenes.rs" && name == "Scene"),
            "the scan did not find `Scene` in scenes.rs, so it is checking a file that moved rather \
             than a crate with no traits"
        );
        // And the twin's own premise: `scenes.rs` is exempt because it is `cfg(test)`, which
        // `the_test_only_list_is_what_the_crate_root_declares_under_cfg_test` is what checks. Without
        // that gate this assertion is what would have made dropping the attribute invisible — the
        // trait would ship and this line would go on passing, because it requires `Scene` to exist.
        assert!(
            TEST_ONLY_MODULES.contains(&"scenes.rs"),
            "`scenes.rs` is the twin of this gate and is no longer on the test-only list"
        );
    }

    /// **The prelude is the nine names ticket 24 states, and the public modules are it and the fuzz
    /// door.**
    ///
    /// It was *nothing is a public module but the prelude* until ticket 25, and the amendment is
    /// stated rather than quietly widened: `crate::fuzz` is the second, it is behind a non-default
    /// feature, it is `#[doc(hidden)]`, and
    /// [`the_fuzz_door_is_behind_a_feature_and_hidden`] is what holds all three. Two is the number
    /// now; a third fails here whatever it is.
    #[test]
    fn the_prelude_re_exports_exactly_nine_names() {
        let statements = re_export_statements();
        let modules: Vec<&String> = statements
            .iter()
            .filter(|s| s.starts_with("pub mod "))
            .collect();
        assert_eq!(
            modules.len(),
            2,
            "the crate has public modules other than the prelude and the fuzz door: {modules:?}"
        );
        assert!(
            modules.iter().any(|m| m.as_str() == "pub mod fuzz;"),
            "the second public module is not the fuzz door: {modules:?}"
        );
        let root = root();
        let start = root
            .find("pub mod prelude")
            .expect("the prelude is in the crate root");
        let block = &root[start..];
        let close = block.find("\n}").expect("the prelude module closes");
        let block = &block[..close];
        assert_eq!(
            block.matches("pub use").count(),
            1,
            "the prelude re-exports in more than one statement, so nine names in one place is no \
             longer what a reader gets"
        );
        let statement = &block[block.find("pub use").expect("the prelude re-exports")..];
        let open = statement
            .find('{')
            .expect("the prelude's re-export is braced");
        let end = statement[open..]
            .find('}')
            .expect("the prelude's re-export closes")
            + open;
        let names: Vec<&str> = statement[open + 1..end]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(
            names, *PRELUDE,
            "the prelude re-exports something other than the nine names, in that order"
        );
    }

    /// **The fuzz door is behind a feature, hidden from rustdoc, and off by default.**
    ///
    /// `crate::fuzz` is the only public module here but the prelude, and it exists because the gate
    /// and the soak have to be the same code: §14's inversion makes the committed corpus the gate,
    /// so the replay test and the fuzz target call one function. The target is in another crate —
    /// `cargo-fuzz` needs nightly and `libfuzzer-sys`, so `fuzz/` is its own workspace — and a
    /// caller in another crate can only reach what is `pub`.
    ///
    /// Three conditions make that a door rather than a hole, and each fails on its own:
    ///
    /// 1. The declaration is gated on `any(test, feature = "fuzz")`, so an ordinary build compiles
    ///    none of it.
    /// 2. It carries `#[doc(hidden)]`, so it is absent from what a runtime author reads.
    /// 3. The manifest declares the feature and **no default feature list turns it on**, which is
    ///    the one of the three a `Cargo.toml` edit could undo silently.
    #[test]
    fn the_fuzz_door_is_behind_a_feature_and_hidden() {
        let root = root();
        let at = root
            .find("pub mod fuzz;")
            .expect("the crate root declares the fuzz door");
        // The attributes immediately above it, which is where a `cfg` and a `doc(hidden)` have to be
        // for either to mean anything about this declaration.
        let above: Vec<&str> = root[..at].lines().rev().take(4).collect();
        assert!(
            above
                .iter()
                .any(|line| line.trim() == r#"#[cfg(any(test, feature = "fuzz"))]"#),
            "`pub mod fuzz;` is not gated on the fuzz feature: {above:?}"
        );
        assert!(
            above.iter().any(|line| line.trim() == "#[doc(hidden)]"),
            "`pub mod fuzz;` is not hidden from rustdoc: {above:?}"
        );

        let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .expect("this crate's manifest is beside its source");
        assert!(
            manifest.contains("\nfuzz = []"),
            "the `fuzz` feature is not declared in Cargo.toml, so `--features fuzz` is a typo \
             nothing catches"
        );
        // A `default = [...]` list naming it would turn every one of the exemptions above into a
        // silent inclusion: `shipped_modules` would still skip the files, and an ordinary dependent
        // would compile them anyway.
        assert!(
            !manifest.contains("default = ["),
            "this crate has a default feature list, and `fuzz` may not be reachable from one"
        );
    }

    /// **The soak-only list is exactly what the crate root declares under the fuzz feature, in both
    /// directions.**
    ///
    /// [`the_test_only_list_is_what_the_crate_root_declares_under_cfg_test`]'s twin, one `cfg`
    /// along, and it exists for the identical reason: [`SOAK_ONLY_MODULES`] exempts a module from
    /// every scan over the shipped surface, and until this gate existed the only thing checked about
    /// a name on it was that the file still existed. A module that stops being soak-only — the exact
    /// move ticket 25 made with `reference.rs`, in the other direction — would stay exempt.
    #[test]
    fn the_soak_only_list_is_what_the_crate_root_declares_under_the_fuzz_feature() {
        let root = root();
        let lines: Vec<&str> = root.lines().collect();
        let mut declared: BTreeSet<String> = BTreeSet::new();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // `mod name;` and `pub mod name;` both, because the fuzz door is the second and the
            // reference compositor is the first.
            let Some(rest) = trimmed
                .strip_prefix("mod ")
                .or_else(|| trimmed.strip_prefix("pub mod "))
                .filter(|rest| rest.ends_with(';'))
            else {
                continue;
            };
            let gated = lines[..index]
                .iter()
                .rev()
                .take_while(|earlier| {
                    let earlier = earlier.trim();
                    earlier.starts_with("#[") || earlier.starts_with("//")
                })
                .any(|earlier| earlier.trim() == r#"#[cfg(any(test, feature = "fuzz"))]"#);
            if gated {
                declared.insert(format!("{}.rs", rest.trim_end_matches(';').trim()));
            }
        }
        let named: BTreeSet<String> = SOAK_ONLY_MODULES.iter().map(|m| m.to_string()).collect();
        assert_eq!(
            named, declared,
            "`SOAK_ONLY_MODULES` and the crate root's `#[cfg(any(test, feature = \"fuzz\"))] mod` \
             declarations disagree. A module that stops being soak-only stays exempt from every scan \
             in this file unless this is checked"
        );
        for module in SOAK_ONLY_MODULES {
            assert!(
                src_dir().join(module).exists(),
                "`{module}` is named soak-only and does not exist, so this list is stale"
            );
            assert!(
                !TEST_ONLY_MODULES.contains(module),
                "`{module}` is on both lists, and the two `cfg`s it would need are different"
            );
        }
    }

    /// **The refused names are not re-exported, and the twin names one that is.**
    ///
    /// A list of absences passes for any reason at all, including the crate root having been
    /// renamed out from under it — which is exactly the failure mode register #19's pairs exist to
    /// close.
    ///
    /// **A `Type::verb` entry is checked against the source's verbs**, because a method is not a
    /// re-export and a name check over `lib.rs` cannot see one. The twin for that half is the
    /// receiving type being *found*: an entry naming a type nothing declares fails here, so a
    /// refused verb cannot be satisfied by its type having gone away.
    #[test]
    fn the_refused_names_are_not_re_exported() {
        let exported = public_names();
        let verbs = verbs_in_source();
        for (name, why) in REFUSED_NAMES {
            match name.split_once("::") {
                Some((target, verb)) => {
                    assert!(
                        exported.contains(target),
                        "`{name}` names `{target}`, which is not on the public surface — a refused \
                         verb whose type has gone away is a refusal nothing can fail"
                    );
                    let empty = BTreeMap::new();
                    let found = verbs.get(target).unwrap_or(&empty);
                    assert!(
                        !found.contains_key(verb),
                        "`{name}` is on the public surface: {why}"
                    );
                }
                None => assert!(
                    !exported.contains(*name),
                    "`{name}` is on the public surface: {why}"
                ),
            }
        }
        for present in ["Capabilities", "Screen", "Slot", "View"] {
            assert!(
                exported.contains(present),
                "`{present}` is not re-exported, so this gate is reading a file that moved"
            );
        }
        assert!(
            verbs["Screen"].contains_key("layers"),
            "the verb scan found no `Screen::layers`, so the `Type::verb` half of this gate is \
             reading a source it cannot parse rather than a crate with no `Screen::link`"
        );
    }

    /// **Refusal 12 is a lint on the crate and not a claim in a document.**
    #[test]
    fn the_crate_root_forbids_unsafe_code() {
        assert!(
            attribute_on_the_crate_root("#![forbid(unsafe_code)]"),
            "the crate root no longer forbids `unsafe`, which is refusal 12"
        );
    }

    /// Whether a crate attribute is **on** the crate root, as a line of its own.
    ///
    /// Not `contains`, and the difference is a gate that passed while doing nothing: the crate
    /// documentation *names* `#![forbid(unsafe_code)]` in the prose that introduces the refusals, so
    /// a `contains` over the whole file goes on finding it after the attribute itself has been
    /// downgraded to a `warn`. Found by mutation, which is the only way that class of defect is
    /// found.
    /// A module's source with every comment line dropped.
    ///
    /// Used where a name in prose would otherwise satisfy a gate about a name in code.
    fn code_of(text: &str) -> String {
        text.lines()
            .filter(|line| {
                let line = line.trim_start();
                !(line.starts_with("//") || line.starts_with("///") || line.starts_with("//!"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn attribute_on_the_crate_root(attribute: &str) -> bool {
        root().lines().any(|line| line.trim() == attribute)
    }

    /// **Refusal 4, over the signatures: nothing on the surface takes a closure or an iterator.**
    ///
    /// *Coordinates and `&str` go in; nothing of yours is held.* The verb this was decided at is
    /// `restyle`, which took `impl Fn(Style) -> Style` in ticket 12's skeleton and takes `&Restyle`
    /// since impl 16 — and `fill_with` is on §12's priced-rather-than-overlooked list for the same
    /// reason. `graphemes` **returns** an iterator over a borrowed `&str`, which is the other
    /// direction and is ADR 0005.
    #[test]
    fn no_public_verb_takes_a_closure_or_an_iterator() {
        // Bare, because `impl Fn` is only the shorthand: `restyle` took `impl Fn(Style) -> Style`
        // in ticket 12's skeleton, and `fn probe<F: Fn(Style) -> Style>(&mut self, f: F)` is the
        // same parameter written the other way. A `where F: Fn(…)` clause is a third spelling and is
        // why `bindings_of` reaches past the return type to pick the clause up.
        for (target, verbs) in verbs_in_source() {
            for (name, (_, signature)) in verbs {
                let bindings = bindings_of(&signature);
                for banned in ["Fn(", "FnMut(", "FnOnce(", "Iterator", "IntoIterator"] {
                    assert!(
                        !bindings.contains(banned),
                        "`{target}::{name}` takes `{banned}`, which is refusal 4: the engine never \
                         iterates application data and holds nothing of the caller's. The \
                         signature is `{signature}`"
                    );
                }
            }
        }
    }

    /// A signature's generics, parameters and `where` clause — **everything but the return type**.
    ///
    /// The return type is deliberately excluded rather than overlooked. `graphemes` returns
    /// `impl Iterator` and ADR 0005 is why; refusal 4 is about what the engine is *handed*, and a
    /// verb that hands an iterator over the engine's own tables back to the caller iterates nothing
    /// of the caller's.
    fn bindings_of(signature: &str) -> String {
        let mut depth = 0i32;
        let mut params_end = signature.len();
        for (index, ch) in signature.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        params_end = index + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        let mut out = signature[..params_end].to_string();
        if let Some(at) = signature.find(" where ") {
            out.push(' ');
            out.push_str(&signature[at..]);
        }
        out
    }

    /// **Every module in `src/` is either parsed by these gates or named as test-only.**
    ///
    /// The gates above are all queries over `shipped_modules()`, so a module that is neither parsed
    /// nor declared test-only is a module whose public items nothing here has looked at.
    #[test]
    fn every_module_is_either_shipped_or_named_test_only() {
        let root = root();
        let mut unseen = Vec::new();
        for module in all_modules() {
            if module == "lib.rs" {
                continue;
            }
            // A `#[path]`-included file is declared under a name of its own — `input/tests.rs` is
            // `mod input_tests` — so the path is what to look for, and a plain module is looked up
            // by its stem.
            let stem = module
                .rsplit('/')
                .next()
                .expect("a path has a last segment")
                .trim_end_matches(".rs");
            // Declared by the crate root, or by the module whose directory it sits in — which is
            // how `ucd/tests.rs` is declared, by `ucd.rs`.
            let parent = match module.rsplit_once('/') {
                Some((directory, _)) => read(&format!("{directory}.rs")),
                None => String::new(),
            };
            let declared = |text: &str| {
                text.contains(&format!("mod {stem};"))
                    || text.contains(&format!("mod {stem} "))
                    || text.contains(&format!("\"{module}\""))
            };
            if declared(&root) || declared(&parent) {
                continue;
            }
            unseen.push(module);
        }
        assert!(
            unseen.is_empty(),
            "modules under src/ that the crate root does not declare: {unseen:?}"
        );
    }

    /// **The test-only list is exactly what the crate root declares under `#[cfg(test)]`, in both
    /// directions.**
    ///
    /// [`TEST_ONLY_MODULES`] exempts a module from every scan in this file, and until this gate
    /// existed the only thing checked about it was that the files still existed. A module that stops
    /// being test-only would have stayed silently exempt — and `lib.rs` predicts exactly that
    /// happening for `reference` at ticket 25. Dropping `#[cfg(test)]` from `mod scenes;` would have
    /// shipped `pub trait Scene` with the zero-trait gate still green, because that gate's positive
    /// twin *requires* `Scene` to be found.
    #[test]
    fn the_test_only_list_is_what_the_crate_root_declares_under_cfg_test() {
        // Over **every** module and not only the crate root: `ucd/tests.rs` is declared by
        // `ucd.rs`, so a scan of `lib.rs` alone reports it as undeclared and the two sets never
        // agree. A `mod` in `foo.rs` resolves against `foo/`, which is what makes the path
        // `ucd/tests.rs`.
        let mut declared: BTreeSet<String> = BTreeSet::new();
        for module in all_modules() {
            let text = read(&module);
            let lines: Vec<&str> = text.lines().collect();
            let directory = match module.as_str() {
                "lib.rs" => String::new(),
                _ => format!("{}/", module.trim_end_matches(".rs")),
            };
            for (index, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                // `mod name;` and never `mod name {`: an inline test module is not a file, and this
                // list is a list of files.
                let Some(rest) = trimmed
                    .strip_prefix("mod ")
                    .filter(|rest| rest.ends_with(';'))
                else {
                    continue;
                };
                let name = rest.trim_end_matches(';').trim();
                let attributes: Vec<&str> = lines[..index]
                    .iter()
                    .rev()
                    .take_while(|earlier| {
                        let earlier = earlier.trim();
                        earlier.starts_with("#[") || earlier.starts_with("//")
                    })
                    .copied()
                    .collect();
                if !attributes.iter().any(|a| a.trim() == "#[cfg(test)]") {
                    continue;
                }
                // A `#[path]` attribute names the file outright; otherwise it is the module's stem
                // under the declaring module's own directory.
                let path = attributes
                    .iter()
                    .find_map(|attribute| {
                        let at = attribute.find("#[path = \"")? + "#[path = \"".len();
                        let end = attribute[at..].find('"')? + at;
                        Some(attribute[at..end].to_string())
                    })
                    .unwrap_or_else(|| format!("{directory}{name}.rs"));
                declared.insert(path);
            }
        }
        // This file is `#[cfg(test)] mod audit;` too, so it is in both sets by the same rule.
        let named: BTreeSet<String> = TEST_ONLY_MODULES.iter().map(|m| m.to_string()).collect();
        assert_eq!(
            named, declared,
            "`TEST_ONLY_MODULES` and the crate root's `#[cfg(test)] mod` declarations disagree. A \
             module that stops being test-only stays exempt from every scan in this file unless this \
             is checked"
        );
        for module in TEST_ONLY_MODULES {
            assert!(
                src_dir().join(module).exists(),
                "`{module}` is named test-only and does not exist, so this list is stale"
            );
        }
    }

    /// **The threading contract's crossing half, asserted by the compiler.**
    ///
    /// The other half is negative and cannot be a bound, so it is register #19's paired
    /// `compile_fail` doctests on [`Screen`](crate::Screen), [`View`](crate::View),
    /// [`Permit`](crate::Permit) and [`LayerStack`](crate::LayerStack) — a `!Send` type is not
    /// expressible as a bound, and this is the direction that is.
    #[test]
    fn the_threading_contract_crosses_where_the_table_says_it_does() {
        fn crosses<T: Send>() {}
        fn shared<T: Send + Sync>() {}
        crosses::<Engine>();
        crosses::<Surface>();
        crosses::<Slot<Vec<u8>>>();
        shared::<WakeHandle>();
        shared::<Slot<Vec<u8>>>();
        fn cloned<T: Clone>() {}
        cloned::<WakeHandle>();
    }

    /// **Every refusal is held by a compile outcome, a lint or an asserted absence — never by
    /// prose.**
    ///
    /// The register is checked rather than read: a `Pair` names a module that must carry a
    /// `compile_fail`, an `Absence` names a test function that must exist, a `Lint` names an
    /// attribute that must be on the crate root, and an `Internal` must say why it is out of reach.
    #[test]
    fn every_refusal_is_held_by_something_that_fails() {
        let sources: BTreeMap<String, String> = all_modules()
            .into_iter()
            .map(|module| {
                let text = read(&module);
                (module, text)
            })
            .collect();
        assert_eq!(REFUSALS.iter().filter(|r| r.number != 0).count(), 12);
        assert_eq!(REFUSALS.iter().filter(|r| r.number == 0).count(), 6);
        for refusal in REFUSALS {
            assert!(
                !refusal.evidence.is_empty(),
                "refusal {} is documented only in prose: {}",
                refusal.number,
                refusal.what
            );
            for evidence in refusal.evidence {
                match evidence {
                    Evidence::Pair { module, hostile } => {
                        let held = doc_blocks(module).into_iter().any(|(attributes, body)| {
                            attributes.contains("compile_fail")
                                && body.iter().any(|line| line.contains(*hostile))
                        });
                        assert!(
                            held,
                            "refusal {}'s own negative case is gone: no `compile_fail` block in \
                             `{module}` contains `{hostile}`. *This file has a compile_fail in it \
                             somewhere* stays true after the one case that mattered was deleted, \
                             which is why the fragment is named",
                            refusal.number
                        );
                    }
                    Evidence::Absence { at } => {
                        // Over code rather than raw text: a deleted gate whose name survives in the
                        // prose that explains it would otherwise go on counting as evidence for
                        // itself.
                        assert!(
                            sources
                                .iter()
                                .any(|(_, text)| code_of(text).contains(&format!("fn {at}("))),
                            "refusal {}'s gate `{at}` does not exist",
                            refusal.number
                        );
                    }
                    Evidence::Lint { attribute } => {
                        assert!(
                            attribute_on_the_crate_root(attribute),
                            "refusal {}'s lint `{attribute}` is not on the crate root",
                            refusal.number
                        );
                    }
                    Evidence::Internal { why } => {
                        assert!(
                            refusal.number == 0 && why.len() > 60,
                            "refusal {} claims to be internal without saying why",
                            refusal.number
                        );
                    }
                }
            }
        }
    }

    /// **The corpus is thirty-six cases, and the count is an equality.**
    ///
    /// The pair is the unit: deleting the hostile line is caught by the first half and **renaming the
    /// item it protects is caught only by the second**, because a rename makes the negative case fail
    /// for `E0433` instead of `E0277` and the mechanism cannot tell those apart. A count catches the
    /// third edit — a case removed entirely, twin and all, which leaves every remaining test green.
    #[test]
    fn the_negative_corpus_is_the_size_the_audit_recorded() {
        let mut negative = 0;
        let mut runnable = 0;
        // The crate root and the shipped modules, which is exactly what rustdoc documents. A
        // `cfg(test)` module is absent from the doc build, so a fence inside one is compiled by
        // nothing at all — counting it here would make these two numbers disagree with what
        // `cargo test --doc` reports, for no reason a reader could see.
        let documented = std::iter::once("lib.rs".to_string()).chain(shipped_modules());
        for module in documented {
            for (attributes, _) in doc_blocks(&module) {
                if attributes.contains("compile_fail") {
                    negative += 1;
                } else if attributes.is_empty() {
                    runnable += 1;
                }
            }
        }
        // And the other half of that: a negative case written into a `cfg(test)` module looks
        // exactly like a gate and is not one, because rustdoc never sees it. Same family as impl
        // 08's *a gate whose subject can be skipped must prove the subject ran*.
        //
        // `SOAK_ONLY_MODULES` is on it for the identical reason one `cfg` along. `cargo doc` builds
        // default features, the `fuzz` feature is not one, and `crate::fuzz` carries `#[doc(hidden)]`
        // besides — so a fence in either of those two files is compiled by nothing at all.
        for module in TEST_ONLY_MODULES.iter().chain(SOAK_ONLY_MODULES) {
            // Rust blocks only. A ```text fence is a picture and `crate::gates` has one — the
            // golden frame's own format — and rustdoc would not compile that in a shipped module
            // either.
            let fences = doc_blocks(module)
                .into_iter()
                .filter(|(attributes, _)| {
                    attributes.is_empty() || attributes.contains("compile_fail")
                })
                .count();
            assert_eq!(
                fences, 0,
                "`{module}` is compiled by neither an ordinary build nor the doc build, and \
                 carries {fences} Rust doc block(s). Nothing compiles them, so a `compile_fail` \
                 there is a case that cannot fail"
            );
        }
        assert_eq!(
            negative, NEGATIVE_CASES,
            "the negative corpus changed size. A case removed with its twin leaves every other test \
             green, which is why this is a count"
        );
        assert_eq!(
            runnable, RUNNABLE_EXAMPLES,
            "the runnable examples changed in number. **The pair is the unit**: counting the hostile \
             lines and not the twins leaves unguarded the half that catches a rename"
        );
    }

    /// **`Config` is a plain struct with `Default` and struct update, and there is no builder.**
    ///
    /// *Every knob visible in one place is worth more than a chain of setters* — and the way a
    /// builder arrives is one `with_` on the config, so the shape is gated rather than intended.
    #[test]
    fn config_is_a_plain_struct_with_no_builder() {
        fn defaulted<T: Default>() {}
        defaulted::<crate::Config>();
        let config = SURFACE
            .iter()
            .find(|i| i.name == "Config")
            .expect("`Config` is on the surface");
        assert!(
            config.verbs.is_empty(),
            "`Config` grew verbs: {:?} — a builder is a chain of setters and this is where the \
             first one lands",
            config.verbs.iter().map(|v| v.name).collect::<Vec<_>>()
        );
        assert_eq!(config.kind, Kind::Struct);
    }

    /// **The three moves are §12's three, and the value receivers are all `Copy`.**
    ///
    /// [`Recv`] records syntax, and `self` by value is a read on a `Copy` type and an ownership
    /// transfer on everything else. This is the gate that keeps the two apart: the types with a
    /// value receiver are exactly the five `Copy` ones plus [`Engine`], whose one value receiver is
    /// `attach` and is in [`MOVES`].
    #[test]
    fn the_types_that_read_by_value_are_copy() {
        fn copy<T: Copy>() {}
        copy::<crate::Buttons>();
        copy::<crate::Mix>();
        copy::<crate::Mods>();
        copy::<crate::Rect>();
        copy::<crate::Style>();
        let by_value: BTreeSet<&str> = SURFACE
            .iter()
            .filter(|i| i.verbs.iter().any(|v| v.recv == Recv::Value))
            .map(|i| i.name)
            .collect();
        let mut expected: BTreeSet<&str> = READ_BY_VALUE.iter().copied().collect();
        expected.insert("Engine");
        assert_eq!(
            by_value, expected,
            "a type reads by value that is neither `Copy` nor transferring ownership"
        );
        for (target, verb, why) in MOVES {
            let item = SURFACE
                .iter()
                .find(|i| i.name == *target)
                .unwrap_or_else(|| panic!("`{target}` is on the surface"));
            assert!(
                item.verbs.iter().any(|v| v.name == *verb),
                "`{target}::{verb}` is in MOVES and not on the surface"
            );
            assert!(
                why.len() > 40,
                "`{target}::{verb}`'s reason is too short to be one"
            );
        }
        assert_eq!(MOVES.len(), 3, "§12's precedence list names three moves");
    }
}
