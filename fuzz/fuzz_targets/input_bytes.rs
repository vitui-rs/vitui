//! the engine's design's second target: byte streams into the input parser.
//!
//! The one surface in the engine that parses input the engine did not produce, and the one whose
//! oracle the engine's design concedes is weaker: no panic, every byte consumed, no unbounded growth. What each of
//! those three means as an assertion is in `vitui_engine::fuzz`, with the middle one spelled out at
//! length — the literal reading of it is a statement about a `for` loop.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    vitui_engine::fuzz::input_bytes(data);
});
