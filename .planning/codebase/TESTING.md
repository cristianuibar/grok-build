# Testing Patterns

**Analysis Date:** 2026-07-16

## Test Framework

**Runner:**
- Built-in Rust test harness via Cargo (`cargo test`)
- Async: `#[tokio::test]` (and `#[tokio::test(start_paused = true)]` for virtual time)
- Config: per-crate `Cargo.toml` `[dev-dependencies]`; workspace pins shared test crates in root `Cargo.toml`

**Assertion Library:**
- Standard `assert!` / `assert_eq!` / `assert_ne!`
- `assert_matches` (`assert_matches!`) for enum/pattern assertions — workspace dep `assert_matches`
- `pretty_assertions` for high-diff equality in UI/markdown/render crates (`xai-grok-pager`, `xai-grok-markdown`, `xai-grok-pager-render`, …)
- `insta` snapshots for multi-line visual/text output (pager tool diff rendering)

**Run Commands:**
```bash
cargo test -p xai-grok-config              # unit + integration for one crate
cargo test -p xai-grok-hooks --test integration
cargo test -p xai-chat-state actor_spawns  # filter by test name
cargo test -p xai-grok-shell -- --ignored  # run #[ignore] binary e2e tests
cargo test -p xai-grok-pager -- --ignored  # PTY / binary-backed scenarios
cargo bench -p xai-fsnotify --bench startup
cargo fmt --all
cargo clippy -p <crate>
```

Prefer **per-crate** targets; full-workspace test runs are heavy.

## Test File Organization

**Location:**
1. **Inline unit tests** — `#[cfg(test)] mod tests { ... }` at bottom of the same source file (most common; ~1100 files use `cfg(test)`)
2. **Sibling test module** — `tests.rs` next to production code (e.g. `crates/codegen/xai-chat-state/src/actor/tests.rs`)
3. **Crate integration tests** — `crates/<area>/<crate>/tests/*.rs` (~320 files)
4. **Co-located domain test dirs** — e.g. `src/agent/mvp_agent/tests/`, `src/app/dispatch/tests/`

**Naming:**
| Kind | Pattern | Example |
|------|---------|---------|
| Unit test fn | `snake_case` behavior phrase | `long_cwd_uses_hash_fallback_within_name_max` |
| Integration file | `test_<area>.rs` or `<area>_e2e.rs` | `test_mcp_integration.rs`, `settings_e2e.rs` |
| Single-suite integration | `integration.rs` | `xai-grok-hooks/tests/integration.rs` |
| Perf / soak | descriptive + optional `perf`/`soak` | `session_load_perf.rs`, `test_leader_soak.rs` |
| Snapshot | insta auto path under `snapshots/` | `...__diff_basic.snap` |

**Structure:**
```
crates/codegen/xai-grok-shell/
├── src/
│   ├── lib.rs                 # #[cfg(test)] pub(crate) mod test_support
│   ├── agent/...              # unit tests inline or submodules
│   └── test_support/          # crate-private helpers
└── tests/
    ├── common/                # shared integration helpers (when needed)
    ├── fixtures/
    ├── test_*.rs              # process-level / e2e
    └── signed_managed_config/ # multi-file suite

crates/codegen/xai-grok-pager/
└── tests/
    ├── scenarios/*.yaml       # declarative PTY scripts
    ├── scripted_scenarios.rs  # runner for YAML scenarios
    ├── pty_e2e/
    └── settings_e2e.rs
```

## Test Structure

**Suite Organization:**

Inline unit tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn long_cwd_uses_hash_fallback_within_name_max() {
        let long_cwd = format!("/Users/test/{}", "中".repeat(30));
        let encoded = encode_cwd_dirname(&long_cwd);
        assert!(encoded.len() <= MAX_DIRNAME_BYTES);
    }
}
```

Platform-gated submodules (preferred when behavior differs by OS):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "windows"))]
    mod posix {
        use super::*;

        #[test]
        fn root_is_unsafe() {
            assert!(!is_project_dir(Path::new("/")));
        }
    }
}
```
Reference: `crates/codegen/xai-file-utils/src/workspace_classifier.rs`

Async actor tests with harness:
```rust
struct TestHarness {
    handle: ChatStateHandle,
    event_rx: mpsc::UnboundedReceiver<ChatStateEvent>,
    // ...
}

#[tokio::test]
async fn actor_spawns_and_shuts_down_via_cancellation() {
    // arrange → act → assert with timeouts
}
```
Reference: `crates/codegen/xai-chat-state/src/actor/tests.rs`

Integration (hooks pipeline):
```rust
#[tokio::test]
async fn hook_allows_via_json() {
    let dir = tempfile::tempdir().unwrap();
    write_hook(dir.path(), "safety.json", r#"..."#);
    let (registry, errors) = load_hooks(Some(dir.path()), None);
    assert!(errors.is_empty(), "errors: {errors:?}");
    // dispatch + assert decision
}
```
Reference: `crates/codegen/xai-grok-hooks/tests/integration.rs`

**Patterns:**
- **Arrange / Act / Assert** with descriptive test names
- **Helpers** at top of test module (`test_config()`, `write_hook()`, envelope builders)
- **Temp dirs** via `tempfile::TempDir` / `tempfile::tempdir()` for filesystem isolation
- **Assert messages** include context: `assert!(errors.is_empty(), "errors: {errors:?}")`
- **Section comments** in large suites (`// Lifecycle tests`)

## Mocking

**Framework:**
- **No mockall-heavy style** as the primary pattern
- **`wiremock`** for HTTP servers in unit/integration tests (tools web search, update client, tracing HTTP)
- **`xai-grok-test-support`** mock inference server for agent/shell/pager end-to-end flows
- Hand-written mocks implementing traits (`MockChatPersistence`, `NoneProvider` / `FreshProvider` for API keys)

**Patterns:**

Wiremock (inline in test):
```rust
#[tokio::test]
async fn static_api_key_is_fallback_when_provider_returns_none() {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(header("Authorization", "Bearer static-key-from-config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ /* ... */ })))
        .mount(&server)
        .await;
    // build client against server.uri(), assert body
}
```
Reference: `crates/codegen/xai-grok-tools/src/implementations/web_search/client.rs`

Mock inference (integration / e2e):
```rust
use xai_grok_test_support::{
    MockInferenceServer, run_headless, assert_headless_success, ScriptedResponse, sse,
};
// start mock server → run_headless / GrokStdioClient / PTY ContentController
```
API reference: `crates/codegen/xai-grok-test-support/README.md`

**What to Mock:**
- Remote inference HTTP (chat/completions, responses, messages)
- External download / update endpoints
- Persistence backends when testing actors in isolation
- Environment (`EnvGuard`, sandboxed `HOME`/`GROK_HOME` via test-support)

**What NOT to Mock:**
- Core pure logic (path encoding, matchers, serializers) — unit test directly
- Prefer real subprocess + mock server for agent protocol contracts over pure in-process fakes when testing wire shape
- Git: use hermetic helpers rather than assuming system git (`xai-test-utils` / `ensure_hermetic_git_on_path`)

## Fixtures and Factories

**Test Data:**
```rust
fn test_config() -> SamplingConfig {
    test_config_with_window(128_000)
}

fn pre_tool_use_envelope(tool_name: &str) -> HookEventEnvelope {
    HookEventEnvelope { /* fixed test session ids, cwd */ }
}
```

**Shared infrastructure crates:**

| Crate | Path | Role |
|-------|------|------|
| `xai-grok-test-support` | `crates/codegen/xai-grok-test-support/` | Mock inference, SSE builders, ACP stdio client, headless runner, env sandbox, UDS fault proxy |
| `xai-test-utils` | `crates/common/xai-test-utils/` | Hermetic git, runfiles/`CARGO_MANIFEST_DIR`, tracing capture counters |
| `xai-grok-pager-pty-harness` | `crates/codegen/xai-grok-pager-pty-harness/` | PTY spawn, virtual screen, scripted YAML scenarios, content controller |

**Location:**
- Small fixtures: constants / helpers in the test module
- Larger suites: `tests/fixtures/`, `tests/common/`
- SSE / API payloads: builders in `xai_grok_test_support::sse` (byte-exact vs collapsing variants are intentional — keep both green)
- YAML PTY scenarios: `crates/codegen/xai-grok-pager/tests/scenarios/*.yaml`

**Env & process hermeticity:**
- Sandbox **both** `HOME` and `GROK_HOME` (Windows uses USERPROFILE-equivalent paths)
- Kill telemetry / real network with test env helpers (`test_env_cmd_tokio`, etc.)
- Binary resolution: `GROK_BINARY` → cargo bin / local debug `xai-grok-pager`
- Process-global env: mark tests `#[serial_test::serial]` and use `EnvGuard`

## Coverage

**Requirements:** None enforced as a numeric gate in this public tree (no tarpaulin/llvm-cov config detected)

**Practical coverage strategy:**
- Unit tests next to pure logic and parsers
- Integration tests for discovery → dispatch pipelines
- `#[ignore]` e2e for binary/PTY paths that CI can opt into
- Snapshot tests for multi-line UI/diff formatting

**View Coverage:**
```bash
# Not standardized in-repo; local optional:
cargo llvm-cov -p <crate>   # if llvm-cov installed
```

## Test Types

**Unit Tests:**
- Scope: single module/function behavior, pure transforms, config parse/validate, path policy
- Approach: `#[cfg(test)]` + `#[test]` / `#[tokio::test]`
- Use `tempfile` when touching disk; avoid network

**Integration Tests:**
- Scope: crate public API across modules (hooks load+dispatch, crash handler install, telemetry, sampler actor)
- Approach: `tests/*.rs` with real temp files and async runtime
- Prefer inline shell command strings in hooks tests to avoid `noexec` tmpdir issues in sandboxes

**E2E / Binary Tests:**
- Shell: `tests/test_built_binary_e2e.rs`, leader stdio, MCP integration — often `#[ignore]` until binary is built
- Pager: PTY harness + YAML scripted scenarios (`scripted_scenarios.rs`); ignored by default (builds/spawns pager)
- Run with `cargo test -p <crate> -- --ignored` after building `xai-grok-pager` / setting `GROK_BINARY`

**Protocol / wire tests:**
- ACP stdio via `GrokStdioClient` / `RawStdioClient` in `xai-grok-test-support`
- Tools API wire shape tests: `crates/codegen/xai-grok-tools-api/tests/wire_shape.rs`

**Subprocess isolation:**
- Crash handler tests re-exec the test binary with env-selected scenarios so fatal signals do not kill the parent (`crates/codegen/xai-crash-handler/tests/integration.rs`)

**Fuzz:**
- Markdown: `crates/codegen/xai-grok-markdown/fuzz/` (cargo-fuzz style targets + seeds)

**Benchmarks:**
- `criterion` benches in select crates (`xai-fsnotify`, `xai-grok-markdown`, `xai-grok-pager`, `xai-grok-shell`, `xai-ratatui-inline`, PTY harness)
- Example: `cargo bench -p xai-fsnotify --bench startup`
- PTY baselines documented under `crates/codegen/xai-grok-pager-pty-harness/benches/pty_baselines/`

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn name() {
    let result = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("timed out waiting for event")
        .expect("channel closed");
}

#[tokio::test(start_paused = true)]
async fn timer_logic_advances_with_virtual_time() {
    // requires tokio test-util feature in dev-dependencies
}
```

**Error Testing:**
```rust
assert!(HookMatcher::new("[invalid").is_err());
assert!(matches!(&errors[0], HookError::InvalidMatcher { .. }));
assert_matches!(req.tool_choice, Some(ConversationToolChoice::Auto));
```

**Snapshot Testing:**
```rust
insta::assert_snapshot!("diff_basic", diff_outputs_to_string(&outputs));
```
Reference: `crates/codegen/xai-grok-pager/src/scrollback/blocks/tool/edit.rs`  
Update intentionally with insta’s accept workflow when output changes.

**Serial / env mutation:**
```rust
#[serial_test::serial]
#[test]
fn mutates_process_env() {
    let _guard = EnvGuard::set("SOME_KEY", "value");
    // ...
} // restores prior value on drop
```

**YAML PTY scenarios:**
```rust
async fn run_scenario(name: &str) {
    let scenario = ScriptedScenario::from_file(&scenario_path(name)).expect("load scenario");
    let report = runner.run(&scenario).await.expect("run scenario");
    assert_eq!(report.status, ScriptedRunStatus::Passed, ...);
}
```
Reference: `crates/codegen/xai-grok-pager/tests/scripted_scenarios.rs`

**Tracing assertions:**
- `xai_test_utils::tracing_capture::MessagePrefixCounter` for counting log prefixes

**Git in tests:**
- `xai_test_utils::git` / `require_git!` and shell `ensure_hermetic_git_on_path`
- `git_workdir()` from test-support for temp repos with forced libgit2 init

## Adding Tests — Quick Guide

| Change type | Where to put tests | Tools |
|-------------|-------------------|--------|
| Pure function / parser | `#[cfg(test)]` in same file | assert*, tempfile if needed |
| New HTTP client behavior | Same file or `tests/` + wiremock | `wiremock` |
| Agent/sampling wire contract | shell `tests/` or sampler tests | `xai-grok-test-support` SSE + mock server |
| TUI interaction | pager `tests/scenarios/*.yaml` + scripted runner | `xai-grok-pager-pty-harness` |
| Multi-line render/diff | unit test + insta snapshot | `insta` |
| Env-dependent config | `#[serial]` + `EnvGuard` | `serial_test`, test-support |
| New test-support capability | `xai-grok-test-support` + update its README | see crate README “Adding a capability” |

## Dev-Dependencies (Workspace Pins)

Common test-related workspace deps (root `Cargo.toml`):
- `tempfile`, `tokio` (+ `test-util` where needed)
- `wiremock`, `serial_test`, `assert_matches`, `pretty_assertions`
- `criterion` (benches), `insta` (via pager crate deps where used)
- `xai-grok-test-support`, `xai-test-utils`

Enable in the crate under test with `{ workspace = true }` in `[dev-dependencies]`.

---

*Testing analysis: 2026-07-16*
