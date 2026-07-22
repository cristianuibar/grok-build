---
phase: 01
title: Product identity & isolated home
reviewers: [codex]
cycle: 1
status: incorporated
date: 2026-07-16
incorporated: 2026-07-16
---

# Phase 1 Plan Reviews

Cross-AI plan review (convergence cycle 1). Reviewer: **Codex** (`gpt-5.6-sol` / high, read-only, source-grounded).

## Synthesis

| Severity | Count | Themes |
|----------|-------|--------|
| HIGH | 10+ | Bundle writer bypasses SoT; updater installs `bin/grok`; incomplete test fixture cutover; narrow isolation proof |
| MEDIUM | several | OnceLock multi-scenario tests; Windows HOME; twin resolver platform mismatch; semicolon verify masks |
| LOW | few | lib.rs docs; grep scope docs |

**Overall risk (Codex):** HIGH until gaps closed.

## Actionable unresolved (for replan)

### HIGH (must address in PLAN.md)

1. **Bundle writer bypass** — `xai-grok-shell/src/bundle.rs` `bundled_root()` writes `$HOME/.grok/bundled` directly; reachable from extension sync / agent init. Cut over to `grok_home()` / bum home.
2. **Updater managed bin** — `xai-grok-update/src/auto_update.rs` `swap_managed_bin_links()` hard-codes `bin/grok`; conflicts with leader `bin/bum` and D-BIN no-alias. Change leaf to `bum` + tests.
3. **OnceLock multi-scenario tests (01-01)** — Cannot run two env-sensitive scenarios in one process; need separate binaries or pure resolver helper.
4. **PTY / leader fixture incomplete cutover (01-04)** — `flows.rs`, `leader.rs`, `scripted.rs`, leader lock/log paths still `.grok`.
5. **Shell tests still set GROK_HOME** — After production ignores GROK_HOME, many shell tests write/read wrong home.
6. **Role/persona discovery** — still reads `$HOME/.grok/roles|personas` via `dirs::home_dir()`.
7. **Grep gate too narrow** — must include `xai-grok-shell` production sources.
8. **Isolation test weak** — hermetic HOME+BUM_HOME+GROK_HOME trap+CODEX_HOME trap; recursive absent/unchanged under `.grok`/`.codex`; not filename list only; supplement version with bundle/updater coverage as needed.

### MEDIUM (incorporate or explicit defer)

- Windows: do not rely on HOME alone for default-home proof.
- Twin resolver should align home_dir vs HOME env semantics (or document/share SoT).
- Leader tests hard-code `bin/grok` — update explicitly.
- Verify commands use `&&` not `;`.
- Auth manager diagnostic reads GROK_HOME (stale product-home input).
- Hermetic isolation test must unset/redirect ambient GROK_HOME/CODEX_HOME.

### LOW (optional)

- Make paths/lib.rs docs mandatory.
- Repo-wide classify old binary name refs (runtime fix now, branding Phase 8).

---

## Codex review (full)

> Source: `/tmp/gsd-review-grok-build-1557405645-codex-1.md` (2026-07-16)

# Cross-AI Plan Review

## Overall assessment

The plans identify the principal home resolver and binary manifest correctly, but the phase is not yet complete enough to guarantee isolation. Two production paths remain outside the proposed inventory:

- Bundle synchronization writes directly beneath `~/.grok/bundled`, bypassing `grok_home()`.
- The live updater independently installs `bin/grok`, conflicting with the proposed `bin/bum` managed-binary contract.

The test migration and final grep gate are also too narrow to catch these paths. Overall phase risk is **HIGH** until those gaps are added.

---

## 01-01 — SoT home cutover and display labels

### Summary

This is a sound foundation: the plan targets the actual central resolver, changes the correct environment variable and default directory, and updates the renderer’s old path labels. Its main weakness is test design around the process-global `OnceLock`, especially the proposed second environment scenario and reliance on `HOME` for default-home behavior.

### Strengths

- The correct source of truth is targeted. Today `default_grok_home()` selects `.grok`, while `grok_home()` reads `GROK_HOME` and creates the resulting directory: `crates/codegen/xai-grok-config/src/paths.rs:14-47`.
- `user_grok_home()` separately probes `GROK_HOME`, so the plan correctly includes it: `crates/codegen/xai-grok-config/src/paths.rs:49-58`.
- The managed executable path really is independently hard-coded as `bin/grok`: `crates/codegen/xai-grok-config/src/paths.rs:60-64`.
- Most session persistence flows through this resolver, supporting the plan’s SoT rationale: `crates/codegen/xai-grok-config/src/paths.rs:149-181`.
- The display helper contains exactly the stale labels the plan proposes changing: `crates/codegen/xai-grok-pager-render/src/util.rs:7-24`.

### Concerns

- **HIGH — The proposed two environment scenarios cannot safely share one integration-test process.** `grok_home()` caches the first result in a process-global `OnceLock`: `crates/codegen/xai-grok-config/src/paths.rs:34-47`. The existing `grok_home_paths.rs` is one test binary with one environment-sensitive test: `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs:1-37`. Adding another test in the same file will not isolate the resolver.
- **MEDIUM — Setting only `HOME` is not portable proof of the default.** The resolver uses `std::env::home_dir()`, not `HOME` directly: `crates/codegen/xai-grok-config/src/paths.rs:25-30`. Existing test-support comments explicitly note that changing `HOME` alone is insufficient on Windows: `crates/codegen/xai-grok-test-support/src/env.rs:158-166`. The ignored-`GROK_HOME` test could therefore resolve or create the real user’s `.bum`.
- **MEDIUM — The in-module unit-test scope is insufficient for environment semantics.** Current path tests only verify the default suffix and canonicalization: `crates/codegen/xai-grok-config/src/paths.rs:303-311`. Mutating environment variables around a global `OnceLock` needs process isolation or a pure, injected resolver.
- **LOW — `lib.rs` should be mandatory rather than optional.** The crate-level contract still documents `$GROK_HOME`: `crates/codegen/xai-grok-config/src/lib.rs:1-10`.

### Suggestions

1. Extract a pure internal resolver such as `resolve_product_home(bum_home, user_home)` and unit-test all precedence rules without process-global environment mutation.
2. Keep one integration test per executable for `BUM_HOME` override and ignored `GROK_HOME`, or have one parent test spawn child modes.
3. Do not use `HOME` alone to sandbox the default resolver on Windows. Either inject the home path into the pure helper or set all platform-relevant home variables in a child process.
4. Make all product-home documentation changes in `paths.rs` and `lib.rs` explicit completion criteria.

### Risk Assessment

**MEDIUM.** The production edits are well targeted, but the planned tests could be nondeterministic, fail to test the intended branch, or touch a developer’s real home.

---

## 01-02 — Twin resolver, workspace fallback, and managed binary

### Summary

The plan correctly finds the alternate fast-worktree resolver, workspace fallback, sandbox re-export, and leader binary helper. However, it misses a second live implementation of managed binary installation in the updater. As written, the leader will look for `bin/bum` while the updater continues creating `bin/grok`.

### Strengths

- The duplicate fast-worktree resolver does read `GROK_HOME` and default to `.grok`: `crates/codegen/xai-fast-worktree/src/db/mod.rs:340-350`.
- Its test fixture also manages `GROK_HOME`, so fixture migration is necessary: `crates/codegen/xai-fast-worktree/src/db/mod.rs:352-407`.
- The workspace fallback independently defaults to `.grok`: `crates/codegen/xai-grok-workspace/src/worktree/mod.rs:621-640`.
- The leader’s managed binary helper currently returns `grok`/`grok.exe`: `crates/codegen/xai-grok-shell/src/leader/mod.rs:1336-1366`.
- The sandbox implementation correctly delegates to the central resolver; only its documentation needs rebranding: `crates/codegen/xai-grok-sandbox/src/paths.rs:11-16`.

### Concerns

- **HIGH — The live updater independently installs `bin/grok`.** `swap_managed_bin_links()` hard-codes a `grok` leaf and creates that link: `crates/codegen/xai-grok-update/src/auto_update.rs:1341-1349`. Activation invokes this path under the managed product home: `crates/codegen/xai-grok-update/src/auto_update.rs:1224-1239`. After this plan, the leader expects `bin/bum`, but the updater still provisions `bin/grok`.
- **HIGH — This conflicts with the “no compatibility alias” decision.** Updating only `grok_application()` and `managed_grok_bin_name()` leaves a live stock-named executable behind.
- **MEDIUM — Leader tests are not all helper-driven.** Several tests hard-code `bin/grok`, including the managed-install preference case: `crates/codegen/xai-grok-shell/src/leader/mod.rs:2111`, `:2130`, and `:2141-2147`. The claim that they should remain green via the helper is inaccurate.
- **MEDIUM — The two resolvers are not actually in lockstep across platforms.** Config uses `std::env::home_dir()`: `crates/codegen/xai-grok-config/src/paths.rs:25-30`. Fast-worktree requires the literal `HOME` environment variable: `crates/codegen/xai-fast-worktree/src/db/mod.rs:340-350`.
- **MEDIUM — Verification masks failures.** `cargo test ...; cargo check ...` returns the status of the final command, so a failed leader test can be hidden by a successful check.

### Suggestions

1. Add `xai-grok-update/src/auto_update.rs` and its managed-install tests to this plan. Change the installed leaf to `bum` consistently.
2. Update the update integration tests that currently assert `bin/grok`, such as `crates/codegen/xai-grok-update/tests/test_install_internal.rs:99-224` and `test_concurrent_convergence.rs:48-96`.
3. Explicitly update all hard-coded leader test paths.
4. Prefer sharing a resolver implementation with `xai-grok-config`, or make the twin consume the same injected environment/home inputs.
5. Replace semicolons in verification with `&&`.

### Risk Assessment

**HIGH.** The current plan creates a split-brain managed installation: one component installs `grok`, while another searches for `bum`.

---

## 01-03 — Ship the binary as `bum`

### Summary

This is the strongest of the five plans. It targets the exact Cargo binary declaration and the known test/PTY resolvers. It should achieve the local Cargo build/run requirement, provided the updater’s independent managed-binary behavior is corrected elsewhere.

### Strengths

- The composition-root manifest is the correct shipping surface. It currently defines both `default-run` and `[[bin]]` as `xai-grok-pager`: `crates/codegen/xai-grok-pager-bin/Cargo.toml:1-16`.
- The package name can remain unchanged while the binary target becomes `bum`.
- Test support currently contains all three stale resolution paths the plan identifies:
  - local debug artifact: `crates/codegen/xai-grok-test-support/src/env.rs:51-70`
  - incorrect Cargo build package/bin arguments: `crates/codegen/xai-grok-test-support/src/env.rs:78-82`
  - `CARGO_BIN_EXE_xai-grok-pager`: `crates/codegen/xai-grok-test-support/src/env.rs:98-114`
- The PTY harness similarly hard-codes the old local artifact, Cargo target, and Cargo-provided environment variable: `crates/codegen/xai-grok-pager-pty-harness/src/env.rs:26-46` and `:71-99`.
- The scenario help strings named by the plan are real: `crates/codegen/xai-grok-pager-pty-harness/src/bin/pty_scenario.rs:23-24` and `scroll_matrix.rs:56-57`.

### Concerns

- **MEDIUM — ID-01 remains incomplete unless the updater is changed.** Cargo will produce `bum`, but the managed updater still creates `bin/grok`: `crates/codegen/xai-grok-update/src/auto_update.rs:1341-1349`.
- **LOW — The grep scope is limited to three crates.** Other stale references may be harmless documentation, but runtime binary resolvers should be inventoried repo-wide before declaring the single-bin cutover complete.
- **LOW — The later integration test should prefer Cargo’s compile-time binary path.** In a package integration test, `env!("CARGO_BIN_EXE_bum")` is more deterministic than a runtime lookup with a fallback build.

### Suggestions

1. Keep this plan’s manifest and harness scope.
2. Add a repo-wide classification of old binary references: runtime resolution must be fixed now; branding/docs can remain deferred.
3. Couple completion of ID-01 to the updater correction in Plan 01-02.
4. Verify both:
   - `cargo build -p xai-grok-pager-bin --bin bum`
   - absence of the old binary target through Cargo metadata or an expected failure for `--bin xai-grok-pager`.

### Risk Assessment

**MEDIUM.** The Cargo and harness changes are low-risk, but the phase-level command identity remains inconsistent because the managed updater is omitted.

---

## 01-04 — Test sandbox cutover

### Summary

The plan recognizes that shared fixtures must move to `BUM_HOME`, but its inventory is materially incomplete. Several PTY and leader helpers construct `.grok` paths directly, and many shell tests will silently stop being sandboxed once production ignores `GROK_HOME`. `cargo check` cannot detect those behavioral regressions.

### Strengths

- The shared test command currently sets `HOME` and `GROK_HOME=$HOME/.grok`, exactly as described: `crates/codegen/xai-grok-test-support/src/env.rs:152-176`.
- Leader test support independently injects `GROK_HOME`: `crates/codegen/xai-grok-test-support/src/leader.rs:128-153`.
- PTY environment construction uses `.grok` and `GROK_HOME`: `crates/codegen/xai-grok-pager-pty-harness/src/content.rs:69-92`.
- Establishing the consistent `HOME=<tmp>` plus `BUM_HOME=<tmp>/.bum` convention is a good basis for later negative assertions.

### Concerns

- **HIGH — PTY setup remains split between `.bum` and `.grok`.** Even after changing `env_for_pager`, other helpers continue seeding or inspecting the old tree:
  - OAuth fixture: `crates/codegen/xai-grok-pager-pty-harness/src/flows.rs:81-91`
  - leader socket and sessions: `crates/codegen/xai-grok-pager-pty-harness/src/leader.rs:31-42` and `:82-85`
  - generated config: `crates/codegen/xai-grok-pager-pty-harness/src/scripted.rs:538-543`
- **HIGH — Leader test support contains additional direct `.grok` paths.** Lock and log helpers use `.grok` independently of the environment variable: `crates/codegen/xai-grok-test-support/src/leader.rs:286-287` and `:352-353`.
- **HIGH — Numerous shell tests will no longer be sandboxed.** Examples include:
  - `crates/codegen/xai-grok-shell/tests/signed_managed_config/common.rs:34`
  - `crates/codegen/xai-grok-shell/tests/test_built_binary_e2e.rs:1298-1302`
  - `crates/codegen/xai-grok-shell/tests/test_debug_logging.rs:92`
  - `crates/codegen/xai-grok-shell/tests/test_mcp_permission_persistence.rs:34`
  
  Once production ignores `GROK_HOME`, these tests can fall through to the real/default `.bum`.
- **HIGH — `cargo check` does not exercise fixture behavior.** It will not reveal that a fixture writes auth/config to `.grok` while the spawned binary reads `.bum`.
- **MEDIUM — “Fix enough” is not a verifiable completion criterion.** Every fixture intended to control product home must move, even if unrelated comments remain deferred.

### Suggestions

1. Inventory executable `GROK_HOME` setters across all tests and fixtures, not just the listed files.
2. Add the PTY `flows.rs`, `leader.rs`, and `scripted.rs` paths to the plan.
3. Explicitly update leader lock, socket, log, session, auth, and config locations to derive from the configured product root.
4. Run representative behavioral suites:
   - test-support leader tests
   - PTY environment and scripted-flow tests
   - shell auth/config/leader integration tests
5. Add a test-fixture guard that rejects `GROK_HOME` as a product-home override and confirms `BUM_HOME` is present before spawning the product.

### Risk Assessment

**HIGH.** Partial fixture migration can both break CI and allow tests to read or write a developer’s actual `.bum` directory.

---

## 01-05 — Legacy read gate and isolation proof

### Summary

The plan includes useful closing work: removing legacy agent discovery, fixing two direct `GROK_HOME` bypasses, and running a real binary under a temporary home. Nevertheless, it does not discover all production stock-home access. Most importantly, authenticated bundle synchronization can still write `~/.grok/bundled`, and role/persona discovery still reads from `$HOME/.grok`. The proposed `--version` test and narrow grep gate will not catch either behavior.

### Strengths

- The legacy agent scan is real. `user_agent_dirs()` explicitly appends `HOME/.grok/agents` and bundled agent directories whenever the configured home differs: `crates/codegen/xai-grok-agent/src/discovery.rs:183-223`.
- Project-local `.grok` paths can remain separate from that legacy user-home branch, matching the phase boundary.
- `ChangelogManager::from_env_home()` directly honors `GROK_HOME`: `crates/codegen/xai-grok-shell-base/src/util/changelog.rs:64-85`. Later code writes changelog data beneath that root: `crates/codegen/xai-grok-shell-base/src/util/changelog.rs:151-164`.
- `voice_probe` also independently reads `GROK_HOME` and falls back to `$HOME/.grok/config.toml`: `crates/codegen/xai-grok-voice/src/bin/voice_probe.rs:118-140`.
- A `--version` run is useful because product-home initialization happens before CLI parsing:
  - memory/debug root: `crates/codegen/xai-grok-pager-bin/src/main.rs:1456-1472`
  - requirements/docs/sentry initialization: `crates/codegen/xai-grok-pager-bin/src/main.rs:1474-1489`
  - crash and active-session paths: `crates/codegen/xai-grok-pager-bin/src/main.rs:1491-1510`
  - argument parsing occurs later: `crates/codegen/xai-grok-pager-bin/src/main.rs:1530-1533`

### Concerns

- **HIGH — A production writer still bypasses the resolver and targets `~/.grok`.** `bundled_root()` directly returns `$HOME/.grok/bundled`: `crates/codegen/xai-grok-shell/src/bundle.rs:86-91`. The module then creates directories and writes bundle files/manifests: `crates/codegen/xai-grok-shell/src/bundle.rs:109-150`.
- **HIGH — That writer is reachable in normal product operation.** Extension synchronization obtains this root and extracts fetched bundles: `crates/codegen/xai-grok-shell/src/extensions/bundle.rs:124-136` and `:204-224`. Agent initialization schedules the synchronization: `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs:2160-2172`.
- **HIGH — Stock-home read gating is incomplete.** Role/persona discovery searches project `.grok`, but also calls the same discovery against `dirs::home_dir()`, thereby reading `$HOME/.grok/roles` and `$HOME/.grok/personas`: `crates/codegen/xai-grok-shell/src/config/mod.rs:385-392`, `:426-435`, and `:463-473`.
- **MEDIUM — Another live `GROK_HOME` read is omitted.** Auth diagnostics read and log whether `GROK_HOME` is configured: `crates/codegen/xai-grok-shell/src/auth/manager.rs:267-278`. Even though it does not select the auth path, it is a stale product-home input and is outside the proposed grep scope.
- **HIGH — The grep gate is too narrow.** It scans config, fast-worktree, shell-base, and voice, but not `xai-grok-shell`, where the bundle writer, role/persona reads, and auth diagnostic live.
- **HIGH — A version-only test cannot prove all product writers are isolated.** It exercises early startup, but not authenticated extension bundle synchronization or managed updates.
- **MEDIUM — “Does not set GROK_HOME” is not hermetic.** The child can inherit a developer’s ambient `GROK_HOME` or `CODEX_HOME`. The test should explicitly remove or redirect both.
- **MEDIUM — Checking only known product filenames is weaker than the success criterion.** An unexpected new file beneath `.grok` would evade the test.
- **LOW — Current Codex foreign-session behavior appears read-only.** It selects `CODEX_HOME` or `$HOME/.codex` for scanning: `crates/codegen/xai-grok-workspace/src/foreign_sessions/codex/mod.rs:18-39`. The negative-write assertion is still worthwhile.

### Suggestions

1. Add `xai-grok-shell/src/bundle.rs` to the phase and derive its root from the central bum home.
2. Update bundle tests and authenticated synchronization tests to use `BUM_HOME`.
3. Change user role/persona discovery to use the resolved bum home rather than `dirs::home_dir()`. Preserve only explicit project-local `cwd/.grok` discovery.
4. Change or remove the auth manager’s `GROK_HOME` diagnostic.
5. Broaden the static gate to all production Rust sources, then allowlist only clearly classified project-local `.grok` paths and test-only stock-home assertions.
6. Make the integration test hermetic:
   - `HOME=<tmp>`
   - `BUM_HOME=<tmp>/.bum`
   - `GROK_HOME=<tmp>/.grok` as a trap, or explicitly remove it
   - `CODEX_HOME=<tmp>/.codex` as a trap
   - disable network paths
7. Assert recursively that `.grok` and `.codex` are absent or byte-for-byte unchanged, not merely free of a known filename list.
8. Supplement `--version` with direct tests for bundle synchronization and managed updater installation.

### Risk Assessment

**HIGH.** The plan’s final proof can pass while a normal authenticated session still writes directly to `~/.grok/bundled`.

---

## Recommended dependency and scope correction

Before execution, revise the phase ordering to:

1. Central home resolver and binary manifest.
2. Every alternate resolver and direct stock-home path, including:
   - fast-worktree
   - workspace fallback
   - bundle cache
   - role/persona discovery
   - changelog and voice
   - updater-managed binary installation
3. Complete test-fixture migration.
4. Direct tests for each exceptional writer.
5. Hermetic binary isolation test plus a repo-wide classified static gate.

With those changes, the phase would have a credible path to ID-01 and ID-03. In its current form, the overall risk remains **HIGH**.## Prior HIGH Reassessment

1. **RESOLVED — bundle writer uses product home.** Current `bundled_root()` bypasses the resolver via `HOME/.grok` at [bundle.rs:86](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/bundle.rs:86). Plan 01-05 explicitly replaces it with `grok_home().join(...)` and updates tests at [01-05-PLAN.md:101](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:101) and [01-05-PLAN.md:108](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:108).

2. **RESOLVED — updater provisions `bin/bum`.** The live function currently selects `grok` at [auto_update.rs:1341](/home/cristian/bum/grok-build/crates/codegen/xai-grok-update/src/auto_update.rs:1341). Plan 01-02 requires `bum`/`bum.exe`, no compatibility alias, and relevant test updates at [01-02-PLAN.md:137](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-02-PLAN.md:137) through [01-02-PLAN.md:145](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-02-PLAN.md:145).

3. **RESOLVED — OnceLock scenarios are isolated.** The production resolver is process-cached at [paths.rs:6](/home/cristian/bum/grok-build/crates/codegen/xai-grok-config/src/paths.rs:6) and [paths.rs:35](/home/cristian/bum/grok-build/crates/codegen/xai-grok-config/src/paths.rs:35). Plan 01-01 now specifies a pure resolver at [01-01-PLAN.md:77](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-01-PLAN.md:77), plus one environment scenario per integration-test process at [01-01-PLAN.md:106](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-01-PLAN.md:106) through [01-01-PLAN.md:124](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-01-PLAN.md:124).

4. **PARTIAL — PTY/test-support and shell inventories are covered, but pager-local fixtures are omitted.** Plan 01-04 covers test-support, the external PTY harness, update tests, and shell tests at [01-04-PLAN.md:10](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-04-PLAN.md:10) through [01-04-PLAN.md:19](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-04-PLAN.md:19). However, the pager’s in-process leader cluster still sets `GROK_HOME` at [leader_cluster/mod.rs:308](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs:308) and [leader_cluster/mod.rs:319](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/leader_cluster/mod.rs:319). Other pager tests perform real product-home writes after setting only `GROK_HOME`, including [session_startup.rs:1025](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/session_startup.rs:1025) and [lifecycle.rs:927](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs:927). Once production ignores `GROK_HOME`, these can target the developer’s real `.bum`.

5. **RESOLVED — roles/personas use product home.** The current user-global branch passes the OS home into helpers that append `.grok` at [config/mod.rs:467](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/config/mod.rs:467). Plan 01-05 explicitly replaces that branch with `grok_home()/roles` and `/personas` while retaining project-local discovery at [01-05-PLAN.md:100](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:100) and [01-05-PLAN.md:107](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:107).

6. **RESOLVED — production grep includes `xai-grok-shell`.** The revised action and automated gate explicitly include shell production sources at [01-05-PLAN.md:159](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:159) and [01-05-PLAN.md:164](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:164). It will catch the current auth diagnostic read at [auth/manager.rs:274](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:274).

7. **RESOLVED — hermetic isolation proof.** Plan 01-05 now requires explicit `HOME`, `BUM_HOME`, trapped/removed `GROK_HOME` and `CODEX_HOME`, recursive trap comparison, and network-feature suppression at [01-05-PLAN.md:124](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:124) through [01-05-PLAN.md:140](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:140). The chosen `--version` path can exercise home creation because main initializes `grok_home()` before command dispatch at [main.rs:1470](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager-bin/src/main.rs:1470).

## New Findings

Two HIGH concerns remain:

- The pager-local fixtures described under item 4 are outside 01-04’s file and grep inventory.
- The fast-worktree twin is not actually specified in lockstep with the new pure resolver. Config treats empty `BUM_HOME` as absent and accepts `OsString` at [01-01-PLAN.md:77](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-01-PLAN.md:77) through [01-01-PLAN.md:80](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-01-PLAN.md:80), but 01-02 prescribes a literal `var("BUM_HOME")` substitution at [01-02-PLAN.md:88](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-02-PLAN.md:88). The current twin’s corresponding shape is visible at [db/mod.rs:340](/home/cristian/bum/grok-build/crates/codegen/xai-fast-worktree/src/db/mod.rs:340). Empty `BUM_HOME` would resolve to an empty path, while non-Unicode paths would be ignored, splitting worktree state from the config resolver.

CYCLE_SUMMARY: current_high=2 current_actionable=5

## Current HIGH Concerns

- **Incomplete fixture cutover:** add pager-local leader/session/trust/effects fixtures—and the remaining shell benchmark setter at [session_list.rs:544](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/benches/session_list.rs:544)—to 01-04’s files, inventory, and verification. Tests must not rely on ignored `GROK_HOME` or a previously pinned `OnceLock`.
- **Fast-worktree resolver is not semantically lockstep:** require `var_os("BUM_HOME")`, reject empty values exactly like `resolve_product_home`, and add empty/non-Unicode coverage where supported.

## Current Actionable Non-HIGH Concerns

- **MEDIUM — bundle tests repeat the OnceLock problem.** Plan 01-05 says to use temporary `BUM_HOME` but omits `extensions/bundle.rs` from its file list. Its current per-test HOME guard is at [extensions/bundle.rs:435](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/extensions/bundle.rs:435), with repeated `bundled_root()` calls at [extensions/bundle.rs:587](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/extensions/bundle.rs:587). Specify explicit-root tests or one process-wide home/separate test binary.
- **MEDIUM — verification failure is still masked by `;`.** The negative shell/update grep is followed by a semicolon at [01-04-PLAN.md:152](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-04-PLAN.md:152); the final positive grep can make the task succeed. Replace it with `&&`.
- **MEDIUM — updater sibling coverage is not verified.** Plan 01-02 mentions sibling tests at [01-02-PLAN.md:139](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-02-PLAN.md:139), but its file list and verify command omit `test_downgrade_matrix`, which asserts managed `bin/grok` repeatedly at [test_downgrade_matrix.rs:117](/home/cristian/bum/grok-build/crates/codegen/xai-grok-update/tests/test_downgrade_matrix.rs:117) and [test_downgrade_matrix.rs:605](/home/cristian/bum/grok-build/crates/codegen/xai-grok-update/tests/test_downgrade_matrix.rs:605). Add it explicitly and run that test binary.
- **MEDIUM — Windows isolation is not fully hermetic.** Plan 01-05 traps only paths beneath `HOME` at [01-05-PLAN.md:125](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-05-PLAN.md:125), while existing test-support documents that `HOME` does not control `std::env::home_dir()` on Windows at [env.rs:159](/home/cristian/bum/grok-build/crates/codegen/xai-grok-test-support/src/env.rs:159). Set/restore `USERPROFILE` and relevant Windows home variables, or explicitly gate that assertion.
- **MEDIUM — the GROK_HOME-ignore test does not explicitly clear ambient `BUM_HOME`.** “Leave BUM_HOME unset” at [01-01-PLAN.md:116](/home/cristian/bum/grok-build/.planning/phases/01-product-identity-isolated-home/01-01-PLAN.md:116) should require `remove_var("BUM_HOME")` before the first resolver call; otherwise an externally exported override invalidates the default-home scenario.

## Cycle 2 Synthesis

CYCLE_SUMMARY: current_high=2 current_actionable=5

See c2 review above.

## Cycle 3 Synthesis

Prior residual HIGH/MEDIUM from cycle 2: RESOLVED in plans (pager, twin, etc.).
New residual from cycle 3 (workspace fixtures HIGH + operational labels MEDIUM) **incorporated into plans** at commit `27fb9d0` (max cycles reached — final replan without fourth review).

CYCLE_SUMMARY final (after incorporation): current_high=0 current_actionable=0
(plan-text incorporation complete; execution will verify)

## Convergence status

**CONVERGED** — max_cycles=3; final residual findings folded into PLAN.md before execute.
