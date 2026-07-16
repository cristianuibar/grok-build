<!-- refreshed: 2026-07-16 -->
# Architecture

**Analysis Date:** 2026-07-16

## System Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│  Composition root: `xai-grok-pager-bin` (`crates/codegen/xai-grok-pager-bin`)│
│  Binary artifact: `xai-grok-pager` (shipped as `grok`)                       │
│  Routes CLI subcommands + launches TUI / agent / leader / ACP modes          │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
┌──────────────────┐  ┌────────────────────┐  ┌────────────────────┐
│ TUI (Pager)      │  │ Agent runtimes     │  │ Leader process     │
│ `xai-grok-pager` │  │ stdio / headless / │  │ Unix-socket IPC    │
│ Action→Dispatch  │  │ serve (shell)      │  │ `shell::leader`    │
│ →Effect loop     │  │ `shell::agent`     │  │ Multi-client hub   │
└────────┬─────────┘  └─────────┬──────────┘  └─────────┬──────────┘
         │  ACP (JSON-RPC)      │                        │
         └──────────────────────┼────────────────────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Agent runtime (`xai-grok-shell`)                                            │
│  MvpAgent · ACP session · tools bridge · auth · MCP · hooks · subagents      │
├──────────────────┬──────────────────┬──────────────────┬────────────────────┤
│ xai-grok-agent   │ xai-chat-state   │ xai-grok-sampler │ xai-grok-tools     │
│ prompt/tools     │ conversation     │ model streaming  │ tool impls         │
│ definition       │ actor            │ + retry actor    │ (bash, edit, …)    │
└────────┬─────────┴────────┬─────────┴────────┬─────────┴─────────┬──────────┘
         │                  │                  │                   │
         ▼                  ▼                  ▼                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Workspace (`xai-grok-workspace`)                                            │
│  FS · VCS · permissions · hunk tracking · worktrees · hub exposure           │
└─────────────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  External: xAI APIs · OAuth · MCP servers · Computer Hub · OS/TTY            │
└─────────────────────────────────────────────────────────────────────────────┘
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

**Overall:** Layered multi-crate workspace with a thin composition-root binary, an Elm-style TUI loop, and actor-based agent/session cores.

**Key Characteristics:**
- **Composition-root binary** — `xai-grok-pager-bin` links pager + shell + optional minimal mode; avoids cargo cycles by wiring IoC hooks at startup (`xai_grok_pager_minimal::install()`).
- **Action → Dispatch → Effect → TaskResult** — TUI is a pure sync reducer (`dispatch`) plus async effect executor (`effects`) driven by a biased `tokio::select!` event loop.
- **Leader/follower IPC** — One leader process per environment owns agent state; TUI/IDE/headless clients speak ACP over a Unix domain socket (`~/.grok/leader.sock` keyed by WS URL).
- **Actor pattern** — Chat state, sampler, hunk tracker, and related subsystems use command channels + dedicated tasks (no shared locks on hot state).
- **ACP (Agent Client Protocol)** — Primary protocol between UI/IDE clients and the agent; also used for editor embedding.
- **Workspace abstraction** — Filesystem, VCS, permissions, and tool config live behind `xai-grok-workspace` so the agent can run local or hub-exposed.

## Layers

**Presentation (TUI):**
- Purpose: Terminal UI — input, scrollback, views, themes, slash commands
- Location: `crates/codegen/xai-grok-pager/src/`, render primitives in `crates/codegen/xai-grok-pager-render/`
- Contains: `app/` (event loop, dispatch, agent_view), `scrollback/`, `views/`, `slash/`, `input/`
- Depends on: shell (ACP client + config/auth helpers), pager-render, acp-lib, ratatui/crossterm
- Used by: `xai-grok-pager-bin` interactive path (`xai_grok_pager::app::run`)

**Composition / CLI:**
- Purpose: Parse args, install process-wide hooks, route subcommands
- Location: `crates/codegen/xai-grok-pager-bin/src/main.rs`, CLI types in `crates/codegen/xai-grok-pager/src/app/cli.rs`
- Contains: `Command` enum (Agent, Login, Mcp, Sessions, Update, …), setup/workspace gates
- Depends on: pager, shell, update, telemetry, workspace, crash-handler
- Used by: end users (`grok` / `xai-grok-pager`)

**Agent runtime:**
- Purpose: Session lifecycle, tool calls, model turns, MCP, subagents, leader
- Location: `crates/codegen/xai-grok-shell/`
- Contains: `agent/`, `session/`, `leader/`, `auth/`, `sampling/`, `tools/`, `remote/`, `relay/`
- Depends on: tools, workspace, agent defs, chat-state, sampler, mcp, hooks, config, auth, telemetry
- Used by: pager TUI (via ACP), `grok agent *`, leader spawn, ACP stdio bridge

**Agent core libraries:**
- Purpose: Portable agent definition, conversation state, model streaming
- Location: `xai-grok-agent`, `xai-chat-state`, `xai-grok-sampler`, `xai-grok-sampling-types`
- Contains: `AgentBuilder`, `ChatStateActor`, `SamplerActor`, streaming transforms
- Depends on: HTTP client types, compaction, token estimation
- Used by: shell session / MvpAgent

**Tools layer:**
- Purpose: Implement model-callable tools (read, edit, bash, web, tasks, plan mode, …)
- Location: `crates/codegen/xai-grok-tools/src/implementations/`
- Contains: `grok_build/` (primary), `codex/`, `opencode/` (ported tool surfaces), registry, computer hub adapters
- Depends on: workspace, sandbox, file-utils, tool-protocol/runtime
- Used by: shell session tool dispatcher

**Workspace / host:**
- Purpose: Host capabilities for tools and sessions
- Location: `crates/codegen/xai-grok-workspace/`
- Contains: `file_system/`, `permission/`, `worktree/`, `hub*`, `session/`, `foreign_sessions/`
- Depends on: hunk-tracker, fsnotify, workspace-types/client, fast-worktree
- Used by: shell tools, workspace-server binary, leader workspace exposure

**Shared / leaf crates:**
- Purpose: Config, telemetry, markdown, sandbox, paths, HTTP, secrets
- Location: `crates/codegen/*` (many small crates), `crates/common/*`, `crates/build/*`
- Depends on: minimal; keep leaf crates free of shell/pager
- Used by: higher layers via workspace.dependencies path deps

## Data Flow

### Primary TUI request path

1. Process start — install minimal hooks, crash handler, jemalloc, validate requirements (`crates/codegen/xai-grok-pager-bin/src/main.rs` ~1456–1528)
2. CLI parse — `PagerArgs::parse_and_apply_cwd()`; if `args.command` is set, run subcommand and exit (`async_main` ~1530+)
3. Interactive path — `xai_grok_pager::app::run` loads config, refreshes auth, prefetches remote settings, resolves leader mode (`crates/codegen/xai-grok-pager/src/app/mod.rs` ~384+)
4. Terminal + event loop — enter alternate screen; `event_loop` selects on keyboard/mouse, ACP messages, task completions, config watchers (`crates/codegen/xai-grok-pager/src/app/event_loop.rs`)
5. Input → Action — key/mouse handlers produce `Action` (`crates/codegen/xai-grok-pager/src/app/actions.rs`)
6. Dispatch — pure sync `dispatch(action)` mutates `AppView` / `AgentView` and returns `Vec<Effect>` (`crates/codegen/xai-grok-pager/src/app/dispatch/`)
7. Effects — `effects::execute` spawns async tasks (ACP `session/prompt`, clipboard, session list, …) (`crates/codegen/xai-grok-pager/src/app/effects/`)
8. TaskResult → Dispatch — completed tasks re-enter dispatch; UI re-renders via ratatui

### Agent turn path (shell session)

1. ACP client sends `session/prompt` (TUI, IDE, or headless) into MvpAgent / session actor (`crates/codegen/xai-grok-shell/src/agent/mvp_agent/`, `session/acp_session*`)
2. Chat state records user message; request builder assembles sampling request (`xai-chat-state`)
3. Sampler streams model tokens/events with retry (`xai-grok-sampler`); updates flow as ACP notifications to clients
4. Model tool calls dispatch into `xai-grok-tools` registry → implementations against workspace FS/sandbox
5. Tool results return into conversation; turn continues until stop; persistence writes session storage under `~/.grok/`

### Leader / multi-client path

1. Client calls `connect_or_spawn` (`crates/codegen/xai-grok-shell/src/leader/`)
2. Leader process runs agent + Unix socket IPC server; namespaces request IDs; tracks session ownership
3. TUI or `grok agent stdio` bridges stdio ↔ leader ACP; reconnect can replay `initialize` / `session/load` (`pager-bin` `StdioReplayState`)

### Headless / scripting path

1. `grok agent` / headless flags → `run_headless` / `run_stdio_agent` / `run_leader` / `run_agent_server` (`crates/codegen/xai-grok-shell/src/agent/app.rs`, `server.rs`)
2. Same agent core; no ratatui event loop; stdout/stderr or WebSocket for I/O

**State Management:**
- **TUI state:** Owned by `AppView` / `AgentView`; mutated only in sync dispatch
- **Session/conversation:** Owned by actors (`ChatStateActor`, session thread in shell)
- **File hunks:** `HunkTrackerActor` (`xai-hunk-tracker`)
- **Process-global:** Config caches, auth managers, fsnotify runtime handle, active-session registry
- **Persistence:** Session JSONL/storage under `$GROK_HOME` (default `~/.grok/`); managed config layers on disk

## Key Abstractions

**Action / Effect / TaskResult:**
- Purpose: Separate pure UI logic from async I/O
- Examples: `crates/codegen/xai-grok-pager/src/app/actions.rs`, `dispatch/`, `effects/`
- Pattern: Elm/TEA-style unidirectional data flow; dispatch must not touch network/FS/terminal

**ACP channels:**
- Purpose: Typed duplex messaging between client UI and agent
- Examples: `crates/codegen/xai-acp-lib/src/`, used from pager event loop and shell agent
- Pattern: Channel + gateway wrappers over JSON-RPC ACP messages

**MvpAgent:**
- Purpose: Host implementation of the ACP agent for Grok Build
- Examples: `crates/codegen/xai-grok-shell/src/agent/mvp_agent/`
- Pattern: Spawns session threads; coordinates auth, models, tools, MCP

**Agent (definition object):**
- Purpose: Bundle tools + system prompt + compaction + model config portably
- Examples: `crates/codegen/xai-grok-agent/src/{agent,builder,config,prompt}.rs`
- Pattern: Builder (`AgentBuilder`) from definitions/presets/plugins

**SamplerHandle / SamplingClient:**
- Purpose: Layered model I/O (raw stream → events → concurrent actor)
- Examples: `crates/codegen/xai-grok-sampler/src/{client,stream,handle,actor}.rs`
- Pattern: Three-layer API documented in crate root

**ChatStateHandle / ChatStateActor:**
- Purpose: Single-owner conversation mutations without locks
- Examples: `crates/codegen/xai-chat-state/src/{actor,handle,commands,events}.rs`
- Pattern: Command channel + optional oneshot query responses + event broadcast

**WorkspaceHandle:**
- Purpose: Unified host operations (FS, git, permissions, sessions)
- Examples: `crates/codegen/xai-grok-workspace/src/handle.rs`, `session/`
- Pattern: Local connect or remote/hub exposure; ops as `WorkspaceOp`

**Tool registry + implementations:**
- Purpose: Map tool names/schemas to async handlers
- Examples: `crates/codegen/xai-grok-tools/src/registry/`, `implementations/grok_build/`
- Pattern: Versioned tool folders; shared types/normalization/retry

**LeaderClient / LeaderServer:**
- Purpose: Multi-client shared agent process
- Examples: `crates/codegen/xai-grok-shell/src/leader/{client,server,protocol,lock}.rs`
- Pattern: Socket path derived from WS URL; file lock prevents multi-leader races

**Config layers:**
- Purpose: Merge managed + user + requirements policy
- Examples: `crates/codegen/xai-grok-config/src/loader.rs`
- Pattern: Ordered deep-merge of TOML layers (system → home managed → user → signed requirements → MDM)

## Entry Points

**Interactive TUI (`grok` with no subcommand):**
- Location: `xai_grok_pager::app::run` via `pager-bin` `async_main`
- Triggers: Default CLI path after parsing
- Responsibilities: Full-screen agent UI, optional leader attach, session start/resume

**CLI subcommands:**
- Location: `Command` match in `crates/codegen/xai-grok-pager-bin/src/main.rs` (~1597+)
- Triggers: `grok login`, `mcp`, `sessions`, `update`, `agent`, `workspace`, …
- Responsibilities: One-shot operations without TUI (or agent modes)

**Agent stdio (`grok agent stdio`):**
- Location: `xai_grok_shell::agent::app::run_stdio_agent`
- Triggers: IDE / desktop spawners, ACP bridges
- Responsibilities: ACP over stdin/stdout, skills watcher, LocalSet agent tasks

**Headless agent:**
- Location: `run_headless` / `run_headless_no_browser`
- Triggers: Scripting/CI (`-p` / agent headless args)
- Responsibilities: Non-interactive turns, optional no-browser auth

**Leader:**
- Location: `run_leader` / `leader::run_leader_server`
- Triggers: `connect_or_spawn` when no healthy leader exists
- Responsibilities: Shared agent + IPC + control plane (workspace start/stop, update relaunch)

**Agent HTTP/WS server:**
- Location: `xai_grok_shell::agent::server::run_agent_server`
- Triggers: `grok agent serve` (bind + secret)
- Responsibilities: Networked agent endpoint for remote clients

**Workspace server binary:**
- Location: `crates/codegen/xai-grok-workspace/src/bin/workspace_server.rs` (`xai-workspace-server`)
- Triggers: Standalone workspace process / hub mode
- Responsibilities: Workspace ops server for remote or multi-process setups

**Utility binaries:**
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

**What happens:** Spawning async work that writes into `AppView` / `AgentView` fields directly.
**Why it's wrong:** Breaks deterministic, testable state transitions and races with the event loop.
**Do this instead:** Return an `Effect` from dispatch; on completion produce a `TaskResult` and re-enter `dispatch` (`crates/codegen/xai-grok-pager/src/app/dispatch/`, `effects/`).

### Adding application logic to the composition-root binary

**What happens:** Growing large features inside `xai-grok-pager-bin/src/main.rs` beyond routing/wiring.
**Why it's wrong:** Binary exists to break dependency cycles and host process setup; logic becomes untestable and duplicated.
**Do this instead:** Put behavior in `xai-grok-pager` (TUI/CLI helpers) or `xai-grok-shell` (agent/runtime); keep main as glue.

### Coupling leaf crates to shell or pager

**What happens:** Adding a dependency from `xai-grok-config` / `xai-chat-state` / common crates back to shell or pager.
**Why it's wrong:** Inflates compile graph and creates cycles; leaf crates are shared by multiple hosts.
**Do this instead:** Keep dependency arrows downward: pager → shell → agent/tools/workspace → common/leaf.

### Bypassing the tool registry for new tools

**What happens:** Calling FS/shell APIs from session code without a registered tool definition.
**Why it's wrong:** Skips permissions, sandbox, hooks, taxonomy, output caps, and versioning.
**Do this instead:** Add under `crates/codegen/xai-grok-tools/src/implementations/grok_build/<tool>/` and register via the tools registry.

### Spawning a second leader casually

**What happens:** Starting agent processes without `connect_or_spawn` / lock path.
**Why it's wrong:** Split-brain sessions, socket races, lost shared workspace exposure.
**Do this instead:** Use `leader::connect_or_spawn` and respect WS-URL-keyed socket/lock paths (`crates/codegen/xai-grok-shell/src/leader/`).

## Error Handling

**Strategy:** Layered — `anyhow::Result` at CLI/TUI boundaries; typed domain errors inside libraries (`thiserror` / custom enums); ACP error objects on the wire.

**Patterns:**
- CLI/TUI: `anyhow` with user-facing `eprintln!` and process exit codes (`pager-bin` main, setup, workspace gates)
- Dispatch: map ACP/network failures into UI state + toasts (e.g. free-usage exhausted helpers in `dispatch/`)
- Sampler: classify errors → retry decisions (`xai-grok-sampler/src/retry.rs`)
- Workspace: `WorkspaceResult` / `WorkspaceError` (`xai-grok-workspace/src/error.rs`)
- Tool protocol: JSON-RPC error codes and `ToolErrorWire` (`xai-tool-protocol`)
- Prefer typed variants over string matching for recoverable UI paths (e.g. `SwitchModelError`)

## Cross-Cutting Concerns

**Logging:**
- `tracing` + `xai-grok-telemetry` layers (unified log, sampling log, hooks log, OTEL, Sentry)
- Headless defaults quieter (`init_tracing_simple`); debug via `GROK_DEBUG_LOG` / env filters
- Prefer structured fields; use `unified_log` for diagnostic sessions

**Validation:**
- Clap for CLI
- Config layer validation + signed requirements fail-closed (`xai_grok_config::validate_requirements`)
- Tool argument schemas / normalization in tools crate
- Folder trust gates for workspace operations (`agent/folder_trust`, workspace `trust`)

**Authentication:**
- OAuth2 / device-code via shell auth + `xai-grok-auth`
- `AuthManager` with proactive refresh and system-power sleep pause
- Bearer injection into sampler/HTTP clients; credentials never logged (secrets sanitizer crate)

**Sandboxing:**
- `xai-grok-sandbox` feature-forwarded as `sandbox-enforce` from binary/pager
- Tool execution paths must honor sandbox policy when enabled

**Hooks & plugins:**
- Lifecycle hooks: `xai-grok-hooks`
- Plugin marketplace: `xai-grok-plugin-marketplace`
- Agent plugins surface: `xai-grok-agent/src/plugins/`

**Markdown / diagrams:**
- `xai-grok-markdown` (+ core) for TUI rendering; Mermaid via `xai-grok-mermaid` / vendored `third_party/mermaid-to-svg`
- Off-thread mermaid worker in pager app (`mermaid_worker`)

---

*Architecture analysis: 2026-07-16*
