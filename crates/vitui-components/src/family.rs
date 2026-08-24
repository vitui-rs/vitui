//! The fifteen families of the union catalogue, and the module each one is the home of.
//!
//! Spec §19: **a module is a family.** The tree follows the survey's families so that a reader who
//! knows what they want finds it without a search, and [`crate::Component::families`] is the join
//! that makes the mapping checkable rather than a naming convention.
//!
//! # The spelling is not a decision, and the rule is
//!
//! Spec §19 is explicit: `architecture.md` §2 lists sixteen module directories under the
//! pre-correction *ten families*, and **no ticket ratified the directory names**. They are a
//! starting point, not a decision, and renaming one reopens nothing. What is settled is the join —
//! a component's home module is the module of the first family it declares — and
//! `inventory::tests::the_module_tree_and_the_families_column_agree` is the gate that refuses a
//! component whose module and whose `families` column disagree.
//!
//! # F15 has no module here, and that is the T2 test rather than an omission
//!
//! Spec §18's F15 row reads *not here — `vitui-runtime`*, by `COMPONENT-HIERARCHY.md`'s T2 test:
//! all twenty-three of its entries **emit no cells**. [`Family::module`] returns `None` for it, and
//! the `None` is the statement — a family with an empty module directory would read as *not yet
//! done*, which is exactly the confusion spec §18 says is never allowed to arise.

/// One family of spec §18's union catalogue.
///
/// The discriminants are the survey's own numbering, which is how every other document refers to
/// them; [`Family::name`] carries the word §18's table uses.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Family {
    /// F1 text — `text`, `chip`, `field`, `fit`. ~48 entries.
    F1Text,
    /// F2 structure — `panel`, `rule`, `block`, operator layers. ~38 entries.
    F2Structure,
    /// F3 scrolling — `scroll_area`, `scrollbar`, `sticky`, `collection`. 18 entries.
    F3Scrolling,
    /// F4 disclosure — `collapsible`, and the family exists to say that all twenty-one entries are
    /// one state machine.
    F4Disclosure,
    /// F5 indicators — `meter`, `chart`, `plot`, `overlay`, deadlines. ~46 entries.
    F5Indicators,
    /// F6 input — `field`, `button`, `chip`, `select`, `collection`, `slider`. ~110 entries.
    F6Input,
    /// F7 collections — `collection` plus columns, an index or tiles. ~50 entries.
    F7Collections,
    /// F8 navigation — `collection` + `overlay`. ~35 entries, and no v1 component of its own.
    F8Navigation,
    /// F9 overlays — `overlay`, layers, placement, scopes, `Trap`. ~35 entries.
    F9Overlays,
    /// F10 charts — `chart`, `plot`, axes, two memos. ~55 entries, every one of them one rasteriser
    /// and one mapping.
    F10Charts,
    /// F11 media — a picture as `Solid`/`Half`, QR, waveform, player chrome. 16 entries, and no v1
    /// component: §14 built the mechanisms and the freeze ships none of them as a row.
    F11Media,
    /// F12 files — `collection` + `scroll_area` + worker + the preview pane. ~35 entries.
    F12Files,
    /// F13 system — `meter`, `table`, `plot`, a log `collection`. ~30 entries, no v1 component.
    F13System,
    /// F14 terminal-native — `canvas` at three rungs, `pty`. ~24 entries, no v1 component; `pty` is
    /// one of the two §18 exemplars that were **not** built (§22).
    F14TerminalNative,
    /// F15 behavioural — **not here**. All twenty-three entries emit no cells, so they are
    /// `vitui-runtime`'s by the T2 test. [`Family::module`] is `None`.
    F15Behavioural,
}

impl Family {
    /// Every family, which is what the join gate iterates.
    ///
    /// The array is also what keeps [`Family::F15Behavioural`] constructed: warnings are denied
    /// workspace-wide, and a variant no row of [`crate::INVENTORY`] names would otherwise be a
    /// build failure rather than the deliberate absence spec §18 records.
    pub const ALL: [Family; 15] = [
        Family::F1Text,
        Family::F2Structure,
        Family::F3Scrolling,
        Family::F4Disclosure,
        Family::F5Indicators,
        Family::F6Input,
        Family::F7Collections,
        Family::F8Navigation,
        Family::F9Overlays,
        Family::F10Charts,
        Family::F11Media,
        Family::F12Files,
        Family::F13System,
        Family::F14TerminalNative,
        Family::F15Behavioural,
    ];

    /// Its number in the survey, 1..=15.
    pub const fn number(self) -> u8 {
        self as u8 + 1
    }

    /// The word spec §18's table uses.
    pub const fn name(self) -> &'static str {
        match self {
            Family::F1Text => "text",
            Family::F2Structure => "structure",
            Family::F3Scrolling => "scrolling",
            Family::F4Disclosure => "disclosure",
            Family::F5Indicators => "indicators",
            Family::F6Input => "input",
            Family::F7Collections => "collections",
            Family::F8Navigation => "navigation",
            Family::F9Overlays => "overlays",
            Family::F10Charts => "charts",
            Family::F11Media => "media",
            Family::F12Files => "files",
            Family::F13System => "system",
            Family::F14TerminalNative => "terminal-native",
            Family::F15Behavioural => "behavioural",
        }
    }

    /// The module directory this family is the home of, or `None` when the family is not this
    /// crate's.
    ///
    /// The name is a spelling and not a decision (§19). The `None` is a decision: F15 emits no
    /// cells and belongs to `vitui-runtime`.
    pub const fn module(self) -> Option<&'static str> {
        Some(match self {
            Family::F1Text => "text",
            Family::F2Structure => "structure",
            Family::F3Scrolling => "scroll",
            Family::F4Disclosure => "disclose",
            Family::F5Indicators => "indicate",
            Family::F6Input => "input",
            Family::F7Collections => "collect",
            Family::F8Navigation => "nav",
            Family::F9Overlays => "overlay",
            Family::F10Charts => "chart",
            Family::F11Media => "media",
            Family::F12Files => "files",
            Family::F13System => "monitor",
            Family::F14TerminalNative => "canvas",
            Family::F15Behavioural => return None,
        })
    }

    /// The ids this family's module is the home of, in [`crate::INVENTORY`]'s order.
    ///
    /// This is the second half of the join. The first half is the `families` column; this is the
    /// module tree saying the same thing from the other side, so that a component moved between
    /// modules without its row moving fails `the_module_tree_and_the_families_column_agree` in
    /// both directions.
    pub const fn members(self) -> &'static [&'static str] {
        match self {
            Family::F1Text => crate::text::MEMBERS,
            Family::F2Structure => crate::structure::MEMBERS,
            Family::F3Scrolling => crate::scroll::MEMBERS,
            Family::F4Disclosure => crate::disclose::MEMBERS,
            Family::F5Indicators => crate::indicate::MEMBERS,
            Family::F6Input => crate::input::MEMBERS,
            Family::F7Collections => crate::collect::MEMBERS,
            Family::F8Navigation => crate::nav::MEMBERS,
            Family::F9Overlays => crate::overlay::MEMBERS,
            Family::F10Charts => crate::chart::MEMBERS,
            Family::F11Media => crate::media::MEMBERS,
            Family::F12Files => crate::files::MEMBERS,
            Family::F13System => crate::monitor::MEMBERS,
            Family::F14TerminalNative => crate::canvas::MEMBERS,
            Family::F15Behavioural => &[],
        }
    }
}
