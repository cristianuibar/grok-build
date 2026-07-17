# NeevCloud on bum — doc set index

## Bottom line

NeevCloud is a **third provider slot** next to `providers.xai` and `providers.codex`. Two halves, both cheap:

- **Inference side is mostly config.** The NeevCloud gateway is plain OpenAI Chat Completions at `{baseURL}/chat/completions` with `Authorization: Bearer <key>`. `ApiBackend::ChatCompletions` (`crates/codegen/xai-grok-sampling-types/src/types.rs:1013-1021`) is already the `#[default]`, `AuthScheme::Bearer` (`crates/codegen/xai-grok-sampler/src/config.rs:18-24`) is already the default, and `SamplingClient::endpoint()` (`crates/codegen/xai-grok-sampler/src/client.rs:844-848`) is a naive `{base}/{path}` join that produces the right URL from `https://code.neevcloud.com/zen/go/v1`. **Zero sampler/stream/retry code.** Populate a `SamplerConfig` and you're done.
- **Auth side rhymes with the existing device flow.** `auth/codex/device.rs` is a JSON-bodied device-code login that already does poll/`slow_down`/terminal classification/persist-only-after-success — but it is Codex's *proprietary* 3-step deviceauth (usercode → poll → authorization-code exchange at `/oauth/token`), explicitly "not RFC 8628" (`auth/codex/device.rs:1-7`). NeevCloud is RFC-8628-shaped and has no code-exchange step, so copy the poll/backoff/persist scaffolding, not the flow. `AuthDocument.providers` is a `BTreeMap<String, AuthStore>` — a third slot needs **no schema change and no `AUTH_DOCUMENT_VERSION` bump**.

The actual work is neither of those. It is **widening ~40 hard-coded-two sites** (`AuthProvider::all() -> [Self; 2]` at `auth/model.rs:61`, `AuthStatusReport.providers: [ProviderAuthStatus; 2]` at `auth/status.rs:37-39`, `ProviderAuthMetaSlots{xai, codex}`, and a set of `matches!(provider, Codex)` guards that **compile fine and silently take the xAI branch**). See [04-rust-implementation.md § 9](04-rust-implementation.md#9-provider-registration--the-23-widening).

On top of both halves sits one adopted design decision: **bum's NeevCloud traffic impersonates the neev CLI on the wire** — same User-Agent, same `x-opencode-*` headers, same `client_id`. Measured fact: *none of it is required today* (a bare request to the gateway returns 200). It is forward-compatibility insurance against a future identity gate, and it is the spec's job to keep every pinned string in one greppable, env-overridable place. See [07-wire-fidelity.md](07-wire-fidelity.md).

There is no `Provider` trait in this workspace. Do not build one.

## Why bother

Free tier, verified live against the real service on Cristian's own account:

| | |
|---|---|
| Quota | **10,000,000 tokens/month**, plan `basic` / label "Free" |
| Models | 8: `minimax-m2`, `minimax-m3`, `qwen3.6-plus`, `qwen3.7-plus`, `qwen3.7-max`, `deepseek-v4-pro`, `glm-5.2`, `nemotron-3-ultra` |
| Wire | OpenAI Chat Completions — the boring, already-supported path |
| Token life | ~1 year, single static bearer (no refresh churn) |
| Cost to add | ~1200 LOC new + mechanical widening. No new crate. |

That is a third free-ish model family in the picker for roughly the diff Codex already paid for — and unlike Codex it does not need a new API backend. `minimax-m2` returned HTTP 200 with a valid usage block on the first live call.

**Scope flag:** `.planning/PROJECT.md` Out of Scope says *"Supporting arbitrary third-party providers beyond xAI + Codex/OpenAI in v1"*. NeevCloud is literally that. The architecture doesn't block it — but whether it lands in v1 or as a post-v1 phase is Cristian's call, not the implementer's. Ask before building.

## Read this first

1. **[01-protocol-reference.md](01-protocol-reference.md)** — verified ground truth. Nothing else in this set may contradict it.
2. **[02-harness-anatomy.md](02-harness-anatomy.md)** — what you're hooking into, and the gap analysis of what widens.
3. **[03-port-plan.md](03-port-plan.md)** — the phased, file-by-file plan, including what is deliberately skipped.
4. **[07-wire-fidelity.md](07-wire-fidelity.md)** — the adopted wire-identity decision. Ranks here because it is a *decision*, not a layer: it owns the fidelity spec and supersedes the "don't pin a UA" policy in 01 § 2 / 04 § 3 (their measured facts stand).
5. Then whichever of the layer docs your task touches.

If you only read two files: 01-protocol-reference.md and 03-port-plan.md.

## Contents

| File | What's in it |
|---|---|
| [01-protocol-reference.md](01-protocol-reference.md) | Verified NeevCloud wire protocol — device code, token poll, refresh, `/api/user`, `/api/orgs`, `/api/config`, the gateway. Constants, header matrix, error taxonomy, the invariants the spec imposes, and what stays unverified. Reverse-engineered from `@neevcode/neev@0.0.2` + confirmed with live authenticated calls. Also carries the gotchas: `access_token == refresh_token == apiKey`, required `x-org-id`, `/api/config` 404 = normal. |
| [02-harness-anatomy.md](02-harness-anatomy.md) | bum's current map: the crate-name trap, the auth file + already-multi-slot 0600 store, the two device-code flows that exist today, status/meta/CLI surface, routing (model id → base URL + credential slot), the two-layer model catalog, the HTTP + sampler layer, and a gap analysis of every 2→3 site. |
| [03-port-plan.md](03-port-plan.md) | The phased plan P0–P6 with a dependency graph: de-fang the two-way guards, protocol client, auth store slot, device login, refresh/ensure_fresh, routing + catalog, UX. Decision points recommendation-first, what's deliberately skipped (incl. why no `Provider` trait), and the GSD artifacts. |
| [04-rust-implementation.md](04-rust-implementation.md) | The concrete Rust: module layout for `auth/neev/{mod,http,device,refresh,console,ensure_fresh}.rs`, the poll loop, why `options.apiKey`/`options.baseURL` are never persisted, the 2→3 provider registration, the routing arm, catalog ingestion, tests, and build order. |
| [05-security-and-privacy.md](05-security-and-privacy.md) | `/api/config` is a hostile channel — parse it with a hard allowlist. One secret to rule them all, why neev's 0644 store must not be copied, privacy (NeevCloud sees every prompt), usage metering, host-validating the server-named base URL, redaction/401/revocation, and a checklist. |
| [06-testing-and-verification.md](06-testing-and-verification.md) | The mock-server stack, the curl smoke ladder, exact reproduction commands, unit + integration tests with zero live account, the two tests a mock cannot replace, testing against the real service safely, and a troubleshooting table. |
| [07-wire-fidelity.md](07-wire-fidelity.md) | **Owner of the fidelity spec.** The adopted decision + the 4-row table proving nothing is required today; both captured identities (console `neev/latest/0.0.2/cli` with the UA template decomposed; the AI-SDK gateway UA + `x-opencode-*`, whose NeevCloud applicability is a flagged strong inference from an `opencode.ai/zen/v1` capture); the single env-overridable `auth/neev/fidelity.rs`; the sampler-clobber analysis and its ~2-line fix; `ses_`/`msg_` id generation on the existing `rand`; why trace headers are skipped; re-derivation via mitmproxy and the capture-diff ritual. |

## Quickstart — smallest end-to-end proof

Ten lines. Run this before writing Rust; if it fails, the port would have failed the same way.

```bash
B=https://code.neevcloud.com; UA='neev/latest/0.0.2/cli'   # the real console UA; not required by the server — sent for fidelity, see 07-wire-fidelity.md
D=$(curl -s -A "$UA" -H 'content-type: application/json' -d '{"client_id":"neev-cli","client":"neev CLI 0.0.2 on linux"}' $B/auth/device/code)
echo "open: $B$(echo "$D" | jq -r .verification_uri_complete)   code: $(echo "$D" | jq -r .user_code)"   # NOTE: path, concatenated
DC=$(echo "$D" | jq -r .device_code)                       # ...approve in browser, then poll:
T=$(curl -s -A "$UA" -H 'content-type: application/json' -d "{\"grant_type\":\"urn:ietf:params:oauth:grant-type:device_code\",\"device_code\":\"$DC\",\"client_id\":\"neev-cli\"}" $B/auth/device/token)
K=$(echo "$T" | jq -r .access_token)                       # sk-… — this is ALSO the refresh_token AND the gateway apiKey
ORG=$(curl -s -A "$UA" -H "authorization: Bearer $K" $B/api/orgs | jq -r '.[0].id')                      # wrk_01…
CFG=$(curl -s -A "$UA" -H "authorization: Bearer $K" -H "x-org-id: $ORG" $B/api/config)                  # 404 here = no config, NOT an error
BASE=$(echo "$CFG" | jq -r .config.provider.opencode.options.baseURL)                                    # https://code.neevcloud.com/zen/go/v1
curl -s -A "$UA" -H "authorization: Bearer $(echo "$CFG" | jq -r .config.provider.opencode.options.apiKey)" \
  -H 'content-type: application/json' -d '{"model":"minimax-m2","max_tokens":20,"messages":[{"role":"user","content":"hi"}]}' $BASE/chat/completions
```

The last call returns HTTP 200 with `"model":"MiniMax-M2"` (note the casing flip — request `minimax-m2`, response `MiniMax-M2`; never round-trip-compare model ids).

Drop `-A "$UA"` and every call still returns 200 — curl's default UA, and even an empty UA, pass Cloudflare fine. Only a *known-bot* signature is banned: `Python-urllib/*` gets 403 `error_code: 1010, browser_signature_banned`, and the same urllib client with any other UA (even `curl/8.0`) gets 200.

## Effort + risk, honestly

**Effort: ~1200 LOC new + ~40 mechanical edits across 3 crates.** No new crate (Codex added zero — it's a module tree in `xai-grok-shell`). The new module is `src/auth/neev/{mod,device,refresh,ensure_fresh,console,fidelity}.rs` — `fidelity.rs` is ~100 LOC of constants + two header builders + an id generator ([07 § 3](07-wire-fidelity.md#3-rust-one-constants-module-authneevfidelityrs)), plus the ~2-line sampler edit. Skip `browser.rs` (no PKCE flow) and `claims.rs` (the token is an opaque `sk-…`, not a JWT). Everything else is filling match arms the compiler points at.

**The bulk of the risk is the widening, not the OAuth.**

| Risk | Severity | Why |
|---|---|---|
| **Poll loop eats 403/404 as "pending"** | High | Independent of any UA question. The Codex poll loop treats 403/404 as *pending* and `continue`s (`auth/codex/device.rs:249-251`) rather than hitting the generic "device auth failed with HTTP {status}" arm (`:252-254`). Copy that arm verbatim and **any** unexpected 403 — Cloudflare, a revoked client, a vendor rule change — looks like a user who never clicks approve, spinning silently until `MAX_WAIT` (15 min). Make 403 terminal. |
| **Cloudflare 403 on the auth host** | Low — defensive only | Measured live against `/api/orgs`: curl with no `-A` → 200, `-A 'neev/latest/0.0.2/cli'` → 200, empty UA → 200, `-A 'Python-urllib/3.13'` → **403** `1010 browser_signature_banned`. Cloudflare bans a small set of known-bot UA signatures, not unknown ones. A `reqwest` client never sends `Python-urllib`, so **Cloudflare is not a reason to pin a UA** — the shared `grok-shell/<ver> (os; arch)` UA would pass fine. bum pins `neev/latest/0.0.2/cli` anyway, for fidelity ([07](07-wire-fidelity.md#1-the-decision-and-the-honest-caveat)) — a different reason, not this one. Still worth ~5 lines: a named `BlockedClient` variant for 403 + `error_code: 1010`, because a bare 403 reads like an auth failure and Cloudflare rules are the vendor's to change. The `/zen/go/v1` gateway is not UA-gated at all. |
| **Sampler clobbers `extra_headers` UA** | Low — a deliberate decision, not a blocker | Load-bearing again now that fidelity is adopted. `extra_headers` are inserted, then `headers.insert(USER_AGENT, …)` runs unconditionally (`crates/codegen/xai-grok-sampler/src/client.rs:443-449` then `:500`), and `insert` replaces — so the gateway identity's UA cannot be delivered via `extra_headers`; it fails **silently**. The `x-opencode-*` headers are unaffected. Two routes: `origin_client` (supported seam, but composes a UA — not byte-exact) or a ~2-line conditional insert at `:500`. Only the latter is byte-exact; [07 § 4](07-wire-fidelity.md#4-the-sampler-clobber-problem) recommends it. Severity stays Low because nothing on the wire is validated today — worst case, the UA is merely *bum's*. |
| **Pinned fidelity strings go stale** | Low — but it's the insurance failing | `0.0.2`, `4.0.23`, `bun/1.3.13`, `latest` are frozen in the binary. If NeevCloud ever enforces a *minimum* client version, the hard-pinned stale UA is what breaks — the policy bought as insurance becomes the outage. Mitigation is the whole design of [07 § 3](07-wire-fidelity.md#3-rust-one-constants-module-authneevfidelityrs) / [§ 7](07-wire-fidelity.md#7-maintenance-the-insurance-can-become-the-outage): one greppable constants module, env-overridable so a mismatch is fixed without a rebuild, plus a documented mitmproxy re-derivation ritual. |
| **Silent xAI fallthrough** | High | `matches!(provider, Codex)` guards at `auth/flow.rs:901` and `:1077` (plus the `Some(Codex) => …` arm at `:781`) compile fine with a third variant and take the xAI path. A neev logout could clear the xAI slot. Convert to exhaustive `match` **before** adding the variant. |
| **String allowlists** | Medium | `parse_provider_wire_id`, `usable_for_wire`, `model_meta_provider` return `None`/`false` for an unknown id. No compile error, no runtime error — the TUI badge just never lights up. Grep for literal `"codex"` before declaring done. |
| **Runtime baseURL vs pure route** | Medium — a decision, not a bug | `resolve_provider_route` is documented as pure and as the sole base-URL authority. NeevCloud's base comes from an async authenticated `/api/config`. Cache it; don't make the route async; don't build a parallel base table. This belongs in the phase CONTEXT's locked decisions. |
| **Secret handling** | Medium | The token *is* the account. Never into `ModelEntry.api_key` (it's `Serialize`d into `models_cache.json`), never into an error body, never into a log. Redaction already covers `sk-…` (`xai-grok-secrets/src/sanitizer.rs:11`). |
| **Bump AUTH_DOCUMENT_VERSION** | Would be catastrophic | Version 2 makes every older bum binary hard-fail on the **whole** file and lose xAI + Codex logins. Adding a providers key is additive. Don't. |

**What's genuinely easy:** the store (already generic over `AuthProvider`), 0600 (free via `open_secure_file`), the 5-minute early-refresh window (`DEFAULT_EARLY_INVALIDATION_SECS = 300` already), the sampler, SSE, usage parsing (NeevCloud's usage block matches `Usage` field-for-field including `completion_tokens_details.reasoning_tokens`), 429/Retry-After, and the missing-provider gate (`missing_provider_gate_error` at `agent/config.rs:5602` is provider-agnostic, and the suggestion string is built as `format!("bum login --provider {id}")` at `:5557` — free). One caveat, not free: `ModelSwitchMissingProviderError::into_acp_error` matches the provider id as a **string** with `"codex" => …, _ => Xai` (`agent/config.rs:5563-5566`), so a neev error would render the xAI display label. Another string-allowlist site.

**Cheapest correct order:** (1) exhaustive-match the `matches!` guards, (2) add the `Neev` variant and let the compiler enumerate the rest, (3) copy `codex/` → `neev/` and swap wire details. Doing (3) first means debugging silent xAI fallthrough *and* a poll loop that swallows its own errors at the same time.
