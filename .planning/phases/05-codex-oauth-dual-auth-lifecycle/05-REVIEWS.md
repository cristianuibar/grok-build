---
phase: 5
reviewers: [codex]
reviewed_at: 2026-07-16T16:34:11Z
cycle: 3
plans_reviewed:
  - 05-01-PLAN.md
  - 05-02-PLAN.md
  - 05-03-PLAN.md
  - 05-04-PLAN.md
  - 05-05-PLAN.md
  - 05-06-PLAN.md
plans_commit: 39d28d0
---
# Cross-AI Plan Review — Phase 5

## Codex Review — Cycle 1

> Source: Codex gpt-5.6-sol/high plan review against live codebase (2026-07-16).
> Overall risk: **HIGH**. AUTH-05 production path and refresh lock design must rework before execute.

# Overall verdict

Revision required. Plans 05-01 through 05-04 establish a reasonable storage, CLI, and OAuth foundation, but Plan 05-05 does not yet guarantee AUTH-05 or healthy long-running dual-provider sessions. The two blockers are:

- Codex refresh is aimed at model preparation, while the actual async per-request reconstruction happens in `session/acp_session_impl/sampler_turn.rs:227-361`. Codex currently receives only a prepared snapshot, explicitly documented at `sampler_turn.rs:278-282`.
- The proposed refresh flow does not explicitly hold `auth.json.lock` across read → IdP refresh → persist. Existing xAI refresh deliberately does so to prevent rotating refresh-token reuse (`auth/manager.rs:74-78`, `1618-1651`, `1693-1725`).

Until those are corrected, overall phase risk is **HIGH**.

## 05-01 — Wave 0 RED harness

### Summary

The plan has a useful requirement-to-test map and correctly favors integration binaries over the known-broken shell library suite. However, the test-home strategy and CLI scaffold scope need revision before this is a dependable Wave 0.

### Strengths

- The named contracts cover all AUTH-02..05 behaviors, including the important production refresh and account-header proofs.
- Integration tests match the existing public-test pattern; `auth_multi_slot.rs` already tests `AuthManager` through public APIs (`tests/auth_multi_slot.rs:5-48`).
- Fake credentials and no live IdP dependency are appropriate.
- Keeping bare login xAI-compatible matches the existing parser test (`pager/src/app/cli.rs:884-925`).

### Concerns

- **HIGH — Per-test `BUM_HOME` is not deterministic in one test process.** `grok_home()` caches the first resolved home in a process-global `OnceLock` (`xai-grok-config/src/paths.rs:63-72`). Multiple lifecycle tests changing `BUM_HOME` can race or share the first test’s directory.
- **MEDIUM — CLI scaffold changes exceed the declared file list.** Adding fields or an `Auth` variant changes exhaustive matches in `pager-bin/src/main.rs:1739-1762`, but `main.rs` is absent from Plan 01’s `files_modified`.
- **MEDIUM — Verification does not exercise the new parser contracts.** The automated command runs only the already-existing bare-login test. New provider, logout conflict, and `auth status` tests could remain broken.
- **LOW — `assert!(false)` RED placeholders provide little integration value.** They prove names exist, not that tests are connected to the intended production seam.

### Suggestions

- Make lifecycle APIs accept an explicit auth path/config in tests, or run home-sensitive cases in separate subprocesses. Do not mutate `BUM_HOME` independently inside one integration binary.
- Add `pager-bin/src/main.rs` to Plan 01 or defer all clap shape changes to Plans 03–04.
- Run every new parser test plus `cargo check -p xai-grok-pager-bin`.
- Prefer compile-safe RED assertions against existing public behavior over unconditional failures.

### Risk Assessment

**MEDIUM.** The coverage design is strong, but global-home contamination could make the entire harness unreliable.

---

## 05-02 — Provider-slot RMW and status

### Summary

This plan builds on the correct storage architecture, but it contains an API-visibility contradiction and an inaccurate initial definition of “usable.”

### Strengths

- The current store already has the right full-document locking foundation and live-lock validation (`auth/storage.rs:414-449`).
- Existing xAI writes preserve sibling providers (`auth/storage.rs:451-457`), making generalization lower risk.
- Provider constants and nested schema already exist (`auth/model.rs:16-35`).
- Paste-safe formatting and always listing both providers are good AUTH-04 decisions.

### Concerns

- **HIGH — `pub(crate)` APIs cannot be called from Cargo integration tests.** `AuthDocument` is crate-private (`auth/model.rs:25-35`), and only selected read functions are publicly exported (`auth/mod.rs:38-46`). Plan 02 says `mutate_provider_store_or_prune` may remain `pub(crate)` while `tests/auth_codex_lifecycle.rs` calls it.
- **MEDIUM — `usable` cannot be derived from `select_provider_access_token`.** That selector deliberately returns an expired token when no fresh candidate exists (`auth/model.rs:343-375`). A nonblank selected key therefore does not mean usable.
- **MEDIUM — Provider identity remains stringly typed.** `read_provider_auth_store` accepts arbitrary provider strings (`auth/storage.rs:203-208`). A generic mutation API should not silently create unknown provider slots.
- **LOW — There is no dedicated plan/claim field.** `GrokAuth` has account-related fields such as `organization_id`, but no Codex plan field (`auth/model.rs:70-125`). Status should display `—` unless the schema is intentionally extended.

### Suggestions

- Either expose a small, intentional public provider-storage API or keep storage tests as module unit tests and test public login/logout behavior from the integration binary.
- Introduce a typed `AuthProvider::{Xai, Codex}` for mutation and CLI dispatch.
- Define:
  - `logged_in`: persisted OAuth session/token exists.
  - `usable`: hard-unexpired access token, or refreshable OAuth session not marked permanently invalid.
- Add status tests for expired/no-refresh-token, expired/refreshable, empty store, malformed store, and asymmetric login.

### Risk Assessment

**MEDIUM.** The storage direction is sound, but the current test/API contract cannot compile as written.

---

## 05-03 — Codex OAuth and CLI login

### Summary

The browser OAuth architecture and endpoint research are mostly well aligned with Codex, but the plan under-tests the device flow and risks passing Codex identity into xAI-specific post-login behavior.

### Strengths

- Existing dependencies already cover PKCE, loopback HTTP, browser launch, JWT parsing, and HTTP; no new dependency is needed (`xai-grok-shell/Cargo.toml:42`, `80-82`, `97`, `110-113`).
- Existing PKCE generation is correct and reusable (`auth/oidc/protocol.rs:333-345`).
- The planned issuer, ports, redirect, scopes, and device endpoints align with current official Codex implementations: [browser OAuth server](https://github.com/openai/codex/blob/main/codex-rs/login/src/server.rs), [device-code flow](https://github.com/openai/codex/blob/main/codex-rs/login/src/device_code_auth.rs).
- Writing only through the Codex provider slot directly addresses AUTH-02 and sibling preservation.

### Concerns

- **HIGH — Common post-login handling is xAI-specific.** Current `run_cli_login` passes the authenticated principal to managed-config synchronization (`auth/flow.rs:928-939`). A simple provider branch that rejoins this tail could send Codex claims into xAI deployment/team configuration logic.
- **HIGH — Expiry derivation is underspecified.** The current official Codex token response does not guarantee an `expires_in` field, while bum’s `GrokAuth` falls back to a long age-based TTL when `expires_at` is absent (`auth/model.rs:114-117`, `394-400`). That could prevent timely Codex refresh.
- **MEDIUM — The device proof validates URLs, not a working login.** Codex device auth is a multi-step usercode → token poll → authorization-code exchange flow. Existing xAI device handling also covers pending, slowdown, denial, and expiry (`auth/device_code.rs:213-288`).
- **MEDIUM — Security failure cases lack named tests.** The threat model requires state validation, but there is no explicit mismatched/missing-state, OAuth-error, or “failure writes nothing” contract.
- **MEDIUM — Port fallback and lifecycle behavior are unproven.** Existing loopback code includes timeout and graceful shutdown (`auth/oidc/login.rs:300-344`); the new path should preserve equivalent guarantees.

### Suggestions

- Branch Codex login before all xAI-only post-login synchronization.
- Derive `expires_at` from the access-token JWT `exp` when the response lacks `expires_in`; treat an unparseable expiry conservatively.
- Add mock tests for:
  - usercode request body;
  - pending and slowdown polling;
  - token result followed by authorization-code exchange;
  - final Codex-slot persistence;
  - denied/expired flows with no write.
- Add state-mismatch, token-exchange failure, callback timeout, and 1455→1457 fallback tests.
- Use a typed provider enum at the clap boundary.

### Risk Assessment

**HIGH.** OAuth login may work nominally, but expiry and device-flow gaps threaten AUTH-02 completeness and later refresh health.

---

## 05-04 — Logout and status CLI

### Summary

The intended UX correctly satisfies AUTH-03/04, but logout must use the new blocking provider-store mutation as disk authority rather than relying on the existing best-effort xAI manager clear.

### Strengths

- Bare logout fail-closed and explicit `--all` are good destructive-action semantics.
- Status output is intentionally greppable and secret-free.
- Selective logout tests cover both directions, not only Codex removal.
- Keeping remote token revocation out of scope is consistent with the phase decision.

### Concerns

- **HIGH — Existing xAI logout can report success without clearing disk.** `AuthManager::remove_scope` uses a nonblocking lock and skips persistence if another process owns it (`auth/manager.rs:483-495`). CLI logout should not use that behavior as its authoritative clear.
- **MEDIUM — `--all` could be non-atomic.** Clearing xAI and Codex through two separate locked calls permits another process to write between them.
- **MEDIUM — TUI coverage does not reach the actual ACP handler.** `/logout` emits an empty request (`pager/src/app/effects/helpers.rs:631-640`), and the shell handler calls xAI-centric `perform_logout` (`shell/src/extensions/auth.rs:119-141`). Plan 04 omits that handler from `files_modified`.
- **MEDIUM — TUI safety is not represented by an automated behavior test.** The current dispatch always emits `Effect::Logout` (`pager/src/app/dispatch/auth.rs:18-20`).
- **LOW — Status handler output capture needs an explicit seam.** A handler that directly prints to global stdout is awkward to test reliably.

### Suggestions

- Make blocking `clear_provider_slot` the disk source of truth. Then separately invalidate in-memory xAI state and telemetry identity.
- Implement `clear_all_provider_slots` as one lock-held full-document mutation.
- For TUI, either:
  - add provider-aware ACP parameters and update `extensions/auth.rs`; or
  - make `/logout` fail closed before sending the ACP request.
- Add dispatch/ACP tests proving `/logout` cannot silently clear credentials.
- Have `run_cli_auth_status` write to an injected `Write` or return a formatted string for the binary to print.

### Risk Assessment

**MEDIUM.** AUTH-03/04 are achievable, but concurrency and the existing TUI path need explicit handling.

---

## 05-05 — Independent refresh and account header

### Summary

This is the phase’s critical plan and requires redesign. As written, it may refresh during model preparation but not when a token expires later in an active session, and it does not clearly protect rotating refresh tokens across processes.

### Strengths

- It correctly requires a production call-site test rather than pure refresh-module tests.
- Permanent failure is scoped to Codex, preserving xAI.
- Snapshot invalidation and positive `ChatGPT-Account-ID` assertions are both necessary.
- Reusing the existing data-only `TokenRefresher` abstraction is reasonable.

### Concerns

- **HIGH — Wrong production hook.** `ModelsManager::sampling_config()` is synchronous and snapshots Codex only when preparing a model (`agent/models.rs:932-975`). Actual requests reconstruct credentials asynchronously in `sampler_turn.rs:227-361`, where only xAI gets a live resolver and Codex is explicitly snapshot-only (`278-282`). A token expiring mid-session will remain stale.
- **HIGH — Refresh-token rotation race.** Existing xAI logic holds the file lock across the IdP call and persistence specifically to prevent refresh-token reuse (`auth/manager.rs:1618-1651`). Plan 05’s action describes reading and later persisting via generic mutation, but not one held guard spanning the exchange.
- **HIGH — Cache invalidation does not update chat-state credentials.** Model preparation stamps the snapshot into credentials (`agent/models.rs:952-964`), and request reconstruction reads those stored credentials (`sampler_turn.rs:256-272`, `318-327`). Clearing an mtime cache does not replace an already-stamped bearer.
- **MEDIUM — `CodexRefresher` ownership is internally inconsistent.** The existing trait requires refreshers to return data and never mutate storage (`auth/refresh/mod.rs:86-143`), while Plan 05 Task 1 partly assigns clearing/persistence behavior to `CodexRefresher`.
- **MEDIUM — Transient-failure behavior is unspecified.** “Return prior token or None” must distinguish hard-unexpired from expired tokens.
- **MEDIUM — Account header trust boundary is too broad.** `inject_url_derived_headers` currently knows only a URL and alpha key (`agent/config.rs:5227-5244`). It should not read global auth state or send account metadata to arbitrary configured hosts.

### Suggestions

Redesign the production path as follows:

1. Call an async `ensure_fresh_codex_auth` from `reconstruct_full_config()` for every Codex request.
2. Under an in-process refresh mutex, acquire `auth.json.lock`.
3. Re-read `providers.codex` under that lock and adopt a sibling’s rotated token if one appeared.
4. Keep the live guard across the refresh request and persist with the guard-held mutation API.
5. Return a typed `{ bearer, account_id }` directly to request construction; do not depend on invalidating a preparation-time snapshot.
6. On transient failure:
   - use the old token only while hard-unexpired;
   - return no credential once hard-expired;
   - never clear either provider.
7. Inject `ChatGPT-Account-ID` only when the routed provider is Codex, the credential is ChatGPT OAuth, and the endpoint is the trusted ChatGPT backend.

Add tests for:

- token expires after model prepare but before the next request;
- two concurrent processes/tasks cause exactly one refresh-token spend;
- fresh tokens make zero IdP calls;
- transient failure with still-valid versus expired access token;
- account header absent on xAI, custom Codex endpoints, and non-session credentials.

### Risk Assessment

**HIGH.** AUTH-05 and dual-session health are not achieved by the current hook and locking design.

---

## 05-06 — Phase gate

### Summary

The regression matrix is useful, but it can only certify the wrong architecture unless Plan 05’s tests exercise the true per-request path and refresh concurrency.

### Strengths

- It includes lifecycle, multi-slot, and provider-routing suites.
- It keeps live OAuth optional and fixture-only CI mandatory.
- It checks both shell and binary compilation.
- It explicitly audits deferred scope.

### Concerns

- **HIGH — A green suite may still miss mid-session expiry.** The named prepare-refresh test can pass through a helper or model-preparation path without touching `sampler_turn.rs`.
- **MEDIUM — Gate-plan file declarations conflict with its action.** It permits fixes in production modules, but `files_modified` lists only tests.
- **MEDIUM — Important UI and parser surfaces are omitted.** There is no full relevant pager parser/dispatch gate.
- **MEDIUM — No formatting or lint gate.**
- **LOW — Grep-based scope audits are weak evidence.** They can miss indirect stock-home readers or endpoint behavior.

### Suggestions

- Require the AUTH-05 integration test to invoke the real async request reconstruction path.
- Add a concurrent refresh-token rotation test with a mock endpoint call counter.
- Add relevant pager CLI and `/logout` dispatch tests.
- Add `cargo fmt --all --check` and targeted clippy checks.
- Make Plan 06 a pure gate, or list all production modules it may repair.
- Replace broad greps with assertions that all credential paths resolve beneath the explicit product home.

### Risk Assessment

**MEDIUM** as a gate plan, but it cannot lower the overall phase below **HIGH** until Plan 05 is corrected.

## Requirement assessment

| Requirement | Assessment |
|---|---|
| AUTH-02 | **Partial** — browser design is strong; device completion, state failures, and expiry persistence need stronger contracts |
| AUTH-03 | **Likely achievable** — use blocking provider-scoped disk mutation and atomic clear-all |
| AUTH-04 | **Likely achievable** — fix `usable` semantics and integration-test visibility |
| AUTH-05 | **Not achieved as written** — refresh is not on the actual per-request path and rotating-token locking is incomplete |
| Dual-session health | **Not yet proven** — stale prepared Codex credentials can survive login, logout, or refresh changes |

Final recommendation: revise Plans 05-01, 05-02, 05-03, and 05-04 locally, but rework Plan 05-05 before execution. The phase should not be approved until the real async request path and cross-process refresh lock are explicit in both implementation tasks and tests.

## CYCLE_SUMMARY (orchestrator)

- unresolved HIGH: 8+ (AUTH-05 path, refresh lock, BUM_HOME OnceLock, pub(crate) integration visibility, xAI post-login on Codex, expiry JWT, logout nonblocking lock, mid-session snapshot)
- actionable non-HIGH: several MEDIUM items folded into replan

## Codex Review — Cycle 2

> Source: Codex gpt-5.6-sol/high (2026-07-16). Overall risk still HIGH (3 residual).

# Cycle 2 verdict

Overall risk: **HIGH**.

Most cycle-1 concerns are now addressed in the plan text, including the correct reconstruct-time hook and lock-held refresh design. However, three HIGH execution risks remain in AUTH-05, plus one actionable MEDIUM.

## Remaining findings

### HIGH — Codex OAuth can override BYOK credentials and leak the bearer to custom endpoints

Plan 05 invokes `ensure_fresh_codex_auth` for every model whose catalog provider is Codex, then unconditionally replaces `api_key` with the OAuth bearer ([05-05-PLAN.md:113](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-05-PLAN.md:113), [05-05-PLAN.md:212](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-05-PLAN.md:212)).

That violates the existing routing contract:

- Model-owned credentials take priority over session OAuth ([config.rs:4781](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4781)).
- Provider OAuth may attach only when `session_oauth_allowed` is true ([config.rs:4427](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4427), [config.rs:4475](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4475)).
- A custom Codex endpoint without its own credential deliberately receives no session OAuth ([config.rs:4838](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4838)).

The plan gates only `ChatGPT-Account-ID`, not the bearer itself. Require the reconstruct override only when the prepared credential is `AuthType::SessionToken` and the endpoint passes the existing Codex OAuth allowlist. Preserve `creds.api_key` for BYOK.

Add tests for:

- `codex_byok_key_not_overridden`
- `codex_oauth_bearer_absent_on_custom_endpoint`
- zero IdP refresh calls for BYOK/custom Codex routes

### HIGH — Permanent failure may deadlock by reacquiring `auth.json.lock`

The plan correctly keeps the file lock across IdP refresh and persistence ([05-05-PLAN.md:114](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-05-PLAN.md:114)), but then mandates permanent-failure cleanup “via `clear_provider_slot`” ([05-05-PLAN.md:119](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-05-PLAN.md:119)).

Plan 02 defines `clear_provider_slot` as the public acquiring operation, while only mutation has a guard-held variant ([05-02-PLAN.md:109](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-02-PLAN.md:109)). Existing acquiring mutations take the blocking lock internally ([storage.rs:419](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:419), [storage.rs:481](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:481)). Calling that while `ensure_fresh` already holds the lock can block indefinitely.

Add `clear_provider_slot_with_lock`, or clear Codex through `mutate_provider_store_or_prune_with_lock` using the existing guard. The current guard-held pattern explicitly avoids reacquisition ([storage.rs:428](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:428)).

### HIGH — The integration-only gate still cannot prove the actual reconstruct hook

Validation requires an integration test to exercise production reconstruction ([05-VALIDATION.md:15](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-VALIDATION.md:15), [05-VALIDATION.md:110](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-VALIDATION.md:110)), while forbidding the shell library-test gate.

But the source surface is private:

- `reconstruct_full_config` is `pub(super)` ([sampler_turn.rs:231](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:231)).
- `SessionActor` is `pub(crate)` ([acp_session.rs:564](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session.rs:564)).
- Its module is also `pub(crate)` ([session/mod.rs:297](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/mod.rs:297)).

Existing tests that directly call reconstruction are internal library tests ([auth_error_no_retry_tests.rs:667](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs:667)). As written, the integration test can test public `ensure_fresh_codex_auth`, but not prove that `reconstruct_full_config` actually invokes it.

The plan must name a concrete executable seam: either a headless request test that observes the outgoing bearer, a narrowly feature-gated test driver, or a runnable internal actor test. Otherwise the prior “green ensure helper, missing production wiring” failure remains possible.

### MEDIUM — Refresh-token and identity preservation are unspecified

The refresh test assumes the endpoint returns a new refresh token ([05-05-PLAN.md:157](/home/cristian/bum/grok-build/.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-05-PLAN.md:157)). OAuth refresh responses may omit it. The existing OIDC implementation preserves identity/account metadata and retains the old refresh token when rotation is absent ([oidc/refresh.rs:189](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/oidc/refresh.rs:189), [oidc/refresh.rs:208](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/oidc/refresh.rs:208)).

Require the Codex exchange to preserve:

- old refresh token when no replacement is returned;
- `organization_id`/ChatGPT account id;
- email, issuer, and client id.

Add a mock response omitting `refresh_token` and account claims.

## Production hooks verified

The revised hook selection is correct:

- Each sampler turn calls `prepare_sampler_for_turn` ([sampler_turn.rs:858](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:858)).
- That rebuilds configuration through `reconstruct_full_config` ([sampler_turn.rs:561](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:561)).
- Reconstruction currently consumes the stale chat-state `creds.api_key` ([sampler_turn.rs:256](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:256), [sampler_turn.rs:318](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:318)).

The xAI rotation-safe reference also matches the revised plan:

- in-process lock: [manager.rs:1596](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1596)
- file lock retained across refresh: [manager.rs:1618](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1618)
- live-lock revalidation immediately before IdP: [manager.rs:1642](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1642)
- IdP call and guarded outcome application: [manager.rs:1687](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1687)
- guard-held persistence without reacquisition: [manager.rs:1821](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1821)

## Resolved from prior cycle

Explicitly resolved in the revised text:

- Wrong prepare-time production hook
- Refresh-token rotation lock span and sibling adoption
- Cache invalidation as the primary freshness mechanism
- `BUM_HOME`/`OnceLock` fixture hygiene
- Public storage/status APIs for integration tests
- Codex entering xAI `post_login_sync`
- Missing `expires_in` handling
- Nonblocking `remove_scope` as logout authority
- Concurrent single-IdP-spend coverage

Not fully resolved: the reconstruct gate is named, but its integration-test execution path remains unavailable.

Recommendation: revise Plan 05 around the three HIGH findings before execution. AUTH-02 through AUTH-04 otherwise look execution-ready.
## Codex Review — Cycle 3

## Overall risk: MEDIUM

## Resolved from cycle 2

- BYOK/custom endpoint bearer override: fully addressed by the Codex + `SessionToken` + trusted-endpoint gate, zero-IdP assertions, and reconstruct-level negative tests.
- Lock-held clear deadlock: fully addressed through `clear_provider_slot_with_lock`/guard-held mutation and explicit prohibition on lock reacquisition.
- Reconstruct seam: fully addressed through registered crate-local `SessionActor::reconstruct_full_config()` tests that assert the resulting key and headers.

## Remaining HIGH (or None.)

None.

## Remaining actionable MEDIUM (or None.)

- Plan 05 Task 1 verifies outer `ensure_fresh` persistence/isolation behavior that Task 2 implements. Move those checks to Task 2 or restrict Task 1 to pure refresh tests.
- Reconstruct test commands should explicitly use `--lib`. Current crate-wide filters may also execute same-named Wave 0 integration RED stubs unless those stubs are removed or made green during Plan 05.

## Recommendation: REVISE
## Convergence

- Cycles: 3 (max)
- Final Codex risk: MEDIUM → residual MEDIUMs folded into plans (`7d8ea21`)
- Unresolved HIGH: 0
- Actionable non-HIGH remaining outside PLAN: 0
- Verdict: **CONVERGED** — ready for execute
