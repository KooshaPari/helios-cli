//! Fuzz target for `harness_normalizer::Normalizer::normalize_json`.
//!
//! Exercises the JSON normalizer which strips whitespace and validates
//! brace balance. The normalizer must never panic; errors are expected
//! for empty or unbalanced input.
//!
//! Run with cargo-fuzz:
//!   cd helios-cli
//!   cargo fuzz run fuzz_normalizer_json

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    let normalizer = harness_normalizer::Normalizer::new();

    // normalize_json must never panic. It may return Err for empty input
    // or unbalanced braces, but it must not crash on arbitrary input.
    let _ = normalizer.normalize_json(input);

    // Also exercise the general normalize path with various flag combos.
    let _ = normalizer.normalize(input);
    let _ = harness_normalizer::Normalizer::new()
        .with_lowercase(true)
        .with_remove_special(true)
        .normalize(input);
});
