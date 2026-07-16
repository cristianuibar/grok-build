# Stack Research

**Domain:** Multi-provider AI coding-agent CLI (Rust fork of Grok Build: xAI + ChatGPT/Codex OAuth)
**Project:** bum
**Researched:** 2026-07-16
**Confidence:** HIGH (OAuth/endpoints/model IDs verified from openai/codex source + OpenAI docs; brownfield stack from this repo’s manifests)

## Executive Recommendation

Stay on the **existing Grok Build workspace stack**. Add Codex/ChatGPT as a **second auth scope + second sampler base URL**, not a second agent runtime.

| Layer | Decision | Why |
|-------|----------|-----|
| Runtime | Keep Rust 2024 / Tokio 1 / existing TUI | Brownfield constraint; agent/TUI already production-grade |
| xAI auth | Keep existing OAuth2/OIDC + device-code under `xai-grok-shell` | Already works against `auth.x.ai` |
| Codex auth | **Reimplement Codex PKCE/device-code in-tree**, mirror `openai/codex` `codex-rs/login` | Official client is open-source Rust; do not depend on third-party `codex-oauth` |
| Credential store | Multi-scope `~/.bum/auth.json` (extend existing scoped store) | Store already multi-scope; isolate from `~/.grok` and `~/.codex` |
| Inference | Keep `xai-grok-sampler` + `async-openai` 0.33 Responses path | Branch only `SamplerConfig.{base_url,api_key,extra_headers,bearer_resolver}` by provider |
| GPT models | Ship GPT-5.6 Sol/Terra/Luna IDs in `default_models.json` | Current Codex/API family as of 2026-07 |
| ChatGPT traffic | `https://chatgpt.com/backend-api/codex` + Bearer + `ChatGPT-Account-ID` | Codex’s ChatGPT-login path (not Platform API key billing) |
| Platform API fallback | Optional later: `https://api.openai.com/v1` + API key | Separate product/billing; not v1 primary |

---

## Recommended Stack

### Core Technologies (brownfield — keep)

| Technology | Version (this repo) | Purpose | Why Recommended |
|------------|---------------------|---------|-----------------|
| Rust | **1.92.0** (`rust-toolchain.toml`), edition **2024** | Entire product | Pinned workspace; do not bump for v1 |
| Tokio | **1** (`features = ["full"]`) | Async runtime | Already drives agent, HTTP, TUI effects |
| ratatui + crossterm | **0.29** / **0.28** | TUI | Out of v1 research scope; keep |
| clap | **4** | CLI (`bum login`, etc.) | Existing subcommand surface |
| reqwest | **0.12.x** (workspace; rustls) | HTTP/SSE | Primary client for auth + sampling |
| oauth2 | **5.0.0** | Token exchange helpers | Already in tree for xAI/MCP; use for Codex PKCE token POSTs where convenient |
| async-openai | **0.33.1** | OpenAI-compatible Responses/Chat types | Sampler already streams Responses via this surface |
| axum | **0.8** | Loopback OAuth callbacks | Grok OIDC already uses loopback; Codex needs fixed port **1455** |
| serde / serde_json / toml | workspace | Config + auth.json | Existing patterns |
| tracing | workspace | Logs | Keep; no new logging stack |

### Multi-Provider Auth (new work)

| Piece | Version / ID | Purpose | Why |
|-------|--------------|---------|-----|
| Codex issuer | `https://auth.openai.com` | Authorize + token + device auth | Hard-coded default in Codex `codex-rs/login` |
| Codex public client_id | `app_EMoamEEZ73f0CkXaXp7hrann` | PKCE public client | Official Codex CLI registration; required for allow-listed redirect URIs |
| Browser OAuth callback | `http://localhost:1455/auth/callback` (fallback **1457**) | PKCE redirect | Whitelisted on OpenAI auth server; not arbitrary ports |
| Device-code API | `{issuer}/api/accounts/deviceauth/{usercode,token}` | Headless login | Codex beta path; user verifies at `{issuer}/codex/device` |
| Refresh / revoke | `https://auth.openai.com/oauth/token` / `.../oauth/revoke` | Session longevity | `grant_type=refresh_token` + public `client_id` (no secret) |
| PKCE | S256 (sha2 + base64url) | Auth code flow | Same as Codex `pkce.rs`; implement with existing `sha2`/`base64` or tiny local helper |
| Loopback server | Prefer **axum** (already in tree) over Codex’s `tiny_http` | Callback UX | Align with Grok OIDC path; avoid new dep |

**Do not add** a second OAuth crate family (`openidconnect` only if discovery is needed; Codex endpoints are fixed paths, so discovery is optional).

### Inference / Sampling (branch, don’t rewrite)

| Piece | Target | Purpose | Why |
|-------|--------|---------|-----|
| Wire protocol | **Responses API** (`api_backend = responses`) | Tool-calling agent turns | Grok Build default is Responses; Codex dropped Chat Completions wire for built-in providers |
| xAI path | Keep `https://cli-chat-proxy.grok.com/v1` (+ existing headers) | Grok models | Existing `SamplerConfig` + xAI bearer |
| Codex ChatGPT path | Base `https://chatgpt.com/backend-api/codex` → `POST …/responses` | Subscription-backed GPT models | Codex chooses this base when `AuthMode` is ChatGPT-family |
| Platform API path (secondary) | `https://api.openai.com/v1` → `…/responses` | API-key billing | Only if user chooses API key; different retention/billing |
| Live bearer | `SamplerConfig::bearer_resolver` | Per-request token | Avoid stale tokens across long sessions |
| Extra headers | `extra_headers` / `header_injector` | `ChatGPT-Account-ID`, FedRAMP | Sampler already supports opaque headers without URL inspection |

### GPT-5.6 / Codex model IDs (ship in selector)

Verified against OpenAI model catalog + Codex docs (2026-07):

| Model ID | Role | Notes |
|----------|------|-------|
| `gpt-5.6-sol` | Flagship coding / complex work | Primary default for Codex-side |
| `gpt-5.6` | Alias → Sol | API alias routes to Sol; Codex CLI accepts `gpt-5.6` |
| `gpt-5.6-terra` | Balanced cost/quality | Everyday coding |
| `gpt-5.6-luna` | Cheapest / high volume | Fast loops, simple tasks |
| `gpt-5.5` | Previous frontier | Optional legacy entry |
| `gpt-5.3-codex-spark` | Near-instant coding (Pro) | Optional; availability plan-gated |
| `gpt-5.4` / `gpt-5.4-mini` | Older family | Optional; not v1 focus |

**Deprecated for ChatGPT sign-in (do not ship as primary):** `gpt-5.2`, `gpt-5.3-codex` (Codex docs: deprecated under ChatGPT login; may still exist on Platform API).

**v1 recommendation:** default Codex model `gpt-5.6-sol` (display “GPT-5.6 Sol”), plus Terra + Luna in picker. Keep existing `grok-build` as xAI default.

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sha2` + `base64` | already workspace | PKCE S256 | Codex authorize/token |
| `chrono` | workspace | `expires_at` / `last_refresh` | Proactive refresh |
| `urlencoding` | add if missing | Form-encoded token body | Token + refresh POST |
| `webbrowser` | optional | Open authorize URL | Mirror Codex UX; can keep Grok’s existing open-browser helper |
| `keyring` / OS secret store | **defer** | Alternate credential store | Codex supports it; v1 file store under `~/.bum` is enough |
| `codex-oauth` (crates.io 0.1.1) | **do not use** | Third-party PKCE wrapper | Low adoption, incomplete vs upstream, hardcodes client id without refresh/store parity |
| New LLM SDK / `openai` official Rust | **do not use** | Alternate client | Duplicates `async-openai` + custom sampler stream |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo check -p xai-grok-shell` / `-p xai-grok-sampler` | Focused builds | Full workspace is heavy |
| `wiremock` / `axum` test servers | Fake auth.openai.com + responses | Mirror existing sampler tests |
| Installed `codex` CLI (manual) | Golden-path comparison | Compare authorize URL, auth.json shape, headers (never commit tokens) |

---

## Codex OAuth: Obtain / Refresh / Store (integration contract)

Source of truth: `openai/codex` `codex-rs/login` (main, fetched 2026-07-16). Official product docs: [Authentication](https://learn.chatgpt.com/docs/auth).

### 1. Browser PKCE login (default)

1. Bind loopback **`127.0.0.1:1455`** (if busy, try **1457**).
2. Generate PKCE: random verifier → S256 challenge (base64url, no pad).
3. Generate opaque `state`.
4. Open authorize URL:

```
https://auth.openai.com/oauth/authorize
  ?response_type=code
  &client_id=app_EMoamEEZ73f0CkXaXp7hrann
  &redirect_uri=http://localhost:1455/auth/callback
  &scope=openid%20profile%20email%20offline_access%20api.connectors.read%20api.connectors.invoke
  &code_challenge=<S256>
  &code_challenge_method=S256
  &id_token_add_organizations=true
  &codex_cli_simplified_flow=true
  &state=<state>
  &originator=<product>   # Codex sets originator; bum should use a bum-specific tag
```

5. On callback: validate `state`, read `code`.
6. Token exchange `POST https://auth.openai.com/oauth/token`  
   `application/x-www-form-urlencoded`:  
   `grant_type=authorization_code&code&redirect_uri&client_id&code_verifier`
7. Response: `id_token`, `access_token`, `refresh_token`.
8. Parse JWT claims from `id_token` for email, plan, **`chatgpt_account_id`**.

### 2. Device-code login (headless / SSH)

1. `POST https://auth.openai.com/api/accounts/deviceauth/usercode` JSON `{ "client_id": "…" }`  
   → `user_code`, `device_auth_id`, `interval`.
2. Show verification URL `https://auth.openai.com/codex/device` + user code.
3. Poll `POST …/deviceauth/token` until success (≤15 min) → authorization_code + PKCE pair from server.
4. Exchange code with `redirect_uri=https://auth.openai.com/deviceauth/callback`.
5. Persist same token structure as browser login.

Requires device-code enabled in ChatGPT account/workspace security settings (Codex docs).

### 3. Refresh

- Endpoint: `POST https://auth.openai.com/oauth/token`
- Body: `grant_type=refresh_token&refresh_token=…&client_id=app_EMoamEEZ73f0CkXaXp7hrann`
- Proactive refresh policy (Codex):
  - If access JWT `exp` ≤ now + **5 minutes** → refresh.
  - Else if `last_refresh` older than **~8 days** → refresh.
- On **401** from inference: reload credentials from disk (same account id), then refresh once, retry.
- Permanent refresh failures: `refresh_token_expired` | `refresh_token_reused` | `refresh_token_invalidated` → force re-login.
- Always **persist** new tokens + `last_refresh` after successful refresh (community agents frequently break here).

### 4. Storage for bum (isolated)

**Path:** `$BUM_HOME/auth.json` (default `~/.bum/auth.json`), mode `0600`, crash-safe write like existing Grok store.

**Do not** read/write `~/.codex/auth.json` or `~/.grok/auth.json` in v1.

**Recommended multi-provider layout** (extend existing scoped map rather than Codex’s single-file shape):

```json
{
  "https://auth.x.ai::<client_id>": { /* existing GrokAuth */ },
  "xai::api_key": { /* optional */ },
  "openai::chatgpt": {
    "key": "<access_token>",
    "auth_mode": "oidc",
    "create_time": "…",
    "user_id": "<chatgpt_user_id>",
    "email": "…",
    "refresh_token": "…",
    "expires_at": "…",
    "oidc_issuer": "https://auth.openai.com",
    "oidc_client_id": "app_EMoamEEZ73f0CkXaXp7hrann",
    "organization_id": "<chatgpt_account_id>"
  }
}
```

Map Codex fields into `GrokAuth` (or a thin `ProviderAuth` enum wrapper) so `AuthManager` refresh/recovery remains shared:

| Codex field | bum field |
|-------------|-----------|
| `tokens.access_token` | `key` |
| `tokens.refresh_token` | `refresh_token` |
| `tokens.account_id` / id_token `chatgpt_account_id` | `organization_id` or dedicated `account_id` |
| access JWT `exp` | `expires_at` |
| `last_refresh` | store alongside or derive |
| `auth_mode: chatgpt` | new mode `Chatgpt` **or** reuse `Oidc` with issuer gate |

**API key optional scope:** `openai::api_key` → Platform path only.

### 5. Request auth headers (ChatGPT OAuth)

When sampling via Codex backend:

```http
Authorization: Bearer <access_token>
ChatGPT-Account-ID: <chatgpt_account_id>
X-OpenAI-Fedramp: true   # only if account claim says FedRAMP
```

Without `ChatGPT-Account-ID`, workspace routing fails on multi-workspace accounts.

---

## How sampling should branch by provider (no agent rewrite)

Existing seam is already correct: **session builds `SamplerConfig` per model**.

```
model picker / default_models.json
        │
        ▼
resolve_model → ModelSpec { provider, model_id, api_backend, base_url? }
        │
        ▼
auth gate: provider has valid credentials?
   no  → block + prompt `bum login --provider {xai|codex}`
   yes → build SamplerConfig
        │
        ├─ provider=xai
        │     base_url = cli-chat-proxy (existing)
        │     bearer = xAI AuthManager scope
        │     extra_headers = existing proxy headers
        │
        └─ provider=openai_chatgpt
              base_url = https://chatgpt.com/backend-api/codex
              api_backend = Responses
              bearer_resolver = Codex access token (live)
              extra_headers = ChatGPT-Account-ID (+ FedRAMP)
        │
        ▼
SamplerActor / SamplingClient  (unchanged stream/retry core)
```

**Implementation points (minimal):**

1. **`default_models.json`**: add `provider: "xai" | "openai_chatgpt"` (or infer from model id prefix).
2. **`resolve_model_to_sampling_config`**: switch base URL + credential provider by model.
3. **`AuthManager`**: multi-principal — “current token for provider P”, proactive refresh per scope.
4. **Model switch mid-session**: rebuild `SamplerConfig` only (already tested in sampler actor tests for model A→B).
5. **Do not** fork TUI event loop, tools registry, or chat-state actors for v1.

**WebSocket transport note (MEDIUM):** Community reports and Codex core indicate ChatGPT Responses may use WebSocket in addition to HTTP SSE. Start with **HTTP Responses streaming** (matches current sampler). If empty streams appear against `chatgpt.com/backend-api/codex`, add a Codex-compatible WS path as a phase-2 transport behind a feature flag — not a full agent rewrite.

---

## Installation

No new top-level package manager. Cargo only.

```bash
# No new mandatory crates for v1 if sha2/base64/urlencoding already present.
# If missing:
# cargo add urlencoding --package xai-grok-shell

# Dev verification (manual)
# cargo check -p xai-grok-shell -p xai-grok-sampler -p xai-grok-models -p xai-grok-auth
```

Binary rename (product stack, not a dep): composition root `xai-grok-pager-bin` → ship as **`bum`**; default home env **`BUM_HOME`** (fallback migrate logic optional later).

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| In-tree Codex PKCE (mirror openai/codex) | crates.io `codex-oauth` 0.1.x | Never for production; toy wrapper only |
| In-tree + existing AuthManager | Depend on / vendor full `codex-login` crate | Only if extracting a clean published crate later; today it’s monorepo-coupled |
| ChatGPT backend `…/backend-api/codex` | Always `api.openai.com/v1` with ChatGPT OAuth token | **Wrong** — OAuth subscription tokens are not Platform API keys |
| Platform API key | ChatGPT OAuth | CI/automation without browser; different billing |
| Multi-scope `~/.bum/auth.json` | Import `~/.codex/auth.json` | Explicit later feature; violates v1 isolation |
| HTTP Responses first | Implement WS Responses day one | Only after HTTP path proven insufficient |
| Reuse Codex public `client_id` | Register own OAuth client | Impossible for third parties without OpenAI allowing new redirect URIs; community reuses public id |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `codex-oauth` crate | Immature (~64 downloads), incomplete vs upstream | In-tree flow from openai/codex |
| Sending ChatGPT OAuth bearer to `api.openai.com` as if it were a Platform key | Scope/permission failures; wrong product surface | `https://chatgpt.com/backend-api/codex` |
| Chat Completions as Codex primary wire | Codex removed `wire_api = "chat"` for built-ins | Responses API only |
| Sharing `~/.codex` or `~/.grok` credentials | Coupling, accidental logout, security boundary blur | `~/.bum` only |
| New agent framework / rewrite sampler | Out of scope; high rewrite risk | Branch `SamplerConfig` |
| Embedding Sentry/Mixpanel/x.ai auto-update as stock | v1 quiet fork requirement | Disable update channel + product telemetry |
| Hardcoding only `gpt-5.3-codex` | Deprecated under ChatGPT login | GPT-5.6 Sol/Terra/Luna |
| Storing refresh tokens without atomic write + lock | Refresh rotation races → `refresh_token_reused` permanent logout | Existing auth.json lock + atomic write patterns |

---

## Stack Patterns by Variant

**If primary goal is ChatGPT subscription coding (v1 default):**
- OAuth PKCE + device-code
- Base URL `https://chatgpt.com/backend-api/codex`
- Models `gpt-5.6-sol` / `terra` / `luna`
- Header `ChatGPT-Account-ID`

**If user provides Platform API key:**
- Store under `openai::api_key`
- Base URL `https://api.openai.com/v1`
- No ChatGPT account header
- Bill Platform rates; some Codex-only features unavailable

**If headless SSH without port forward:**
- Prefer device-code (`bum login --device-auth`)
- Fallback: complete browser login on a GUI machine and copy `~/.bum/auth.json` (treat as secret)

**If FedRAMP workspace:**
- Set `X-OpenAI-Fedramp: true` from id_token claim

---

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `async-openai` 0.33.1 | Responses streaming in `xai-grok-sampler` | Keep; validate ChatGPT backend SSE event names against proxy if stream differs |
| `oauth2` 5.0.0 | Custom Codex token form POST | Can use raw reqwest for exact Codex form encoding |
| `reqwest` 0.12 | rustls TLS | Keep workspace default (0.13 only for MCP isolation) |
| Codex redirect port 1455 | Single process | Conflict if stock `codex login` runs simultaneously — document; fallback 1457 |
| GPT-5.6 family | ChatGPT paid + API | Rollout GA 2026-07-09; availability may still be plan/region gated |

---

## Confidence by Claim

| Claim | Confidence | Basis |
|-------|------------|-------|
| Codex issuer, client_id, ports, scopes, PKCE params | **HIGH** | openai/codex `codex-rs/login` source |
| Refresh URL + proactive 5m / 8d policy | **HIGH** | `manager.rs` constants + docs |
| ChatGPT inference base `…/backend-api/codex` | **HIGH** | `CHATGPT_CODEX_BASE_URL` in `model-provider-info` |
| Headers Bearer + `ChatGPT-Account-ID` | **HIGH** | `BearerAuthProvider` in codex `model-provider` |
| GPT-5.6 Sol/Terra/Luna model IDs | **HIGH** | OpenAI models docs + Codex models page |
| ChatGPT OAuth ≠ Platform API key | **HIGH** | Docs + third-party integration writeups |
| HTTP SSE sufficient for v1 | **MEDIUM** | Sampler is SSE; WS may be required later for ChatGPT path |
| Exact ChatGPT Responses request body parity with cli-chat-proxy | **MEDIUM** | Need integration tests against live backend |
| Legal/ToS of reusing public Codex client_id in a private fork | **MEDIUM** | Common practice; not a formal OpenAI partner API |

---

## Sources

- openai/codex `codex-rs/login` (server.rs, device_code_auth.rs, token_data.rs, auth/storage.rs, auth/manager.rs) — OAuth flow, storage, refresh — **HIGH**
- openai/codex `codex-rs/model-provider-info` — `CHATGPT_CODEX_BASE_URL`, Responses wire — **HIGH**
- openai/codex `codex-rs/model-provider` `bearer_auth_provider.rs` — auth headers — **HIGH**
- [Codex Authentication docs](https://learn.chatgpt.com/docs/auth) — login UX, credential store, device-code, CI notes — **HIGH**
- [Codex Models docs](https://learn.chatgpt.com/docs/models) — gpt-5.6-sol/terra/luna commands, deprecations — **HIGH**
- [OpenAI Models catalog](https://developers.openai.com/api/docs/models) / [GPT-5.6 Sol](https://developers.openai.com/api/docs/models/gpt-5.6-sol) — IDs, alias `gpt-5.6` — **HIGH**
- [OpenAI release notes — GPT-5.6 family](https://openai.com/products/release-notes/) — GA 2026-07-09 — **HIGH**
- This repo `.planning/codebase/{STACK,INTEGRATIONS,ARCHITECTURE}.md` — brownfield baselines — **HIGH**
- crates.io `codex-oauth` — existence only; **not recommended** — **HIGH** (reject)

---

*Stack research for: bum multi-provider OAuth + routing*  
*Researched: 2026-07-16*  
*Mode: ecosystem (brownfield delta)*  
