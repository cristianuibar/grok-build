# Technology Stack — v1.1 Upstream Grok Build Parity

**Project:** bum
**Milestone:** v1.1 Upstream Grok Build parity
**Researched:** 2026-07-22
**Comparison target:** official `xai-org/grok-build` public commit `3af4d5d39897855bdcc74f23e690024a5dc05573`
**Upstream source revision:** `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899` (`SOURCE_REV`)
**bum tip inspected:** `128992c5a151e857c728bcbb054c2a8c7c47ce7a`
**Overall confidence:** MEDIUM — exact Git objects, manifests, lockfiles, source, commit metadata, and isolated builds were cross-checked; the mandated confidence classifier rates verified web search MEDIUM and Git/GitHub providers LOW, so claims based only on repository archaeology are explicitly identified as direct evidence rather than over-promoted.

## Executive Recommendation

Integrate upstream as a **content lineage**, not a Git merge. The two branches have no merge base, and their publication roots are not identical: bum begins at `c1b5909ec707c069f1d21a93917af044e71da0d7`, official upstream at `c68e39f60462f28d9be5e683d9cbe2c57b1a5027`, and those roots already differ by 179 files. A direct `git diff main upstream/main` is useful as a final-state audit but cannot distinguish initial publication skew, later official work, and bum v1.0 work.

Use a three-leg comparison:

1. `c1b5909..c68e39f` — publication-root skew that must be classified, not assumed upstream-new.
2. `c68e39f..3af4d5d` — the six official post-publication syncs that define the upstream parity payload.
3. `c1b5909..128992c` — bum's own overlay, used to find conflicts and preserve product contracts.

The stack does **not** need a language, runtime, TUI, ACP, MCP, protobuf, or HTTP framework replacement. Rust remains 1.92.0 / edition 2024; Tokio, ratatui, crossterm, ACP 0.10.4, reqwest 0.12, rmcp 2.1, tonic/prost 0.14, and Cargo lockfile format 4 remain unchanged. The substantive stack additions are:

- a pinned `async-openai` 0.33.1 fork for a distinct `ReasoningEffort::Max` wire variant;
- `rhai` 1.25.1 plus a new `xai-workflow` crate;
- optional `dhat` 0.3.3 heap-soak support;
- several small target/feature/dev-test dependency changes;
- new workflow, model-provider, auth-provider, worktree-GC, configuration, and schema surfaces.

Adopt the upstream mechanics, security fixes, and feature infrastructure, but adapt every overlap through bum's invariants. In particular: keep the binary `bum`, `BUM_HOME` / `~/.bum`, dual xAI + Codex OAuth, explicit mixed-model provider binding, cross-provider subagents, and hard-off stock updater/product telemetry. Do not replace bum's model router with upstream's similarly named custom gateway configuration.

## Pinned Baseline and Comparison Method

### Exact revisions

| Role | Commit | Date | Notes |
|---|---|---:|---|
| bum publication root | `c1b5909ec707c069f1d21a93917af044e71da0d7` | 2026-07-15 | `Publish harness and TUI open-source` |
| upstream publication root | `c68e39f60462f28d9be5e683d9cbe2c57b1a5027` | 2026-07-16 | Same subject, different tree and unrelated history |
| official sync 1 | `8adf9013a0929e5c7f1d4e849492d2387837a28d` | 2026-07-16 | version 0.2.101, auth/config/security fixes |
| official sync 2 | `98c3b2438aa922fbbe6178a5c0a4c48f85edc8ce` | 2026-07-17 | version 0.2.102, credentials/config/security/test changes |
| official sync 3 | `7cfcb20d2b50b0d18801a6c0af2e401c0e060894` | 2026-07-18 | version 0.2.105, Grok 4.5, `dhat`, config/security |
| official sync 4 | `ba76b0a683fa52e4e60685017b85905451be17bc` | 2026-07-19 | version 0.2.106, ACP session methods/scheduler |
| official sync 5 | `a881e6703f46b01d8c7d4a5437683546df30449d` | 2026-07-20 | named auth providers, signed-config nonce |
| pinned official tip | `3af4d5d39897855bdcc74f23e690024a5dc05573` | 2026-07-21 | version 0.2.109, workflows/model providers/Max/worktree GC |
| source monorepo revision | `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899` | — | exact content of upstream `SOURCE_REV` |
| bum tip inspected | `128992c5a151e857c728bcbb054c2a8c7c47ce7a` | 2026-07-22 | v1.1 milestone start |

`git ls-remote upstream refs/heads/main` returned the same `3af4d5d...` SHA, and GitHub commit metadata independently matched it. Record **both** the public commit and `SOURCE_REV` in the parity ledger; only the public commit is fetchable from this repository.

### Reproducible commands

```bash
# Refresh and pin before implementation begins.
git fetch upstream main
git rev-parse main upstream/main
git ls-remote upstream refs/heads/main
git show -s --format='%H %P %aI %s' upstream/main
git show upstream/main:SOURCE_REV

# This intentionally exits 1 and prints nothing: no shared ancestor.
git merge-base main upstream/main

# Three comparison legs. --no-renames is useful for a stable path ledger;
# repeat with --find-renames for human review.
git diff --no-renames --name-status c1b5909ec707c069f1d21a93917af044e71da0d7 \
  c68e39f60462f28d9be5e683d9cbe2c57b1a5027
git diff --no-renames --name-status c68e39f60462f28d9be5e683d9cbe2c57b1a5027 \
  3af4d5d39897855bdcc74f23e690024a5dc05573
git diff --no-renames --name-status c1b5909ec707c069f1d21a93917af044e71da0d7 \
  128992c5a151e857c728bcbb054c2a8c7c47ce7a

# Final-state audit only; do not treat every line as an upstream change.
git diff --stat main 3af4d5d39897855bdcc74f23e690024a5dc05573
git diff --name-status main 3af4d5d39897855bdcc74f23e690024a5dc05573
```

Direct evidence: upstream root-to-tip is 872 files changed (133 added, 20 deleted, 718 modified, one rename); 220 of the 358 non-planning paths changed by bum also appear in the upstream post-root payload. Twenty-four stack/config/schema-related paths overlap, including `Cargo.lock`, model catalog, pager/shell manifests, config parsing, sampler config, and managed config. This is too conflict-heavy for bulk checkout or patch application.

## Recommended Stack

### Core framework — unchanged

| Technology | Pinned/current version | Decision | Rationale |
|---|---:|---|---|
| Rust | 1.92.0, edition 2024 | Keep | `rust-toolchain.toml` is byte-identical in the relevant upstream lineage; no toolchain migration exists. |
| Cargo | resolver 2, lockfile v4 | Keep | Upstream adds one member and lock entries, not a package-manager change. |
| Tokio | 1 (`full` workspace feature) | Keep | Workflow uses existing Tokio channels/runtime. |
| ratatui / crossterm | 0.29 / 0.28 | Keep | No version change. |
| ACP | `agent-client-protocol` 0.10.4 (`unstable`) | Keep | Wire behavior expands through x.ai extension methods, not a dependency bump. |
| HTTP | reqwest 0.12; reqwest 0.13 only in MCP | Keep | No root version change. |
| MCP | rmcp 2.1 | Keep | OAuth behavior changes, dependency version does not. |
| Protobuf/gRPC | prost/tonic 0.14 | Keep | The checked-in `grok-tools.proto` blob is identical at bum root, upstream root, and upstream tip. |
| Config/data | serde, serde_json, TOML 0.9 | Keep | New typed surfaces use the existing stack. |

### Dependency and manifest changes to adopt

| Change | Exact resolved version/source | Purpose | Integration decision |
|---|---|---|---|
| `[patch.crates-io] async-openai` | git `https://github.com/our-forks/async-openai.git`, rev `95b52ebdedf42143083cf3d6f0e0be7c84e9c808`; resolves 0.33.1 | Adds exactly `ReasoningEffort::Max` | Adopt first, with lockfile. The fork commit is a one-line enum addition. Then port upstream's distinct `Max` mappings without deleting bum's Codex catalog clamping/summary behavior. |
| `rhai` | manifest `1.25` + `serde`; lock 1.25.1 | Embedded workflow scripts | Adopt only with `xai-workflow`; do not add a second script engine. |
| `xai-workflow` | new in-tree crate 0.1.0 | Rhai engine, host channel, journal/replay, metadata validation | Adopt as upstream parity infrastructure. Keep workflows off by default as upstream does; this does not authorize a separate bum workflow redesign in v1.1. |
| `dhat` | manifest 0.3; lock 0.3.3 | Session lifecycle heap soak | Adopt as optional shell feature `dhat-heap`, not a default production dependency path. |
| `shlex` | existing workspace dependency | Pager external-editor/doctor shell parsing | Add pager dependency. |
| `xai-tty-utils` | existing workspace crate | Agent terminal/environment behavior | Add agent dependency. |
| `sha2`, `thiserror`, `url` | existing workspace dependencies | Sandbox network/path policy | Add to sandbox manifest with source changes. |
| Windows `Win32_Storage_FileSystem` | existing `windows` crate feature | File-system safety behavior | Add only on Windows target dependency. |
| Unix `libc` for fast-worktree | existing workspace dependency | macOS process CWD scan and Unix PID liveness | Move from Linux-only to Unix production dependency as upstream does. |

Official lock consequences include the async-openai git source, `rhai`, `rhai_codegen`, `smartstring`, `thin-vec`, `const-random`, `tiny-keccak`, and optional `dhat` support packages. Do not hand-edit `Cargo.lock`.

### Build and test topology changes

1. Add `crates/codegen/xai-workflow` to workspace members.
2. Add upstream `test-support` features in `xai-grok-env`, memory, pager-render, pager, shell-base, and workspace; wire test dev-dependencies accordingly.
3. Preserve bum's extra `xai-grok-config/test-support` seam if its signed-managed-config tests still require it. Upstream removed that use; deleting it blindly would regress bum-only tests.
4. Replace the monolithic pager `pty_e2e` target with upstream's coherent families: `pty_e2e_smoke`, `queue`, `scroll_selection`, `minimal`, `config_ui`, `shell_tools`, `persistence`, and `clipboard`; add `doctor_early_dispatch`.
5. Add `[lints] workspace = true` to pager as upstream does.
6. Keep `PAGER_BINARY` pointing to the built **bum** artifact in Cargo-driven PTY tests. Upstream comments assume `xai-grok-pager`; adapt the test harness rather than rename the binary.
7. Root `Cargo.toml` declares itself generated, but the public repository contains no generator. Treat the pinned root diff as authoritative output, update per-crate manifests first, then make the minimal root member/dependency/patch changes and document them in the parity ledger.

### Version policy

Official user-facing crates progressed `0.1.220-alpha.4 → 0.2.101 → 0.2.102 → 0.2.105 → 0.2.106 → 0.2.109`. At the pinned tip, pager-bin, pager, shell, and `xai-grok-version` are 0.2.109. Not every crate was mass-bumped (for example tools/workspace can retain their prior package versions), so do not globally rewrite manifest versions.

Recommendation: align the four lockstepped source-version crates to the 0.2.109 upstream baseline, but preserve the user-facing bum release identity in `bum version` and release metadata. If bum requires a semver-distinct package version, use a documented bum suffix rather than claiming to be the stock binary; never restore upstream's binary name/default-run merely to match its version.

## Protocol and Schema Changes

### Confirmed protocol facts

| Surface | Baseline → target | Integration |
|---|---|---|
| Checked-in protobuf | No public `.proto` change; blob `60ada1a810baa78f4385e723a81d6f590a7cb9c3` at all inspected refs | Do not regenerate or invent protobuf fields. `cargo test -p xai-grok-tools-api` remains the gate. |
| Tool protocol | `PROTOCOL_VERSION` remains `1.0.0` | Keep version; port behavioral/security changes. |
| Leader protocol | `LEADER_PROTOCOL_VERSION` remains `1`; diff in `leader/protocol.rs` is formatting-only | Keep version; validate mixed old/new process rejection and capability tests. |
| ACP crate | stays 0.10.4 | Port new x.ai methods and update conversion/session tests. |
| Tool metadata schema | adds open-set `ToolKind = workflow` | Adopt schema and Rust enum/conversion together. Unknown values must remain tolerated. |
| Workflow metadata | strict Rhai first statement `let meta = #{...};`, deny unknown fields, kebab-case name, bounded strings/phases | Adopt engine + metadata validation atomically. |
| Workflow structured output | self-contained JSON Schema, external refs disabled, schema/output/regex size limits | Adopt with workflow host; preserve limits as security boundaries. |

New ACP extension surfaces observed in source include `x.ai/session/state`, `x.ai/session/import`, `x.ai/session/usage`, `x.ai/workflows/list`, `x.ai/internal/reload_workflows`, `x.ai/mcp/setup`, and `x.ai/auth/cancel`. These are additions on the existing ACP extension mechanism, not evidence of an ACP dependency upgrade.

The official sync message says `Proto: ClientToolResult and ChatConfig client-side tools`, but neither identifier appears in the exported public tree and the sole checked-in proto is unchanged. This is evidence of a monorepo-side/proxy contract not exported in this public snapshot. **Do not fabricate an implementation.** Record it as a parity gap requiring a later upstream snapshot or authoritative schema; verify observable client-tool behavior through existing public types/tests only.

## Configuration Changes and bum Adaptation

### Adopted configuration surfaces

| Surface | Upstream behavior | Required bum adaptation |
|---|---|---|
| `[workflows] enabled` / `GROK_WORKFLOWS` | Off by default; selects host workflow driver and discovers `.grok/workflows/*.rhai` | Keep off by default. Translate user-home scope to `~/.bum`; decide project compatibility path separately rather than global search/replace. Preserve provider selection for every spawned agent. |
| `[model_providers.<id>]` | Shared endpoint/backend/header/context/auth defaults for custom models | Integrate below bum's explicit xAI/Codex routing. Do not confuse this gateway-template ID with bum's model `provider` identity. A custom endpoint must never inherit xAI/Codex session credentials accidentally. |
| `[auth_provider.<name>]` | Trusted-config command mints rotating per-model bearer, memory-only cache | Do not replace either OAuth lifecycle. If exposed, treat as custom BYOK helper only; project config cannot define commands, first-party credentials are stripped, failure must not fall back to a bum session token. Arbitrary new built-in providers remain out of scope. |
| `[worktree.auto_gc]` / `GROK_WORKTREE_AUTO_GC*` | Enabled policy, age/kind TTL, dry-run, rebuild, platform-specific guards | Adopt resolver and safety guards together. Persist stamps under bum home. Start validation with dry-run and manual worktrees set to `never`. |
| `login_shell_capture` | Replaces `persistent_local_shell` remote setting | Migrate resolver/tests as a semantic rename; accept old persisted/managed input only if compatibility requires it. |
| `scheduler_background_loops` / `GROK_SCHEDULER_BACKGROUND_LOOPS` | Controls background loop turns | Adopt with durable scheduler lifecycle/version changes, not as a standalone flag. |
| `compaction_tool_choice` / `GROK_COMPACTION_TOOL_CHOICE` | Configures compaction tool-choice behavior | Merge without removing bum's Codex reasoning summary/effort fields. Test both providers. |
| `[ui].combine_queued_prompts` | Batches follow-ups into one model turn | Adopt UI + ACP queue + prompt queue changes together. |
| `[toolset.web_fetch].allow_local` / `GROK_WEB_FETCH_ALLOW_LOCAL` | Explicit loopback only; private/link-local/metadata remain blocked | Adopt fail-closed network policy and redirect checks as one security unit. |
| Managed text transactions | Marker-owned edits, stale-plan detection, locking, backup, validation, atomic publish | Adopt for doctor/fix flows, but change artifact labels/paths from grok to bum where user-visible or stored under product home. |
| Model catalog | Stock default becomes Grok 4.5 with new model metadata | Merge Grok 4.5 as `provider = "xai"`; do not replace the mixed catalog or delete GPT-5.6 Sol/Terra/Luna. Default choice remains a bum product decision. |
| Reasoning effort | `max` becomes distinct above `xhigh` | Adopt distinct enum/wire value. Preserve bum's per-model effort menus, clamping, and Codex `reasoning.summary` omission behavior that are absent from stock upstream. |

Other public config-type additions include `workflows_enabled`, `worktree_auto_gc`, `ssh_wrap`, and new UI/diagnostic fields. `persistent_local_shell` is the one public field removed in the root-to-tip config-type comparison. New environment tokens also include `GROK_SESSION_REGISTRY`, `GROK_WEB_FETCH_ALLOW_LOCAL`, worktree auto-GC controls, workflow control, clipboard OSC52 kill switch, auth-provider handback variables, and test-only variables. Rename user-facing `GROK_*` controls to `BUM_*` only where bum's existing identity policy requires it; preserve compatibility aliases only deliberately and test precedence.

### Auth/provider overlap

Upstream adds substantial useful auth hardening: single-flight interactive auth, owner-only `auth.json` and corrupt backups, safer missing-field default for coding-data retention, named per-model token helpers, API-key resolution fixes, and MCP credential hardening. Port the security and concurrency behavior into bum's multi-slot store. Do **not** copy upstream's single-principal assumptions over bum's xAI/Codex slots.

Required auth regression matrix after adaptation:

- xAI login/refresh/logout does not mutate Codex credentials;
- Codex login/refresh/logout does not mutate xAI credentials;
- owner-only permission tightening covers all bum auth slots and backups;
- named helper credentials never enter `auth.json` and never receive first-party secrets;
- model switch resolves the correct provider after config reload;
- subagents inherit the chosen child model/provider, not the parent credential;
- 401 refresh is single-flight per credential slot/provider and fails closed.

## Generated-File Implications

| Artifact | Status | Rule |
|---|---|---|
| Root `Cargo.toml` | Header says auto-generated; no public generator found | Apply minimal authoritative root changes after per-crate manifests; record exact diff and do not refactor formatting. |
| `Cargo.lock` | Cargo-generated, format 4 | Regenerate via Cargo after all manifest changes. Verify async-openai git SHA, Rhai 1.25.1, dhat 0.3.3, and `xai-workflow`. |
| Protobuf Rust output | Generated into `OUT_DIR` by `xai-proto-build` | No checked-in proto delta; rebuild/tests suffice. Do not check in generated Rust. |
| `prompt_encrypted.rs` | Existing auto-generated source; not part of stack delta | Do not hand-edit unless a prompt-source change explicitly requires regeneration. |
| JSON schemas/model catalog | Checked-in source artifacts | Edit/merge intentionally, validate JSON, and update Rust conversion tests in the same phase. |
| `SOURCE_REV` | New upstream provenance file | Add a bum parity ledger or retained `SOURCE_REV` with both public and monorepo revisions; never let it imply bum's whole tree equals stock upstream after adaptations. |

Pinned upstream checksums for audit:

- `Cargo.lock`: `6b2eb2c1633bdf2dcfd030208c19a36c52df862522cb72c707982b0bb365e5b8`
- `Cargo.toml`: `71965098017b6d6c4e228454beab58d8854257afbc648f65190c9fd7891f53bb`
- `rust-toolchain.toml`: `8632e4f532c10ff7728fdef1bb73f9f391577ca2b23ed133272c714bd371e696`
- `default_models.json`: `620cfe2b3e4ba8d5e74560ce22a4d42f0f5418a2fb5f192bd38c35b234f3c148`

## Explicit Items Not to Adopt Verbatim

| Upstream item | Decision | Reason |
|---|---|---|
| binary `xai-grok-pager`, default-run stock name | Reject | User-facing product is `bum`. |
| `GROK_HOME`, `~/.grok`, managed `bin/grok` | Reject in product-home paths | Must remain `BUM_HOME`, `~/.bum`, managed `bin/bum`. |
| stock `default_models.json` replacement | Reject | Would erase Codex models and provider bindings. Merge Grok 4.5 instead. |
| stock single-provider auth model | Reject | Dual OAuth and selective lifecycle are validated bum capabilities. |
| use `[model_providers]` as bum's provider router | Reject | Upstream uses it for reusable custom gateway settings; bum's provider binding is a security/routing identity. |
| stock updater healing and `agent → grok` entrypoint reconciliation | Reject | bum hard-disables stock x.ai update checks and must not create/repair stock Grok entrypoints. |
| product analytics, Mixpanel, Sentry, internal OTLP phone-home | Reject | Quiet-fork invariant. Keep local tracing and explicit user-owned external OTEL only if it remains opt-in and content-gated. |
| arbitrary third-party providers as first-class shipped choices | Defer | Milestone retains xAI + Codex scope. Named helper/config infrastructure may be ported without expanding the supported catalog. |
| new bespoke bum workflow design | Defer | Import upstream's disabled-by-default workflow engine for parity; custom bum workflow product work remains post-v1.1. |
| unexported `ClientToolResult` / `ChatConfig` proto guess | Reject | No public schema or source implementation exists at the pinned snapshot. |
| blind package-version mass bump | Reject | Only the official lockstep product crates reached 0.2.109. |

## Migration Order

1. **Provenance and inventory** — freeze `3af4d5d...`, `SOURCE_REV`, three comparison legs, per-path disposition (`adopt`, `adapt`, `superseded`, `exclude`, `unavailable`). This prevents later ambiguity.
2. **Manifest/test scaffolding** — port per-crate feature/target/test changes, add `xai-workflow`, root member, Rhai/dhat declarations, async-openai patch, then regenerate the lockfile. Keep `bum` binary/default-run.
3. **Security/config primitives** — owner-only auth/MCP files, signed managed-cache changes, managed-text transactions, sandbox/network policy, and tolerant config types before exposing new controls.
4. **Provider/model/sampler layer** — distinct `Max`, Grok 4.5 catalog merge, upstream model/auth provider config adapted beneath bum routing, and dual-provider regression tests. This is the highest conflict area.
5. **Protocol and service behavior** — ACP extensions, session state/import/usage, scheduler durability, workflow tool metadata/schema, tool conversions. Keep protocol version constants unchanged unless an actual wire break is proven.
6. **Workflow/worktree feature integration** — enable new crate/host/store/UI paths behind upstream defaults; verify cross-provider workflow agents and worktree GC dry-run before normal operation.
7. **Pager/build topology** — UI settings, doctor, queue batching, PTY family split, `PAGER_BINARY=.../bum`, screenshots/snapshots.
8. **Identity/privacy reconciliation** — static sweep for stock path/binary/update/telemetry regressions, then full focused build and live dual-provider smoke tests.

The ordering is intentional: generated dependency state must stabilize before code ports; parsers/security boundaries must exist before consumers; provider conflicts must be resolved before workflows or subagents can safely spawn models; identity/privacy gates run again after every stock feature is present.

## Validation Commands

### Baseline validation already performed

An archive of `3af4d5d...` (outside the bum tree) passed:

```bash
cargo metadata --locked --no-deps --format-version 1
cargo check -p xai-workflow --locked
cargo test -p xai-workflow --locked       # 54 passed
cargo check -p xai-grok-shell --locked
```

The archive reports 80 workspace packages versus bum's current 79. Current bum also passes `cargo metadata --no-deps` and resolves registry `async-openai` 0.33.1 before the patch.

### Required implementation gates

```bash
# Dependency and metadata integrity
cargo metadata --locked --format-version 1 >/tmp/metadata.json
cargo tree -p xai-grok-shell -i async-openai --locked
cargo tree -p xai-workflow --locked

git grep -n '95b52ebdedf42143083cf3d6f0e0be7c84e9c808' -- Cargo.toml Cargo.lock
git diff --check
cargo fmt --all --check

# New stack and schemas
cargo check -p xai-workflow --locked
cargo test -p xai-workflow --locked
cargo test -p xai-grok-tools-api --locked
cargo test -p xai-grok-config --locked
cargo test -p xai-grok-config-types --locked
cargo test -p xai-grok-sandbox --locked
cargo test -p xai-grok-models --locked

# Provider/sampler/shell integration
cargo check -p xai-grok-sampler -p xai-grok-shell --locked
cargo test -p xai-grok-sampling-types --locked
cargo test -p xai-grok-sampler --locked
cargo test -p xai-grok-shell --lib --locked
cargo clippy -p xai-workflow -p xai-grok-sampler -p xai-grok-shell --all-targets --locked

# Product composition and pager
cargo build -p xai-grok-pager-bin --bin bum --locked
cargo test -p xai-grok-pager --lib --locked
PAGER_BINARY="$PWD/target/debug/bum" cargo test -p xai-grok-pager --test pty_e2e_smoke --locked -- --ignored

# Catalog/schema syntax
python3 -m json.tool crates/codegen/xai-grok-models/default_models.json >/dev/null
python3 -m json.tool crates/codegen/xai-grok-tools/schema/tool_meta.schema.json >/dev/null

# Identity/privacy static gates (review every hit, not just count them)
git grep -n -E '~/.grok|GROK_HOME|bin/grok|name = "xai-grok-pager"' -- crates README.md
git grep -n -E 'api\.mixpanel\.com|SENTRY_DSN|auto_update\(' -- crates/codegen/xai-grok-pager-bin crates/codegen/xai-grok-telemetry crates/codegen/xai-grok-update
```

Run hermetic product probes with a temporary `BUM_HOME`, then repeat the established live xAI↔Codex switch and cross-provider subagent matrix. Workflow tests must include xAI parent/Codex child and Codex parent/xAI child because the new engine introduces another model-spawn path.

## Evidence vs Inference

### Directly evidenced

- Exact tips, absence of merge base, source revision, commit sequence, file counts, versions, dependencies, lock resolutions, unchanged toolchain, unchanged public proto blob, protocol constants, schema diff, configuration structs/docs, and isolated successful builds/tests come from exact Git objects and commands above.
- The async-openai fork commit adds exactly one line: `ReasoningEffort::Max`; upstream lock resolves that SHA as async-openai 0.33.1.
- Rhai resolves to 1.25.1 with `serde`; official crate feature metadata shows that this enables serde support in Rhai and its collection/string dependencies.

### Inference requiring implementation verification

- The safest integration shape is semantic porting with a parity ledger rather than a synthetic merge. This follows from unrelated histories, 179-file root skew, and 220 overlapping changed paths.
- Upstream model-provider defaults should sit below bum's provider identity. This is an architectural recommendation based on the two different meanings of “provider,” not an upstream guarantee.
- Importing the workflow crate while leaving it disabled satisfies upstream stack parity without advancing bum's separate custom-workflow product scope. Roadmapping should make that boundary explicit.
- The sync-message-only `ClientToolResult` / `ChatConfig` work may live in a non-exported monorepo component. Its location cannot be established from the pinned public tree.

## Sources

- Official repository: https://github.com/xai-org/grok-build
- Pinned official commit: https://github.com/xai-org/grok-build/commit/3af4d5d39897855bdcc74f23e690024a5dc05573
- Pinned async-openai fork commit: https://github.com/our-forks/async-openai/commit/95b52ebdedf42143083cf3d6f0e0be7c84e9c808
- Rhai 1.25.1 feature metadata: https://docs.rs/crate/rhai/1.25.1/features
- Repository evidence: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, per-crate manifests, `SOURCE_REV`, config types/docs, schemas, and commit history at the SHAs listed above.

---

*Recommendation: pin the public and source revisions, port stack changes in dependency order, and treat bum identity/provider/privacy behavior as adaptation constraints—not conflicts to resolve in upstream's favor.*
