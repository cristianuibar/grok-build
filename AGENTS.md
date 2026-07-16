<!-- GSD:project-start source:PROJECT.md -->

## Project

**bum**

**bum** is a full-product fork of Grok Build (this repo): a terminal AI coding agent TUI/CLI that you launch as `bum` instead of `grok`. v1 keeps the Grok Build agent/TUI harness and makes it a multi-provider daily driver — xAI OAuth for Grok models, ChatGPT/Codex OAuth for GPT-5.6 models, with per-model routing and an isolated `~/.bum` identity. Later milestones will layer custom agentic workflows on the same harness.

**Core Value:** You can run one CLI (`bum`), log into both xAI and Codex, and freely switch between Grok and GPT-5.6 models in a real coding session without leaving the tool.

### Constraints

- **Tech stack**: Stay on this Rust workspace (edition 2024, Tokio, existing TUI/agent crates) — fork evolution, not a rewrite
- **Identity**: ChatGPT OAuth for Codex (not API-key-only as the primary path); xAI OAuth preserved for Grok
- **Routing**: Per-model provider selection — mixed picker, not a global “mode” that filters the whole session
- **Storage**: `~/.bum` isolation; no credential sharing with stock CLIs in v1
- **Privacy**: No xAI auto-update; no product telemetry phone-home
- **Compatibility**: Tool/session behavior should remain usable as a daily driver for both providers (provider gaps documented if OpenAI/Codex cannot support a Grok-only tool feature)
- **Naming**: Product and CLI are `bum`; avoid half-rename that still presents as Grok Build to the user

<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->

## Technology Stack

## Languages

- Rust (edition **2024**, toolchain **1.92.0** pinned in `rust-toolchain.toml`) — entire product: TUI, agent runtime, tools, auth, telemetry, MCP, sandbox
- Protocol Buffers — tool wire API in `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto` (codegen via `tonic` / `prost` / `xai-proto-build`)
- TOML / JSON / YAML — config (`~/.grok/config.toml`), models (`default_models.json`), marketplace indexes, session persistence
- Node.js (≥20, optional packaging only) — npm wrapper package `@xai-official/grok` under `crates/codegen/xai-grok-pager/npm/` (postinstall binary selector)
- Shell / PowerShell — install scripts referenced by auto-update (`install.sh` / `install.ps1` at `https://x.ai/cli`)
- Python — minor helper scripts (e.g. `crates/codegen/xai-grok-agent/scripts/encrypt_templates.py`, hook examples)

## Runtime

- Native Rust binary (default artifact name `xai-grok-pager`; shipped/installed as `grok`)
- Async runtime: **Tokio 1** (`features = ["full"]` at workspace level)
- Allocator (Unix, default feature): **jemalloc** via `tikv-jemallocator` / `tikv-jemalloc-ctl` / `tikv-jemalloc-sys` in `crates/codegen/xai-grok-pager-bin`
- TLS: **rustls** (reqwest `rustls-tls`; binary also depends on `rustls` 0.23 with `ring`)
- **Cargo** (workspace, `resolver = "2"`)
- Lockfile: `Cargo.lock` present (committed)
- Root `Cargo.toml` is **generated** — treat as read-only; edit per-crate manifests
- Optional distribution: npm (`@xai-official/grok` + platform packages)

## Frameworks

- **ratatui 0.29** + **crossterm 0.28** — fullscreen TUI (`xai-grok-pager`, `xai-ratatui-inline`, `xai-ratatui-textarea`)
- **tokio** — multi-thread async agent loop, tools, HTTP, websockets
- **clap 4** — CLI parsing
- **agent-client-protocol (ACP) 0.10.4** — editor embedding protocol (`xai-acp-lib`, shell ACP session)
- **async-openai 0.33** — OpenAI-compatible Responses / Chat Completions client surface in sampler
- **rmcp 2.1** — Model Context Protocol client (isolated in `xai-grok-mcp` with **reqwest 0.13**)
- **tonic / prost 0.14** — gRPC/protobuf for tools API and OTLP internals
- **axum 0.8** — local loopback servers (OAuth callbacks, test fixtures, device-auth helpers)
- Built-in `cargo test` (per-crate; workspace-wide builds are slow)
- **insta** — snapshot tests (pager)
- **pretty_assertions**, **assert_matches**, **serial_test**
- **mockito**, **wiremock**, **axum** test servers — HTTP/SSE fixtures
- **criterion** — benchmarks (markdown, pager, fsnotify, PTY harness)
- Crate-local integration tests under many `crates/codegen/*/tests/`
- **rustc / cargo** via `rust-toolchain.toml`
- **protoc** — `bin/protoc` (dotslash launcher) or `$PROTOC` / PATH (`crates/build/xai-proto-build`)
- **rustfmt** (`rustfmt.toml`: `use_field_init_shorthand = true`)
- **clippy** (`clippy.toml` — bans raw `canonicalize` in favor of `dunce`)
- Profiles: `dev`, `release`, `release-dist` (thin LTO, CGU=1), `release-dist-jemalloc`, `x-prod`, `bench` in root `Cargo.toml`
- Target rustflags / jemalloc page sizes: `.cargo/config.toml`

## Key Dependencies

- `reqwest 0.12` (workspace default; rustls, stream, json, multipart, http2, blocking, socks) — almost all product HTTP
- `reqwest 0.13` (MCP crate only) — rmcp transport isolation
- `async-openai` — model streaming / Responses API wire shape
- `serde` / `serde_json` / `toml 0.9` — config and persistence
- `rusqlite 0.37` (`bundled`) + `sqlite-vec` — memory / FTS / vector index (`xai-grok-memory`, `xai-sqlite-journal`)
- `gix` / `git2` — Git status and metadata without shelling out for hot paths
- `portable-pty` / `alacritty_terminal` / `vte` — integrated terminal
- `syntect` / `two-face` / `pulldown-cmark` — syntax highlight + markdown render
- `mermaid-to-svg` (vendored under `third_party/`) + `resvg` / `tiny-skia` — diagram rendering
- `oauth2 5` — OAuth/OIDC token exchange (auth + MCP)
- `sentry 0.42` — crash/error reporting (`xai-grok-telemetry`)
- `opentelemetry` / `opentelemetry-otlp` / `tracing-opentelemetry` — product + external OTEL
- `aws-sdk-s3` / `gcloud-storage` — optional direct object storage for uploads (`xai-file-utils`)
- `nono` (unix) — Landlock / Seatbelt sandbox (`xai-grok-sandbox`)
- `nucleo` (git pin) — fuzzy matching
- `obfstr` / `cryptify` — string / control-flow obfuscation on shipping binary
- `tracing` + `tracing-subscriber` + `tracing-appender` / `tracing-chrome` — logging and chrome traces
- `prometheus` — optional metrics (computer-hub)
- `blake3` / `sha2` / `ring` / `rsa` — hashing and crypto
- `notify` / `notify-debouncer-mini` — filesystem watch
- `tokio-tungstenite` — WebSockets (relay, STT, computer hub)
- `minijinja` — templating (agent prompts)
- `jsonschema` / `schemars` — schema validation
- `ts-rs` — TypeScript type export where needed
- `fastrace` + OTLP bridges — distributed-style span donation (computer hub)

## Configuration

- Backend endpoints resolved in `crates/codegen/xai-grok-env` (production defaults + `GROK_PRODUCTION_*` overrides)
- User config: TOML under `$GROK_HOME` (default `~/.grok`) via `xai-grok-config` — layers: requirements > user > managed/MDM
- Auth credentials: `~/.grok/auth.json` (or `GROK_AUTH` / `GROK_AUTH_PATH`); API key `XAI_API_KEY`
- Telemetry tokens/DSN may be compile-time (`option_env!`) or runtime (`SENTRY_DSN`, Mixpanel, events URL)
- Version override: `GROK_VERSION` (build), `GROK_TEST_VERSION` (tests)
- `Cargo.toml` (workspace root, generated)
- Per-crate `Cargo.toml` under `crates/codegen/`, `crates/common/`, `crates/build/`, `prod/mc/`, `third_party/`
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml`
- Binary features on `xai-grok-pager-bin`: `jemalloc` (default), `sandbox-enforce` (default), `release-dist`

## Platform Requirements

- Rust 1.92.0 via rustup (auto from `rust-toolchain.toml`)
- `protoc` (repo `bin/protoc` + [dotslash](https://dotslash-cli.com), or system `$PROTOC`)
- macOS and Linux supported build hosts; Windows best-effort / not fully tested from this tree
- Target specific crates for check/test (`cargo check -p <crate>`) — full workspace is heavy
- Prebuilt binaries for macOS, Linux, Windows (x64/arm64) via:
- Distribution profile: `cargo build --profile release-dist` (hardened link flags on musl targets)
- Product docs: [docs.x.ai/build/overview](https://docs.x.ai/build/overview); marketing: [x.ai/cli](https://x.ai/cli)

## Workspace Layout (tech-relevant)

| Area | Path | Role |
|------|------|------|
| Binary composition root | `crates/codegen/xai-grok-pager-bin` | Links pager + shell + update + crash handler |
| TUI | `crates/codegen/xai-grok-pager` | Scrollback, prompt, modals, rendering |
| Agent runtime | `crates/codegen/xai-grok-shell` | Leader/stdio/headless, auth, session, remote |
| Tools | `crates/codegen/xai-grok-tools` | Shell, edit, search, web_search, computer tools |
| Inference | `crates/codegen/xai-grok-sampler` | Streaming HTTP to model backends |
| Shared leaves | `crates/common/*` | Circuit breaker, tool protocol/runtime, computer hub, compaction |
| Vendored | `third_party/*` | Mermaid layout/render stack (dagre, graphlib, mermaid-to-svg) |
| Proxy types | `prod/mc/cli-chat-proxy-types` | Shared types with cli-chat-proxy |

## Key Commands

<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

## Naming Patterns

- Rust modules: `snake_case.rs` (e.g. `workspace_classifier.rs`, `signed_policy.rs`)
- Module as directory when multi-file: `signed_policy/mod.rs` + sibling files (`tests.rs`, helpers)
- Integration tests under `tests/`: prefer descriptive names
- Snapshot files: auto-named under `snapshots/` next to the module (insta layout)
- Crate package names: `xai-` prefix, kebab-case (`xai-grok-hooks`, `xai-file-utils`)
- Binary artifact: `xai-grok-pager` (shipped as `grok`)
- `snake_case` for all functions and methods
- Test functions: descriptive behavior phrases, no `test_` prefix required when the module is already a test module — e.g. `actor_spawns_and_shuts_down_via_cancellation`, `static_api_key_is_fallback_when_provider_returns_none`
- Getters without `get_` prefix: `endpoints()`, `cli_chat_proxy_base_url()`, `as_str()`
- Constructors: `new`, `with_*` builders, domain verbs (`spawn`, `load_*`, `dispatch_*`)
- Conversion helpers: `from_*`, `into_*`, `*_str`
- `snake_case` locals and parameters
- Prefer short, domain-clear names (`dir`, `ctx`, `handle`, `token`)
- Env var names: `SCREAMING_SNAKE` with `GROK_` prefix for product knobs (`GROK_BINARY`, `GROK_HOME`, `GROK_LEADER_SOCKET`)
- `PascalCase` for structs, enums, traits (`WebSearchClient`, `SamplingError`, `HookDecision`)
- Error enums: `*Error` suffix (`SamplingError`, `ToolErrorWire`, `RequirementsError`)
- Type aliases: `Result<T>` scoped to the module’s error type when a crate has a primary error
- Constants: `SCREAMING_SNAKE` (`DEFAULT_TOOL_OUTPUT_BYTES`, `PRODUCTION_ENDPOINTS` for static data may be `PascalCase` when typed structs)
- Trait object / shared handles: `Shared*` prefix (`SharedApiKeyProvider`, `SharedAttributionCallback`)
- Prefer focused modules over mega-files; split large domains (`agent/`, `session/`, `implementations/`)
- Private modules by default (`mod loader`); only `pub mod` the intentional public surface
- Re-export curated API at crate root via `pub use` (see `crates/codegen/xai-grok-config/src/lib.rs`)

## Code Style

- Tool: `rustfmt` (component pinned in `rust-toolchain.toml`)
- Config: `rustfmt.toml` at repo root — `use_field_init_shorthand = true`
- Run: `cargo fmt --all`
- Edition: workspace `edition = "2024"` (`Cargo.toml` `[workspace.package]`)
- Tool: `clippy` (component pinned in `rust-toolchain.toml`)
- Root config: `clippy.toml` — applies to codegen crates; **nearest** `clippy.toml` wins (no merge)
- Workspace clippy lints: `[workspace.lints.clippy]` in root `Cargo.toml`; crates opt in with:
- Notable allowed lints (workspace): `doc_lazy_continuation`, `doc_overindented_list_items`, `needless_lifetimes`, `single_range_in_vec_init`, `too_many_arguments`, `uninlined_format_args`, `useless_format`
- **Disallowed methods** (in `clippy.toml`): `std::fs::canonicalize`, `std::path::Path::canonicalize`, `tokio::fs::canonicalize` — use `dunce::canonicalize` (or tools helpers in `xai_grok_tools::util::fs` for async)
- Per-crate `#![allow(...)]` is common on large crates for dead-code/unused during rapid development (`xai-grok-shell`, `xai-grok-env`, etc.)
- Prefer crate-level allow lists over scattering many local allows when the crate is transitional
- Run: `cargo clippy -p <crate>` (prefer per-crate; full workspace is slow)
- Pinned stable in `rust-toolchain.toml` (currently `1.92.0`)
- Bump policy (from file comments): one point version at a time; wait weeks after release; re-run `cargo check/clippy --all-targets --workspace`

## Import Organization

- Group with blank lines between std / external / crate when the file is large
- Prefer `use crate::module::Item` over deep relative paths for cross-module refs
- Import macros/types used in tests inside the `#[cfg(test)]` module, not at crate top, when only tests need them
- Wiremock and heavy test deps: often imported inside the individual test function (see `crates/codegen/xai-grok-tools/src/implementations/web_search/client.rs`)
- No `paths` aliases in cargo sense; use package names as crates (`xai_grok_hooks`, `xai_file_utils`)
- Common re-export aliases: `use xai_grok_http as http` (shell)
- Prefer `{ workspace = true }` in per-crate `Cargo.toml`
- Root `Cargo.toml` is **generated** — treat as read-only; edit per-crate manifests and workspace dep versions only when regenerating workflow allows
- Add new shared deps under `[workspace.dependencies]` when used by multiple crates

## Error Handling

- **Library / domain errors:** `thiserror` enums with `#[error("...")]` display strings
- **Application / orchestration:** `anyhow::Result` + `.context()` / `.with_context()` at boundaries (shell, harness, CLI paths)
- Prefer structured variants with fields over stringly-typed errors when callers branch on kind
- Wire-facing errors: `Serialize`/`Deserialize` with stable `code` discriminators (`#[serde(tag = "code", rename_all = "snake_case")]`)
- Map foreign errors with `map_err` into domain types; keep messages human-readable for logs and UI
- Local `pub type Result<T> = std::result::Result<T, DomainError>` next to the primary error enum when the crate is error-centric

#[derive(Debug, thiserror::Error)]

- Silent `unwrap()` in production paths — reserve for invariants with `.expect("why this cannot fail")`
- Returning bare `String` as the only error type for public APIs when variants matter
- Using `std::fs::canonicalize` (Windows verbatim path hazard) — use `dunce::canonicalize`
- Return `Result` from `new` when config can be invalid (e.g. `WebSearchClient::new`)
- Document when `Err` is returned in the doc comment

## Logging

- Use structured fields: `tracing::warn!(error = %e, path = %path.display(), "failed to read file")`
- Prefer `%` / `?` format in field values for `Display` / `Debug`
- Levels: `error!` for failures that need attention, `warn!` for degraded modes, `info!` for lifecycle milestones, `debug!` for detailed state changes
- Shell also routes through `xai_grok_telemetry` / `xai_tracing_macros` (`teprintln!`, `timed!`, `tprintln!`) for specialized instrumentation
- Avoid logging secrets (API keys, tokens); redaction lives in `xai-grok-secrets` / sanitizer paths
- External I/O failures that are handled but should be visible
- Feature fallbacks (e.g. sqlite-vec unavailable → FTS-only)
- Actor/state mutations that are hard to reconstruct from return values alone

## Comments

- Crate- and module-level `//!` docs explaining purpose, merge order, contracts (see `crates/codegen/xai-grok-config/src/lib.rs`)
- Non-obvious invariants, Windows/Unix differences, wire-format compatibility
- Regression rationale on tests (“API-key users past the 30-day client TTL saw 401…”)
- `// SAFETY:` immediately above every `unsafe` block explaining soundness (required style in this tree)
- Not applicable (Rust) — use rustdoc `///` on public items
- Document error conditions, field meanings for wire types, and platform `cfg` behavior
- Prefer linking with `` [`Type`] `` / `` [`fn`] `` rustdoc references
- Heavy test files use comment banners: `// ====` or `// ── Section ──`

## Function Design

- Prefer single-purpose functions; large match-driven handlers are OK when domain-driven
- Extract helpers for repeated setup (especially in tests: `test_config()`, `write_hook()`, harness structs)
- Prefer owned types at API boundaries when stored; borrow (`&str`, `&Path`) for read-only
- Pass config by reference when not consumed: `new(config: &WebSearchConfig, ...)`
- Use `Option` for optional hooks/providers rather than multiple constructors when modes are few
- Builder / `with_*` chains for optional fields on types that grow many knobs
- `Result` for fallible work; domain error type in libraries, `anyhow` at binary/orchestration edges
- Avoid returning `(bool, String)` when an enum communicates outcomes better (`HookDecision`, `EmptyReason`)
- Async: `async fn` with Tokio runtime; trait methods may use `async_trait` where needed

## Module Design

- Public API is deliberate: keep internals `pub(crate)` or private
- Re-export the stable surface at `lib.rs` with a short comment when selective (config campaigns note)
- Shared test-only modules: `#[cfg(test)] pub(crate) mod test_support` (shell)
- `mod.rs` for multi-file modules; `lib.rs` is the crate barrel
- Avoid deep re-export cascades that hide ownership — prefer one clear owning module
- `crates/codegen/` — CLI / TUI / agent product crates
- `crates/common/` — shared leaf libraries
- `crates/build/` — build-time helpers (`xai-proto-build`)
- `third_party/` — vendored upstream; do not apply first-party style “cleanups” casually
- `prod/` — production-adjacent types (e.g. proxy types)

## Serde & Wire Formats

- Derive `Serialize`/`Deserialize` with explicit `rename_all` when wire format is fixed:
- Tagged errors: `#[serde(tag = "code", rename_all = "snake_case")]`
- Optional fields: `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Keep display strings and serde renames aligned when both exist (pin shared constants if needed)

## Platform & Safety

- Gate OS-specific code with `#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_os = "macos")]`, `#[cfg(target_os = "linux")]`
- Mirror production `cfg` in tests (platform modules under `#[cfg(test)]`)
- Unix-only features (leader IPC, crash-handler signal tests) document the gate at the top of the test file
- Minimize; always pair with `// SAFETY: ...`
- Env mutation in tests: only under `#[serial_test::serial]` with RAII guards (`EnvGuard` in `xai-grok-test-support`)
- Use `dunce::canonicalize` for path equality / containment
- Prefer `Path`/`PathBuf`; display with `%path.display()` in logs

## Async & Concurrency

- Actors with channels (`mpsc`) and `CancellationToken` (`xai-chat-state`)
- Prefer structured shutdown (cancel token / drop handles) over leaking tasks
- Timeouts in tests with `tokio::time::timeout` and clear `.expect("timed out waiting for …")`
- Virtual time: `#[tokio::test(start_paused = true)]` when testing timers (requires `tokio` `test-util` in dev-deps)

## Project-Specific Conventions

- `cargo fmt --all`
- `cargo clippy -p <crate>` against root `clippy.toml`
- Prefer green unit/integration tests for the crate you touched
- Shared test infrastructure README must update in the same PR as `src/` changes (`crates/codegen/xai-grok-test-support/README.md`)
- Tool implementations may originate from codex/opencode ports — honor `THIRD_PARTY_NOTICES` / crate-local notices when editing

<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

## System Overview

```text

```

## Component Responsibilities

| Component | Responsibility | File / crate |
|-----------|----------------|--------------|
| Composition root binary | CLI parse, jemalloc, crash handler, mode routing, minimal-mode IoC install | `crates/codegen/xai-grok-pager-bin/src/main.rs` |
| Pager TUI | Scrollback, prompt, modals, slash cmds, Action/Effect event loop | `crates/codegen/xai-grok-pager/` |
| Pager render | Themes, terminal, syntax, glyphs, clipboard (presentation layer) | `crates/codegen/xai-grok-pager-render/` |
| Pager minimal | Scrollback-native alternate render mode (fn-pointer IoC) | `crates/codegen/xai-grok-pager-minimal/` |
| Shell runtime | Agent entry points, auth, leader, session, MCP/hooks glue | `crates/codegen/xai-grok-shell/` |
| MvpAgent | ACP agent surface: sessions, tools, sampling orchestration | `crates/codegen/xai-grok-shell/src/agent/mvp_agent/` |
| Agent definitions | Portable Agent type: tools, system prompt, compaction, plugins | `crates/codegen/xai-grok-agent/` |
| Chat state | Actor-owned conversation history, tokens, compaction modes | `crates/codegen/xai-chat-state/` |
| Sampler | HTTP streaming, retries, concurrent request actor | `crates/codegen/xai-grok-sampler/` |
| Tools | Tool registry + implementations (grok_build, codex, opencode ports) | `crates/codegen/xai-grok-tools/` |
| Workspace | Host FS, git/jj, permissions, hub server, worktree pool | `crates/codegen/xai-grok-workspace/` |
| Config | Layered TOML config merge, managed policy, paths | `crates/codegen/xai-grok-config/` |
| Auth | OAuth/device auth, credentials, middleware | `crates/codegen/xai-grok-auth/`, `shell/src/auth/` |
| MCP | MCP server client, OAuth, transports | `crates/codegen/xai-grok-mcp/` |
| Hooks | Hook discovery, matching, command/HTTP runners | `crates/codegen/xai-grok-hooks/` |
| ACP lib | Typed ACP channels, gateways, stdin line reader | `crates/codegen/xai-acp-lib/` |
| Telemetry | OTEL, Sentry, unified log, sampling/hooks layers | `crates/codegen/xai-grok-telemetry/` |
| Tool protocol | Computer Hub wire types (JSON-RPC frames, tool I/O) | `crates/common/xai-tool-protocol/` |
| Compaction | Shared conversation compaction engines | `crates/common/xai-grok-compaction/` |

## Pattern Overview

- **Composition-root binary** — `xai-grok-pager-bin` links pager + shell + optional minimal mode; avoids cargo cycles by wiring IoC hooks at startup (`xai_grok_pager_minimal::install()`).
- **Action → Dispatch → Effect → TaskResult** — TUI is a pure sync reducer (`dispatch`) plus async effect executor (`effects`) driven by a biased `tokio::select!` event loop.
- **Leader/follower IPC** — One leader process per environment owns agent state; TUI/IDE/headless clients speak ACP over a Unix domain socket (`~/.grok/leader.sock` keyed by WS URL).
- **Actor pattern** — Chat state, sampler, hunk tracker, and related subsystems use command channels + dedicated tasks (no shared locks on hot state).
- **ACP (Agent Client Protocol)** — Primary protocol between UI/IDE clients and the agent; also used for editor embedding.
- **Workspace abstraction** — Filesystem, VCS, permissions, and tool config live behind `xai-grok-workspace` so the agent can run local or hub-exposed.

## Layers

- Purpose: Terminal UI — input, scrollback, views, themes, slash commands
- Location: `crates/codegen/xai-grok-pager/src/`, render primitives in `crates/codegen/xai-grok-pager-render/`
- Contains: `app/` (event loop, dispatch, agent_view), `scrollback/`, `views/`, `slash/`, `input/`
- Depends on: shell (ACP client + config/auth helpers), pager-render, acp-lib, ratatui/crossterm
- Used by: `xai-grok-pager-bin` interactive path (`xai_grok_pager::app::run`)
- Purpose: Parse args, install process-wide hooks, route subcommands
- Location: `crates/codegen/xai-grok-pager-bin/src/main.rs`, CLI types in `crates/codegen/xai-grok-pager/src/app/cli.rs`
- Contains: `Command` enum (Agent, Login, Mcp, Sessions, Update, …), setup/workspace gates
- Depends on: pager, shell, update, telemetry, workspace, crash-handler
- Used by: end users (`grok` / `xai-grok-pager`)
- Purpose: Session lifecycle, tool calls, model turns, MCP, subagents, leader
- Location: `crates/codegen/xai-grok-shell/`
- Contains: `agent/`, `session/`, `leader/`, `auth/`, `sampling/`, `tools/`, `remote/`, `relay/`
- Depends on: tools, workspace, agent defs, chat-state, sampler, mcp, hooks, config, auth, telemetry
- Used by: pager TUI (via ACP), `grok agent *`, leader spawn, ACP stdio bridge
- Purpose: Portable agent definition, conversation state, model streaming
- Location: `xai-grok-agent`, `xai-chat-state`, `xai-grok-sampler`, `xai-grok-sampling-types`
- Contains: `AgentBuilder`, `ChatStateActor`, `SamplerActor`, streaming transforms
- Depends on: HTTP client types, compaction, token estimation
- Used by: shell session / MvpAgent
- Purpose: Implement model-callable tools (read, edit, bash, web, tasks, plan mode, …)
- Location: `crates/codegen/xai-grok-tools/src/implementations/`
- Contains: `grok_build/` (primary), `codex/`, `opencode/` (ported tool surfaces), registry, computer hub adapters
- Depends on: workspace, sandbox, file-utils, tool-protocol/runtime
- Used by: shell session tool dispatcher
- Purpose: Host capabilities for tools and sessions
- Location: `crates/codegen/xai-grok-workspace/`
- Contains: `file_system/`, `permission/`, `worktree/`, `hub*`, `session/`, `foreign_sessions/`
- Depends on: hunk-tracker, fsnotify, workspace-types/client, fast-worktree
- Used by: shell tools, workspace-server binary, leader workspace exposure
- Purpose: Config, telemetry, markdown, sandbox, paths, HTTP, secrets
- Location: `crates/codegen/*` (many small crates), `crates/common/*`, `crates/build/*`
- Depends on: minimal; keep leaf crates free of shell/pager
- Used by: higher layers via workspace.dependencies path deps

## Data Flow

### Primary TUI request path

### Agent turn path (shell session)

### Leader / multi-client path

### Headless / scripting path

- **TUI state:** Owned by `AppView` / `AgentView`; mutated only in sync dispatch
- **Session/conversation:** Owned by actors (`ChatStateActor`, session thread in shell)
- **File hunks:** `HunkTrackerActor` (`xai-hunk-tracker`)
- **Process-global:** Config caches, auth managers, fsnotify runtime handle, active-session registry
- **Persistence:** Session JSONL/storage under `$GROK_HOME` (default `~/.grok/`); managed config layers on disk

## Key Abstractions

- Purpose: Separate pure UI logic from async I/O
- Examples: `crates/codegen/xai-grok-pager/src/app/actions.rs`, `dispatch/`, `effects/`
- Pattern: Elm/TEA-style unidirectional data flow; dispatch must not touch network/FS/terminal
- Purpose: Typed duplex messaging between client UI and agent
- Examples: `crates/codegen/xai-acp-lib/src/`, used from pager event loop and shell agent
- Pattern: Channel + gateway wrappers over JSON-RPC ACP messages
- Purpose: Host implementation of the ACP agent for Grok Build
- Examples: `crates/codegen/xai-grok-shell/src/agent/mvp_agent/`
- Pattern: Spawns session threads; coordinates auth, models, tools, MCP
- Purpose: Bundle tools + system prompt + compaction + model config portably
- Examples: `crates/codegen/xai-grok-agent/src/{agent,builder,config,prompt}.rs`
- Pattern: Builder (`AgentBuilder`) from definitions/presets/plugins
- Purpose: Layered model I/O (raw stream → events → concurrent actor)
- Examples: `crates/codegen/xai-grok-sampler/src/{client,stream,handle,actor}.rs`
- Pattern: Three-layer API documented in crate root
- Purpose: Single-owner conversation mutations without locks
- Examples: `crates/codegen/xai-chat-state/src/{actor,handle,commands,events}.rs`
- Pattern: Command channel + optional oneshot query responses + event broadcast
- Purpose: Unified host operations (FS, git, permissions, sessions)
- Examples: `crates/codegen/xai-grok-workspace/src/handle.rs`, `session/`
- Pattern: Local connect or remote/hub exposure; ops as `WorkspaceOp`
- Purpose: Map tool names/schemas to async handlers
- Examples: `crates/codegen/xai-grok-tools/src/registry/`, `implementations/grok_build/`
- Pattern: Versioned tool folders; shared types/normalization/retry
- Purpose: Multi-client shared agent process
- Examples: `crates/codegen/xai-grok-shell/src/leader/{client,server,protocol,lock}.rs`
- Pattern: Socket path derived from WS URL; file lock prevents multi-leader races
- Purpose: Merge managed + user + requirements policy
- Examples: `crates/codegen/xai-grok-config/src/loader.rs`
- Pattern: Ordered deep-merge of TOML layers (system → home managed → user → signed requirements → MDM)

## Entry Points

- Location: `xai_grok_pager::app::run` via `pager-bin` `async_main`
- Triggers: Default CLI path after parsing
- Responsibilities: Full-screen agent UI, optional leader attach, session start/resume
- Location: `Command` match in `crates/codegen/xai-grok-pager-bin/src/main.rs` (~1597+)
- Triggers: `grok login`, `mcp`, `sessions`, `update`, `agent`, `workspace`, …
- Responsibilities: One-shot operations without TUI (or agent modes)
- Location: `xai_grok_shell::agent::app::run_stdio_agent`
- Triggers: IDE / desktop spawners, ACP bridges
- Responsibilities: ACP over stdin/stdout, skills watcher, LocalSet agent tasks
- Location: `run_headless` / `run_headless_no_browser`
- Triggers: Scripting/CI (`-p` / agent headless args)
- Responsibilities: Non-interactive turns, optional no-browser auth
- Location: `run_leader` / `leader::run_leader_server`
- Triggers: `connect_or_spawn` when no healthy leader exists
- Responsibilities: Shared agent + IPC + control plane (workspace start/stop, update relaunch)
- Location: `xai_grok_shell::agent::server::run_agent_server`
- Triggers: `grok agent serve` (bind + secret)
- Responsibilities: Networked agent endpoint for remote clients
- Location: `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` (`xai-workspace-server`)
- Triggers: Standalone workspace process / hub mode
- Responsibilities: Workspace ops server for remote or multi-process setups
- `ptyctl`, `code-graph`, `fast-worktree`, playground bins under pager/markdown — tooling and benches, not the product CLI

## Architectural Constraints

- **Threading:** Tokio multi-thread runtime at process root; agent ACP handlers often run on `LocalSet` / `spawn_local` (session work is thread-affine). `LocalRef<T>` encodes unsafe-but-scoped single-thread pointers in MvpAgent.
- **Global state:** Auth managers, config caches, fsnotify runtime handle (`register_fs_watch_runtime`), active sessions registry, PTY session registry, telemetry firehose. Prefer actor handles over new process-globals.
- **Circular imports:** Pager library must not depend on `xai-grok-pager-minimal` (minimal depends on pager); composition root installs hooks. Root `Cargo.toml` is **generated** — edit per-crate manifests only.
- **Dispatch purity:** `app/dispatch` must stay free of terminal/network/filesystem side effects; put I/O in `effects` or shell.
- **Workspace feature gates:** `grok workspace` is remote-settings gated (`workspace_command_enabled`) with `GROK_WORKSPACE_COMMAND` local override.
- **Edition / toolchain:** Workspace edition `2024`; pinned Rust in `rust-toolchain.toml` (1.92.0 at analysis date).
- **Profiles:** `release` for local; `release-dist` for shipping (thin LTO, single CGU); do not slow default release with dist settings.

## Anti-Patterns

### Mutating TUI state outside dispatch

### Adding application logic to the composition-root binary

### Coupling leaf crates to shell or pager

### Bypassing the tool registry for new tools

### Spawning a second leader casually

## Error Handling

- CLI/TUI: `anyhow` with user-facing `eprintln!` and process exit codes (`pager-bin` main, setup, workspace gates)
- Dispatch: map ACP/network failures into UI state + toasts (e.g. free-usage exhausted helpers in `dispatch/`)
- Sampler: classify errors → retry decisions (`xai-grok-sampler/src/retry.rs`)
- Workspace: `WorkspaceResult` / `WorkspaceError` (`xai-grok-workspace/src/error.rs`)
- Tool protocol: JSON-RPC error codes and `ToolErrorWire` (`xai-tool-protocol`)
- Prefer typed variants over string matching for recoverable UI paths (e.g. `SwitchModelError`)

## Cross-Cutting Concerns

- `tracing` + `xai-grok-telemetry` layers (unified log, sampling log, hooks log, OTEL, Sentry)
- Headless defaults quieter (`init_tracing_simple`); debug via `GROK_DEBUG_LOG` / env filters
- Prefer structured fields; use `unified_log` for diagnostic sessions
- Clap for CLI
- Config layer validation + signed requirements fail-closed (`xai_grok_config::validate_requirements`)
- Tool argument schemas / normalization in tools crate
- Folder trust gates for workspace operations (`agent/folder_trust`, workspace `trust`)
- OAuth2 / device-code via shell auth + `xai-grok-auth`
- `AuthManager` with proactive refresh and system-power sleep pause
- Bearer injection into sampler/HTTP clients; credentials never logged (secrets sanitizer crate)
- `xai-grok-sandbox` feature-forwarded as `sandbox-enforce` from binary/pager
- Tool execution paths must honor sandbox policy when enabled
- Lifecycle hooks: `xai-grok-hooks`
- Plugin marketplace: `xai-grok-plugin-marketplace`
- Agent plugins surface: `xai-grok-agent/src/plugins/`
- `xai-grok-markdown` (+ core) for TUI rendering; Mermaid via `xai-grok-mermaid` / vendored `third_party/mermaid-to-svg`
- Off-thread mermaid worker in pager app (`mermaid_worker`)

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
