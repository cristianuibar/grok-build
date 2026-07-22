# Requirements: bum v1.1 Upstream Grok Build parity

**Defined:** 2026-07-22
**Core Value:** One CLI (`bum`) can log into both xAI and Codex and freely switch between Grok and GPT-5.6 models in a real coding session, including cross-provider subagent orchestration.
**Pinned upstream:** `xai-org/grok-build` `3af4d5d39897855bdcc74f23e690024a5dc05573` (`0.2.109`), `SOURCE_REV` `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`

## v1.1 Requirements

### Baseline and Provenance

- [ ] **SYNC-01**: Maintainer can reproduce the milestone target from the pinned upstream public SHA, upstream version, source revision, tree identity, and recorded checksums.
- [ ] **SYNC-02**: Maintainer can trace every upstream item `U001–U168` to exactly one disposition: `PORT`, `ADAPT`, `ALREADY/SUPERSEDED`, `EXCLUDE`, or `UNAVAILABLE`.
- [ ] **SYNC-03**: Maintainer can audit publication-root skew, upstream root-to-pin changes, bum root-to-current changes, additions, deletions, tests, and final tree differences without relying on a nonexistent merge base.
- [ ] **SYNC-04**: Maintainer can identify every intentional final difference from upstream with its rationale, protected bum invariant, implementation evidence, and verification evidence.
- [ ] **SYNC-05**: Maintainer can run a documented future-sync procedure against a newer upstream revision without moving the frozen v1.1 pin.
- [ ] **SYNC-06**: User can inspect bum's own version plus the pinned upstream version, public SHA, and source revision through version and diagnostic output.

### Security Parity

- [ ] **SEC-01**: User is protected by upstream-equivalent command-permission and auto-mode RCE defenses, including unsafe safe-command arguments and environment-splitting variants.
- [ ] **SEC-02**: User is protected by upstream-equivalent SSRF and redirect validation in web-fetching and network tool paths.
- [ ] **SEC-03**: User credentials, authentication artifacts, and sensitive generated files receive upstream-equivalent owner-only handling and bearer scoping.
- [ ] **SEC-04**: User is protected by upstream-equivalent plugin integrity, Git operand, managed-policy rollback/replay, hook, and MCP identity/OAuth issuer defenses.
- [ ] **SEC-05**: User receives upstream-equivalent sandbox, no-controlling-TTY Landlock, repository trust, rules, and prompt-injection protections.
- [ ] **SEC-06**: Maintainer can run the complete adapted upstream adversarial security corpus without weakening bum's provider, storage, or privacy boundaries.

### Foundations

- [ ] **FOUND-01**: Maintainer can build bum from the coherent upstream dependency and crate foundation, including required `async-openai`, Rhai, workflow, diagnostic, relocation, and worktree primitives.
- [ ] **FOUND-02**: Maintainer can regenerate and verify dependency lock state from owning manifests without hand-editing generated root state.
- [ ] **FOUND-03**: User receives applicable upstream file moves, deletions, protocol extensions, configuration schemas, and test topology without silent test loss.

### Product Identity and Privacy

- [ ] **ID-01**: Every imported CLI, TUI, runtime, workflow, diagnostic, storage, and managed-binary path presents `bum` as the product.
- [ ] **ID-02**: Every imported persistent feature defaults to `BUM_HOME` and `~/.bum`; documented compatibility aliases cannot redirect credentials or default storage to stock locations.
- [ ] **PRIV-01**: Normal bum operation cannot invoke the stock updater, minimum-version enforcement, feedback transport, Mixpanel, Sentry, internal OTEL, or implicit upload.
- [ ] **PRIV-02**: Cross-host session import or mirroring works only with an explicit operator-configured destination and never creates a default remote.
- [ ] **PRIV-03**: Maintainer can run hermetic filesystem and egress tests over startup, shutdown, workflows, relocation, worktrees, doctor, and all supported run modes.

### Providers, Authentication, Catalog, and Sampling

- [ ] **PROV-01**: Built-in xAI/Codex inference ownership remains distinct from upstream custom connection profiles and command-backed credential helpers.
- [ ] **PROV-02**: Custom connection profiles cannot receive xAI or Codex OAuth credentials merely because of a label or endpoint setting.
- [ ] **PROV-03**: User retains independent xAI and ChatGPT/Codex OAuth login, refresh, status, selective logout, and sibling-safe recovery.
- [ ] **PROV-04**: User retains explicit per-model provider-to-endpoint-to-credential-to-wire routing with no model-name guessing or global provider mode.
- [ ] **PROV-05**: User can select Grok 4.5 alongside existing GPT-5.6 Sol/Terra/Luna entries without changing bum's current default model.
- [ ] **PROV-06**: User can select distinct `max` reasoning effort only where the target model advertises support.
- [ ] **PROV-07**: User can switch between xAI and Codex in one session without credential crossover, foreign reasoning leakage, history loss, or Codex wire regression.
- [ ] **PROV-08**: User sees exact token usage for both providers; subscription-backed Codex sessions do not display speculative dollar costs.

### Sessions, Scheduling, and Worktrees

- [ ] **SESS-01**: User receives upstream-equivalent eager durable persistence, rewind response identity, moved-directory relocation, and explicit session import behavior.
- [ ] **SESS-02**: User can combine queued follow-ups and use background task/status behavior without losing or duplicating turns.
- [ ] **SCHED-01**: User receives durable scheduler deletion, lifecycle versioning, serialized loop work units, and correct pause, cancel, and reconnect behavior.
- [ ] **WORK-01**: User receives guarded worktree auto-GC with configurable kind-aware TTLs, dry-run, process/CWD liveness protection, and stale-registration cleanup.
- [ ] **WORK-02**: Worktree and session lifecycle behavior remains safe and documented across supported platforms, with unsupported platform limits stated explicitly.

### Workflows and Cross-Provider Orchestration

- [ ] **FLOW-01**: User can run the imported upstream workflow runtime and persistence model, with the feature disabled by default until explicitly invoked or configured.
- [ ] **FLOW-02**: Workflow children use the same provider-aware model resolution, credential preflight, endpoint trust, and fail-before-side-effects rules as Task children.
- [ ] **FLOW-03**: User can run Grok-parent-to-Codex-child and Codex-parent-to-Grok-child workflows without changing the parent model or crossing credentials.
- [ ] **FLOW-04**: Workflow budgets, pause, cancel, reconnect, journal/restart, and background execution behave deterministically under regression tests.
- [ ] **FLOW-05**: Workflow artifacts named upstream but absent from the public pin remain explicitly `UNAVAILABLE` rather than being fabricated or auto-installed.

### TUI, Tools, Extensions, and Documentation

- [ ] **UX-01**: User receives applicable upstream canonical editor, external editor, navigation, rendering, status, minimal/fullscreen, clipboard, SSH, image-paste, and terminal behavior.
- [ ] **UX-02**: User can run read-only `bum doctor` from the CLI and TUI and receive bum-specific terminal, voice, authentication, and environment diagnostics.
- [ ] **EXT-01**: User receives applicable upstream skills, plugins, MCP, hooks, project-rules, collision handling, non-overwrite, and full Markdown-read behavior.
- [ ] **EXT-02**: Imported skills, plugins, and hooks never silently overwrite user content or bypass bum's trust, permission, provider, home, or privacy policies.
- [ ] **DOC-01**: User documentation accurately describes bum commands, paths, providers, workflows, diagnostics, compatibility aliases, privacy behavior, and platform limits.

### Parity Closure

- [ ] **DONE-01**: Release validation has zero pending provenance rows and no unexplained reduction in discovered tests, fixtures, snapshots, or platform coverage.
- [ ] **DONE-02**: Automated and live regression gates prove bum identity, home isolation, quiet-fork behavior, dual-provider routing, switching, Task children, and Workflow children.
- [ ] **DONE-03**: Maintainer can create a reviewed two-parent ancestry bridge whose tree is byte-for-byte identical to the validated parity tree.
- [ ] **DONE-04**: Maintainer can perform a dry-run subsequent upstream sync using the established ancestry, ledger, protected-path inventory, and verification process.

## Future Requirements

### Workflow Product

- **WFPROD-01**: User can author and manage bespoke bum workflow products beyond the upstream parity infrastructure.

### Provider Expansion

- **PROV-09**: User can configure additional third-party providers as shipped first-class model choices.

### Distribution and Internal Rebrand

- **DIST-01**: User can install bum through an official independently signed distribution channel.
- **ID-03**: Maintainer can remove residual internal `xai-grok-*` crate and compatibility naming where it does not harm upstream synchronization.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Bespoke bum workflow product/design | v1.1 imports and validates upstream workflow infrastructure; product-specific workflow design follows later. |
| New shipped first-class providers beyond xAI and Codex | Preserve the validated v1.0 provider scope while integrating upstream profile infrastructure. |
| Official x.ai/npm distribution | bum remains an independent local/team product and must not adopt stock distribution/update behavior. |
| Complete internal crate rename | High-conflict cleanup is not necessary for user-facing parity and would complicate future synchronization. |
| Fabricated `ClientToolResult` / `ChatConfig` schema | Commit summaries mention this work, but the pinned public source exports no authoritative implementation. |
| Fabricated workflow-authoring skills | Matching skill files are absent from the pinned public source and must remain `UNAVAILABLE`. |
| Automatic stock workflow-skill installation | Would overwrite or alter user content and violate the explicit non-overwrite requirement. |
| Speculative dollar cost for ChatGPT subscription traffic | Public API pricing is not authoritative for subscription-backed Codex usage; v1.1 reports tokens only. |
| Implicit cross-host mirroring or upload | Conflicts with the quiet-fork privacy contract; only explicit operator-configured destinations are allowed. |
| Changing bum's default model solely for upstream parity | Grok 4.5 is added to the mixed catalog, but default selection remains a separate bum product decision. |

## Approved Integration Decisions

| Decision | Choice |
|----------|--------|
| Integration architecture | Build from the pinned upstream baseline and port a curated bum contract overlay. |
| Future ancestry | Create a tree-preserving two-parent bridge only after parity validation succeeds. |
| Provider concepts | Keep built-in inference ownership separate from custom connection profiles and credential helpers. |
| Cross-host session behavior | Explicit operator opt-in only; no default remote. |
| Grok 4.5 | Add to the mixed catalog without changing bum's current default model. |
| Provenance display | Expose bum version and upstream version/SHA/`SOURCE_REV` through version and diagnostics. |
| Codex usage cost | Report exact token usage only; do not estimate dollars. |
| Legacy identity compatibility | Support explicit bounded aliases while keeping bum names/paths primary and storage isolated. |

## Traceability

Each approved v1.1 requirement maps to exactly one roadmap phase.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SYNC-01 | Phase 13 | Pending |
| SYNC-02 | Phase 13 | Pending |
| SYNC-03 | Phase 13 | Pending |
| SYNC-04 | Phase 13 | Pending |
| SYNC-05 | Phase 13 | Pending |
| SYNC-06 | Phase 13 | Pending |
| SEC-01 | Phase 14 | Pending |
| SEC-02 | Phase 14 | Pending |
| SEC-03 | Phase 14 | Pending |
| SEC-04 | Phase 14 | Pending |
| SEC-05 | Phase 14 | Pending |
| SEC-06 | Phase 14 | Pending |
| FOUND-01 | Phase 15 | Pending |
| FOUND-02 | Phase 15 | Pending |
| FOUND-03 | Phase 15 | Pending |
| ID-01 | Phase 16 | Pending |
| ID-02 | Phase 16 | Pending |
| PRIV-01 | Phase 16 | Pending |
| PRIV-02 | Phase 16 | Pending |
| PRIV-03 | Phase 16 | Pending |
| PROV-01 | Phase 17 | Pending |
| PROV-02 | Phase 17 | Pending |
| PROV-03 | Phase 17 | Pending |
| PROV-04 | Phase 17 | Pending |
| PROV-05 | Phase 17 | Pending |
| PROV-06 | Phase 17 | Pending |
| PROV-07 | Phase 17 | Pending |
| PROV-08 | Phase 17 | Pending |
| SESS-01 | Phase 18 | Pending |
| SESS-02 | Phase 18 | Pending |
| SCHED-01 | Phase 18 | Pending |
| WORK-01 | Phase 18 | Pending |
| WORK-02 | Phase 18 | Pending |
| FLOW-01 | Phase 19 | Pending |
| FLOW-02 | Phase 19 | Pending |
| FLOW-03 | Phase 19 | Pending |
| FLOW-04 | Phase 19 | Pending |
| FLOW-05 | Phase 19 | Pending |
| UX-01 | Phase 20 | Pending |
| UX-02 | Phase 20 | Pending |
| EXT-01 | Phase 20 | Pending |
| EXT-02 | Phase 20 | Pending |
| DOC-01 | Phase 20 | Pending |
| DONE-01 | Phase 21 | Pending |
| DONE-02 | Phase 21 | Pending |
| DONE-03 | Phase 21 | Pending |
| DONE-04 | Phase 21 | Pending |

**Coverage:**
- v1.1 requirements: 47 total
- Mapped to phases: 47
- Unmapped: 0
- Duplicate mappings: 0
- Unique coverage: 100%

---
*Requirements defined: 2026-07-22*
*Last updated: 2026-07-22 after v1.1 roadmap creation and unique coverage validation*
