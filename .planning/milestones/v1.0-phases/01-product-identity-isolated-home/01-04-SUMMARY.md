---
phase: 01-product-identity-isolated-home
plan: 04
subsystem: testing
tags: [BUM_HOME, .bum, test-fixtures, isolation, OnceLock, PTY, workspace, D-HOME, D-VERIFY, ID-03]

requires:
  - phase: 01-product-identity-isolated-home
    provides: "Production resolve_product_home / grok_home use BUM_HOME only (01-01); twin/managed bin (01-02); bin bum (01-03)"
provides:
  - "Shared test sandboxes set BUM_HOME (+ HOME) with product tree under .bum"
  - "PTY env_for_pager / flows / leader / scripted product roots on BUM_HOME/.bum"
  - "Shell + pager-local + workspace product-home fixtures sandboxed via BUM_HOME"
  - "session_list bench product-home setter uses BUM_HOME"
affects:
  - 01-product-identity-isolated-home
  - 01-05 isolation proof
  - all product-home integration tests

tech-stack:
  added: []
  patterns:
    - "Product-home sandbox: HOME=tmp, BUM_HOME=home.join(\".bum\") (or BUM_HOME=temp product root for in-process OnceLock fixtures)"
    - "Project-local layout stays .grok (cwd skills/config); only product home is .bum / BUM_HOME"
    - "OnceLock: set BUM_HOME before first grok_home(); serial/LockedTestEnv for env mutation"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-test-support/src/env.rs
    - crates/codegen/xai-grok-test-support/src/leader.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/content.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/flows.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/leader.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/scripted.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs
    - crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs
    - crates/codegen/xai-grok-pager/src/app/session_startup.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs
    - crates/codegen/xai-grok-shell/tests
    - crates/codegen/xai-grok-shell/benches/session_list.rs
    - crates/codegen/xai-grok-shell/src/agent/folder_trust.rs
    - crates/codegen/xai-grok-workspace/src/folder_trust.rs
    - crates/codegen/xai-grok-workspace/src/permission/resolution.rs
    - crates/codegen/xai-grok-workspace/src/trust.rs
    - crates/codegen/xai-grok-update/tests/common/mod.rs

key-decisions:
  - "Product-home test env key is BUM_HOME only — no dual-read of GROK_HOME for sandboxing"
  - "PTY product_home() helper centralizes content.home()/.bum to reduce path drift"
  - "Project-local skill/config dirs under workspace .grok intentionally not renamed this plan"

patterns-established:
  - "Child process: test_env_cmd_tokio / env_for_pager set BUM_HOME=home/.bum + HOME=home"
  - "In-process unit fixture: EnvGuard/EnvVarGuard/LockedTestEnv set BUM_HOME to temp product root before first resolve"
  - "Leader lock/log/socket helpers derive from home/.bum when HOME is the sandbox root"

requirements-completed: [ID-03]

coverage:
  - id: D1
    description: "test-support env + leader harness (socket, lock, log) sandbox via BUM_HOME/.bum"
    requirement: ID-03
    verification:
      - kind: other
        ref: "cargo check -p xai-grok-test-support"
        status: pass
      - kind: other
        ref: "! rg join(\".grok\") in test-support env.rs + leader.rs"
        status: pass
    human_judgment: false
  - id: D2
    description: "PTY harness content/flows/leader/scripted product home is BUM_HOME/.bum with env_for_pager_shape green"
    requirement: ID-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-pty-harness env_for_pager"
        status: pass
      - kind: other
        ref: "! rg join(\".grok\")|GROK_HOME in content/flows/leader/scripted"
        status: pass
    human_judgment: false
  - id: D3
    description: "Shell/pager/workspace/update/bench product-home fixtures use BUM_HOME; behavioral suites green where compilable"
    requirement: ID-03
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test test_config_update_isolation"
        status: pass
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test test_mcp_permission_persistence"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-workspace --lib folder_trust"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-workspace --lib trust"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-workspace --lib permission::resolution::tests::load_claude_env"
        status: pass
      - kind: other
        ref: "cargo check -p xai-grok-update -p xai-grok-pager"
        status: pass
      - kind: other
        ref: "cargo test -p xai-grok-shell leader (lib) — pre-existing compile break; deferred"
        status: unknown
    human_judgment: false

duration: 16min
completed: 2026-07-16
status: complete
---

# Phase 1 Plan 04: Product-home test fixture cutover Summary

**All inventoried product-home test sandboxes now set `BUM_HOME` / write under `.bum`, matching production isolation so suites no longer re-validate `GROK_HOME` coupling**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-16T03:08:28Z
- **Completed:** 2026-07-16T03:24:09Z
- **Tasks:** 3
- **Files modified:** 61

## Accomplishments

- Shared test-support: `test_env_cmd_tokio` + leader spawn/socket/lock/log use `BUM_HOME` and `home/.bum`
- Full PTY cutover: `env_for_pager`, OAuth seed, cluster sessions/socket, scripted config, plan_approval sessions root
- Shell integration tests, pager-local leader_cluster/session/lifecycle, workspace folder_trust/trust/permission helpers, and `session_list` bench sandboxed via `BUM_HOME`

## Task Commits

Each task was committed atomically:

1. **Task 1: test-support env + leader harness** - `68407cc` (feat)
2. **Task 2: Full PTY product-home cutover** - `626c043` (feat)
3. **Task 3: Update + shell + pager + bench + workspace inventory** - `33c0947` (feat)

**Plan metadata:** `191a77f` (docs: complete plan)

## Files Created/Modified

### Task 1
- `crates/codegen/xai-grok-test-support/src/env.rs` — `BUM_HOME=home/.bum`
- `crates/codegen/xai-grok-test-support/src/leader.rs` — spawn env, socket, lock, log under `.bum`

### Task 2
- `crates/codegen/xai-grok-pager-pty-harness/src/content.rs` — `product_home()`, `BUM_HOME` in env + unit assert
- `flows.rs` / `leader.rs` / `scripted.rs` / `scenarios/plan_approval_resume.rs` — product tree `.bum`

### Task 3 (inventory)
- Shell `tests/*` product-home setters and `.bum` path roots; `benches/session_list.rs`
- Shell unit fixtures: `folder_trust.rs`, `mvp_agent/tests.rs`, `config/tests.rs`
- Pager: `leader_cluster`, `session_startup`, `lifecycle`, effects/cta/acp fixtures; `tests/pty_e2e/*` product paths
- Workspace: `folder_trust`, `permission/resolution`, `trust` helpers (worktree already on `BUM_HOME`)
- Update: `tests/common/mod.rs` already on `BUM_HOME` (comment polish)

## Decisions Made

- **No dual-read of `GROK_HOME`** for test convenience — production ignores it; tests must set `BUM_HOME` before first resolve
- **`product_home()` on ContentController** for PTY path construction consistency
- **Leave project-local `.grok`** (workspace skills/hooks/config) unchanged — not product home
- **Isolation-proof fixture** `pager/tests/grok_home_ignore_grok_home.rs` still *sets* the ignored `GROK_HOME` trap intentionally (out of plan negative-grep scope)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] PTY scenario residual product sessions path**
- **Found during:** Task 2 inventory
- **Issue:** `plan_approval_resume.rs` still joined `home/.grok/sessions` after content env moved to `.bum`
- **Fix:** Sessions root → `home/.bum/sessions`
- **Files modified:** `crates/codegen/xai-grok-pager-pty-harness/src/scenarios/plan_approval_resume.rs`
- **Committed in:** `626c043`

**2. [Rule 2 - Missing critical functionality] Broader shell/pager product-home inventory**
- **Found during:** Task 3 greps
- **Issue:** Plan listed representative files; inventory found additional shell integration tests, shell unit fixtures, pager effects/cta/acp tests, and full `pty_e2e` product path joins still on `GROK_HOME`/`.grok`
- **Fix:** Bulk cutover of product-home env setters and product path segments; preserved project-local `.grok` for workspace skills
- **Files modified:** 50+ under shell/pager/workspace (see commit `33c0947`)
- **Committed in:** `33c0947`

### Deferred Issues

**1. Pre-existing: `xai-grok-shell` lib tests do not compile** (`cargo test -p xai-grok-shell leader`)
- Documented in `deferred-items.md` (also found in 01-02)
- Verified shell product-home via integration tests instead

**2. Pre-existing: some `permission::resolution` tests merge host `~/.claude`**
- Fail without `HOME` isolation when developer has Claude settings
- Product-home-isolated `load_claude_env_*` tests pass under `BUM_HOME`

## Known Stubs

None.

## Threat Flags

None new — test sandbox boundary only; T-01-07 mitigated by complete BUM_HOME fixture inventory.

## Self-Check: PASSED

- [x] SUMMARY path exists
- [x] Commits `68407cc`, `626c043`, `33c0947` present on branch
- [x] Key artifacts contain `BUM_HOME` (test-support, pty content, leader_cluster, workspace folder_trust/trust)
- [x] Plan negative greps clean for listed product-home setter paths
