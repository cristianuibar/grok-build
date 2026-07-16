# Pitfalls Research

**Domain:** Multi-provider OAuth coding-agent CLI fork (bum = Grok Build + Codex OAuth)
**Researched:** 2026-07-16
**Confidence:** MEDIUM–HIGH (HIGH for in-tree Grok auth/update/telemetry surfaces; MEDIUM for peer-agent / community multi-provider failure modes)

## Critical Pitfalls

### Pitfall 1: Single-slot auth store becomes a second-provider overwrite

**What goes wrong:**
The shell’s `AuthManager` + `~/.grok/auth.json` model is built around **one** primary Grok/xAI credential shape (`GrokAuth`, scope keys like OIDC issuer or `xai::api_key`). Adding Codex/ChatGPT OAuth by stuffing a second token into the same “active key” slot (or reusing the same JSON shape without a provider namespace) causes: last login wins, silent loss of the other provider’s refresh token, or one refresh path wiping both.

**Why it happens:**
Single-provider clients grow a god-auth path. Multi-provider is bolted on as “another way to fill `key`” instead of a **provider-keyed credential map** with independent refresh chains.

**How to avoid:**
- Design `~/.bum/auth` (or multi-entry `auth.json`) as **provider-scoped** from day one: e.g. `xai` + `openai_codex` records, each with own access/refresh/expires/client_id/issuer.
- Never share refresh locks, `invalid_grant` wipe thresholds, or “active auth” enrichment (`/user`) across providers.
- Keep MCP OAuth store separate (already true) and **also** keep product providers separate from MCP.

**Warning signs:**
- Logging out of Codex clears Grok (or vice versa).
- Only one “logged in” indicator in UI despite two logins.
- Refresh for provider A rewrites the whole file and drops B’s fields.
- Tests only cover “one auth file → one bearer.”

**Phase to address:**
**Auth foundation / multi-provider identity** (before model picker multi-provider UX). Do not ship GPT models until dual-store + dual-refresh is solid.

---

### Pitfall 2: Wrong credential on the wrong backend (token/audience mismatch)

**What goes wrong:**
User selects a GPT model but the sampler still hits `cli-chat-proxy.grok.com` with an xAI bearer—or hits OpenAI with an xAI session token. Or ChatGPT subscription OAuth is sent as if it were a Platform API key (or both headers are attached). Result: 401/403 storms, opaque errors, accidental billing, or `HTTP 400 Multiple authentication credentials` (seen in multi-provider agents when env key + OAuth both inject).

**Why it happens:**
Inference path is a single `AuthCredentialProvider` + one base URL. Model switch only changes `model=` string, not **transport + credential + error classifier**.

**How to avoid:**
- Bind every model catalog entry to a **provider id** (routing table: model → backend + auth provider + request adapter).
- Resolve credentials **per request** from that provider id; never from “last successful login.”
- Attach **exactly one** auth mechanism per request (Bearer OAuth *or* API key, not both).
- Fail closed before the HTTP call if the selected model’s provider has no valid credential (project “missing-provider gate”).

**Warning signs:**
- Proxy logs show GPT model IDs on xAI proxy.
- Switching models does not change host in debug HTTP logs.
- Errors mention invalid API key while UI shows “ChatGPT logged in.”
- Dual env vars (`XAI_API_KEY` + `OPENAI_API_KEY`) both present and both attached.

**Phase to address:**
**Provider-aware request routing** (same milestone as Codex login; must land with or immediately after dual auth).

---

### Pitfall 3: ChatGPT/Codex OAuth ≠ OpenAI Platform API key (billing & capability trap)

**What goes wrong:**
Implementers treat “Codex OAuth” as a free way to call arbitrary OpenAI Platform APIs. Official Codex auth model is explicit: **Sign in with ChatGPT = subscription access / plan limits / workspace policy**; **API key = usage-based Platform billing**. ChatGPT plan does **not** pay for API-key spend. Feature availability, retention, and RBAC differ by sign-in method. Users get rate-limit / entitlement errors, surprise Platform bills, or models missing from one route.

**Why it happens:**
Both produce “a bearer string,” so forks reuse one OpenAI-compatible client. Docs/UX never name which pipe is active. Peer tools (OpenClaw-class) repeatedly open issues about Plus vs Codex OAuth vs API-key routing confusion.

**How to avoid:**
- v1 primary path = **ChatGPT/Codex OAuth** for GPT-5.6 coding models (per PROJECT.md); document API key as optional secondary if implemented at all.
- Label the model picker and status bar: provider + auth kind (`Codex · ChatGPT` vs `OpenAI · API key`).
- Use the **Codex-supported inference endpoint and model IDs** for subscription tokens; do not assume Chat Completions Platform endpoints accept ChatGPT OAuth.
- Surface quota/plan errors distinctly from “not logged in.”

**Warning signs:**
- Support questions: “I have Plus/Pro but bum bills my API org.”
- Same model works in stock `codex` CLI, fails in bum with 401/403.
- Entitlement errors after “successful” OAuth.

**Phase to address:**
**Codex OAuth + GPT model catalog** (docs + UX in same phase as login).

---

### Pitfall 4: Half-rebrand (identity leakage to stock Grok paths and product strings)

**What goes wrong:**
Binary is named `bum` but still defaults to `GROK_HOME` → `~/.grok`, User-Agent/`GROK_CLIENT_NAME` still “grok”, install strings say “Run `grok`”, marketplace still pulls xAI plugins as if stock, and users’ stock Grok sessions get clobbered (or bum silently shares credentials with `grok`). Product feels like a broken Grok Build, not a daily-driver identity.

**Why it happens:**
Home, env vars, binary name, chrome, and HTTP identity are **scattered** across `xai-grok-env`, shell config, update crate, pager chrome, and dozens of string literals. Rename stops at the binary target.

**How to avoid:**
- Single source of truth for product home default (`~/.bum`) and env override (`BUM_HOME` or keep `GROK_HOME` alias **documented** but default path must be bum).
- Inventory path writers: `auth.json`, `config.toml`, sessions, MCP credentials, marketplace-cache, memory DB.
- Inventory identity: User-Agent, client name/version, update “run CLI” messages, TUI titles, help text.
- Explicit non-goal in v1: do **not** import/share `~/.grok` or `~/.codex` (PROJECT.md); fail closed rather than half-share.
- Grep gates in CI for `~/.grok` defaults and `xai-grok-pager` user-facing names where product surface is intended.

**Warning signs:**
- Fresh login creates `~/.grok/auth.json`.
- Running stock `grok` and `bum` fights over the same config keys.
- Status line still says Grok Build / xAI-only branding with GPT models selected.

**Phase to address:**
**Product rename + isolated home** (early phase; unblocks clean auth testing without nuking real `~/.grok`).

---

### Pitfall 5: “Disable telemetry” leaves phone-home channels open

**What goes wrong:**
A single `telemetry_enabled = false` is treated as enough, but egress is multi-channel: Mixpanel (`xai-mixpanel`), product events client, Sentry DSN, external OTEL, cli-chat-proxy storage/trace upload, auto-update against `https://x.ai/cli`, asset CDN, plugin marketplace `xai-org/plugin-marketplace`, relay/gateway WS. Fork still phones home as a stock client; privacy goal fails; worst case auto-update **overwrites the fork binary** with upstream Grok.

**Why it happens:**
Each channel has its own kill switch / compile-time token / minimum_version force-update path. `minimum_version` can still attempt auto-update even when user expects quiet local builds.

**How to avoid:**
- Enumerate egress matrix from INTEGRATIONS.md and kill or no-op each for bum builds:
  - product telemetry modes → default `Disabled`
  - Sentry → no DSN / no init
  - Mixpanel → no token / no-op client
  - auto-update + minimum_version network → off; never install upstream over `bum`
  - optional: disable remote storage/trace upload and marketplace auto-fetch if they identify as stock Grok
- Prefer **compile-time** defaults for a private fork (not only runtime TOML a remote config can flip).
- Keep local debug logs; only block **network** phone-home.
- Add a smoke test: boot with network mocked/denied; assert no unexpected hosts (or unit-test each client’s disabled path).

**Warning signs:**
- Process still contacts `api.mixpanel.com`, Sentry, or `x.ai/cli` after “telemetry off.”
- Binary replaces itself after a version floor from remote config.
- `test_config_update_isolation`-style bugs: writes re-enable `auto_update`.

**Phase to address:**
**Quiet local fork** (can parallel rebrand; must land before daily-driver use).

---

### Pitfall 6: Global provider “mode” instead of per-model routing

**What goes wrong:**
UX becomes “switch to OpenAI mode” for the whole session. User cannot keep Grok for one turn and GPT for the next without losing context or re-login. Or mode is sticky and the model list filters incorrectly. Conflicts with PROJECT decision: **mixed picker, switch anytime**.

**Why it happens:**
Easier to gate one `AuthManager` + one sampler client behind a global enum than to route per model.

**How to avoid:**
- Model selector owns provider; session stores **current model id**, not “current provider mode.”
- History stays in one session; only the **next sample** chooses backend.
- Document provider gaps (tools that only work on one backend) rather than forcing global mode.

**Warning signs:**
- Settings expose “Provider: xAI | OpenAI” that resets model list and clears session.
- Switching model requires restart or re-auth every time.

**Phase to address:**
**Multi-provider session / model picker** (after dual auth + routing table exist).

---

### Pitfall 7: Auth error classification wipes the wrong store (or both)

**What goes wrong:**
This codebase already documents a **historical auth wipe race**: treating HTTP **403** as auth failure forced OIDC refresh + `auth_required` and raced `invalid_grant_threshold` into wiping `auth.json`. With two providers, a content-policy 403 or wrong-model 403 on OpenAI can be misread as “session dead,” refresh the wrong issuer, or wipe **both** credentials. Subagents/background turns amplify concurrent refresh.

**Why it happens:**
Single `is_auth_error` heuristic + one wipe path. Provider-agnostic middleware.

**How to avoid:**
- Keep **401-only** (or provider-documented auth statuses) for refresh triggers; never map policy 403 → wipe.
- Scope invalid_grant counters and wipe to **one provider record**.
- On auth failure, prompt login for **that** provider only; leave the other intact.
- Regression tests: 403 from sampling must not delete Codex or xAI tokens.

**Warning signs:**
- “I hit a content filter and got logged out of everything.”
- auth.json disappears after non-auth API errors.
- Rapid re-login loops (also seen in multi-profile openai-codex agents).

**Phase to address:**
**Auth foundation + routing error mapping** (extend existing `is_auth_error` contract when OpenAI paths land).

---

### Pitfall 8: Mid-session model switch without transport/capability renegotiation

**What goes wrong:**
Model string changes but: tool schema stays xAI-proxy shaped; system/prompt templates assume Grok; context window/token limits stay hardcoded; streaming event parser mismatches Responses vs Chat Completions; web search tool still tags auth as xAI; subagents ignore parent model and burn the default provider’s quota (peer agents: delegate model override ignored → unauthorized credits).

**Why it happens:**
“OpenAI-compatible” is treated as fully fungible. Agent loop was written for one proxy.

**How to avoid:**
- Explicit **request adapter** per provider (headers, base URL, tool wire format, max tokens).
- Propagate model+provider into subagent/delegate config; never silent fallback to paid default.
- On switch, re-validate tools against provider capabilities; disable or warn for unsupported tools.
- Test: switch Grok → GPT mid-session mid-tool-loop; assert host + auth + tool payload.

**Warning signs:**
- First message after switch always fails; second works (stale client).
- Subagent logs show different model than parent selected.
- Tool calls 400 only after provider switch.

**Phase to address:**
**Provider-aware routing + multi-provider session**; subagent inheritance in same or immediately following phase.

---

### Pitfall 9: Reusing stock Codex/OpenAI OAuth client_id and redirect semantics

**What goes wrong:**
Fork copies Codex CLI’s public client_id / redirect / loopback port conventions. Tokens may work until policy changes; refresh collides with stock `codex` CLI credential cache; multi-instance device/browser flows race; community discussion already notes multiple clients reusing the same Codex-era client ids. Legal/ToS and operational isolation suffer.

**Why it happens:**
Fastest path to “login works like Codex.”

**How to avoid:**
- Prefer a **bum-owned** OAuth client registration if OpenAI allows; if must mirror Codex CLI flow for subscription access, isolate storage under `~/.bum` and never write `~/.codex/auth.json`.
- Document dependency on whatever OAuth bridge is used; plan for client_id rotation.
- Do not read stock Codex auth as a silent fallback (out of scope for v1).

**Warning signs:**
- Logging into bum logs out stock Codex (shared keyring/file).
- Refresh in one CLI invalidates the other.

**Phase to address:**
**Codex OAuth login** design spike before implementation freeze.

---

### Pitfall 10: Incomplete home isolation (credentials yes, everything else no)

**What goes wrong:**
Auth moves to `~/.bum` but sessions, config.toml, MCP credentials, marketplace cache, memory SQLite, or debug logs still use `GROK_HOME`/`~/.grok`. Or env still honors a leftover `GROK_HOME` pointing at production Grok data while binary is bum—users think they isolated, then leak or clobber.

**Why it happens:**
Path helpers are duplicated; some crates take explicit `grok_home: PathBuf`, others call a global default.

**How to avoid:**
- One home resolver used by auth, config, session storage, MCP, updates, telemetry file sinks.
- Integration test: run with temp home; assert **zero** writes under `~/.grok` and `~/.codex`.
- Document env precedence: `BUM_HOME` / `GROK_HOME` alias behavior explicitly.

**Warning signs:**
- `find` after a session shows mixed homes.
- MCP OAuth tokens land in the wrong tree.

**Phase to address:**
**Isolated home** phase; verify again after each major feature lands.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Bolt Codex token into existing `GrokAuth.key` | Fast demo login | Wipe races, wrong refresh issuer, impossible multi-login | **Never** for v1 ship |
| Global “OpenAI mode” flag | Simple UI | Breaks mixed-picker product promise; rewrite later | Never (conflicts with Key Decisions) |
| Reuse cli-chat-proxy for GPT “somehow” | One HTTP client | Wrong auth, wrong models, ToS/billing confusion | Never for real GPT-5.6 |
| String-replace rebrand only in binary crate | `bum` launches | Half-identity, stock home, phone-home | Only as first PR if followed by home/identity inventory same phase |
| Disable Mixpanel only | Quick privacy win | Sentry/update/CDN still egress | Never as the only mitigation |
| Share `~/.codex` auth read-only | Skip OAuth UI | Couples lifecycle to stock CLI; out of scope | Defer post-v1 if ever |
| Append multi-provider into god files (`config.rs`, settings modal) | No extraction | Unreviewable auth security changes | Avoid; extract provider module boundaries even if god files remain |
| Ignore dual workspace-types / ACP TODOs for v1 | Stay on critical path | OK if **not** touching those surfaces | Acceptable **only** if multi-provider work does not expand ACP/workspace wire types |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| xAI OAuth (`auth.x.ai`) | Break existing device/browser flow while adding Codex | Preserve path; regression tests for device-code + browser; store under bum home only |
| Codex/ChatGPT OAuth | Treat as Platform API key | Subscription route + correct Codex endpoints; label auth kind in UX |
| OpenAI Platform API key | Assume same models/limits as ChatGPT login | Separate optional path; separate billing messaging |
| cli-chat-proxy | Send all models through proxy | Proxy **only** for xAI/Grok-routed models |
| Auth middleware (reqwest) | One middleware injects one global token | Provider-scoped credential injection per request |
| MCP OAuth | Merge MCP tokens into product auth.json | Keep MCP store separate; only change its home root |
| Auto-update (`x.ai/cli`) | Leave enabled “for convenience” | Disable; never replace bum with upstream grok |
| Sentry / Mixpanel / OTEL | Runtime toggle only | Default off; no secrets DSN in bum builds |
| Plugin marketplace | Auto-sync xAI marketplace as stock | Pin/disable or use bum-controlled sources |
| Subagents / leader multi-process | Refresh token only in main process | Shared provider store with flock + broadcast/re-read after refresh |

## Performance Traps

Less about “10k users,” more about interactive agent latency and auth storms.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Refresh stampede on multi-provider 401 | Latency spikes, invalid_grant, logout | Single-flight refresh **per provider**; coalesce waiters | Parallel tools + subagents |
| Full model list fetch every keystroke | Picker jank | Cache catalog; invalidate on login/logout | Large multi-provider lists |
| Rebuilding entire sampler client each turn | Multi-100ms overhead | Pool clients per provider base URL | Long sessions |
| Logging full Authorization headers | Disk fill + secret leak | Redact at telemetry/log boundaries | Any debug=true user |

## Security Mistakes

Domain-specific (beyond generic OWASP).

| Mistake | Risk | Prevention |
|---------|------|------------|
| Plaintext dual tokens in world-readable auth.json | Account takeover for **two** ecosystems | Mode `0600`; prefer OS keyring if added; never log tokens |
| Copying auth.json between machines without clock awareness | Weird expiry / mint_age; security confusion | Keep existing clock-skew comments; document isolation |
| Putting ChatGPT OAuth tokens into prompts, traces, or upload paths | Credential exfiltration via product telemetry | Redact at all egress; disable upload for quiet fork |
| Using stock Codex client as “impersonation” without isolation | Cross-product session fixation / logout wars | Isolated `~/.bum`; no shared auth.json |
| Prompt injection exfiltrating `~/.bum/auth` via bash tools | Same as any agent; **higher** impact with two cloud accounts | Permissions + sandbox; deny globs on auth paths; teach model less about secret paths |
| Treating XOR prompt templates as confidentiality | False sense of security | Already non-boundary; don’t “encrypt” tokens the same way |
| Mis-handling 403 → wipe | Forced re-auth / DoS of local credentials | 401-only auth errors; provider-scoped wipe |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Silent fail mid-turn when GPT selected but Codex logged out | “Agent is broken” | Block + prompt **Codex** login before send |
| Single “Login” that only does xAI | Users never discover second provider | Explicit `login xai` / `login codex` (CLI + TUI) |
| No provider badge on model rows | Picks GPT without realizing billing/auth pipe | Badge provider + auth state (✓/login needed) |
| Errors say “unauthorized” without provider | User re-logs the wrong account | “OpenAI/Codex credentials expired” vs “xAI credentials expired” |
| Rebrand incomplete in help/`--version` | Distrust / support confusion | Consistent `bum` naming in all user-facing surfaces |
| Auto-update notification for stock Grok | Fear of overwrite or noise | No update channel in quiet fork |

## "Looks Done But Isn't" Checklist

- [ ] **Dual login:** Can be logged into **both** xAI and Codex simultaneously; logout is per-provider
- [ ] **Home isolation:** Cold start writes **only** under `~/.bum` (or test home)—not `~/.grok` / `~/.codex`
- [ ] **Routing:** Debug/trace shows Grok models → xAI proxy host; GPT models → OpenAI/Codex host
- [ ] **Missing-provider gate:** Selecting GPT with no Codex auth never fires an xAI request
- [ ] **Refresh isolation:** Force-expire Codex access token; Grok continues; only Codex re-auths
- [ ] **403 safety:** Synthetic content-policy 403 does not wipe auth files
- [ ] **No dual headers:** Request has either OAuth bearer or API key, never both
- [ ] **Telemetry quiet:** No Mixpanel/Sentry/update calls on default bum config
- [ ] **Auto-update off:** minimum_version / update path cannot install upstream over bum
- [ ] **Picker:** Mixed list; switch mid-session without restart
- [ ] **Subagents:** Inherit selected model/provider (or explicit override)—no silent default provider spend
- [ ] **Identity:** `--help`, TUI chrome, User-Agent, and error strings say bum (not stock grok product)
- [ ] **xAI regression:** Existing device-code / browser xAI login still works under bum home
- [ ] **Credential file perms:** auth store not group/world readable
- [ ] **God-file containment:** New multi-provider code not only appended into 10k-LOC modules without module boundary

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Auth file wiped / corrupted | MEDIUM | Delete provider record; re-OAuth; keep other provider if dual-store designed correctly—if single-slot, re-login both |
| Wrong-backend billing (API key used unintentionally) | HIGH (money) | Rotate keys; disable API-key path; clear env; audit Platform usage dashboard |
| Stock home clobber (`~/.grok`) | MEDIUM–HIGH | Restore backup if any; stop bum; fix home default; re-login stock grok |
| Auto-update overwrote bum binary | HIGH | Reinstall bum from known artifact; disable update; pin path |
| Reauth loop | LOW–MEDIUM | Clear only failing provider tokens; fix clock; ensure single-flight refresh; upgrade after fix |
| Partial rebrand confusion | LOW | Complete string/path inventory; ship patch release |
| Subagent wrong provider spend | MEDIUM | Stop session; fix inheritance; review provider dashboards |

## Pitfall-to-Phase Mapping

Suggested phase themes for roadmap (names illustrative; orchestrator may renumber).

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Half-rebrand / path identity | **Phase: Product rename + `~/.bum` home** | Temp-home e2e: zero `~/.grok` writes; chrome says bum |
| Incomplete home isolation | Same phase + regression after later phases | Path inventory test suite |
| Quiet fork / phone-home | **Phase: Disable telemetry + auto-update** | Network allowlist test / unit no-op clients |
| Single-slot auth overwrite | **Phase: Multi-provider auth store** | Dual login/logout/refresh tests |
| Auth wipe / 403 misclass | Same auth phase | 403 does not delete tokens |
| Codex OAuth client/storage | **Phase: Codex/ChatGPT OAuth login** | Login UX; tokens only under bum; stock codex untouched |
| ChatGPT OAuth vs API key confusion | Same + docs/UX | Labels; no silent Platform billing path |
| Wrong credential / backend | **Phase: Provider-aware routing** | Host+auth assertions per model |
| Global mode vs per-model | **Phase: Mixed model picker / multi-provider session** | Mid-session switch without mode flag |
| Mid-session capability drift / subagents | Routing + session phase | Switch + tool loop + subagent model tests |
| Client_id / stock Codex collision | Codex OAuth design + isolation | No writes to `~/.codex` |
| God-file security blind spots | Continuous; extract auth/routing modules when touching | Review checklist on auth PRs |

**Ordering rationale:** Home + identity first (safe sandbox for auth). Quiet fork early (no overwrite/phone-home while iterating). Dual auth store before UX. Routing before or with GPT models in picker. Session/switch UX last among core v1, once pipes are correct.

## Sources

**In-repo (HIGH confidence for this fork):**
- `.planning/PROJECT.md` — v1 scope: dual OAuth, per-model routing, `~/.bum`, quiet fork, no credential sharing
- `.planning/codebase/CONCERNS.md` — auth wipe race (403), auth diagnostic gaps, god files, ACP debt
- `.planning/codebase/INTEGRATIONS.md` — auth.x.ai, cli-chat-proxy, Mixpanel, Sentry, auto-update `x.ai/cli`, `GROK_HOME`/`auth.json`, MCP separate OAuth
- `crates/codegen/xai-grok-shell/src/auth/model.rs` — single `GrokAuth` shape, API key scope, refresh fields
- `crates/codegen/xai-grok-update` — auto_update / minimum_version force paths

**External (MEDIUM unless noted):**
- OpenAI Codex authentication docs — ChatGPT sign-in vs API key; plan vs Platform billing; credential storage under CODEX_HOME (`learn.chatgpt.com/docs/auth`) — **HIGH for official behavior**
- Community / multi-agent trackers (Hermes-agent, OpenClaw): dual-auth header collisions, openai-codex reauth loops across profiles, OAuth token not synced to sub-agents, model override ignored on delegates, ChatGPT vs Codex OAuth vs API-key UX confusion, client_id reuse discussions
- OAuth practice: merge-not-replace refresh responses; single-flight refresh; wrong-audience tokens fail at resource servers

**Gaps (phase-specific research later):**
- Exact ChatGPT OAuth client registration options and allowed inference endpoints for third-party CLIs (confirm at implementation time against current Codex CLI behavior)
- Precise GPT-5.6 model IDs and tool/schema parity on Codex subscription route vs Platform
- Whether OS keyring should be v1 or post-v1 for `~/.bum` secrets

---
*Pitfalls research for: bum multi-provider OAuth coding-agent fork*
*Researched: 2026-07-16*
