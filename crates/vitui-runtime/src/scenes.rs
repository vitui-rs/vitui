//! The twenty scenes, as a normative list rather than an appendix.
//!
//! > **A scene is removed only by a ticket naming the property it can no longer distinguish.**
//!
//! The engine's own scene list (`crates/vitui-engine/src/scenes.rs`) argues why a scene list is part
//! of a gate, from the case that produced the rule: three scenes that scored identically on every
//! candidate validated the wrong damage model while reporting success, and the two that
//! discriminated were not on the list. That reasoning is not repeated here. What is different one
//! layer up is worth stating, because it changes the shape of this file:
//!
//! **The engine's scenes are one type driven by one harness; the runtime's are not.** A wheel chain,
//! a 1M-row list, a superseded decode and a theme swap have nothing in common to be a `trait Scene`
//! over — no shared `step`, no shared unit, no number that could be compared across them. So this
//! list is a **register of where each scene lives**, and the numbers stay in the twenty-odd places
//! that already produce them.
//!
//! That makes the third rule the load-bearing one:
//!
//! - **Numbers are kept per scene and never summed.** There is no total here and there is no field
//!   one could go in. The engine records that its 27x scroll-detector regression was visible only
//!   because numbers were kept per scene; the runtime's own version is the chunked data
//!   source at **roughly 230% of the whole frame budget**, which disappears into any average taken
//!   over the other nineteen. That figure was 321% when it was first measured, which is the second half of
//!   the same argument: **a number that moves by a third is not a number to average, and it is not a
//!   number to gate on either** — see the `decided`.
//! - **Every scene names where its numbers come from**, and
//!   [`tests::every_scene_names_something_that_exists`] opens the file and looks. A list of prose
//!   would let a scene stop being run without anything saying so, which is the failure the list
//!   exists to prevent.
//! - **A scene may cite a report, and may not rest on one.** Unlike [`crate::register`], where
//!   refinement 2 forbids a *gate* resting on a report, a scene is not a gate: it is a shape the
//!   gates are driven over, and *what it decided* is often a number nobody gates on. But
//!   [`State::Wired`] says **it runs**, and this workspace runs `cargo test` and two named
//!   examples — so a scene whose only instrument is an `examples/*.rs` is compiled and never
//!   evaluated, and calling that wired is a claim the code contradicts. **Twenty wired, none red.**
//!   The chunked scene was the exception until it got a test rather than a rewording: its microsecond
//!   figure is still a report, and what is gated beside it is the relation the figure is evidence
//!   for — `data::tests::a_chunked_source_is_three_orders_off_a_slice_and_doubles_with_the_offset`.
//!   That is the shape available whenever a scene's headline number is a timing, and it is the
//!   shape to reach for before a scene is recorded wired on a file nothing runs.
//!
//! # `decided` is the column that stops a scene from being deleted
//!
//! Every row carries what it decided, in the spec's own figures. That is not decoration: the rule at
//! the top is *a scene is removed only by a ticket naming the property it can no longer
//! distinguish*, and a reader who cannot see what a scene distinguishes will conclude it
//! distinguishes nothing.

use crate::register::{Instrument, State};

/// One scene of the normative list.
#[derive(Clone, Copy, Debug)]
pub struct Scene {
    /// Its number in the table, which is how everything else refers to it.
    pub number: u8,
    /// The scene, in the words.
    pub name: &'static str,
    /// What it decided, with the figures recorded beside it.
    pub decided: &'static str,
    /// Where its numbers are produced, or the ticket that will produce them.
    pub state: State,
}

/// The scene list, scene for scene.
pub const SCENES: [Scene; 20] = [
    Scene {
        number: 1,
        name: "the dense IDE screen, 300x80",
        decided: "312 interactive regions, 43 tab stops, ~33 us a frame",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/screen.rs",
                    name: "the_screen_is_thirty_two_splits_and_a_hundred_and_nineteen_lanes",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_keyboard_walkthrough_visits_every_tab_stop_exactly_once",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/crate_line_numbers.rs",
                },
            ],
        },
    },
    Scene {
        number: 2,
        name: "a 1M-row virtualised list",
        decided: "the data-volume invariant: 15 876x, flat 0.987x from 1k to 1M",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "visible_rows_bounds_a_million_rows_to_a_screenful",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_entry_count_is_identical_over_two_thousand_and_a_million_rows",
                },
            ],
        },
    },
    Scene {
        number: 3,
        name: "200 -> 800 keyed widgets",
        decided: "duplicate detection's slope: 3.95x against 9.26x. **The detector the budget walks \
                  past**: the same defect costs the dense screen 1.19x and clears a 100 us gate",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/id.rs",
                    name: "growth_is_sub_quadratic",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/id_numbers.rs",
                },
            ],
        },
    },
    Scene {
        number: 4,
        name: "a modal over the dense screen",
        decided: "the scrim: 116.62 -> 1.19 us, with the base layer and draw-before-pad both in place",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "padding_before_text_re_damages_cells_and_text_before_padding_re_damages_none",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/overlay_numbers.rs",
                },
            ],
        },
    },
    Scene {
        number: 5,
        name: "a chart folding 2 000 rows a frame",
        decided: "the memo: 84.91 us against 1.28 ns, which is 66 301x",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/data.rs",
                    name: "an_unknown_revision_means_memoise_nothing_and_the_counter_says_so",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/data.rs",
                    name: "an_id_keyed_memo_swept_like_a_grab_pays_a_fold_every_time_its_widget_returns",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/data_numbers.rs",
                },
            ],
        },
    },
    Scene {
        number: 6,
        name: "a search box filtering 600 keyed rows",
        decided: "the vanish rule: 90 002 probes against 601, and a quiet frame pays 0",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_vanish_rule_is_bounded_by_the_ring_and_not_by_the_ring_squared",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/focus.rs",
                    name: "a_quiet_frame_pays_no_probes",
                },
            ],
        },
    },
    Scene {
        number: 7,
        name: "an 8 000-key paste",
        decided: "`next_key`'s O(n^2): 11.75 ms against 53.79 us, and the paste is one frame",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/route.rs",
                    name: "draining_an_eight_thousand_key_paste_is_linear",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "an_eight_thousand_key_paste_is_one_frame",
                },
            ],
        },
    },
    Scene {
        number: 8,
        name: "a 300 ms fade at each colour tier",
        decided: "19 / 2 / 2 wakeups gated, 19 at every tier ungated",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "the_same_fade_is_nineteen_ungated_and_nineteen_two_two_gated",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "a_three_hundred_millisecond_fade_runs_nineteen_frames",
                },
            ],
        },
    },
    Scene {
        number: 9,
        name: "a list at its end inside a scroll area",
        decided: "wheel chaining: 0 clicks reach the area under an axis matcher, against 3 under \
                  the direction matcher",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "an_axis_matcher_moves_the_enclosing_area_by_zero",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "four_directions_differ_from_two_axes_on_sixteen_of_sixty_four",
                },
            ],
        },
    },
    Scene {
        number: 10,
        name: "three nested scroll areas",
        decided: "the chain stops at the innermost that can move: 20 = 1 + 15 + 1 + 3",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "twenty_clicks_into_a_five_row_list_move_the_area_by_three",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "a_warm_frame_loses_only_the_end_stop_residue",
                },
            ],
        },
    },
    Scene {
        number: 11,
        name: "a 40-field form in a 6-row viewport",
        decided: "scroll-into-view: 1 frame resolved in `end`, 2 in the draw",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "into_view_fires_for_a_keyboard_move_and_a_press_does_not_pull",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "an_entry_below_the_fold_asks_for_the_difference_and_one_above_it_asks_for_its_own_row",
                },
            ],
        },
    },
    Scene {
        number: 12,
        name: "1M rows through a `scroll_area` and through a `list`",
        decided: "the two mechanisms as a ratio: 887-889x against 0.997-1.003x",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "a_scroll_area_costs_the_content_and_a_list_costs_the_window",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/scroll_numbers.rs",
                },
            ],
        },
    },
    Scene {
        number: 13,
        name: "a file browser with a preview pane, 20 selections",
        decided: "staleness: 19 of 20 frames wrong without the generation, 20 of 20 with it",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "the_bare_slot_shows_the_wrong_file_on_nineteen_of_twenty_frames",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "the_generation_shows_the_selected_file_on_twenty_of_twenty_frames",
                },
            ],
        },
    },
    Scene {
        number: 14,
        name: "a decode superseded before it finishes",
        decided: "cancel is a bracket: 1 of 210 against 210 of 210",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "cancellation_is_one_unit_of_two_hundred_and_ten_when_it_is_polled_and_all_of_it_when_it_is_not",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "a_superseded_job_that_never_polls_lands_nothing_and_wakes_nobody",
                },
            ],
        },
    },
    Scene {
        number: 15,
        name: "a directory read streamed in 100 batches",
        decided: "a `Slot` of deltas loses 9 000 of 10 000 rows; a `Drain` loses none",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "a_slot_of_deltas_loses_nine_thousand_of_ten_thousand_rows",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "a_drain_loses_none_of_ten_thousand_rows_on_the_same_schedule",
                },
            ],
        },
    },
    Scene {
        number: 16,
        name: "sixty frames with a job in flight",
        decided: "0 wakeups against 60 polled. **Re-gated over the wake ledger by ticket 06**, \
                  which is the pairing ticket 16's answer says ticket 19 should carry: it was \
                  counted at the two call sites first, because the ledger did not exist yet",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "sixty_frames_with_a_job_in_flight_ask_for_no_wakeups",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "sixty_frames_with_a_job_in_flight_ask_for_no_wakeups",
                },
            ],
        },
    },
    Scene {
        number: 17,
        name: "an imported theme flat at the tier under test",
        decided: "`hover_interest` is `NONE`, the level is `Drag`. **Not an absolute**: the same \
                  screen is at `Motion` on a non-flat truecolor theme and drops only after \
                  `resolve(tier)`",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "one_hovering_region_raises_the_frame_and_a_flat_theme_does_not",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "hover_interest_follows_what_the_terminal_can_show",
                },
            ],
        },
    },
    Scene {
        number: 18,
        name: "a Cyrillic layout at three key tiers",
        decided: "33 / 28 / 14 of 33 reachable, 0 wrong at every tier. **A scene pinned by the test \
                  rig**, because a terminal cannot report which of the two legacy cases it is",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "the_loss_is_one_family_and_the_survivor_is_ascii",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "a_us_layout_reaches_everything_at_every_tier",
                },
            ],
        },
    },
    Scene {
        number: 19,
        name: "a chunked data source",
        decided: "the trap through the storage, and the scene that disappears into any number \
                  averaged over the other nineteen. **The figure moved and the finding did not**: \
                  §14 measured 320.85 us, 321% of the budget, and the shipped module reads \
                  226.92-237.21 us over six runs — **roughly 230%**, a third smaller, 4% of spread \
                  between two runs of one binary, and still multiples of a whole frame. So what \
                  R 20 gated is the relation the timing is evidence for, which is the same number \
                  on every machine: **one screenful is 78 index reads against 609 460 chunk hops \
                  and 39 003 081 node hops**, and a walked source costs twice as much twice as far \
                  down the list where an indexed one costs the same",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/data.rs",
                    name: "a_chunked_source_is_three_orders_off_a_slice_and_doubles_with_the_offset",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/data_numbers.rs",
                },
            ],
        },
    },
    Scene {
        number: 20,
        name: "a theme swap over the dense screen",
        decided: "41.92 us, 20 060 damaged cells against 0 for a steady frame",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/swap.rs",
                    name: "a_steady_frame_is_zero_bytes_on_the_wire_and_a_swap_frame_is_a_screen",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/theme_set_numbers.rs",
                },
            ],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// **Twenty scenes, numbered 1..=20, once each.**
    ///
    /// The count is the gate: the list is normative and a scene is removed only by a ticket naming
    /// the property it can no longer distinguish, so a nineteenth-and-a-half is a diff that has to
    /// argue for itself.
    #[test]
    fn the_scene_list_is_twenty_scenes() {
        assert_eq!(SCENES.len(), 20);
        let numbers: BTreeSet<u8> = SCENES.iter().map(|s| s.number).collect();
        assert_eq!(numbers, (1..=20).collect::<BTreeSet<u8>>());
        let names: BTreeSet<&str> = SCENES.iter().map(|s| s.name).collect();
        assert_eq!(names.len(), SCENES.len(), "two scenes share a name");
    }

    /// **Twenty wired, none red**, and that is R 20 closing the last one.
    ///
    /// This list was nineteen and one until the chunked scene got a test — the chunked
    /// data source, whose only instrument was a report no CI job runs — a `#[test]` over the
    /// relation its microsecond figure is evidence for. Saying *how many* is what stops a red scene
    /// arriving unremarked, and at all-green it has the second job [`crate::register`]'s count has:
    /// **a list that says it is all-green** makes the next red row a deliberate edit to this number
    /// rather than a quiet one.
    #[test]
    fn twenty_are_wired_and_none_are_red() {
        let red: Vec<u8> = SCENES
            .iter()
            .filter(|s| matches!(s.state, State::Red { .. }))
            .map(|s| s.number)
            .collect();
        assert_eq!(
            red,
            Vec::<u8>::new(),
            "a red scene is back. A new one is fine and has to be argued for here, in this test's \
             documentation, and in the module comment above — the count is the thing that stops it \
             arriving unremarked"
        );
        assert_eq!(SCENES.len() - red.len(), 20);
    }

    /// **A wired scene runs something `cargo test` runs.**
    ///
    /// [`State::Wired`]'s documentation says *it runs*, and a scene whose instruments are all
    /// [`Instrument::Report`] does not: this workspace runs `cargo test` and two named examples, so
    /// every other `examples/*.rs` is compiled and never evaluated. A report **beside** a test is
    /// the number a human reads and is welcome; a report **instead of** one is the register's
    /// refinement 2 arriving on the scene list.
    #[test]
    fn a_wired_scene_is_not_only_a_report() {
        for scene in SCENES {
            let State::Wired { by } = scene.state else {
                continue;
            };
            assert!(
                by.iter().any(|i| !matches!(i, Instrument::Report { .. })),
                "scene #{} is wired to reports alone, which nothing runs. Either it names a test \
                 as well, or it is red against the ticket that will give it one",
                scene.number
            );
        }
    }

    /// **Every scene says what it decided.**
    ///
    /// The rule at the top of this module is that a scene is removed only by a ticket naming the
    /// property it can no longer distinguish — and a row with an empty `decided` is a row nobody can
    /// argue with, which makes it the first one deleted.
    #[test]
    fn every_scene_says_what_it_decided() {
        for scene in SCENES {
            assert!(
                scene.decided.len() > 20,
                "scene #{} does not say what it decided",
                scene.number
            );
        }
    }

    /// **Every scene names something that exists.**
    ///
    /// The same non-vacuity rule [`crate::register`] is built around: a list of prose lets a scene
    /// stop being run with nothing saying so.
    #[test]
    fn every_scene_names_something_that_exists() {
        for scene in SCENES {
            let State::Wired { by } = scene.state else {
                continue;
            };
            assert!(
                !by.is_empty(),
                "scene #{} is wired without saying where",
                scene.number
            );
            for instrument in by {
                let source = read(instrument.file());
                match *instrument {
                    Instrument::Unit { file, name } => assert!(
                        source.contains(&format!("fn {name}(")),
                        "scene #{}: `{file}` has no `fn {name}`",
                        scene.number
                    ),
                    Instrument::Report { file } => assert!(
                        !source.is_empty(),
                        "scene #{}: the report `{file}` is empty",
                        scene.number
                    ),
                    Instrument::Pair { .. } | Instrument::Attribute { .. } => panic!(
                        "scene #{} names a compile outcome. A scene is a shape gates are driven \
                         over, not a gate",
                        scene.number
                    ),
                }
            }
        }
    }

    /// **There is no total, and this test is where that is written down.**
    ///
    /// Numbers are kept per scene and never summed. `Scene` has no numeric field to sum, which is
    /// the structural half; this is the half that says so out loud, so that adding one is a diff
    /// that has to delete an assertion with a reason attached.
    #[test]
    fn the_list_carries_no_number_that_could_be_summed() {
        let source = read("crates/vitui-runtime/src/scenes.rs");
        let after = source
            .split_once("pub struct Scene {")
            .expect("the type is in this file")
            .1;
        // Field lines only, to the closing brace at column zero. `split_once('}')` was the first
        // version and it stopped at the first brace *anywhere* after the header — including one
        // inside a field's own doc comment, which would have silently truncated the scanned body
        // and let everything below it through.
        let fields: Vec<&str> = after
            .lines()
            .take_while(|line| *line != "}")
            .map(str::trim)
            .filter(|line| !line.starts_with("///") && !line.starts_with("//"))
            .collect();
        assert!(
            fields.iter().any(|f| f.starts_with("pub number:")),
            "the field scan found no fields, so it has stopped looking at `Scene`"
        );
        for field in &fields {
            let Some((_, ty)) = field.split_once(':') else {
                continue;
            };
            let ty = ty.trim().trim_end_matches(',');
            // Everything a number could arrive as. `number: u8` is the one exception and it is an
            // **identifier**, not a measurement — summing scene numbers is meaningless rather than
            // merely wrong, which is exactly the property that keeps it safe.
            let numeric = ty.starts_with('u')
                || ty.starts_with('i')
                || ty.starts_with('f')
                || ty.starts_with("NonZero")
                || ty == "Duration"
                || ty == "Instant"
                || ty == "usize"
                || ty == "isize";
            assert!(
                !numeric || field.starts_with("pub number:"),
                "`Scene` gained the numeric field `{field}`. **Numbers are kept per scene and \
                 never summed**, and a field here is the first step to a total. `number` is the \
                 one exception because it identifies rather than measures"
            );
        }
    }
}
