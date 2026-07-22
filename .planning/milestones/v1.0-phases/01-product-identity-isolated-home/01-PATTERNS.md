# Phase 1: Product identity & isolated home - Pattern Map

**Mapped:** 2026-07-16
**Files analyzed:** 16
**Analogs found:** 15 / 16

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/codegen/xai-grok-config/src/paths.rs` | utility / config | file-I/O (path resolve + create_dir) | self (edit in place) | exact |
| `crates/codegen/xai-fast-worktree/src/db/mod.rs` (`resolve_grok_home`, `GrokHomeFixture`) | utility / store | file-I/O | `xai-grok-config/src/paths.rs` + self fixture | exact (twin) |
| `crates/codegen/xai-grok-pager-bin/Cargo.toml` (`[[bin]]`, `default-run`) | config | transform (build artifact name) | self `[[bin]]` block | exact |
| `crates/codegen/xai-grok-test-support/src/env.rs` | utility / test | request-response (spawn env) | self `grok_binary` + `test_env_cmd_tokio` | exact |
| `crates/codegen/xai-grok-pager-pty-harness/src/env.rs` | utility / test | request-response | `xai-grok-test-support/src/env.rs` | exact |
| `crates/codegen/xai-grok-pager-pty-harness/src/content.rs` (`env_for_pager`) | test harness | request-response | `test_env_cmd_tokio` | exact |
| `crates/codegen/xai-grok-pager-render/src/util.rs` (display home labels) | utility | transform (path → display string) | self `display_grok_home_prefix` | exact |
| `crates/codegen/xai-grok-shell/src/leader/mod.rs` (`managed_grok_bin_name`) | utility | file-I/O | self + `grok_application()` | exact |
| `crates/codegen/xai-grok-config/src/paths.rs` (`grok_application`) | utility | file-I/O | self lines 60–64 | exact |
| `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` | test | file-I/O | self isolated-binary OnceLock pattern | exact |
| `crates/codegen/xai-grok-update/tests/common/mod.rs` (`test_home`) | test | file-I/O | self OnceLock + set env | exact |
| `crates/codegen/xai-grok-agent/src/discovery.rs` (`user_agent_dirs` legacy) | utility | file-I/O (read scan) | self legacy `~/.grok` branch | exact |
| `crates/codegen/xai-grok-workspace/src/worktree/mod.rs` (fallback `.grok`) | utility | file-I/O | `resolve_grok_home` twin | role-match |
| `crates/codegen/xai-grok-sandbox/src/paths.rs` (docs/comment) | utility | file-I/O | re-export of config SoT | role-match |
| New isolation integration test (e.g. under pager or shell `tests/`) | test | file-I/O + process spawn | compose `test_env_cmd_tokio` + `test_built_binary_e2e` + `grok_home_paths` | partial |
| Path writers (`grok_home().join(...)`) — no mass rewrite if SoT cutover | utility | file-I/O | marketplace/mcp/sessions joins | exact (leave call sites) |

## Pattern Assignments

### `crates/codegen/xai-grok-config/src/paths.rs` (utility, file-I/O) — **SoT cutover**

**Analog:** same file (edit in place). This is the single source of truth; almost all product-state writers hang off `grok_home()`.

**Imports / module header pattern** (lines 1–6):
```rust
//! Filesystem locations for grok config files and binaries.

use std::path::PathBuf;
use std::sync::OnceLock;

static GROK_HOME: OnceLock<PathBuf> = OnceLock::new();
```
Keep the `OnceLock` name/symbol this phase (discretion: internal `grok_home` may remain; document that it returns bum product home).

**Default home + dunce pattern** (lines 14–32) — change `.grok` → `.bum`, update docs:
```rust
/// Keep the dunce canonicalization in sync with the hand-rolled duplicate in
/// `xai_fast_worktree::db::resolve_grok_home` (deliberately standalone crate).
pub fn default_grok_home() -> PathBuf {
    #[allow(deprecated)]
    let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dunce::canonicalize(&home).unwrap_or(home).join(".grok") // → join(".bum")
}
```

**Env override + create_dir pattern** (lines 34–47) — change env key `GROK_HOME` → `BUM_HOME` only (do **not** dual-read `GROK_HOME`):
```rust
pub fn grok_home() -> PathBuf {
    GROK_HOME
        .get_or_init(|| {
            let grok_home = if let Ok(v) = std::env::var("GROK_HOME") { // → "BUM_HOME"
                PathBuf::from(v)
            } else {
                default_grok_home()
            };
            let _ = std::fs::create_dir_all(&grok_home);
            grok_home
        })
        .clone()
}
```

**`user_grok_home` env probe** (lines 49–58) — switch `var_os("GROK_HOME")` → `var_os("BUM_HOME")`.

**Managed application binary under home** (lines 60–64) — rename managed leaf `grok` → `bum`:
```rust
pub fn grok_application() -> PathBuf {
    let name = if cfg!(windows) { "grok.exe" } else { "grok" }; // → "bum.exe" / "bum"
    grok_home().join("bin").join(name)
}
```

**Sessions path writer pattern** (lines 149–166) — leave structure; auto-fixed via SoT:
```rust
pub fn sessions_cwd_dir(cwd: &str) -> PathBuf {
    grok_home().join("sessions").join(encode_cwd_dirname(cwd))
}
```

**Unit test for default dir** (lines 303–311) — flip assertion to `.bum`:
```rust
#[test]
fn default_grok_home_has_no_verbatim_prefix() {
    let home = default_grok_home();
    assert!(!home.to_string_lossy().starts_with(r"\\?\"));
    assert!(home.ends_with(".grok")); // → ends_with(".bum")
}
```

**New unit tests to add (same module style):**
- Default ends with `.bum`
- With sandboxed HOME + unset `BUM_HOME` → still `.bum` under that HOME
- `GROK_HOME` set, `BUM_HOME` unset → **ignored** (product home still default `.bum` under HOME) — requires isolated process or pre-OnceLock setup (see isolated binary pattern below)

**Leave alone this phase:** `system_config_dir()` → `/etc/grok` (Phase 8); project-local `.grok/` is not this file.

---

### `crates/codegen/xai-fast-worktree/src/db/mod.rs` (utility, file-I/O) — **twin resolver**

**Analog:** `xai-grok-config/src/paths.rs` (`default_grok_home` / `grok_home`) — comments already require lockstep.

**Core twin pattern** (lines 340–349):
```rust
pub fn resolve_grok_home() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("GROK_HOME") { // → "BUM_HOME"
        return Ok(PathBuf::from(v));
    }
    let home = PathBuf::from(std::env::var("HOME").context("neither $GROK_HOME nor $HOME is set")?);
    // Canonicalize ... must stay in sync with xai_grok_config::default_grok_home();
    Ok(dunce::canonicalize(&home).unwrap_or(home).join(".grok")) // → ".bum"
}
```
Update error context string to mention `$BUM_HOME` / `$HOME`.

**Test fixture pattern** (lines 352–410) — switch env key; keep Mutex + TempDir RAII:
```rust
#[cfg(test)]
pub(crate) struct GrokHomeFixture {
    _lock: std::sync::MutexGuard<'static, ()>,
    prev: Option<std::ffi::OsString>,
    pub home: PathBuf,
    _tmp: tempfile::TempDir,
}

impl GrokHomeFixture {
    pub(crate) fn new() -> Self {
        let lock = GROK_HOME_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::TempDir::new().unwrap();
        let home = tmp.path().join("grok-home"); // optional: "bum-home"
        std::fs::create_dir_all(&home).unwrap();
        let _ = WorktreeDb::open(&home);
        let prev = std::env::var_os("GROK_HOME"); // → "BUM_HOME"
        unsafe { std::env::set_var("GROK_HOME", &home) }; // → "BUM_HOME"
        Self { _lock: lock, prev, home, _tmp: tmp }
    }
}
// Drop restores BUM_HOME the same way
```

**Downstream that uses twin (fallback must match):**  
`crates/codegen/xai-grok-workspace/src/worktree/mod.rs` lines 621–631:
```rust
fn grok_home() -> std::path::PathBuf {
    xai_fast_worktree::resolve_grok_home().unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join(".grok") // → ".bum" to match twin default if resolve fails
    })
}
```

---

### `crates/codegen/xai-grok-pager-bin/Cargo.toml` (config, build transform) — **ID-01 binary**

**Analog:** current ship surface (lines 7–16):
```toml
default-run = "xai-grok-pager"
[[bin]]
name = "xai-grok-pager"
path = "src/main.rs"
```

**Target pattern:**
```toml
default-run = "bum"
[[bin]]
name = "bum"
path = "src/main.rs"
```

**Rules to copy:**
- Package `name = "xai-grok-pager-bin"` stays (CONTEXT: crate paths unchanged)
- Root workspace `Cargo.toml` is generated — edit **only** this per-crate manifest
- Prefer single `[[bin]]` named `bum` (no dual-bin alias unless CI forces it)
- Comment block at top can note composition-root purpose; update “artifact named …” wording to `bum`

**Local invocable pattern (dev):**
```bash
cargo run -p xai-grok-pager-bin --bin bum
cargo build -p xai-grok-pager-bin --bin bum
```

---

### `crates/codegen/xai-grok-test-support/src/env.rs` (utility/test, spawn env)

**Analog:** same file — three sub-patterns.

#### EnvGuard RAII (lines 9–49) — reuse as-is

```rust
pub struct EnvGuard {
    key: &'static str,
    prior: Option<OsString>,
}

impl EnvGuard {
    pub fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
        let prior = std::env::var_os(key);
        // SAFETY: callers are `#[serial]`, so no other thread touches the env.
        unsafe { std::env::set_var(key, value) };
        Self { key, prior }
    }
    pub fn unset(key: &'static str) -> Self { /* remove_var + restore on drop */ }
}
```

**Phase 1 usage for unit tests that can run before OnceLock init:**
```rust
let _home = EnvGuard::set("BUM_HOME", temp_home.path());
// NEVER set GROK_HOME as product override in production code paths
```

#### Binary resolution (lines 66–116) — rename artifact + package

```rust
fn local_grok_binary_path() -> PathBuf {
    target_dir()
        .join("debug")
        .join(format!("xai-grok-pager{}", std::env::consts::EXE_SUFFIX))
        // → format!("bum{}", EXE_SUFFIX)
}

// build args today wrongly use -p xai-grok-pager; research: prefer pager-bin
.args(["build", "-p", "xai-grok-pager", "--bin", "xai-grok-pager"])
// → ["build", "-p", "xai-grok-pager-bin", "--bin", "bum"]

// CARGO_BIN_EXE_xai-grok-pager → CARGO_BIN_EXE_bum
if let Ok(path) = std::env::var("CARGO_BIN_EXE_xai-grok-pager") { ... }
```

Keep env override name `GROK_BINARY` this phase (full `GROK_*` renames = Phase 8) unless planner chooses optional `BUM_BINARY` dual-read — CONTEXT allows leaving non-home knobs.

#### Child-process sandbox env (lines 152–177) — switch home key

```rust
pub fn test_env_cmd_tokio(
    cmd: &mut tokio::process::Command,
    mock_url: &str,
    home: &std::path::Path,
) {
    cmd.env("HOME", home)
        // Windows: HOME alone insufficient — set explicit product home
        .env("GROK_HOME", home.join(".grok")) // → .env("BUM_HOME", home.join(".bum"))
        .env("GROK_CLI_CHAT_PROXY_BASE_URL", mock_url)
        .env("GROK_XAI_API_BASE_URL", mock_url)
        .env("XAI_API_KEY", "test-key-for-ci")
        .env("GROK_TELEMETRY_ENABLED", "false")
        .env("GROK_FEEDBACK_ENABLED", "false")
        .env("GROK_TRACE_UPLOAD", "false")
        .env("GROK_INSTRUMENTATION", "disabled")
        .env("GROK_DISABLE_AUTOUPDATER", "1");
}
```

**Convention pick (research Q2):** keep parity with today — `BUM_HOME = home.join(".bum")` while `HOME = home`, so isolation asserts can check `HOME/.bum` for product files and prove no `HOME/.grok`.

---

### `crates/codegen/xai-grok-pager-pty-harness/src/env.rs` (utility/test)

**Analog:** `xai-grok-test-support/src/env.rs` binary resolver (pty-harness already uses correct package).

**Current pattern** (lines 26–99) — already `-p xai-grok-pager-bin`:
```rust
fn local_pager_binary_path() -> Result<PathBuf> {
    Ok(target_dir()?
        .join("debug")
        .join(format!("xai-grok-pager{}", std::env::consts::EXE_SUFFIX))) // → bum
}

cmd.args([
    "build",
    "-p",
    "xai-grok-pager-bin",
    "--bin",
    "xai-grok-pager", // → "bum"
]);

// CARGO_BIN_EXE_xai-grok-pager → CARGO_BIN_EXE_bum
// PAGER_BINARY override stays as-is (Phase 8 naming optional)
```

---

### `crates/codegen/xai-grok-pager-pty-harness/src/content.rs` (`env_for_pager`)

**Analog:** `test_env_cmd_tokio` — keep mirror contract.

**Core pattern** (lines 79–92):
```rust
pub fn env_for_pager(&self) -> Vec<(String, String)> {
    let home = self.home.path().to_string_lossy().into_owned();
    let grok_home = self
        .home
        .path()
        .join(".grok") // → ".bum"
        .to_string_lossy()
        .into_owned();
    vec![
        ("HOME".into(), home),
        ("GROK_HOME".into(), grok_home), // → ("BUM_HOME".into(), ...)
        // ... telemetry / mock URL knobs unchanged this phase
    ]
}
```

Update unit `env_for_pager_shape` assertions that expect `GROK_HOME` key and `.grok` path (lines ~267–278).

---

### `crates/codegen/xai-grok-pager-render/src/util.rs` (utility, display labels)

**Analog:** self — Phase 1 must fix **path labels** (CONTEXT exception for wrong home path).

**Core pattern** (lines 9–24):
```rust
pub use xai_grok_config::grok_home;

pub fn pager_toml_path() -> PathBuf {
    grok_home().join("pager.toml") // auto-fixed via SoT
}

pub fn display_grok_home_prefix() -> String {
    if grok_home() == xai_grok_config::default_grok_home() {
        "~/.grok".to_string() // → "~/.bum"
    } else {
        "$GROK_HOME".to_string() // → "$BUM_HOME"
    }
}
```

**Unit tests** (lines 399–411) — flip `~/.grok` / `$GROK_HOME` / skip-if-env checks to `BUM_HOME` / `~/.bum`.  
`abbreviate_path_uses_home_when_under_default_grok` builds `{home}/.grok/...` — change to `.bum`.

---

### `crates/codegen/xai-grok-shell/src/leader/mod.rs` (`managed_grok_bin_name`)

**Analog:** `grok_application()` in config paths (same leaf name).

**Core pattern** (lines 1355–1367):
```rust
fn managed_grok_bin_name() -> &'static str {
    if cfg!(windows) { "grok.exe" } else { "grok" } // → "bum.exe" / "bum"
}
fn resolve_binary_impl(
    grok_home: &Path,
    current_exe: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, ConnectionError> {
    let managed_bin = grok_home.join("bin").join(managed_grok_bin_name());
    // prefer managed when current_exe is under product home
    ...
}
```

Leader socket defaults under `grok_home()` via `leader/lock.rs` (`socket_path_for_ws_url` uses `grok_home()`) — **no socket env rename** this phase (`GROK_LEADER_SOCKET` stays).

---

### `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` (test, isolated OnceLock)

**Analog:** self + update `tests/common` OnceLock pattern.

**Core pattern** — **process-isolated binary** so OnceLock sees env at first call:
```rust
//! `GROK_HOME` override tests in an isolated binary so `grok_home()`'s
//! process-wide `OnceLock` initializes from the overridden env var.

#[test]
fn grok_home_override_path_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let grok_home = tmp.path().to_path_buf();
    unsafe {
        std::env::set_var("GROK_HOME", &grok_home); // → "BUM_HOME"
    }

    assert_eq!(
        xai_grok_pager::util::pager_toml_path(),
        grok_home.join("pager.toml")
    );
    assert_eq!(
        xai_grok_pager::util::display_grok_home_prefix(),
        "$GROK_HOME" // → "$BUM_HOME"
    );
    // ...
}
```

Optional: rename file to `bum_home_paths.rs` only if planner wants clarity; not required.

---

### `crates/codegen/xai-grok-update/tests/common/mod.rs` (`test_home`)

**Analog:** self OnceLock-per-integration-binary pattern (lines 38–61).

```rust
pub fn test_home() -> &'static PathBuf {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.keep();
        unsafe {
            std::env::set_var("GROK_HOME", &path); // → "BUM_HOME"
            // other GROK_* clearances can stay until Phase 8
        }
        path
    })
}
```

**When to use:** any integration binary that calls `grok_home()` and cannot re-init OnceLock mid-process. Prefer this over EnvGuard when the first call is unavoidable at process start.

---

### `crates/codegen/xai-grok-agent/src/discovery.rs` (`user_agent_dirs` legacy)

**Analog:** self (lines 197–223) — read-side identity leak when product home is bum.

```rust
pub(crate) fn user_agent_dirs(
    home: Option<&Path>,
    grok_home: Option<&Path>,
) -> Vec<(std::path::PathBuf, AgentScope)> {
    // Legacy literal ~/.grok when GROK_HOME points elsewhere
    let legacy_grok = home
        .map(|h| h.join(".grok"))
        .filter(|legacy| grok_home != Some(legacy.as_path()));
    // ...
}
```

**Phase 1 target pattern:** remove or gate off stock-home legacy scan so bum does not treat `~/.grok/agents` as first-class user state (research recommendation). Keep **project** `cwd/.grok` discovery elsewhere untouched.

**Tests to update:** `user_agent_dirs_includes_legacy_grok_when_grok_home_differs` (lines 766+) — expect legacy **excluded** after cutover, or rename test to assert isolation.

---

### Path writers under `grok_home().join(...)` (leave call sites)

**Analog examples (exact — no rewrite if SoT changes):**

| Writer | Pattern |
|--------|---------|
| Marketplace | `xai_grok_config::grok_home().join("marketplace-cache")` (`plugin-marketplace/src/git.rs:147`) |
| MCP OAuth lock | `grok_home().join(format!("mcp_auth_{safe}.lock"))` |
| MCP logs | `grok_home().join("logs").join("mcp")` |
| Sandbox | `grok_home().join("sandbox.toml")`, `sandbox-events.jsonl` |
| Auth default | `grok_home.join("auth.json")` via `AuthManager::new` |
| Sessions | `grok_home().join("sessions").join(...)` |

**Auth path priority** (`auth/manager.rs` 280–298) — leave override env **names** Phase 8; default path fixed by SoT:
```rust
// GROK_AUTH inline → GROK_AUTH_PATH → grok_home.join("auth.json")
let path = std::env::var("GROK_AUTH_PATH")
    .map(PathBuf::from)
    .unwrap_or_else(|_| grok_home.join("auth.json"));
```

**Re-exports (no logic):** keep thin re-export modules; they pick up SoT automatically:
- `xai-grok-shell-base/src/util/grok_home.rs`
- `xai-grok-tools/src/util/grok_home.rs`
- `xai-grok-pager-render` `pub use xai_grok_config::grok_home`
- `xai-grok-sandbox/src/paths.rs` → `xai_grok_config::grok_home()` (update doc comment `$GROK_HOME`/`~/.grok` → `$BUM_HOME`/`~/.bum`)

---

### New isolation integration test (partial analog)

**No single existing “zero writes under .grok/.codex” test.** Compose:

1. **Sandbox env** from `test_env_cmd_tokio` / `env_for_pager` (`HOME` + `BUM_HOME`, telemetry off)
2. **Binary resolve** from `grok_binary()` / `pager_binary()` after rename
3. **OnceLock safety** from `grok_home_paths` or child process (spawn `bum` so parent OnceLock irrelevant)
4. **Assertions sketch:**
```rust
// After short headless / path-init under TempDir HOME + BUM_HOME=home.join(".bum"):
let bum_home = home.join(".bum");
assert!(bum_home.exists() || /* known product file under bum_home */);
assert!(!home.join(".grok").exists() || no_product_files(home.join(".grok")));
assert!(!home.join(".codex").exists() || no_product_writes(home.join(".codex")));
// Prefer: known product filenames only appear under BUM_HOME
```

**Closest structural e2e shell:** `crates/codegen/xai-grok-shell/tests/test_built_binary_e2e.rs` — uses `test_env_cmd_tokio` + `grok_binary()`; adapt for isolation asserts rather than mock inference if a lighter entrypoint exists (`--version` alone may not write state; prefer a path that calls `grok_home()` / config ensure, or pure library init in isolated test binary).

## Shared Patterns

### Product home resolution (OnceLock SoT)
**Source:** `crates/codegen/xai-grok-config/src/paths.rs` lines 6–47  
**Apply to:** all production writers (via `grok_home()`), sandbox re-export, sessions, auth default path  
**Rules:**
- Default: `dunce::canonicalize(home_dir).join(".bum")`
- Env: **only** `BUM_HOME` (never honor `GROK_HOME`)
- Side effect: `create_dir_all` on first resolve
- Twin must match: `xai_fast_worktree::db::resolve_grok_home`

### EnvGuard + serial tests
**Source:** `crates/codegen/xai-grok-test-support/src/env.rs` lines 9–49  
**Apply to:** unit tests mutating env in shared process  
```rust
#[serial_test::serial]
fn example() {
    let _g = EnvGuard::set("BUM_HOME", tmp.path());
    // assertions — but only if grok_home() not yet initialized in process
}
```

### Isolated integration-binary home
**Source:** `xai-grok-update/tests/common/mod.rs` `test_home` + `pager/tests/grok_home_paths.rs`  
**Apply to:** tests that must call `grok_home()` after setting env  
- Integration test file = separate process → own OnceLock  
- Set `BUM_HOME` **before** first `grok_home()` call

### Binary artifact resolution
**Source:** test-support `grok_binary` + pty-harness `pager_binary`  
**Apply to:** all harnesses that spawn the product CLI  
**Order pattern:**
1. Explicit override env (`GROK_BINARY` / `PAGER_BINARY`)
2. `CARGO_BIN_EXE_bum`
3. Local `target/debug/bum` + `cargo build -p xai-grok-pager-bin --bin bum`

### Child sandbox env bundle
**Source:** `test_env_cmd_tokio` + `ContentController::env_for_pager`  
**Apply to:** any spawned product process in tests  
Must set: `HOME`, **`BUM_HOME`** (not `GROK_HOME`), mock URLs, telemetry off, autoupdater disabled. Windows rationale in comments must stay.

### Path display labels
**Source:** `pager-render/src/util.rs` `display_grok_home_prefix`  
**Apply to:** any user-visible path prefix for product home (Phase 1 path correctness only)  
Default → `~/.bum`; override → `$BUM_HOME`

### Managed install leaf name
**Source:** `grok_application()` + `managed_grok_bin_name()`  
**Apply to:** managed `home/bin/*` layout  
Unix `bum`, Windows `bum.exe`

### Auth file under home
**Source:** `shell/src/auth/manager.rs` 261–298  
**Apply to:** auth store location  
Default `product_home/auth.json`; `GROK_AUTH` / `GROK_AUTH_PATH` names deferred Phase 8

### Project vs product home
**Source:** CONTEXT + discovery/project paths  
**Apply to:** greps and renames  
- **Change:** `$HOME/.bum` product root / `BUM_HOME`  
- **Do not change:** workspace `cwd/.grok/*`, `.grok-plugin`, project hooks layout

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| New isolation test: zero product writes under `.grok` / `.codex` | test | file-I/O + spawn | No existing assert of that shape; compose from sandbox env + child spawn patterns above |

## Metadata

**Analog search scope:**  
`crates/codegen/xai-grok-config`, `xai-fast-worktree`, `xai-grok-pager-bin`, `xai-grok-test-support`, `xai-grok-pager-pty-harness`, `xai-grok-pager-render`, `xai-grok-shell` (leader, auth), `xai-grok-agent` (discovery), `xai-grok-sandbox`, `xai-grok-workspace` (worktree), `xai-grok-update/tests`, writer samples (mcp, marketplace)

**Files scanned:** ~25 primary sources + greps for `GROK_HOME`, `CARGO_BIN_EXE`, `grok_home().join`, `managed_grok_bin_name`, `display_grok_home_prefix`

**Pattern extraction date:** 2026-07-16

**Planner wave alignment (from RESEARCH):**
1. SoT home cutover — paths.rs + unit tests + display labels  
2. Twin + managed paths — fast-worktree, workspace fallback, `grok_application`, `managed_grok_bin_name`  
3. Binary rename + harnesses — pager-bin, test-support, pty-harness  
4. Test env mass-update — `BUM_HOME` in sandboxes / fixtures  
5. Isolation proof + legacy read gate — new test + discovery legacy off  
