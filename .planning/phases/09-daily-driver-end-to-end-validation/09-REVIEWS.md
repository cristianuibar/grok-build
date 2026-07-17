---
phase: 9
reviewers: [codex]
reviewed_at: 2026-07-17T15:06:52Z
plans_reviewed: ['09-01-PLAN.md', '09-02-PLAN.md', '09-03-PLAN.md', '09-04-PLAN.md', '09-05-PLAN.md']
cycle: 3
prior_cycle: 2
current_high: 0
current_actionable: 2
verdict: REPLAN
replanned_at: 2026-07-17
cycle2_replanned_at: 2026-07-17
---

# Cross-AI Plan Review — Phase 9

**Phase:** 09-daily-driver-end-to-end-validation  
**Reviewers:** Codex only (`--codex`)  
**Cycle:** 1  
**Plans:** 09-01-PLAN.md, 09-02-PLAN.md, 09-03-PLAN.md, 09-04-PLAN.md, 09-05-PLAN.md

---

## Codex review (cycle 1)

# Plan Review — Phase 9

## Summary

The plan sequence is fundamentally sound: it separates fixture-only regression evidence from required live OAuth UAT and refuses to treat automated tests as proof of OPS-03–06. It reuses real prior-phase seams rather than proposing a parallel validation framework. The main gaps are operational: duplicated residual tests, incomplete gate verification in Plan 02, weak protection for a live UAT workspace/evidence, and no required check of actual user-facing CLI chrome despite visible remaining Grok branding.

## 09-01 — Validation map and `p9_` smoke

### Strengths

- The proposed fixture approach matches the existing integration harness: it already creates a process-wide temporary `BUM_HOME`, writes a dual-provider `auth.json`, and avoids ambient homes ([cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:74), [cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:92)).
- The planned dual-slot check can exercise the real storage API. `read_provider_auth_store` distinguishes missing slots from malformed storage and does not consult stock homes ([storage.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:197)).
- The planned login-hint assertion is anchored in a real fail-closed mechanism: no credentials returns a provider-specific error, while BYOK or a usable slot does not ([config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:5612)).
- The plan correctly avoids network-backed tests; the existing fixture smoke already proves both provider slots are readable and usable with fake credentials ([cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:308)).

### Concerns

- **MEDIUM — [C1-M1] likely redundant test code.** Both proposed `p9_` behaviors are already directly covered by `p7_wave0_harness_smoke_compiles_and_runs` and `p7_missing_provider_gate_error_suggests_bum_login_for_empty_codex` ([cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:132), [cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:312)). New copies add phase-specific discoverability but little behavioral coverage, creating future maintenance duplication.
- **LOW — [C1-L1] the optional route test is weaker than its wording implies.** The existing route test only asserts separate host/slot selection, not actual HTTP Authorization isolation; the latter intentionally lives in the in-crate `p7_isolation_*` tests ([cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:266)).

### Suggestions

- Add one narrowly differentiated `p9_` test—or rename/alias a single existing fixture test if project conventions allow—rather than duplicating both existing checks.
- If retaining a route residual, label it “route metadata isolation”; rely on `p7_isolation_*` for bearer/backend isolation.

### Risk assessment

**LOW–MEDIUM.** The plan is safe and grounded in real fixtures, but its extra tests risk becoming documentation-by-duplication rather than useful regression coverage.

---

## 09-02 — Automated hybrid-gate half

### Strengths

- The gate’s `discover()` pattern is proven in prior phases and genuinely fails closed on absent filters before executing tests ([07-PHASE-GATE.md](/home/cristian/bum/grok-build/.planning/phases/07-cross-provider-multi-agent-orchestration/07-PHASE-GATE.md:38)).
- The selected switch tests exercise real session model changes, including a post-switch sample route, not merely a pure provider enum decision ([model_switch_gate.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/model_switch_gate.rs:664), [model_switch_gate.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/model_switch_gate.rs:713)).
- The planned `p7_isolation_*` filters are real child-sample authorization tests in the shell’s subagent unit suite, and both provider directions exist ([subagent tests](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs:4573), [subagent tests](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs:4617)).
- Including home isolation is valuable: it launches the actual `bum` binary under trapped `GROK_HOME`/`CODEX_HOME` and asserts state is written only below `BUM_HOME` ([home_isolation.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs:81)).

### Concerns

- **MEDIUM — [C1-M2] Task 2’s verification command is not the gate it claims to run.** Its action says to run all P0/P1 groups, but the verification omits `p6_missing_provider`, `switch_changes_next_sample_route`, `p7_spawn_missing_provider`, `p7_parent_model`, `p8_no_auto_update`, and `p8_sentry`. These are real filters: for example, the actual missing-provider tests are present ([model_switch_gate.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/model_switch_gate.rs:544)) and the binary has both p8 checks ([main.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager-bin/src/main.rs:2260), [main.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager-bin/src/main.rs:2300)).
- **LOW — [C1-L2] `home_isolation` execution lacks the required non-vacuous discovery assertion.** The existing Phase 8 gate first verifies a `hermetic|home` test name before execution ([08-PHASE-GATE.md](/home/cristian/bum/grok-build/.planning/phases/08-quiet-fork-rebrand-polish/08-PHASE-GATE.md:130)); Plan 02 Task 2 should do the same.
- **LOW — [C1-L3] unnecessary test runtime.** Running `p7_isolation` and then both direction-specific filters executes the direction tests multiple times. The aggregate filter includes both directions; Phase 7 uses separate discovery checks for direction names and one aggregate execution ([07-PHASE-GATE.md](/home/cristian/bum/grok-build/.planning/phases/07-cross-provider-multi-agent-orchestration/07-PHASE-GATE.md:109)).

### Suggestions

- Make Plan 02 Task 2’s `<automated>` command exactly mirror the full inventory, or invoke the documented gate script block as the sole source of truth.
- For isolation, discover each direction, then execute only aggregate `p7_isolation` once.
- Replace the bare home-isolation invocation with `discover xai-grok-pager-bin hermetic --test home_isolation`.

### Risk assessment

**MEDIUM.** The intended gate is strong, but the plan’s executable verification currently permits an incomplete early automated result.

---

## 09-03 — UAT runbook and preflight

### Strengths

- The login commands are correct: bare `bum login` maps to xAI while `--provider codex` maps to Codex ([cli.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/cli.rs:46), [main.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager-bin/src/main.rs:1778)).
- `bum auth status` is suitable for the checklist: its status formatter deliberately prints no raw access or refresh credentials ([status.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/status.rs:231)).
- The default model IDs are real embedded entries: `grok-build` and `gpt-5.6-sol` ([default_models.json](/home/cristian/bum/grok-build/crates/codegen/xai-grok-models/default_models.json:2), [default_models.json](/home/cristian/bum/grok-build/crates/codegen/xai-grok-models/default_models.json:19)).
- The matrix correctly requires productive tool use, same-process switching, and both spawn directions rather than relying on a login-only smoke.

### Concerns

- **MEDIUM — [C1-M3] no safe UAT workspace requirement.** The runbook tells the operator to perform real edit/shell operations but does not require a disposable worktree/test file, a baseline `git status`, or cleanup verification. This can unintentionally modify the development checkout while producing ambiguous evidence.
- **MEDIUM — [C1-M4] it does not explicitly validate the actual CLI chrome.** User-visible strings still say “Grok” and refer to `~/.grok` in help text ([cli.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/cli.rs:29), [cli.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/cli.rs:110), [cli.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/cli.rs:749)). A required `bum --help`/login/switch chrome check should be part of preflight, especially since Plan 04 permits residual chrome fixes.
- **LOW — [C1-L4] the planned secret scan is too narrow.** JWT/private-key regexes do not detect opaque OAuth tokens, copied auth JSON, or accidentally committed credential-like values. The storage format itself includes credential fields, so a filename/content guard is needed ([storage.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:206)).

### Suggestions

- Require a disposable repository/worktree and a harmless fixture file; record the initial and final `git status --short`.
- Add a required preflight check for `target/debug/bum --help`, `bum auth status`, login prompts, and `/model` labels; treat product-visible “grok” residue as an explicit blocker decision.
- Add a check that no `auth.json` is staged/tracked and scan the phase diff, not only two secret formats.

### Risk assessment

**MEDIUM.** The live-runbook architecture is good, but safe operational execution and the rebrand verification are under-specified.

---

## 09-04 — Live dual-login UAT and blocker fixes

### Strengths

- Correctly makes the human test blocking and refuses fixture substitution.
- The stated missing-provider behavior is genuinely fail-closed: a missing Codex slot prevents a switch and emits `bum login --provider codex` ([model_switch_gate.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/model_switch_gate.rs:544)).
- Its OPS-06 requirement is meaningful: existing automated tests verify that the parent model remains unchanged after a cross-provider spawn ([subagent tests](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs:4700)).

### Concerns

- **MEDIUM — [C1-M5] conditional source changes are absent from `files_modified`.** Task 3 authorizes fixes to auth, routing, chrome, and tools, but the declared file list contains only `09-UAT.md`. That makes execution reporting/review blind to likely code changes, such as the real switch handler ([model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:57)) or the spawn gate ([handle_request.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:68)).
- **MEDIUM — [C1-M6] live evidence is underspecified for “child results return.”** The checklist needs a minimum redacted observation format: parent model, child model, effort, task outcome, and return-to-parent evidence. Otherwise a spawned child that starts but never returns could be marked as a pass.
- **LOW — [C1-L5] the checkpoint should explicitly distinguish a provider outage/service limit from auth expiry and product defects.** The plan mentions re-auth but needs a final classification policy so Phase 9 does not falsely claim a product fix is needed—or falsely pass after an availability failure.

### Suggestions

- Declare conditional source paths as “may modify,” or require Task 3 to update summary/frontmatter with every actual code path changed.
- Add structured OPS-06 evidence fields: parent model, requested child model/effort, result received, parent model after completion, and redacted error classification if failed.
- Add explicit outcomes: `PASS`, `PRODUCT BLOCKER`, `AUTH/ACCOUNT BLOCKED`, `PROVIDER OUTAGE/LIMIT`; only PASS may feed Plan 05 GREEN.

### Risk assessment

**MEDIUM.** The human gate is correctly mandatory, but auditability and response handling need tightening.

---

## 09-05 — Hybrid close and verification

### Strengths

- The hard formula is the right one: it requires non-vacuous automated results plus signed live proof, rather than letting an artifact claim success.
- It correctly re-runs tests instead of trusting prior summaries.
- It maps Phase 7’s deferred live E2E work to actual Phase 9 live evidence; this is appropriate because the Phase 7 gate explicitly deferred live dual-login E2E ([07-PHASE-GATE.md](/home/cristian/bum/grok-build/.planning/phases/07-cross-provider-multi-agent-orchestration/07-PHASE-GATE.md:30)).
- It preserves provider separation as an auditable concern; route selection itself uses distinct credential-slot values ([cross_provider_subagent.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs:275)).

### Concerns

- **MEDIUM — [C1-M7] “repair thin wrappers until green” is too permissive at final gate time.** If a previously required filter disappears or fails, adding a wrapper can conceal a regression. The appropriate default is to stop, explain the mismatch, and restore/repair the original behavioral test—not introduce a superficial discoverable test.
- **LOW — [C1-L6] full-gate home isolation still lacks explicit discover-before-execute in its shown verification command**, despite the plan’s own cargo hygiene standard and the existing Phase 8 precedent ([08-PHASE-GATE.md](/home/cristian/bum/grok-build/.planning/phases/08-quiet-fork-rebrand-polish/08-PHASE-GATE.md:130)).
- **MEDIUM — [C1-M8] final secret verification remains regex-only.** Verification must ensure no credential file is tracked or included in the commit, not merely that a JWT-shaped string is absent.

### Suggestions

- Change failure handling to: preserve required filter names; diagnose target/filter mismatch; restore behavioral coverage; require a reviewer-visible explanation for any replacement test.
- Use the same `discover` implementation for home isolation.
- Before GREEN, require `git ls-files | rg '(^|/)(auth\.json|.*token.*)$'` to be clean for phase artifacts and inspect `git diff --check` plus a targeted diff scan.

### Risk assessment

**MEDIUM.** The final evidence model is strong, but final-gate mutation and secret checks need stricter controls.

## Overall risk assessment

**MEDIUM.** The plans can achieve the Phase 9 goal because the live proof is mandatory and the referenced unit/integration mechanisms exist. Address the Plan 02 incomplete verification, add a safe live-UAT workspace and explicit CLI-chrome check, and tighten final-gate/evidence controls before execution.

### Verdict

**REPLAN** — No HIGH blockers to the hybrid methodology itself, but eight MEDIUM and six LOW findings must land in the PLAN.md files before execution. Highest priority: complete Plan 02 gate verification (C1-M2), disposable UAT workspace (C1-M3), CLI chrome preflight (C1-M4), structured OPS-06 evidence (C1-M6), and no thin-wrapper escape hatch at final gate (C1-M7).

CYCLE_SUMMARY: current_high=0 current_actionable=14

---

## Consensus Summary

Single reviewer (Codex). Source-grounded against real test files, handlers, CLI, models, and prior-phase gates.

### Agreed Strengths

- Hybrid gate correctly separates fixture residual from mandatory live dual-login UAT (D-16 / no fixture-only PASS for OPS-03..06).
- Plans re-use proven `discover` + prior-phase filters rather than inventing a parallel framework.
- Live proof is blocking; Phase 7 deferred dual-login E2E is properly mapped to Phase 9 UAT.
- Referenced mechanisms (dual-slot store, missing-provider login hints, switch route, p7 isolation, home isolation, default model IDs) exist in source.

### Agreed Concerns

- Plan 02 Task 2 verify command under-runs the full P0/P1 inventory claimed by the action.
- Live UAT lacks a disposable workspace and structured OPS-06 evidence fields.
- CLI chrome residual is not a required preflight check despite remaining Grok/`~/.grok` user-facing strings.
- Final-gate “repair thin wrappers until green” can mask missing filters; secrets checks need file-path guards beyond JWT regex.

### Divergent Views

N/A — single reviewer.

### Findings index (cycle 1)

| ID | Sev | Plan | One-liner |
|----|-----|------|-----------|
| C1-M1 | MEDIUM | 09-01 | Duplicate p9_ coverage of existing p7 harness/login-hint tests |
| C1-M2 | MEDIUM | 09-02 | Task 2 verify omits several required P0/P1 filters |
| C1-M3 | MEDIUM | 09-03 | No disposable UAT workspace / git-status baseline |
| C1-M4 | MEDIUM | 09-03 | No required CLI chrome / rebrand preflight |
| C1-M5 | MEDIUM | 09-04 | Conditional product-fix paths missing from files_modified |
| C1-M6 | MEDIUM | 09-04 | OPS-06 live evidence underspecified for child return |
| C1-M7 | MEDIUM | 09-05 | “Thin wrapper until green” too permissive at final gate |
| C1-M8 | MEDIUM | 09-05 | Secret verify is regex-only; need auth.json/path guards |
| C1-L1 | LOW | 09-01 | Optional route residual overclaims HTTP auth isolation |
| C1-L2 | LOW | 09-02 | home_isolation lacks discover-before-execute |
| C1-L3 | LOW | 09-02 | Redundant p7_isolation execute of direction filters |
| C1-L4 | LOW | 09-03 | Preflight secret scan too narrow |
| C1-L5 | LOW | 09-04 | No PASS vs PRODUCT/AUTH/OUTAGE classification |
| C1-L6 | LOW | 09-05 | Final-gate home isolation missing discover step |

CYCLE_SUMMARY: current_high=0 current_actionable=14

## Replan dispositions (cycle 1 → plans)

All 14 actionable findings incorporated into PLAN.md executable content. `current_actionable=0`.

| ID | Sev | Plan | Disposition | Where |
|----|-----|------|-------------|-------|
| C1-M1 | MEDIUM | 09-01 | Incorporated | Task 1: single differentiated p9_ residual; forbid dual p7 clones |
| C1-M2 | MEDIUM | 09-02 | Incorporated | Full P0/P1 inventory SoT table; Task 2 verify matches full inventory |
| C1-M3 | MEDIUM | 09-03 | Incorporated | Disposable worktree/fixture + initial/final git status in UAT + preflight |
| C1-M4 | MEDIUM | 09-03 | Incorporated | CLI chrome preflight; residual Grok chrome = blocker decision |
| C1-M5 | MEDIUM | 09-04 | Incorporated | Conditional may-modify paths in files_modified + SUMMARY path recording |
| C1-M6 | MEDIUM | 09-04 | Incorporated | OPS-06 structured evidence fields (parent/child/effort/result/parent_after) |
| C1-M7 | MEDIUM | 09-05 | Incorporated | Final-gate filter failure policy: restore original; no thin-wrapper default |
| C1-M8 | MEDIUM | 09-05 | Incorporated | Secrets GREEN gate: content + auth.json path + phase-diff scan |
| C1-L1 | LOW | 09-01 | Incorporated | Optional route residual labeled metadata-only; bearer = p7_isolation |
| C1-L2 | LOW | 09-02 | Incorporated | home_isolation discover-before-execute in PHASE-GATE + Task 2 verify |
| C1-L3 | LOW | 09-02 | Incorporated | list both isolation dirs; execute aggregate p7_isolation once |
| C1-L4 | LOW | 09-03 | Incorporated | Preflight secrets: auth.json/token paths + phase-diff scan |
| C1-L5 | LOW | 09-04 | Incorporated | Outcome taxonomy PASS \| PRODUCT BLOCKER \| AUTH/ACCOUNT BLOCKED \| PROVIDER OUTAGE/LIMIT |
| C1-L6 | LOW | 09-05 | Incorporated | Final-gate home_isolation discover-before-execute in Task 1 verify |

### Review Feedback Deferred / Rejected

None — all cycle-1 actionable findings claimed incorporated in replan (cycle 2 re-verifies below).

---

## Codex review (cycle 2) — 2026-07-17T15:01:18Z

Post-replan disposition audit of all 14 cycle-1 findings against executable PLAN.md content (not disposition tables alone). Source-grounded Codex review.

### Status of prior findings

| Finding | Status | Evidence |
|---|---|---|
| C1-M1 | RESOLVED | Single required `p9_` residual, explicit ban on dual p7 clones, discovery ≤3: [09-01-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-01-PLAN.md:128), [09-01-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-01-PLAN.md:141) |
| C1-M2 | RESOLVED | Full P0/P1 inventory SoT + Task 2 verify runs every listed filter: [09-02-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-02-PLAN.md:100), [09-02-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-02-PLAN.md:175) |
| C1-M3 | RESOLVED | Disposable workspace + initial/final `git status` + cleanup required: [09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:116) |
| C1-M4 | RESOLVED | CLI chrome preflight + residual Grok chrome blocker decision: [09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:118) |
| C1-M5 | RESOLVED | Conditional may-modify paths + SUMMARY path recording: [09-04-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md:115), frontmatter `files_modified` |
| C1-M6 | RESOLVED | OPS-06 structured fields; start-without-return is not PASS: [09-04-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md:129) |
| C1-M7 | RESOLVED | Final-gate restore-original / forbid thin wrappers: [09-05-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md:104) |
| C1-M8 | PARTIAL | Hard policy requires content + path + phase-diff ([09-05-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md:114)), but Task 3 automated verify only checks `auth.json` (not token-like paths / phase-diff content) and ends with masking `\|\| true` ([09-05-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md:206)) |
| C1-L1 | RESOLVED | Optional route residual labeled metadata-only; bearer stays `p7_isolation_*`: [09-01-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-01-PLAN.md:128) |
| C1-L2 | RESOLVED | home_isolation discover-before-execute in Plan 02 SoT/verify: [09-02-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-02-PLAN.md:147), [09-02-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-02-PLAN.md:175) |
| C1-L3 | RESOLVED | List both isolation dirs; execute aggregate `p7_isolation` once: [09-02-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-02-PLAN.md:146), [09-02-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-02-PLAN.md:175) |
| C1-L4 | PARTIAL | UAT task requires path + phase-diff checks ([09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:119)); script only “may print” suggestions and Task 2 verify checks only `auth.json` ([09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:147), [09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:152)) |
| C1-L5 | RESOLVED | Outcome taxonomy PASS \| PRODUCT BLOCKER \| AUTH/ACCOUNT BLOCKED \| PROVIDER OUTAGE/LIMIT: [09-04-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md:102) |
| C1-L6 | RESOLVED | Final-gate home_isolation discover-before-execute: [09-05-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md:148), [09-05-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md:156) |

### HIGH

None.

### MEDIUM

- **[C2-M1] Final secrets verify is a no-op (C1-M8 residual).** Plan 05 Task 3 automated chain ends with `... && git diff --check ... || true`, so shell precedence makes the entire verify succeed even when artifact checks, JWT/private-key scans, or tracked `auth.json` fail. It also never runs a mandatory phase-diff secret-content inspection. Fix: remove trailing broad `|| true`; isolate optional `git diff --check` so it cannot mask required gates; add path guard for token-like credential filenames and scoped phase-diff content scan. Evidence: [09-05-PLAN.md:201](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md), [09-05-PLAN.md:206](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md).

- **[C2-M2] Plan 04 Task 3 post-fix residual re-run is prose-only.** Task 3 authorizes may-modify fixes across switch/spawn/auth/cli/tools, but automated verify re-runs only `p9_`. Require discover+execute of the affected p6/p7/p8 filters based on changed path / failure class (not solely narrative action). Evidence: [09-04-PLAN.md:223](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md), [09-04-PLAN.md:231](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md).

### LOW

- **[C2-L1] Preflight secrets path/diff promise not mechanical (C1-L4 residual).** Make token-like path patterns and an explicit scoped `git diff` inspection required in the preflight script or Task 2 verify, rather than optional “may print” suggestions. Evidence: [09-03-PLAN.md:147](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md), [09-03-PLAN.md:152](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md).

### Verdict

**REPLAN** — no HIGH findings; 12/14 cycle-1 items fully RESOLVED; remaining enforcement gaps: final secret-gate verify can be bypassed (`|| true`), post-blocker residual re-run is not executable in Task 3 verify, and preflight secrets path/diff stay advisory.

### Summary

The replan substantively landed inventory completeness, disposable UAT workspace, chrome preflight, OPS-06 structured evidence, outcome taxonomy, thin-wrapper ban, and isolation hygiene. Residual risk is **verification enforcement**: the final GREEN secrets command can mask failures, and conditional product fixes lack a bound automated residual recheck.

CYCLE_SUMMARY: current_high=0 current_actionable=3

### Findings index (cycle 2 residual)

| ID | Sev | Plan | One-liner | Origin |
|----|-----|------|-----------|--------|
| C2-M1 | MEDIUM | 09-05 | Task 3 secrets verify ends in `\|\| true` / incomplete path+diff | C1-M8 residual + new |
| C2-M2 | MEDIUM | 09-04 | Task 3 verify only re-runs `p9_`, not affected p6/p7/p8 | new |
| C2-L1 | LOW | 09-03 | Token-path + phase-diff preflight not mechanical | C1-L4 residual |

### Cycle 1 → cycle 2 disposition rollup

| Bucket | Count |
|--------|-------|
| C1 fully RESOLVED | 12 (M1–M7, L1–L3, L5–L6) |
| C1 PARTIAL (still actionable) | 2 (M8→C2-M1, L4→C2-L1) |
| New cycle-2 actionable | 1 (C2-M2) |
| **current_actionable** | **3** |

## Consensus Summary (cycle 2)

Single reviewer (Codex). Disposition audit + residual enforcement review.

### Agreed Strengths

- Hybrid methodology, full P0/P1 inventory SoT, and live dual-login bar remain intact post-replan.
- Disposable UAT workspace, chrome preflight, OPS-06 structure, and outcome taxonomy are now executable plan content.
- Final-gate thin-wrapper ban and isolation discover hygiene are real (not table-only).

### Agreed Concerns

- Final secrets GREEN verify can succeed despite failures (`|| true` precedence).
- Post-blocker automated residual is under-specified in Plan 04 Task 3 verify.
- Preflight secrets path/diff still mostly advisory.

### Divergent Views

N/A — single reviewer.

## Current HIGH Concerns

None.

## Current Actionable Non-HIGH Concerns

None — all cycle-2 residuals incorporated (see replan dispositions below).

## Replan dispositions (cycle 2 → plans)

All 3 cycle-2 actionable findings incorporated into PLAN.md executable content. `current_actionable=0`.

| ID | Sev | Plan | Disposition | Where |
|----|-----|------|-------------|-------|
| C2-M1 | MEDIUM | 09-05 | INCORPORATED | Task 3 secrets verify: removed trailing broad `\|\| true`; required auth.json + token-like path guards + phase-diff content scan; optional `git diff --check` isolated so it cannot mask required gates |
| C2-M2 | MEDIUM | 09-04 | INCORPORATED | Post-fix residual map + Task 3 automated verify path-detects may-modify classes and discover+executes affected p6/p7/p8 filters (always p9_ + p6_dual_login + p7_isolation safety net) |
| C2-L1 | LOW | 09-03 | INCORPORATED | Task 2 preflight action requires fail-closed path + phase-diff secret gates (not "may print"); Task 2 verify enforces token-like `git ls-files` + scoped phase-diff content scan |

### Review Feedback Deferred / Rejected (cycle 2)

None.

CYCLE_SUMMARY: current_high=0 current_actionable=0

---

## Codex review (cycle 3) — 2026-07-17T15:06:52Z

Final convergence check after cycle-2 replan dispositions claimed `current_actionable=0`. Disposition audit of C2-M1 / C2-M2 / C2-L1 against executable PLAN.md content (Task action + `<automated>` verify), source-grounded. Flag only genuine remaining / NEW actionable items.

### Status of prior findings

| Finding | Status | Evidence |
|---|---|---|
| C2-M1 | PARTIAL | Required secret checks and isolated optional `git diff --check` are executable in [09-05-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md:214); no broad trailing `\|\| true` on the required chain. **However** the global token-like path guard (`git ls-files \| rg -qi '(^\|/)([^/]*token[^/]*\|credentials\.json\|\.oauth)($\|/)'`) always fails against legitimate tracked source files — e.g. [token_type.rs](crates/codegen/xai-grok-shell/src/auth/token_type.rs), [token.rs](crates/common/xai-grok-compaction/src/token.rs), `xai-token-estimation/*`, skill-token PTY fixtures (8 hits repo-wide). Final GREEN cannot complete as written. |
| C2-M2 | RESOLVED | Task action requires union-of-touched-class rechecks plus `p9_` ([09-04-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md:247)); Post-fix residual map + automated verify path-detects classes and discover+executes mapped filters ([09-04-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md:167), [09-04-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-04-PLAN.md:252)). Referenced p6/p7/p8 mechanisms exist in source. |
| C2-L1 | PARTIAL | Task action requires fail-closed path/diff checks ([09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:153)); Task 2 verify includes mechanical token tokens + path/diff gates ([09-03-PLAN.md](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md:158)). Same unscoped `token` filename pattern matches ordinary source, so preflight cannot pass on this repo. |

### HIGH

None.

### MEDIUM

- **[C3-M1] Token-like path guard is unscoped (C2-M1 residual).** Plan 05 Task 3 secrets GREEN verify (and Plan 03 Task 2 preflight) ban any tracked path whose basename contains `token` repo-wide. That collides with first-party code (`token_type.rs`, `token.rs`, `xai-token-estimation`, skill-token fixtures). Scope the guard to credential/artifact paths (e.g. `auth.json`, `credentials.json`, `.oauth`, `*token*` only under home/config/phase artifact dirs — or basename exact-match credential names), not all source. Keep auth.json + phase-diff content scans as-is. Evidence: [09-05-PLAN.md:214](.planning/phases/09-daily-driver-end-to-end-validation/09-05-PLAN.md), [09-03-PLAN.md:158](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md); repo hits via `git ls-files | rg -i 'token'` (8 false positives).

### LOW

- **[C3-L1] Same unscoped token-path pattern in Plan 03 preflight (C2-L1 residual).** Mirror the scoped fix from C3-M1 into Plan 03 Task 2 action + verify and the preflight script requirements so both halves of the secrets gate use the same non-false-positive pattern. Evidence: [09-03-PLAN.md:153](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md), [09-03-PLAN.md:158](.planning/phases/09-daily-driver-end-to-end-validation/09-03-PLAN.md).

### Verdict

**REPLAN** — no HIGH findings; C2-M2 fully RESOLVED; C2-M1 and C2-L1 remain PARTIAL for one shared root cause (over-broad token-like filename guard). No other new plan gaps found.

### Summary

Cycle-2 replan landed the structural fixes (no `|| true` mask, path+phase-diff secret policy, path-mapped residual recheck). Convergence is blocked only by an executable false-positive in the token-path guard that prevents any GREEN run on this tree. Narrow the filename pattern once across Plans 03 and 05; then re-review should converge.

CYCLE_SUMMARY: current_high=0 current_actionable=2

### Findings index (cycle 3 residual)

| ID | Sev | Plan | One-liner | Origin |
|----|-----|------|-----------|--------|
| C3-M1 | MEDIUM | 09-05 | Token-like `git ls-files` guard matches legitimate source (always fail) | C2-M1 residual |
| C3-L1 | LOW | 09-03 | Same unscoped token-path pattern in preflight verify/script | C2-L1 residual |

### Cycle 2 → cycle 3 disposition rollup

| Bucket | Count |
|--------|-------|
| C2 fully RESOLVED | 1 (C2-M2) |
| C2 PARTIAL (still actionable) | 2 (C2-M1→C3-M1, C2-L1→C3-L1) |
| New cycle-3 findings (unrelated) | 0 |
| **current_actionable** | **2** |

## Consensus Summary (cycle 3)

Single reviewer (Codex). Final convergence disposition audit.

### Agreed Strengths

- C2-M2 post-fix residual map + path-detect discover/execute is real executable plan content.
- C2-M1 structural secrets chain (content + auth.json + phase-diff + no trailing soft-fail) is present; optional `git diff --check` is isolated.
- C2-L1 mechanical preflight tokens and fail-closed intent are present in Task 2 action/verify.
- No new gaps beyond the token-path false positive.

### Agreed Concerns

- Repo-wide `*token*` basename guard cannot pass on this codebase; must scope before execution.

### Divergent Views

N/A — single reviewer.

## Current HIGH Concerns

None.

## Current Actionable Non-HIGH Concerns

1. **C3-M1 (MEDIUM, 09-05)** — Scope token-like path guard so it does not match first-party source (`token.rs`, `token_type.rs`, etc.); retain auth.json + credential-name + phase-diff content gates.
2. **C3-L1 (LOW, 09-03)** — Apply the same scoped path-guard pattern to Plan 03 preflight action/verify/script requirements.

CYCLE_SUMMARY: current_high=0 current_actionable=2
