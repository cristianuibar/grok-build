---
phase: 10
slug: codex-responses-wire-parity
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-07-18
evidence_recorded: 2026-07-18
automated_suite: recorded
live_ops: pending
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for Codex Responses wire parity and the two Phase 9 reliability blockers it closes.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` crate-local unit and integration suites |
| **Config file** | Workspace `Cargo.toml`; no new framework setup |
| **Quick run command** | `cargo test -p xai-grok-sampling-types encrypted_content --lib` |
| **Full suite command** | Focused sampling-types, sampler, and shell commands specified in each PLAN.md |
| **Estimated runtime** | Under 60 seconds per focused command, subject to normal Rust incremental build state |
| **Consolidated run (Plan 10-05)** | 2026-07-18 — see [Automated Evidence (Plan 10-05)](#automated-evidence-plan-10-05) |

## Sampling Rate

- **After every task commit:** Run the task's focused `<automated>` command.
- **After every plan wave:** Run all focused suites affected by that wave.
- **Before `$gsd-verify-work`:** Run `cargo fmt --all -- --check` and the consolidated Phase 10 suite.
- **Max feedback latency:** one focused Rust test command; do not use watch mode.

## Per-Task Verification Map

| Behavior | Requirements | Threat Ref | Automated Proof | 10-05 Result |
|----------|--------------|------------|-----------------|--------------|
| Profile disabled by default and cannot leak through sampler or shell literals | OPS-04 | T-10-01 | sampler-core plus shell production/test-fixture compile checks | **PASS** (profile + shell check/`--no-run`) |
| Trusted request body preserves generic conversion while applying profile body fields and tools-only controls | OPS-04 | T-10-02, T-10-03 | sampling-types/sampler serialization tests | **PASS** |
| Delta-only completed response is non-empty; terminal text is not duplicated; multi-output order and incomplete behavior remain intact; genuine empty retry remains bounded | OPS-04 | T-10-04, T-10-05 | sampler stream unit tests plus `test_actor` request-counter controls | **PASS** |
| Cross-provider switch retains ordinary context but removes foreign encrypted payloads, including a late old-provider response | OPS-05 | T-10-06, T-10-07 | actor-owned provider/epoch unit tests plus deterministically held scripted-response capture | **PASS** |
| Trusted headers do not leak and session metadata remains stable | OPS-04 | T-10-08, T-10-09 | reconstruction matrix plus actual outbound header capture, collision stripping, and trusted-to-untrusted switch regression | **PASS** |
| Encrypted-content variants are classified only as HTTP 400 recovery cases | OPS-05 | T-10-10 | sampling-types predicate, shell classification, and end-to-end one-request/no-compaction recovery test | **PASS** |
| Live evidence is not fabricated, secret-bearing, or falsely marked PASS | OPS-04, OPS-05 | T-10-11, T-10-12, T-10-13 | redacted human checkpoint and validation-artifact inspection | **pending live** (not PASS) |

## Wave 0 Requirements

- [x] Existing Rust test infrastructure covers all Phase 10 requirements.
- [x] Each plan names concrete regression tests and a non-vacuous focused `cargo test` command.

## Automated Evidence (Plan 10-05)

**Run date (UTC):** 2026-07-18  
**Operator:** executor (Plan 10-05 Task 1)  
**Host workspace:** `/home/cristian/bum/grok-build`  
**Method:** Each command from `10-05-PLAN.md` Task 1 `<automated>` chain run independently; status is the real exit code.  
**Log (local, uncommitted):** `/tmp/p10-05-evidence/run.log`  
**Honesty rule:** No live dual-login PASS is claimed from these results (T-10-11).

### Command results

| # | Command (short name) | Exact command | Result | Duration | OPS layer | Owning plan |
|---|----------------------|---------------|--------|----------|-----------|-------------|
| 1 | sampler_profile_defaults | `cargo test -p xai-grok-sampler --lib client::tests::responses_wire_profile_defaults_and_propagates_to_client_defaults -- --exact` | **PASS** | 5s | OPS-04 T-10-01 profile defaults | 10-01 |
| 2 | sampler_profile_on_off | `cargo test -p xai-grok-sampler --lib client::tests::trusted_codex_responses_profile_on_off_serializes_exactly -- --exact` | **PASS** | 1s | OPS-04 T-10-02/03 body serialization | 10-01 |
| 3 | sampler_lib_no_run | `cargo test -p xai-grok-sampler --lib --no-run` | **PASS** | 1s | OPS-04 T-10-01 compile | 10-01 |
| 4 | sampler_test_actor_no_run | `cargo test -p xai-grok-sampler --test test_actor --no-run` | **PASS** | 1s | OPS-04 T-10-04/05 compile | 10-02 |
| 5 | sampling_types_no_system_role | `cargo test -p xai-grok-sampling-types --lib conversation::tests::responses_api_never_emits_system_role_in_input -- --exact` | **PASS** | 1s | OPS-04 request shaping | prior + 10-01 |
| 6 | sampling_types_encrypted_predicate | `cargo test -p xai-grok-sampling-types --lib error::tests::encrypted_content_error_accepts_spaced_case_insensitive_400_only -- --exact` | **PASS** | <1s | OPS-05 T-10-10 | 10-04 |
| 7 | stream_delta_fallback | `cargo test -p xai-grok-sampler --lib stream::responses::tests::text_delta_then_completed_uses_fallback_when_terminal_empty -- --exact` | **PASS** | 1s | OPS-04 T-10-04 delta-only | 10-02 |
| 8 | stream_terminal_wins | `cargo test -p xai-grok-sampler --lib stream::responses::tests::text_delta_then_completed_terminal_text_wins -- --exact` | **PASS** | 1s | OPS-04 T-10-04 no terminal dup | 10-02 |
| 9 | stream_multi_output | `cargo test -p xai-grok-sampler --lib stream::responses::tests::text_delta_then_completed_multi_output_index_preserves_arrival_order -- --exact` | **PASS** | <1s | OPS-04 multi-output order | 10-02 |
| 10 | stream_incomplete | `cargo test -p xai-grok-sampler --lib stream::responses::tests::text_delta_then_incomplete_preserves_length_without_fallback -- --exact` | **PASS** | <1s | OPS-04 incomplete path | 10-02 |
| 11 | actor_delta_only_once | `cargo test -p xai-grok-sampler --test test_actor delta_only_terminal_empty_completes_once_without_retry -- --exact` | **PASS** | 1s | OPS-04 T-10-05 once-complete | 10-02 |
| 12 | actor_empty_retry_bounded | `cargo test -p xai-grok-sampler --test test_actor genuinely_empty_completed_response_retries_boundedly -- --exact` | **PASS** | 3s | OPS-04 T-10-05 bounded empty retry | 10-02 |
| 13 | shell_cross_provider_sanitize | `cargo test -p xai-grok-shell --lib session::acp_session::model_switch::tests::cross_provider_transition_sanitizes_existing_and_late_reasoning -- --exact` | **PASS** | 3s | OPS-05 T-10-06/07 | 10-03 |
| 14 | shell_same_provider_preserve | `cargo test -p xai-grok-shell --lib session::acp_session::model_switch::tests::same_provider_codex_transition_preserves_encrypted_reasoning -- --exact` | **PASS** | 1s | OPS-05 same-provider keep | 10-03 |
| 15 | shell_unknown_prior_sanitize | `cargo test -p xai-grok-shell --lib session::acp_session::model_switch::tests::unknown_prior_provider_sanitizes_encrypted_reasoning -- --exact` | **PASS** | 1s | OPS-05 unknown prior | 10-03 |
| 16 | gate_held_grok_after_codex | `cargo test -p xai-grok-shell --test model_switch_gate held_grok_response_after_codex_switch_is_sanitized -- --exact` | **PASS** | 35s | OPS-05 T-10-07 held late response | 10-03 |
| 17 | gate_codex_to_codex | `cargo test -p xai-grok-shell --test model_switch_gate codex_to_codex_transition_retains_encrypted_reasoning -- --exact` | **PASS** | 34s | OPS-05 codex→codex retain | 10-03 |
| 18 | shell_reconstruct_metadata | `cargo test -p xai-grok-shell --lib session::acp_session::codex_reconstruct_refresh_tests::trusted_codex_reconstruct_enables_profile_and_metadata -- --exact` | **PASS** | 1s | OPS-04 T-10-08/09 metadata | 10-04 |
| 19 | gate_trusted_headers_sent | `cargo test -p xai-grok-shell --test model_switch_gate trusted_codex_wire_headers_are_sent_and_stable -- --exact` | **PASS** | 33s | OPS-04 T-10-08 headers | 10-04 |
| 20 | gate_headers_no_leak | `cargo test -p xai-grok-shell --test model_switch_gate codex_wire_headers_do_not_leak_to_xai_byok_or_custom -- --exact` | **PASS** | 37s | OPS-04 T-10-08 no leak | 10-04 |
| 21 | gate_trusted_to_untrusted | `cargo test -p xai-grok-shell --test model_switch_gate trusted_to_untrusted_switch_strips_codex_identity_headers -- --exact` | **PASS** | 33s | OPS-04 T-10-09 declassify | 10-04 |
| 22 | shell_encrypted_terminal | `cargo test -p xai-grok-shell --lib session::acp_session::auth_error_no_retry_tests::encrypted_content_400_is_classified_terminal -- --exact` | **PASS** | 1s | OPS-05 T-10-10 classify | 10-04 |
| 23 | gate_encrypted_no_compaction | `cargo test -p xai-grok-shell --test model_switch_gate encrypted_content_400_is_terminal_before_compaction_or_resubmit -- --exact` | **PASS** | 33s | OPS-05 T-10-10 no compact/resubmit | 10-04 |
| 24 | shell_check_lib | `cargo check -p xai-grok-shell --lib` | **PASS** | 136s | OPS-04 T-10-01 production literals | 10-06 |
| 25 | shell_lib_no_run | `cargo test -p xai-grok-shell --lib --no-run` | **PASS** | 2s | OPS-04 T-10-01 test literals | 10-07 |
| 26 | shell_provider_routing_no_run | `cargo test -p xai-grok-shell --test provider_routing --no-run` | **PASS** | 1s | OPS-04 routing compile | 10-04/07 |
| 27 | fmt_check | `cargo fmt --all -- --check` | **FAIL** | 13s | hygiene (not OPS behavior) | **deferred** — pre-existing workspace fmt drift |

### Automated suite summary

| Bucket | Count | Notes |
|--------|-------|-------|
| Behavior / compile focused commands PASS | **26 / 26** | All sampler, sampling-types, shell unit/integration, and compile gates green |
| Hygiene FAIL | **1** | `cargo fmt --all -- --check` — see [Deferred items](#deferred-items) |
| Invented live PASS | **0** | Live OPS rows remain non-pass until redacted dual-login |

**Behavior-layer verdict:** Automated Phase 10 reliability and wire contracts are **green**.  
**Hygiene verdict:** Workspace-wide rustfmt check is **non-pass** (out of product scope for 10-05; does not waive live OPS).  
**Phase / OPS live verdict:** **not green** — fixtures alone cannot satisfy OPS-04/OPS-05 (T-10-11).

### Output snippets (no secrets)

Representative `test result: ok` lines from the consolidated run:

- `client::tests::responses_wire_profile_defaults_and_propagates_to_client_defaults ... ok`
- `client::tests::trusted_codex_responses_profile_on_off_serializes_exactly ... ok`
- `conversation::tests::responses_api_never_emits_system_role_in_input ... ok`
- `error::tests::encrypted_content_error_accepts_spaced_case_insensitive_400_only ... ok`
- `stream::responses::tests::text_delta_then_completed_uses_fallback_when_terminal_empty ... ok`
- `delta_only_terminal_empty_completes_once_without_retry ... ok`
- `genuinely_empty_completed_response_retries_boundedly ... ok`
- `cross_provider_transition_sanitizes_existing_and_late_reasoning ... ok`
- `held_grok_response_after_codex_switch_is_sanitized ... ok`
- `trusted_codex_wire_headers_are_sent_and_stable ... ok`
- `codex_wire_headers_do_not_leak_to_xai_byok_or_custom ... ok`
- `encrypted_content_400_is_terminal_before_compaction_or_resubmit ... ok`

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| GPT-5.6 daily-driver turn completes once after Codex login | OPS-04 | Requires real dual-login session and actual service response | Rebuild `bum`; select `gpt-5.6-sol`; send `hi`; then read/edit a file; retain only redacted evidence and confirm no retry. |
| Grok → GPT-5.6 session switch remains productive | OPS-05 | Requires real provider credentials and switched live history | In one real session use Grok, switch to `gpt-5.6-sol`, send `hi`, and confirm no encrypted-content 400 while ordinary context remains usable. |

## Live OPS evidence (Plan 10-05 Task 2)

> **Secrets policy (T-10-12):** No tokens, auth files, account IDs, or raw secret-bearing transcripts.  
> **Outage policy (T-10-13):** Auth, account, provider outage, or product failure must be recorded as an explicit non-pass class — never inferred PASS.

### Preflight (executor)

| Step | Result | Notes |
|------|--------|-------|
| Consolidated automated suite (behavior) | **PASS** (26/26) | See table above |
| `cargo fmt --all -- --check` | **FAIL** | Deferred hygiene; not a live-auth blocker |
| Dual OAuth sessions usable | **PASS** (operator 2026-07-18) | Both xAI and Codex slots usable; prompts to both work |
| Post-fix binary rebuild | **PASS** | Operator retested on post-fix `target/debug/bum` |

### OPS-04 — GPT-5.6 daily-driver turn (live)

| Field | Value |
|-------|-------|
| **Requirement** | OPS-04 |
| **Status** | **PASS** (operator) |
| **Model** | GPT-5.6 / Codex models |
| **Turn completes once (no post-success retry)** | ✅ no post-success retry |
| **Read/edit/tool after reply** | ✅ dual-login daily-driver turns usable |
| **Redacted outcome** | Successful GPT turns without retry storm |
| **Non-pass class (if not PASS)** | — |
| **Date** | 2026-07-18 |

**PASS criteria:** One completed GPT-5.6 turn with no post-success retry, plus a normal read/edit/tool action, with redacted observations only.

### OPS-05 — Grok → GPT-5.6 same-process switch (live)

| Field | Value |
|-------|-------|
| **Requirement** | OPS-05 |
| **Status** | **PASS** (operator retest after store=false id-strip fix) |
| **Models** | Grok → **gpt-5.6-luna** (Codex) |
| **No encrypted-content / decryption 400** | ✅ none observed |
| **No store-false item-id 404** | ✅ none after fix |
| **Ordinary prior context still useful** | ✅ switch continued productively |
| **Redacted outcome** | Grok turn then switch to GPT Luna completed without 404/errors; session remained usable |
| **Non-pass class (if not PASS)** | — (earlier failure on pre-fix binary was `product_failure` / `rs_*` store 404 — fixed) |
| **Date** | 2026-07-18 retest |

**Root cause (fixed):** Residual `rs_*` ids under `store: false` → 404. Wire strip via `strip_input_item_ids_for_store_false`. See research §4.3.

**PASS criteria:** Same bum process: Grok turn, switch to GPT-5.6, follow-up succeeds without encrypted-content 400 **and** without store-false item-id 404.

### Related: model identity (product quality — not OPS gate)

| Field | Value |
|-------|-------|
| **Symptom (initial)** | On Grok / early session, self-report as Grok; template still said “Grok Build” |
| **After switch to GPT Luna** | Correctly: “You’re using GPT-5.6 Luna (Codex) in the bum CLI/TUI” |
| **Interpretation** | Routing to Codex works; identity follows `system_prompt_label` after switch. Residual “Grok Build” product wording in templates can still bias answers until rebranded |
| **Follow-up** | Agent templates rebranded to bum; catalog labels for GPT + Grok; rebuild binary to pick up prompt encrypt |

### Operator sign-off

- [x] Live OPS-04 PASS  
- [x] Live OPS-05 PASS (Grok → gpt-5.6-luna, no 404)

## Deferred items

| Item | Class | Impact | Owner |
|------|-------|--------|-------|
| `cargo fmt --all -- --check` fails across ~68 files (pager, shell auth, config, models, tools, update, etc.) | Pre-existing style drift; not introduced by 10-05 evidence task | Does not break focused Phase 10 behavior tests; blocks full-workspace fmt hygiene gate | Out of scope for 10-05 product validation — track as workspace hygiene / separate chore; do **not** mass-`cargo fmt` mid-phase without an explicit format plan (see Phase 10 process note on formatter spill) |

## Validation Sign-Off

- [x] All approved plan tasks have `<automated>` verification.
- [x] Sampling continuity has no three consecutive tasks without automated proof.
- [x] Existing infrastructure covers all required test layers.
- [x] No watch-mode flags are permitted.
- [x] Focused automated evidence recorded (Plan 10-05 Task 1, 2026-07-18) — 26/26 behavior commands PASS.
- [ ] `cargo fmt --all -- --check` green (deferred hygiene FAIL recorded above).
- [x] Live OPS-04 redacted PASS recorded (Task 2 human).
- [x] Live OPS-05 redacted PASS recorded (Task 2 human) — Grok → gpt-5.6-luna.
- [ ] `nyquist_compliant: true` is set only after full phase gate (fmt hygiene still deferred).

**Approval:** automated suite green; **live OPS-04 and OPS-05 PASS** (2026-07-18). Phase 9 hybrid green can proceed when remaining Phase 9 plans close. No secrets in this artifact (T-10-12).
