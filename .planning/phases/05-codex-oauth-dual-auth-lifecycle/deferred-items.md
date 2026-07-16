# Phase 5 deferred items

## Pre-existing (out of scope for 05-01)

### `xai-grok-pager` lib-test target does not compile

- **Symptom:** `cargo test -p xai-grok-pager --lib` (and unscoped `cargo test -p xai-grok-pager <filter>`) fails with ~169 errors: pager modules import `#[cfg(test)]` helpers from `xai-grok-pager-render` (`set_protocol_for_test`, `pin_theme`, clipboard probe hooks, etc.) that are not visible because dependency crates are not built with `cfg(test)`.
- **Impact on 05-01:** Clap parse unit tests live in `cli.rs` (per plan) but are **not runnable** via the plan’s bare `cargo test -p xai-grok-pager <name>` form. Runnable twin: `cargo test -p xai-grok-pager --test auth_cli_parse <name>`.
- **Not fixed in 05-01:** Pre-existing; outside dual-auth scope. Track for a tooling/hygiene plan (feature-gate test helpers on pager-render or stop re-exporting them into non-test consumer test modules).
