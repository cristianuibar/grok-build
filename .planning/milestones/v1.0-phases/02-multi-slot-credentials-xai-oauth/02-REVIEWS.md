---
phase: 02
title: Multi-slot credentials & xAI OAuth
reviewers: [codex]
cycle: 3
status: incorporated
date: 2026-07-16
incorporated_date: 2026-07-16
---

# Phase 2 Plan Reviews

Cross-AI plan review (convergence cycle 1). Reviewer: **Codex** (`gpt-5.6-sol` / high, source-grounded).

## Synthesis

| Severity | Count | Themes |
|----------|-------|--------|
| HIGH | 5 | Unlocked merge RMW; try_devbox_recovery whole-file purge; GROK_AUTH_PATH out-of-home writes; concurrent clobber; API-key unlock |
| MEDIUM | several | Unsupported schema version recovery; sampler turn seam vs helper-only Bearer proof; login mock multi-slot asserts; util filter in phase gate |
| LOW | few | write_auth_json visibility; clap parser test |

**Overall risk (Codex):** HIGH until gaps closed.

## Actionable unresolved (for replan)

### HIGH (must address in PLAN.md)

1. **Lock-scoped document mutation** — `write_auth_json` has no lock; “merge-safe” is only sequential. Add centralized locked full-document mutation (acquire `AuthFileLock`, `still_live()`, read document, mutate one provider slot, write) and concurrency tests for simultaneous xAI/Codex mutations.
2. **`try_devbox_recovery` whole-file purge** — `AuthManager::try_devbox_recovery` deletes entire `auth.json` then rewrites xAI; destroys `providers.codex`. Replace with clear-all-xAI-scopes while preserving sibling providers under lock.
3. **`GROK_AUTH_PATH` isolation hole** — AuthManager accepts arbitrary path override and writes there, including `~/.grok/auth.json`. Resolve: ignore/restrict write path to bum home, or force persist into `$BUM_HOME/auth.json` with test that `~/.grok` cannot receive writes.
4. **API-key RMW unlocked** — `store_api_key` / `clear_api_key` must use same locked mutation API as OAuth writes.
5. **Prune/delete policy incomplete** without (2) — ensure all whole-file delete sites are multi-provider-aware.

### MEDIUM (incorporate or explicit defer)

- Unsupported `version > 1`: fail closed with non-recoverable error — do **not** treat as corrupt empty-store recovery.
- Agent-turn proof: include `ShellAuthCredentialProvider` + nested → AuthManager → `sampling_config.api_key` → sampler Bearer seam (not only `GrokAuthCredentials` helpers).
- Phase gate filters must include `util::grok_auth_credentials` and `auth::credential_provider` if those tests are the proof.
- Mock browser + device-code login tests: assert nested schema, `providers.xai`, seeded codex survives, path under temp bum home.
- `FileDeleted` only after successful `remove_file`.

### LOW (optional)

- Keep `write_auth_json` / `AuthDocument` crate-private (`pub(crate)` / `pub(super)`), do not widen public surface.
- Optional Clap test for login flags / no required provider arg.

---

## Codex review (full)

> Source: `/tmp/gsd-review-grok-build-1557405645-codex-2.md` (2026-07-16)

# Plan Review — Phase 2

## Overall assessment

The four-plan decomposition is sensible: storage schema first, destructive/removal paths next, runtime credential proof in parallel, then CLI and suite gating. The plans are well grounded in the current auth architecture.

They are not ready to execute unchanged, however. Three HIGH-risk gaps remain:

1. The proposed read-merge-write is not serialized by `auth.json.lock`.
2. `try_devbox_recovery` still deletes the entire auth document.
3. `GROK_AUTH_PATH` can redirect writes outside the bum home, contradicting “credentials stored only under the bum auth store.”

Overall phase risk: **HIGH until those gaps are incorporated into executable tasks and tests.**

---

## 02-01-PLAN.md — Multi-slot schema and storage

### Summary

This is the correct foundational plan: preserve `AuthStore` as the xAI scope map while introducing an on-disk envelope at the storage boundary. Legacy migration, raw JSON shape assertions, sibling-slot preservation, and reuse of the existing atomic/fallback writer are all well chosen. The principal flaw is that “merge-safe” is only sequentially safe; without a lock around the complete read-modify-write, concurrent writers can still lose sibling data.

### Strengths

- Keeping `AuthStore` as the caller-facing xAI scope map minimizes brownfield disruption. It is currently a flat `BTreeMap<String, GrokAuth>` used throughout `AuthManager`, so adapting only at storage is a low-risk seam. [auth/model.rs:234](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/model.rs:234), [auth/manager.rs:300](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:300)

- Raw-file assertions are necessary. `read_auth_json` currently hides the physical representation by directly returning the scope map; testing only through it would not prove `providers.xai` was actually emitted. [auth/storage.rs:50](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:50)

- Reusing the current writer preserves meaningful security and recovery behavior: secure file opening, fsync, atomic rename, and the disk-full in-place fallback already exist. [auth/storage.rs:193](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:193), [auth/storage.rs:234](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:234), [auth/storage.rs:258](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:258)

- The existing OAuth implementations both converge on `AuthManager::update`, confirming that the storage adapter is the right integration point. [auth/oidc/login.rs:534](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/oidc/login.rs:534), [auth/device_code.rs:493](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/device_code.rs:493)

### Concerns

- **HIGH — The proposed merge is not safe against concurrent writers.** `write_auth_json` currently performs no locking, and its temporary name is PID-based, so even same-process concurrent writes target the same temporary path. [auth/storage.rs:193](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:193), [auth/storage.rs:258](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:258)

  The existing lock contract explicitly says writers must validate that the guard still owns the live lock before writing. [auth/storage.rs:13](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:13)

  Sequential “seed Codex, then write xAI” tests will pass even though this race remains:

  1. xAI reads document A.
  2. Codex writes document B.
  3. xAI writes its stale copy of A and removes the Codex update.

- **MEDIUM — Version handling is underspecified and potentially destructive.** If `version > 1` is silently accepted, a v1 writer may downgrade the document and discard future top-level fields. If it is mapped to ordinary `InvalidData`, the current recovery path backs up the file and proceeds with an empty store, which is also inappropriate for a valid-but-unsupported future schema. [auth/storage.rs:158](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:158)

- **LOW — “Public write_auth_json” is inaccurate.** `read_auth_json` is public, but `write_auth_json` is currently `pub(super)` and is not re-exported. The plan should preserve that restricted surface rather than accidentally widen it. [auth/storage.rs:193](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:193), [auth/mod.rs:41](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/mod.rs:41)

### Suggestions

- Add a centralized locked document mutation API, such as `mutate_auth_document`, that:

  - acquires or accepts `AuthFileLock`;
  - validates `still_live()` immediately before persistence;
  - reads the full document only after locking;
  - applies a closure to one provider slot;
  - writes before releasing the guard.

- Add a deterministic concurrent-writer test proving simultaneous xAI/Codex mutations preserve both slots.

- Either omit schema versioning for now or reject unsupported versions with a distinct non-recoverable error. Do not treat an unsupported version as corrupt JSON.

- Keep `AuthDocument` and its write API `pub(crate)` unless Phase 5 requires a wider surface.

### Risk Assessment

**HIGH.** The schema is sound, but its central “no clobber” guarantee is false under concurrency without lock-scoped mutation.

---

## 02-02-PLAN.md — API keys and pruning

### Summary

This plan correctly identifies both existing whole-file deletion hazards and centralizes the prune policy. It closes the obvious logout and API-key paths, but misses another production path that explicitly purges the whole auth document.

### Strengths

- The plan precisely targets real destructive behavior: `clear_api_key` deletes `auth.json` whenever the xAI map becomes empty. [auth/storage.rs:364](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:364)

- `AuthManager::write_scope_removal` has the same assumption and currently reports `FileDeleted` after removing the last xAI scope. [auth/manager.rs:479](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:479)

- Sharing one prune helper between API-key removal and OAuth scope removal is preferable to duplicating provider-empty logic.

- Preserving the existing `ScopeRemoval` outcome distinctions is useful because production telemetry records the disk mutation result. [auth/manager.rs:463](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:463)

### Concerns

- **HIGH — The plan misses `try_devbox_recovery`, which unconditionally deletes the complete file.** That method explicitly purges `auth.json` and then writes new xAI credentials. Once a Codex slot exists, this destroys it. [auth/manager.rs:1390](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1390), [auth/manager.rs:1428](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1428)

- **HIGH — API-key mutation remains unlocked.** `store_api_key` currently does a read-modify-write without acquiring `auth.json.lock`; `clear_api_key` is similarly unlocked. The plan says to keep the lock unchanged but never requires acquiring it around the full mutation. [auth/storage.rs:350](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:350), [auth/storage.rs:365](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:365)

- **MEDIUM — File-deletion outcomes must check actual deletion.** Current code ignores `remove_file` errors in both clear paths. Returning `FileDeleted` when deletion failed makes telemetry and tests misleading. [auth/storage.rs:369](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/storage.rs:369), [auth/manager.rs:487](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:487)

### Suggestions

- Add `try_devbox_recovery` to this plan’s files and tasks. Replace whole-file purge with “clear all xAI scopes while preserving sibling providers,” using the same locked document mutation API.

- Require the prune helper to operate under a live `AuthFileLock`.

- Add tests for:

  - Codex survives devbox xAI recovery/purge.
  - API-key store/clear racing with a sibling mutation preserves both providers.
  - `FileDeleted` is returned only after successful deletion; deletion errors propagate.

### Risk Assessment

**HIGH.** The intended policy is correct, but a production whole-file purge and unlocked mutations remain outside the plan.

---

## 02-03-PLAN.md — AuthManager and agent credential proof

### Summary

The AuthManager load/update tests are valuable and should catch adapter regressions. The credential proof, however, is aimed partly at helper surfaces rather than the primary sampler path used for an actual agent turn. As written, it proves that a credential can reach a bearer-capable helper, not fully that the coding-turn sampler uses it.

### Strengths

- `AuthManager::new` really does load through `read_auth_json`, while `update` persists through `write_auth_json`; the planned nested-load and sibling-preservation tests exercise the correct boundary. [auth/manager.rs:300](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:300), [auth/manager.rs:794](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:794)

- Non-expired fixtures can prove `get_valid_token` without a live IdP or refresh call.

- `ShellAuthCredentialProvider` obtains the in-memory xAI token and `GrokAuthCredentials::apply` attaches the expected Bearer and xAI token-auth header. [auth/credential_provider.rs:42](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/credential_provider.rs:42), [util/grok_auth_credentials.rs:119](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs:119)

### Concerns

- **MEDIUM — The proposed credential test does not necessarily prove the primary agent-turn path.** Main inference seeds `sampling_config.api_key` directly from `AuthManager`, passes it into the per-turn sampling config, and the sampler constructs the Bearer header. [agent_ops.rs:2344](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:2344), [sampler_turn.rs:310](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:310), [sampler/client.rs:419](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampler/src/client.rs:419)

  `GrokAuthCredentials::with_auth_manager` currently appears only in its own tests, so using that alone would overstate “agent-turn proof.” [util/grok_auth_credentials.rs:58](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs:58), [util/grok_auth_credentials.rs:141](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs:141)

- **MEDIUM — The phase-gate command may omit the util credential test.** `grok_auth_credentials` lives under `util`, not `auth`, so Plan 04’s `auth::` filter will not include it. [util/mod.rs:2](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/util/mod.rs:2), [auth/mod.rs:1](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/mod.rs:1)

- **LOW — Tests call `AuthManager::update` without satisfying its documented lock invariant.** This is common in existing tests, but a new storage-integrity test should not normalize an ambiguous production contract. [auth/manager.rs:784](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:784)

### Suggestions

- Make `ShellAuthCredentialProvider` coverage mandatory, not optional.

- Add one targeted actual-turn seam test:

  `nested auth.json → AuthManager → sampling_config.api_key → sampler request → Authorization: Bearer`.

- Use separate valid filters, for example:

  - `cargo test -p xai-grok-shell --lib auth::manager::tests::`
  - `cargo test -p xai-grok-shell --lib auth::credential_provider::tests::`
  - `cargo test -p xai-grok-shell --lib util::grok_auth_credentials::tests::`

- Clarify whether `update` requires a caller-held guard or owns locking after the Plan 01 refactor.

### Risk Assessment

**MEDIUM.** The runtime will probably continue working through the adapter, but the planned evidence does not fully substantiate the “agent turn” success criterion.

---

## 02-04-PLAN.md — CLI/path seal and phase gate

### Summary

The CLI wiring and default bum-home path are correctly identified. Existing browser and device-code implementations already persist through `AuthManager`, so a compile and full suite are useful regression gates. The major unresolved issue is that `GROK_AUTH_PATH` still permits credential persistence outside the bum store.

### Strengths

- The CLI already exposes mutually exclusive `--oauth` and `--device-auth` flags with no provider argument. [pager/app/cli.rs:20](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager/src/app/cli.rs:20)

- The composition root routes those flags directly to the four-argument `run_cli_login` call, matching the plan. [pager-bin/main.rs:1739](/home/cristian/bum/grok-build/crates/codegen/xai-grok-pager-bin/src/main.rs:1739)

- Product-home resolution is genuinely bum-specific by default: non-empty `BUM_HOME`, otherwise `~/.bum`; `GROK_HOME` is deliberately not an input. [config/paths.rs:15](/home/cristian/bum/grok-build/crates/codegen/xai-grok-config/src/paths.rs:15), [config/paths.rs:63](/home/cristian/bum/grok-build/crates/codegen/xai-grok-config/src/paths.rs:63)

- The stale device-code documentation is real and correctly scoped for repair. [auth/device_code.rs:205](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/device_code.rs:205)

- Existing mock coverage is useful: loopback exercises callback, exchange, and persistence; device-code tests exercise polling and successful token construction. [auth/oidc/login.rs:560](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/oidc/login.rs:560), [auth/device_code.rs:843](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/device_code.rs:843)

### Concerns

- **HIGH — `GROK_AUTH_PATH` contradicts bum-only credential storage.** `AuthManager::new` accepts an arbitrary path override, and subsequent updates write to that path. It can therefore persist directly into `~/.grok/auth.json` if configured. [auth/manager.rs:295](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:295), [auth/manager.rs:815](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:815)

  Checking only the default `{grok_home}/auth.json` path does not satisfy the locked “only under bum product home” decision.

- **MEDIUM — Device-code success does not currently assert on-disk persistence.** Its test helper creates a temporary manager and returns only the auth result; the temp directory is dropped before the test can inspect `auth.json`. [auth/device_code.rs:815](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/device_code.rs:815), [auth/device_code.rs:843](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/device_code.rs:843)

- **MEDIUM — The loopback persistence assertion checks token text, not the multi-slot schema or sibling survival.** [auth/oidc/login.rs:653](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/oidc/login.rs:653)

- **LOW — Compile/inspection alone does not protect the CLI contract.** A small Clap parser test would better pin aliases, mutual exclusion, and the absence of a required provider argument.

### Suggestions

- Decide explicitly how `GROK_AUTH_PATH` fits the isolation requirement:

  - remove/ignore it for writable auth storage;
  - restrict it to the resolved bum `auth.json`;
  - or make it read-only and persist all refreshed/login credentials into bum home.

  Add an environment-guarded test proving a value pointing at `~/.grok/auth.json` cannot receive writes.

- Extend both mock login tests to assert:

  - `version == 1`;
  - credentials appear under `providers.xai`;
  - a seeded `providers.codex` payload survives login;
  - no alternate home/path is touched.

- Make the phase gate include the util credential tests or run the entire shell library suite.

### Risk Assessment

**HIGH.** Default path behavior is correct, but the writable path override leaves AUTH-01’s isolation guarantee unfulfilled.

---

## Required revisions before execution

1. Add lock-scoped full-document mutation and concurrency tests to 02-01/02-02.
2. Cover the whole-file purge in `try_devbox_recovery`.
3. Resolve or explicitly amend the requirement conflict around `GROK_AUTH_PATH`.
4. Test the actual sampler bearer path, not only credential helper surfaces.
5. Add on-disk multi-slot assertions to both browser and device-code mock login tests.
6. Define fail-closed handling for unsupported schema versions.

With those changes incorporated, the wave structure and overall approach should converge to **LOW–MEDIUM implementation risk**.
---

# Cycle 2 Codex re-review

Date: 2026-07-16

## Resolution checklist (prior findings)

1. **STILL OPEN — Lock-scoped document mutation + concurrency tests.**  
   Storage-level locking and concurrency tests are specified in [02-01-PLAN.md:110](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-01-PLAN.md:110) and [02-01-PLAN.md:141](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-01-PLAN.md:141). However, the plan makes `write_auth_json` acquire the lock while only “documenting for later” how callers already holding the lock should use a guard-aware variant ([02-01-PLAN.md:232](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-01-PLAN.md:232)).

   Existing production callers already hold `auth.json.lock` before calling `write_auth_json`, including startup cleanup ([manager.rs:314](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:314)), scope removal ([manager.rs:455](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:455)), refresh persistence ([manager.rs:1737](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:1737)), and enrichment ([enrichment.rs:145](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs:145)). Enrichment also explicitly permits unlocked RMW after timeout ([enrichment.rs:142](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs:142)). Unconditionally relocking inside `write_auth_json` risks self-deadlock; leaving these call sites unchanged violates the “all production mutations locked” must-have.

   The plan needs executable tasks covering `manager.rs`, `manager/enrichment.rs`, and `manager/lock.rs`, with distinct acquiring and caller-held-guard mutation APIs plus tests for already-locked update/enrichment paths.

2. **RESOLVED — `try_devbox_recovery` sibling preservation.**  
   Explicit implementation and seeded-Codex tests are required in [02-02-PLAN.md:179](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-02-PLAN.md:179) and [02-02-PLAN.md:189](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-02-PLAN.md:189).

3. **RESOLVED — `GROK_AUTH_PATH` direct out-of-home writes.**  
   Product-home-only persistence and a foreign-path regression test are specified in [02-04-PLAN.md:124](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-04-PLAN.md:124) and [02-04-PLAN.md:125](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-04-PLAN.md:125).

4. **RESOLVED — locked `store_api_key` / `clear_api_key`.**  
   Both helpers are routed through locked full-document mutation in [02-02-PLAN.md:146](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-02-PLAN.md:146).

5. **RESOLVED — all whole-file purges multi-provider-aware.**  
   The plan mandates a production-wide purge audit and shared prune policy in [02-02-PLAN.md:182](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-02-PLAN.md:182) and [02-02-PLAN.md:190](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-02-PLAN.md:190).

Medium findings are incorporated:

- Unsupported versions fail closed: [02-01-PLAN.md:181](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-01-PLAN.md:181).
- Agent-turn and sampler Bearer seam: [02-03-PLAN.md:154](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-03-PLAN.md:154).
- Browser/device mock multi-slot assertions: [02-04-PLAN.md:168](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-04-PLAN.md:168).
- Honest `FileDeleted`: [02-02-PLAN.md:188](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-02-PLAN.md:188).
- Complete phase-gate filters: [02-04-PLAN.md:182](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-04-PLAN.md:182).

## New concerns

### HIGH — Lexical containment permits symlink escape

The `GROK_AUTH_PATH` task permits either canonicalization **or `starts_with(product_home)`** ([02-04-PLAN.md:124](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-04-PLAN.md:124)), while its test covers only a directly foreign path ([02-04-PLAN.md:125](/home/cristian/bum/grok-build/.planning/phases/02-multi-slot-credentials-xai-oauth/02-04-PLAN.md:125)). A lexically in-home path can traverse a symlink to `~/.grok` or elsewhere. The actual secure writer uses ordinary `OpenOptions::open`, which follows symlinks ([secure_file.rs:72](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell-base/src/util/secure_file.rs:72), [secure_file.rs:82](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell-base/src/util/secure_file.rs:82)).

Require canonical containment of the existing parent—or simply accept only the exact product-home `auth.json`—and add a Unix symlink-escape regression test. Also correct the plan’s nonexistent `xai-grok-shell/src/util/secure_file.rs` reference; the implementation is in `xai-grok-shell-base`.

## Risk Assessment

**HIGH.** Two execution-blocking concerns remain: lock ownership/reentrancy across existing writers and symlink-safe home containment.

## Summary

Cycle 2 is not yet converged. Four prior HIGH findings and all prior MEDIUM findings are incorporated, but the central locking change is not safely threaded through existing lock-holding writers, and the path-isolation design still permits a symlink escape.

---

# Cycle 2 incorporation (planner)

Date: 2026-07-16  
Status: **incorporated** into PLAN.md files (executable contract updated).

| Finding | Severity | Plan change |
|---------|----------|-------------|
| Lock reentrancy / dual mutation API + rewire lock-holders | HIGH | **02-01** Task 1 + Task 3: acquiring `mutate_auth_document` vs guard-held `mutate_auth_document_with_lock`; rewire manager startup cleanup, scope removal, update/save, enrichment; already-locked no-deadlock tests. **02-02** prune/devbox/scope-removal keep dual-API discipline (with-lock when held). |
| GROK_AUTH_PATH symlink escape / lexical starts_with | HIGH | **02-04** Task 1: accept only exact product-home `auth.json` **or** `dunce::canonicalize` parent containment; forbid starts_with-only; Unix symlink-escape regression test; secure_file path corrected to `xai-grok-shell-base`. |
| Prior HIGH/MEDIUM (devbox, API keys, purges, version, agent seam, mocks, FileDeleted, phase gate) | — | Kept; not regressed. |
---

# Cycle 3 Codex re-review — CONVERGED

Date: 2026-07-16

## Resolution checklist

1. **RESOLVED — Dual mutation API**
   - Plan 01 now requires distinct acquiring and caller-held-guard APIs.
   - Startup cleanup, scope removal, refresh persistence, and locked enrichment are explicitly rewired to the guard-held path.
   - Enrichment after lock timeout uses the acquiring merge path.
   - Concurrency and already-locked no-deadlock tests are mandatory.
   - This matches the current lock ownership visible in [manager.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:314) and [enrichment.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs:145).

2. **RESOLVED — Symlink-safe product-home containment**
   - Plan 04 permits only exact product-home `auth.json` or canonical-parent containment using `dunce::canonicalize`.
   - Lexical `starts_with` alone is explicitly forbidden.
   - A Unix symlink-escape regression test is required.
   - This directly addresses the current arbitrary override in [manager.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/auth/manager.rs:295) and the symlink-following writer in [secure_file.rs](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell-base/src/util/secure_file.rs:72).

3. **Prior MEDIUMs**
   - Unsupported-version fail-closed handling: incorporated.
   - Honest `FileDeleted`: incorporated.
   - Primary agent-turn Bearer seam: incorporated.
   - Browser/device-code multi-slot assertions: incorporated.
   - Credential-provider/util phase-gate filters: incorporated.

**Unresolved HIGH: 0**  
**Actionable MEDIUM absent from PLAN.md: 0**

## New concerns (only if blocking)

None.

## Risk Assessment (MEDIUM)

The plans are complete, but implementation remains moderately intricate because production has both unlocked login writers and caller-held refresh/enrichment writers. The dual API contract and required no-deadlock tests adequately control that risk.

## Summary

**CONVERGED.** Both cycle-2 HIGH findings are fully incorporated, all previously actionable MEDIUM findings remain covered, and no new blocking gap was found.