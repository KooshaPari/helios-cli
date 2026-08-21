//! Fuzz target for `helios_config::HeliosConfig::from_file`.
//!
//! Writes the fuzzed bytes to a temporary TOML file and calls
//! `HeliosConfig::from_file` to exercise the config deserialization
//! path. The function must never panic; errors are expected for
//! malformed TOML.
//!
//! Run with cargo-fuzz:
//!   cd helios-cli
//!   cargo fuzz run fuzz_config_load

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    // Write fuzzed content to a temporary TOML file.
    let dir = std::env::temp_dir();
    let path = dir.join(format!("helios_fuzz_config_{}.toml", std::process::id()));

    let mut f = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(_) => return,
    };
    if f.write_all(data).is_err() {
        let _ = std::fs::remove_file(&path);
        return;
    }
    drop(f);

    // from_file must never panic. It may return Err for invalid TOML,
    // but it must not crash or hang on arbitrary input.
    let _ = helios_config::HeliosConfig::from_file(&path);

    // Also exercise from_file with a .yaml extension (triggers YAML path).
    let yaml_path = dir.join(format!("helios_fuzz_config_{}.yaml", std::process::id()));
    let _ = std::fs::copy(&path, &yaml_path);
    let _ = helios_config::HeliosConfig::from_file(&yaml_path);

    // Clean up temp files (best-effort).
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&yaml_path);
});
