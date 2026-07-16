# External Integrations

**Analysis Date:** 2026-07-16

## APIs & External Services

**xAI / Grok model & chat proxy:**
- Primary OpenAI-compatible inference path via **cli-chat-proxy**
  - Default base URL: `https://cli-chat-proxy.grok.com/v1` (`crates/codegen/xai-grok-env/src/lib.rs`)
  - Override: `GROK_PRODUCTION_CLI_CHAT_PROXY_BASE_URL` or documented `GROK_CLI_CHAT_PROXY_BASE_URL` for enterprise proxies
  - Client: `async-openai` + custom streaming in `crates/codegen/xai-grok-sampler`
  - Auth: Bearer session token or API key via `xai-grok-auth` / shell `AuthManager`
- Direct xAI API surface used for voice and some agent defaults:
  - `https://api.x.ai` / `https://api.x.ai/v1` (STT, model base URL defaults)
  - STT WebSocket: `wss://api.x.ai/v1/stt` (`crates/codegen/xai-grok-voice`)
- Asset CDN: `https://assets.grok.com` (version/channel assets, installers)

**Authentication (SpaceXAI / customer IdP):**
- Default browser OAuth at **`https://auth.x.ai`** (`XAI_OAUTH2_ISSUER` in `crates/codegen/xai-grok-shell/src/auth/config.rs`)
- Accounts app origins: `https://accounts.x.ai` (device-code / account UI)
- Device-code flow for headless environments (`auth/device_code.rs`, pager auth UI)
- Customer OIDC SSO: configurable issuer + client_id (`[grok_com_config.oidc]`, env `GROK_OIDC_ISSUER` / `GROK_OIDC_CLIENT_ID`)
- External auth provider command (stdout token) for air-gapped / CI
- API key: `XAI_API_KEY` from [console.x.ai](https://console.x.ai); stored/looked up alongside session in auth store
- Local-dev issuer: `http://localhost:22255` when `GROK_LOCAL_AUTH=1`
- Credential store: `~/.grok/auth.json` (`GROK_HOME`, `GROK_AUTH_PATH`, inline `GROK_AUTH`)

**WebSockets (product surfaces):**
- Relay (web frontend driving local agent): `wss://code.grok.com/ws/code-agent` — override `GROK_PRODUCTION_WS_URL`
- Cloud sandbox gateway: `wss://grok.com/ws/gw/` — override `GROK_PRODUCTION_GATEWAY_WS_URL` / `GROK_GATEWAY_URL`
- WS Origin: `https://grok.com`

**Web search / fetch tools:**
- `WebSearch` / `WebFetch` tools in `crates/codegen/xai-grok-tools/src/implementations/web_search/`
  - Uses Responses-style API tool calling against the authenticated proxy/backend
  - Auth consumer tag `WebSearch` for 401 attribution

**MCP (Model Context Protocol):**
- Client crate: `crates/codegen/xai-grok-mcp` (`rmcp 2.1`)
- Transports: stdio, streamable HTTP (reqwest 0.13)
- Per-server OAuth (browser + loopback callback via axum); DCR or BYO client id
- Credentials: `$GROK_HOME/mcp_credentials.json`
- Config-driven server list + managed MCP from proxy

**Agent Client Protocol (ACP):**
- Library: `crates/codegen/xai-acp-lib` on `agent-client-protocol`
- Used for editor embedding and headless ACP session mode in `xai-grok-shell`

**Plugin marketplace:**
- Official git source: `https://github.com/xai-org/plugin-marketplace.git` (`xai-grok-plugin-marketplace`)
- Also understands Claude-compatible marketplace indexes (e.g. anthropics official schema URLs in tests)
- Cache: `$GROK_HOME/marketplace-cache`; installs via git clone

**Computer Hub:**
- SDK: `crates/common/xai-computer-hub-sdk` — WebSocket pool, tool harness, OIDC provider hook, OTLP/fastrace donation
- Adapter: `xai-computer-hub-mcp-adapter` for MCP-facing computer tools
- Used by tool implementations under `xai-grok-tools` (`computer/`)

**Auto-update / distribution:**
- Version check / install against `https://x.ai/cli` (`CLI_BASE_URL_PRIMARY` in `xai-grok-update`)
- Installer modes: `internal` (install.sh/CDN), `npm` (`@xai-official/grok`), `gh-release` (GitHub Releases `xai-org-shared/grok-build`)
- Env override: `GROK_INSTALLER`

**Product analytics:**
- Mixpanel HTTP API: `https://api.mixpanel.com/track` and `/engage` (`crates/codegen/xai-mixpanel`)
  - Token: config / `GROK_TELEMETRY_BUILD_MIXPANEL_TOKEN` (compile-time option)

**Error tracking:**
- Sentry (`sentry` crate) in `crates/codegen/xai-grok-telemetry/src/sentry.rs`
  - DSN: env `SENTRY_DSN` or compile-time `option_env!("SENTRY_DSN")`
  - Scrubs secrets and home/username paths before send

## Data Storage

**Databases:**
- **SQLite (bundled rusqlite 0.37)** — local memory index, FTS5, sqlite-vec KNN (`crates/codegen/xai-grok-memory`)
  - Journal mode selection via `xai-sqlite-journal` (WAL local; rollback on network mounts)
  - Connection: local files under Grok home / project paths (no remote DB URL)
- Session / chat history — JSONL and related files under shell storage (`xai-grok-shell/src/session/storage/`)

**File Storage / object storage:**
- **cli-chat-proxy storage API** — `StorageClient` in `crates/codegen/xai-file-utils` posts to proxy `/v1/storage/*` (upload, download, exists, limits) with live auth refresh
- Optional **GCS** (`gcloud-storage`) and **S3** (`aws-sdk-s3`) for direct/signed upload paths used by trace/export helpers
- Trace / feedback / config-file uploads orchestrated from `xai-grok-shell/src/upload/`

**Caching:**
- In-process: `moka`, `OnceLock` shared HTTP clients (`xai-grok-http`, sampler shared pool)
- Disk: marketplace-cache, managed deployment config cache (TTL via `GROK_DEPLOYMENT_CONFIG_CACHE_TTL_SECS`)
- HTTP connection pools (sampler: `GROK_POOL_*`, `GROK_CONNECT_TIMEOUT_SECS`, kill-switch `GROK_SAMPLER_SHARED_CLIENT=0`)

## Authentication & Identity

**Auth Provider:**
- SpaceXAI OAuth2/OIDC at `auth.x.ai` (default)
- Customer OIDC IdPs (Okta, Azure AD, Auth0, etc.)
- Device authorization grant
- API keys (`XAI_API_KEY` / auth.json)
- External command provider
- Deployment / managed config can disable API-key auth (`GROK_DISABLE_API_KEY_AUTH`) or force team UUID

**Implementation:**
- Traits: `HttpAuth`, `AuthCredentialProvider` in `crates/codegen/xai-grok-auth`
- Full flows + token refresh: `crates/codegen/xai-grok-shell/src/auth/`
- Middleware attaches credentials to outbound HTTP (reqwest-middleware)
- MCP OAuth separate store (does not share grok.com tokens)

## Monitoring & Observability

**Error Tracking:**
- Sentry (see above)

**Product telemetry:**
- `xai-grok-telemetry` — events client, Mixpanel, session metrics modes (`Disabled` | `SessionMetrics` | `Enabled`)
- Events URL / API key: runtime config + optional compile-time `GROK_TELEMETRY_BUILD_EVENTS_*`
- Redaction: `xai-grok-secrets` scrubbers on outbound payloads

**OpenTelemetry:**
- Internal OTLP export paths + **external OTEL** toggles:
  - `GROK_EXTERNAL_OTEL`, `otel_endpoint`, `otel_protocol` (`http/protobuf` | `grpc`)
  - Metrics/logs exporters: `otlp` | `console` | `none`
  - Content gates: `otel_log_user_prompts`, `otel_log_tool_details`
- Libraries: `opentelemetry`, `opentelemetry-otlp`, `opentelemetry-http`, `tracing-opentelemetry`

**Logs:**
- `tracing` / `tracing-subscriber` (env-filter, JSON, chrome)
- Optional files: `GROK_LOG_FILE`, `GROK_DEBUG_LOG`, hooks/sampling log env flags
- Local debug and sampling logs under telemetry module

## CI/CD & Deployment

**Hosting:**
- End-user CLI distributed via x.ai CDN, npm, GitHub Releases (not a long-running cloud service in this repo)
- Backend services (cli-chat-proxy, assets, relay, gateway) are external to this tree; client URLs live in `xai-grok-env`

**CI Pipeline:**
- Not fully present in this extracted tree (synced from SpaceXAI monorepo); local validation is cargo check/test/clippy
- Release-dist profile + musl RELRO flags prepared for shipping binaries

## Environment Configuration

**Required / commonly used env vars:**

| Variable | Purpose |
|----------|---------|
| `XAI_API_KEY` | API-key auth fallback |
| `GROK_HOME` | Config/credential root (default `~/.grok`) |
| `GROK_AUTH` / `GROK_AUTH_PATH` | Inline or alternate auth JSON |
| `GROK_PRODUCTION_CLI_CHAT_PROXY_BASE_URL` | Override chat proxy base |
| `GROK_PRODUCTION_ASSET_SERVER_URL` | Override assets CDN |
| `GROK_PRODUCTION_WS_URL` | Override relay WebSocket |
| `GROK_PRODUCTION_GATEWAY_WS_URL` / `GROK_GATEWAY_URL` | Cloud sandbox gateway |
| `GROK_PRODUCTION_WS_ORIGIN` | WebSocket origin |
| `GROK_OIDC_ISSUER` / `GROK_OIDC_CLIENT_ID` | Customer SSO |
| `GROK_OAUTH2_ISSUER` / `GROK_OAUTH2_CLIENT_ID` | OAuth2 provider config |
| `GROK_LOCAL_AUTH` | Point OAuth at local accounts-app |
| `SENTRY_DSN` | Error reporting |
| `GROK_INSTALLER` | Update channel: npm / internal / gh-release |
| `GROK_CLIENT_NAME` / `GROK_CLIENT_VERSION` | Origin / User-Agent product identity |
| `GROK_SHELL` | Preferred shell |
| `PROTOC` | Override protoc binary for builds |
| `GROK_VERSION` | Injected package version at build |

**Secrets location:**
- Runtime: user home `$GROK_HOME` (`auth.json`, `mcp_credentials.json`, config.toml) — **never commit**
- Build-time embeds: optional `option_env!` for Sentry DSN / Mixpanel / events keys in release pipelines
- Redaction helpers: `crates/codegen/xai-grok-secrets`

**Do not read or document secret values from `.env` files if present.**

## Webhooks & Callbacks

**Incoming:**
- Local OAuth callback HTTP servers (loopback random port) for:
  - Grok login OIDC/OAuth (`auth` flow)
  - MCP server OAuth (`xai-grok-mcp/src/oauth.rs`)
- No public inbound webhook endpoints in this client repo

**Outgoing:**
- HTTPS to cli-chat-proxy, api.x.ai, assets.grok.com, Mixpanel, Sentry, OTLP collectors, storage proxy
- WebSockets to relay, gateway, STT, computer hub
- Git clone HTTPS to marketplace repositories
- Optional user-configured MCP HTTP endpoints

## Integration Map (by crate)

| Concern | Primary crates |
|---------|----------------|
| Endpoint defaults | `xai-grok-env` |
| HTTP clients / UA | `xai-grok-http`, `xai-grok-sampler` |
| Auth traits | `xai-grok-auth` |
| Auth flows | `xai-grok-shell` (`src/auth/`) |
| Inference | `xai-grok-sampler`, `xai-grok-models` |
| Tools + web search | `xai-grok-tools` |
| MCP | `xai-grok-mcp` |
| Telemetry | `xai-grok-telemetry`, `xai-mixpanel` |
| Storage uploads | `xai-file-utils`, `xai-grok-shell` (`upload/`) |
| Updates | `xai-grok-update` |
| Voice STT | `xai-grok-voice` |
| Plugins | `xai-grok-plugin-marketplace`, `xai-grok-agent` |
| Computer hub | `xai-computer-hub-sdk`, `xai-computer-hub-mcp-adapter` |
| Sandbox | `xai-grok-sandbox` |
| Config | `xai-grok-config`, `prod/mc/cli-chat-proxy-types` |

## Default Models (shipped)

From `crates/codegen/xai-grok-models/default_models.json`:
- Default model: `grok-build` (Responses API backend)
- Web search model slot: `grok-4.20-multi-agent`
- Image description / session summary: `grok-build`

---

*Integration audit: 2026-07-16*
