---
phase: 01-product-identity-isolated-home
verified: 2026-07-16T03:53:32Z
status: passed
score: 3/3 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
deferred:
  - truth: "Version / chrome strings still present as 'grok' (e.g. `bum version` prints `grok 0.1.220-alpha.4`)"
    addressed_in: "Phase 8"
    evidence: "Phase 8 goal: 'Product presents fully as bum and does not phone home or auto-update as stock Grok Build'; quiet fork & rebrand polish / full chrome strings"
  - truth: "Residual GROK_HOME / ~/.grok wording in non-gate shell comments/docs (watcher, campaigns, plan_mode comments)"
    addressed_in: "Phase 8"
    evidence: "Phase 8 rebrand polish; product-root env reads and mandatory operational labels already cut over in Phase 1"
notes:
  mvp_goal_format: "ROADMAP phase goal is not formal 'As a… I want to… so that…' user-story shape (valid=false). User Flow Coverage below uses plan objectives + roadmap success criteria. Consider `/gsd mvp-phase 1` if formal MVP UAT scripting is required."
---

# Phase 1: Product identity & isolated home — Verification Report

**Phase Goal:** User launches `bum` and all product state lives under isolated `~/.bum` (or `BUM_HOME`)  
**Verified:** 2026-07-16T03:53:32Z  
**Status:** passed  
**Re-verification:** No — initial verification  
**Mode:** mvp  

**Requirements:** ID-01, ID-03  

## User Flow Coverage

User story (from plan objectives / outcome of roadmap goal):  
«As a bum user, I want to have all product state under isolated `~/.bum` (or `BUM_HOME`), so that stock Grok/Codex homes are never used as bum's product root.»

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Invoke CLI | Primary command name is `bum` (not `grok` / `xai-grok-pager`) | `crates/codegen/xai-grok-pager-bin/Cargo.toml`: `default-run = "bum"`, sole `[[bin]] name = "bum"`; `cargo metadata` bins = `[bum]`; `target/debug/bum` exists; `cargo build -p xai-grok-pager-bin --bin xai-grok-pager` fails with "no bin target" | ✓ |
| Launch / one-shot run | Process starts and reports version | `./target/debug/bum version` under hermetic `HOME`+`BUM_HOME` exits 0 | ✓ |
| Product state lands under bum home | Config/auth/sessions/docs under `~/.bum` or `$BUM_HOME` | Hermetic run created `$BUM_HOME/active_sessions.json`, `docs/…`; `paths::grok_home` uses `BUM_HOME` then `.bum`; auth/session writers call `grok_home()` | ✓ |
| No stock product home | No product writes under `~/.grok` / `~/.codex` | `home_isolation::hermetic_temp_home_writes_only_under_bum_home` **PASS**; manual hermetic tree has no `.grok`; static gate: no production `var_os("GROK_HOME")` product-home reads in config/fast-worktree/shell-base/voice/shell | ✓ |
| **Outcome** | Stock Grok/Codex homes are never used as bum's product root | SoT + twin + hermetic isolation + agent discovery skip stock `~/.grok` agents | ✓ |

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | User can invoke the primary CLI as `bum` (not `grok` / `xai-grok-pager` as the shipped command name) | ✓ VERIFIED | `Cargo.toml` ship name `bum`; only bin target; binary builds and runs; old target name rejected by cargo |
| 2 | Fresh run creates/uses config, auth, and session paths under `~/.bum` (or `BUM_HOME`), not `~/.grok` or stock Codex paths | ✓ VERIFIED | `resolve_product_home` / `grok_home()` → `BUM_HOME` or `~/.bum`; sessions via `grok_home().join("sessions")`; auth via `AuthManager::new(&grok_home, …)`; hermetic run writes only under `.bum` |
| 3 | Running with a temporary home shows no writes under `~/.grok` / `~/.codex` for product state | ✓ VERIFIED | Integration test `hermetic_temp_home_writes_only_under_bum_home` traps `GROK_HOME`/`CODEX_HOME`, asserts stock trees unchanged/empty; re-run PASS |

**Score:** 3/3 truths verified (0 present-but-behavior-unverified)

### Supporting Plan Truths (sampled, non-reducing)

| Truth | Status | Evidence |
| ----- | ------ | -------- |
| Default product home ends in `.bum` | ✓ | `paths` unit tests 19/19; `default_grok_home_has_no_verbatim_prefix` |
| `GROK_HOME` ignored as product home | ✓ | `grok_home_ignore_grok_home` binary test PASS; pure API has no GROK_HOME param |
| Display labels `~/.bum` / `$BUM_HOME` | ✓ | `display_grok_home_prefix` + pager-render util tests PASS |
| Managed bin leaf `bin/bum` | ✓ | `grok_application_leaf_is_bum`; leader `managed_grok_bin_name` → `bum`; updater `swap_managed_bin_links` uses `bum` |
| Twin fast-worktree / workspace fallback `.bum` | ✓ | `resolve_grok_home` reads `BUM_HOME`; workspace fallback `.bum` |
| Harnesses set `BUM_HOME` | ✓ | test-support, pty content/leader/scripted, leader_cluster, session_startup |
| No stock-home legacy agent scan | ✓ | `user_agent_dirs_uses_product_home_not_stock_grok_when_differs` PASS |
| `bundled_root` under product home | ✓ | `test_bundled_root_product_home` 2/2 PASS |
| Roles/personas user discovery under product home | ✓ | `SubagentsConfig::resolve` uses `xai_grok_config::grok_home()` for roles/personas |
| Operational labels teach `BUM_HOME` / `~/.bum` | ✓ | hub_auth, mcp credentials, hooks trust, mcp_doctor — gate C clean, gate D hits present |
| Static gate: no production GROK_HOME product-root env reads | ✓ | Gate A exit 1 (no matches) across config/fast-worktree/shell-base/voice/shell src |

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Version / chrome still says `grok` | Phase 8 | Quiet fork & rebrand polish — full bum chrome/strings |
| 2 | Residual comment-level GROK_HOME / `~/.grok` wording outside mandatory label files | Phase 8 | Rebrand polish; not product-root writers |
| 3 | Pre-existing `xai-grok-shell` lib-test compile break (~31 errors) | Outside phase (infra) | Documented in `deferred-items.md`; production `cargo check` + integration isolation tests green |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/codegen/xai-grok-config/src/paths.rs` | Pure resolver + BUM_HOME SoT | ✓ VERIFIED | `resolve_product_home`, `grok_home`, leaf `bum`; substantive + unit-tested |
| `crates/codegen/xai-grok-config/src/lib.rs` | Docs `$BUM_HOME` / `~/.bum` | ✓ VERIFIED | Crate docs re-export product home contract |
| `crates/codegen/xai-grok-pager-render/src/util.rs` | Display `~/.bum` / `$BUM_HOME` | ✓ VERIFIED | `display_grok_home_prefix`; tests green |
| `crates/codegen/xai-grok-pager-bin/Cargo.toml` | `[[bin]]` / `default-run` `bum` | ✓ VERIFIED | Sole bin target |
| `crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs` | Hermetic isolation proof | ✓ VERIFIED | 162 lines; test PASS |
| `crates/codegen/xai-fast-worktree/src/db/mod.rs` | Twin `BUM_HOME` resolver | ✓ VERIFIED | Semantic lockstep with SoT |
| `crates/codegen/xai-grok-shell/src/leader/mod.rs` | Managed bin `bum` | ✓ VERIFIED | `managed_grok_bin_name` |
| `crates/codegen/xai-grok-update/src/auto_update.rs` | Install leaf `bum` | ✓ VERIFIED | `swap_managed_bin_links` |
| `crates/codegen/xai-grok-shell/src/bundle.rs` | `bundled_root` via `grok_home` | ✓ VERIFIED | No `$HOME/.grok/bundled` |
| `crates/codegen/xai-grok-agent/src/discovery.rs` | No stock `~/.grok` agent scan | ✓ VERIFIED | Tests assert product home only |
| `crates/codegen/xai-grok-test-support/src/env.rs` | `CARGO_BIN_EXE_bum` + `BUM_HOME` sandbox | ✓ VERIFIED | Resolvers + sandbox env |
| Operational label files (hub_auth, mcp credentials, hooks trust, mcp_doctor) | BUM_HOME guidance | ✓ VERIFIED | Gates C/D |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| Cargo `[[bin]] bum` | `CARGO_BIN_EXE_bum` / harness resolvers | test-support + pty-harness | ✓ WIRED | `CARGO_BIN_EXE_bum` used; build `--bin bum` in support |
| `resolve_product_home` / `grok_home()` | Auth, sessions, marketplace, logs | `grok_home().join(...)` | ✓ WIRED | auth flow, session persistence, mcp credentials, hub_auth |
| `bum` process (hermetic HOME+BUM_HOME) | Filesystem product sinks | startup writers | ✓ WIRED | home_isolation + manual `version` run |
| `bundled_root` | extension sync / agent init | `grok_home().join(bundled)` | ✓ WIRED | bundle.rs + product-home tests |
| `SubagentsConfig::resolve` | product home roles/personas | `grok_home` not `home_dir/.grok` | ✓ WIRED | config/mod.rs discover under product_home |
| Twin `resolve_grok_home` | worktrees.db | open_default | ✓ WIRED | BUM_HOME + `.bum` |
| `swap_managed_bin_links` | leader managed name / `grok_application` | product home `bin/bum` | ✓ WIRED | Both use leaf `bum` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `grok_home()` | product home PathBuf | `var_os("BUM_HOME")` or `home_dir()/.bum` | Yes — env or default path | ✓ FLOWING |
| Hermetic `bum version` | filesystem under BUM_HOME | process startup (docs extract, active_sessions) | Yes — non-empty tree observed | ✓ FLOWING |
| Session persistence | `sessions_root` | `grok_home().join("sessions")` | Path derived from product home SoT | ✓ FLOWING |
| Auth manager | auth store path | `AuthManager::new(&grok_home, …)` | Path under product home | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Build ship binary | `cargo build -p xai-grok-pager-bin --bin bum` | Finished ok; `target/debug/bum` present | ✓ PASS |
| Old bin gone | `cargo build -p xai-grok-pager-bin --bin xai-grok-pager` | `error: no bin target named xai-grok-pager`; available: `bum` | ✓ PASS |
| Paths unit tests | `cargo test -p xai-grok-config --lib paths` | 19 passed | ✓ PASS |
| Hermetic isolation | `cargo test -p xai-grok-pager-bin --test home_isolation` | `hermetic_temp_home_writes_only_under_bum_home` ok | ✓ PASS |
| Ignore GROK_HOME | `cargo test -p xai-grok-pager --test grok_home_ignore_grok_home` | ok | ✓ PASS |
| BUM_HOME helpers | `cargo test -p xai-grok-pager --test grok_home_paths` | ok | ✓ PASS |
| Bundled root isolation | `cargo test -p xai-grok-shell --test test_bundled_root_product_home` | 2 passed | ✓ PASS |
| Agent discovery | `cargo test -p xai-grok-agent discovery` | stock-home exclusion tests ok | ✓ PASS |
| Display labels | `cargo test -p xai-grok-pager-render --lib util` | `display_grok_home_prefix_default_install` ok | ✓ PASS |
| Live hermetic invoke | `env -i HOME=… BUM_HOME=… bum version` | exit 0; writes only under `.bum`; no `.grok` | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| _(none declared)_ | — | No `scripts/**/probe-*.sh` for this phase | SKIP |

### Static Gates (plan 01-05)

| Gate | Check | Result |
| ---- | ----- | ------ |
| A | No production `var(_os)?("GROK_HOME")` in config/fast-worktree/shell-base/voice/shell | Clean (no matches) |
| B | No `join(".grok")` in bundle.rs / paths.rs / fast-worktree db | Clean |
| C | No `$GROK_HOME` / `~/.grok` in hub_auth, mcp credentials, hooks trust, mcp_doctor | Clean |
| D | Those files teach `BUM_HOME` / `~/.bum` | Hits present |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| ID-01 | 01-02, 01-03, 01-05 | Launch product as `bum` CLI binary | ✓ SATISFIED | Sole ship bin `bum`; managed leaf `bum`; builds/runs |
| ID-03 | 01-01, 01-02, 01-04, 01-05 | State under isolated `~/.bum` / `BUM_HOME` | ✓ SATISFIED | SoT cutover + hermetic isolation + harness sandboxes |

No orphaned phase-1 requirements beyond ID-01 / ID-03.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TBD/FIXME/XXX in key phase artifacts | — | — |
| Shell residual comments | various | Comment-level GROK_HOME / `~/.grok` wording | ℹ️ Info | Deferred to Phase 8 rebrand; not product-root writers |
| `bum version` output | runtime | Still prefixes brand as `grok` | ℹ️ Info | Deferred Phase 8 chrome |

### Human Verification Required

_None for phase goal achievement._ Automated hermetic isolation + binary ship name cover the three roadmap success criteria. Optional Phase 8 polish (full chrome) is out of scope.

### Gaps Summary

No actionable gaps. Phase goal is achieved in the codebase:

1. **Ship name** is `bum`.
2. **Product home SoT** is `BUM_HOME` / `~/.bum`; writers hang off `grok_home()`.
3. **Hermetic temp-home** runs prove zero product state under `.grok` / `.codex`.

Deferred polish (version chrome, residual comment strings, pre-existing shell lib-test compile) does not block Phase 1 success criteria.

### MVP note

ROADMAP goal text is not formal user-story shape (`user-story.validate` → invalid). Verification proceeded against explicit Success Criteria + plan user stories. If the project requires formal MVP UAT scripts keyed to a single goal string, reformat via `/gsd mvp-phase 1`.

---

_Verified: 2026-07-16T03:53:32Z_  
_Verifier: Claude (gsd-verifier)_
