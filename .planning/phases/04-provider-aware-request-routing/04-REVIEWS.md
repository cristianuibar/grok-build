---
phase: 4
reviewers: [codex]
reviewed_at: 2026-07-16T13:18:04Z
cycle: 3
plans_reviewed:
  - 04-01-PLAN.md
  - 04-02-PLAN.md
  - 04-03-PLAN.md
  - 04-04-PLAN.md
  - 04-05-PLAN.md
plans_commit: 851dc02
---
# Cross-AI Plan Review — Phase 4

## Codex Review — Cycle 1

> Historical audit trail. Plans at cycle-1 review time (pre-replan `1b9196d`). Do not treat as current residual list.


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

## Consensus Summary — Cycle 1

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


---

# Cycle 2 Codex re-review

Date: 2026-07-16  
Plans: post-replan commit `1b9196d` (cycle-1 HIGHs absorbed into PLAN.md)  
Reviewer: Codex (source-grounded)

# Phase 4 Plan Review — Convergence Cycle 2

## Summary

The revised plans are substantially stronger and correctly target the major production seams: catalog stamping, dual-slot credentials, model preparation, session reconstruction, HTTP authorization, and secret-safe logging. However, convergence is not complete. Seven of the thirteen cycle-1 HIGH findings are fully resolved at the plan level; six are only partially resolved. Five distinct HIGH-risk design gaps remain, primarily around testing the real switch path, representing an unknown provider safely, preserving endpoint trust provenance, carrying credential metadata through `SetSessionModel`, and handling an empty live bearer resolver.

Reviewed against clean `HEAD 1b9196d`.

## Strengths

- Plan 02 now makes `resolve_provider_route` an explicit production authority and Plan 03 requires `default_models()` to call it. This directly addresses the current provider-blind stamping at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3460), where every model currently inherits the xAI inference URL and `xai_api_base_url`.

- Provider rebinding is now covered. The current override order changes `base_url` before `provider` and never renormalizes it at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3689). Plan 03 adds an explicit provider-only rebind test and production action.

- The dual-key credential API and both-present `never_cross_slot` test are meaningful improvements. They directly target current callers that unconditionally supply the xAI `AuthManager` token in [models.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/models.rs:949) and [agent_ops.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:1092).

- Plan 04 correctly rejects arbitrary `BTreeMap` ordering for Codex credential selection. Provider slots are scope maps at [model.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/model.rs:25), while the existing `lookup_auth` intentionally selects a requested scope and rejects legacy web login at [model.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/model.rs:314).

- Reconstruction is now explicitly in scope. This is necessary because the current resolver always reads the xAI `AuthManager` at [sampler_turn.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:251) and attaches it whenever the existing gate is active at [sampler_turn.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:339).

- Plan 05 adds genuine on-wire authorization tests using an established harness. The repository already demonstrates exact `Authorization` inspection at [test_sampling_client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/test_sampling_client.rs:1077).

- The full-key log leak is now explicitly fixed. Both unsafe logging sites are real: [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:408) and [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:422).

- Scope boundaries are well maintained: no Codex OAuth lifecycle, login modal, or Phase 6 provider gate is introduced.

## Cycle-1 HIGH Status

| Prior HIGH | Status | Assessment |
|---|---|---|
| 1. Switch test bypasses production path | **PARTIALLY RESOLVED** | The new helper simulates the transition, but still explicitly runs “without full ACP session” in [04-04-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:118). |
| 2. `never_cross_slot` does not prove isolation | **FULLY RESOLVED** | Plan 03 requires both tokens simultaneously through the new dual-key API. |
| 3. Resolver may remain test-only | **FULLY RESOLVED** | Plans 03–04 mandate production calls, especially catalog stamping. |
| 4. Custom URL may receive provider OAuth | **PARTIALLY RESOLVED** | Host policy exists, but endpoint/override provenance is not represented reliably in the proposed credential API. |
| 5. Single-key wrapper remains unsafe | **FULLY RESOLVED** | Production dual-slot builders must use the dual-key API; remaining callers are inventoried. |
| 6. Provider rebinding leaves stale base URL | **FULLY RESOLVED** | Plan 03 adds renormalization and tests. |
| 7. Tests skip real prepare/reconstruct | **PARTIALLY RESOLVED** | Shared helpers are better, but the planned switch test remains a simulation rather than the actual handler/reconstruct path. |
| 8. Reconstruction provider identity is discretionary | **PARTIALLY RESOLVED** | Catalog provider is selected, but the proposed type cannot safely represent unknown. |
| 9. First token from provider map is arbitrary | **FULLY RESOLVED** | Deterministic mode/expiry selection and adversarial fixtures are specified. |
| 10. Model-switch credential metadata is xAI-biased | **PARTIALLY RESOLVED** | The plan identifies the file but does not define how `auth_type` travels with the prepared config. |
| 11. Missing credentials fail only after remote 401 | **PARTIALLY RESOLVED** | No-key/no-resolver is handled, but a present resolver returning no token remains untested and potentially unauthenticated. |
| 12. No on-wire Authorization assertion | **FULLY RESOLVED** | Plan 05 adds exact wire-header tests and zero-request checks. |
| 13. Full secret logged on invalid header | **FULLY RESOLVED** | Sampler client is now modified and the full-key fields are removed. |

## Concerns

- **HIGH — unknown provider cannot be represented by the proposed `ModelAuthFacts.provider`.** Plan 04 requires `provider: ModelProvider`, then says an unknown catalog lookup must fail closed ([04-04-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:264)). But `ModelProvider` has only `Xai` and `Codex`, with `Xai` as its default at [config.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3367). The proposed helper also accepts a non-optional `ModelProvider` ([04-04-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:115)). An absent model can therefore accidentally collapse to xAI and attach its live resolver.

- **HIGH — the switch test still does not exercise the actual switch pipeline.** The plan’s helper explicitly simulates chat-state writes without the ACP session. In production, `SetSessionModel` carries only `SamplerConfig` at [commands.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/commands.rs:161), the handler writes fields separately at [model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:48), and reconstruction later rebuilds the full client at [sampler_turn.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:256). A parallel orchestration helper can still pass while one of these production transfers is wrong.

- **HIGH — provider-correct `auth_type` has no defined carrier.** Plan 04 says to derive it from the prepared `sampling_config.api_key` or “credentials already returned” ([04-04-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:214)). But `SamplerConfig` stores the key and auth scheme, not whether it came from a session or BYOK, while `SetSessionModel` carries no separate `ResolvedCredentials`. The current handler consequently re-resolves metadata with the xAI token at [model_switch.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:61). The plan needs an explicit data-flow change, not “prefer” wording.

- **HIGH — first-party OAuth eligibility loses endpoint provenance.** The proposed `resolve_credentials_for_provider` receives only `ModelEntry` and two keys, then ambiguously suggests reconstructing endpoints or comparing stamped URLs ([04-03-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:190), [04-03-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:193)). `ModelEntry` currently stores only the resolved `base_url`; it does not record whether that URL came from a trusted provider endpoint or a model override. This can either reject a legitimate configured Codex endpoint or attach OAuth to an override misclassified as trusted.

- **HIGH — fail-closed does not cover a resolver that exists but returns no bearer.** The current request code silently leaves existing headers unchanged when `current_bearer()` returns `None` at [client.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:550). Plan 05’s required test only covers `api_key=None` and `bearer_resolver=None` ([04-05-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-05-PLAN.md:169)). A stale or empty resolver can therefore still send an unauthenticated request.

- **MEDIUM — provider-store read errors are collapsed into absence.** Plan 04 proposes `read_provider_auth_store(...) -> Option<AuthStore>` ([04-04-PLAN.md](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:200)), while the underlying parser returns `io::Result` at [storage.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:95). Missing file, malformed JSON, unsupported version, and an empty provider slot should remain distinguishable for diagnostics, even though each fails closed.

## Suggestions

- Change `ModelAuthFacts.provider` to `Option<ModelProvider>` and make resolver attachment require `provider == Some(ModelProvider::Xai)`. Test empty, absent, and config-load-failure cases.

- Carry prepared credential provenance explicitly through model switching. For example, add `auth_type` to `SetSessionModel`, or return a `PreparedSamplingConfig { sampler_config, auth_type, provider }` consumed directly by the session handler.

- Factor the exact production transformations into shared functions that production calls:

  - `SamplerConfig + auth_type -> chat-state SamplingConfig/Credentials`
  - `chat-state state + ModelAuthFacts -> reconstructed SamplerConfig`

  Then test those exact functions in sequence. Avoid a separate `next_sample_after_model_switch` simulation containing duplicate field-copy logic.

- Pass the effective `EndpointsConfig` into `resolve_credentials_for_provider`, or persist an explicit route/trust classification on the resolved model entry. Do not infer override provenance by comparing URL strings against a fresh default configuration.

- At the HTTP boundary, resolve the live bearer before sending. If neither a nonblank static key nor a nonblank resolved bearer exists, return local `SamplingError::Auth`. Add tests for:

  - resolver present but returns `None`
  - resolver returns whitespace
  - static key blank plus empty resolver
  - zero mock-server requests in all three cases

- Return `Result<Option<AuthStore>>` from provider-store loading and log a redacted diagnostic for parse/version/I/O failures.

## Risk Assessment

**Overall risk: HIGH.**

The revised plans close most structural problems and are much closer to executable convergence. The remaining gaps are concentrated in the most security-sensitive seam: determining whether an xAI OAuth bearer may be attached, carrying credential provenance across model switching, and proving the actual next-turn path. Until those mechanisms are made explicit, the implementation could satisfy the planned helper tests while still attaching xAI credentials to an unknown route or sending an unauthenticated request when a live resolver is empty.

---

## Consensus Summary — Cycle 2

Single external reviewer (Codex). Weight is high: findings cite concrete `file:line` evidence against the current tree and against plan text at `1b9196d`.

### Progress vs cycle 1

| Bucket | Count | Notes |
|--------|-------|-------|
| Cycle-1 HIGHs FULLY RESOLVED | 7 | never_cross_slot dual-key; resolver production authority; single-key dual-slot discipline; provider rebind URL renormalization; deterministic Codex token selection; on-wire Authorization tests; full-key log leak fix |
| Cycle-1 HIGHs PARTIALLY RESOLVED | 6 | switch/prepare path still simulated; custom-URL OAuth provenance; unknown provider typing; auth_type carrier; empty live-resolver fail-closed |
| Net residual HIGH (cycle 2) | **5** | See below — partials re-expressed as explicit residual design gaps |
| Actionable MEDIUM still open | **1** | Provider-store read errors collapsed to `Option` |

### Agreed Strengths (cycle 2)

- Production authority for `resolve_provider_route` + catalog stamping call sites.
- Dual-key credentials + both-present `never_cross_slot`.
- Deterministic Codex scope selection (not BTreeMap first entry).
- On-wire Authorization + zero-request fail-closed tests planned.
- Sampler full-key logging fix in scope.
- Scope discipline (no Phase 5/6 OAuth UX).

### Residual HIGH concerns (must replan)

1. **Unknown provider typing** — `ModelAuthFacts.provider: ModelProvider` cannot represent unknown; `Xai` default can attach live resolver incorrectly.
2. **Switch test still simulates** — `next_sample_after_model_switch` without production field-transfer functions can green while `SetSessionModel` / reconstruct stay wrong.
3. **`auth_type` carrier undefined** — model_switch still re-resolves with xAI token unless prepared provenance is carried explicitly.
4. **OAuth host trust provenance** — `resolve_credentials_for_provider(ModelEntry, keys)` lacks `EndpointsConfig` / trust classification; URL string compare is brittle.
5. **Empty live resolver** — preflight only covers `api_key=None && bearer_resolver=None`; present resolver returning `None`/whitespace can still send unauthenticated HTTP.

### Residual actionable non-HIGH

1. **MEDIUM — provider-store errors as absence** — use `Result<Option<AuthStore>>` (or equivalent) and redacted diagnostics for parse/version/I/O vs missing file.

### Divergent Views

N/A — only Codex ran (`--codex` only).

### Recommended plan revisions

1. `ModelAuthFacts.provider: Option<ModelProvider>`; attach xAI resolver only when `Some(Xai)`.
2. Factor production transforms (`SamplerConfig+auth_type → chat-state`; `chat-state+facts → reconstruct`) and test those exact functions; drop duplicate simulation logic.
3. Explicit `auth_type` (and provider) carrier on `SetSessionModel` or `PreparedSamplingConfig`.
4. Pass effective `EndpointsConfig` or persist route/trust on model entry into credential resolution.
5. Preflight: require nonblank static key **or** nonblank resolved bearer before HTTP; test resolver-None, whitespace, blank+empty.
6. Distinguish auth-store load failures diagnostically.

### Cycle 2 status

| Severity | Unresolved count |
|----------|------------------|
| HIGH | 5 |
| MEDIUM actionable | 1 |
| LOW actionable | 0 |

**Converged:** no  
**Overall residual risk:** HIGH (security-sensitive seams still underspecified)

### Next step

```text
/gsd:plan-phase 4 --reviews
```


---

# Cycle 3 Codex re-review (FINAL before max)

Date: 2026-07-16  
Plans: post-replan commits `51a30ee`/`851dc02` (HEAD `851dc02`)  
Reviewer: Codex (source-grounded)  
Max cycles: 3 — this is the final convergence review cycle

# Cycle 3 Plan Review — Phase 4

## Summary

The five plans at HEAD `851dc027edcf54b4310a82d8bda49a98d40891a1` are implementation-ready for MOD-04/MOD-05. All five cycle-2 HIGH findings and the provider-store MEDIUM are now concretely addressed with named types, production call sites, shared production transforms, and automated verification. The plans correctly target the actual defects at HEAD: provider-blind catalog stamping, single-slot credential resolution, xAI-biased model switching, provider-blind resolver reconstruction, and unauthenticated requests when a live bearer resolver is empty. One new MEDIUM ambiguity remains around whether a custom xAI `models_base_url` is trusted for session OAuth.

## Strengths

- **Plan 01 establishes honest RED contracts.** It explicitly prevents the switch test from claiming success through two pure constructors and requires both credentials to coexist in `never_cross_slot` ([04-01-PLAN.md:15](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-01-PLAN.md:15), [04-01-PLAN.md:206](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-01-PLAN.md:206)). This matches the current unsafe single-key function, which accepts any supplied session token without provider provenance ([config.rs:4376](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4376), [config.rs:4384](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4384)).

- **Plans 02–03 create one production routing authority.** `default_models()` currently assigns the xAI inference base and `api_base_url` to every model, including Codex entries ([config.rs:3432](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3432), [config.rs:3457](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3457)). Plans 02–03 require `resolve_provider_route` to drive catalog stamping, provider rebinds, and credential trust rather than duplicating route tables ([04-02-PLAN.md:81](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-02-PLAN.md:81), [04-03-PLAN.md:162](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:162)).

- **Provider override rebinding is correctly included.** Today `ConfigModelOverride::apply` updates `base_url` and `provider` independently, so changing only the provider leaves the old route attached ([config.rs:3689](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3689), [config.rs:3701](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:3701)). Plan 03 requires re-normalization through the resolver while preserving explicit URL overrides ([04-03-PLAN.md:168](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:168)).

- **Plan 04 now traces the real switch path.** Production currently sends only `SamplerConfig` through `SetSessionModel` ([commands.rs:161](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/commands.rs:161), [model_switch handler:193](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:193)), then re-resolves `auth_type` from the xAI `AuthManager` inside the session ([model_switch.rs:61](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:61)). The revised plan mandates a prepared carrier and shared transform A used by both production and tests ([04-04-PLAN.md:126](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:126), [04-04-PLAN.md:291](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:291)).

- **Reconstruction is made provider-aware at the actual attachment point.** At HEAD, `reconstruct_full_config` activates the xAI resolver using only auth method, BYOK status, and xAI-host classification ([sampler_turn.rs:273](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:273), [sampler_turn.rs:339](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:339)). Plan 04 requires `Option<ModelProvider>` and production use of transform B, with resolver attachment limited to `Some(Xai)` ([04-04-PLAN.md:345](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:345), [04-04-PLAN.md:354](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:354)).

- **Plan 05 verifies the HTTP boundary, not just configuration fields.** The current client only overrides headers when `current_bearer()` returns `Some`, so an empty resolver can leave the request unauthenticated ([client.rs:549](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:549), [client.rs:596](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:596)). The plan requires local authentication errors and zero requests for missing, blank, `None`, and whitespace resolver results ([04-05-PLAN.md:91](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-05-PLAN.md:91), [04-05-PLAN.md:206](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-05-PLAN.md:206)).

## Cycle-2 Residual Status

| Residual | Status | Plan and source evidence |
|---|---|---|
| Unknown provider typing | **FULLY RESOLVED** | Current `ModelAuthFacts` has no provider field ([config.rs:4487](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4487)). Plan 04 requires `provider: Option<ModelProvider>`, maps catalog hits to `Some`, all unknown/config-failure cases to `None`, and tests `None`, `Some(Xai)`, and `Some(Codex)` attachment behavior ([04-04-PLAN.md:135](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:135), [04-04-PLAN.md:334](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:334), [04-04-PLAN.md:374](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:374)). |
| Switch test still simulates | **FULLY RESOLVED** | Current production field transfer occurs directly in `handle_set_session_model` ([model_switch.rs:48](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:48)). Plan 04 mandates production transforms A/B, requires production to call them, forbids parallel field-copy helpers, and runs `switch_changes_next_sample_route` against those functions ([04-04-PLAN.md:116](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:116), [04-04-PLAN.md:140](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:140), [04-04-PLAN.md:302](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:302)). |
| `auth_type` carrier undefined | **FULLY RESOLVED** | Current `SetSessionModel` lacks provenance and production re-resolves it from the xAI manager ([commands.rs:161](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/commands.rs:161), [model_switch.rs:62](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:62)). Plan 04 requires `PreparedSamplingConfig` or explicit command fields carrying `auth_type` and provider; model switching must consume the prepared value verbatim ([04-04-PLAN.md:156](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:156), [04-04-PLAN.md:282](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:282), [04-04-PLAN.md:293](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:293)). |
| OAuth host trust provenance | **FULLY RESOLVED** | Current credential resolution has no endpoint/trust input ([config.rs:4376](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4376)). Plan 03 fixes the required signature to include the effective `EndpointsConfig`, explicitly bans fresh-default comparisons, requires the same endpoint instance at catalog/prepare time, and tests configured Codex endpoints plus custom-host denial ([04-03-PLAN.md:127](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:127), [04-03-PLAN.md:199](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:199), [04-03-PLAN.md:210](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-03-PLAN.md:210)). |
| Empty live resolver | **FULLY RESOLVED** | Current `post` silently does nothing when a resolver returns `None` ([client.rs:552](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:552)). Plan 05 specifies the complete usable-material matrix, request-time protection, whitespace handling, and zero-request mock tests for resolver `None`, resolver whitespace, and blank-static combinations ([04-05-PLAN.md:91](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-05-PLAN.md:91), [04-05-PLAN.md:188](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-05-PLAN.md:188), [04-05-PLAN.md:200](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-05-PLAN.md:200)). |
| Provider-store errors treated as absence | **FULLY RESOLVED** | Existing parsing already distinguishes `NotFound`, `InvalidData`, and unsupported versions through `io::ErrorKind` ([storage.rs:95](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:95), [storage.rs:108](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:108), [storage.rs:122](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:122)). Plan 04 preserves this through `Result<Option<AuthStore>, E>`, redacted diagnostics, fail-closed callers, and a missing-vs-malformed test ([04-04-PLAN.md:173](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:173), [04-04-PLAN.md:280](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-04-PLAN.md:280)). |

## Concerns

- **MEDIUM — xAI custom endpoint trust is internally inconsistent and lacks a regression test.** Plan 02 says that an empty model override gives `session_oauth_allowed: true` ([04-02-PLAN.md:108](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-02-PLAN.md:108)), but its implementation task says trust is determined from the final URL ([04-02-PLAN.md:197](/home/cristian/bum/grok-build/.planning/phases/04-provider-aware-request-routing/04-02-PLAN.md:197)). This matters because `resolve_inference_base_url()` uses `models_base_url` when configured, even if it is a non-xAI host ([config.rs:310](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:310)). The current reconstruction gate protects this case by requiring `is_first_party_xai_url(base_url)` ([sampler_turn.rs:43](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:43), [sampler_turn.rs:52](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:52)). If the executor follows the unconditional-default wording or persists that result as route trust, an xAI OAuth token could be attached to a custom `models_base_url`. Existing planned custom-host tests cover explicit model overrides, principally Codex, but do not lock this configured xAI endpoint case.

## Suggestions

- Resolve the Plan 02 contradiction explicitly: `session_oauth_allowed` should be derived from the final resolved URL for xAI routes, including when `models_base_url` supplied that URL.
- Add a test with `provider=Xai`, `endpoints.models_base_url=https://byok.example/v1`, and an xAI session token. Assert `session_oauth_allowed == false` and that credential resolution does not select the session token. If custom xAI endpoint OAuth is intentionally supported, define that endpoint as an explicit trusted endpoint instead of treating every `models_base_url` as trusted.

## Risk Assessment

**Overall residual risk: MEDIUM.** The core MOD-04/MOD-05 routing, credential isolation, model-switch propagation, reconstruction policy, fail-closed behavior, and on-wire verification are all concretely planned against the correct production seams. No cycle-2 HIGH remains. The only actionable issue is a security-relevant ambiguity around xAI custom endpoint trust; clarifying one rule and adding one test should bring residual plan risk to LOW.

- **Residual HIGH count: 0**
- **Residual actionable MEDIUM/LOW count: 1**


---

## Consensus Summary — Cycle 3

Single external reviewer (Codex). Weight is high: findings cite concrete `file:line` evidence against the current tree and against plan text at `851dc02`.

### Progress vs cycle 2

| Bucket | Count | Notes |
|--------|-------|-------|
| Cycle-2 residual HIGHs FULLY RESOLVED | **5/5** | unknown provider Option typing; production transforms (not simulation); auth_type carrier; EndpointsConfig OAuth trust; empty live-resolver fail-closed |
| Cycle-2 residual MEDIUM FULLY RESOLVED | **1/1** | provider-store `Result<Option<AuthStore>>` + redacted diagnostics |
| Net residual HIGH (cycle 3) | **0** | All prior HIGHs closed at plan level with concrete tasks/verify |
| Actionable MEDIUM still open | **1** | Plan 02 internal inconsistency: empty override → session_oauth_allowed true vs final-URL first-party rule for custom `models_base_url` |

### Agreed Strengths (cycle 3)

- Plans are implementation-ready for MOD-04/MOD-05 against real production seams.
- Dual-key credentials + EndpointsConfig trust provenance + production stamping authority.
- PreparedSamplingConfig / SetSessionModel auth_type+provider carrier; shared transforms A/B.
- Option\<ModelProvider\> fail-closed reconstruction; xAI resolver only on Some(Xai).
- Local fail-closed matrix including empty/whitespace live resolver + on-wire Authorization mocks.
- Sampler full-key log leak fix remains in scope.
- Scope discipline (no Phase 5/6 OAuth UX).

### Residual concerns (cycle 3)

#### HIGH

None. All cycle-1/2 HIGHs are FULLY RESOLVED in current PLAN.md text with concrete tasks and automated verify.

#### Actionable non-HIGH

1. **MEDIUM — xAI custom endpoint trust inconsistency** (`04-02-PLAN.md` ~108 vs ~197): narrative says empty model override → `session_oauth_allowed: true`, but Task 2 derives trust from **final** base_url first-party classification. `resolve_inference_base_url()` can return a non-xAI `models_base_url` ([config.rs:310](crates/codegen/xai-grok-shell/src/agent/config.rs)). Existing reconstruction already uses `is_first_party_xai_url` ([sampler_turn.rs:43](crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs)). Missing regression: `provider=Xai` + custom `models_base_url` must deny session OAuth. Not yet locked as an explicit test in plans.

### Divergent Views

N/A — only Codex ran (`--codex` only).

### Cycle 3 status

| Severity | Unresolved count |
|----------|------------------|
| HIGH | 0 |
| MEDIUM actionable | 1 |
| LOW actionable | 0 |

**Converged on HIGH:** yes (0 residual HIGH)  
**Fully clean (HIGH+actionable):** no (1 MEDIUM)  
**Overall residual risk:** MEDIUM → LOW after clarifying Plan 02 final-URL rule + one regression test

### Recommended disposition for the residual MEDIUM

Optional micro-replan or absorb at execute time (executor-safe if Task 2 step 5 final-URL rule wins and test is added):

1. Lock rule: `session_oauth_allowed` from **final resolved URL** first-party check for that provider (including when URL came from `models_base_url` / `codex_base_url`, not only from model override).
2. Soften/remove the blanket “empty override ⇒ session_oauth_allowed true” narrative (04-02 ~108).
3. Add test: `provider=Xai`, `endpoints.models_base_url=https://byok.example/v1` → `session_oauth_allowed == false`; credential path does not select xAI session token.

Because this is **max cycle 3** and residual is MEDIUM (not HIGH), orchestrator may treat HIGH-convergence as achieved and proceed to execute, or run one optional replan for cleanliness.

### Next step

```text
# HIGH-converged (preferred path at max cycles):
/gsd:execute-phase 4

# Optional cleanliness if MEDIUM must be plan-locked first:
/gsd:plan-phase 4 --reviews
```

---

## Current HIGH Concerns

None.

## Current Actionable Non-HIGH Concerns

1. **MEDIUM — xAI custom `models_base_url` session OAuth trust** — Plan 02 narrative (empty override ⇒ `session_oauth_allowed: true`) conflicts with Task 2 final-URL first-party rule. Custom non-xAI `models_base_url` could be treated as trusted for OAuth if executor follows the narrative. Fix: derive trust from final URL + add regression test (`provider=Xai`, custom models_base_url → deny session OAuth). Not deferred; not yet explicit in PLAN verify list.
