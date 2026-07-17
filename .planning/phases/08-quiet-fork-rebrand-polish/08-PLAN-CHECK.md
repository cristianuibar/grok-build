# Phase 8 Plan Check — Quiet fork & rebrand polish

**Checked:** 2026-07-17 (re-verify after revision)  
**Plans verified:** 6 (`08-01` … `08-06`)  
**Status:** **ISSUES FOUND** (1 blocker, 2 warnings)

**Phase goal:** Product presents fully as **bum** and does not phone home or auto-update as stock Grok Build.

---

## Verdict

| Result | Detail |
|--------|--------|
| **FAIL** | Plan `08-03` YAML frontmatter is broken — closing `---` is glued to the last `key_links` line (`pattern: "bum \\(pager\\)"---`). `verify.plan-structure` → `valid: false` (all required frontmatter fields appear missing to the parser). |

### Prior blockers (revision) — resolved

| Prior blocker | Status |
|---------------|--------|
| Nyquist 8e: `08-VALIDATION.md` missing | **FIXED** — plan-time scaffold present |
| Research resolution: Open Questions not RESOLVED | **FIXED** — `## Open Questions (RESOLVED)` + Q1–Q3 marked |

### Hygiene claims — verified

| Claim | Status |
|-------|--------|
| Plan 06 verify no longer `bash -n … \|\| true` | **OK** — content/file asserts only |
| Plan 05 mandates `p8_sentry_*` | **OK** — Task 3 action: soft-land SUMMARY-only forbidden |
| Plan 01 discover no `2>/dev/null` | **OK** — list stderr not swallowed |
| Plan 03 Open Grok Build residual rebrand | **OK** — Task 1b + must_haves (blocked only by frontmatter parse) |

Goal-backward coverage of ID-02 / OPS-01 / OPS-02 and D-01..D-15 remains strong once Plan 03 frontmatter is repaired.

---

## Coverage Summary

| Requirement | Plans | Status |
|-------------|-------|--------|
| **ID-02** product chrome as bum | 01, 02, 03, 06 | Covered (03 structure invalid until frontmatter fix) |
| **OPS-01** stock auto-update off | 01, 04, 06 | Covered |
| **OPS-02** telemetry/phone-home off | 01, 05, 06 | Covered |

| ROADMAP success criterion | Delivered by |
|---------------------------|--------------|
| 1 UI chrome / help / strings present as **bum** | 02 + 03 (+ 05 feedback copy) |
| 2 Stock xAI auto-update channel disabled | 04 |
| 3 Product telemetry / phone-home disabled by default | 01 baseline + 05 |

### Locked decisions (CONTEXT D-01..D-15)

| Decision | Primary plan(s) | Notes |
|----------|-----------------|-------|
| D-01 product chrome → bum | 02, 03 | Includes Open bum residual (03 Task 1b) |
| D-02 keep model brands | 01, 02, 06 | No catalog edit |
| D-03 no crate renames | 02, 03, 06 | Exclusions |
| D-04 hard-off auto-update | 04 | `should_check_for_updates` always false |
| D-05 explicit update no-op + message | 04 | UI-SPEC locked copy |
| D-06 min-version ignore | 04 | hard no-op |
| D-07 code defaults + tests | 04 | unwrap_or(false), settings, first-run |
| D-08 TelemetryMode::Disabled | 01, 05 | Baseline + lock |
| D-09 Mixpanel off default | 05 | via Disabled |
| D-10 Sentry off default | 05 | force disabled + **mandatory** `p8_sentry_*` |
| D-11 local logs OK | 05 | explicit non-change |
| D-12 clap bum / bum TUI | 02 | |
| D-13 leave GROK_* internal | 02, 03, 05, 06 | |
| D-14 hero/welcome rebrand | 02, 03 | drop Beta marketing |
| D-15 /feedback quiet | 05 | no SendFeedback |

### Deferred ideas

| Deferred idea | In plans? |
|---------------|-----------|
| Public signed x.ai install channel | **Excluded** |
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

- Acyclic; wave numbers consistent with `depends_on` (once 03 frontmatter parses).
- Wave 2 parallel (02 ‖ 03): no shared `files_modified`.
- Sequential ownership of `pager-bin/src/main.rs`: 03 → 04 → 05.

### Plan structure (gsd-tools `verify.plan-structure`)

| Plan | Tasks | valid | Notes |
|------|-------|-------|-------|
| 01 | 2 | ✅ | |
| 02 | 3 | ✅ | 10 files (warning threshold) |
| 03 | 4 | ❌ | Frontmatter parse fail — see blocker |
| 04 | 3 | ✅ | |
| 05 | 3 | ✅ | |
| 06 | 3 | ✅ | |

---

## Dimension Results

| # | Dimension | Result |
|---|-----------|--------|
| 1 | Requirement coverage | **PASS** — ID-02, OPS-01, OPS-02 in frontmatter + tasks |
| 2 | Task completeness | **FAIL** — Plan 03 frontmatter unparsable; tool reports missing required fields |
| 3 | Dependency correctness | **PASS** (content) / blocked at tool level for 03 |
| 4 | Key links planned | **PASS** (content in 03 present but outside closed YAML fence) |
| 5 | Scope sanity | **PASS** with warnings — 02 files=10; 03 tasks=4 |
| 6 | Verification derivation | **PASS** — user-observable / gate-testable must_haves |
| 7 | Context compliance | **PASS** — D-01..D-15 mapped; deferred excluded |
| 7b | Scope reduction | **PASS** — no silent v1/static reduction of locked decisions |
| 7c | Architectural tier compliance | **PASS** — matches RESEARCH responsibility map |
| 8 | Nyquist compliance | **PASS** — VALIDATION exists; all tasks have `<automated>` |
| 9 | Cross-plan data contracts | **PASS** — string/default flips; no conflicting transforms |
| 10 | CLAUDE.md compliance | **SKIPPED** (no project `./CLAUDE.md`; AGENTS.md honored) |
| 11 | Research resolution | **PASS** — Open Questions (RESOLVED) Q1–Q3 |
| 12 | Pattern compliance | **PASS** — PATTERNS/UI-SPEC cited |
| — | Verify command format | **PASS** — prior `\|\| true` / `2>/dev/null` hygiene fixed |
| — | Numeric/factual claim authority | **PASS** |

---

## Dimension 8: Nyquist Compliance

```bash
ls .planning/phases/08-quiet-fork-rebrand-polish/*-VALIDATION.md
# → 08-VALIDATION.md present (plan-time scaffold)
```

| Task | Plan | Wave | Automated Command | Status |
|------|------|------|-------------------|--------|
| T1 | 01 | 1 | `cargo test … p8_telemetry --list` + execute + VALIDATION exists | ✅ |
| T2 | 01 | 1 | pager `p8_` list+run; model + home_isolation | ✅ |
| T1–T3 | 02 | 2 | `p8_cli_brand`, `p8_welcome`, `p8_` + billing + model | ✅ |
| T1 | 03 | 2 | `p8_oauth_return` | ✅ |
| T1b | 03 | 2 | `rg` Open bum / not Open Grok Build | ✅ |
| T2–T3 | 03 | 2 | `p8_minimal_welcome`, `p8_bin_banner` | ✅ |
| T1–T3 | 04 | 3 | pager-bin + update + settings filters | ✅ |
| T1–T3 | 05 | 4 | `p8_feedback`, shell feedback/telemetry, `p8_` bin (sentry) | ✅ |
| T1–T3 | 06 | 5 | VALIDATION content; PHASE-GATE content; model + isolation | ✅ |

| Check | Status |
|-------|--------|
| 8e VALIDATION.md exists | ✅ |
| 8a Automated verify | ✅ all tasks |
| 8b Feedback latency | ✅ no `--watch`; cargo targeted |
| 8c Sampling continuity | ✅ |
| 8d Wave 0 MISSING links | ✅ none |

**Overall Dimension 8: ✅ PASS**

---

## Dimension 11: Research Resolution

`08-RESEARCH.md` § `## Open Questions (RESOLVED)`:

1. **Q1** remote telemetry/feedback — RESOLVED (Plan 05 hard-prefer Disabled/false)  
2. **Q2** agent/system prompt naming — RESOLVED (out of chrome gate)  
3. **Q3** user-guide docs — RESOLVED (scoped to runtime-extracted help)

**PASS**

---

## Blockers (must fix before execute)

### 1. [task_completeness] Plan 08-03 frontmatter not closed / unparsable

- **Severity:** BLOCKER  
- **Plan:** `08-03`  
- **Description:** Line ends with `pattern: "bum \\(pager\\)"---` — the document fence is not on its own line. YAML frontmatter never closes; `gsd-tools query verify.plan-structure` reports missing `phase`, `plan`, `type`, `wave`, `depends_on`, `files_modified`, `autonomous`, `must_haves`. Execute tooling that reads frontmatter will not see `depends_on: [08-01]`, requirements, or must_haves.  
- **Fix:** Split the fence onto its own line:

```yaml
      pattern: "bum \\(pager\\)"
---
```

Re-run `node gsd-tools.cjs query verify.plan-structure …/08-03-PLAN.md` → expect `valid: true`.

---

## Warnings (should fix; not blocking after Plan 03 repair)

### 1. [scope_sanity] Plan 02 lists 10 files

- **Severity:** WARNING  
- **Plan:** 02  
- **Fix:** Optional split clap/hero vs picker/billing if executor context is tight.

### 2. [scope_sanity] Plan 03 has 4 tasks

- **Severity:** WARNING  
- **Plan:** 03  
- **Description:** Task count at warning threshold after Task 1b residual rebrand.  
- **Fix:** Optional merge Task 1+1b if desired; not required.

### 3. [nyquist / gate consistency] Plan 06 softens `p8_sentry` as “if present”

- **Severity:** WARNING (informational after Plan 05 mandate)  
- **Plan:** 06 Task 2  
- **Description:** Plan 05 Task 3 **requires** `p8_sentry_*`; Plan 06 lists `p8_sentry if present`. Prefer unconditional subgroup in PHASE-GATE once 05 lands.  
- **Fix:** List `p8_sentry` as required subgroup in Plan 06 action + VALIDATION row (already planned in VALIDATION map).

---

## Structured Issues

```yaml
issues:
  - plan: "08-03"
    dimension: task_completeness
    severity: blocker
    description: "YAML frontmatter not closed — last key_links pattern line ends with glued '---'; verify.plan-structure valid:false / missing all required frontmatter fields."
    fix_hint: "Put closing --- on its own line after pattern: \"bum \\\\(pager\\\\)\""

  - plan: "08-02"
    dimension: scope_sanity
    severity: warning
    description: "Plan 02 lists 10 files (warning threshold)."
    fix_hint: "Optional split if context budget is tight."

  - plan: "08-03"
    dimension: scope_sanity
    severity: warning
    description: "Plan 03 has 4 tasks (warning threshold)."
    fix_hint: "Optional merge OAuth + Open bum residual."

  - plan: "08-06"
    task: 2
    dimension: verification_derivation
    severity: warning
    description: "PHASE-GATE action says p8_sentry if present; Plan 05 mandates p8_sentry_*."
    fix_hint: "Require p8_sentry subgroup in gate after Plan 05."
```

---

## What already works (do not regress)

- **Prior Nyquist/research blockers fixed** at plan time.
- **Goal-backward:** three ROADMAP success criteria owned by concrete plans.
- **Green-only protocol:** Plan 01 scaffolds green `p8_` only; product asserts in 02–05.
- **OPS-01 traps:** Plan 04 covers chokepoints from RESEARCH.
- **OPS-02 traps:** Plan 05 feedback short-circuit + Sentry force-off + mandatory unit.
- **ID-02 residual:** Open Grok Build → Open bum in Plan 03 Task 1b.
- **D-02 protected;** deferred ideas excluded; threat models present.

---

## Must-fix before `/gsd:execute-phase 8`

1. **Repair `08-03-PLAN.md` frontmatter fence** (one-line fix).  
2. Confirm `verify.plan-structure` → `valid: true` for all six plans.  
3. Re-run plan-check (expect **PASS** with warnings only).

---

## Recommendation

**Return to planner** for the single Plan 03 frontmatter fix. Do **not** execute until structure validates.

After fix: expected **PLAN_CHECK_PASS** (warnings only: file/task counts, optional p8_sentry gate wording).
