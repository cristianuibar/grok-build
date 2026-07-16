# Codebase Concerns

**Analysis Date:** 2026-07-16

## Tech Debt

**God files (10k+ LOC modules):**
- Issue: Several production modules exceed 9–12k lines, mixing domain logic, config resolution, and tests. Hard to navigate, review, or split safely without ripple breakage.
- Files:
  - `crates/codegen/xai-grok-pager/src/views/settings_modal.rs` (~12.5k)
  - `crates/codegen/xai-grok-shell/src/agent/config.rs` (~11.3k; ~4.9k of which is `#[cfg(test)]`)
  - `crates/codegen/xai-grok-pager/src/app/app_view.rs` (~10.3k)
  - `crates/codegen/xai-grok-pager/src/views/dashboard/state.rs` (~10.3k)
  - `crates/codegen/xai-ratatui-textarea/src/textarea.rs` (~9.7k)
  - `crates/codegen/xai-grok-workspace/src/handle.rs` (~9.5k)
  - `crates/codegen/xai-grok-sampling-types/src/conversation.rs` (~9.5k)
  - `crates/codegen/xai-grok-mcp/src/servers.rs` (~7.5k)
- Impact: Slow compile of dependents, high merge conflict rate, reviewers miss security-sensitive changes. New features tend to append rather than extract.
- Fix approach: Split by domain (settings UI vs. dashboard state; `AgentConfig` resolution vs. structs; MCP server lifecycle vs. transport). Keep unit tests co-located but in sibling modules (`*_tests.rs` or `mod tests` files). Prefer extracting pure types to `xai-grok-config-types` / `xai-grok-sampling-types` first.

**Workspace wire-type placeholders / dual type systems:**
- Issue: `xai-grok-workspace-types` defines many **placeholder** structs that intentionally diverge from canonical types in other crates. ~24 `TODO(workspace)` markers track alignment debt. Proto generation is not implemented.
- Files:
  - `crates/codegen/xai-grok-workspace-types/src/types/mod.rs` (documents placeholders)
  - `crates/codegen/xai-grok-workspace-types/src/types/{config,git,hunk,session,permission,tools,search,memory,plugins,files,interaction}.rs`
  - `crates/codegen/xai-grok-workspace-types/src/lib.rs` (`// TODO: proto generation`)
  - `crates/codegen/xai-grok-workspace-types/src/identity.rs` (`HunkId` alignment)
- Impact: Wire/API drift risk between hub, shell, pager, and future WASM SDK. Silent field mismatches when systems are extracted. Extra mapping layers.
- Fix approach: For each domain, promote one crate as source of truth; re-export or generate the other. Land planned `.proto` codegen before expanding the RPC surface further.

**ACP 0.10 non-exhaustive matching (~21 TODOs):**
- Issue: `agent-client-protocol` 0.10 types are `#[non_exhaustive]`. Match arms leave unknown variants unhandled or silently ignored; comments defer proper rejection.
- Files (representative):
  - `crates/codegen/xai-grok-mcp/src/servers.rs`
  - `crates/codegen/xai-grok-shell/src/util/config/mcp.rs`
  - `crates/codegen/xai-grok-shell/src/extensions/mcp.rs`
  - `crates/codegen/xai-grok-shell/src/session/managed_mcp.rs`
  - `crates/codegen/xai-grok-shell/src/tools/todo.rs`
  - `crates/codegen/xai-grok-workspace/src/permission/resolution.rs`
  - `crates/codegen/xai-grok-workspace/src/permission/prompter.rs`
  - `crates/codegen/xai-grok-pager/src/slash/acp_command.rs`
  - `crates/codegen/xai-grok-pager-render/src/appearance/permission_cursor.rs`
- Impact: New ACP protocol variants may be dropped or mis-mapped until someone greps for `TODO(acp-0.10)`.
- Fix approach: Centralize conversion helpers with explicit `_ => reject/log` paths; one crate owns ACP → internal mapping (`xai-acp-lib` or shell util).

**Sandbox crate compiles with broad allow-lints:**
- Issue: `xai-grok-sandbox` starts with a crate-level `#![allow(unused_imports, unused_variables, unused_mut, unreachable_code, dead_code)]`, masking incomplete paths and platform stubs.
- Files: `crates/codegen/xai-grok-sandbox/src/lib.rs`
- Impact: Dead / unreachable security code can ship unnoticed; platform gaps look like intentional stubs.
- Fix approach: Remove blanket allows; use `cfg`-gated modules and `#[cfg_attr]` for platform-specific dead code only.

**Generated workspace root Cargo.toml:**
- Issue: Root `Cargo.toml` is auto-generated from the monorepo sync; treat as read-only (documented in `README.md`). Dependency versions and member list can drift from developer expectations when editing locally.
- Files: `Cargo.toml`, `README.md` (repository layout note)
- Impact: Local edits to workspace root are lost or fight the generator; full-workspace builds are intentionally discouraged (slow).
- Fix approach: Always edit per-crate `Cargo.toml`. Document the generator entrypoint for contributors who work in this export tree only.

**Legacy config / protocol compat shims:**
- Issue: Multiple layers keep legacy field names and conversion paths (memory `recency_decay`, MCP credentials shapes, tool-protocol hello without capabilities, doom-loop config namespace).
- Files:
  - `crates/codegen/xai-grok-config-types/src/memory.rs`
  - `crates/codegen/xai-grok-mcp/src/credentials.rs`
  - `crates/common/xai-tool-protocol/tests/serde_roundtrip.rs`
  - `crates/codegen/xai-grok-config-types/src/lib.rs` (removed `doom_loop_*` keys)
- Impact: Dual code paths increase test surface and risk of “fixed in new path, broken in legacy.”
- Fix approach: Age out with version gates or one-time migrations; add sunset dates in comments where retention is intentional.

**Tracing middleware parked as dead_code:**
- Issue: Fastrace tonic/http middleware marked TODO to move; currently `#[allow(dead_code)]`.
- Files: `crates/common/xai-tracing/src/fastrace.rs`
- Impact: Incomplete observability migration; dead code may bit-rot.
- Fix approach: Move into grpc/http clients or delete if superseded.

## Known Bugs

**Hunk attribution via fs_notify (External vs AgentEdit) — still demonstrated as broken:**
- Symptoms: When agent tool writes are only observed through fs_notify → `handle_file_change`, hunks are classified as `HunkSource::External` instead of `AgentEdit { prompt_index }`. Accept/reject/attribution UX and undo semantics then treat agent edits like external editor changes.
- Files:
  - `crates/codegen/xai-hunk-tracker/src/actor/tests.rs` (`test_bug_fs_notify_path_creates_external_hunks_not_agent_hunks`)
  - Integration point described in test: CLI shell must call `record_agent_write` from tool paths **before** fs_notify fires
- Trigger: Agent write tools that only touch disk and rely on the watcher path.
- Workaround: Call `record_agent_write` from `search_replace` / `write_file` / equivalent tool execution paths before the notify event is processed.
- Note: Several other historic hunk bugs (per-hunk accept clearing siblings, reject reverting whole file, agent-to-agent prompt_index merge) have **FIXED** assertions in the same test file — treat remaining “BUG DEMONSTRATION” comments carefully; check assertions for `FIXED:` vs current-broken expectations.

**Hunk tracker snapshot restore does not re-notify clients:**
- Symptoms: After `restore_snapshot`, TUI / VS Code extension may show stale hunk state until manual refresh.
- Files: `crates/codegen/xai-hunk-tracker/src/actor/mod.rs` (`restore_snapshot`, TODO re-emit `HunkEvent::*` or `StateRestored`)
- Trigger: Session rewind / snapshot restore flows.
- Workaround: Client full refresh if available.

**Idle session unload race (check-then-act):**
- Symptoms: Background-only sessions (monitors, scheduler fires, background tools/subagents) can be idle-unloaded while work is live, or a turn can arrive between `IsBusy` and `Shutdown`.
- Files: `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs` (`session_has_live_work`, `TODO(PR-4)`)
- Trigger: Detached sessions with only background activity; or prompt arriving during unload race window.
- Workaround: Keep-resident defaults and auto-wake turns; work is “bounded and recoverable on reload” per comments — not zero-loss.

**Compaction cancel can drop post-cancel prompts:**
- Symptoms: Client-side cancel of `/compact` without server-side cancellation; history may be replaced when compaction finishes, losing prompts sent after cancel.
- Files: `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs` (`// TODO: Add dispatch_cancel_command()`)
- Trigger: User cancels compact, continues chatting before compact task ends.
- Workaround: Avoid sending prompts until compact completes, or wait for shell-side cancel token support.

**Auth diagnostic metadata incomplete for External TTL:**
- Symptoms: Attribution/debug JSON for token expiry may not match `AuthManager::is_token_expired` for External auth mode when `expires_at` is `None`.
- Files: `crates/codegen/xai-grok-shell/src/auth/attribution.rs`
- Trigger: External auth tokens without `expires_at`.
- Workaround: Prefer full `AuthManager` APIs for real expiry decisions; attribution path is diagnostic-only.

**Historical auth wipe race (mitigated, still sensitive):**
- Symptoms: Treating HTTP 403 as auth failure could force OIDC refresh + `auth_required` and race with `invalid_grant_threshold` to wipe `auth.json`.
- Files: `crates/codegen/xai-grok-sampling-types/src/error.rs` (`is_auth_error` documents 401-only; comments on race)
- Trigger: Content-safety / policy 403s previously misclassified.
- Workaround: Keep 403 out of auth-refresh paths; any new status mapping must re-read this contract.

## Security Considerations

**OS sandbox platform gaps:**
- Risk: Sandbox is Landlock/Seatbelt via `nono` (unix + `enforce` feature). Unsupported platforms log and continue **without** sandbox. Linux deny-glob enforcement is **best-effort** (expanded at launch; post-launch matches not covered). Network stays open at process level (LLM API); child network blocked via seccomp where implemented.
- Files:
  - `crates/codegen/xai-grok-sandbox/src/lib.rs`
  - `crates/codegen/xai-grok-sandbox/src/deny/glob.rs`
  - `crates/codegen/xai-grok-sandbox/src/child_net.rs`
- Current mitigation: Profile system (`ProfileName`), deny globs, child network filter on Linux, violation logging.
- Recommendations: Fail closed or loud-fail when profile ≠ `off` but apply fails; document Windows “no sandbox” clearly in user-facing docs; tighten Linux post-launch deny if Landlock allows.

**Permission / shell bypass surface (high complexity):**
- Risk: Agent runs shell commands; security depends on tree-sitter bash parsing + classifiers correctly detecting writes, redirections, wrappers, and pager/open-file tricks. Parser failures force Ask (good), but incomplete word-handling and wrapper unwrapping are explicit TODOs.
- Files:
  - `crates/codegen/xai-grok-workspace/src/permission/` (~20k LOC total)
  - `crates/codegen/xai-grok-workspace/src/permission/shell_access.rs` (~2k)
  - `crates/codegen/xai-grok-workspace/src/permission/bash_command_splitting.rs`
  - `crates/codegen/xai-grok-workspace/src/permission/auto_mode.rs` (LLM classifier + fast paths)
  - `crates/codegen/xai-grok-workspace/src/permission/manager.rs` (~4.9k)
- Current mitigation: Extensive unit tests for git grep open-pager variants, sed redirects into `.env`, etc.; syntax errors → prompt.
- Recommendations: Treat permission changes as security reviews; fuzz bash command corpus; prefer sandbox + path denylist as defense in depth, not classifier alone.

**Non-atomic agent file writes:**
- Risk: `LocalFs::write_file` uses `tokio::fs::write` after `create_dir_all` — no temp+rename. Crash/power loss can leave truncated files. Trait documents `// TODO: handle atomic write`. Default `try_read_file` is exists-then-read (TOCTOU / double round-trip).
- Files:
  - `crates/codegen/xai-grok-workspace/src/file_system/fs.rs`
  - `crates/codegen/xai-grok-workspace/src/file_system/local_fs.rs`
- Current mitigation: Some higher-level paths (e.g. hub auth) use rename patterns (`hub_auth.rs`).
- Recommendations: Implement write-to-temp + `rename` in `LocalFs` and require backends to override `try_read_file`.

**Secret redaction is heuristic, not cryptographic:**
- Risk: `xai-grok-secrets` uses regex redaction for common key shapes (sk-, AWS, GitHub, JWT, PEM, assignment patterns). Novel secrets, short tokens, or non-string channels may leak into logs/telemetry/prompts.
- Files: `crates/codegen/xai-grok-secrets/src/sanitizer.rs`, `crates/codegen/xai-grok-secrets/src/lib.rs`
- Current mitigation: Central redaction helpers; path and URL redaction.
- Recommendations: Redact at every egress (telemetry, feedback, remote sync, debug logs); add allowlist of sensitive config keys; never rely on redaction alone for secrets storage.

**Prompt templates only XOR-obfuscated:**
- Risk: Built-in system prompts are XOR-encrypted with a position-dependent seed — obfuscation against casual `strings`, not confidentiality. Anyone with the binary recovers templates trivially.
- Files:
  - `crates/codegen/xai-grok-agent/scripts/encrypt_templates.py`
  - `crates/codegen/xai-grok-agent/src/prompt/template.rs`
  - `crates/codegen/xai-grok-agent/src/prompt/prompt_encrypted.rs` (generated)
- Current mitigation: Decrypt on demand into `Zeroizing` buffers.
- Recommendations: Do not treat template “encryption” as a security boundary; if IP protection matters, use proper packaging or server-side prompts.

**MCP OAuth / credentials file locking:**
- Risk: MCP credential store uses `libc::flock` and filesystem paths; multi-process races and shared home directories need careful permission bits.
- Files: `crates/codegen/xai-grok-mcp/src/credentials.rs`, `crates/codegen/xai-grok-mcp/src/oauth.rs`
- Current mitigation: Exclusive flock around updates; legacy credential fixtures still deserialize.
- Recommendations: Ensure files are mode 0600; document multi-instance leader/follower credential sharing.

**Windows path canonicalization footguns:**
- Risk: `std`/`tokio` `canonicalize` produce `\\?\` verbatim paths that break git, leak into model context, and break path-equality keys. Clippy bans raw canonicalize for codegen crates, but Bazel-only paths may not enforce the same `clippy.toml`.
- Files: `clippy.toml` (repo root; duplicated note for monorepo)
- Current mitigation: Prefer `dunce::canonicalize`; tools crate helpers for async.
- Recommendations: Keep ban enforced; audit any `#[allow(clippy::disallowed_methods)]` in non-test code.

**Plugin / marketplace install surface:**
- Risk: Installing plugins from git/local paths expands trusted code execution surface (hooks, MCP, tools).
- Files: `crates/codegen/xai-grok-plugin-marketplace/src/installer.rs`, agent plugins under `crates/codegen/xai-grok-agent/src/plugins/`
- Current mitigation: Installer tests for path escape / registry save failure injection.
- Recommendations: Default to stricter trust prompts; pin hashes; sandbox plugin-run hooks.

## Performance Bottlenecks

**Streaming markdown open-block clone is O(lines²) residual:**
- Problem: Each open-code highlight pass clones `committed_lines`, documented as O(lines)/pass → O(lines²)/stream.
- Files: `crates/codegen/xai-grok-markdown/src/open_code_highlighter.rs`
- Cause: Owned return type for open-block path; accepted residual (syntect work already amortized).
- Improvement path: Borrowed return through `Replace` + render pipeline (noted in TODO).

**Full-workspace builds are slow by design:**
- Problem: ~80+ workspace members; README directs per-crate `cargo check -p` / `cargo test -p`.
- Files: `Cargo.toml`, `README.md`
- Cause: Large dependency graph (shell/pager pull most of the tree).
- Improvement path: Continue crate extraction; avoid adding dependencies from leaf crates to `xai-grok-shell`.

**Large TUI / dashboard state:**
- Problem: Multi-thousand-line view state and render modules (settings modal, dashboard, extensions modal) can pressure frame budget when rebuilt wholesale.
- Files:
  - `crates/codegen/xai-grok-pager/src/views/settings_modal.rs`
  - `crates/codegen/xai-grok-pager/src/views/dashboard/{state,render}.rs`
  - `crates/codegen/xai-grok-pager/src/views/extensions_modal.rs`
- Cause: Monolithic state + render coupling.
- Improvement path: Incremental dirty flags; split modals into sub-widgets already partially present under `views/`.

**MCP SSE reconnect flood (mitigated, regression-sensitive):**
- Problem: Upstream rmcp zero-backoff SSE reconnect can spam reconnects/WARN logs on abnormal body death.
- Files: `crates/codegen/xai-grok-mcp/tests/repro_sse_flood.rs`, MCP HTTP client / `WarnBudget`
- Cause: Transport reconnect policy.
- Improvement path: Keep regression test green; any rmcp upgrade must re-run SSE flood repro.

**Codebase graph / content search at scale:**
- Problem: Indexing and content search over large trees can spike memory/CPU; batching exists but unbounded paths are guarded by tests.
- Files: `crates/codegen/xai-codebase-graph/`, `crates/codegen/xai-grok-workspace/src/file_system/content.rs`, `index.rs`
- Improvement path: Enforce hard caps (`MAX_LIST_COLLECT`, `MAX_READ_BYTES` in walk module); prefer streaming APIs already present.

## Fragile Areas

**Permission policy stack:**
- Files: `crates/codegen/xai-grok-workspace/src/permission/*` (manager, resolution, auto_mode, shell_access, bash_command_splitting, claude_settings)
- Why fragile: Multiple policy sources (user, project, Claude-import, hub, ACP prompter) with order-sensitive resolution; bash AST edge cases.
- Safe modification: Add tests next to the classifier table for every new allow/block rule; run `auto_mode` and `shell_access` unit tests; never “simplify” wrapper stripping without golden corpus.
- Test coverage: Large unit suites exist; e2e coverage for combined auto_mode + sandbox + MCP is thinner.

**Hunk tracker mutation model:**
- Files: `crates/codegen/xai-hunk-tracker/src/actor/{mod,mutations,actions,hunks}.rs`, `diff.rs`
- Why fragile: Baseline vs current content invariants; accept/reject; external vs agent sources; TooLarge/Binary baselines; git restore interactions (tests document subtle regressions).
- Safe modification: Prefer actor commands; extend `actor/tests.rs` with multi-hunk + recompute scenarios; do not set baseline = entire file without line-level patching.
- Test coverage: Very strong unit coverage (~5.8k lines of tests); integration with shell fs_notify still the weak link.

**Session lifecycle / leader / multi-process:**
- Files:
  - `crates/codegen/xai-grok-shell/src/agent/mvp_agent/`
  - `crates/codegen/xai-grok-shell/src/leader/`
  - `crates/codegen/xai-grok-workspace/src/handle.rs` (drain metrics, termination grace)
- Why fragile: Concurrent sessions, drain/shutdown, protocol version negotiation, env-var overrides for K8s drain.
- Safe modification: Prefer single-command actor gates for unload; keep drain metrics; test protocol version rejection paths.
- Test coverage: Substantial shell tests; race windows documented as open (`TODO(PR-4)`).

**Config resolution megamodule:**
- Files: `crates/codegen/xai-grok-shell/src/agent/config.rs`, `crates/codegen/xai-grok-shell/src/util/config/`
- Why fragile: Layered env/TOML/remote/managed config with many feature flags; easy to break precedence.
- Safe modification: Add resolve tests for each new key; avoid inlining new structs into the 11k-line file — extract modules first.
- Test coverage: Heavy in-file unit tests; isolation test for auto_update overwrite (`tests/test_config_update_isolation.rs`).

**ACP / MCP protocol boundary:**
- Files: `crates/codegen/xai-grok-mcp/`, shell ACP session modules, pager `acp_handler`
- Why fragile: Non-exhaustive types, multiple transports, OAuth, managed MCP gateway naming.
- Safe modification: Always handle unknown variants; run MCP integration tests including SSE flood.

## Scaling Limits

**Workspace / monorepo export size:**
- Current capacity: ~1.3M lines of Rust under `crates/`; pager ~414 src files; shell ~394 src files; tools ~207.
- Limit: Developer machine full `cargo build --workspace` time and CI matrix cost; incremental compile suffers on god files.
- Scaling path: Keep extracting types crates; discourage new code in `xai-grok-shell`/`xai-grok-pager` root modules; prefer leaf crates under `crates/common` or `crates/codegen`.

**Permission classifier LLM calls:**
- Current capacity: Auto mode may invoke remote classifier for non-fast-path commands.
- Limit: Latency + rate limits under chatty shell agent loops; Unavailable → prompt (fail closed-ish).
- Scaling path: Expand safe fast-paths carefully with tests; cache verdicts per command shape.

**Search index remote sync:**
- Current capacity: GCS download path incomplete (remote timestamp always `0`; ad-hoc download URL).
- Limit: Stale/local bootstrap logic may skip needed downloads or redownload incorrectly.
- Scaling path: Implement HEAD metadata + proper GCS/proxy download (`crates/codegen/xai-grok-shell/src/session/storage/search_remote_sync.rs`).

**File size / binary handling in hunk tracker:**
- Current capacity: TooLarge / Binary baselines suppress diffs but keep file tracked (tested).
- Limit: Huge files still cost I/O on write/read paths before classification.
- Scaling path: Early size checks at tool layer; streaming writes.

## Dependencies at Risk

**agent-client-protocol (unstable feature):**
- Risk: Workspace pins `agent-client-protocol` with `features = ["unstable"]` and 0.10 non_exhaustive API churn.
- Impact: Widespread match/TODO churn across shell, MCP, pager, workspace permission.
- Migration plan: Track ACP releases; centralize adapters; drop unstable features when stable surface covers needs.

**rmcp / MCP streamable HTTP:**
- Risk: Reconnect and handshake behavior (SSE flood, legacy fast-fail regressions) tightly coupled to upstream.
- Impact: WARN spam, connection storms, flaky MCP tools.
- Migration plan: Keep `repro_sse_flood` and handshake regression tests; pin carefully; abstract transport behind `McpHttpClient`.

**nono (sandbox enforce feature):**
- Risk: Kernel-level sandbox API differences across Linux/macOS; musl builds disable enforce helpers path.
- Impact: Security posture differs by platform; silent no-op on unsupported.
- Migration plan: Integration tests per OS; document profile matrix; consider bubblewrap path already partially present (`is_inside_bwrap`).

**tree-sitter-bash:**
- Risk: Grammar failures force conservative Ask, but subtle mis-parses could allow dangerous commands if classified as plain words incorrectly.
- Impact: Permission bypass if a construct is treated as “plain.”
- Migration plan: Corpus of attack command lines in `shell_access` / `auto_mode` tests; grammar upgrades need full permission test run.

**Monorepo sync / third_party mermaid stack:**
- Risk: Tree is “synced periodically from the SpaceXAI monorepo”; third_party graph/mermaid crates may lag or diverge.
- Impact: Fixes may land monorepo-first; export consumers see delayed patches.
- Migration plan: Track sync cadence; avoid deep forks of `third_party/` without upstreaming.

## Missing Critical Features

**Atomic filesystem writes for workspace FS trait:**
- Problem: Documented TODO; agent edits can partially write.
- Blocks: Strong durability guarantees for crash recovery / multi-process tools.

**Server-side cancel for `/compact` (and related long tasks):**
- Problem: No `dispatch_cancel_command` / cancellation token on compaction spawn.
- Blocks: Safe cancel UX without losing subsequent prompts.

**Workspace proto / gRPC codegen:**
- Problem: Wire types claim source-of-truth status but no `.proto` emission yet.
- Blocks: Language-agnostic clients and schema evolution tooling.

**GCS search-index sync completeness:**
- Problem: Metadata check and proper download API not implemented.
- Blocks: Reliable remote index bootstrap for large monorepos.

**Aggregate `SessionActivity` for unload safety:**
- Problem: Background monitors/schedulers/subagents not fully counted in idle checks.
- Blocks: Correct detached-session lifecycle without work loss.

**Minimum-version restart via exec() deferred:**
- Problem: Floor-driven update installs then exits with “Run `grok`” instead of re-exec due to SIGABRT/broken-pipe risk.
- Files: `crates/codegen/xai-grok-update/src/minimum_version.rs`
- Blocks: Seamless forced-upgrade relaunch.

## Test Coverage Gaps

**Shell ↔ hunk-tracker fs_notify attribution path:**
- What's not tested: End-to-end that real tool writes call `record_agent_write` before notify (unit test intentionally demonstrates External classification for notify-only path).
- Files: `crates/codegen/xai-hunk-tracker/src/actor/tests.rs`; tool implementations under `crates/codegen/xai-grok-tools/`; shell forwarders
- Risk: Production still mis-attributes agent edits as External after unit-level “fixes” elsewhere.
- Priority: **High**

**Windows sandbox / path semantics:**
- What's not tested: README states Windows builds are best-effort and not currently tested from this tree; sandbox unsupported.
- Files: `README.md`, `crates/codegen/xai-grok-sandbox/`, path helpers using `dunce`
- Risk: Verbatim path bugs, missing sandbox, broken git clone destinations.
- Priority: **High** for Windows users; Medium for unix-primary development

**Session idle unload under background load:**
- What's not tested: Full race of `session_has_live_work` vs. monitor/scheduler/background subagent with unload.
- Files: `crates/codegen/xai-grok-shell/src/agent/mvp_agent/session_lifecycle.rs`
- Risk: Lost background work or unexpected KillOnDrop.
- Priority: **High**

**Compaction cancel + concurrent prompt:**
- What's not tested: Prompt issued after client cancel of compact vs. late history replace.
- Files: `crates/codegen/xai-grok-pager/src/app/dispatch/turn.rs`, compaction crates
- Risk: User messages silently discarded.
- Priority: **Medium**

**Crates with little or no dedicated tests:**
- What's not tested: `ptyctl-cli`, `xai-grok-models`, `xai-tracing-macros` (no `#[cfg(test)]` / tests dir found).
- Risk: Regressions in CLI codegen helpers, model catalog, tracing macros.
- Priority: **Low–Medium** (blast radius smaller than shell/pager)

**Permission + sandbox combination e2e:**
- What's not tested: Full matrix of auto_mode Allow + active Landlock/Seatbelt deny globs + bash redirect attacks.
- Files: `crates/codegen/xai-grok-workspace/src/permission/`, `crates/codegen/xai-grok-sandbox/tests/`
- Risk: Layered defenses fail open when only one layer is tested.
- Priority: **High**

**Remote search index download:**
- What's not tested: Real GCS HEAD/download with auth (implementation incomplete).
- Files: `crates/codegen/xai-grok-shell/src/session/storage/search_remote_sync.rs`
- Risk: Production fallback always local; silent stale indexes.
- Priority: **Medium**

---

*Concerns audit: 2026-07-16*
