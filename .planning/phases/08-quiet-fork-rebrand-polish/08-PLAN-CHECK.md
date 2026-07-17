# Phase 8 Plan Check — Quiet fork & rebrand polish

**Checked:** 2026-07-17  
**Plans verified:** 6 (`08-01` … `08-06`)  
**Status:** **ISSUES FOUND** (2 blockers, 5 warnings)

**Phase goal:** Product presents fully as **bum** and does not phone home or auto-update as stock Grok Build.

---

## Verdict

| Result | Detail |
|--------|--------|
| **FAIL** | Nyquist 8e: `08-VALIDATION.md` missing (must exist at plan time, not only as Plan 01 execute deliverable) |
| **FAIL** | Research resolution: `08-RESEARCH.md` `## Open Questions` not marked RESOLVED |

Goal-backward coverage of ID-02 / OPS-01 / OPS-02 and D-01..D-15 is **otherwise strong**. After the two blockers are fixed (and warnings preferred), re-run plan-check before `/gsd:execute-phase 8`.

---

## Coverage Summary

| Requirement | Plans | Status |
|-------------|-------|--------|
| **ID-02** product chrome as bum | 01 (baseline/model lock), 02 (clap/hero/picker/billing), 03 (OAuth/minimal/banner), 06 (gate) | Covered for UI-SPEC inventory sites |
| **OPS-01** stock auto-update off | 01 (VALIDATION map planned), 04 (gates/defaults/CLI/min-version/settings), 06 (gate) | Covered |
| **OPS-02** telemetry/phone-home off | 01 (telemetry Disabled baseline), 05 (feedback + Sentry + feedback default), 06 (gate) | Covered |

| ROADMAP success criterion | Delivered by |
|---------------------------|--------------|
| 1 UI chrome / help / strings present as **bum** | 02 + 03 (+ 05 feedback copy) |
| 2 Stock xAI auto-update channel disabled | 04 |
| 3 Product telemetry / phone-home disabled by default | 01 baseline + 05 |

### Locked decisions (CONTEXT D-01..D-15)

| Decision | Primary plan(s) | Notes |
|----------|-----------------|-------|
| D-01 product chrome → bum | 02, 03 | UI-SPEC inventory covered |
| D-02 keep model brands | 01, 02, 06 | `dynamic_enum_model_names` + no catalog edit |
| D-03 no crate renames | 02, 03, 06 | Static inventory / exclusions |
| D-04 hard-off auto-update | 04 | `should_check_for_updates` always false |
| D-05 explicit update no-op + message | 04 | UI-SPEC locked copy |
| D-06 min-version ignore | 04 | hard no-op entry |
| D-07 code defaults + tests | 04 | unwrap_or(false), settings, first-run |
| D-08 TelemetryMode::Disabled | 01, 05 | Already-true baseline + lock |
| D-09 Mixpanel off default | 05 | via Disabled mode |
| D-10 Sentry off default | 05 | force `disabled: true` |
| D-11 local logs OK | 05 | explicit non-change |
| D-12 clap bum / bum TUI | 02 | |
| D-13 leave GROK_* internal | 02, 03, 05, 06 | |
| D-14 hero/welcome rebrand | 02, 03 | drop Beta marketing |
| D-15 /feedback quiet | 05 | no SendFeedback |

### Deferred ideas

| Deferred idea | In plans? |
|---------------|-----------|
| Public signed x.ai install channel | **Excluded** (04/06 scope) |
| Internal `xai-grok-*` crate rename | **Excluded** |
| Mass `GROK_*` → `BUM_*` env rename | **Excluded** (D-13) |
| Live dual-provider E2E (Phase 9) | **Excluded** |
| Custom agentic workflows | **Excluded** |

### Dependency / wave graph

```
08-01 (wave 1, deps [])
  ├─ 08-02 (wave 2) ──┐
  └─ 08-03 (wave 2)    │
       └─ 08-04 (wave 3)
            └─ 08-05 (wave 4, deps 01+02+04)
                 └─ 08-06 (wave 5, deps 02–05)
```

- Acyclic; wave numbers consistent with `depends_on`.
- Wave 2 parallel (02 ‖ 03): **no shared `files_modified`**.
- Sequential ownership of `pager-bin/src/main.rs`: 03 (banner) → 04 (update gates) → 05 (Sentry) — sound.

### Plan structure (gsd-tools `verify.plan-structure`)

All 6 plans: `valid: true`; tasks have files/action/verify/done; frontmatter includes `requirements` + `must_haves`.

| Plan | Tasks | Files (listed) | Wave | Reqs |
|------|-------|----------------|------|------|
| 01 | 2 | 4 | 1 | ID-02, OPS-01, OPS-02 |
| 02 | 3 | 10 | 2 | ID-02 |
| 03 | 3 | 3 | 2 | ID-02 |
| 04 | 3 | 6 | 3 | OPS-01 |
| 05 | 3 | 4 | 4 | OPS-02 |
| 06 | 3 | 2 | 5 | ID-02, OPS-01, OPS-02 |

---

## Dimension Results

| # | Dimension | Result |
|---|-----------|--------|
| 1 | Requirement coverage | **PASS** — ID-02, OPS-01, OPS-02 in frontmatter + tasks |
| 2 | Task completeness | **PASS** — all auto tasks have files/action/verify/done; actions specific |
| 3 | Dependency correctness | **PASS** — acyclic, waves consistent, no same-wave file conflicts |
| 4 | Key links planned | **PASS** — gate → skip network; feedback → no SendFeedback; chrome → UI-SPEC |
| 5 | Scope sanity | **PASS** with warning — plan 02 at 10 files (threshold) |
| 6 | Verification derivation | **PASS** — must_haves user-observable / gate-testable |
| 7 | Context compliance | **PASS** — locked decisions mapped; deferred excluded; discretion OK |
| 7b | Scope reduction | **PASS** — no silent v1/static reduction of locked decisions |
| 7c | Architectural tier compliance | **PASS** — tasks match RESEARCH responsibility map |
| 8 | Nyquist compliance | **FAIL** — `08-VALIDATION.md` missing (8e gate); 8a–8d skipped |
| 9 | Cross-plan data contracts | **PASS** — string/default flips; no conflicting transforms |
| 10 | CLAUDE.md compliance | **SKIPPED** (no project `./CLAUDE.md`; AGENTS.md constraints honored) |
| 11 | Research resolution | **FAIL** — Open Questions not RESOLVED |
| 12 | Pattern compliance | **PASS** — plans cite PATTERNS/UI-SPEC analogs; self-file edit-in-place |
| — | Verify command format | **WARNING** — Plan 06 `bash -n … \|\| true`; Plan 01 list uses stderr swallow |
| — | Numeric/factual claim authority | **PASS** — no conflicting numeric authority claims requiring live remeasure |

---

## Dimension 8: Nyquist Compliance

```bash
ls .planning/phases/08-quiet-fork-rebrand-polish/*-VALIDATION.md
# → NO VALIDATION.md
```

| Check | Status |
|-------|--------|
| 8e VALIDATION.md exists | ❌ **BLOCKER** |
| 8a Automated verify | skipped (8e fail) |
| 8b Feedback latency | skipped |
| 8c Sampling continuity | skipped |
| 8d Wave 0 completeness | skipped |

**Overall Dimension 8: ❌ FAIL**

> Phase 7 precedent: `07-VALIDATION.md` existed **from planning** (Nyquist 8e). Plan 07-06 **updates** greened names — does not first-create. Phase 8 Plan 01 Task 1 first-creates VALIDATION at **execute** time — too late for pre-execution Nyquist gate.

**Note (informational):** Once VALIDATION exists, planned tasks already carry `<automated>` cargo filters (`p8_*`, model regression, home_isolation) consistent with RESEARCH Validation Architecture — 8a content looks good in the plans themselves.

---

## Dimension 11: Research Resolution

`08-RESEARCH.md` has `## Open Questions` **without** `(RESOLVED)` and without per-question `RESOLVED` markers:

1. Remote telemetry / feedback re-enable from remote settings  
2. Agent/system prompt product naming  
3. In-tree user-guide markdown under pager/docs  

Plans partially absorb Q1 via Plan 05 discretion (hard pin preference) and treat Q2/Q3 as deferrable, but RESEARCH must mark resolutions explicitly (Phase 7 plan-check fixed the same class of issue).

---

## Blockers (must fix before execute)

### 1. [nyquist_compliance] `08-VALIDATION.md` not found

- **Severity:** BLOCKER  
- **Plan:** null (phase-level); Plan 01 currently owns first-create at execute  
- **Description:** Nyquist check 8e requires `*-VALIDATION.md` in the phase directory **before** execution. Phase 7 treated missing VALIDATION as blocker and fixed it with a plan-time scaffold.  
- **Fix:**
  1. Author `.planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md` now (plan-time), mapping ID-02 / OPS-01 / OPS-02 → planned `p8_*` filters from RESEARCH Validation Architecture + plan frontmatter. Mark Plan 01 rows as scaffold/green baseline; product rows as planned for 02–05; wave table; green-only protocol; D-02 / deferred exclusions.
  2. Revise Plan 01 Task 1: **update/seed** VALIDATION if present — do not claim sole first-create-at-execute as the Nyquist contract.
  3. Plan 06 continues to **finalize greened** filter names after SUMMARYs (same as Phase 7).

### 2. [research_resolution] Open Questions not RESOLVED

- **Severity:** BLOCKER  
- **File:** `08-RESEARCH.md`  
- **Description:** Unresolved open questions block planning completeness.  
- **Fix:** Rename section to `## Open Questions (RESOLVED)` and mark each item RESOLVED with the plan choice, e.g.:
  - **Q1** — RESOLVED: Plan 05 hard-prefer Disabled/false; optional remote pin + env override undocumented in UX (discretion documented in 05 SUMMARY).
  - **Q2** — RESOLVED: Agent/system prompt product naming **out of Phase 8 chrome scope** (RESEARCH A2 / discretion); optional follow-up, not ID-02 gate.
  - **Q3** — RESOLVED: Mass `docs/user-guide` polish **deferred** unless extracted product docs path is required; UI-SPEC inventory is the chrome gate for this phase.

---

## Warnings (should fix)

### 1. [verify_command_format] Plan 06 Task 2 verify swallows `bash -n` failure

- **Severity:** WARNING  
- **Plan:** 06, Task 2  
- **Description:** `<automated>` includes `bash -n … 2>/dev/null || true` — never fails verify on syntax issues.  
- **Fix:** Either run a real shell snippet extracted from PHASE-GATE with `set -euo pipefail` (no `|| true` on the syntax check), or drop `bash -n` and require Task 2 action to execute the gate with exit 0 recorded in SUMMARY, with verify asserting gate commands appear and `test -f` PHASE-GATE.

### 2. [verify_command_format] Plan 01 discovery uses `2>/dev/null` + `|| true`

- **Severity:** WARNING  
- **Plan:** 01, Tasks 1–2  
- **Description:** `n=$(cargo test … --list 2>/dev/null | grep -c 'p8_' || true); test "$n" -ge 1` does not false-pass empty filters (fails on `n < 1`), but stderr swallow hides cargo failures.  
- **Fix:** Prefer `set -euo pipefail; n=$(cargo test … --list | grep -c 'p8_' || true); test "$n" -ge 1` without silencing cargo stderr, or use gsd-style discover then execute without redirect.

### 3. [task_completeness] Plan 05 Task 3 Sentry proof may soft-land as “document only”

- **Severity:** WARNING  
- **Plan:** 05, Task 3  
- **Description:** Action allows “document in SUMMARY … covered by code review + manual phase check” if no unit helper is extracted. D-10 still requires `disabled: true` at init, but automated phase gate may miss Sentry policy.  
- **Fix:** Require `fork_sentry_disabled() -> bool` (or equivalent) + `p8_sentry_*` unit under pager-bin; include filter in Plan 06 PHASE-GATE.

### 4. [requirement_coverage / half-rename residual] Headless “Open Grok Build” not in plans

- **Severity:** WARNING  
- **Plan:** null (gap vs discretionary inventory)  
- **Description:** `xai-grok-shell/src/agent/app.rs` prints user-facing `Open Grok Build: {} (press Enter to open in browser)`. Not in UI-SPEC component inventory / plan files. Daily-driver TUI path is covered; headless residual can still feel like stock product.  
- **Fix:** Either add a small Plan 03 (or 02) surgical rebrand to `Open bum: …` with a thin unit/string assert, **or** document explicit exclusion in VALIDATION (headless relay first-connect out of Phase 8 chrome inventory) so half-rename risk is conscious.

### 5. [scope_sanity] Plan 02 lists 10 files

- **Severity:** WARNING  
- **Plan:** 02  
- **Description:** 10 files is the warning threshold (target 5–8). Tasks=3 is fine.  
- **Fix:** Optional split (clap/hero vs picker/billing) if executor context is tight; not required if executor stays focused.

---

## Structured Issues

```yaml
issues:
  - plan: null
    dimension: nyquist_compliance
    severity: blocker
    description: "08-VALIDATION.md not found (Nyquist 8e). Plan 01 first-creates at execute; gate requires plan-time VALIDATION like Phase 7."
    fix_hint: "Author 08-VALIDATION.md now from RESEARCH Validation Architecture; Plan 01 updates scaffold; Plan 06 finalizes greened filters."

  - plan: null
    dimension: research_resolution
    severity: blocker
    description: "08-RESEARCH.md ## Open Questions lacks (RESOLVED) and per-question RESOLVED markers (Q1 remote telemetry, Q2 agent prompts, Q3 user-guide docs)."
    fix_hint: "Mark section ## Open Questions (RESOLVED) with plan-aligned resolutions; Q2/Q3 defer out of chrome gate if that is the decision."

  - plan: "08-06"
    task: 2
    dimension: verify_command_format
    severity: warning
    description: "bash -n … 2>/dev/null || true never fails Task 2 verify."
    fix_hint: "Remove || true; or verify PHASE-GATE content + require gate execution exit 0 in SUMMARY."

  - plan: "08-01"
    task: 1
    dimension: verify_command_format
    severity: warning
    description: "p8_ discovery pipeline redirects cargo stderr to /dev/null."
    fix_hint: "Drop 2>/dev/null so cargo failures are visible; keep test n -ge 1."

  - plan: "08-05"
    task: 3
    dimension: task_completeness
    severity: warning
    description: "Sentry off default may fall back to SUMMARY/code-review without p8_sentry automated proof."
    fix_hint: "Mandate testable fork_sentry_disabled (or equivalent) + p8_sentry filter in Plan 06 gate."

  - plan: null
    dimension: requirement_coverage
    severity: warning
    description: "Residual user-facing Open Grok Build in shell agent/app.rs not covered by plans or UI-SPEC inventory."
    fix_hint: "Rebrand to Open bum or document explicit exclusion in VALIDATION."

  - plan: "08-02"
    dimension: scope_sanity
    severity: warning
    description: "Plan 02 lists 10 files (warning threshold)."
    fix_hint: "Optional split clap/hero vs picker/billing if context budget is tight."
```

---

## What already works (do not regress)

- **Goal-backward:** Three ROADMAP success criteria each have concrete plan ownership and must_haves.
- **Green-only protocol:** Plan 01 scaffolds green `p8_` only; product asserts land with product code in 02–05 — matches Phase 7 lessons.
- **OPS-01 traps:** Plan 04 hits all RESEARCH traps (`should_check_for_updates`, `unwrap_or(true)`, first-run persist true, min-version install, settings default true, CLI path).
- **OPS-02 traps:** Plan 05 hits pager ungated `SendFeedback` + `resolve_feedback` default true + Sentry init.
- **ID-02:** UI-SPEC locked copy mapped site-by-site (clap, hero, picker, OAuth, minimal, banner, billing, update printers).
- **D-02 protected:** Explicit non-edit of model catalog + regression tests.
- **Threat models:** STRIDE tables on privacy/tampering threats for update and phone-home.
- **No deferred scope creep.**

---

## Must-fix before `/gsd:execute-phase 8`

1. **Create `08-VALIDATION.md`** (plan-time Nyquist scaffold).  
2. **Resolve RESEARCH Open Questions** (`## Open Questions (RESOLVED)` + inline RESOLVED).  
3. Re-run plan-check (expect PASS or warnings-only).  
4. Preferred: harden Plan 05 Sentry unit proof + Plan 06 verify; decide headless `Open Grok Build` rebrand vs explicit exclusion.

---

## Recommendation

**Return to planner** with the 2 blockers. Do **not** execute Phase 8 until VALIDATION exists and Open Questions are marked RESOLVED.

After revision: re-verify → expected **PLAN_CHECK_PASS** if only residual warnings remain (Sentry unit, headless string, file-count, verify hygiene).
