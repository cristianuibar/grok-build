---
phase: 4
reviewers: [codex]
reviewed_at: 2026-07-16T12:55:45Z
plans_reviewed:
  - 04-01-PLAN.md
  - 04-02-PLAN.md
  - 04-03-PLAN.md
  - 04-04-PLAN.md
  - 04-05-PLAN.md
---

# Cross-AI Plan Review — Phase 4

## Codex Review

# Phase 4 Plan Review

## Overall assessment

The plans identify the correct production choke points and sequence the broad work sensibly: endpoint definition → catalog routing → credentials → session reconstruction → safety gate. However, the validation strategy mostly proves configuration values through pure helpers, not the actual next-turn HTTP path. Several high-risk mechanisms—provider discovery during reconstruction, missing-credential rejection, Codex token selection, and on-wire authorization—remain either discretionary or untested.

**Overall risk: HIGH.** The implementation could make every planned integration test green while still sending an unauthenticated request, selecting an arbitrary Codex credential, or failing to exercise the real model-switch/reconstruction path.

---

## 04-01 — Wave 0 provider-routing harness

### Summary

A useful public-API test scaffold, but several named contracts are weaker than their names imply. In particular, independently resolving one fake token per provider does not prove cross-slot isolation, and building two `SamplerConfig` values does not prove that the existing switch command updates the next sampled request.

### Strengths

- The plan correctly targets the existing public catalog surface. The established Phase 3 integration tests already use `resolve_model_list`, `Config`, and `ModelProvider` this way in [model_catalog.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/model_catalog.rs:7).
- The RED Codex route is grounded in a real defect: `default_models()` currently assigns `resolve_inference_base_url()` to every entry in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3460), including the Codex entries present in [default_models.json](/home/cristian/bum/grok-build/crates/codegen/xai-grok-models/default_models.json:19).
- Testing proxy-derived headers is appropriate. Header injection is URL-dependent and isolated at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4758), so a Codex-host negative assertion protects a real boundary.
- The chosen integration-test dependencies already exist: `serial_test` and test support are available in [Cargo.toml](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/Cargo.toml:199).

### Concerns

- **HIGH — `switch_changes_next_sample_route` does not test the switch path.** Comparing two calls to `sampling_config_for_model` only exercises a pure constructor. Production switching passes through `prepare_sampling_config_for_model`, `SessionCommand::SetSessionModel`, chat-state updates, and later reconstruction. Those mechanisms are visible in [model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:115), [model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:48), and [sampler_turn.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:231). The planned test bypasses all three.
- **HIGH — the initial `never_cross_slot` contract does not prove isolation.** Calling `resolve_credentials(codex, Some("codex-fake"))` and `resolve_credentials(xai, Some("xai-fake"))` independently cannot detect a production caller passing the xAI token to the Codex model. The current function accepts any supplied session key without provider validation at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4384).
- **MEDIUM — committing deliberately failing integration tests can poison ordinary CI.** The task gate runs only `--list` and smoke, while the full integration binary remains red. Unless execution occurs on an isolated planning branch or RED tests are immediately followed by implementation before CI, normal `cargo test --tests` will fail.
- **MEDIUM — environment-dependent defaults can make route assertions flaky.** `Config::default()` reads endpoint environment variables in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:543), and `resolve_inference_base_url()` honors `models_base_url` in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:310).

### Suggestions

- Add a production-path test seam that applies a model switch to chat state and then reconstructs the full sampler config.
- Keep the initial cross-slot test explicitly named as a catalog/fixture test; reserve `never_cross_slot` for the later dual-slot API test.
- Construct deterministic `EndpointsConfig` values instead of relying on ambient endpoint variables.
- Avoid leaving RED tests committed between waves unless the project workflow explicitly tolerates a red branch.

### Risk assessment

**MEDIUM-HIGH.** The harness is valuable, but its strongest test names overstate what is actually exercised.

---

## 04-02 — Codex endpoints and pure route resolver

### Summary

The Codex endpoint addition matches existing configuration patterns, but the proposed “single authority” resolver is not actually required by the later plans’ production paths. It risks becoming a public, test-only abstraction while catalog stamping and credential resolution continue to route from `ModelInfo.base_url` directly.

### Strengths

- Adding the endpoint to `EndpointsConfig` is consistent with existing xAI defaults and environment resolution in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:540).
- `#[serde(default)]` on `EndpointsConfig` supports backward-compatible deserialization of configuration files without the new field at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:141).
- Keeping `api_backend` model-driven is correct. It is already stored on `ModelInfo` and copied into `SamplerConfig` at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4708).
- Keeping provider distinct from `agent_type` follows the existing explicit model schema at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3363).

### Concerns

- **HIGH — `resolve_provider_route` may never become production authority.** Plan 03 stamps URLs directly inside `default_models()`, while `resolve_credentials` continues to use `info.base_url`; Plan 04 then consumes those credentials. Unless later code calls `resolve_provider_route`, the pure resolver can pass every test while production routing follows a separate implementation.
- **HIGH — URL override plus provider OAuth slot can leak a bearer to an arbitrary host.** Local model overrides can set both `base_url` and provider in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3689). The proposed resolver preserves the Codex slot when a custom URL wins. Without requiring an own credential for non-first-party URLs, a Codex OAuth bearer could be sent to that custom host.
- **MEDIUM — provider changes without base URL changes need defined behavior.** `ConfigModelOverride::apply` can change provider independently at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3701). A bundled xAI entry overridden to `provider=codex` could retain its old proxy URL unless the route resolver is applied after all model overrides.
- **LOW — `String` for the endpoint field makes explicitness harder to distinguish.** Existing proxy configuration uses `Option<String>` specifically to retain whether it was configured at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:143). This may not matter now, but the asymmetry should be intentional.

### Suggestions

- Make `resolve_provider_route` part of the final model-resolution pipeline after remote and local overrides, not merely a helper called by tests.
- Define safe custom-host behavior: provider OAuth credentials should only go to an allowlisted first-party host; custom URLs should require model-owned credentials.
- Add tests for “provider overridden, base URL absent” and “custom URL without own credential.”
- Prefer one route-result type consumed by both catalog construction and sampling preparation.

### Risk assessment

**MEDIUM-HIGH.** The pure design is good, but production adoption and custom-host credential safety are underspecified.

---

## 04-03 — Catalog stamping and provider-aware credentials

### Summary

This plan fixes the confirmed catalog defect and introduces the right dual-slot concept. The major weakness is ambiguous backward compatibility: the old single-key API cannot tell whether its argument is xAI or Codex, while numerous existing callers currently supply an xAI token unconditionally.

### Strengths

- Branching default stamping by provider directly addresses the confirmed unconditional assignments at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3457).
- Setting `api_base_url: None` for Codex avoids the current xAI API-key fallback surface, which is used by `resolve_credentials` at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4390).
- Preserving model-owned credential priority maintains the existing `api_key/env_key` behavior in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4378).
- The dual-key API is a materially stronger seam than attempting to infer token provenance from token contents.

### Concerns

- **HIGH — the single-key wrapper remains inherently unsafe.** If `resolve_credentials(model, session_key)` means “the correct token for this model,” callers must already have performed provider selection. Today `ModelsManager` passes the xAI `AuthManager` token for every current model at [models.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/models.rs:949), and prepare does the same at [agent_ops.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:1095). The API provides no type-level protection against repeating that error.
- **HIGH — provider rebinding and default stamping order are incomplete.** Default entries are stamped before config overrides, while provider overrides are applied later in [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3203). Changing only `provider` can leave the old provider’s base URL.
- **MEDIUM — several credential call sites remain outside the two named production builders.** `try_resolve_model_credentials` calls the old resolver at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4476), and `resolve_model_to_sampling_config` does likewise at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4782). The phase should inventory each caller and explicitly defer or convert it.
- **MEDIUM — `XAI_API_KEY` environment testing needs isolation.** The existing resolver reads process state directly at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4390). The plan should mandate a serial guard for both setting and restoring the variable.
- **MEDIUM — `api_base_url: None` changes an existing unit-test assumption.** The plan calls updating the colocated unit test optional, but stale tests are future defects even if `--lib` is currently broken.

### Suggestions

- Deprecate or make the single-key resolver crate-private; use a `ProviderCredentials` structure at production call sites.
- Re-run route normalization after provider overrides, or make override application route-aware.
- Add a call-site table for every `resolve_credentials` use and record whether Phase 4 converts or intentionally defers it.
- Update stale colocated unit tests even if they are not a phase gate.

### Risk assessment

**HIGH.** Catalog stamping is straightforward, but ambiguous credential APIs can preserve precisely the cross-slot bug this phase is meant to eliminate.

---

## 04-04 — Prepare, ModelsManager, and reconstruction

### Summary

This is the critical plan and currently the least deterministic. It correctly identifies the xAI-only live bearer overwrite, but leaves provider recovery during reconstruction, Codex credential selection, and switch-path verification too open-ended.

### Strengths

- The plan correctly identifies the overwrite mechanism: `AuthManagerBearerResolver` always returns the xAI manager’s current token at [sampler_turn.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:245), and the sampler replaces the request Authorization from that resolver at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:552).
- Reusing the existing switch command is appropriate. It already calls prepare and then sends `SetSessionModel` at [model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:193).
- Keeping `ShellAuthCredentialProvider` xAI-specific avoids contaminating unrelated storage/upload services.
- A provider-scoped bearer policy is the right architectural direction.

### Concerns

- **HIGH — tests still do not exercise `prepare_sampling_config_for_model` or `reconstruct_full_config`.** Both methods are private, and the plan substitutes tests of public policy helpers. Those helpers can be correct while production forgets to call them. This does not prove success criterion 3.
- **HIGH — provider inference in reconstruction is left discretionary.** Chat-state `SamplingConfig` has no provider field at [types.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampling-types/src/types.rs:1034). The existing `ModelAuthFacts` contains only BYOK status and auth scheme at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4487). URL heuristics are fragile for custom endpoints and should not decide which bearer source is allowed.
- **HIGH — “first provider access token” is not a sound selection rule.** A provider slot is a `BTreeMap` of scopes in [model.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/model.rs:34). Choosing the first entry would be lexicographic, not based on intended Codex scope, account, expiry, or auth mode. The existing helper `lookup_auth` deliberately uses a requested scope at [model.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/model.rs:314).
- **HIGH — model-switch credential metadata remains xAI-biased.** After switching, the session fetches the xAI AuthManager key and passes it to `resolve_chat_state_auth_type` at [model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:61). This file is not in the plan’s modification list.
- **MEDIUM — synchronous auth-file reads may occur on hot paths.** Storage parsing uses blocking `File::open` and `read_to_string` at [storage.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:95). Reading this for each prepare/sampling-config call adds blocking I/O and bypasses a live credential manager.
- **MEDIUM — provider is already available through catalog lookup; URL heuristics are unnecessary.** `resolve_model_auth_facts` resolves the selected model through effective configuration at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4495). Extending these facts with provider is safer than matching hosts.
- **MEDIUM — snapshot-only Codex credentials will not satisfy the “prefer live bearer” objective.** This may be an acceptable Phase 4 compromise, but the plan should clearly mark it as a temporary limitation and ensure Phase 5 owns replacement.

### Suggestions

- Add `provider` to the resolved model facts or to a shared sampling-route type carried through chat state. Do not infer credential ownership from URL strings.
- Add a crate-local test module or explicit test seam that invokes the real prepare → chat-state update → reconstruct sequence.
- Select Codex credentials by a stable scope/account identifier and validate nonblank key, auth mode, and expiry.
- Update session `model_switch.rs` in this plan so `auth_type` is derived from provider-correct credentials.
- Cache the selected Codex credential or expose it through a provider credential manager instead of reading `auth.json` synchronously per turn.
- Make unknown provider resolution fail closed: never default an unknown/custom route to the xAI live resolver.

### Risk assessment

**HIGH.** This plan owns the security-critical runtime wiring but currently proves only helper functions around it.

---

## 04-05 — Observability, fail-closed behavior, and phase gate

### Summary

The planned gate is insufficient for its claims. Missing credentials currently produce an unauthenticated HTTP request and only fail after the remote server responds. In addition, existing sampler logging can expose full malformed credentials, while the proposed tests do not inspect actual HTTP Authorization headers.

### Strengths

- Logging provider, base URL, auth type, and boolean credential presence is operationally useful and avoids needing secret values.
- Keeping Phase 5 OAuth UX and Phase 6 login prompts out of scope is appropriate.
- The final regression commands include both the new routing test and existing model catalog tests.
- Both positive and negative proxy-header tests protect a real boundary.

### Concerns

- **HIGH — the proposed behavior is not local fail-closed.** `SamplingClient::new` accepts `api_key: None` and simply installs no Authorization header at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:404). It then sends the request; only a remote 401 becomes `SamplingError::Auth` at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:805). That is “send unauthenticated and wait for rejection,” not “block a live sample when credentials are absent.”
- **HIGH — the phase gate does not assert on-wire Authorization.** The proposed suite compares `SamplerConfig.api_key` values. An existing mock inference harness already demonstrates how to inspect the actual header in [test_sampling_client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/test_sampling_client.rs:1076), but the new plans do not use it.
- **HIGH — existing logging violates the stated no-secret rule.** Invalid header conversion logs the complete `api_key` at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:408) and again at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:422). Plan 05 does not include this sampler file.
- **MEDIUM — even normal request logs emit credential prefixes.** The client records up to 20 characters of Authorization and 12 characters of API key at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:571). That may be existing diagnostic policy, but it conflicts with an unqualified “never log raw secrets.”
- **MEDIUM — grep-based absence of UX symbols is not meaningful verification.** Scope control is better established through the changed-file list and diff review than a negative search for prompt text.
- **MEDIUM — no test proves the Codex URL receives no xAI live resolver after reconstruction.** Testing `should_attach_*` alone does not verify the private constructor uses it.

### Suggestions

- Add a local preflight error when both `api_key` and the provider-appropriate live resolver are absent. Test that no mock server request is received.
- Add mock-server tests for both providers asserting:
  - exact request path,
  - exact Authorization token,
  - absence/presence of `X-XAI-Token-Auth`,
  - no cross-provider token after a simulated model switch.
- Remove full-key logging from invalid-header paths and reconsider credential-prefix logging.
- Include `xai-grok-sampler/src/client.rs` in Plan 05’s files if local fail-closed and logging cleanup are implemented there.
- Make the phase gate include at least one real HTTP-boundary test, not only config construction.

### Risk assessment

**HIGH.** The final gate can report success without proving the live security and routing properties named by MOD-04/MOD-05.

---

## Required revisions before execution

At minimum, I recommend revising the plan set to add these four acceptance requirements:

1. A test exercises the real `prepare → SetSessionModel/chat-state → reconstruct` path.
2. A mock HTTP test proves the request URL and actual Authorization header for both providers.
3. Missing provider credentials return locally before any network request.
4. Provider identity is carried or catalog-resolved during reconstruction; URL heuristics and arbitrary “first scope” token selection are prohibited.

With those changes, the architecture would be credible and the overall risk would drop to **MEDIUM**.
---

## Consensus Summary

Single external reviewer (Codex). Consensus below is therefore that reviewer’s grounded findings (not multi-model agreement). Weight is high because findings cite concrete `file:line` evidence against the current tree.

### Agreed Strengths

- Plans target the real production choke points: catalog `default_models()` stamping, `resolve_credentials`, prepare/ModelsManager, and live `AuthManagerBearerResolver` overwrite on sample turns.
- Wave 0 RED harness anchors on a confirmed defect: Codex catalog rows still get xAI inference base URLs.
- Proxy-header positive/negative tests protect a real boundary (URL-dependent `X-XAI-Token-Auth` injection).
- Scope discipline is mostly good: Phase 5 OAuth UX and Phase 6 login prompts stay out of Phase 4.
- Dual-key credential API and provider-scoped bearer policy are the right architectural direction vs inferring token provenance from contents.

### Agreed Concerns

Priority for `/gsd:plan-phase 4 --reviews` — all currently **unresolved**:

1. **HIGH — Validation proves helpers, not the next-turn HTTP path.** Named tests (`switch_changes_next_sample_route`, `never_cross_slot`, phase gate) largely compare pure `SamplerConfig` construction. Production path is prepare → `SetSessionModel`/chat-state → reconstruct → live bearer overwrite; none of the gate tests exercise that sequence end-to-end.
2. **HIGH — Local fail-closed is not specified/enforced.** `SamplingClient` accepts `api_key: None`, sends unauthenticated HTTP, and only maps remote 401 → `SamplingError::Auth`. Phase gate can pass without blocking missing credentials before network I/O.
3. **HIGH — On-wire Authorization is unasserted.** Suite compares config fields; existing mock inference harness can inspect real headers but plans do not use it. Cross-slot token misuse could green-test.
4. **HIGH — Reconstruction provider identity is discretionary / heuristic.** Chat-state `SamplingConfig` has no provider field; URL heuristics + “first scope” token selection are unsafe. Provider should be catalog-resolved or carried explicitly; Codex scope selection needs a stable identifier.
5. **HIGH — `resolve_provider_route` may remain test-only.** Catalog stamping and credentials can route via `ModelInfo.base_url` without calling the pure resolver → dual implementations drift.
6. **HIGH — Custom URL + provider OAuth can leak bearers to arbitrary hosts.** Overrides can set custom `base_url` while preserving provider OAuth slot without first-party allowlist / own-credential requirement.
7. **HIGH — Single-key credential API remains unsafe; model_switch remains xAI-biased.** Existing callers pass xAI tokens unconditionally; session `model_switch.rs` still derives auth from xAI AuthManager and is not in the modify list.
8. **HIGH — Secret logging.** Invalid header paths log full `api_key`; Plan 05 does not touch sampler client.

### Divergent Views

N/A — only Codex ran (`--codex` only).

### Recommended plan revisions (from Codex)

Before execute-phase, plans should add acceptance requirements for:

1. A test of real `prepare → SetSessionModel/chat-state → reconstruct`.
2. Mock HTTP tests asserting request URL + actual Authorization for both providers.
3. Local preflight error when credentials and provider-appropriate live resolver are absent (no network).
4. Provider identity carried or catalog-resolved during reconstruction; ban URL heuristics and arbitrary first-scope selection.

With those, Codex estimates overall risk drops HIGH → MEDIUM.

### Next step

```text
/gsd:plan-phase 4 --reviews
```
