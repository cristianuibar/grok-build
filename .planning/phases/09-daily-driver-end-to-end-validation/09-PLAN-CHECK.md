# Phase 9 — Plan Check (goal-backward)

**Phase:** 09 — Daily-driver end-to-end validation  
**Checked:** 2026-07-17  
**Plans:** 5 (`09-01` … `09-05`)  
**Checker:** gsd-plan-checker (revision gate)  
**Verdict:** **PASS**

---

## Phase goal (source of truth)

From ROADMAP.md:

> bum is usable as the default coding agent for real work across both providers and cross-provider agents

**Requirements:** OPS-03, OPS-04, OPS-05, OPS-06  

**Success criteria:**

1. Real coding session on xAI after xAI login (tools, edit, shell)
2. Real coding session on GPT-5.6 after Codex login (supported tools)
3. Same-session switch Grok ↔ GPT-5.6 without restart
4. Parent-Grok→Codex-child **and** parent-Codex→Grok-child succeed under dual login

**Locked methodology (CONTEXT):** hybrid gate = automated residual (fixtures) **∧** signed live dual-login UAT; live OPS rows never green on fixtures alone.

---

## Coverage matrix

| Requirement / criterion | Primary delivery | Auto residual | Live UAT | Evidence close | Status |
|-------------------------|------------------|---------------|----------|----------------|--------|
| OPS-03 / SC-1 xAI productive turn | 09-04 T2 human | residual auth/routing/home + p9_ (01–02) | 09-03 UAT §OPS-03 | 09-05 VERIFICATION | Covered |
| OPS-04 / SC-2 GPT-5.6 productive turn | 09-04 T2 human | dual-slot + routing residual | 09-03 UAT §OPS-04 + gaps | 09-05 | Covered |
| OPS-05 / SC-3 mid-session switch | 09-04 T2 human | p6_dual_login + switch_changes (02, 05) | 09-03 UAT §OPS-05 | 09-05 | Covered |
| OPS-06 / SC-4 both spawn dirs | 09-04 T2 human | p7_isolation both dirs + eager/spawn (02, 05) | 09-03 UAT both dirs | 09-05 clears P7 deferred | Covered |
| Hybrid both halves | 09-02 auto + 09-03/04 human + 09-05 formula | ✅ | ✅ | ✅ | Covered |
| D-16 no fixture-only live PASS | 01–05 D-16 language; 04 checkpoint; 05 gate formula | — | required | refuse fixture waive | Covered |
| Secrets hygiene | 03 preflight/UAT; 04 redact; 05 scan | fixture-only auto | redacted notes | VERIFICATION scan | Covered |
| D-14 no deferred PROJECT outs | scope boundaries all plans | — | blocker-only product | exclusions restated | Covered |

**All roadmap requirement IDs appear in every plan’s `requirements` frontmatter.** Live proof is concentrated in Plan 04 (human) + Plan 05 (evidence); automated residual in 01–02/05. This is correct for a validation-first hybrid phase.

---

## Wave / dependency graph

```text
Wave 1: 09-01  (depends_on: [])
Wave 2: 09-02  (depends_on: 09-01)  ║  09-03 (depends_on: 09-01)
Wave 3: 09-04  (depends_on: 09-02, 09-03)   ← human after auto gate + UAT runbook
Wave 4: 09-05  (depends_on: 09-02, 09-04)   ← hybrid close after UAT
```

| Check | Result |
|-------|--------|
| Acyclic | ✅ |
| No future refs | ✅ |
| Wave numbers consistent | ✅ |
| Human plan 04 after UAT (03) + auto (02) | ✅ |
| Plan 05 after live UAT (04) | ✅ (03 transitive via 04) |

---

## Plan structure summary

| Plan | Wave | Tasks | Files (declared) | Autonomous | Structure |
|------|------|-------|------------------|------------|-----------|
| 01 | 1 | 2 (tdd + auto) | VALIDATION + p9_ test file | true | Valid — Files/Action/Verify/Done |
| 02 | 2 | 2 | PHASE-GATE + VALIDATION | true | Valid |
| 03 | 2 | 2 | UAT + preflight + VALIDATION | true | Valid |
| 04 | 3 | 3 (auto + human-verify + auto) | UAT | false | Valid — blocking human checkpoint |
| 05 | 4 | 3 | VERIFICATION + VALIDATION + PHASE-GATE | true | Valid |

Scope: 2–3 tasks/plan — within budget (no 5+ task plans).

---

## Dimension results

### 1. Requirement coverage — ✅ PASS

OPS-03..06 each have:

- Automated residual inventory (VALIDATION finalize 01, PHASE-GATE 02/05)
- Required live UAT rows (03 write, 04 execute)
- Hybrid evidence (05 VERIFICATION + nyquist only if both halves)

Roadmap SC 1–4 map 1:1 to OPS rows in Plan 05 Task 2 truths table.

### 2. Task completeness — ✅ PASS

All `auto` tasks have `<files>`, `<action>`, `<verify>` (with `<automated>`), `<done>`.  
Plan 04 Task 2 is `checkpoint:human-verify` with `how-to-verify`, `human-check`, `resume-signal`.

### 3. Dependency correctness — ✅ PASS

See graph above. No cycles; wave assignment matches `depends_on`.

### 4. Key links planned — ✅ PASS

| Link | Planned in |
|------|------------|
| p9_ → PHASE-GATE discover | 01 key_links → 02 action |
| Prior p6/p7/p8 → PHASE-GATE P0/P1 | 02 action + must_haves |
| UAT OPS rows → VERIFICATION live column | 03 → 05 key_links |
| PHASE-GATE re-run → VERIFICATION spot-checks | 05 Task 1→2 |
| UAT fail → in-phase blocker fix | 04 Task 3 + fix policy table |
| Phase 7 deferred live E2E → OPS-06 clear | 05 Task 2 |

### 5. Scope sanity — ✅ PASS

2–3 tasks/plan; validation-first; product code only on OPS blockers (D-03/D-13). No soak, full catalog, install channel, crate rename.

### 6. Verification derivation — ✅ PASS

`must_haves.truths` are user-observable / gate-observable (live productive turns, hybrid GREEN formula, no secrets), not “library installed” trivia. Artifacts and key_links support truths.

### 7. Context compliance — ✅ PASS

| Locked decision | Implementation |
|-----------------|----------------|
| D-01 hybrid proof | 01 inventory + 02 auto + 03/04 human + 05 close |
| D-02 both halves for GREEN | 02 human placeholder; 05 HARD gate formula |
| D-03/D-13 in-phase blockers only | 04 fix policy table |
| D-04 live under BUM_HOME; no CI secrets | 03/04 preflight |
| D-05 OPS-03/04 read+edit/shell | 03 tables + 04 how-to-verify |
| D-06 same-process switch | 03/04 OPS-05 |
| D-07 both spawn dirs | 03/04 OPS-06 mandatory both rows |
| D-08 default models | grok-build + gpt-5.6-sol |
| D-09 evidence who/when/models | UAT sign-off + VERIFICATION |
| D-10 capability gaps honest | 03/04/05 |
| D-11 green-only auto + p9_ | 01 protocol + 02 |
| D-12 secrets | redaction + scan verifies |
| D-14 outs excluded | all plan scope boundaries |
| D-15 Cristian runs live UAT | 04 checkpoint |
| D-16 no fixture waive | repeated; 05 refuse |

Deferred ideas (install channel, crate rename, mass GROK_*, custom workflows, full catalog, IdP, soak) **not** planned.

### 7b. Scope reduction — ✅ PASS

No silent v1/static/placeholder reduction of locked OPS live bar. Hybrid is full dual-proof, not auto-only.

### 7c. Architectural tier compliance — ✅ PASS

Matches RESEARCH Responsibility Map: Cargo/CI fixtures for auto; shell auth + IdP for live OAuth; agent/tools for productive turns; TUI for switch; planning artifacts for evidence; product fixes at broken seam only.

### 8. Nyquist compliance — ✅ PASS

| Check | Result |
|-------|--------|
| 8e VALIDATION.md exists | ✅ `09-VALIDATION.md` present (draft dual map) |
| 8a automated verify on tasks | ✅ all auto tasks; human checkpoint has automated shell + human-check |
| 8b feedback latency | ✅ targeted cargo filters; no `--watchAll`; no sole full-workspace gate |
| 8c sampling continuity | ✅ no 3 consecutive implementation tasks without automated verify |
| 8d Wave 0 completeness | ✅ VALIDATION present; p9_ created Plan 01; UAT Plan 03; PHASE-GATE Plan 02 |

**Nyquist task map (abbrev):**

| Task | Plan | Wave | Automated | Status |
|------|------|------|-----------|--------|
| p9_ dual-slot smoke | 01-T1 | 1 | `cargo test … cross_provider_subagent p9_` discover≥1 | ✅ |
| Finalize VALIDATION | 01-T2 | 1 | rg dual map / p9_ | ✅ |
| Write PHASE-GATE | 02-T1 | 2 | rg discover/p6/p7/p8/human | ✅ |
| Execute auto half | 02-T2 | 2 | discover chain (subset — see W1) | ✅ |
| Write UAT | 03-T1 | 2 | rg OPS/Sign-off | ✅ |
| Preflight script | 03-T2 | 2 | bash -n + secret scan | ✅ |
| Build + residual | 04-T1 | 3 | build bum + discover residual | ✅ |
| Human UAT matrix | 04-T2 | 3 | header rg + human-check | ✅ |
| Blocker fix / redact | 04-T3 | 3 | UAT scan + p9_ | ✅ |
| Full PHASE-GATE re-run | 05-T1 | 4 | full P0/P1/p9 discover chain | ✅ |
| VERIFICATION | 05-T2 | 4 | file + secret scan | ✅ |
| Nyquist close | 05-T3 | 4 | dual artifacts + human/GREEN | ✅ |

Live measurement (research host): `p6_dual_login` n=3, `p7_isolation_codex_parent_grok` n=1, `p8_telemetry` n=3 — prior filters exist for gate composition.

### 9. Cross-plan data contracts — ✅ PASS

Shared entities are test filter names and doc sections (not streaming transforms). Plan 01 records actual `p9_` names → Plan 02 discovers them; Plan 03 UAT → Plan 04 fill → Plan 05 evidence. No conflicting sanitize/strip contracts.

### 10. CLAUDE.md / AGENTS.md compliance — ✅ PASS

Stays on Rust workspace; per-crate cargo filters; no new packages; validation-first aligns with fork evolution; no mass rename / install channel.

### 11. Research resolution — ⚠️ WARNING (not blocker)

`09-RESEARCH.md` still has `## Open Questions` **without** `(RESOLVED)` suffix. Each item has a **Recommendation** that Plans 01–05 already implement (P0/P1 subset, 1–3 p9_, TUI prefer, UAT+VERIFICATION evidence).  

**Fix (optional hygiene):** rename to `## Open Questions (RESOLVED)` and prefix each answer with `RESOLVED:`.

### 12. Pattern compliance — ✅ PASS

Plans cite 07/08 PHASE-GATE discover pattern, `cross_provider_subagent.rs` for p9_, Phase 2/3 UAT elevated to required, 07/08 VERIFICATION analogs — matches PATTERNS.md File Classification.

### Verify command format sanity — ✅ PASS (with W2)

- Chains use `&&` / `set -euo pipefail`
- Discover uses `grep -c || true` then `test n -ge 1` (fail-closed on empty) — not the swallowed-pass anti-pattern
- No `pnpm`/`^` package-list anchors
- No watch mode

### Cargo hygiene (phase-specific checklist) — ✅ PASS

| Rule | Plans honor |
|------|-------------|
| One TESTNAME filter | stated HARD; discover one filter per call |
| `&&` chains only | verify blocks use `&&` |
| Green-only / no intentional-red | 01 expected-green protocol |
| No sole unfiltered `--lib` / workspace | 02 forbidden notes |
| Fixture-only automated network | D-04/D-12 throughout |

---

## Warnings (non-blocking)

### W1 — Plan 02 Task 2 verify is a subset of the documented automated half

**Dimension:** nyquist / task_completeness (quality)  
**Severity:** warning  

**Issue:** Task 2 *action* requires full PHASE-GATE automated half (P0/P1 + p9_). Task 2 *verify* only asserts:

- p9_, p6_dual_login, p7_isolation (+ both dirs), p7_eager, p8_telemetry, home_isolation  

Missing from verify vs Task 1 inventory / Plan 05 full chain:

- `p6_missing_provider`
- `switch_changes_next_sample_route`
- `p7_spawn_missing_provider`
- `p7_parent_model`
- `p8_no_auto_update`, `p8_sentry`

**Why not blocker:** Plan 05 Task 1 re-runs the **full** inventory as gate close; Plan 02 action still obligates full run.

**Optional fix in `09-02-PLAN.md` Task 2 `<automated>`:** align with Plan 05 Task 1 discover chain (or invoke “source PHASE-GATE automated section”).

### W2 — `home_isolation` in Plan 02 Task 2 verify lacks discover n≥1

**Dimension:** cargo hygiene  
**Severity:** warning  

Last step is bare `cargo test -p xai-grok-pager-bin --test home_isolation` without list count ≥1. Prefer `discover xai-grok-pager-bin hermetic --test home_isolation` (or equivalent filter) for consistency with VALIDATION locked rules.

### W3 — Wave 2 concurrent edits to `09-VALIDATION.md`

**Dimension:** dependency_correctness  
**Severity:** warning  

Plans 02 and 03 both depend only on 01 and both modify VALIDATION (auto status vs human Instructions links). Parallel execute could conflict.

**Optional fix:** `09-03` `depends_on: [09-01, 09-02]` **or** move VALIDATION human-link update entirely into Plan 02 after UAT exists, **or** document sequential execute order 02→03 within wave.

### W4 — Plan 04 `files_modified` omits conditional product paths

**Dimension:** task_completeness  
**Severity:** warning  

Task 3 may touch auth/routing/chrome/tools seams, but frontmatter only lists `09-UAT.md`. Acceptable for conditional work; executor should expand SUMMARY `files_modified` if fixes land.

### W5 — RESEARCH Open Questions formal stamp

**Dimension:** research_resolution  
**Severity:** warning  

See Dimension 11. Recommendations already adopted; stamp RESOLVED for artifact hygiene.

---

## Blockers

**None.**

---

## Goal-backward conclusion

Starting hypothesis (“plans will not deliver”) is **disproved** for the phase goal:

1. Every OPS-03..06 / SC row has concrete tasks for live proof + residual + evidence.
2. Hybrid gate both halves are planned (02 auto; 03 runbook; 04 human; 05 formula).
3. D-16 is enforceable in UAT checkpoint + VERIFICATION scoring — cannot honest-green on fixtures alone.
4. Cargo hygiene and secrets rules are explicit and verified.
5. Wave order places human UAT after automated green + runbook.
6. No deferred PROJECT outs / scope reduction of the live bar.

Warnings improve sampling completeness and parallel-edit safety; they do not prevent goal achievement if executors follow `<action>` text and Plan 05 full re-run.

---

## Recommendation

**Proceed to `/gsd:execute-phase 09`.**

Optional pre-execute polish (not required for PASS):

1. Widen `09-02` Task 2 verify to full P0/P1 chain (W1).
2. Serialize Wave 2 VALIDATION edits (W3).
3. Mark RESEARCH Open Questions RESOLVED (W5).

---

## PLAN CHECK COMPLETE

**Verdict: PASS**

Plans verified: 5  
Blockers: 0  
Warnings: 5  
Issues requiring planner revision: none
