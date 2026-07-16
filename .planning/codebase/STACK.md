# Technology Stack

**Analysis Date:** 2026-07-16

## Languages

**Primary:**
- Rust (edition **2024**, toolchain **1.92.0** pinned in `rust-toolchain.toml`) — entire product: TUI, agent runtime, tools, auth, telemetry, MCP, sandbox
  - Components: `rustfmt`, `clippy`
  - Host targets declared: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (macOS / Windows via rustup host + `.cargo/config.toml` rustflags)

**Secondary:**
- Protocol Buffers — tool wire API in `crates/codegen/xai-grok-tools-api/proto/grok-tools.proto` (codegen via `tonic` / `prost` / `xai-proto-build`)
- TOML / JSON / YAML — config (`~/.grok/config.toml`), models (`default_models.json`), marketplace indexes, session persistence
- Node.js (≥20, optional packaging only) — npm wrapper package `@xai-official/grok` under `crates/codegen/xai-grok-pager/npm/` (postinstall binary selector)
- Shell / PowerShell — install scripts referenced by auto-update (`install.sh` / `install.ps1` at `https://x.ai/cli`)
- Python — minor helper scripts (e.g. `crates/codegen/xai-grok-agent/scripts/encrypt_templates.py`, hook examples)

## Runtime

**Environment:**
- Native Rust binary (default artifact name `xai-grok-pager`; shipped/installed as `grok`)
- Async runtime: **Tokio 1** (`features = ["full"]` at workspace level)
- Allocator (Unix, default feature): **jemalloc** via `tikv-jemallocator` / `tikv-jemalloc-ctl` / `tikv-jemalloc-sys` in `crates/codegen/xai-grok-pager-bin`
- TLS: **rustls** (reqwest `rustls-tls`; binary also depends on `rustls` 0.23 with `ring`)

**Package Manager:**
- **Cargo** (workspace, `resolver = "2"`)
- Lockfile: `Cargo.lock` present (committed)
- Root `Cargo.toml` is **generated** — treat as read-only; edit per-crate manifests
- Optional distribution: npm (`@xai-official/grok` + platform packages)

## Frameworks

**Core:**
- **ratatui 0.29** + **crossterm 0.28** — fullscreen TUI (`xai-grok-pager`, `xai-ratatui-inline`, `xai-ratatui-textarea`)
- **tokio** — multi-thread async agent loop, tools, HTTP, websockets
- **clap 4** — CLI parsing
- **agent-client-protocol (ACP) 0.10.4** — editor embedding protocol (`xai-acp-lib`, shell ACP session)
- **async-openai 0.33** — OpenAI-compatible Responses / Chat Completions client surface in sampler
- **rmcp 2.1** — Model Context Protocol client (isolated in `xai-grok-mcp` with **reqwest 0.13**)
- **tonic / prost 0.14** — gRPC/protobuf for tools API and OTLP internals
- **axum 0.8** — local loopback servers (OAuth callbacks, test fixtures, device-auth helpers)

**Testing:**
- Built-in `cargo test` (per-crate; workspace-wide builds are slow)
- **insta** — snapshot tests (pager)
- **pretty_assertions**, **assert_matches**, **serial_test**
- **mockito**, **wiremock**, **axum** test servers — HTTP/SSE fixtures
- **criterion** — benchmarks (markdown, pager, fsnotify, PTY harness)
- Crate-local integration tests under many `crates/codegen/*/tests/`

**Build/Dev:**
- **rustc / cargo** via `rust-toolchain.toml`
- **protoc** — `bin/protoc` (dotslash launcher) or `$PROTOC` / PATH (`crates/build/xai-proto-build`)
- **rustfmt** (`rustfmt.toml`: `use_field_init_shorthand = true`)
- **clippy** (`clippy.toml` — bans raw `canonicalize` in favor of `dunce`)
- Profiles: `dev`, `release`, `release-dist` (thin LTO, CGU=1), `release-dist-jemalloc`, `x-prod`, `bench` in root `Cargo.toml`
- Target rustflags / jemalloc page sizes: `.cargo/config.toml`

## Key Dependencies

**Critical:**
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

**Infrastructure:**
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

**Environment:**
- Backend endpoints resolved in `crates/codegen/xai-grok-env` (production defaults + `GROK_PRODUCTION_*` overrides)
- User config: TOML under `$GROK_HOME` (default `~/.grok`) via `xai-grok-config` — layers: requirements > user > managed/MDM
- Auth credentials: `~/.grok/auth.json` (or `GROK_AUTH` / `GROK_AUTH_PATH`); API key `XAI_API_KEY`
- Telemetry tokens/DSN may be compile-time (`option_env!`) or runtime (`SENTRY_DSN`, Mixpanel, events URL)
- Version override: `GROK_VERSION` (build), `GROK_TEST_VERSION` (tests)

**Build:**
- `Cargo.toml` (workspace root, generated)
- Per-crate `Cargo.toml` under `crates/codegen/`, `crates/common/`, `crates/build/`, `prod/mc/`, `third_party/`
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.cargo/config.toml`
- Binary features on `xai-grok-pager-bin`: `jemalloc` (default), `sandbox-enforce` (default), `release-dist`

## Platform Requirements

**Development:**
- Rust 1.92.0 via rustup (auto from `rust-toolchain.toml`)
- `protoc` (repo `bin/protoc` + [dotslash](https://dotslash-cli.com), or system `$PROTOC`)
- macOS and Linux supported build hosts; Windows best-effort / not fully tested from this tree
- Target specific crates for check/test (`cargo check -p <crate>`) — full workspace is heavy

**Production:**
- Prebuilt binaries for macOS, Linux, Windows (x64/arm64) via:
  - Install scripts: `https://x.ai/cli/install.sh` / `install.ps1`
  - npm: `@xai-official/grok`
  - GitHub Releases (`xai-org-shared/grok-build`, installer `gh-release`)
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

```bash
cargo run -p xai-grok-pager-bin              # build + launch TUI
cargo build -p xai-grok-pager-bin --release  # target/release/xai-grok-pager
cargo check -p <crate>
cargo test -p <crate>
cargo clippy -p <crate>
cargo fmt --all
cargo build -p xai-grok-pager-bin --profile release-dist
```

---

*Stack analysis: 2026-07-16*
