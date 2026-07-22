---
phase: 04-provider-aware-request-routing
reviewed: 2026-07-16T14:05:04Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - crates/codegen/xai-grok-sampler/src/client.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
  - crates/codegen/xai-grok-shell/src/agent/models.rs
  - crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs
  - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
  - crates/codegen/xai-grok-shell/src/auth/mod.rs
  - crates/codegen/xai-grok-shell/src/auth/model.rs
  - crates/codegen/xai-grok-shell/src/auth/storage.rs
  - crates/codegen/xai-grok-shell/src/sampling/mod.rs
  - crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs
  - crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs
  - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
  - crates/codegen/xai-grok-shell/src/session/commands.rs
  - crates/codegen/xai-grok-shell/tests/provider_routing.rs
findings:
  critical: 1
  warning: 6
  info: 3
  total: 10
status: clean
fixed_at: 2026-07-16T14:32:23Z
fix_report: 04-REVIEW-FIX.md
---

# Phase 4: Code Review Report

**Reviewed:** 2026-07-16T14:05:04Z  
**Fixed:** 2026-07-16T14:32:23Z (see `04-REVIEW-FIX.md` — CR-01 + WR-01…WR-06)  
**Depth:** standard
**Files Reviewed:** 15
**Status:** clean (post-fix)


## Summary

Adversarial standard-depth review of Phase 4 provider-aware routing (plans 04-01…04-05): dual-key credential resolve, catalog stamp/`resolve_provider_route`, prepare/reconstruct transforms, Codex auth snapshot, and SamplingClient local fail-closed.

Core dual-key isolation (`resolve_credentials_for_provider`, never_cross_slot tests), catalog Codex stamping, SetSessionModel auth_type carrier, and reconstruct “xAI resolver only for `Some(Xai)`” are directionally sound. Tests in `provider_routing.rs` cover the main happy/negative contracts well.

However, the new local fail-closed preflight has a hole: usable-bearer preflight can pass while Authorization insertion silently fails, allowing unauthenticated HTTP — which breaks the D-11 guarantee. Several host-trust, auxiliary-path, and async/blocking gaps also remain.

## Critical Issues

### CR-01: Fail-closed preflight can still send unauthenticated HTTP when live bearer is nonblank but not a valid header

**Severity:** BLOCKER / P0 / CRITICAL  
**File:** `crates/codegen/xai-grok-sampler/src/client.rs:585-621`  
**Issue:** `ensure_usable_auth_material` treats any nonblank `current_bearer()` as usable auth. Later, `post` only inserts the Authorization/`x-api-key` header inside `if let Ok(v) = HeaderValue::from_str(...)`. If the live token contains characters illegal in HTTP headers (control chars, etc.), preflight returns `Ok`, insertion is skipped, static headers may be empty, and the request is still sent with **no** auth material. That violates the D-11 “no network without usable credentials” contract introduced in 04-05.

**Fix:** After applying resolver (and static) headers, re-check usable auth material and error if still missing; or treat `HeaderValue` construction failure as `SamplingError::Auth` and do not build the request.

```rust
fn post(&self, url: impl reqwest::IntoUrl) -> Result<reqwest::RequestBuilder> {
    self.ensure_usable_auth_material()?;
    let mut headers = self.default_headers.clone();
    if let Some(resolver) = &self.bearer_resolver
        && let Some(fresh) = resolver.current_bearer()
    {
        let fresh = fresh.trim();
        if !fresh.is_empty() {
            match self.defaults.auth_scheme {
                AuthScheme::XApiKey => {
                    headers.remove(AUTHORIZATION);
                    let v = HeaderValue::from_str(fresh).map_err(|_| {
                        SamplingError::Auth(
                            "Invalid live bearer: cannot convert to HTTP header".into(),
                        )
                    })?;
                    headers.insert(HeaderName::from_static("x-api-key"), v);
                }
                AuthScheme::Bearer => {
                    headers.remove(HeaderName::from_static("x-api-key"));
                    let v = HeaderValue::from_str(&format!("Bearer {fresh}")).map_err(|_| {
                        SamplingError::Auth(
                            "Invalid live bearer: cannot convert to Authorization header"
                                .into(),
                        )
                    })?;
                    headers.insert(AUTHORIZATION, v);
                }
            }
        }
    }
    // Defense in depth: never send if headers still lack usable material.
    let tmp = SamplingClientView { /* or free fn over headers + scheme */ };
    // simplest: duplicate has_usable check against `headers` here
    Ok(self.http.post(url).headers(headers))
}
```

## Warnings

### WR-01: `is_first_party_codex_url` host-only branch defeats path allowlist on default chatgpt.com

**Severity:** WARNING / P1 / HIGH  
**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:4496-4515`  
**Issue:** Docs require host `chatgpt.com`/`www.chatgpt.com` **and** path prefix `/backend-api/codex`. The second branch only compares **hosts** to `resolve_codex_base_url()`. With the product default `https://chatgpt.com/backend-api/codex`, any URL on `chatgpt.com` (e.g. `https://chatgpt.com/`, `https://chatgpt.com/backend-api/conversation`) gets `session_oauth_allowed = true`, so product session OAuth may attach outside the Codex backend path.

**Fix:** Require path alignment for the configured endpoint as well (e.g. host match **and** path starts with configured path, or exact prefix of the full configured base). Keep path check for stock chatgpt hosts even when host matches configured host.

```rust
// After host match to configured endpoint:
let cfg_path = configured.path().trim_end_matches('/');
let req_path = parsed.path();
if host.eq_ignore_ascii_case(cfg_host)
    && (cfg_path.is_empty() || req_path.starts_with(cfg_path))
{
    return true;
}
```

Add a regression test: `https://chatgpt.com/` with default endpoints → `session_oauth_allowed == false`.

### WR-02: `OverrideModelName` reuses existing api_key as untyped provider-slot session key

**Severity:** WARNING / P1 / HIGH  
**File:** `crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs:326-332`  
**Issue:** After a name-only model override, credentials are re-resolved via `try_resolve_model_credentials(model_name, existing.api_key)`. That single-key API maps the string into the **target** model’s provider slot. If `model_name` hits a catalog entry for the other provider, the previous provider’s token is treated as that slot’s session OAuth key. The path also updates only `api_key`/`auth_type`, **not** `base_url`, so routing and credentials can diverge under dual-provider catalogs.

**Fix:** Either (a) stop re-resolving credentials on OverrideModelName (name/header-only, as the command docs suggest), or (b) dual-key resolve with provider-correct keys and, if provider changes, refuse or run full prepare/SetSessionModel.

### WR-03: Blocking disk I/O for Codex snapshot on prepare / ModelsManager hot paths

**Severity:** WARNING / P1 / MEDIUM  
**Files:**
- `crates/codegen/xai-grok-shell/src/agent/config.rs:4686-4698`
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:1105-1107`
- `crates/codegen/xai-grok-shell/src/agent/models.rs:958`

**Issue:** `snapshot_codex_session_key_from_auth_store` does synchronous `File::open`/`read_to_string` via `read_provider_auth_store`. It is invoked from async model-switch prepare and from `ModelsManager::sampling_config` (agent construction). On Tokio multi-thread this can stall a worker under disk latency; it also re-reads the full auth document on every prepare/switch.

**Fix:** Cache Codex snapshot with mtime/inode invalidation, or spawn_blocking for the read; long-term Phase 5 live AuthManager should remove prepare-time disk reads.

### WR-04: Subagent Codex path cannot use Codex session credentials; endpoints may drift

**Severity:** WARNING / P1 / MEDIUM  
**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:1021-1031`  
**Issue:** For `ModelProvider::Codex`, both slots are forced to `None` (`codex_key = None`), so Codex OAuth never applies to subagents (BYOK-only). Comment marks Phase 7, but product impact is real now that Codex is in the catalog. Additionally `EndpointsConfig::default()` is used instead of the parent session’s effective endpoints, so OAuth host policy for any future dual-key wiring can diverge from catalog stamp provenance.

**Fix:** Snapshot Codex the same way as prepare (or pass dual keys from parent context) and pass parent `EndpointsConfig` into `resolve_credentials_for_provider`.

### WR-05: Prepare forces SessionToken auth_type for Codex when only xAI session-based method is active

**Severity:** WARNING / P2 / MEDIUM  
**File:** `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:1121-1148`  
**Issue:** Two post-resolve rewrites are still provider-blind:
1. Preferred `Oidc` clears `api_key` and forces `SessionToken` for any model without own credentials.
2. When `!has_session_key && is_session_based_auth()` (xAI ACP method), `auth_type` is forced to `SessionToken` even for Codex with empty codex slot.

Result: Codex prepare can stamp `auth_type = SessionToken` with `api_key = None`. Live sample fail-closed saves correctness, but chat-state provenance is wrong and any path that branches on `auth_type` without checking the key will misbehave.

**Fix:** Gate both rewrites on `model.info.provider == ModelProvider::Xai` (or on `provider_session_key` presence for the active slot).

### WR-06: Credential resolve uses catalog `info.base_url` even when route falls through from blank/whitespace base

**Severity:** WARNING / P2 / MEDIUM  
**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:4726-4743`  
**Issue:** `resolve_provider_route(..., Some(info.base_url))` treats blank/whitespace overrides as absent and computes a real default `route.base_url` + `session_oauth_allowed`. Session attachment then still uses `info.base_url.clone()` (blank), so OAuth may be allowed against the default host policy while the request base remains empty/whitespace.

**Fix:** Prefer `route.base_url` for the returned `ResolvedCredentials.base_url` (or normalize blank catalog bases before resolve).

## Info

### IN-01: `PreparedSamplingConfig.provider` is carried then discarded by chat-state transform

**Severity:** INFO / P2 / LOW  
**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:4627-4653`, `session/.../model_switch.rs:52-65`  
**Issue:** `provider` is required on `SetSessionModel` and stored on `PreparedSamplingConfig`, but `apply_prepared_sampling_to_chat_state_fields` never persists it. Reconstruct relies on catalog `ModelAuthFacts.provider` instead. Dead field increases API surface without runtime effect.

**Fix:** Persist provider on session/chat-state if reconstruct should trust prepare, or stop plumbing an unused field through `SessionCommand`.

### IN-02: Single-key `resolve_credentials` still cannot type-check dual-slot provenance

**Severity:** INFO / P2 / LOW  
**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:4537-4545`, tests at `provider_routing.rs:740-758`  
**Issue:** Documented and tested: single-key maps any string into the model’s slot. Auxiliary callers (`resolve_credentials_enforced`, web-search helpers, OverrideModelName) remain footguns under dual-provider catalogs. Prefer dual-key APIs everywhere production dual tokens exist.

### IN-03: Codex credentials are snapshot-only until next prepare/switch (Phase 5)

**Severity:** INFO / P2 / LOW  
**Files:** `config.rs:4678-4683`, `sampler_turn.rs:278-285`  
**Issue:** Accepted Phase 4 limitation: no live Codex bearer refresh on reconstruct; expired Codex tokens mid-session fail until re-switch. Ensure Phase 5 owns replacement; no action required for Phase 4 ship beyond docs/UX expectations.

## Test proof quality (`provider_routing.rs`)

Strengths:
- Dual-token `never_cross_slot` with both slots populated
- Catalog stamp for xAI vs Codex bases / `api_base_url`
- Custom-host OAuth deny + configured Codex endpoint allow
- Transform A/B composition (`switch_changes_next_sample_route`)
- Fail-closed suite (missing/blank/empty/whitespace resolver) with zero HTTP
- On-wire Authorization mock proofs for xAI vs Codex

Gaps:
- No regression for `https://chatgpt.com/` path-less OAuth deny (WR-01)
- No OverrideModelName dual-provider test (WR-02)
- No HeaderValue-invalid live bearer fail-closed test (CR-01)

---

_Reviewed: 2026-07-16T14:05:04Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Depth: standard_  
