---
phase: 03-model-catalog-gpt-5-6-entries
fixed_at: 2026-07-16T14:45:00Z
review_path: .planning/phases/03-model-catalog-gpt-5-6-entries/03-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 3: Code Review Fix Report

**Fixed at:** 2026-07-16T14:45:00Z
**Source review:** `.planning/phases/03-model-catalog-gpt-5-6-entries/03-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (WR-02, WR-04 — explicit fix scope; WR-01/03/05 deferred)
- Fixed: 2
- Skipped: 0 (in-scope)

**Out of scope (not fixed, per request):**
- WR-01 — wrong base_url for GPT until Phase 4 routing (intentional catalog-only)
- WR-03 — remote provider force untested (optional)
- WR-05 — config override rebinding Codex (intentional / user config may override)
- IN-01..IN-03 — info-tier

## Fixed Issues

### WR-02: Empty prefetch + Codex re-append can make GPT the effective default

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/config.rs`, `crates/codegen/xai-grok-shell/tests/model_catalog.rs`
**Commit:** `83675bb`
**Applied fix:** After remove-then-append of bundled Codex defaults, if preferred default id (`default_model()` / `grok-build`) is missing **and** no `provider == Xai` rows remain, re-inject missing bundled **xAI** defaults at the front of the catalog. Non-empty remote catalogs still win (any remaining xAI row skips reinject). Updated `empty_prefetch_still_gets_codex_defaults` and added `empty_prefetch_default_stays_on_grok_build_not_gpt`; unit test renamed to expect reinject rather than empty map.

### WR-04: Invalid-provider override test can false-green

**Files modified:** `crates/codegen/xai-grok-shell/tests/model_catalog.rs`
**Commit:** `3aa9b2b`
**Applied fix:** `override_invalid_provider_warns_keeps_model` now requires `field == Some("provider")` **and** `kind == InvalidValue` (no longer accepts any InvalidValue or reason substring).

## Verification

```text
cargo test -p xai-grok-shell --test model_catalog
# 24 passed; 0 failed
```

---

_Fixed: 2026-07-16T14:45:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
