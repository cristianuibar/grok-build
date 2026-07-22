# Phase 1: Product identity & isolated home - Research

**Researched:** 2026-07-16
**Domain:** Brownfield Rust CLI rebrand — binary artifact name + product home path isolation
**Confidence:** HIGH

## Summary

Phase 1 is a **brownfield cutover**, not a scaffold. The product already has a single primary home resolver in `xai-grok-config` (`grok_home()` / `default_grok_home()` / `user_grok_home()`), with thin re-exports in `xai-grok-shell-base` and `xai-grok-tools`, plus **one deliberate duplicate** in `xai-fast-worktree` that must stay in sync. Almost all product-state writers hang off that API via `grok_home().join(...)`. Changing the default dir to `~/.bum` and the override env to `BUM_HOME` (and **not** reading `GROK_HOME`) therefore cuts over config, auth, sessions, MCP credentials, marketplace cache, memory, logs, leader socket, and related sinks in one place — provided the duplicate resolver and test harnesses are updated too.

The shipped CLI name is controlled solely by `[[bin]] name` / `default-run` in `crates/codegen/xai-grok-pager-bin/Cargo.toml` (today `xai-grok-pager`). Crate package names stay `xai-grok-*`. Phase 1 success is: `cargo run -p xai-grok-pager-bin --bin bum` (or install) yields an invocable **`bum`**, and a temp-home run writes product state only under that home (zero product writes under `~/.grok` and `~/.codex`). UI chrome, full `GROK_*` renames, telemetry/auto-update kills, and npm distribution naming stay out of scope (Phase 8 / later).

**Primary recommendation:** Cut over **only** the home default + env override at the config-paths SoT (and the fast-worktree twin), rename the composition-root `[[bin]]` to `bum`, update test-support binary/`BUM_HOME` sandbox helpers, and prove isolation with unit + temp-home integration tests — leave project-local `.grok/` layouts and internal crate names alone.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
#### Binary shipping
- Rename the shipped `[[bin]]` artifact so the primary command is **`bum`** (not `grok` / `xai-grok-pager` as the user-facing name)
- Keep internal crate paths/packages (e.g. `xai-grok-pager-bin`) as-is this phase — only the user-facing binary name changes
- Phase 1 success = local `cargo build` / `cargo install` (or project-documented build) yields an invocable **`bum`** binary; npm/install-channel/docs polish deferred to Phase 8
- No compatibility alias from `grok` → `bum` in v1

#### Home path isolation
- Default product home is **`~/.bum`**
- Primary env override is **`BUM_HOME`**
- Do **not** honor `GROK_HOME` as the product home (avoids silent coupling to stock Grok paths)
- No automatic migration from `~/.grok` / stock Codex stores — clean re-login after switch (PROJECT.md out-of-scope: no import/share)

#### Rename depth (phase boundary vs Phase 8)
- Phase 1 must inventory and cut over **all product-state path writers** that use `$GROK_HOME` / `~/.grok` (config, auth, sessions, memory, marketplace cache, logs, leader socket, extracted docs, etc.) so a temp-home run leaves zero product state under `~/.grok`
- UI chrome, help text, and “Grok Build” branding strings → **Phase 8** (except anything that hardcodes a wrong home path or wrong CLI name for invocation)
- Introduce `BUM_HOME` as the home root; leave the broader `GROK_*` knob family for Phase 8 unless a var forces `~/.grok` defaults
- Leave in-repo project layout conventions (e.g. `.grok-plugin`) alone — only product **home** paths change in this phase

#### Verification & test strategy
- Prove isolation with integration/unit tests using a temporary home (`BUM_HOME` or sandboxed HOME) and assert product state is written only under that root
- Gate includes **zero writes** under both `~/.grok` and `~/.codex` for product state during a fresh temp-home run
- Prefer unit tests for home-resolution pure logic plus at least one integration-style check that a temp-home run creates expected dirs under bum home only
- Local developer path: document/build so `cargo run -p xai-grok-pager-bin --bin bum` (or equivalent after rename) works

### Claude's Discretion
- Exact inventory of path call sites and the cleanest central home-resolution API (single source of truth for default dir + env)
- Whether to keep a thin internal alias for `GROK_HOME` **only inside tests** temporarily vs full test-support rename in this phase
- How to stage binary rename in Cargo.toml without breaking workspace CI that still references `xai-grok-pager` internally

### Deferred Ideas (OUT OF SCOPE)
- Full UI chrome / help / “Grok Build” string rebrand → Phase 8
- Disable xAI auto-update + product telemetry → Phase 8 (OPS-01/02)
- Multi-slot credentials & xAI OAuth under bum store → Phase 2
- Import/share stock `~/.grok` or Codex credentials → out of scope for v1
- Full `GROK_*` → `BUM_*` env rename family → Phase 8 unless required for home defaults
- Rename crate packages / `.grok-plugin` project layout → later / not Phase 1
- npm install channel / public distribution naming → Phase 8 / out of scope for official x.ai channel
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ID-01 | User can launch the product as the `bum` CLI binary (not `grok` / `xai-grok-pager` as the primary command name) | `[[bin]]` + `default-run` rename in `xai-grok-pager-bin`; test-support / harness binary resolution (`CARGO_BIN_EXE_*`, local debug path); managed `bin/bum` under product home |
| ID-03 | Config, auth, sessions, and related state live under an isolated `~/.bum` home (or `BUM_HOME`), not `~/.grok` or stock Codex paths | SoT cutover in `xai-grok-config::paths`; twin `resolve_grok_home` in fast-worktree; inventory of product-state writers; isolation tests; no `GROK_HOME` honor; no `~/.codex` writes |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CLI binary artifact name (`bum`) | Composition root (binary crate) | Test harnesses | Cargo `[[bin]]` is the ship name; e2e/PTY resolve via env / cargo bin path |
| Default product home (`~/.bum`) | Shared config library (`xai-grok-config`) | fast-worktree duplicate | OnceLock SoT; all writers should call `grok_home()` |
| Env override (`BUM_HOME`) | Shared config library | Test support / update test fixtures | Single env read in SoT; tests must set before first `grok_home()` |
| Auth file location | Shell auth (`AuthManager`) | Config home | Default `home/auth.json`; optional `GROK_AUTH_PATH` / `GROK_AUTH` still Phase-8 family unless path forces wrong home |
| Sessions / leader socket / locks | Shell + pager | Config home | Paths under product home; socket override `GROK_LEADER_SOCKET` remains non-home knob |
| Sandbox writable product dir | Sandbox crate | Config home | `xai_grok_sandbox` re-exports `xai_grok_config::grok_home()` as always-writable |
| Project-local `.grok/` layout | Workspace / project FS | — | **Do not rename** in Phase 1 (CONTEXT) |
| Stock Codex home (`.codex`) | Foreign-sessions read-only compat | — | Must not become bum write root; isolation tests assert no product writes there |

## Standard Stack

> No new third-party packages. Phase 1 is path + binary rename on the existing Rust workspace. [VERIFIED: codebase]

### Core

| Library / surface | Version / pin | Purpose | Why standard |
|-------------------|---------------|---------|--------------|
| Rust toolchain | **1.92.0** (`rust-toolchain.toml`) | Build product | Workspace pin [VERIFIED: codebase] |
| Cargo workspace | resolver 2, edition 2024 | Multi-crate product | Existing monorepo [VERIFIED: codebase STACK] |
| `xai-grok-config` | path crate | Home SoT + config layers | Already owns `paths.rs` OnceLock [VERIFIED: codebase] |
| `xai-grok-pager-bin` | path crate | Composition root binary | Only place for `[[bin]]` ship name [VERIFIED: codebase] |
| Tokio | workspace | Async runtime | Existing agent/TUI stack [VERIFIED: codebase] |
| `dunce` | workspace | Home canonicalize without Windows `\\?\` | Already required in `default_grok_home` [VERIFIED: codebase] |
| `serial_test` + `tempfile` | workspace | Env-mutating isolation tests | Existing `EnvGuard` / serial patterns [VERIFIED: codebase] |

### Supporting

| Library / surface | Purpose | When to use |
|-------------------|---------|-------------|
| `xai-grok-test-support` | `EnvGuard`, `test_env_cmd_tokio`, `grok_binary()` | Any test spawning binary or mutating env |
| `xai-fast-worktree` | Twin `resolve_grok_home` for worktrees DB | Must update in lockstep with config paths |
| `insta` / crate-local `tests/` | Snapshots / integration | Prefer unit path tests first; one headless isolation check |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Single SoT env `BUM_HOME` only | Dual-read `BUM_HOME` then `GROK_HOME` | Dual-read re-couples to stock Grok if user has `GROK_HOME` set — **forbidden by locked decision** |
| Rename crate packages to `bum-*` | Keep `xai-grok-*` | Package rename is huge blast radius; deferred by CONTEXT |
| Migrate `~/.grok` → `~/.bum` | Clean cutover | Migration is out of v1 scope; half-migration is Pitfall 10 |
| Honor project `.grok` as product home | Only `$HOME/.bum` | Project layout is intentional multi-root; leave alone |

**Installation:** none (no new crates).

**Version verification:** toolchain and deps already pinned in-repo; no registry packages added. [VERIFIED: codebase]

## Package Legitimacy Audit

> Phase installs **no** external packages.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | N/A — no new deps |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Home path inventory (central API + writers)

### Single source of truth [VERIFIED: codebase]

**File:** `crates/codegen/xai-grok-config/src/paths.rs`

| API | Current behavior | Phase 1 target |
|-----|------------------|----------------|
| `default_grok_home()` | `home_dir()/.grok` via `dunce::canonicalize` | `home_dir()/.bum` |
| `grok_home()` | `OnceLock`; env **`GROK_HOME`** else default; **creates dir** | env **`BUM_HOME`** only (ignore `GROK_HOME`); default `~/.bum`; still create |
| `user_grok_home()` | `Some` if `GROK_HOME` or home dir | `Some` if `BUM_HOME` or home dir |
| `grok_application()` | `$HOME_PRODUCT/bin/grok` | `$HOME_PRODUCT/bin/bum` (align managed binary) |
| `sessions_cwd_dir` / `ensure_sessions_cwd_dir` | under `grok_home()/sessions/...` | automatic via SoT |
| `system_config_dir()` | `/etc/grok` | leave for Phase 8 (not user product home; not required for ID-03) |

**Re-exports (no independent logic):** [VERIFIED: codebase]

- `xai_grok_config` lib re-exports paths
- `crates/codegen/xai-grok-shell-base/src/util/grok_home.rs` → re-export
- `crates/codegen/xai-grok-tools/src/util/grok_home.rs` → re-export
- `xai_grok_pager_render::util` re-exports `grok_home`
- `xai_grok_sandbox::paths::grok_home()` → `xai_grok_config::grok_home()`

**Deliberate duplicate (must edit in lockstep):** [VERIFIED: codebase]

- `xai_fast_worktree::db::resolve_grok_home()` — reads `GROK_HOME` then `$HOME/.grok`; comments say keep sync with `default_grok_home`
- Test fixture `GrokHomeFixture` sets `GROK_HOME` — switch to `BUM_HOME`

### Product-state sinks under home (non-exhaustive, from `grok_home().join` usage) [VERIFIED: codebase]

| Relative path / artifact | Domain |
|--------------------------|--------|
| `config.toml`, `managed_config.toml`, `requirements.toml`, `pager.toml` | Config / UI appearance |
| `auth.json` (+ lock / corrupt backups) | Auth (default path via `grok_home().join("auth.json")`) |
| `sessions/` | Session persistence |
| `memory/`, memory logs | Memory SQLite / markdown |
| `marketplace-cache/`, `plugin-data/`, plugins dirs | Marketplace / plugins |
| `mcp_credentials.json`, `mcp_auth_*.lock`, `logs/mcp` | MCP OAuth + logs |
| `leader.sock` / `leader.lock` / `leader.log` (defaults) | Leader IPC |
| `active_sessions.json` | Crash recovery |
| `debug/`, unified/sampling/hooks logs, `agent_id` | Telemetry **file** sinks (network kill = Phase 8) |
| `worktrees.db` (via fast-worktree) | Worktree pool |
| `workspace/`, `bin/<managed>`, `version.json`, downloads | Update / managed install layout |
| `skills/`, bundled extract, docs extract | Builtin extract on startup |
| trust / hooks trust / disabled-hooks | Trust stores |
| `models_cache.json`, personas, etc. | Shell / pager extras |

**Auth path priority** (`AuthManager::new`): [VERIFIED: codebase]

1. Inline `GROK_AUTH` (JSON, read-only) — leave name Phase 8; does not force `~/.grok`
2. `GROK_AUTH_PATH` — absolute override; leave name Phase 8
3. Else `grok_home().join("auth.json")` — **fixed by SoT cutover**

### Must distinguish: product home vs project layout [VERIFIED: codebase + CONTEXT]

| Pattern | Phase 1 action |
|---------|----------------|
| `$HOME/.grok` / `$GROK_HOME` product root | **Cut over** → `~/.bum` / `BUM_HOME` |
| Workspace project `.grok/config.toml`, `.grok/hooks`, `.grok/sandbox.toml`, rewind checkpoints under cwd | **Leave** |
| `.grok-plugin` project convention | **Leave** |
| Legacy discovery that **reads** literal `$HOME/.grok` when home override differs (e.g. `user_agent_dirs` in `xai-grok-agent/src/discovery.rs`) | **Recommend remove or gate off** for product isolation so bum does not pull stock Grok user agents/plugins as “ours” (read-side identity leak; writes still via SoT) |
| Foreign Codex sessions under `CODEX_HOME` / `~/.codex` | **Read-only compat** — assert bum does not **write** product state there |

### Scale of env string [VERIFIED: codebase]

- ~**91** Rust files reference `GROK_HOME` (string), including production + tests
- ~**78** files use `join(".grok")` (mix of product default, project layout, tests)
- Prefer **SoT + test harness** edits over mass rename of internal function names (`grok_home`) this phase

## Binary name (ID-01)

### Ship surface [VERIFIED: codebase]

`crates/codegen/xai-grok-pager-bin/Cargo.toml`:

```toml
default-run = "xai-grok-pager"
[[bin]]
name = "xai-grok-pager"
path = "src/main.rs"
```

**Change to:**

```toml
default-run = "bum"
[[bin]]
name = "bum"
path = "src/main.rs"
```

Package name stays `xai-grok-pager-bin`. Root workspace `Cargo.toml` is **generated** — edit per-crate manifests only. [VERIFIED: codebase AGENTS/STACK]

### Downstream binary resolution that breaks if not updated [VERIFIED: codebase]

| Location | Current | After rename |
|----------|---------|--------------|
| `xai-grok-test-support/src/env.rs` | builds `-p xai-grok-pager --bin xai-grok-pager`; `CARGO_BIN_EXE_xai-grok-pager`; `target/debug/xai-grok-pager` | Prefer `-p xai-grok-pager-bin --bin bum`; `CARGO_BIN_EXE_bum`; `target/debug/bum` (fix wrong `-p xai-grok-pager` for binary — package vs bin) |
| `xai-grok-pager-pty-harness` | `CARGO_BIN_EXE_xai-grok-pager` | `CARGO_BIN_EXE_bum` |
| Shell `managed_grok_bin_name()` | `"grok"` / `grok.exe` under `home/bin` | `"bum"` / `bum.exe` for managed layout under `~/.bum/bin` |
| `grok_application()` | `bin/grok` | `bin/bum` |
| npm packages under `xai-grok-pager/npm/` | ship as `grok` | **Deferred Phase 8** per CONTEXT |

### CI / workspace note (discretion) [ASSUMED]

There is no evidence in this research pass of a green full-workspace CI job that hard-fails on the old bin name beyond test-support/harness references. Staging recommendation:

1. Rename `[[bin]]` + update all `CARGO_BIN_EXE_*` / local path helpers in the **same PR**.
2. Keep crate name `xai-grok-pager-bin` so path deps unchanged.
3. Optional interim: dual `[[bin]]` targets (`bum` + keep `xai-grok-pager`) only if a hidden CI gate appears — **default plan: single bin `bum`** (CONTEXT: no `grok`→`bum` alias; ship name is bum). Prefer fixing harnesses over dual bins.

## Architecture Patterns

### System Architecture Diagram

```text
                    ┌─────────────────────────────┐
                    │  User invokes `bum`         │
                    │  (artifact from pager-bin)  │
                    └──────────────┬──────────────┘
                                   │
                                   ▼
                    ┌─────────────────────────────┐
                    │ Composition root            │
                    │ xai-grok-pager-bin          │
                    │ → pager TUI / shell / CLI   │
                    └──────────────┬──────────────┘
                                   │
           env BUM_HOME? ──────────┤
                                   ▼
                    ┌─────────────────────────────┐
                    │ xai_grok_config::grok_home  │
                    │ OnceLock product home       │
                    │ default: ~/.bum             │
                    └──────────────┬──────────────┘
                                   │
          ┌────────────┬───────────┼────────────┬──────────────┐
          ▼            ▼           ▼            ▼              ▼
     config.toml   auth.json   sessions/   memory/      leader.sock
     marketplace   mcp_creds   debug logs  worktrees.db  skills extract
          │            │           │            │              │
          └────────────┴───────────┴────────────┴──────────────┘
                                   │
                    MUST NOT write product state to:
                    ~/.grok  |  ~/.codex  |  stock Codex home
```

### Recommended implementation approach

1. **SoT cutover first** — `default_grok_home` → `.bum`; env → `BUM_HOME`; update unit tests in `paths.rs` and isolated `grok_home_paths` integration binary.
2. **Twin + managed bin** — `resolve_grok_home`, `managed_grok_bin_name`, `grok_application`.
3. **Display prefixes that encode wrong home** — `display_grok_home_prefix` currently returns `~/.grok` / `$GROK_HOME` [VERIFIED: codebase `pager-render/util.rs`]. Phase 1 must fix **path labels** (CONTEXT exception for wrong home path), not full chrome: `~/.bum` / `$BUM_HOME`.
4. **Binary rename + harnesses** — `[[bin]]`, test-support, pty-harness.
5. **Test support env** — `test_env_cmd_tokio` / leader helpers: set `BUM_HOME` to sandbox path (recommend `home.join(".bum")` or set `BUM_HOME` to the temp root itself — pick one convention and stick to it; today they set `GROK_HOME` to `home.join(".grok")`).
6. **Legacy stock `~/.grok` reads** — turn off product-home legacy scan of literal `~/.grok` when product home is bum (discovery/plugins).
7. **Isolation proof tests** — see Validation Architecture.
8. **Docs for local dev** — one-liner in phase notes / PROJECT or short comment in bin Cargo.toml: `cargo run -p xai-grok-pager-bin --bin bum`.

### Pattern 1: OnceLock + early env set

**What:** `grok_home()` caches first resolution forever in-process.  
**When:** Production OK; tests must set `BUM_HOME` **before first call**, use isolated test binaries, or process-per-test (nextest).  
**Example (target):**

```rust
// Source: adapted from crates/codegen/xai-grok-config/src/paths.rs [VERIFIED: codebase]
pub fn default_product_home() -> PathBuf {
    let home = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dunce::canonicalize(&home).unwrap_or(home).join(".bum")
}

pub fn grok_home() -> PathBuf {
    GROK_HOME
        .get_or_init(|| {
            let root = if let Ok(v) = std::env::var("BUM_HOME") {
                PathBuf::from(v)
            } else {
                default_product_home()
            };
            let _ = std::fs::create_dir_all(&root);
            root
        })
        .clone()
}
```

Internal function name `grok_home` may remain for blast-radius control (discretion); document that it returns the **bum product home**.

### Pattern 2: EnvGuard + serial tests

**What:** RAII env restore; `#[serial_test::serial]` for process-global env.  
**Example:**

```rust
// Source: crates/codegen/xai-grok-test-support/src/env.rs [VERIFIED: codebase]
let _home = EnvGuard::set("BUM_HOME", temp_home.path());
// NEVER leave GROK_HOME as product override in production code paths
```

### Anti-Patterns to Avoid

- **Half-rebrand:** rename binary only, leave `default_grok_home` as `.grok` — Pitfall 4 [CITED: .planning/research/PITFALLS.md]
- **Dual-honor `GROK_HOME`:** re-couples to stock Grok when users export it [CONTEXT locked]
- **Renaming project `.grok/` trees:** out of scope; breaks project config/hooks
- **Mass renames of `GROK_*` knobs** in Phase 1 except home default
- **Migration scripts** from `~/.grok` — forbidden for v1
- **Assuming npm / `install.sh` rename** — Phase 8

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Product home resolution | New path crate or ad-hoc `HOME.join` in each crate | `xai_grok_config::{grok_home, default_grok_home, user_grok_home}` | OnceLock + create_dir + dunce already solved; writers already hang off it |
| Windows path sandboxing | `HOME` alone | Set product home env (`BUM_HOME`) + HOME | Documented: Windows home is not only `$HOME` [VERIFIED: test-support env.rs] |
| Env mutation cleanup | manual set/remove in tests | `EnvGuard` + `#[serial]` | Panic-safe restore |
| Atomic auth write | custom lock protocol | existing `auth.json` + `AuthFileLock` under new home | Already production-hardened |
| Isolation proof | manual “looks fine” | temp-home automated asserts | Pitfall 10 [CITED: PITFALLS.md] |

**Key insight:** Blast radius is large in *references*, but *logic* is concentrated — cut SoT + twin + harnesses, then grep for remaining production `GROK_HOME` / literal `".grok"` defaults that bypass the API.

## Runtime State Inventory

> Phase includes path identity rename — inventory required.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | Real user `~/.grok` (large existing tree on this machine); possible empty/partial `~/.bum` | **No migration** (locked). New writes go to `~/.bum`. Existing `~/.grok` left untouched. Code edit only for new path writers. |
| Live service config | Leader socket/lock under product home; managed install under `home/bin` | Code edit: new defaults under `~/.bum`. Old leader under `~/.grok` not reused. |
| OS-registered state | None found for product home string (no systemd unit for grok home) [ASSUMED for this host] | None |
| Secrets/env vars | `GROK_HOME`, `GROK_AUTH_PATH`, `GROK_AUTH`, `GROK_BINARY`, `GROK_LEADER_SOCKET`; user shell may export `GROK_HOME` | Product home must **ignore** `GROK_HOME`. Auth overrides can keep names until Phase 8. Developers: set `BUM_HOME` for tests. |
| Build artifacts | `target/debug/xai-grok-pager`, `CARGO_BIN_EXE_xai-grok-pager`, possible installed `grok` binary | After rename: rebuild; update harness paths; reinstall local bin as `bum` |

**Nothing found in category:** OS-registered product-home units — none observed on research host (no package manager unit scanned beyond filesystem homes). [ASSUMED]

## Common Pitfalls

### Pitfall 1: Half-rebrand (binary only)
**What goes wrong:** `bum` still writes `~/.grok` and clobbers stock Grok.  
**Why:** Home logic separate from Cargo bin name.  
**How to avoid:** Same-phase SoT cutover + isolation test.  
**Warning signs:** Fresh run creates `~/.grok/auth.json`.  
**Source:** [CITED: .planning/research/PITFALLS.md Pitfall 4]

### Pitfall 2: Incomplete home isolation
**What goes wrong:** Auth under `~/.bum` but sessions/logs/MCP still under `~/.grok` because of bypass or twin resolver.  
**How to avoid:** Grep production `GROK_HOME` and `join(".grok")` after SoT change; update fast-worktree; run zero-write isolation test.  
**Source:** [CITED: PITFALLS.md Pitfall 10]

### Pitfall 3: OnceLock freezes wrong home in tests
**What goes wrong:** Test sets `BUM_HOME` after another test called `grok_home()`.  
**How to avoid:** Isolated test binary (existing `grok_home_paths` pattern), process-level init before any call, or nextest process isolation; update-suite `test_home()` OnceLock pattern.  
**Source:** [VERIFIED: codebase update tests + pager `grok_home_paths.rs`]

### Pitfall 4: Honoring leftover `GROK_HOME`
**What goes wrong:** User has `export GROK_HOME=~/.grok` from stock CLI; bum silently uses stock home.  
**How to avoid:** Read **only** `BUM_HOME` (locked decision).  
**Source:** [VERIFIED: CONTEXT.md]

### Pitfall 5: Renaming project `.grok` layout
**What goes wrong:** Breaks project hooks/MCP/sandbox configs in repos.  
**How to avoid:** Grep with care; change only **default product home** string and env, not `dir.join(".grok")` project paths.  
**Source:** [VERIFIED: CONTEXT.md]

### Pitfall 6: Binary harness drift
**What goes wrong:** `cargo test` e2e still looks for `xai-grok-pager` artifact → suite red.  
**How to avoid:** Update test-support + pty-harness + any `CARGO_BIN_EXE_*` in the same change set as `[[bin]]`.  
**Source:** [VERIFIED: codebase]

### Pitfall 7: `create_dir_all` on first `grok_home()`
**What goes wrong:** Calling `grok_home()` creates the product directory; isolation tests that only resolve paths may still create dirs under wrong root if env unset.  
**How to avoid:** Always set `BUM_HOME` (or sandboxed HOME) before any product entrypoint.  
**Source:** [VERIFIED: codebase paths.rs]

## Code Examples

### Home resolution unit test (target)

```rust
// Source pattern: crates/codegen/xai-grok-config/src/paths.rs tests [VERIFIED: codebase]
#[test]
fn default_home_is_dot_bum() {
    let home = default_grok_home(); // or renamed default_product_home
    assert!(home.ends_with(".bum"));
    assert!(!home.to_string_lossy().starts_with(r"\\?\"));
}
```

### Isolated env override binary test (adapt existing)

```rust
// Source: crates/codegen/xai-grok-pager/tests/grok_home_paths.rs [VERIFIED: codebase]
// Change GROK_HOME → BUM_HOME; $GROK_HOME display → $BUM_HOME
unsafe { std::env::set_var("BUM_HOME", &tmp_path); }
assert_eq!(display_grok_home_prefix(), "$BUM_HOME");
```

### Temp-home child process sandbox

```rust
// Source: crates/codegen/xai-grok-test-support/src/env.rs [VERIFIED: codebase]
cmd.env("HOME", home)
  .env("BUM_HOME", home.join(".bum")) // or BUM_HOME = home if product root == sandbox root
  // do NOT set GROK_HOME
  .env("GROK_TELEMETRY_ENABLED", "false")
  .env("GROK_DISABLE_AUTOUPDATER", "1");
```

### Isolation assertion sketch

```rust
// Recommended Phase 1 test shape [ASSUMED: pattern, not existing test]
// After short headless `bum -p ...` or path-init entrypoint under TempDir HOME + BUM_HOME:
assert!(bum_home.join("config.toml").exists() || bum_home.exists());
assert!(!real_or_sandbox_root.join(".grok").exists()
    || !product_files_under(sandbox.join(".grok")));
// Prefer: entire product root is BUM_HOME; find no writes outside it for known product filenames
```

## State of the Art

| Old Approach | Current Approach (post Phase 1) | When | Impact |
|--------------|----------------------------------|------|--------|
| Product home `~/.grok` + `GROK_HOME` | `~/.bum` + `BUM_HOME` only | Phase 1 | Isolation from stock Grok |
| Binary `xai-grok-pager` / install name `grok` | Ship name `bum` (crates keep xai-grok-*) | Phase 1 | ID-01 |
| Dual-read home aliases | Explicit non-honor of `GROK_HOME` | Phase 1 locked | No silent coupling |
| Full chrome rebrand + quiet fork | Deferred | Phase 8 | Scope control |

**Deprecated/outdated for this fork:**

- Treating `GROK_HOME` as the product home override for bum
- Expecting credential continuity from `~/.grok` without re-login

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | No OS-level service units register the product home path on typical Linux desktops | Runtime State | May miss re-registration if someone packaged grok as a service |
| A2 | Dual `[[bin]]` interim not required for CI | Binary name | CI may hardcode old bin name outside test-support — planner should grep CI configs in plan Wave 0 |
| A3 | Isolation test can be satisfied without full TUI PTY (headless / path-init + short agent) | Validation | If only full PTY proves writes, e2e cost rises |
| A4 | Leaving `GROK_AUTH` / `GROK_AUTH_PATH` names is OK for Phase 1 | Auth | Low — paths still under bum home when unset |

**If this table is empty:** N/A — four assumed items above.

## Open Questions

1. **Internal API naming (`grok_home` vs `product_home`)** — **(RESOLVED)**
   - What we know: callers are widespread; rename is optional.
   - **Chosen:** Keep public symbol `grok_home()` / `default_grok_home()` this phase; document that they return the bum product home. No required `product_home()` alias (blast-radius control; D-SCOPE).

2. **Test harness env: `BUM_HOME=tmp` vs `BUM_HOME=tmp/.bum`** — **(RESOLVED)**
   - What we know: today `GROK_HOME=home.join(".grok")` while also setting `HOME`.
   - **Chosen:** Convention `HOME=tmp` and `BUM_HOME=home.join(".bum")` (mirror prior `HOME` + `GROK_HOME=home.join(".grok")`). Assert product files under BUM_HOME and zero product writes under `HOME/.grok` / `HOME/.codex` (plans 01-04 / 01-05).

3. **Legacy `~/.grok` discovery reads** — **(RESOLVED)**
   - What we know: agent/plugin discovery intentionally scans legacy `~/.grok` when override differs.
   - **Chosen:** Disable stock-home legacy user-agent product scans for bum isolation (plan 01-05 Task 1). Keep project-local cwd `.grok` discovery (D-PLUGIN). No credential import (D-MIGRATE).

4. **`/etc/grok` system config** — **(RESOLVED)**
   - What we know: `system_config_dir` is `/etc/grok`.
   - **Chosen:** Leave `/etc/grok` unchanged in Phase 1 (D-SCOPE / Phase 8).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (rustup) | build/test | ✗ in bare PATH (`rustc`/`cargo` not found without rustup env) | pin 1.92.0 via `rust-toolchain.toml` | Activate rustup (`source ~/.cargo/env` or similar) before cargo |
| Node.js | optional npm packaging | ✓ | v22.22.1 | Not required for Phase 1 |
| protoc / bin/protoc | workspace crates | present as project tooling [ASSUMED if building full workspace] | repo `bin/protoc` | PATH / `$PROTOC` |
| Writable temp dirs | isolation tests | ✓ | — | — |
| Network | not required for Phase 1 path/binary | — | — | mock/disable as existing tests |

**Missing dependencies with no fallback:**

- Rust/cargo must be available via rustup for implementers (toolchain file present).

**Missing dependencies with fallback:**

- Full workspace build optional; per-crate `cargo test -p xai-grok-config` / `xai-grok-pager-bin` preferred.

## Validation Architecture

> `workflow.nyquist_validation` is **true** in `.planning/config.json`. [VERIFIED: config]

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (crate-local); `serial_test`; `tempfile`; optional nextest |
| Config file | none global — per-crate; `rust-toolchain.toml` |
| Quick run command | `cargo test -p xai-grok-config paths -- --nocapture` (and home unit tests) |
| Full suite command | `cargo test -p xai-grok-config -p xai-grok-test-support -p xai-grok-pager-bin -p xai-fast-worktree` (expand as harnesses updated) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ID-01 | Binary artifact named `bum` | build smoke | `cargo build -p xai-grok-pager-bin --bin bum` | ❌ Wave 0 (after rename) |
| ID-01 | Harness resolves `bum` binary | unit | `cargo test -p xai-grok-test-support` (or compile-check env helpers) | ⚠️ update existing |
| ID-03 | Default home ends with `.bum` | unit | `cargo test -p xai-grok-config default_grok_home` | ⚠️ update existing assert `.grok` |
| ID-03 | Override via `BUM_HOME` only | unit / isolated bin | adapt `grok_home_paths` + paths tests | ⚠️ rewrite env key |
| ID-03 | `GROK_HOME` ignored | unit | new test: set GROK_HOME + unset BUM_HOME → still default `.bum` under sandboxed HOME | ❌ Wave 0 |
| ID-03 | Zero product writes under `.grok` / `.codex` | integration | new temp-home test spawning short CLI or calling init extract paths | ❌ Wave 0 |
| ID-03 | fast-worktree uses same home | unit | existing `GrokHomeFixture` tests with `BUM_HOME` | ⚠️ update |

### Sampling Rate

- **Per task commit:** quick unit tests for touched crate (`xai-grok-config` and/or pager-bin)
- **Per wave merge:** multi-crate test list above
- **Phase gate:** isolation integration green + `cargo build -p xai-grok-pager-bin --bin bum`

### Wave 0 Gaps

- [ ] Update `xai-grok-config` path unit tests (`.bum`, `BUM_HOME`)
- [ ] Update/rename `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` for `BUM_HOME`
- [ ] New test: `GROK_HOME` ignored for product home
- [ ] New isolation test: temp HOME + `BUM_HOME`, assert no product files under `.grok` / `.codex`
- [ ] Update `xai-grok-test-support` binary resolution + env sandbox keys
- [ ] Update `xai-fast-worktree` `resolve_grok_home` + `GrokHomeFixture`
- [ ] Grep CI/scripts for `--bin xai-grok-pager` / `CARGO_BIN_EXE_xai-grok-pager`

## Security Domain

> `security_enforcement` enabled (default).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | partial | Existing OAuth/auth.json under new home; no new auth protocol this phase |
| V3 Session Management | partial | Session files under isolated home only |
| V4 Access Control | yes | Path isolation from stock CLI homes; no credential share |
| V5 Input Validation | yes | Env path taken as `PathBuf` (existing); avoid path traversal into unexpected roots via explicit absolute `BUM_HOME` |
| V6 Cryptography | no new | Existing secure file writers for auth; do not hand-roll crypto |

### Known Threat Patterns for this phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Writing tokens into stock `~/.grok` or `~/.codex` | Information disclosure / Tampering | SoT home + isolation tests; no dual-home write |
| User `GROK_HOME` env forces stock home | Elevation / Disclosure | Do not honor `GROK_HOME` |
| World-readable auth.json | Disclosure | Keep existing secure write modes under new path |
| Accidental migration/import of foreign creds | Tampering | Explicit non-goal; no import code |

## Recommended plan split (for planner)

| Wave / plan | Focus | Exit criteria |
|-------------|-------|---------------|
| **1 — SoT home cutover** | `xai-grok-config/paths.rs` + unit tests; display home prefix; docs comments | Default `.bum`; `BUM_HOME` works; `GROK_HOME` ignored |
| **2 — Twin + managed paths** | fast-worktree `resolve_grok_home`; `grok_application`; `managed_grok_bin_name`; sandbox comments | Same root as config |
| **3 — Binary rename + harnesses** | pager-bin `[[bin]]`; test-support; pty-harness `CARGO_BIN_EXE_*` | `cargo build --bin bum` green; harnesses find binary |
| **4 — Test env mass-update (scoped)** | test-support leader/env; update test `GROK_HOME` fixtures; shell e2e helpers that set product home | Tests set `BUM_HOME` |
| **5 — Isolation proof + legacy read gate** | New integration isolation test; disable stock `~/.grok` legacy product discovery if required | Zero product writes under `.grok`/`.codex` under temp home |

Keep function symbol renames optional to limit diff noise.

## Project Constraints (from AGENTS.md / PROJECT)

- Stay on Rust workspace — fork evolution, not rewrite [VERIFIED: AGENTS.md]
- Identity: isolated `~/.bum`; no credential sharing with stock CLIs in v1
- Naming: product and CLI are `bum`
- Root `Cargo.toml` generated — edit per-crate manifests
- Prefer per-crate `cargo check/test -p <crate>`
- `dunce::canonicalize` not `std::fs::canonicalize`

## Sources

### Primary (HIGH confidence)

- `crates/codegen/xai-grok-config/src/paths.rs` — SoT home API
- `crates/codegen/xai-grok-pager-bin/Cargo.toml` — `[[bin]]` / `default-run`
- `crates/codegen/xai-grok-test-support/src/env.rs` — EnvGuard, sandbox, binary resolve
- `crates/codegen/xai-fast-worktree/src/db/mod.rs` — twin resolver + `GrokHomeFixture`
- `crates/codegen/xai-grok-shell/src/auth/manager.rs` — auth path resolution
- `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` — OnceLock isolation pattern
- `.planning/phases/01-product-identity-isolated-home/01-CONTEXT.md` — locked decisions
- `.planning/REQUIREMENTS.md` — ID-01, ID-03
- Workspace greps of `GROK_HOME` / `grok_home()` / `join(".grok")` (2026-07-16)

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` — half-rebrand, incomplete isolation
- `.planning/research/ARCHITECTURE.md` / `SUMMARY.md` — phase order identity first
- `.planning/codebase/TESTING.md` — HOME + GROK_HOME sandbox guidance

### Tertiary (LOW confidence)

- Host-specific presence of `~/.bum` / `~/.grok` directories during research (environment snapshot only)
- CI matrix completeness for bin name outside in-repo harnesses

## Metadata

**Confidence breakdown:**

- Standard stack: **HIGH** — no new deps; existing workspace verified
- Architecture: **HIGH** — SoT + twin + writers mapped in code
- Pitfalls: **HIGH** — project research + concrete OnceLock/harness traps verified
- Isolation e2e shape: **MEDIUM** — exact entrypoint for minimal write-set test left to planner

**Research date:** 2026-07-16  
**Valid until:** 2026-08-15 (stable brownfield paths; re-grep if large auth/home refactors land)
