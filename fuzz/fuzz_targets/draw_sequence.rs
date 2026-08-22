//! §14's first target: draw sequences against the naive reference compositor.
//!
//! Four lines of body, and that is the design. The decoding and the oracle are
//! `vitui_engine::fuzz::draw_sequence`, inside the engine, because the committed corpus replayed as
//! an ordinary `cargo test` is the gate and the soak has to be the same code — see
//! `crates/vitui-engine/src/fuzz.rs` and `../README.md`.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    vitui_engine::fuzz::draw_sequence(data);
});
