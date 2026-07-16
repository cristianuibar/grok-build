# Project Research Summary

**Project:** bum
**Domain:** Multi-provider AI coding-agent CLI (Rust fork of Grok Build: xAI + ChatGPT/Codex OAuth)
**Researched:** 2026-07-16
**Confidence:** HIGH

## Executive Summary

**bum** is a full-product brownfield fork of Grok Build: ship as `bum` with isolated `~/.bum`, keep the existing agent/TUI harness, and add a thin **provider plane** so one CLI can OAuth to both xAI (Grok) and ChatGPT/Codex (GPT-5.6), then route each sample by **active model**, not a global session mode. Experts building multi-provider agent CLIs (Claude Code, Codex CLI, OpenCode) treat provider as the unit of credentials + default base URL + catalog source; they fail closed on missing auth and isolate product home from stock vendor CLIs. This research strongly endorses that pattern inside the existing monorepo rather than a rewrite or OpenRouter-first multi-cloud surface.

**Recommended approach:** Stay on Rust 2024 / Tokio / existing sampler (`async-openai` Responses path, `SamplerConfig` + `BearerResolver`). Reimplement Codex PKCE/device-code **in-tree** (mirror `openai/codex` `codex-rs/login`; do not use crates.io `codex-oauth`). Store multi-scope credentials only under `~/.bum/auth.json`. Route Grok → cli-chat-proxy + xAI bearer; route GPT → `https://chatgpt.com/backend-api/codex` + ChatGPT Bearer + `ChatGPT-Account-ID`. Ship static GPT-5.6 Sol/Terra/Luna in the picker with explicit `provider` on every model entry. Order work **identity → multi-slot auth → catalog → routing → Codex OAuth → gate/UX → quiet rebrand polish**.

**Key risks:** (1) single-slot auth overwrite / shared wipe wiping both providers; (2) wrong token on wrong backend (or ChatGPT OAuth treated as Platform API key); (3) half-rebrand still writing `~/.grok` and phone-home via Mixpanel/Sentry/auto-update; (4) global “provider mode” fighting mixed mid-session switch. Mitigate with provider-scoped credential hub, per-request routing from model metadata, fail-closed missing-provider gate, full home/identity inventory early, and compile-time kill of stock egress channels.

## Key Findings

### Recommended Stack

Details: [STACK.md](./STACK.md)

Stay on the **existing Grok Build workspace**. Codex is a second auth scope + second sampler base URL, not a second agent runtime. Browser OAuth uses fixed loopback port **1455** (fallback **1457**), public Codex `client_id`, and issuer `https://auth.openai.com`. Inference stays Responses API; ChatGPT subscription traffic uses the Codex backend base, not `api.openai.com` with OAuth tokens. Platform API key is optional secondary only.

**Core technologies:**
- **Rust 1.92 / edition 2024 + Tokio 1** — entire product runtime — brownfield constraint; do not bump for v1
- **In-tree Codex PKCE/device-code** (reqwest, oauth2 5, axum loopback, sha2/base64) — ChatGPT login parity with Codex CLI — official client is open-source Rust; avoid immature third-party crates
- **Multi-scope `~/.bum/auth.json`** — dual OAuth isolation — extend existing scoped store; never share `~/.grok` / `~/.codex`
- **`xai-grok-sampler` + `async-openai` 0.33 Responses** — streaming agent turns — branch only `SamplerConfig` fields by provider
- **GPT-5.6 Sol / Terra / Luna** (`gpt-5.6-sol` default Codex-side) — coding model family for ChatGPT sign-in — verified against Codex/OpenAI docs 2026-07
- **ratatui/crossterm TUI + clap CLI** — daily-driver surface — out of rewrite scope; keep

### Expected Features

Details: [FEATURES.md](./FEATURES.md)

v1 validates Core Value: *one CLI, log into both, switch Grok ↔ GPT-5.6 in a real coding session*. Baseline TUI, tools, sessions, and xAI OAuth already exist; v1 is identity + dual OAuth + routing + quiet fork.

**Must have (table stakes):**
- Product CLI as `bum` + isolated home `~/.bum` — clear product identity
- xAI OAuth preserved + first-class ChatGPT/Codex OAuth (browser + device-code)
- Per-provider login / logout / status
- Mixed model picker with Grok + GPT-5.6 family; switch anytime
- Provider-aware routing + missing-provider gate (block + prompt correct login)
- Independent token refresh per provider; clear provider-scoped auth errors
- Quiet local fork (no xAI auto-update / product telemetry)
- Daily-driver bar: tools, permissions, sessions work on both backends

**Should have (competitive):**
- True dual-subscription OAuth in one harness (primary differentiator vs two CLIs or OpenRouter-first tools)
- Per-model provider binding (not session-global mode)
- Fail-closed login gate at selection time with provider-specific UX
- Isolated identity from stock `grok` / `codex` homes

**Defer (v2+ / v1.x polish):**
- Arbitrary third-party providers, custom agentic workflows, multi-account aliases
- Import/share stock credential stores, cost dashboards, cloud remote agents
- Reasoning-effort knobs / richer status UI / capability matrix docs (v1.x once dual path works)
- API-key fallback secondary (P2 — useful for CI; not primary identity story)

### Architecture Approach

Details: [ARCHITECTURE.md](./ARCHITECTURE.md)

v1 inserts a thin **provider plane** between session/sampler and external IdPs/APIs. Agent tools, workspace, ACP, and Action→Effect TUI stay as today. Sampler remains URL-agnostic; shell builds correct `SamplerConfig` per model. One `auth.json` multi-key map with multi-slot credential hub; models carry explicit `ProviderId`.

**Major components:**
1. **ProviderRegistry** — known providers (`xai`, `openai_codex`), issuers, default base URLs, login capability
2. **CredentialHub** — multi-slot load/store/refresh/401 recovery per provider over shared `auth.json` lock
3. **ModelCatalog** — merge remote/static Grok catalog + static GPT-5.6 entries with `provider` field
4. **RequestRouter** — model → base URL, api backend, credential slot, headers (no URL sniffing for identity)
5. **Provider-aware BearerResolver** — per-request bearer from correct provider manager
6. **Missing-provider gate** — fail closed on set_model / pre-turn with typed `ProviderAuthRequired`
7. **Product home / quiet branding** — `BUM_HOME` → `~/.bum`, binary `bum`, no-op update/telemetry

### Critical Pitfalls

Details: [PITFALLS.md](./PITFALLS.md)

1. **Single-slot auth overwrite** — Design provider-scoped records + independent refresh/wipe from day one; dual login/logout/refresh tests before shipping GPT UX.
2. **Wrong credential / wrong backend** — Bind every catalog entry to `ProviderId`; resolve credentials per request; never attach dual auth headers; never send ChatGPT OAuth to Platform API as if it were an API key.
3. **Half-rebrand / incomplete home isolation** — Inventory all path writers and identity strings early; temp-home e2e must show zero writes under `~/.grok` / `~/.codex`.
4. **Phone-home channels still open** — Kill Mixpanel, Sentry, auto-update/`minimum_version`, and stock marketplace phone-home with compile-time defaults, not a single TOML flag.
5. **Auth error misclassification (403 → wipe)** — Keep 401-only refresh triggers; scope invalid_grant counters to one provider; never wipe both on policy 403.
6. **Global provider mode** — Session stores current model id only; history shared; next sample chooses backend.

## Implications for Roadmap

Based on research, suggested phase structure (architecture A–G + pitfalls ordering):

### Phase 1: Product identity & isolated home
**Rationale:** All credential and session work must land under `~/.bum` without clobbering stock Grok; half-rebrand is a top critical pitfall.
**Delivers:** Default home `~/.bum` (`BUM_HOME`), binary path toward `bum`, path/env smoke under new home.
**Addresses:** Product rename (partial), isolated home cutover (P1).
**Avoids:** Half-rebrand path leakage; incomplete home isolation (auth yes / everything else no).

### Phase 2: Provider types & multi-slot credential store
**Rationale:** Dual OAuth without provider-scoped store causes last-login-wins and cross-wipe; foundation before second login UX.
**Delivers:** `ProviderId` + auth scope keys; CredentialHub / multi-scope load; xAI continues green as `ProviderId::Xai`.
**Addresses:** Preserve xAI under bum store; dual credential architecture for later Codex.
**Avoids:** Single-slot overwrite; shared wipe thresholds; bolting Codex into `GrokAuth.key`.

### Phase 3: Model catalog provider tagging + static GPT-5.6
**Rationale:** Routing and gates need explicit model→provider metadata; list both ecosystems without global mode.
**Delivers:** `provider` (+ OpenAI base_url) on `ModelInfo` / `default_models.json`; GPT-5.6 Sol/Terra/Luna (and alias) in merged picker list.
**Addresses:** Mixed model picker entries; GPT-5.6 family IDs (P1 catalog half).
**Avoids:** Floating models without provider; global mode filtering the whole session.

### Phase 4: Provider-aware request routing & bearer
**Rationale:** Wrong host/token is silent failure/billing risk; must be correct before live GPT turns.
**Delivers:** `resolve_credentials` / `sampling_config_for_model` branch on provider; hub-scoped `BearerResolver`; golden tests for Grok vs GPT headers/base_url; no xAI proxy headers on OpenAI URLs.
**Addresses:** Provider-aware routing (P1).
**Avoids:** Wrong credential/backend; dual headers; URL-only sniffing for identity.

### Phase 5: Codex / ChatGPT OAuth login
**Rationale:** Real GPT-5.6 requires ChatGPT subscription OAuth path (not Platform key primary); can start once scope keys exist (overlaps late Phase 3–4).
**Delivers:** Browser PKCE (ports 1455/1457) + device-code; refresh (5m JWT / ~8d last_refresh policy); logout/status; tokens only under `~/.bum`; CLI `bum login openai` / `logout` / status.
**Addresses:** Codex/ChatGPT OAuth; per-provider login/logout/status; token refresh dual-store (P1).
**Avoids:** ChatGPT OAuth ≠ Platform API confusion; stock `~/.codex` coupling; non-persisted refresh → `refresh_token_reused`.

### Phase 6: Missing-provider gate + multi-provider UX
**Rationale:** Daily-driver safety — no mid-stream 401; mixed switch without restart; headless must print clear login command.
**Delivers:** Gate on set_model / pre-turn; TUI dual auth badges + login prompts; headless/ACP typed errors; mid-session model switch without transport freeze.
**Addresses:** Missing-provider gate; multi-provider session UX; clear auth errors (P1).
**Avoids:** Silent fail mid-turn; single “Login” that only does xAI; global mode; mid-switch capability drift (basic).

### Phase 7: Quiet fork, rebrand polish & daily-driver validation
**Rationale:** Privacy and product identity complete ship criteria; end-to-end proves Core Value.
**Delivers:** Disable auto-update + product telemetry/Mixpanel/Sentry phone-home; remaining chrome/docs as `bum`; e2e dual login, switch mid-session, tools both paths.
**Addresses:** Quiet local fork; product rename completeness; daily-driver bar (P1).
**Avoids:** Residual phone-home; auto-update overwriting fork; incomplete identity strings.

### Phase Ordering Rationale

- **Identity first** — safe sandbox for auth tests without nuking real `~/.grok`.
- **Multi-slot auth before UX** — dual store/refresh is the hard failure mode; picker without it is a trap.
- **Catalog → routing before (or with) live OAuth** — unit-test correct `SamplerConfig` with fake tokens; then real Codex login unlocks GPT turns.
- **Gate after routing + OAuth** — fail-closed needs both “which provider” and invokable login.
- **Quiet fork early enough to parallel polish** — can start mid-Phase 6; must land before trusting daily use (overwrite risk).
- **No agent rewrite** — stack research forbids forking TUI/tools/chat-state for v1.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 5 (Codex OAuth):** Live authorize/token/device-code parity, `chatgpt_account_id` claims, refresh failure taxonomy — confirm against current `openai/codex` + live IdP; legal/ToS of public `client_id` reuse remains MEDIUM.
- **Phase 4–5 (ChatGPT inference path):** Exact Responses request body / SSE event parity vs cli-chat-proxy; **HTTP SSE vs WebSocket** for `chatgpt.com/backend-api/codex` (start HTTP; flag WS if empty streams).
- **Phase 6 (capability gaps):** Tool/schema quirks and subagent model inheritance on OpenAI path — document first real gaps when found.

Phases with standard patterns (skip research-phase):
- **Phase 1 (home/binary):** Path rename and env override are well-mapped in-tree.
- **Phase 2 (multi-scope store):** Extend existing `AuthStore` BTreeMap + flock patterns.
- **Phase 3 (static catalog):** `default_models.json` + `ModelsManager` merge already exist.
- **Phase 7 (telemetry kill switches):** INTEGRATIONS/CONCERNS already enumerate egress surfaces.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Codex OAuth endpoints/client_id/ports/headers and GPT-5.6 IDs verified from openai/codex + OpenAI docs; brownfield versions from this repo |
| Features | HIGH | Aligns tightly with PROJECT.md + competitor UX patterns (Claude/Codex/OpenCode); daily-driver bar is product judgment (MEDIUM) |
| Architecture | HIGH | Provider plane maps to existing seams (`ModelsManager`, `sampling_config_for_model`, `BearerResolver`, multi-key auth store) |
| Pitfalls | MEDIUM–HIGH | HIGH for in-tree auth wipe, home, update/telemetry; MEDIUM for community multi-provider failure modes |

**Overall confidence:** HIGH

### Gaps to Address

- **ChatGPT Responses transport:** Whether HTTP SSE is sufficient for v1 or WS is required — validate with integration tests against live Codex backend; feature-flag WS if needed.
- **Request body parity:** Exact ChatGPT backend Responses payload vs existing sampler — phase planning should add golden/live fixtures.
- **GPT-5.6 ID / plan gating:** Availability may be plan/region gated; reconfirm IDs at implement time; gate UX on auth + entitlement errors.
- **OAuth client_id ToS:** Reusing Codex public client is common practice but not a formal partner API — document risk; isolate storage; plan rotation if OpenAI tightens.
- **OS keyring:** Defer to post-v1; file store `0600` is enough for launch.
- **API-key secondary path:** P2 — implement only after ChatGPT OAuth primary is solid to avoid billing confusion.
- **Subagent model inheritance:** Ensure parent model/provider propagates; verify in Phase 6/7 tests.

## Sources

### Primary (HIGH confidence)
- openai/codex `codex-rs/login`, `model-provider-info`, `model-provider` — OAuth flow, `CHATGPT_CODEX_BASE_URL`, bearer headers
- [Codex Authentication](https://learn.chatgpt.com/docs/auth) — login UX, device-code, credential store
- [Codex Models](https://learn.chatgpt.com/docs/models) — gpt-5.6-sol/terra/luna, deprecations
- [OpenAI Models catalog](https://developers.openai.com/api/docs/models) / GPT-5.6 Sol — IDs and aliases
- This repo `.planning/PROJECT.md`, `.planning/codebase/{ARCHITECTURE,STACK,INTEGRATIONS,CONCERNS}.md` — brownfield seams and constraints
- In-tree: `xai-grok-shell` auth/models/config, `xai-grok-sampler` (`SamplerConfig`, `BearerResolver`), `xai-grok-models/default_models.json`

### Secondary (MEDIUM confidence)
- OpenCode / Claude Code / Continue / Aider multi-provider UX patterns — table stakes for login, picker, homes
- Community multi-agent trackers (dual-auth headers, reauth loops, subagent model override) — failure modes
- HTTP SSE sufficiency for ChatGPT Codex backend — needs live validation

### Tertiary (LOW confidence)
- Legal durability of third-party reuse of Codex public OAuth client_id — common practice, not formalized
- Whether FedRAMP header path is needed for Buff Up Media accounts — only if claim present

---
*Research completed: 2026-07-16*
*Ready for roadmap: yes*
