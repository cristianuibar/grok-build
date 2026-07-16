# Coding Conventions

**Analysis Date:** 2026-07-16

## Naming Patterns

**Files:**
- Rust modules: `snake_case.rs` (e.g. `workspace_classifier.rs`, `signed_policy.rs`)
- Module as directory when multi-file: `signed_policy/mod.rs` + sibling files (`tests.rs`, helpers)
- Integration tests under `tests/`: prefer descriptive names
  - `test_<feature>.rs` or `test_<feature>_e2e.rs` in shell (`crates/codegen/xai-grok-shell/tests/`)
  - `integration.rs` for single-suite crates (`crates/codegen/xai-grok-hooks/tests/integration.rs`)
  - Feature-named files for multi-suite crates (`pty_e2e.rs`, `settings_e2e.rs`)
- Snapshot files: auto-named under `snapshots/` next to the module (insta layout)
- Crate package names: `xai-` prefix, kebab-case (`xai-grok-hooks`, `xai-file-utils`)
- Binary artifact: `xai-grok-pager` (shipped as `grok`)

**Functions:**
- `snake_case` for all functions and methods
- Test functions: descriptive behavior phrases, no `test_` prefix required when the module is already a test module — e.g. `actor_spawns_and_shuts_down_via_cancellation`, `static_api_key_is_fallback_when_provider_returns_none`
- Getters without `get_` prefix: `endpoints()`, `cli_chat_proxy_base_url()`, `as_str()`
- Constructors: `new`, `with_*` builders, domain verbs (`spawn`, `load_*`, `dispatch_*`)
- Conversion helpers: `from_*`, `into_*`, `*_str`

**Variables:**
- `snake_case` locals and parameters
- Prefer short, domain-clear names (`dir`, `ctx`, `handle`, `token`)
- Env var names: `SCREAMING_SNAKE` with `GROK_` prefix for product knobs (`GROK_BINARY`, `GROK_HOME`, `GROK_LEADER_SOCKET`)

**Types:**
- `PascalCase` for structs, enums, traits (`WebSearchClient`, `SamplingError`, `HookDecision`)
- Error enums: `*Error` suffix (`SamplingError`, `ToolErrorWire`, `RequirementsError`)
- Type aliases: `Result<T>` scoped to the module’s error type when a crate has a primary error
- Constants: `SCREAMING_SNAKE` (`DEFAULT_TOOL_OUTPUT_BYTES`, `PRODUCTION_ENDPOINTS` for static data may be `PascalCase` when typed structs)
- Trait object / shared handles: `Shared*` prefix (`SharedApiKeyProvider`, `SharedAttributionCallback`)

**Modules:**
- Prefer focused modules over mega-files; split large domains (`agent/`, `session/`, `implementations/`)
- Private modules by default (`mod loader`); only `pub mod` the intentional public surface
- Re-export curated API at crate root via `pub use` (see `crates/codegen/xai-grok-config/src/lib.rs`)

## Code Style

**Formatting:**
- Tool: `rustfmt` (component pinned in `rust-toolchain.toml`)
- Config: `rustfmt.toml` at repo root — `use_field_init_shorthand = true`
- Run: `cargo fmt --all`
- Edition: workspace `edition = "2024"` (`Cargo.toml` `[workspace.package]`)

**Linting:**
- Tool: `clippy` (component pinned in `rust-toolchain.toml`)
- Root config: `clippy.toml` — applies to codegen crates; **nearest** `clippy.toml` wins (no merge)
- Workspace clippy lints: `[workspace.lints.clippy]` in root `Cargo.toml`; crates opt in with:

```toml
[lints]
workspace = true
```

- Notable allowed lints (workspace): `doc_lazy_continuation`, `doc_overindented_list_items`, `needless_lifetimes`, `single_range_in_vec_init`, `too_many_arguments`, `uninlined_format_args`, `useless_format`
- **Disallowed methods** (in `clippy.toml`): `std::fs::canonicalize`, `std::path::Path::canonicalize`, `tokio::fs::canonicalize` — use `dunce::canonicalize` (or tools helpers in `xai_grok_tools::util::fs` for async)
- Per-crate `#![allow(...)]` is common on large crates for dead-code/unused during rapid development (`xai-grok-shell`, `xai-grok-env`, etc.)
- Prefer crate-level allow lists over scattering many local allows when the crate is transitional
- Run: `cargo clippy -p <crate>` (prefer per-crate; full workspace is slow)

**Toolchain:**
- Pinned stable in `rust-toolchain.toml` (currently `1.92.0`)
- Bump policy (from file comments): one point version at a time; wait weeks after release; re-run `cargo check/clippy --all-targets --workspace`

## Import Organization

**Order (typical observed pattern):**
1. `std::...` and `core` items
2. External crates (`tokio`, `serde`, `anyhow`, `reqwest`, …)
3. Workspace / internal crates (`xai_grok_*`, `xai_tool_*`)
4. `crate::...` / `super::...`

**Style:**
- Group with blank lines between std / external / crate when the file is large
- Prefer `use crate::module::Item` over deep relative paths for cross-module refs
- Import macros/types used in tests inside the `#[cfg(test)]` module, not at crate top, when only tests need them
- Wiremock and heavy test deps: often imported inside the individual test function (see `crates/codegen/xai-grok-tools/src/implementations/web_search/client.rs`)

**Path Aliases:**
- No `paths` aliases in cargo sense; use package names as crates (`xai_grok_hooks`, `xai_file_utils`)
- Common re-export aliases: `use xai_grok_http as http` (shell)

**Dependencies:**
- Prefer `{ workspace = true }` in per-crate `Cargo.toml`
- Root `Cargo.toml` is **generated** — treat as read-only; edit per-crate manifests and workspace dep versions only when regenerating workflow allows
- Add new shared deps under `[workspace.dependencies]` when used by multiple crates

## Error Handling

**Patterns:**
- **Library / domain errors:** `thiserror` enums with `#[error("...")]` display strings
  - Example: `SamplingError` in `crates/codegen/xai-grok-sampling-types/src/error.rs`
  - Example: wire errors `ToolErrorWire` in `crates/common/xai-tool-protocol/src/error_wire.rs`
- **Application / orchestration:** `anyhow::Result` + `.context()` / `.with_context()` at boundaries (shell, harness, CLI paths)
- Prefer structured variants with fields over stringly-typed errors when callers branch on kind
- Wire-facing errors: `Serialize`/`Deserialize` with stable `code` discriminators (`#[serde(tag = "code", rename_all = "snake_case")]`)
- Map foreign errors with `map_err` into domain types; keep messages human-readable for logs and UI
- Local `pub type Result<T> = std::result::Result<T, DomainError>` next to the primary error enum when the crate is error-centric

**Do this:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum SamplingError {
    #[error("API error (status {status}): {message}")]
    Api {
        status: StatusCode,
        message: String,
        // ... structured context fields
    },
}
```

**Avoid:**
- Silent `unwrap()` in production paths — reserve for invariants with `.expect("why this cannot fail")`
- Returning bare `String` as the only error type for public APIs when variants matter
- Using `std::fs::canonicalize` (Windows verbatim path hazard) — use `dunce::canonicalize`

**Assertions in fallible constructors:**
- Return `Result` from `new` when config can be invalid (e.g. `WebSearchClient::new`)
- Document when `Err` is returned in the doc comment

## Logging

**Framework:** `tracing` (workspace dep)

**Patterns:**
- Use structured fields: `tracing::warn!(error = %e, path = %path.display(), "failed to read file")`
- Prefer `%` / `?` format in field values for `Display` / `Debug`
- Levels: `error!` for failures that need attention, `warn!` for degraded modes, `info!` for lifecycle milestones, `debug!` for detailed state changes
- Shell also routes through `xai_grok_telemetry` / `xai_tracing_macros` (`teprintln!`, `timed!`, `tprintln!`) for specialized instrumentation
- Avoid logging secrets (API keys, tokens); redaction lives in `xai-grok-secrets` / sanitizer paths

**When to log:**
- External I/O failures that are handled but should be visible
- Feature fallbacks (e.g. sqlite-vec unavailable → FTS-only)
- Actor/state mutations that are hard to reconstruct from return values alone

## Comments

**When to Comment:**
- Crate- and module-level `//!` docs explaining purpose, merge order, contracts (see `crates/codegen/xai-grok-config/src/lib.rs`)
- Non-obvious invariants, Windows/Unix differences, wire-format compatibility
- Regression rationale on tests (“API-key users past the 30-day client TTL saw 401…”)
- `// SAFETY:` immediately above every `unsafe` block explaining soundness (required style in this tree)

**JSDoc/TSDoc:**
- Not applicable (Rust) — use rustdoc `///` on public items
- Document error conditions, field meanings for wire types, and platform `cfg` behavior
- Prefer linking with `` [`Type`] `` / `` [`fn`] `` rustdoc references

**Section banners:**
- Heavy test files use comment banners: `// ====` or `// ── Section ──`

## Function Design

**Size:**
- Prefer single-purpose functions; large match-driven handlers are OK when domain-driven
- Extract helpers for repeated setup (especially in tests: `test_config()`, `write_hook()`, harness structs)

**Parameters:**
- Prefer owned types at API boundaries when stored; borrow (`&str`, `&Path`) for read-only
- Pass config by reference when not consumed: `new(config: &WebSearchConfig, ...)`
- Use `Option` for optional hooks/providers rather than multiple constructors when modes are few
- Builder / `with_*` chains for optional fields on types that grow many knobs

**Return Values:**
- `Result` for fallible work; domain error type in libraries, `anyhow` at binary/orchestration edges
- Avoid returning `(bool, String)` when an enum communicates outcomes better (`HookDecision`, `EmptyReason`)
- Async: `async fn` with Tokio runtime; trait methods may use `async_trait` where needed

## Module Design

**Exports:**
- Public API is deliberate: keep internals `pub(crate)` or private
- Re-export the stable surface at `lib.rs` with a short comment when selective (config campaigns note)
- Shared test-only modules: `#[cfg(test)] pub(crate) mod test_support` (shell)

**Barrel Files:**
- `mod.rs` for multi-file modules; `lib.rs` is the crate barrel
- Avoid deep re-export cascades that hide ownership — prefer one clear owning module

**Crate layout conventions:**
- `crates/codegen/` — CLI / TUI / agent product crates
- `crates/common/` — shared leaf libraries
- `crates/build/` — build-time helpers (`xai-proto-build`)
- `third_party/` — vendored upstream; do not apply first-party style “cleanups” casually
- `prod/` — production-adjacent types (e.g. proxy types)

## Serde & Wire Formats

**Patterns:**
- Derive `Serialize`/`Deserialize` with explicit `rename_all` when wire format is fixed:
  - API JSON often `camelCase` or `snake_case` per endpoint
  - Enums: `#[serde(rename_all = "snake_case")]` or `lowercase` for simple enums
- Tagged errors: `#[serde(tag = "code", rename_all = "snake_case")]`
- Optional fields: `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Keep display strings and serde renames aligned when both exist (pin shared constants if needed)

## Platform & Safety

**Platform `cfg`:**
- Gate OS-specific code with `#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "linux")]`
- Mirror production `cfg` in tests (platform modules under `#[cfg(test)]`)
- Unix-only features (leader IPC, crash-handler signal tests) document the gate at the top of the test file

**Unsafe:**
- Minimize; always pair with `// SAFETY: ...`
- Env mutation in tests: only under `#[serial_test::serial]` with RAII guards (`EnvGuard` in `xai-grok-test-support`)

**Paths:**
- Use `dunce::canonicalize` for path equality / containment
- Prefer `Path`/`PathBuf`; display with `%path.display()` in logs

## Async & Concurrency

**Runtime:** Tokio (`features = ["full"]` or targeted features per crate)

**Patterns:**
- Actors with channels (`mpsc`) and `CancellationToken` (`xai-chat-state`)
- Prefer structured shutdown (cancel token / drop handles) over leaking tasks
- Timeouts in tests with `tokio::time::timeout` and clear `.expect("timed out waiting for …")`
- Virtual time: `#[tokio::test(start_paused = true)]` when testing timers (requires `tokio` `test-util` in dev-deps)

## Project-Specific Conventions

**Target specific crates in commands:**
```sh
cargo check -p <crate>
cargo test -p <crate>
cargo clippy -p <crate>
```
Full-workspace builds are slow (`README.md` Development section).

**Presubmit quality bar:**
- `cargo fmt --all`
- `cargo clippy -p <crate>` against root `clippy.toml`
- Prefer green unit/integration tests for the crate you touched

**Documentation freshness:**
- Shared test infrastructure README must update in the same PR as `src/` changes (`crates/codegen/xai-grok-test-support/README.md`)

**Third-party / ports:**
- Tool implementations may originate from codex/opencode ports — honor `THIRD_PARTY_NOTICES` / crate-local notices when editing

---

*Convention analysis: 2026-07-16*
