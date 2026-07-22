# Roadmap: bum

## Milestones

- ✅ **v1.0 Multi-provider daily driver** - Phases 1–12.1 (shipped 2026-07-22)
- 🚧 **v1.1 Upstream Grok Build parity** - Phases 13–21 (planned)

## Overview

Milestone v1.1 brings bum to applicable parity with the frozen Grok Build 0.2.109 source through an audited content integration. Work proceeds in the research-approved dependency order: provenance → P0 security → foundations → product perimeter → provider seam → sessions/scheduler/worktrees → workflows → TUI/extensions/docs → closure. Each phase preserves bum's shipped identity, isolated home, dual OAuth, explicit per-model routing, cross-provider children, Codex wire behavior, and quiet-fork privacy contract.

## Milestone Context

- **Frozen upstream public SHA:** `3af4d5d39897855bdcc74f23e690024a5dc05573`
- **Frozen upstream version:** `0.2.109`
- **Frozen upstream `SOURCE_REV`:** `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`
- **History constraint:** bum and the pinned upstream line have no Git merge base; their publication roots are not tree-equivalent. No normal merge, rebase, or `--allow-unrelated-histories` operation may be used to resolve v1.1 content.
- **Approved integration architecture:** establish the pinned upstream tree as the coherent baseline, then apply a curated bum contract overlay. Upstream inclusion is the default; every deviation must be ledgered as an adaptation, tested supersession, exclusion, or unavailable public artifact.
- **Ancestry policy:** only after the parity tree passes all validation may closure create a reviewed two-parent ancestry bridge. The bridge must preserve the validated tree byte-for-byte and is a history-recording operation, never the integration strategy.
- **Numbering:** v1.0 ended at Phase 12.1, so planned v1.1 work continues at Phase 13.

## Phases

- [ ] **Phase 13: Immutable Baseline & Provenance** - Freeze and expose the exact target, reconcile all source evidence, and establish the repeatable parity ledger
- [ ] **Phase 14: P0 Security Parity** - Restore the complete upstream security floor before general feature integration
- [ ] **Phase 15: Upstream Foundations & Dependency Closure** - Establish the coherent upstream-derived crate, dependency, structural, and test foundation
- [ ] **Phase 16: Product Perimeter & Quiet-Fork Contracts** - Adapt all imported paths to bum identity, isolated storage, explicit remotes, and no phone-home
- [ ] **Phase 17: Provider, Authentication, Catalog & Sampling Seam** - Integrate upstream provider capabilities without weakening bum's dual-provider ownership and routing
- [ ] **Phase 18: Durable Sessions, Scheduler & Worktrees** - Deliver safe persistence, queued/background lifecycle, scheduling, relocation, and guarded worktree cleanup
- [ ] **Phase 19: Workflows & Cross-Provider Orchestration** - Import the off-by-default workflow runtime on bum's provider-safe child execution path
- [ ] **Phase 20: TUI, Tools, Extensions & Documentation** - Deliver applicable user-facing terminal, diagnostic, extension, and guidance parity
- [ ] **Phase 21: Parity Closure & Ancestry Bridge** - Prove complete parity, validate bum contracts, bridge histories without changing the tree, and dry-run the next sync

## Phase Details

### Phase 13: Immutable Baseline & Provenance

**Goal**: Maintainers can reproduce, inspect, and repeat the exact v1.1 synchronization scope without relying on nonexistent ancestry
**Depends on**: Phase 12.1 (v1.0 shipped)
**Requirements**: SYNC-01, SYNC-02, SYNC-03, SYNC-04, SYNC-05, SYNC-06
**Success Criteria** (what must be TRUE):

  1. Maintainer can reproduce the frozen 0.2.109 target and verify its public SHA, `SOURCE_REV`, tree identity, and recorded checksums independently.
  2. Maintainer can inspect a complete ledger in which every `U001–U168` item has exactly one terminal disposition and the publication-root skew, upstream root-to-pin, bum root-to-current, additions, deletions, and tests are reconciled without a merge base.
  3. Maintainer can inspect every intentional bum-versus-upstream difference with a rationale, protected invariant, implementation evidence, and verification evidence.
  4. Maintainer can run the documented future-sync procedure against a newer target and receive a bounded comparison report without changing the frozen v1.1 pin.
  5. User can run bum version and diagnostics and distinguish bum's own version from upstream version 0.2.109, public SHA, and `SOURCE_REV`.

**Deeper phase research**: Required — define the ledger schema, three-leg 872-path/change-cluster reconciliation, exact tree/checksum evidence, and frozen-pin reporting workflow.
**Plans**: TBD

### Phase 14: P0 Security Parity

**Goal**: Users receive the complete applicable upstream security protections before imported feature paths are enabled
**Depends on**: Phase 13
**Requirements**: SEC-01, SEC-02, SEC-03, SEC-04, SEC-05, SEC-06
**Success Criteria** (what must be TRUE):

  1. Unsafe safe-command arguments, environment-splitting variants, credential plugins, redirects, and known auto-mode RCE forms cannot bypass ask/deny policy.
  2. Web-fetching and hook/network requests reject private, metadata, mapped, and unsafe redirect destinations under the applicable policy.
  3. Authentication artifacts and sensitive generated files remain owner-only, and xAI/Codex bearers never escape their trusted endpoint boundaries.
  4. Mutable or malicious plugin/Git inputs, managed-policy rollback or replay, unsafe hooks, and ambiguous or issuer-mismatched MCP operations fail closed.
  5. Sandbox, no-controlling-TTY Landlock, repository trust, rules, sensitive-edit, and prompt-injection protections hold while the complete adapted adversarial corpus passes without weakening bum's provider, home, or privacy boundaries.

**Plans**: TBD

### Phase 15: Upstream Foundations & Dependency Closure

**Goal**: Maintainers have a coherent upstream-derived foundation on which all bum adaptations and later parity features can run
**Depends on**: Phase 14
**Requirements**: FOUND-01, FOUND-02, FOUND-03
**Success Criteria** (what must be TRUE):

  1. Maintainer can build the upstream-derived bum foundation with the required pinned `async-openai`, Rhai, workflow, diagnostic, relocation, worktree, and platform primitives present and coherent.
  2. Maintainer can reproduce Cargo metadata and locked dependency state from reviewed owning manifests, without hand-editing generated root or lock state.
  3. Maintainer can audit imported moves, deletions, protocol/config extensions, fixtures, and test topology and find every applicable structural change represented with no unexplained test loss.

**Deeper phase research**: Required — resolve public-export root-manifest maintenance, dependency ownership, structural move/deletion mapping, and test-discovery baselines before integration plans are finalized.
**Plans**: TBD

### Phase 16: Product Perimeter & Quiet-Fork Contracts

**Goal**: Every imported capability behaves as part of bum and remains inside bum's identity, storage, remote, and privacy boundaries
**Depends on**: Phase 15
**Requirements**: ID-01, ID-02, PRIV-01, PRIV-02, PRIV-03
**Success Criteria** (what must be TRUE):

  1. User sees `bum` consistently across imported CLI, TUI, runtime, workflow, diagnostic, storage, and managed-binary paths.
  2. All imported persistent behavior defaults to `BUM_HOME` or `~/.bum`, and compatibility aliases cannot redirect credentials or default product state into stock homes.
  3. Startup, shutdown, workflows, relocation, worktrees, doctor, and every supported run mode cannot invoke stock update/minimum-version, feedback, Mixpanel, Sentry, internal OTEL, or implicit-upload egress.
  4. Cross-host session import or mirroring operates only after an operator explicitly configures a destination; a fresh installation has no default remote.
  5. Maintainer can run hermetic filesystem and network-capture gates across all new execution paths and observe only approved bum-home access and explicitly allowed destinations.

**Plans**: TBD
**UI hint**: yes

### Phase 17: Provider, Authentication, Catalog & Sampling Seam

**Goal**: Users retain safe dual-provider operation while gaining applicable upstream catalog, effort, custom-connection, authentication, and usage capabilities
**Depends on**: Phase 16
**Requirements**: PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06, PROV-07, PROV-08
**Success Criteria** (what must be TRUE):

  1. User can configure custom connection profiles and command-backed credential helpers without those concepts becoming built-in xAI/Codex inference ownership or receiving first-party OAuth merely from labels or endpoints.
  2. User can independently log in, refresh, inspect, recover, and selectively log out of xAI and ChatGPT/Codex without damaging the sibling credential slot.
  3. User can switch Grok ↔ GPT within one persisted session and each turn uses the explicitly bound provider, endpoint, credential, and wire profile with no credential crossover, foreign reasoning leakage, history loss, or Codex wire regression.
  4. User can select Grok 4.5 beside GPT-5.6 Sol/Terra/Luna without an automatic default-model change, and can choose distinct `max` effort only for models that advertise it.
  5. User sees exact per-provider token usage while subscription-backed Codex traffic never shows a speculative dollar amount.

**Deeper phase research**: Required — define inference-owner/profile/helper types and precedence, independent endpoint trust, dual-provider request goldens, capability-aware `max`, and provider-honest usage reporting.
**Plans**: TBD

### Phase 18: Durable Sessions, Scheduler & Worktrees

**Goal**: Users can trust session, background, scheduler, relocation, and worktree state across crashes, reconnects, moves, and cleanup
**Depends on**: Phase 17
**Requirements**: SESS-01, SESS-02, SCHED-01, WORK-01, WORK-02
**Success Criteria** (what must be TRUE):

  1. User can resume eagerly persisted sessions, rewind the intended response, move a working directory, and explicitly import compatible session state without losing history or provider continuity.
  2. User can combine queued follow-ups and use background task/status behavior without turns being lost, duplicated, or left permanently pending.
  3. Scheduled and loop work survives lifecycle transitions with durable deletion/versioning and correct pause, cancel, serialization, and reconnect behavior.
  4. User can inspect or run kind-aware worktree auto-GC with configurable TTLs and dry-run, while live processes/current working directories are protected and stale registrations are cleaned safely.
  5. User can rely on documented safe session/worktree behavior on supported platforms and can see explicit limits where a platform cannot provide equivalent liveness guarantees.

**Deeper phase research**: Required — design relocation/import privacy boundaries, persistence compatibility, scheduler cancellation ownership, and platform-safe worktree GC/liveness behavior.
**Plans**: TBD

### Phase 19: Workflows & Cross-Provider Orchestration

**Goal**: Users can explicitly enable and run upstream workflows with deterministic lifecycle and the same provider isolation as Task children
**Depends on**: Phase 18
**Requirements**: FLOW-01, FLOW-02, FLOW-03, FLOW-04, FLOW-05
**Success Criteria** (what must be TRUE):

  1. Workflow execution remains disabled by default, and once explicitly enabled a user can run an imported workflow whose runtime state and journal persist correctly.
  2. Every workflow child passes through the same effective-model resolution, endpoint trust, credential preflight, and fail-before-side-effects gate as a Task child.
  3. User can run Grok-parent → Codex-child and Codex-parent → Grok-child workflows without changing the parent model or crossing credentials.
  4. Workflow budgets, pause, cancel, reconnect, background execution, journal replay, and restart outcomes remain deterministic under interruption and regression tests.
  5. User and maintainer can see public-pin workflow artifacts that are genuinely absent marked `UNAVAILABLE`; no authoring skill is fabricated or silently installed.

**Deeper phase research**: Required — map provider-neutral workflow hosting to bum child preflight, establish cancellation/budget ownership and journal/restart semantics, and resolve public-source gaps without fabrication.
**Plans**: TBD

### Phase 20: TUI, Tools, Extensions & Documentation

**Goal**: Users receive the applicable upstream terminal experience, diagnostics, extension behavior, and accurate bum guidance
**Depends on**: Phase 19
**Requirements**: UX-01, UX-02, EXT-01, EXT-02, DOC-01
**Success Criteria** (what must be TRUE):

  1. User can rely on canonical and external editing, navigation, rendering, status, minimal/fullscreen, clipboard, SSH, image-paste, and terminal behaviors across supported environments.
  2. User can run read-only `bum doctor` from the CLI or TUI and receive bum-specific terminal, voice, authentication, and environment findings.
  3. User can use applicable upstream skills, plugins, MCP, hooks, project rules, qualified collision handling, and full Markdown reads.
  4. Existing user skill/plugin/hook content is never silently overwritten, and imported extensions cannot bypass trust, permission, provider, home, or privacy policy.
  5. User documentation accurately describes bum commands, paths, providers, workflows, diagnostics, compatibility aliases, privacy behavior, and platform limits without actionable stock-product guidance.

**Plans**: TBD
**UI hint**: yes

### Phase 21: Parity Closure & Ancestry Bridge

**Goal**: Maintainers can prove complete v1.1 parity, record the validated integration in Git without changing its tree, and repeat the process for a future pin
**Depends on**: Phase 20
**Requirements**: DONE-01, DONE-02, DONE-03, DONE-04
**Success Criteria** (what must be TRUE):

  1. Maintainer can produce a release report with zero pending provenance rows and no unexplained reduction in discovered tests, fixtures, snapshots, ignored-state coverage, or supported platform evidence.
  2. Automated and live gates demonstrate bum identity/home/privacy plus dual-provider routing, switching, Task children, and Workflow children in both provider directions.
  3. Maintainer can create and independently verify a reviewed two-parent ancestry bridge whose tree is byte-for-byte identical before and after the bridge, with both old bum history and the pinned upstream commit reachable.
  4. Maintainer can dry-run the next upstream synchronization using the established ancestry, ledger, protected-path inventory, final-difference allowlist, and verification process and receive a bounded review queue.

**Deeper phase research**: Required — script and review exact bridge plumbing, pre/post tree-identity proof, final endpoint-diff reconciliation, and next-pin/synthetic dry-run mechanics.
**Plans**: TBD

## Progress

**Execution Order:** Phase 13 → 14 → 15 → 16 → 17 → 18 → 19 → 20 → 21

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 13. Immutable Baseline & Provenance | v1.1 | 0/TBD | Not started | - |
| 14. P0 Security Parity | v1.1 | 0/TBD | Not started | - |
| 15. Upstream Foundations & Dependency Closure | v1.1 | 0/TBD | Not started | - |
| 16. Product Perimeter & Quiet-Fork Contracts | v1.1 | 0/TBD | Not started | - |
| 17. Provider, Authentication, Catalog & Sampling Seam | v1.1 | 0/TBD | Not started | - |
| 18. Durable Sessions, Scheduler & Worktrees | v1.1 | 0/TBD | Not started | - |
| 19. Workflows & Cross-Provider Orchestration | v1.1 | 0/TBD | Not started | - |
| 20. TUI, Tools, Extensions & Documentation | v1.1 | 0/TBD | Not started | - |
| 21. Parity Closure & Ancestry Bridge | v1.1 | 0/TBD | Not started | - |
