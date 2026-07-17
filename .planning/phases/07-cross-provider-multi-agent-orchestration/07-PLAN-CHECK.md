# Phase 7 Plan Check — Cross-provider multi-agent orchestration

**Checked:** 2026-07-17  
**Plans verified:** 6 (`07-01` … `07-06`)  
**Status:** **ISSUES FOUND** (not PLAN_CHECK_PASS)

**Phase goal:** Parent on one provider can spawn a child on another with correct model, effort, credentials, and backend routing.

---

## Coverage Summary

| Requirement | Plans | Status |
|-------------|-------|--------|
| AGENT-01 same-provider regression | 01 (anchor), 06 (gate) | Covered (verify filters thin — see warning) |
| AGENT-02 cross-provider explicit model | 01, 03, 04, 05 | Covered |
| AGENT-03 reasoning effort on spawn | 01, 02, 03, 05 | Covered (Plan 03 verify gap — blocker) |
| AGENT-04 child route/credentials isolation | 01, 03, 05 | Covered with D-12 reduction risk — blocker |
| AGENT-05 fail-closed missing provider + bg preflight | 01, 03, 04, 05 | Covered (eager bg + shell authority) |
| AGENT-06 NL orchestration surface | 01, 02, 05, 06 | Covered per CONTEXT (schema/docs + automated; live E2E → Phase 9) |

| ROADMAP success criterion | Delivered by |
|---------------------------|--------------|
| 1 Same-provider no regression | 01 + 06 |
| 2 Spawn child on different provider | 03 + 05 |
| 3 Effort on launch | 02 + 03 |
| 4 Child model→provider→creds→backend | 03 + 05 |
| 5 NL orchestration + fail-closed login | 02 + 04 + 05 (automated path) |

### Locked decisions (CONTEXT D-01..D-16)

| Decision | Primary plan(s) | Notes |
|----------|-----------------|-------|
| D-01 omit model inherit | 01, 04, 06 | Preserve existing; no new product path |
| D-02 catalog-wide explicit model | 03 | Tool fail-closed unknown |
| D-03 expose + wire reasoning_effort | 02 | Task schema + overrides |
| D-04 invalid effort reject | 02 + 03 | Dual boundary Task + shell Tool |
| D-05 spawn-time usable check | 03 + 04 | Shell authoritative + Task eager bg |
| D-06 no parent fallback | 03 + 05 | Unknown model + missing slot |
| D-07 login CLI hint | 03 + 04 + 05 | `bum login --provider` |
| D-08 same-provider no extra friction | 03 + 04 | |
| D-09 child SamplingConfig from child model | 03 + 05 | |
| D-10 child bearer only | 05 | Isolation proofs |
| D-11 parent model unchanged | 03 + 05 | |
| D-12 dual fake-token proofs | 05 | **Minimum path softens Authorization request** |
| D-13 NL schema/docs | 02 | |
| D-14 AGENT-01 hard gate | 06 | |
| D-15 resume model pin | 03 + 06 | |
| D-16 out of scope | 06 + all scope boundaries | No deferred creep |

### Dependency / wave graph

```
07-01 (wave 1, deps [])
  ├─ 07-02 (wave 2)
  └─ 07-03 (wave 2)
       └─ 07-04 (wave 3, deps 02+03)
            └─ 07-05 (wave 4, deps 03+04)
                 └─ 07-06 (wave 5, deps 02–05)
```

- Acyclic; wave numbers consistent with `depends_on`.
- Parallel wave 2 (02 ‖ 03) has **no** shared `files_modified` conflict.
- Sequential ownership on `task/mod.rs` (01→02→04) and `cross_provider_subagent.rs` (01→03→04→05) is sound.

### Plan structure (gsd-tools)

All 6 plans: `valid: true`, 2 tasks each, files/action/verify/done present.

---

## Dimension Results

| # | Dimension | Result |
|---|-----------|--------|
| 1 | Requirement coverage | PASS (frontmatter + tasks map AGENT-01..06) |
| 2 | Task completeness | PASS structure; verify integrity issues below |
| 3 | Dependency correctness | PASS |
| 4 | Key links planned | PASS (Task→overrides, shell gate, eager preflight, dual-token isolation) |
| 5 | Scope sanity | PASS (2 tasks/plan; focused files) |
| 6 | Verification derivation | Mostly PASS; Plan 03/04 verify holes |
| 7 | Context compliance | PASS decisions present; D-12 delivery softened |
| 7b | Scope reduction | **FAIL** — Plan 05 D-12 minimum path |
| 7c | Architectural tier | PASS — API/Backend matches responsibility map; no TUI modal required |
| 8 | Nyquist compliance | **FAIL** — `07-VALIDATION.md` missing (8e gate) |
| 9 | Cross-plan data contracts | PASS — effort + gate semantics compatible |
| 10 | CLAUDE.md / AGENTS.md | PASS — Rust crate targets, no forbidden stack |
| 11 | Research resolution | **FAIL** — Open Questions not marked RESOLVED |
| 12 | Pattern compliance | PASS — plans reference PATTERNS analogs (Task schema, model_switch gate, provider_routing) |
| — | Deferred / scope creep | PASS — no workflow engine, cost dashboards, Phase 8/9 creep |

### Dimension 8 detail

- `workflow.nyquist_validation: true`
- RESEARCH has Validation Architecture
- `ls …/*-VALIDATION.md` → **missing**
- Per check 8e: **BLOCKING FAIL** — skip 8a–8d as formal gate (note: individual plan `<automated>` blocks exist and are mostly well-formed)

### Dimension 11 detail

RESEARCH `## Open Questions` (no `(RESOLVED)` suffix). Three questions lack inline `RESOLVED` markers even though Plans 03–04 implement the recommendations (typed error optional; Tool hard-fail unsupported effort; `TaskProviderCredentialGate` resource).

---

## Blockers (must fix before execute)

### 1. [nyquist_compliance] VALIDATION.md not found for phase 07

- **Plan:** phase-level (should land before execute; not only as Plan 06 product)
- **Severity:** blocker
- **Description:** Dimension 8e requires `07-VALIDATION.md` at plan-check time. Prior phases (01–06) carried a validation scaffold into execution; Plan 06 alone “writes” VALIDATION late, so waves 1–5 lack an authoritative Nyquist map during execution.
- **Fix:** Add `.planning/phases/07-cross-provider-multi-agent-orchestration/07-VALIDATION.md` now (draft from RESEARCH Validation Architecture + planned `p7_` filters). Plan 06 may **update** filter names after greening (Phase 6 pattern: update existing, not create-only).

### 2. [research_resolution] RESEARCH Open Questions not marked resolved

- **Plan:** `07-RESEARCH.md`
- **Severity:** blocker
- **Description:** Section is `## Open Questions` without `(RESOLVED)`. Q1–Q3 have recommendations but no resolution markers.
- **Fix:** Rename to `## Open Questions (RESOLVED)` and mark each question RESOLVED with the plan choices:
  1. Tool/string `send_failure` + login suggestion; optional distinct ACP code not required
  2. Tool provenance: hard-fail explicit effort on models without reasoning-effort support
  3. Inject `TaskProviderCredentialGate` Resources `Fn` parallel to `TaskModelValidator`

### 3. [scope_reduction] Plan 05 allows resolve-only minimum that skips D-12 request Authorization

- **Plan:** `07-05-PLAN.md` Task 1
- **Severity:** blocker
- **Description:** CONTEXT D-12 requires automated proofs that **child request Authorization + base URL** match the child provider (dual fake tokens). Plan 05 action allows “Minimum: resolve_model_override_to_config base_url + credential key_prefix” and treats mock HTTP Authorization as optional “Best”. Executor can green without proving request-level Authorization.
- **Fix:** Make at least one direction (preferably both) assert **mock HTTP Authorization** (or equivalent outbound request header) against fixture tokens, reusing `provider_routing` patterns. Resolve-only key_prefix may remain as a unit complement, not the sole AGENT-04/D-12 bar. Align must_haves wording so Authorization is required, not interchangeable with key_prefix alone.

### 4. [task_completeness / verify integrity] Plan 03 Task 2 verify swallows lib `p7_` failures

- **Plan:** `07-03-PLAN.md` Task 2
- **Severity:** blocker
- **Description:** Verify is:
  ```bash
  cargo test -p xai-grok-shell --lib p7_ -- --nocapture 2>/dev/null || true
  cargo test … p7_tool …
  cargo test … reasoning_effort_explicit …
  ```
  Behavior names `p7_invalid_effort_tool_provenance_fails_closed` — that name is **not** matched by filter `p7_tool` (substring is `effort_tool`, not `p7_tool`). Combined with `|| true`, invalid-effort fail-closed unit tests can stay RED while verify still exits 0 if only `p7_tool_*` + role regression pass.
- **Fix:** Remove `|| true` on the required path. Either rename tests to `p7_tool_*` and run a single fail-closed filter, or require both:
  - `cargo test -p xai-grok-shell --lib p7_invalid_effort` (or `--lib p7_` without swallow)
  - `cargo test -p xai-grok-shell --test cross_provider_subagent p7_tool`

---

## Warnings (should fix; execution may still work after blockers)

### 1. [task_completeness] Plan 04 Task 2 verify ends with `|| true`

- **Plan:** `07-04-PLAN.md` Task 2
- **Severity:** warning
- **Description:** Second half of verify can fully fail (`lib p7_` and `TaskProviderCredentialGate` filters) and still pass if `cross_provider_subagent p7_` alone is green (possibly from Plan 03 tests).
- **Fix:** Require a named injection filter (e.g. `p7_credential_gate` / `p7_eager_injection`) with discovery ≥1 and no trailing `|| true`.

### 2. [requirement_coverage] AGENT-01 roles/personas not in Plan 06 automated verify command

- **Plan:** `07-06-PLAN.md` Task 1–2
- **Severity:** warning
- **Description:** must_haves and action mention roles/personas; verify only runs `reasoning_effort_explicit` + `resume_model_pinning`. Personas/roles could regress without phase-gate detection.
- **Fix:** Add concrete existing filters (or thin `p7_same_provider_*` wrapping them) to VALIDATION + PHASE-GATE discover list once test names are confirmed from the subagent suite.

### 3. [dependency_correctness / docs] Plan 01 labeled “Wave 0” but frontmatter `wave: 1`

- **Plan:** `07-01-PLAN.md`
- **Severity:** warning
- **Description:** Content says Wave 0 RED harness; YAML `wave: 1`. Plan 06 wave table uses Wave1=01. Consistent with deps, but “Wave 0” wording may confuse Nyquist docs.
- **Fix:** Pick one: call Plan 01 wave 1 RED harness in prose, or set `wave: 0` if tooling supports it — align VALIDATION wave table.

### 4. [verification_derivation] AGENT-06 live multi-turn E2E not in phase

- **Plan:** 05 + 06 + CONTEXT
- **Severity:** info / warning
- **Description:** ROADMAP criterion 5 reads E2E-like; CONTEXT + Plan 05 correctly limit to schema/docs + automated spawn/effort/isolation (Phase 9 for live matrix). Acceptable if VALIDATION states this explicitly so verifier does not demand live dual-login NL.

### 5. [scope_sanity] Plan 04 omit-model preflight has escape hatch

- **Plan:** `07-04-PLAN.md` discretion
- **Severity:** warning
- **Description:** “document limitation and still gate all explicit cross-provider models” if parent model resource unavailable. Explicit models stay gated (main path). Ensure SUMMARY documents any inherit-only skip so D-05 same-provider-if-missing is not silently dropped.

---

## Structured Issues

```yaml
issues:
  - plan: null
    dimension: nyquist_compliance
    severity: blocker
    description: "07-VALIDATION.md missing (Dimension 8e gate). Phase cannot claim Nyquist sampling continuity during waves 1–5."
    fix_hint: "Create 07-VALIDATION.md now from RESEARCH Validation Architecture + planned p7_ filters; Plan 06 updates names after green."

  - plan: null
    dimension: research_resolution
    severity: blocker
    description: "07-RESEARCH.md ## Open Questions lacks (RESOLVED) and inline RESOLVED markers for Q1–Q3."
    fix_hint: "Mark section and each question RESOLVED with Plan 03/04 decisions."

  - plan: "07-05"
    dimension: scope_reduction
    severity: blocker
    task: 1
    description: "D-12 requires child request Authorization + base_url dual-fake proofs; Plan 05 allows resolve-only key_prefix minimum."
    fix_hint: "Require mock HTTP Authorization assert for ≥1 direction (prefer both); keep resolve unit as complement only."

  - plan: "07-03"
    dimension: task_completeness
    severity: blocker
    task: 2
    description: "Verify uses lib p7_ || true and p7_tool filter that may not match p7_invalid_effort_* — effort fail-closed can stay RED with green verify."
    fix_hint: "Fail-closed verify on named effort + unknown-model tests; remove || true."

  - plan: "07-04"
    dimension: task_completeness
    severity: warning
    task: 2
    description: "Verify trailing || true can pass without injection/lib credential-gate tests."
    fix_hint: "Named p7_ injection filter with discovery ≥1; no swallow."

  - plan: "07-06"
    dimension: requirement_coverage
    severity: warning
    description: "AGENT-01 roles/personas mentioned in prose but not in Task 1/2 automated verify filters."
    fix_hint: "Add concrete role/persona (or p7_same_provider) filters to VALIDATION + phase gate."

  - plan: "07-01"
    dimension: dependency_correctness
    severity: warning
    description: "Prose Wave 0 vs frontmatter wave: 1 inconsistency."
    fix_hint: "Align naming with VALIDATION wave table."
```

---

## What already looks strong (do not regress in revision)

- Dual-layer AGENT-05: shell `handle_subagent_request` authority (03) + Task eager preflight before bg started notice (04) — closes RESEARCH Pitfall 1.
- No parent-fallback for Tool unknown model (03) — closes critical pitfall.
- Effort schema + invalid reject at Task (02) and Tool shell fail-closed (03).
- Dual-direction isolation intent + no-wrong-backend missing-slot (05).
- Explicit D-16 / deferred exclusions; no workflow engine or Phase 8/9 creep.
- Threat models present on all plans; fixture-token-only policy consistent.
- 2 tasks/plan, clear key_links, pattern analogs referenced.

---

## Recommendation

**4 blocker(s)** require planner revision before `/gsd:execute-phase 07`.

Priority order:
1. Create `07-VALIDATION.md` scaffold  
2. Resolve RESEARCH open questions markers  
3. Harden Plan 05 D-12 Authorization requirement  
4. Fix Plan 03 Task 2 verify (and preferably Plan 04 Task 2)

Then re-run plan-check. Do **not** execute until re-check returns **PLAN_CHECK_PASS**.
