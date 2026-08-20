//! Fuzz target for helios-cli input parsing.
//!
//! Exercises:
//!   - `harness_spec::parser::parse_auto`  — auto-detecting YAML/JSON spec parser
//!   - `harness_utils::parse_kv`           — key-value pair parser
//!   - `harness_utils::parse_tags`         — comma-separated tag parser
//!
//! Run with cargo-fuzz:
//!   cd helios-cli
//!   cargo fuzz run fuzz_parse_input
//!
//! Or run a single iteration:
//!   cargo fuzz run fuzz_parse_input -- -max_total_time=10

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to UTF-8; skip invalid UTF-8 inputs (the parsers only
    // accept &str, not &[u8]).
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // ---------------------------------------------------------------
    // 1. fuzz_parse_auto — auto-detecting YAML/JSON spec parser
    // ---------------------------------------------------------------
    // parse_auto must never panic. It may return Err for invalid input,
    // but it must not crash.
    let _ = harness_spec::parser::parse_auto(input);

    // ---------------------------------------------------------------
    // 2. fuzz_parse_kv — key-value parser with various delimiters
    // ---------------------------------------------------------------
    // Try multiple delimiter / pair-separator combos that callers actually
    // use in the codebase.
    let _ = harness_utils::parse_kv(input, ',', '=');
    let _ = harness_utils::parse_kv(input, ';', ':');
    let _ = harness_utils::parse_kv(input, '\n', '=');
    let _ = harness_utils::parse_kv(input, '|', ' ');

    // ---------------------------------------------------------------
    // 3. fuzz_parse_tags — comma-separated tag parser
    // ---------------------------------------------------------------
    let _ = harness_utils::parse_tags(input);

    // ---------------------------------------------------------------
    // 4. fuzz_parse_auto with JSON-priming
    // ---------------------------------------------------------------
    // If input starts with '{', it will be treated as JSON by parse_auto.
    // Verify that parse_json (which is called internally) never panics.
    if input.trim_start().starts_with('{') {
        let _ = harness_spec::parser::parse_json(input);
    }

    // ---------------------------------------------------------------
    // 5. fuzz_parse_auto with YAML-priming
    // ---------------------------------------------------------------
    // If input does NOT start with '{', parse_auto falls through to YAML.
    // Verify that parse_yaml never panics.
    if !input.trim_start().starts_with('{') {
        let _ = harness_spec::parser::parse_yaml(input);
    }
});
