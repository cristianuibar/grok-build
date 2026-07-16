---
phase: 01-product-identity-isolated-home
plan: 05
subsystem: isolation
tags: [BUM_HOME, .bum, isolation, discovery, bundle, hermetic, D-HOME, D-VERIFY, D-WRITERS, D-MIGRATE, D-PLUGIN, D-SCOPE, ID-01, ID-03]

requires:
  - phase: 01-product-identity-isolated-home
    provides: "SoT BUM_HOME (01-01); twin/managed bin (01-02); bin bum (01-03); test fixtures BUM_HOME (01-04)"
provides:
  - "No stock ~/.grok user-agent scan when product home is bum"
  - "User roles/personas discovered under product home; project cwd/.grok preserved"
  - "bundled_root via grok_home()/bundled; extension tests inject explicit roots (OnceLock-safe)"
  - "Hermetic home_isolation proof: HOME+BUM_HOME, GROK/CODEX traps, recursive zero stock writes"
  - "Production product-root readers no longer honor GROK_HOME (changelog, voice_probe, auth diag)"
  - "Operational path labels teach BUM_HOME / ~/.bum (hub_auth, mcp credentials, hooks trust, mcp_doctor)"
  - "Classified shell-inclusive static gate green"
affects:
  - phase-1-verify-work
  - multi-provider-auth
  - all product writers via grok_home/bundled_root

tech-stack:
  added: [tempfile (pager-bin dev-dep for home_isolation)]
  patterns:
    - "bundled_root = xai_grok_config::grok_home().join(\"bundled\")"
    - "OnceLock-safe tests: inject temp root or isolated integration binary with BUM_HOME set once"
    - "Hermetic child: env_clear + HOME + BUM_HOME + trap GROK_HOME/CODEX_HOME + telemetry off"

key-files:
  created:
    - crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs
    - crates/codegen/xai-grok-shell/tests/test_bundled_root_product_home.rs
  modified:
    - crates/codegen/xai-grok-agent/src/discovery.rs
    - crates/codegen/xai-grok-shell/src/bundle.rs
    - crates/codegen/xai-grok-shell/src/extensions/bundle.rs
    - crates/codegen/xai-grok-shell/src/config/mod.rs
    - crates/codegen/xai-grok-shell/src/config/tests.rs
    - crates/codegen/xai-grok-shell-base/src/util/changelog.rs
    - crates/codegen/xai-grok-voice/src/bin/voice_probe.rs
    - crates/codegen/xai-grok-shell/src/auth/manager.rs
    - crates/codegen/xai-grok-workspace/src/hub_auth.rs
    - crates/codegen/xai-grok-mcp/src/credentials.rs
    - crates/codegen/xai-grok-mcp/src/lib.rs
    - crates/codegen/xai-grok-hooks/src/trust.rs
    - crates/codegen/xai-grok-shell/src/mcp_doctor.rs
    - crates/codegen/xai-grok-pager-bin/Cargo.toml
    - Cargo.lock

key-decisions:
  - "Drop legacy HOME/.grok agent scan entirely when product home differs — no dual-read migration (D-MIGRATE)"
  - "User roles/personas under product_home/roles|personas via discover_*_in_dir, not home/.grok via dirs::home_dir"
  - "Extension bundle unit tests inject explicit TempDir roots instead of multi-scenario BUM_HOME/OnceLock fights"
  - "Changelog from_env_home reads BUM_HOME only (or falls back to grok_home()); never GROK_HOME"
  - "Operational labels only (D-SCOPE) — no Phase-8 chrome/telemetry rebrand"

patterns-established:
  - "Production writer roots: always grok_home()/join — never dirs::home_dir().join(\".grok\")"
  - "Lib-test OnceLock: write fixtures into process grok_home() under #[serial], or use integration binary"
  - "Isolation proof: recursive snapshot equality on trap trees, not filename allowlists"

requirements-completed: [ID-01, ID-03]

coverage:
  - id: D1
    description: "Agent discovery does not scan stock ~/.grok user agents when product home differs; project .grok preserved"
    requirement: ID-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-agent discovery — user_agent_dirs_uses_product_home_not_stock_grok_when_differs"
        status: pass
    human_judgment: false
  - id: D2
    description: "bundled_root under product home; roles/personas user discovery under product home"
    requirement: ID-03
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test test_bundled_root_product_home"
        status: pass
      - kind: other
        ref: "cargo check -p xai-grok-shell --lib (production resolve uses grok_home)"
        status: pass
      - kind: other
        ref: "cargo test -p xai-grok-shell bundle|config (lib) — pre-existing compile break; deferred"
        status: unknown
    human_judgment: false
  - id: D3
    description: "Hermetic temp-home bum run: product writes under BUM_HOME; recursive zero stock .grok/.codex trap writes"
    requirement: ID-03
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-pager-bin --test home_isolation"
        status: pass
    human_judgment: false
  - id: D4
    description: "Production product-root env readers + operational path labels use BUM_HOME / ~/.bum; shell-inclusive gate clean"
    requirement: ID-01
    verification:
      - kind: other
        ref: "! rg GROK_HOME env reads in config/fast-worktree/shell-base/voice/shell production sources"
        status: pass
      - kind: other
        ref: "! rg $GROK_HOME|~/.grok in hub_auth, credentials, trust, mcp_doctor"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-config --lib paths"
        status: pass
      - kind: other
        ref: "cargo build -p xai-grok-pager-bin --bin bum"
        status: pass
    human_judgment: false

duration: 20min
completed: 2026-07-16
status: complete
---

# Phase 1 Plan 05: Isolation proof + production cutover Summary

**Legacy stock-home agent/role/bundle reads gated; hermetic recursive isolation proven; production GROK_HOME bypasses and operational product-home labels cut over to BUM_HOME / ~/.bum**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-07-16T03:26:10Z
- **Completed:** 2026-07-16T03:46:00Z
- **Tasks:** 3/3
- **Files modified:** 16 (+ Cargo.lock)

## Accomplishments

- Removed legacy `~/.grok` user-agent scan; product-home agents/bundled only; project `cwd/.grok` kept
- `SubagentsConfig::resolve` discovers user roles/personas under `grok_home()`; `bundled_root()` → `grok_home()/bundled`
- Extension bundle tests use injected temp roots (OnceLock-safe); integration proof for bundled_root under BUM_HOME
- Hermetic `home_isolation` integration test: `CARGO_BIN_EXE_bum version` under HOME+BUM_HOME with GROK/CODEX traps; recursive unchanged asserts
- Changelog, voice_probe, auth diagnostic cut off GROK_HOME; hub_auth / mcp credentials / hooks trust / mcp_doctor teach BUM_HOME / ~/.bum
- Classified static gates clean; `bum` builds; Phase 1 isolation criteria ready for verify-work

## Task Commits

Each task was committed atomically:

1. **Task 1: Stock-home read gate — agents + roles/personas + bundle writer + extension tests** - `4c36451` (feat)
2. **Task 2: Hermetic temp-home isolation integration test** - `4a2f363` (feat) + `2e25e9f` (chore Cargo.lock tempfile)
3. **Task 3: Production bypass + operational product-home labels + classified gate + bum build** - `9f35952` (feat)

**Plan metadata:** _(final docs commit after this file)_

## Files Created/Modified

- `crates/codegen/xai-grok-agent/src/discovery.rs` — drop legacy stock-home agent dirs; tests assert not included
- `crates/codegen/xai-grok-shell/src/bundle.rs` — `bundled_root` via `grok_home()`
- `crates/codegen/xai-grok-shell/src/extensions/bundle.rs` — `test_bundled_root` injects temp paths
- `crates/codegen/xai-grok-shell/src/config/mod.rs` — user roles/personas under product home
- `crates/codegen/xai-grok-shell/src/config/tests.rs` — priority resolve fixtures under process product home
- `crates/codegen/xai-grok-shell/tests/test_bundled_root_product_home.rs` — integration OnceLock-safe proof
- `crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs` — hermetic isolation
- `crates/codegen/xai-grok-pager-bin/Cargo.toml` — tempfile dev-dep; bin remains `bum`
- `crates/codegen/xai-grok-shell-base/src/util/changelog.rs` — `from_env_home` reads BUM_HOME
- `crates/codegen/xai-grok-voice/src/bin/voice_probe.rs` — BUM_HOME + `~/.bum/config.toml`
- `crates/codegen/xai-grok-shell/src/auth/manager.rs` — diagnostic key BUM_HOME
- `crates/codegen/xai-grok-workspace/src/hub_auth.rs` — product-home path labels
- `crates/codegen/xai-grok-mcp/src/credentials.rs` + `lib.rs` — BUM_HOME store docs/errors
- `crates/codegen/xai-grok-hooks/src/trust.rs` — BUM_HOME missing-home guidance
- `crates/codegen/xai-grok-shell/src/mcp_doctor.rs` — `~/.bum/config.toml` labels

## Decisions Made

- **No credential import/migration** from stock Grok home (D-MIGRATE) — clean re-login only
- **Extension tests inject roots** rather than process-wide BUM_HOME multi-scenario (option a from plan)
- **Priority resolve test** writes under process `grok_home()` + serial instead of HOME env mutation
- **Operational labels only** — Phase-8 chrome/telemetry out of scope (D-SCOPE)
- **Project-local `.grok`** under cwd remains for plugins/agents/roles (D-PLUGIN)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical verification path] Integration test for bundled_root**
- **Found during:** Task 1
- **Issue:** `cargo test -p xai-grok-shell bundle|config` cannot compile the full lib-test crate (pre-existing; deferred since 01-02)
- **Fix:** Added `tests/test_bundled_root_product_home.rs` integration binary + production `cargo check --lib`
- **Files modified:** `crates/codegen/xai-grok-shell/tests/test_bundled_root_product_home.rs`
- **Commit:** `4c36451`

**2. [Rule 2 - Correctness] Priority resolve test reworked for OnceLock**
- **Found during:** Task 1
- **Issue:** Existing test mutated HOME expecting `$HOME/.grok/roles` and multi-scenario env; incompatible with product-home OnceLock
- **Fix:** Serial test writes fixtures under process `grok_home()` / `bundled_root()`
- **Files modified:** `crates/codegen/xai-grok-shell/src/config/tests.rs`
- **Commit:** `4c36451`

### Pre-existing (deferred)

- `xai-grok-shell` lib tests still fail to compile (~32 errors: `EnvVarGuard`, `WorkspaceOps::for_test`, `MemoryStorage::with_paths`, etc.) — same as 01-02/01-04; documented in `deferred-items.md`

## Allowlist / residual stock-home matches

Intentional residuals **not** required clean by this plan's gates:

- Project-layout `cwd/.grok` discovery (agents, roles, personas, config) — D-PLUGIN
- Isolation-proof comments and hermetic trap path names (`.grok-trap`)
- `#[cfg(test)]` fixtures that still place files under injected product roots named with historical path segments in unit fixtures (e.g. discover tests pass `home.join(".grok")` as the **product-home argument**, not stock scan)
- OnceLock symbol name `GROK_HOME` in paths.rs (internal static, not env key)
- Phase-8 chrome / telemetry / marketing strings outside operational label files

## Threat Flags

None new beyond plan mitigations (T-01-08 … T-01-15 applied).

## Known Stubs

None.

## Auth Gates

None.

## Verification Commands Run

```text
cargo test -p xai-grok-agent discovery -- --nocapture
cargo test -p xai-grok-shell --test test_bundled_root_product_home -- --nocapture
cargo check -p xai-grok-shell --lib
cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture
cargo build -p xai-grok-pager-bin --bin bum
cargo check -p xai-grok-shell-base -p xai-grok-voice -p xai-grok-shell -p xai-grok-mcp -p xai-grok-hooks -p xai-grok-workspace
cargo test -p xai-grok-config --lib paths -- --nocapture
! rg GROK_HOME env-read patterns (shell-inclusive production trees)
! rg join(".grok") in bundle/paths/db
! rg $GROK_HOME|~/.grok in hub_auth, credentials, trust, mcp_doctor
rg BUM_HOME|~/.bum in operational label files
```

## Self-Check: PASSED

- All key files present
- Commits 4c36451, 4a2f363, 9f35952, 2e25e9f present
