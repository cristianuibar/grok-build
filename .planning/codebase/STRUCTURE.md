# Codebase Structure

**Analysis Date:** 2026-07-16

## Directory Layout

```
grok-build/
├── Cargo.toml                 # Generated workspace root (read-only; edit per-crate)
├── Cargo.lock
├── rust-toolchain.toml        # Pinned Rust (rustfmt + clippy)
├── rustfmt.toml
├── clippy.toml
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├── SECURITY.md
├── THIRD-PARTY-NOTICES
├── bin/
│   └── protoc                 # protoc launcher (dotslash) for proto codegen
├── crates/
│   ├── build/                 # Build-time helpers (e.g. proto)
│   ├── codegen/               # Product crates (pager, shell, tools, …)
│   └── common/                # Shared leaf crates (tool protocol, compaction, …)
├── prod/
│   └── mc/cli-chat-proxy-types/  # Shared API types pulled by the closure
├── third_party/               # Vendored source (Mermaid stack, etc.)
└── .planning/
    └── codebase/              # GSD architecture maps (this tree)
```

### `crates/codegen/` (primary product surface)

```
crates/codegen/
├── xai-grok-pager-bin/        # Composition-root binary → artifact xai-grok-pager
├── xai-grok-pager/            # TUI application library
├── xai-grok-pager-render/     # Presentation primitives (theme, terminal, syntax)
├── xai-grok-pager-minimal/    # Alternate scrollback-native UI mode
├── xai-grok-pager-pty-harness/# PTY integration tests / benches
├── xai-grok-shell/            # Agent runtime + leader + session
├── xai-grok-shell-base/       # Small shell-shared primitives (env, cpu_profile)
├── xai-grok-shell-session-support/
├── xai-grok-tools/            # Tool implementations
├── xai-grok-tools-api/        # Proto API for tools
├── xai-grok-workspace/        # Host FS/VCS/permissions/hub
├── xai-grok-workspace-client/
├── xai-grok-workspace-types/
├── xai-grok-agent/            # Agent definition + prompt assembly
├── xai-chat-state/            # Conversation actor
├── xai-grok-sampler/          # Model streaming actor
├── xai-grok-sampling-types/
├── xai-acp-lib/               # ACP channel utilities
├── xai-grok-config/           # Config load/merge
├── xai-grok-config-types/
├── xai-grok-auth/
├── xai-grok-mcp/
├── xai-grok-hooks/
├── xai-grok-memory/
├── xai-grok-telemetry/
├── xai-grok-markdown/
├── xai-grok-markdown-core/
├── xai-grok-mermaid/
├── xai-grok-sandbox/
├── xai-grok-update/
├── xai-grok-version/
├── xai-hunk-tracker/
├── xai-codebase-graph/
├── xai-fast-worktree/
├── xai-fsnotify/
├── xai-file-utils/
├── …                          # Additional support crates (paths, secrets, voice, …)
```

### `crates/common/`

```
crates/common/
├── xai-tool-protocol/         # Computer Hub wire protocol types
├── xai-tool-runtime/
├── xai-tool-types/
├── xai-grok-compaction/       # Shared compaction engines
├── xai-computer-hub-core/
├── xai-computer-hub-sdk/
├── xai-computer-hub-mcp-adapter/
├── xai-circuit-breaker/
├── xai-interjection-core/
├── xai-test-utils/
└── xai-tracing/
```

## Directory Purposes

**`crates/codegen/xai-grok-pager-bin/`:**
- Purpose: Ship the user-facing binary; wire optional minimal mode; process-level setup
- Contains: `src/main.rs` only (large routing file)
- Key files: `src/main.rs`, `Cargo.toml` (features: `jemalloc`, `sandbox-enforce`, `release-dist`)

**`crates/codegen/xai-grok-pager/`:**
- Purpose: Full TUI product logic
- Contains: `src/app/` (core loop), views, scrollback, slash commands, CLI defs, docs
- Key files:
  - `src/lib.rs` — module tree
  - `src/app/mod.rs` — `run()`, terminal lifecycle
  - `src/app/cli.rs` — `Command` / `PagerArgs`
  - `src/app/actions.rs` — Action/Effect/TaskResult
  - `src/app/dispatch/` — pure reducers
  - `src/app/effects/` — async effect executor
  - `src/app/event_loop.rs` — tokio select loop
  - `docs/user-guide/` — shipped user documentation

**`crates/codegen/xai-grok-pager-render/`:**
- Purpose: Extracted presentation layer re-exported by pager (`theme`, `render`, `terminal`, …)
- Contains: assets (tmTheme), large `src/` render tree
- Key files: re-exports consumed as `crate::theme` etc. from pager via `pub use xai_grok_pager_render::…`

**`crates/codegen/xai-grok-shell/`:**
- Purpose: Agent runtime library (no primary product binary; used by pager-bin and tests)
- Contains: agent, session, leader, auth, sampling glue, remote/relay
- Key files:
  - `src/lib.rs` — public modules
  - `src/agent/app.rs` — `run_stdio_agent`, `run_headless`, `run_leader`
  - `src/agent/mvp_agent/` — ACP agent host
  - `src/session/` — session actor, ACP session, persistence, goals
  - `src/leader/` — multi-client IPC
  - `src/auth/` — credential lifecycle

**`crates/codegen/xai-grok-tools/`:**
- Purpose: All model-facing tool implementations
- Contains: `implementations/grok_build/` (canonical), `codex/`, `opencode/`, registry, computer adapters
- Key files: `src/lib.rs`, `src/registry/`, `src/implementations/mod.rs`, `THIRD_PARTY_NOTICES.md`

**`crates/codegen/xai-grok-workspace/`:**
- Purpose: Host capabilities for tools and multi-process workspace exposure
- Contains: file_system, permission, worktree, hub server, foreign session importers
- Key files: `src/lib.rs`, `src/handle.rs`, `src/bin/workspace_server.rs`

**`crates/codegen/xai-grok-agent/`:**
- Purpose: First-class Agent type (prompt, tools, plugins, compaction policy)
- Contains: builder, discovery, prompt templates under `templates/`
- Key files: `src/lib.rs`, `src/builder.rs`, `src/agent.rs`, `templates/prompt.md`

**`crates/codegen/xai-chat-state/` + `xai-grok-sampler/`:**
- Purpose: Extracted actors for conversation and model streaming
- Contains: actor/handle/commands/events pattern (same family as hunk-tracker)
- Key files: each crate’s `src/lib.rs` documents the layer diagram

**`crates/common/`:**
- Purpose: Protocol and runtime pieces shared outside the monorepo sync boundary
- Contains: tool wire protocol, computer hub SDK, compaction
- Key files: `xai-tool-protocol/src/lib.rs` (method catalog + frames)

**`crates/build/`:**
- Purpose: Build scripts only (`xai-proto-build` finds protoc)
- Contains: `find_protoc.rs`, lib used by tools-api build.rs

**`third_party/`:**
- Purpose: Vendored upstream source (not crates.io deps)
- Contains: `mermaid-to-svg`, `dagre_rust`, `graphlib_rust`, `ordered_hashmap`
- Key files: `third_party/README.md`, `NOTICE`

**`prod/mc/`:**
- Purpose: Minimal production-shared types needed by the CLI closure
- Contains: `cli-chat-proxy-types`

## Key File Locations

**Entry Points:**
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — process entry (`fn main` / `async_main`)
- `crates/codegen/xai-grok-pager/src/app/mod.rs` — TUI `run()`
- `crates/codegen/xai-grok-shell/src/agent/app.rs` — agent mode runners
- `crates/codegen/xai-grok-shell/src/agent/server.rs` — networked agent server
- `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` — workspace server binary

**Configuration:**
- Root: `Cargo.toml` (workspace members + dependency versions — **generated**), `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`
- Runtime user config: loaded via `xai-grok-config` → `$GROK_HOME/config.toml` (default `~/.grok/`)
- Crate manifests: each `crates/**/Cargo.toml`
- Pager CLI flags: `crates/codegen/xai-grok-pager/src/app/cli.rs`

**Core Logic:**
- TUI dispatch: `crates/codegen/xai-grok-pager/src/app/dispatch/`
- TUI effects: `crates/codegen/xai-grok-pager/src/app/effects/`
- Session/ACP: `crates/codegen/xai-grok-shell/src/session/`
- Tools: `crates/codegen/xai-grok-tools/src/implementations/grok_build/`
- Leader: `crates/codegen/xai-grok-shell/src/leader/`
- Workspace ops: `crates/codegen/xai-grok-workspace/src/`

**Testing:**
- Co-located unit tests in many `src/**` modules
- Crate-level `tests/` integration dirs (e.g. `xai-grok-pager/tests/`, hooks, tools, workspace, sampler)
- Pager PTY harness: `crates/codegen/xai-grok-pager-pty-harness/`
- Shared test helpers: `crates/codegen/xai-grok-test-support/`, `crates/common/xai-test-utils/`
- Snapshots: `insta` `.snap` files under pager scrollback blocks, etc.

**Documentation:**
- Root `README.md` — layout + build
- `crates/codegen/xai-grok-pager/docs/user-guide/` — end-user guide
- Crate READMEs where present (pager, agent, crash-handler, test-support)

## Naming Conventions

**Crates:**
- Product crates: `xai-grok-<domain>` (e.g. `xai-grok-pager`, `xai-grok-shell`)
- Shared non-product: `xai-<domain>` (e.g. `xai-chat-state`, `xai-acp-lib`, `xai-hunk-tracker`)
- Common/protocol: under `crates/common/xai-*`
- Path on disk matches package name directory

**Files (Rust):**
- Modules: `snake_case.rs` (`event_loop.rs`, `folder_trust.rs`)
- Large feature areas: directory module `name/mod.rs` with submodules
- Integration tests: `tests/*.rs` or nested `*_tests` modules/dirs in shell session
- Binaries: `src/bin/<name>.rs` or package `[[bin]]` path
- Proto: `proto/*.proto` next to build.rs when needed (`xai-grok-tools-api`)

**Types / items:**
- Types/traits/enums: `PascalCase` (`Action`, `MvpAgent`, `WorkspaceHandle`)
- Functions/methods: `snake_case` (`run_headless`, `connect_or_spawn`)
- Constants: `SCREAMING_SNAKE_CASE` (`DEFAULT_TOOL_OUTPUT_BYTES`, `HEADLESS_ENTRYPOINT`)
- Env vars: `GROK_*` prefix (`GROK_HOME`, `GROK_WORKSPACE_COMMAND`, `GROK_DEBUG_LOG`)

**Directories:**
- Tool families: `implementations/grok_build/<tool_name>/` with optional `versions/`
- Dispatch domains: `app/dispatch/<domain>.rs` or nested folders (`session/`, `settings/`)
- Tests: `tests/` at crate root for integration; `src/**/tests.rs` or `#[cfg(test)] mod tests` for unit

## Where to Add New Code

**New TUI feature (views, keybindings, slash command):**
- Primary code: `crates/codegen/xai-grok-pager/src/`
  - State transitions: `src/app/dispatch/`
  - Async work: `src/app/effects/`
  - New Action variants: `src/app/actions.rs`
  - Slash command: `src/slash/commands/`
  - UI chrome: `src/views/` or `src/scrollback/`
- CLI surface (if user-facing flag/subcommand): `src/app/cli.rs` + match arm in `xai-grok-pager-bin/src/main.rs` only if routing needed
- Tests: unit tests next to dispatch; PTY/integration in `xai-grok-pager/tests/` or pty-harness

**New agent/runtime behavior (session, auth, leader, MCP glue):**
- Implementation: `crates/codegen/xai-grok-shell/src/`
  - Session turn logic: `session/` / `session/acp_session_impl/`
  - Entry modes: `agent/app.rs`
  - Leader protocol: `leader/`
- Prefer extracting reusable actors into sibling crates (`xai-chat-state`, `xai-grok-sampler`) when logic is host-agnostic

**New model tool:**
- Implementation: `crates/codegen/xai-grok-tools/src/implementations/grok_build/<tool_name>/`
- Register in tools registry (`src/registry/`) and any tool taxonomy/version tables
- Wire permissions/sandbox via workspace APIs — do not call raw FS from shell session
- Tests: unit under the tool module; integration under tools crate `tests/` when needed

**New agent definition / prompt / plugin surface:**
- `crates/codegen/xai-grok-agent/src/` (+ `templates/` for prompt markdown)

**New workspace/host capability:**
- `crates/codegen/xai-grok-workspace/src/` (new module + re-export from `lib.rs` if public)
- Types shared across processes: `xai-grok-workspace-types`
- Client bindings: `xai-grok-workspace-client`

**New shared protocol / hub wire type:**
- `crates/common/xai-tool-protocol/` (or hub SDK under `xai-computer-hub-*`)

**New config field:**
- Types: `xai-grok-config-types` if pure data
- Load/merge/validation: `xai-grok-config`
- Consumers: shell `agent/config` or pager settings as appropriate

**Utilities:**
- Cross-crate helpers: prefer an existing leaf (`xai-file-utils`, `xai-grok-shared`, `xai-tty-utils`) over dumping into shell/pager
- Telemetry helpers: `xai-grok-telemetry`
- Test-only helpers: `xai-grok-test-support` or `xai-test-utils`

**Do not:**
- Edit root `Cargo.toml` by hand (generated) — add path deps in the crate’s own `Cargo.toml` and regenerate workspace listing via monorepo process
- Add product logic only in `pager-bin` (routing/wiring only)
- Create upward deps from common/leaf → shell/pager

## Special Directories

**`crates/codegen/xai-grok-pager/docs/`:**
- Purpose: User guide and feature docs shipped/extracted at runtime
- Generated: Partial extract to `$GROK_HOME` on launch (`docs::extract_user_guide_docs`)
- Committed: Yes

**`crates/codegen/xai-grok-pager/npm/`:**
- Purpose: npm packaging helpers for the CLI distribution
- Generated: No
- Committed: Yes

**`crates/codegen/xai-grok-agent/templates/`:**
- Purpose: System / subagent / apply_patch prompt templates (may be encrypted via scripts)
- Generated: Encryption scripts under `scripts/`
- Committed: Yes (templates + script)

**`third_party/`:**
- Purpose: Vendored Mermaid/layout stack
- Generated: No
- Committed: Yes — treat as upstream; see `NOTICE`

**`bin/protoc`:**
- Purpose: Proto compiler launcher for build scripts
- Generated: No (dotslash wrapper)
- Committed: Yes

**`.planning/`:**
- Purpose: GSD planning artifacts (codebase maps, phase plans)
- Generated: By GSD commands
- Committed: Typically gitignored / local — do not leak secrets

**`target/` (cargo output):**
- Purpose: Build artifacts
- Generated: Yes
- Committed: No

## Build & Navigation Tips

- Prefer crate-scoped cargo: `cargo check -p xai-grok-pager`, `cargo test -p xai-grok-config` (full workspace is slow)
- Run the product: `cargo run -p xai-grok-pager-bin`
- Binary name in `target/`: `xai-grok-pager` (installs rename to `grok`)
- Dependency versions live in workspace `[workspace.dependencies]`; crates should use `{ workspace = true }` where available
- Large crates (pager ~380k LOC src, shell ~317k, tools ~111k) — open the specific submodule path above rather than searching from crate root blindly

## Crate Dependency Guidance (where code sits)

```text
xai-grok-pager-bin
  ├── xai-grok-pager ──► xai-grok-pager-render
  │         │
  │         └── xai-grok-shell ──► xai-grok-tools
  │                    │           xai-grok-workspace
  │                    │           xai-grok-agent
  │                    │           xai-chat-state
  │                    │           xai-grok-sampler
  │                    │           xai-grok-mcp / hooks / config / auth / telemetry
  │                    └── xai-acp-lib
  └── xai-grok-pager-minimal (optional IoC; depends on pager, not reverse)
```

Use this arrow direction when placing new modules: **UI → runtime → domain actors/tools → host/workspace → common/leaf**.

---

*Structure analysis: 2026-07-16*
