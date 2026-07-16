---
phase: 04-provider-aware-request-routing
fixed_at: 2026-07-16T14:35:00Z
review_path: .planning/phases/04-provider-aware-request-routing/04-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 4: Code Review Fix Report

**Fixed at:** 2026-07-16T14:35:00Z  
**Source review:** `.planning/phases/04-provider-aware-request-routing/04-REVIEW.md`  
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (CR-01 + WR-01…WR-06; Info left for optional follow-up)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: Fail-closed preflight can still send unauthenticated HTTP when live bearer is nonblank but not a valid header

**Files modified:** `crates/codegen/xai-grok-sampler/src/client.rs`  
**Commit:** `3ebb85d`  
**Applied fix:** `HeaderValue::from_str` failures for live bearer now return `SamplingError::Auth` (never silently skip insert). Added defense-in-depth check that request headers still hold usable auth for the scheme after resolver application. Regression test `invalid_live_bearer_header_value_fail_closed` covers control-character bearer.

### WR-01: `is_first_party_codex_url` host-only branch defeats path allowlist on default chatgpt.com

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/config.rs`, `crates/codegen/xai-grok-shell/tests/provider_routing.rs`  
**Commit:** `a90dd21`  
**Applied fix:** Configured-endpoint branch now requires path prefix alignment with `resolve_codex_base_url()` (host-only match insufficient). Regression `chatgpt_root_without_codex_path_disallows_session_oauth` covers `https://chatgpt.com/`, bare host, non-Codex path, and `www`.

### WR-02: `OverrideModelName` reuses existing.api_key as untyped provider-slot key

**Files modified:** `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs`  
**Commit:** `d74391f`  
**Applied fix:** Removed single-key `try_resolve_model_credentials` on `OverrideModelName`. Command is name/header/context-window only and keeps existing credentials + base_url (matches command docs). Full provider switch remains on `SetSessionModel` / prepare.

### WR-03: Blocking disk I/O for Codex snapshot on prepare / ModelsManager hot paths

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/config.rs`  
**Commit:** `9491dc2`  
**Applied fix:** `snapshot_codex_session_key_from_auth_store` caches token by path + mtime + file length so repeated prepare / `sampling_config` paths do not re-parse unchanged `auth.json`. Phase 5 live AuthManager still owns removing prepare-time disk reads entirely.

### WR-04: Subagent Codex path cannot use Codex session credentials; endpoints may drift

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs`  
**Commit:** `07cdfc3`  
**Applied fix:** Subagent model override resolve uses `models_manager.endpoints()` (catalog stamp provenance) and `snapshot_codex_session_key_from_auth_store()` for the Codex slot. Still never cross-applies parent xAI token into Codex.

### WR-05: Prepare forces SessionToken auth_type for Codex when only xAI session-based method is active

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs`  
**Commit:** `30dc953`  
**Applied fix:** Preferred `Oidc` wipe-to-SessionToken and session-based ACP `auth_type` override are gated on `model.info.provider == ModelProvider::Xai`.

### WR-06: Credential resolve uses catalog `info.base_url` even when route falls through from blank/whitespace base

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/config.rs`, `crates/codegen/xai-grok-shell/tests/provider_routing.rs`  
**Commit:** `44a4973`  
**Applied fix:** `resolve_credentials_for_provider` stamps `route.base_url` as the request base (blank catalog base re-resolves to provider default). Regression `blank_catalog_base_url_uses_route_base_for_credentials`.

## Deferred / not in scope this pass

### Info findings (IN-01…IN-03)

Left unfixed per `--fix` Critical+Warning scope:

- **IN-01:** `PreparedSamplingConfig.provider` carried then discarded by chat-state transform — API surface cleanup / persist decision.
- **IN-02:** Single-key `resolve_credentials` still cannot type-check dual-slot provenance — prefer dual-key at remaining auxiliary call sites over time.
- **IN-03:** Codex credentials snapshot-only until next prepare/switch — accepted Phase 4 limitation; Phase 5 live AuthManager.

## Test results

```text
cargo test -p xai-grok-shell --test provider_routing
# 50 passed; 0 failed

cargo test -p xai-grok-sampler --lib invalid_live_bearer
# 1 passed

cargo check -p xai-grok-shell -p xai-grok-sampler
# ok
```

## Commit map

| Finding | Commit   | Message (short) |
|---------|----------|-----------------|
| CR-01   | `3ebb85d` | Reject invalid live bearer headers before sampling HTTP |
| WR-02   | `d74391f` | Keep OverrideModelName from re-resolving credentials across providers |
| WR-05   | `30dc953` | Gate prepare SessionToken auth_type rewrites to xAI only |
| WR-04   | `07cdfc3` | Wire subagent Codex resolve to snapshot and parent endpoints |
| WR-01   | `a90dd21` | Require Codex OAuth path alignment with configured base URL |
| WR-03   | `9491dc2` | Cache Codex auth.json snapshot by mtime and file length |
| WR-06   | `44a4973` | Stamp credential resolve base_url from provider route |

---

_Fixed: 2026-07-16T14:35:00Z_  
_Fixer: Claude (gsd-code-fixer)_  
_Iteration: 1_  
