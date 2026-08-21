//! Fuzz target for `harness_spec::parser::parse_yaml`.
//!
//! Exercises the YAML specification parser with arbitrary byte sequences
//! converted to UTF-8. The parser must never panic; errors are expected
//! for malformed input.
//!
//! Run with cargo-fuzz:
//!   cd helios-cli
//!   cargo fuzz run fuzz_spec_yaml

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // parse_yaml must never panic. It may return Err for invalid YAML,
    // but it must not crash or hang on arbitrary input.
    let _ = harness_spec::parser::parse_yaml(input);
});
