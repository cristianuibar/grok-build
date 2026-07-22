---
phase: 07-cross-provider-multi-agent-orchestration
reviewed: 2026-07-17T09:35:48Z
fixed: 2026-07-17T09:50:21Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - crates/common/xai-tool-types/src/task.rs
  - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs
  - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs
  - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs
  - crates/codegen/xai-grok-agent/src/builder.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
  - crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs
  - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
  - crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs
  - crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: clean
---

# Phase 7: Code Review Report

**Reviewed:** 2026-07-17T09:35:48Z  
**Fixed:** 2026-07-17T09:50:21Z  
**Depth:** standard  
**Files Reviewed:** 11  
**Status:** clean (Critical/Warning fixed; Info remaining optional)

## Summary

Adversarial review of Phase 7 product changes (plans 01–06): Task `reasoning_effort` schema/wire, async `preflight_spawn`, missing-provider spawn gate, dual-slot credential resolve, and isolation seams.

**What looks solid**

- Task vocabulary parse is fail-closed before spawn (`parse_task_reasoning_effort` + allowlist; blank/`null`/`undefined` omitted; `max` → `xhigh`).
- Async Task preflight is real coordinator RPC (not a sync slug-only Fn); `Unavailable` / `Denied` block the background “started” notice.
- Spawn gate runs **before** `insert_pending` / worktree on the shell path; pure `oauth_provider_slot_usable` is shared with model switch; Codex is never inferred from live xAI alone.
- `resolve_model_override_to_config` isolates slots (`Xai → (xai_key, None)`, `Codex → (None, codex_key)`); no parent bearer rewrite into the child provider slot on the explicit-model path.
- Plan 05 isolation harness exercises wire `Authorization` both directions with dual fake tokens.

**Fixes applied (2026-07-17)**

1. **CR-01** — Effort apply provenance now follows explicit runtime effort, not model stamp (`effort_apply_provenance`). Role/persona/definition effort soft-skips under Task `Tool` model stamp.
2. **WR-01** — Explicit Tool effort × model support gated in `preflight_subagent_spawn` and before `insert_pending`.
3. **WR-02** — Removed API key prefixes from subagent spawn/resolve logs; boolean presence only.
4. **WR-03** — Tool/explicit unknown model fails closed (no parent sampling-config fallback); final model re-checks provider gate.

---

## Critical Issues

### CR-01: Role/persona/definition effort hard-fails under Task provenance — FIXED

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs`  
**Commit:** `759fd00`  
**Status:** Fixed via `effort_apply_provenance` + unit tests.

---

## Warnings

### WR-01: Effort support fail-closed runs after pending/worktree — FIXED

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs`  
**Commit:** `759fd00`  
**Status:** Preflight + pre-pending explicit Tool effort gate.

### WR-02: Partial bearer tokens logged at info — FIXED

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/{handle_request,mod}.rs`  
**Commit:** `ca4a784`  
**Status:** `api_key_present` booleans only; no key material.

### WR-03: Unknown model → parent sampling config fallback — FIXED

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs`  
**Commit:** `759fd00`  
**Status:** Fail closed for Tool/explicit model; final credential re-gate.

---

## Info (optional / not fixed this pass)

### IN-01: Preflight returns `Ok` when agent definition is missing

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:161-172`

**Issue:** Unknown `subagent_type` yields `SubagentPreflightOutcome::Ok` (“spawn path will reject”). Task normally validates type first, so production impact is low; any caller that only preflights could still get a green light.

**Fix:** Return `Denied { message: format!("Unknown subagent type: …") }` (or `Unavailable` if preferred) so preflight is a strict superset of pre-spawn rejects where cheap.

### IN-02: Effort level not checked against model’s offered menu

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs`

**Issue:** Only `model_supports_reasoning_effort` (boolean) is consulted. A model that supports effort but only offers `{low, medium}` will still accept Task `xhigh` and stamp it onto `SamplerConfig`. Parent session switch path has richer `model_offers_reasoning_effort` logic.

**Fix:** For Tool provenance, reject when the parsed `ReasoningEffort` is not in `models_manager.model_reasoning_efforts(model_id)` (when the menu is non-empty).

---

## Areas Reviewed Without Defect

| Area | Verdict |
|------|---------|
| Task `reasoning_effort` schema + sanitize/canonicalize | Correct allowlist; invalid rejected before spawn |
| Async `SubagentBackend::preflight_spawn` | ChannelBackend timeout/fail-closed; coordinator Preflight arm side-effect free |
| Missing-provider gate placement | Before pending/worktree; shared pure helpers; login-shaped message |
| Dual-slot resolve (explicit model) | No cross-slot bearer; Codex never from xAI live alone |
| Plan 05 Authorization isolation tests | Both directions assert fixture tokens; parent model stability covered |
| NL Task description effort guidance | Present after model guidance |

---

## Counts

| Severity | Count (open) |
|----------|------:|
| Critical (P0 / BLOCKER) | 0 |
| Warning (P1) | 0 |
| Info | 2 |
| **Open total** | **2** |

**Ship recommendation:** Critical/Warning items fixed. Info (unknown-type preflight, effort menu) optional follow-ups.

See also: `07-REVIEW-FIX.md`.

---

_Reviewed: 2026-07-17T09:35:48Z_  
_Fixed: 2026-07-17T09:50:21Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Fixer: Claude (gsd-code-fixer)_  
_Depth: standard_  
