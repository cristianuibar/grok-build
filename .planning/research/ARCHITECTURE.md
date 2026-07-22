# Architecture Research: Upstream Synchronization for bum v1.1

**Domain:** Controlled synchronization of a heavily customized Rust application fork
**Researched:** 2026-07-22
**Recommendation:** Rebase the *product architecture*, not the existing Git branch: build the integrated tree from the pinned upstream target, port bum's explicit contract overlay onto it, validate by component and invariant, then create one intentional ancestry-bridge commit after the tree is proven.
**Overall confidence:** HIGH for repository topology and component mapping; MEDIUM for feature-level intent because public upstream history consists of monorepo snapshots rather than feature commits.

## Executive Decision

Do **not** run a normal `git merge main upstream/main`, and do not treat `--allow-unrelated-histories` as a conflict-resolution strategy. The two branches have no merge base. Git reports independent roots:

- bum root: `c1b5909ec707c069f1d21a93917af044e71da0d7`
- upstream root: `c68e39f60462f28d9be5e683d9cbe2c57b1a5027`
- pinned upstream target inspected: `3af4d5d39897855bdcc74f23e690024a5dc05573`
- upstream monorepo source recorded by `SOURCE_REV`: `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`
- `git merge-base main upstream/main` returns no commit.

The similarly named public roots are also not identical snapshots: their direct tree comparison changes 179 files (`+8,492/-1,184`). Therefore, declaring either root to be the other's parent would assert ancestry that the repository cannot prove.

The safest architecture is an **upstream base plus downstream contract overlay**:

```text
upstream/main @ pinned SHA
        │
        ├── verified upstream tree (all features/fixes present by default)
        │
        ├── bum identity + storage overlay
        ├── bum dual-auth + provider-routing overlay
        ├── bum Codex wire + cross-provider agent overlay
        ├── bum privacy/no-phone-home overlay
        └── adapted docs/tests/packaging overlay
                │
                ▼
       verified integration tree
                │
                └── intentional two-parent ancestry bridge
                    (parent 1: upstream-derived integration line;
                     parent 2: previous bum main; tree unchanged)
```

This is preferable to applying 127,000 lines of upstream snapshot deltas onto today's bum tree because it makes upstream inclusion the default and downstream divergence the reviewable exception. It is also preferable to blindly replaying all 478 local commits: planning commits and superseded implementation steps are not a durable product patch stack. Port only the behavior required by bum's invariant ledger, using local commits as provenance and implementation evidence.

After the one-time bridge, future synchronization becomes conventional: the previous pinned upstream SHA is a real ancestor of the integrated line, so the next upstream snapshot can be merged or rebased in an isolated integration branch with a real three-way base. The ancestry bridge must happen **only after** the resolved tree and ledger pass all gates; it is a history-recording operation, not a way to obtain the resolved content.

## Observed Repository Topology

| Fact | Observation | Architectural consequence |
|---|---|---|
| Shared ancestry | None | Ordinary merge/rebase semantics are unavailable for this first sync. |
| Local history | 479 commits at inspection time | Contains product work plus extensive planning/verification history; not a clean downstream patch queue. |
| Upstream history | 7 public commits | Each post-root commit is a large “Synced from monorepo” snapshot, not a feature-sized change. |
| Upstream root → target | 872 files, `+127,391/-54,015` | Snapshot-by-snapshot cherry-picking still creates very large, mixed-domain changes. |
| Current bum ↔ target | 1,345 paths overall, `+138,194/-151,932` | Endpoint diff is useful for inventory, not as a patch to apply wholesale. |
| Root snapshot skew | 179 files, `+8,492/-1,184` | The same root subject does not establish content identity or parentage. |
| New crate | `crates/codegen/xai-workflow` | Must be integrated as a new leaf/domain crate before shell and pager workflow consumers. |
| Generated root manifest | `Cargo.toml` says it is auto-generated | Root dependency/member changes are generated export artifacts; do not make it the first manual integration surface. |
| Export provenance | `SOURCE_REV` exists upstream | Preserve both the public export SHA and monorepo `SOURCE_REV` in the sync ledger. |

The upstream target is internally coherent: its generated root manifest, per-crate manifests, and lockfile were exported together. Starting the integration branch at that commit preserves this coherence. Starting from current bum and copying directories incrementally would repeatedly break the dependency graph and makes it easy to miss deleted or moved files.

## Recommended Durable Architecture

### 1. Upstream baseline layer

A read-only baseline ref pins the exact public export:

```text
refs/remotes/upstream/main          moving discovery ref
refs/tags/bum-upstream/<date-or-rev> immutable reviewed target
integration/upstream-<short-sha>     temporary working branch from pinned target
```

The baseline is the source of truth for applicable upstream content. It must be validated standalone before any bum overlay is added. Record:

- upstream repository URL;
- public export commit SHA and tree SHA;
- upstream `SOURCE_REV` monorepo SHA;
- fetch timestamp;
- prior reviewed upstream SHA (none for this first lineage);
- Cargo.lock checksum and toolchain version;
- release/changelog range represented by the snapshot.

Do not use a moving `upstream/main` directly in plans or verification evidence after the pin.

### 2. Bum contract overlay layer

Treat the following as protected architecture contracts, not scattered textual edits:

| Contract | Owning boundaries | Required outcome |
|---|---|---|
| Product identity | pager-bin, pager chrome/docs, version/update helpers | Binary and product are `bum`; no user-facing regression to stock `grok`. |
| Home isolation | `xai-grok-config`, `xai-grok-paths`, shell home helpers, test support | `BUM_HOME` / `~/.bum` remains authoritative; no stock home credential/session sharing. |
| Dual authentication | `xai-grok-shell/src/auth`, composition-root login/logout/status | Independent xAI and Codex OAuth slots, refresh, selective logout, and corruption-safe sibling preservation. |
| Provider identity and routing | model catalog, agent config, sampler reconstruction | Every built-in model has an explicit inference provider; model selection chooses endpoint and credential per request. |
| Codex wire behavior | sampling types, sampler, shell turn reconstruction | Trusted Codex Responses request shape, account attribution, effort handling, encrypted-reasoning cleanup, and tolerant streaming remain intact. |
| Cross-provider children | subagent coordinator and `Task` backend | Child model/provider is resolved independently of parent; missing provider fails before pending work is created. |
| Quiet operation | update, telemetry, feedback, composition root | Stock update and product telemetry paths remain hard-off/default-off as specified; no silent reactivation through new upstream entry points. |

Keep this overlay small in *conceptual ownership*, even where its implementation touches many files. Every final difference from the pinned upstream tree must map to one of these contracts, an upstream adaptation, or an explicit exclusion.

### 3. Synchronization control plane

Add a durable, machine-readable ledger plus a human summary. Recommended logical artifacts (exact paths may be chosen during planning):

```text
sync/
├── upstream.toml                 # current/previous public SHA, tree, SOURCE_REV
├── ledger.toml                   # change-unit dispositions and evidence
├── invariants.toml               # protected bum contracts + validation commands
├── path-map.toml                 # moves/splits and ownership boundaries
├── allowlist.toml                # expected final upstream↔bum tree differences
└── reports/<target-sha>/         # generated inventory/coverage reports
```

A ledger entry should be addressable by a stable change-unit ID rather than only a pathname:

```toml
[[change]]
id = "UP-3af4d5-workflow-engine"
source_public_from = "a881e670..."
source_public_to = "3af4d5d3..."
source_monorepo = "0f4d7c91..."
paths = ["crates/codegen/xai-workflow/**", "crates/codegen/xai-grok-shell/src/session/workflow/**"]
disposition = "adapted" # imported | adapted | excluded | superseded | pending
reason = "Route workflow children through bum provider-aware subagent preflight"
bum_commits = ["<integration commit sha>"]
contract_ids = ["AGENT-CROSS-PROVIDER", "AUTH-PROVIDER-GATE"]
validation = ["cargo test -p xai-workflow", "<workflow cross-provider gate>"]
```

Required dispositions:

- **imported** — upstream behavior retained without bum-specific semantic changes;
- **adapted** — upstream behavior retained but implementation changed to preserve a bum contract;
- **excluded** — upstream behavior intentionally omitted, with user/product/security rationale;
- **superseded** — bum already has equal or stronger behavior, with evidence;
- **pending** — temporary only; release gate requires zero pending entries.

Coverage must work in two directions:

1. every upstream changed path/change cluster in the reviewed range has a disposition;
2. every final difference between the integrated tree and pinned upstream has a contract/exclusion owner.

A prose checklist alone is insufficient for 872 upstream-touched paths.

## Component Mapping

### Stable one-to-one boundaries

| Upstream component | bum component | Integration policy |
|---|---|---|
| `xai-grok-pager-bin` | Same crate, ships `bum` | Import upstream routing/doctor/wrap changes; reapply binary, version, login-provider, update, and privacy composition. Keep it wiring-only. |
| `xai-grok-pager` | Same TUI crate | Import Action→Dispatch→Effect changes and new views; adapt chrome, auth state, model badges, and commands. Never bypass dispatch purity. |
| `xai-grok-pager-render` / minimal / PTY harness | Same crates | Import terminal, clipboard, tmux, minimal-mode and harness changes as a coherent presentation cluster. Update expected product text, not behavior, in fixtures. |
| `xai-grok-shell` | Same runtime crate | Primary conflict surface. Integrate upstream session/auth/config/workflow changes, then port provider and Codex contracts into the new module layout. |
| `xai-chat-state` | Same actor crate | Import persistence/compaction/request-building changes before shell session adaptation. Preserve bum's provider-switch cleanup semantics at shell boundaries. |
| `xai-grok-sampler` / sampling types | Same transport crates | Import upstream stream/error/client changes first; then reapply Codex wire profile and tolerant SSE behavior with provider-specific tests. |
| `xai-grok-tools` | Same registry/implementation crate | Import upstream tool, scheduler, notification and workflow tools; merge bum Task model/effort schema rather than replacing it. |
| `xai-grok-workspace` | Same host crate | Import permission, execution-risk, FS/worktree changes as security-sensitive foundations. |
| config/hooks/MCP/sandbox/plugin crates | Same leaf/domain crates | Integrate before shell/pager consumers; preserve bum home/privacy only at their explicit seams. |

### New upstream boundaries

| New boundary | Responsibility | Consumers | Bum adaptation |
|---|---|---|---|
| `xai-workflow` | Rhai workflow engine, metadata validation, host request protocol, budget, journal, outcomes | shell workflow manager | New crate. Keep provider-agnostic; do not put OAuth/routing in the engine. |
| `shell/session/workflow/*` | Per-session run manager, host service, registry, store, tracker, notifications | ACP session, tools, pager | New orchestration layer. Its `SpawnAgent` conversion must use bum's existing child model/provider preflight. |
| `shell/session/workflows/deep_research.rhai` | Built-in workflow definition | workflow registry | New data artifact. Rebrand user paths and validate models are not xAI-only assumptions. |
| `tools/.../workflow` | Model-callable workflow launch schema and handle | shell session resource bridge | New tool. Preserve depth limits, sandbox/permission behavior, and provider-independent child model resolution. |
| pager workflow ingest/blocks/views/overlay | Workflow progress presentation and controls | pager ACP handler, tasks pane | New presentation path. Keep ACP updates and dispatch/effects boundaries; no workflow runtime state in the TUI. |
| `shell/auth/auth_provider.rs` | Command-backed rotating credential helper for custom models | model config and turn auth recovery | New generic mechanism. It complements, but must not replace or absorb, bum's built-in xAI/Codex OAuth store. |
| `shell/agent/model_providers.rs` | Named custom gateway defaults (`[model_providers.<id>]`) | model override resolver | New operator-facing connection template. Distinguish it from bum's built-in provider identity enum. |
| `config/managed_text/*` | Validated managed text/config transactions | config/shell consumers | New config foundation; import before consumers and review home/path behavior. |
| `session/storage/relocation/*` | Session discovery/relocation after cwd or host changes | persistence/resume | New persistence flow; validate all paths remain under bum home. |
| diagnostics + `doctor_cmd` | Terminal/tmux/clipboard/keyboard probes and fixes | composition root and slash command | New user feature; rebrand commands/paths and ensure fixes never install stock `grok` aliases. |

### Semantic naming collision to resolve deliberately

Upstream introduces string-valued `model_provider` / `[model_providers.<id>]` for reusable custom gateway connection settings. bum already uses `ModelProvider` as a closed built-in inference-ownership enum (`Xai`, `Codex`) that determines credentials, trusted endpoints, wire policy, and login gates.

These are **not the same abstraction**:

```text
Built-in inference ownership: Xai | Codex
    decides OAuth slot, trusted-host policy, wire profile, login gate

Named model connection profile: "gateway" | "corp-proxy" | ...
    supplies inherited base_url, headers, env keys, helper reference
```

Do not collapse them or infer one from the other. Recommended durable naming:

- rename/express bum's closed concept as `InferenceProvider` (or `CredentialProvider`) in the integrated architecture;
- retain upstream's `model_provider: Option<String>` as `ModelConnectionProfileId` internally if renaming is practical;
- resolve connection-profile defaults first, then determine immutable inference ownership for built-in catalog models;
- custom models remain BYOK/helper-driven unless explicitly and safely bound to a built-in provider;
- endpoint trust remains an independent check—provider labels alone must never authorize bearer forwarding.

Likewise, upstream's command-backed `auth_provider` is a custom credential-helper reference, while bum's xAI/Codex OAuth is first-party product authentication. Both can feed a request credential, but they have different storage, refresh, trust, and login UX. Keep separate types and define precedence explicitly.

## Renamed, Split, Moved, and Removed Files

Important detected moves/splits include:

| Old path/behavior | Upstream target | Integration instruction |
|---|---|---|
| `pager/src/views/settings_modal.rs` | `views/settings_modal/{mod,state,input,render,tests}.rs` | Port bum settings/auth/model UI changes into the owning submodules. Do not resurrect the monolith. |
| `pager/src/diagnostics.rs` | `diagnostics/mod.rs` plus `model`, `fix`, `view`, `probes/*`; separate `doctor_cmd/*` | Map old terminal setup behavior into doctor/probe command boundaries. |
| `/terminal-setup` command | Removed/superseded by `/doctor` and `grok doctor` | Preserve upstream replacement, rebrand to `bum doctor`; record supersession rather than exclusion. |
| sticky minimal screen-mode test | `minimal_cli_screen_mode_does_not_persist.rs` | This is a behavior change, not a pure rename; accept upstream semantics unless a bum contract says otherwise. |
| endline wake marker test | markerless wakeup test | Treat as changed transcript protocol/UX; update downstream fixtures deliberately. |
| minimal transcript thinking test | committed thinking-body behavior | Preserve new upstream behavior while revalidating Codex reasoning rendering. |
| large goal classifier/strategist tests | Removed/restructured goal/session flow | Do not copy deleted tests back automatically; map their assertions to current upstream owners where still required. |
| built-in shell skill files | Several removed; workflow-oriented discovery added | Inventory semantic replacement before classifying deletion. Paths alone are insufficient. |
| textarea logic | New `editor.rs`, `editor_keys.rs`, `editor_tests/*` alongside `textarea.rs` | Port bum input behavior at editor abstraction, not into legacy monolith. |

Run rename detection at multiple thresholds for each future sync, but store explicit accepted mappings in `path-map.toml`. Git's rename detection is heuristic; a split such as settings modal cannot be represented by a single rename record.

## Data Flow Changes and Required Adaptations

### A. Model request and authentication flow

Upstream target adds custom connection profiles and command-backed auth helpers to the existing session-token/BYOK gate:

```text
raw config
  → parse [model_providers.*] and [auth_provider.*]
  → resolve per-model inherited endpoint/headers/helper
  → memoize model auth facts
  → reconstruct SamplingConfig before turn
  → optional helper mint or xAI session bearer
  → sampler
```

bum's integrated flow must become:

```text
catalog/config model
  → apply named connection-profile defaults
  → resolve built-in InferenceProvider (Xai/Codex/custom)
  → enforce endpoint trust and credential-source precedence
  → provider-specific availability/preflight gate
  → reconstruct SamplingConfig
       • Xai: xAI slot/resolver + xAI URL-derived headers
       • Codex: ensure_fresh Codex OAuth + trusted account/wire headers
       • custom: static/env key or upstream command-backed helper
  → sampler with no cross-provider credential leakage
  → provider-specific 401 recovery only
```

The highest-risk files are `shell/src/agent/config.rs`, `auth/{storage,manager,model,meta,credential_provider}.rs`, `session/acp_session_impl/sampler_turn.rs`, `sampler/src/client.rs`, and `sampling-types/src/conversation.rs`. These files have substantial changes on both lines and must be integrated as one provider-runtime phase with golden request tests.

### B. Workflow and child-agent flow

Upstream workflow execution is correctly split:

```text
Workflow tool / slash command
  → Session WorkflowManager
  → xai-workflow Rhai engine
  → WorkflowHostRequest::SpawnAgent(AgentOpts)
  → shell host_service builds SubagentRequest
  → existing subagent event/coordinator
  → result + journal + tracker
  → ACP WorkflowUpdated
  → pager workflow block/tasks pane
```

Do not add provider logic to `xai-workflow`. Adapt at `host_service` where `AgentOpts.model` becomes `SubagentRuntimeOverrides.model`. Before emitting `SubagentEvent::Spawn`, call the same effective-model resolution and missing-provider preflight used by bum's `Task` path. This guarantees:

- a workflow can choose Grok or GPT independently of its parent;
- missing Codex/xAI credentials reject before a workflow child appears pending;
- child effort/wire profile follows the effective child model;
- no parent bearer is inherited merely because the workflow was launched by that parent;
- workflow retries preserve provider isolation.

Add a workflow-specific contract test in both directions (Grok parent → Codex child and Codex parent → Grok child), not just existing Task tests.

### C. Session lifecycle and persistence flow

Upstream adds eager JSONL durability, mirrored/external session state, relocation across directories/hosts, workflow journals, stop gates, and queue combination. These all interact with bum's `~/.bum` isolation and mixed-provider history.

Integrate in this dependency order:

1. `xai-chat-state` commands/persistence/compaction changes;
2. shell storage JSONL and relocation modules;
3. ACP session spawn/state fields and run loop;
4. stop gate, prompt queue, notifications and workflow manager;
5. model-switch/provider-reasoning cleanup;
6. pager resume/session UI.

Validate that relocation searches/imports compatible foreign sessions only where explicitly intended, while bum's own durable state and credentials remain rooted under `~/.bum`. Session import compatibility must not become credential import.

### D. TUI flow

The Action → Dispatch → Effect → TaskResult architecture remains the controlling pattern. Upstream adds workflow updates, doctor, external-editor prompt editing, input/editor extraction, usage/cost status, new terminal/wrap behavior, and extensive task/status changes.

Mapping rules:

- ACP notifications enter through `app/acp_handler`, including `WorkflowUpdated`;
- reducers own modal/status/workflow snapshot state;
- effects own diagnostics fixes, editor subprocesses, list fetches, and auth refresh;
- shell owns actual workflow/session/provider state;
- render crates own terminal, clipboard, tmux and appearance behavior;
- pager-bin only routes `bum doctor`, wrap, login and other process-level commands.

Do not resolve TUI conflicts by taking entire local versions of `app_view.rs`, `actions.rs`, `effects/mod.rs`, or dispatch tests; that would silently discard upstream state variants.

## Merge-Risk Hotspots

Measured overlap (both upstream and bum changed the path from their respective roots) includes at least 220 paths. Highest-risk architectural hotspots include:

| Hotspot | Why risky | Required review seam |
|---|---|---|
| `shell/src/agent/config.rs` | Upstream model/auth-provider profiles overlap bum provider enum/catalog/routing | Config parse matrix + model→provider→endpoint→credential goldens. |
| `shell/src/auth/storage.rs` and manager | Upstream auth refactors overlap bum multi-slot RMW, Codex OAuth and corruption recovery | Dual-slot persistence, lock, refresh, logout, sibling-preservation tests. |
| `session/.../sampler_turn.rs` | Upstream helper auth and retry overlap Codex ensure-fresh/wire/header policy | Captured HTTP requests for xAI, Codex, custom helper and hostile endpoint. |
| `sampler/src/client.rs` / Responses stream | Upstream error fixes overlap bum Codex Responses fidelity | Stored/non-stored Responses, empty terminal frames, tolerant SSE and attribution. |
| `agent/subagent/*` and Task backend | Upstream child/workflow work overlaps cross-provider gates and effort | Four parent/child provider combinations plus missing-slot failures. |
| `pager-bin/src/main.rs` | Upstream doctor/wrap/login/update routing overlaps binary identity and quiet mode | CLI snapshot/static gates; no application logic added. |
| pager `app_view`, dispatch/effects, task-result tests | High upstream UI churn overlaps auth/model badges and product text | Reducer tests first, then PTY; never bulk choose one side. |
| `xai-grok-update` / telemetry | Upstream update and metrics changes can reintroduce egress | Compile/static entry-point inventory plus runtime no-egress probe. |
| `default_models.json` | Upstream Grok model changes overlap GPT-5.6 catalog | Schema validation, unique IDs, provider tags, effort sets and default model policy. |
| generated `Cargo.toml` / `Cargo.lock` | New crate, Rhai, async-openai patch and downstream wire dependencies interact | Resolve manifests first; lockfile last; clean locked build. |

Large upstream-only areas (workflow engine, managed text, diagnostics, relocation, editor extraction, auto-GC) should be imported rather than rewritten. Large overlapping files require semantic adaptation, not “ours/theirs” conflict selection.

## Approach Comparison

| Approach | Initial correctness | Reviewability | Future sync | Recommendation |
|---|---:|---:|---:|---|
| `git merge --allow-unrelated-histories upstream/main` into current main | Low: empty/common-root semantics create repository-wide add/add ambiguity | Low | Creates ancestry, but from an untrustworthy first resolution | Reject. The flag bypasses a safety check; it does not reconstruct the missing base. |
| Synthetic graft/`git replace --graft` then merge | Medium only if the asserted parent is proven | Medium | Fragile/local; replace refs can be absent or disabled and have Git caveats | Use only as a temporary analysis experiment, never as shared synchronization state. |
| Apply six upstream snapshot patches to current bum | Medium | Medium-low: each snapshot mixes hundreds of files and unrelated features | Histories remain unrelated unless bridged later | Not preferred. Useful only as a comparison oracle against the final result. |
| Cherry-pick upstream public commits | Low-medium | Low: commits are monorepo export snapshots, not atomic feature commits | Histories still awkward; conflicts huge | Reject as primary method. |
| Replay all bum commits onto upstream target | Medium-low | Low: includes planning, intermediate red/green states, fixes of fixes and obsolete code layouts | Could create upstream ancestry | Reject mechanically. Use commit history only to reconstruct contract-level patches and tests. |
| **Start at pinned upstream target, port a curated bum contract overlay, then bridge histories after validation** | **High** | **High: upstream inclusion by default, downstream deviations explicit** | **High: real upstream ancestor for subsequent syncs** | **Recommended.** |
| Continue forever with content imports + ledger, no ancestry bridge | High if disciplined | High | Medium: every future sync remains endpoint/patch archaeology | Acceptable fallback if history policy forbids a bridge, but higher long-term cost. |

### Why the recommended bridge is not an unsafe merge

The content is resolved and committed on an upstream-derived integration line first. The final bridge should record two parents while retaining the already-validated integration tree unchanged. Conceptually:

```text
U0--U1--...--UT--B1--B2--...--Bn--J
                                      \
L0--...-------------------------------LM
```

`UT` is the pinned upstream target; `B1..Bn` are curated bum adaptations; `LM` is old bum main; `J` records both histories but has the same tree as `Bn`. The exact safe Git plumbing/merge command should be scripted and reviewed in its own plan. The commit message and ledger must explicitly state that `J` is a **verified content-migration ancestry join**, not evidence that the independent roots were historically identical.

This preserves old bum history, makes `UT` a real ancestor, and permits main to advance to `J` without losing the old line. Never create `J` before parity and invariant gates are green.

## Generated Files and Build Order

### Manifest rules

- Root `Cargo.toml` is explicitly auto-generated in both trees.
- No generator entry point is exposed in this public export.
- Upstream adds the `xai-workflow` workspace member, `rhai`, `dhat`, and a pinned `[patch.crates-io]` for `async-openai`.
- Sixteen per-crate manifests change in the upstream range.
- `Cargo.lock` changes materially and is maintained by Cargo, not manually edited.

Because the integration branch starts at the upstream target, use its generated root manifest and lockfile as the coherent baseline. Apply bum's required per-crate manifest differences (for example binary identity or Codex-specific dependencies) in the owning crate manifests. Any unavoidable generated root delta must be documented as export-maintenance debt; do not casually hand-edit workspace membership or versions. After all manifests settle, regenerate/update and verify `Cargo.lock` once, then run `cargo check --locked` from a clean state.

### Dependency-aware crate validation order

```text
1. New/leaf foundations
   xai-workflow, config-types, shared protocol/types, hooks, sandbox

2. Host and state
   xai-grok-config, fast-worktree, workspace, chat-state, sampling-types

3. Transport and tools
   sampler, tools, MCP, agent definitions

4. Runtime
   xai-grok-shell (auth/config first; session/workflow/subagent second)

5. Presentation
   ratatui-textarea, pager-render, pager-minimal, pager

6. Composition and packaging
   pager-bin, PTY harness, docs, generated root/lock closure
```

A crate compiling does not close its phase if its cross-boundary contract is untested. For example, `xai-workflow` unit tests do not prove provider-safe child spawning; that closes only at shell integration.

## Recommended Integration Phases

### Phase 1 — Pin, inventory, and prove the upstream baseline

**New:** immutable upstream pin, source/tree hashes, path/change inventory, initial ledger schema, path map, validation matrix.

**Modified:** no product source.

Actions:

1. Freeze `3af4d5d3...` (or a deliberately newer fetched SHA if milestone scope changes before implementation) and its `SOURCE_REV`.
2. Build/test upstream target standalone on its own branch/worktree.
3. Generate root-skew, upstream-range, current-tree, added/deleted, rename/split and churn reports.
4. Classify release-note features and security fixes into component change units.
5. Define bum invariant tests before porting.

**Exit gate:** target reproducible; zero unclassified upstream paths in the initial range inventory; no source integration started against a moving ref.

### Phase 2 — Establish upstream-derived integration line and foundational crates

**New:** `xai-workflow`, managed-text modules, relocation primitives, diagnostics/editor leaf modules as present in target.

**Modified:** per-crate dependency manifests only where bum overlay requires it.

Validate upstream target in dependency order. Do not join histories yet. This phase confirms that failures introduced later belong to downstream adaptation, not the baseline.

**Exit gate:** clean upstream baseline builds with pinned lock and targeted leaf tests.

### Phase 3 — Port product perimeter: identity, home, privacy

**New:** explicit invariant/egress validation around newly added doctor, wrap, workflow paths and command entry points.

**Modified:** config/path helpers, pager-bin, version/update/telemetry/feedback, test-support path fixtures, product chrome foundations.

Port `bum`, `BUM_HOME`, `~/.bum`, hard-off update, telemetry and feedback policies before auth/session data is exercised. Adapt upstream's new `.grok/workflows`, relocation, doctor fix, resume and alias paths to bum identity where they represent bum-owned state.

**Exit gate:** hermetic home test; `bum version`; no stock binary/home in authoritative runtime paths; no update/telemetry egress under defaults.

### Phase 4 — Converge config, auth, catalog, and provider routing

**New:** upstream command-backed auth helpers and named model connection profiles, kept separate from first-party provider ownership.

**Modified:** shell agent config, model catalog, auth module, sampler reconstruction, composition login/logout/status.

Resolve the semantic naming collision, establish precedence, port dual OAuth and missing-provider gates, then test request construction before adding workflows. Keep endpoint trust independent of labels.

**Exit gate:** xAI and Codex auth lifecycle tests; malicious/custom endpoint isolation; mixed catalog; mid-session switching; custom helper tests; no credential crosses provider boundaries.

### Phase 5 — Integrate state, sampling, and persistence changes

**New:** storage relocation, eager durability, stop gate, queue combination and usage/cost state.

**Modified:** chat-state, sampling types, sampler, ACP session spawn/run/model-switch/persistence.

Take upstream state and durability changes first, then reapply Codex Responses wire and mixed-provider history cleanup. This phase must precede workflow orchestration because workflows persist journals and depend on stable session lifecycle.

**Exit gate:** session resume/relocation under bum home; compaction/rewind; xAI and Codex captured request/stream tests; effort preference and provider-switch encrypted reasoning tests.

### Phase 6 — Integrate subagents and workflows

**New:** workflow engine, shell workflow service/manager/store/tracker, workflow tool, built-in scripts, ACP updates.

**Modified:** agent builder/config tool sets, subagent coordinator, Task backend/types, session resource wiring.

Wire workflow child spawning through bum's provider-aware preflight. Keep workflow engine provider-neutral. Verify cancellation, budget, journal, pause/resume and process-restart behavior in addition to provider combinations.

**Exit gate:** workflow unit tests plus shell integration; both cross-provider directions; missing-provider preflight; no pending orphan; tool/schema/effort compatibility.

### Phase 7 — Integrate TUI, terminal, diagnostics, and docs

**New:** workflow blocks/overlay, doctor UI/CLI, external editor, editor extraction, usage/cost display, terminal/tmux fixes.

**Modified:** Action/Effect enums, ACP handler, app/agent view, tasks pane, settings, model/auth badges, pager tests and user guide.

Port local UI contracts into upstream's split modules. Update behavior-sensitive tests according to upstream semantics; rebrand expected strings separately from functional changes.

**Exit gate:** reducer tests, pager tests, PTY/minimal/fullscreen smoke, doctor JSON/human output, dual-provider model/auth UI, workflow progress/cancel UX.

### Phase 8 — Provenance closure, ancestry bridge, and release validation

**New:** final ledger, allowlist, generated reports, future-sync runbook, intentional ancestry bridge.

**Modified:** generated manifest/lock only as justified, release documentation.

Actions:

1. Compare final integrated tree directly against pinned upstream target (two-endpoint diff, not merge-base diff).
2. Require every difference to map to an invariant/adaptation/exclusion ledger entry.
3. Require every upstream change unit to be imported/adapted/excluded/superseded; zero pending.
4. Run clean locked build, targeted matrix, full feasible workspace tests, PTY, and live xAI↔Codex UAT.
5. Scan for stock identity/home and all known phone-home entry points.
6. Create the reviewed two-parent ancestry bridge without changing the validated tree.
7. Verify the pinned upstream commit is now an ancestor and the old bum tip remains reachable.

**Exit gate:** parity ledger complete; bum invariants green; ancestry claims explicit; next-sync dry run documented.

## Validation Seams

### Structural/provenance gates

- `git merge-base` absence is recorded before migration; after the intentional bridge, the pinned upstream target must be an ancestor.
- Public upstream SHA, tree SHA and `SOURCE_REV` must match the ledger.
- Added/deleted/moved/split reports are generated from Git objects, not working-directory copies.
- Final upstream↔bum endpoint diff contains only allowlisted, owner-mapped differences.
- No `pending` ledger disposition at release.
- Patch IDs may identify exact patch reuse, but do not prove semantic equivalence for adapted changes.
- `git range-diff` is a human review aid for iterations of the downstream overlay, not a stable machine ledger format.

### Build gates

- leaf/domain crates before shell/pager consumers;
- `cargo fmt --all --check`;
- crate-scoped check/test throughout phases;
- clean `cargo check --locked -p xai-grok-pager-bin` after manifest closure;
- feature checks for default `jemalloc`/`sandbox-enforce` and shipping profile where feasible;
- PTY tests after render/pager integration, not as the first signal.

### Bum invariant gates

| Invariant | Minimum automated evidence |
|---|---|
| Identity/home | Hermetic process with conflicting HOME/GROK_HOME/BUM_HOME; all bum-owned state under temp `~/.bum`; `bum version` product token. |
| Dual auth | Independent login/status/refresh/logout; one-slot corruption or logout never destroys sibling slot. |
| Routing | Captured destination, Authorization and reserved headers for Grok, Codex, custom helper and hostile endpoints. |
| Switching | Grok→Codex→Grok in one persisted session; effort and provider reasoning state correct. |
| Cross-provider children | Parent/child matrix for Task and Workflow; missing-provider failure before pending state. |
| Privacy | Static entry-point inventory plus runtime network capture proving update/telemetry/feedback paths remain disabled by default. |
| Compatibility | Interactive, headless and ACP smoke for both providers; provider gaps explicitly documented. |

### Upstream feature gates

Use upstream 0.2.102–0.2.109 changelogs as a feature checklist, but bind each item to code/tests rather than trusting prose. High-level clusters include:

- terminal/minimal/clipboard/wrap/doctor and external editor;
- auth single-flight/recovery and command-backed custom credentials;
- named model connection providers and max effort;
- session relocation/mirroring/eager durability;
- stop-hook feedback, permission/auto-mode behavior and sandbox/network policy;
- workflow orchestration and task/status presentation;
- prompt queue combination, usage/cost, and read-file skill behavior;
- performance and durability work in chat state, fast worktree and persistence.

## Future Synchronization Runbook

After v1.1 establishes real ancestry:

1. `git fetch upstream` and pin a target; never integrate a moving ref.
2. Read old/new `SOURCE_REV` and public changelogs.
3. Generate `previous_upstream..new_upstream` change inventory, rename map, manifest delta and hotspot overlap against current bum overlay.
4. Open an integration branch from current bum main; merge the pinned upstream target with `--no-commit` or rebase only if policy chooses, now using the real previous upstream ancestor.
5. Resolve by component ownership, never global “ours/theirs”. Enable `rerere` only as an assistant and review every reused resolution before staging.
6. Update ledger dispositions and final tree-difference allowlist in the same change.
7. Run dependency-ordered tests, bum invariant matrix, upstream feature/security gates, then live dual-provider UAT.
8. Advance the recorded upstream/SOURCE_REV only when all gates pass.

Do not carry synthetic replace refs, grafts, or local rerere state as the source of truth. The committed tree, real ancestry, ledger, and tests are the durable synchronization architecture.

## Anti-Patterns to Avoid

### Treating `--allow-unrelated-histories` as reconstruction

It only overrides Git's refusal. With no common base, it cannot know that two same-subject roots are related or which 179 root-skew changes are export differences.

### Bulk choosing upstream or bum versions of hotspot files

Taking upstream wholesale can delete Codex and privacy contracts. Taking bum wholesale can erase upstream workflow/auth/session fixes. Integrate semantic units behind typed boundaries and tests.

### Calling every final diff an exclusion

Most final differences should be contract adaptations. “Excluded” means the upstream behavior is absent; it requires a stronger rationale and user/security impact statement.

### Letting generated files lead the integration

A root manifest or lockfile that compiles can still hide missing per-crate architecture. Resolve source and owning manifests first; lock closure comes last.

### Mixing custom gateway profiles with built-in provider identity

A named gateway is connection configuration; xAI/Codex is credential and wire ownership. Conflation risks sending a first-party OAuth bearer to an operator-provided URL.

### Implementing workflow-specific auth

Workflow children are ordinary subagent requests. Reuse one provider-aware preflight and request resolver; otherwise Task and Workflow paths will drift.

### Bridging histories before validation

Once a two-parent join exists, Git will assume related ancestry in future operations. An incorrect join makes later merges look safer than they are. Bridge only the final proven tree.

## Research Flags for Roadmap

- **Provider/config convergence:** requires phase-specific design review because upstream's `model_provider` and `auth_provider` concepts overlap bum names but not semantics.
- **Workflow child routing:** requires dedicated research/test design around model/effort/provider propagation and cancellation ownership.
- **Session relocation/import:** requires privacy review to separate session compatibility from credential/home sharing.
- **Generated root manifest:** public export exposes no generator; decide and document how downstream-only workspace dependency changes are maintained without pretending the generated root is hand-owned.
- **Ancestry bridge mechanics:** script and peer-review the exact operation in the final phase; prove tree identity before and after.
- **Upstream snapshot semantics:** public commits do not expose monorepo feature commits, so some change-unit grouping remains an informed reconstruction from diffs, modules, tests, and changelogs.

## Sources

### Repository evidence (direct inspection)

- Local refs and histories: `main`, `upstream/main`, roots, tree hashes, commit counts, merge-base check.
- Upstream provenance: `SOURCE_REV`, upstream `README.md`, changelogs `0.2.102`–`0.2.109`.
- Tree comparisons: local root ↔ upstream root; upstream root ↔ target; current main ↔ target; per-snapshot stats; added/deleted and rename detection; per-crate churn and overlap ranking.
- Architecture maps: `.planning/PROJECT.md`, `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`, `.planning/codebase/CONCERNS.md`.
- Upstream component sources: `xai-workflow`, shell workflow/session/auth/model-provider modules, pager workflow/doctor/settings splits, workspace/config/sampler manifests and tests.

### Official documentation

- [Git merge: unrelated histories and merge semantics](https://git-scm.com/docs/git-merge)
- [Git replace/graft behavior and caveats](https://git-scm.com/docs/git-replace)
- [Git patch-id for likely equivalent patches](https://git-scm.com/docs/git-patch-id)
- [Git range-diff for human comparison of patch-series iterations](https://git-scm.com/docs/git-range-diff)
- [Git rerere for reusable but review-required conflict resolution](https://git-scm.com/docs/git-rerere)
- [Git diff: direct endpoint comparison versus merge-base forms](https://git-scm.com/docs/git-diff)
- [Cargo.toml versus Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)

---

*Architecture research for v1.1 upstream Grok Build parity. No source implementation performed.*
