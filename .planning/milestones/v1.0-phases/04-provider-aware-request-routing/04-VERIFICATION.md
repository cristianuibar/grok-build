---
phase: 04-provider-aware-request-routing
verified: 2026-07-16T14:03:45Z
status: passed
score: 3/3 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
---

# Phase 4: Provider-aware request routing Verification Report

**Phase Goal:** Selecting a model routes each turn to the correct backend with that provider’s credentials  
**Verified:** 2026-07-16T14:03:45Z  
**Status:** passed  
**Re-verification:** No — initial verification  
**Mode:** mvp  
**Requirements:** MOD-04, MOD-05  

> **MVP goal format note:** ROADMAP `mode: mvp` goal is outcome-phrased, not full `As a…, I want to…, so that…`. User Flow Coverage below uses the plan-level user story (valid per `user-story.validate`). Live dual-provider daily-driver is deferred to Phase 9 per VALIDATION.md.

## User Flow Coverage

User story: *As a bum user, I want to select a model to route each turn to the correct backend with that provider’s credentials, so that Grok uses xAI/cli-chat-proxy and GPT uses the Codex/ChatGPT backend without cross-slot tokens.*

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Select Grok/xAI model | Next sample uses cli-chat-proxy / inference base + xAI credential only | `xai_model_routes_to_proxy_with_xai_token`, `on_wire_authorization_xai_fake`; `prepare_prepared_sampling_config_for_model` dual-key xAI slot | ✓ |
| Select GPT/Codex model | Next sample uses `https://chatgpt.com/backend-api/codex` + Codex credential; never cli-chat-proxy / xAI token | `codex_model_routes_to_codex_backend_with_codex_token`, `on_wire_authorization_codex_fake`, `no_proxy_headers_on_codex` | ✓ |
| Switch active model | Resolved base URL + credential slot change for next sample; Codex never attaches xAI AuthManager bearer | `switch_changes_next_sample_route` (transforms A/B); `model_switch.rs` → `apply_prepared_sampling_to_chat_state_fields`; reconstruct uses `reconstruct_attach_policy_from_facts` | ✓ |
| Outcome | Grok and GPT route correctly without cross-slot tokens | `never_cross_slot` dual-token isolation; mock Authorization never swaps slots | ✓ |

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Requests for Grok/xAI models use the xAI / cli-chat-proxy path with xAI credentials (not Codex tokens or Platform API semantics) | ✓ VERIFIED | Catalog stamps `grok-build` via `resolve_provider_route` → inference/proxy base; `resolve_credentials_for_provider` + dual-key prepare select xAI slot only; tests `xai_model_routes_to_proxy_with_xai_token`, `never_cross_slot`, `on_wire_authorization_xai_fake` **PASS** |
| 2 | Requests for GPT/Codex models use the OpenAI/Codex (ChatGPT backend) path with ChatGPT OAuth credentials — not the xAI proxy | ✓ VERIFIED | `gpt-5.6-sol` base = `CODEX_BASE_URL_DEFAULT` (`https://chatgpt.com/backend-api/codex`); no `cli-chat-proxy`; Codex slot only; no `X-XAI-Token-Auth`; tests `codex_model_routes_to_codex_backend_with_codex_token`, `on_wire_authorization_codex_fake`, `no_proxy_headers_on_codex`, `codex_skips_xai_api_key_env_fallback` **PASS** |
| 3 | Switching the active model changes resolved base URL / credential slot for the next sample (fake tokens OK) | ✓ VERIFIED | Production prepare → `PreparedSamplingConfig`/`SetSessionModel` carrier (`auth_type`+`provider`) → transform A `apply_prepared_sampling_to_chat_state_fields` → transform B attach policy; test `switch_changes_next_sample_route` **PASS** (assert base_url change + token change + Codex never attaches xAI resolver) |

**Score:** 3/3 truths verified (0 present, behavior-unverified)

### Supporting truths (plan must-haves — all exercised)

| Truth | Status | Evidence |
|-------|--------|----------|
| `resolve_provider_route` production authority (base + slot + session_oauth_allowed) | ✓ | `config.rs` ~4458; green pure-route suite |
| Dual-token `never_cross_slot` isolation | ✓ | Both tokens present; each model gets only its slot |
| Local fail-closed (None+None, blank static, empty/whitespace live resolver) | ✓ | `missing_credentials_*`, `empty_live_resolver_*`, `whitespace_live_resolver_*` — 0 HTTP |
| On-wire Authorization mock (xAI vs Codex fake) | ✓ | MockInferenceServer captures `Bearer xai-fake-token` / `Bearer codex-fake-token` |
| `ModelAuthFacts.provider: Option`; None/Codex never attach xAI bearer | ✓ | `should_attach_*_matrix`, `model_auth_facts_*`, reconstruct policy tests |
| No Phase 5 Codex OAuth UX / Phase 6 missing-provider gate shipped (D-13) | ✓ | Codex is prepare-time auth-store snapshot only; no login UX / switch-gate surface in phase files |
| Secret-safe invalid-header logs (`api_key_len` only) | ✓ | `invalid_header_path_does_not_log_full_key` |
| model_catalog regression green | ✓ | 24/24 pass |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/codegen/xai-grok-shell/tests/provider_routing.rs` | Wave 0+ phase gate binary | ✓ VERIFIED | 1425 lines; 48 tests all pass |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | Route/credential/transform authority | ✓ VERIFIED | `CODEX_BASE_URL_DEFAULT`, `ProviderRoute`, `resolve_provider_route`, dual-key credentials, transforms A/B, `ModelAuthFacts` |
| `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` | Provider-aware prepare | ✓ VERIFIED | `prepare_prepared_sampling_config_for_model` dual-key + Codex snapshot |
| `crates/codegen/xai-grok-shell/src/agent/models.rs` | ModelsManager dual-key sampling_config | ✓ VERIFIED | Uses `prepare_sampling_credentials` + `snapshot_codex_session_key_from_auth_store` |
| `crates/codegen/xai-grok-shell/src/session/commands.rs` | SetSessionModel auth carrier | ✓ VERIFIED | `auth_type` + `provider` fields |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | Consumes carrier via transform A | ✓ VERIFIED | No xAI AuthManager re-resolve of auth_type |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | Reconstruct attach policy | ✓ VERIFIED | `reconstruct_attach_policy_from_facts` → bearer_resolver only Some(Xai) |
| `crates/codegen/xai-grok-sampler/src/client.rs` | Local fail-closed + safe logs | ✓ VERIFIED | `ensure_usable_auth_material` before HTTP; `api_key_len` only on invalid header |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `ModelProvider` | `resolve_provider_route` | catalog stamp / credential / prepare | ✓ WIRED | `default_models` + override rebind call resolver |
| `prepare_prepared_sampling_config_for_model` | `resolve_credentials_for_provider` / dual keys | xAI AuthManager + Codex snapshot | ✓ WIRED | agent_ops.rs ~1090–1191 |
| `SetSessionModel` / model_switch | `apply_prepared_sampling_to_chat_state_fields` | prepared auth_type+provider | ✓ WIRED | model_switch.rs ~50–65 |
| `reconstruct_full_config` | `reconstruct_attach_policy_from_facts` | Option provider attach | ✓ WIRED | sampler_turn.rs ~282–351 |
| `SamplingClient::post` | `ensure_usable_auth_material` | Auth error before network | ✓ WIRED | client.rs ~586–599 |
| Mock HTTP tests | Authorization header | Bearer token capture | ✓ WIRED | on_wire_* tests |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Catalog entry base_url | `ModelEntry.info.base_url` | `resolve_provider_route` at stamp | Provider-specific defaults (proxy vs codex) | ✓ FLOWING |
| Resolved credentials | `api_key` / `base_url` / `auth_type` | Dual-key slot select + EndpointsConfig trust | Fake tokens in tests; production slots from AuthManager / auth.json | ✓ FLOWING |
| Chat-state after switch | `SamplingConfig` + `Credentials` | Transform A from PreparedSamplingConfig | Prepared provider fields, not re-resolved xAI | ✓ FLOWING |
| Live HTTP Authorization | Bearer header | SamplerConfig.api_key / bearer_resolver | Mock proves exact Bearer per provider | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Full MOD-04/05 phase gate | `cargo test -p xai-grok-shell --test provider_routing -- --nocapture` | **48 passed; 0 failed** (0.06s) | ✓ PASS |
| model_catalog regression | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | **24 passed; 0 failed** | ✓ PASS |
| Compile shell + sampler | `cargo check -p xai-grok-shell -p xai-grok-sampler` | Finished ok | ✓ PASS |
| SC-1 named test | `xai_model_routes_to_proxy_with_xai_token` | ok (in full suite) | ✓ PASS |
| SC-2 named test | `codex_model_routes_to_codex_backend_with_codex_token` | ok | ✓ PASS |
| SC-3 named test | `switch_changes_next_sample_route` | ok | ✓ PASS |
| Dual-token isolation | `never_cross_slot` | ok | ✓ PASS |
| On-wire Authorization | `on_wire_authorization_xai_fake` + `on_wire_authorization_codex_fake` | ok | ✓ PASS |
| Fail-closed | `missing_credentials_fail_closed_no_http` + empty/whitespace resolver | ok | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| N/A | Phase uses cargo integration tests, not `scripts/*/tests/probe-*.sh` | — | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| MOD-04 | 04-01…05 | Grok/xAI → xAI/cli-chat-proxy path + xAI credentials | ✓ SATISFIED | Route stamp + dual-key + on-wire Bearer xai-fake + proxy headers |
| MOD-05 | 04-01…05 | GPT/Codex → ChatGPT backend + Codex credentials, not xAI proxy | ✓ SATISFIED | Codex base stamp + dual-key + on-wire Bearer codex-fake + no proxy headers |

No orphaned Phase 4 requirements (REQUIREMENTS.md maps only MOD-04/MOD-05 to this phase).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TBD/FIXME/XXX debt markers in phase-touched production paths | — | None |
| — | — | No stub empty handlers or static hollow returns on routing path | — | None |

`todo_gate` matches in config/agent_ops are product TodoGate flags, not debt markers.

### Human Verification Required

None for Phase 4 automated gate.

Live dual-provider daily-driver turns require real ChatGPT OAuth and are **explicitly deferred** to Phase 5 (OAuth lifecycle) / Phase 9 (e2e) per `04-VALIDATION.md` Manual-Only Verifications. Not a Phase 4 gap.

### Gaps Summary

No gaps. All three roadmap success criteria are true in the codebase with behavioral proof:

1. **MOD-04 / SC-1:** Catalog + pure resolve + prepare + mock HTTP send xAI path/token.
2. **MOD-05 / SC-2:** Catalog + pure resolve + prepare + mock HTTP send Codex ChatGPT backend path/token without xAI proxy headers or xAI token.
3. **SC-3:** Production prepare → chat-state apply → reconstruct attach policy path changes base URL and credential slot; Codex never wires xAI AuthManager bearer.

Deferred by design (not gaps): live Codex multi-principal AuthManager refresh (Phase 5), missing-provider switch UX (Phase 6), live dual-driver e2e (Phase 9).

---

_Verified: 2026-07-16T14:03:45Z_  
_Verifier: Claude (gsd-verifier)_
