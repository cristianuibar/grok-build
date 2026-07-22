# Phase 6 Plan Checker Report

**Phase:** 06 — Mid-session switch & missing-provider gate  
**Checked:** 2026-07-17  
**Plans verified:** 6 (`06-01` … `06-06`)  
**Requirements:** MOD-03, MOD-06  
**Status:** **PASSED** (after checker fixes; warnings remaining, non-blocking)

---

## Goal-backward summary

| ROADMAP success criterion | Covering plans | Status |
|---------------------------|----------------|--------|
| 1. Switch models mid-session anytime; next turn uses new model (no CLI restart) | 05 (dual free switch + route), 03 (post-login retry + success scrollback), Phase 4 residual | Covered |
| 2. Missing usable credentials block switch + provider login prompt (not silent 401) | 01 (shell gate + typed ACP), 02 (QuestionView), 03 (Login now + deferred) | Covered |
| 3. Both providers logged in → free Grok↔GPT in one continuous session | 05 (`dual_login` / `free_switch`) | Covered |

| Requirement | Plans claiming `requirements:` | Status |
|-------------|--------------------------------|--------|
| MOD-03 | 03, 05, 06 | Covered |
| MOD-06 | 01, 02, 03, 04, 05, 06 | Covered |

---

## Dimension results

### Dimension 1: Requirement Coverage — PASS

- Both roadmap requirement IDs appear in plan frontmatter.
- MOD-06 shell authority (01) + TUI prompt (02) + recovery (03) + badge (04).
- MOD-03 free dual switch + next-sample route (05); confirmation reuse (03).
- No PROJECT.md phase-relevant requirement silently dropped.

### Dimension 2: Task Completeness — PASS

All 12 tasks (`auto` / `tdd`) have Files + Action + Verify + Done. Structure validation: `valid: true` for all six plans.

### Dimension 3: Dependency Correctness — PASS (after fix)

| Plan | Wave | depends_on | Consistent |
|------|------|------------|------------|
| 01 | 1 | [] | Yes |
| 02 | 2 | 01 | Yes |
| 04 | 2 | 01 | Yes (wave corrected 3→2) |
| 05 | 2 | 01 | Yes |
| 03 | 3 | 02, 04 | Yes (04 added so AuthComplete can call badge cache API) |
| 06 | 4 | 02, 03, 04, 05 | Yes |

No cycles. No same-wave file conflicts (02 vs 05 split shell/pager; 03 waits on 04).

### Dimension 4: Key Links Planned — PASS

Critical wiring planned:

- `model_switch::apply` → `ModelSwitchMissingProviderError::into_acp_error`
- Effect map_err → `SwitchModelError::MissingProvider` → QuestionView
- Login now → `deferred_model_switch` → AuthComplete → `Effect::SwitchModel`
- AppView usable cache → `build_model_items` `needs login` badge
- Dual fixtures → free switch → next-sample route

### Dimension 5: Scope Sanity — PASS

| Plan | Tasks | Files (approx) | Verdict |
|------|-------|----------------|---------|
| 01–06 | 2 each | 1–8 | Within budget (2–3 tasks target) |

### Dimension 6: Verification Derivation — PASS

must_haves are user-observable (block at switch, QuestionView, free dual switch, badge, deferred retry). Artifacts map to truths; key_links present.

### Dimension 7: Context Compliance — PASS

| Locked area (CONTEXT) | Plan coverage |
|----------------------|---------------|
| Gate at switch time | 01 |
| usable = refreshable OK; not xAI AuthManager alone for Codex | 01, 04 |
| Same-provider no extra friction | 05 |
| TUI modal + Login now + CLI fallback | 02, 03 |
| Auto-retry pending switch | 03 |
| ACP typed error | 01 |
| Full mixed catalog + light badge | 04 |
| No optimistic current on fail | 02 |
| Mid-turn allowed / next-turn only; keep history | 05 |
| Separate path from IncompatibleAgent | 01, 02 |
| `/model` / picker / ACP same policy | 01 shell authority |

**Deferred excluded:** Phase 7 subagents, Phase 8 rebrand, stock credential import — not in plans.

**Discretion used appropriately:** Codex Login now CLI-primary (A1), badge suffix MVP, modal chrome via QuestionView family.

**No scope reduction** of locked decisions inventing “v1 static” stubs for core gate/prompt.

### Dimension 7b: Scope Reduction — PASS

No “static for now / future enhancement” that shrinks MOD-03/MOD-06 delivery. Codex CLI-primary Login now is explicit CONTEXT discretion + RESEARCH A1, not silent reduction of the gate.

### Dimension 7c: Architectural Tier Compliance — PASS

Matches RESEARCH Architectural Responsibility Map:

- Gate + usable query + ACP error → shell (API)
- QuestionView / deferred / badge → pager (client)
- No auth policy in pure dispatch or browser-only gate

### Dimension 8: Nyquist Compliance — PASS (after checker fix)

| Check | Result |
|-------|--------|
| 8e VALIDATION.md exists | Was **missing** → checker seeded `06-VALIDATION.md` |
| 8a `<automated>` on tasks | All 12 tasks have automated verify |
| 8b Feedback latency | Narrow cargo filters; no `--watchAll` |
| 8c Sampling continuity | Waves 1–3 implementation tasks all verified |
| 8d Wave 0 MISSING links | No `MISSING` automated placeholders |

| Task | Plan | Wave | Automated Command | Status |
|------|------|------|-------------------|--------|
| T1–T2 | 01 | 1 | shell missing_provider / model_switch_missing_provider* | ✅ |
| T1–T2 | 02 | 2 | pager missing_provider / keep_current / incompatible_agent | ✅ |
| T1–T2 | 05 | 2 | provider_routing dual_login / free_switch / byok / history | ✅ |
| T1–T2 | 03 | 3 | login_now / deferred / auth_complete | ✅ |
| T1–T2 | 04 | 2 | provider_auth_usable / needs_login / build_model_items | ✅ |
| T1–T2 | 06 | 4 | VALIDATION file + aggregate cargo filters | ✅ |

Overall: ✅ PASS

### Dimension 9: Cross-Plan Data Contracts — PASS

Shared types (`ModelSwitchMissingProviderError`, `deferred_model_switch`, usable bools) flow producer→consumer with compatible shapes. No strip/sanitize conflict.

### Dimension 10: CLAUDE.md Compliance — SKIPPED (no `./CLAUDE.md`)

AGENTS.md / stack conventions respected (Rust crates, cargo test per-crate, no new packages).

### Dimension 11: Research Resolution — PASS (after checker fix)

`## Open Questions (RESOLVED)` with inline resolutions aligned to Plans 01/03/04.

### Dimension 12: Pattern Compliance — PASS

Plans cite PATTERNS analogs (IncompatibleAgent twin, QuestionView, deferred_model_switch, build_model_items). Shared patterns (TEA purity, typed ACP, no token logs) reflected in threat models and actions.

### Verify command format — PASS

No `pnpm ls | grep -E '^…'`, no `2>/dev/null || echo "0"` false-green patterns, no hard-coded unmeasured test counts as sole gate.

---

## Issues found (pre-fix) and disposition

### Blockers fixed by checker

1. **[nyquist_compliance] VALIDATION.md missing**  
   - Severity: blocker  
   - Fix applied: seeded `06-VALIDATION.md` from RESEARCH Validation Architecture + plan filters; Plan 06 Task 1 updated to refine post-SUMMARY.

2. **[research_resolution] Open Questions unresolved**  
   - Severity: blocker  
   - Fix applied: marked `## Open Questions (RESOLVED)` with Plan 01/03/04 decisions.

### Warnings remaining (non-blocking)

1. **[context_compliance] Codex Login now auto-retry after external CLI**  
   - Plan: 03  
   - Severity: warning  
   - Description: For Codex, Login now may be CLI-primary. Auto-retry is wired to AuthComplete; external `bum login --provider codex` may not fire mid-session AuthComplete unless auth-meta refresh / focus path also applies deferred.  
   - Fix (optional during execute): ensure any mid-session auth-status refresh path that marks Codex usable also calls the deferred apply helper; document UAT step if poll is deferred.

2. **[verification_derivation] Plan 05 verification claimed pager non-cancel**  
   - Plan: 05  
   - Severity: warning (fixed in plan text)  
   - Description: `<verification>` claimed pager assertion without pager files.  
   - Fix applied: verification text corrected to shell-only next-sample proof.

3. **[dependency_correctness] Plan 04 wave was 3 with depends_on only 01**  
   - Severity: warning (fixed)  
   - Fix applied: wave → 2; Plan 03 depends_on includes 04 for cache API.

---

## Checker-applied edits

| File | Change |
|------|--------|
| `06-VALIDATION.md` | Created (Nyquist seed) |
| `06-RESEARCH.md` | Open Questions → RESOLVED |
| `06-01-PLAN.md` | Explicit bootstrap/load gate policy |
| `06-03-PLAN.md` | depends_on +04; AuthComplete must refresh usable cache |
| `06-04-PLAN.md` | wave 3 → 2 |
| `06-05-PLAN.md` | verification text accuracy |
| `06-06-PLAN.md` | Task 1 updates seeded VALIDATION (not invent from empty) |

---

## Plan summary

| Plan | Tasks | Wave | Focus | Status |
|------|-------|------|-------|--------|
| 01 | 2 | 1 | Shell gate + typed ACP error | Valid |
| 02 | 2 | 2 | QuestionView + no optimistic current | Valid |
| 04 | 2 | 2 | Dual-slot cache + needs login badge | Valid |
| 05 | 2 | 2 | Free dual switch + BYOK/history | Valid |
| 03 | 2 | 3 | Login now + AuthComplete deferred retry | Valid |
| 06 | 2 | 4 | VALIDATION refine + phase gate | Valid |

---

## Recommendation

Plans will achieve Phase 6 goal (MOD-03 + MOD-06) if executed in wave order. Residual Codex CLI AuthComplete wiring is an execution-time care item, not a replan.

**Next:** `/gsd:execute-phase 6`

---

## PLAN CHECK PASSED
