# Session Overview

Goal: continue the Windows sandbox hardening lane in `codex-rs/windows-sandbox-rs` and
close the remaining elevated launch env-inheritance gap.

Current state:
- Non-elevated helper launch paths already scrub unsafe injection env vars.
- Elevated `ShellExecuteExW` launch now scrubs the same env vars temporarily and restores
  them immediately after launch.
- The sandbox crate now passes after resolving protocol merge markers and a `ToolSearchOutput`
  constructor mismatch in `codex-protocol`.
