# Project Research Summary

**Project:** bum — v1.1 Upstream Grok Build parity
**Domain:** Controlled synchronization of a heavily customized Rust terminal AI coding-agent fork
**Researched:** 2026-07-22
**Confidence:** MEDIUM-HIGH

## Executive Summary

bum v1.1 is a security-sensitive content integration, not a conventional upgrade merge. The official target is public commit `3af4d5d39897855bdcc74f23e690024a5dc05573` (0.2.109), whose exported monorepo source is `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`; research inspected bum at `128992c5a151e857c728bcbb054c2a8c7c47ce7a`. There is **no merge base** between bum and upstream. Their publication roots—bum `c1b5909ec707c069f1d21a93917af044e71da0d7` and upstream `c68e39f60462f28d9be5e683d9cbe2c57b1a5027`—already differ by 179 paths. Therefore `--allow-unrelated-histories`, bulk checkout, ordinary rebase, or tip-to-tip patching cannot establish trustworthy parity.

The recommended architecture is an immutable upstream baseline plus a curated bum contract overlay, validated before any optional ancestry bridge. Upstream inclusion should be the default; every deviation must map to a bum invariant, an adaptation, a tested supersession, or an explicit exclusion. Completeness is governed by a machine-readable ledger covering **every U001–U168 item exactly once**, all 0.2.102–0.2.109 release-note behavior, path/change clusters, additions, deletions, tests, and final upstream↔bum differences. Security is P0: permission/RCE, SSRF, credential-file, plugin, managed-policy, sandbox, MCP, prompt-injection, and repo-trust fixes must land or be proven equally mitigated before feature parity work.

The principal risk is producing a compiling stock Grok tree that silently erases bum v1.0. Non-negotiable invariants are the `bum` binary/product, `BUM_HOME` and isolated `~/.bum`, independent xAI and ChatGPT/Codex OAuth, explicit mixed per-model routing, cross-provider children, Codex wire behavior, and no stock updater or product telemetry phone-home. The highest-conflict seam is provider/auth configuration: upstream connection profiles and command-backed credential helpers are useful additions, but they are not bum's built-in inference-provider identity and must never authorize first-party bearer forwarding to arbitrary endpoints.

## Verified Baseline and Scope

### Verified facts

- Public upstream pin: `3af4d5d39897855bdcc74f23e690024a5dc05573`, version 0.2.109.
- Upstream `SOURCE_REV`: `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`.
- bum tip inspected: `128992c5a151e857c728bcbb054c2a8c7c47ce7a`.
- No merge base exists between bum and upstream.
- Publication roots are unrelated and non-equivalent: bum `c1b5909ec707c069f1d21a93917af044e71da0d7`; upstream `c68e39f60462f28d9be5e683d9cbe2c57b1a5027`; 179 paths differ.
- Official post-publication syncs are `8adf9013a0929e5c7f1d4e849492d2387837a28d`, `98c3b2438aa922fbbe6178a5c0a4c48f85edc8ce`, `7cfcb20d2b50b0d18801a6c0af2e401c0e060894`, `ba76b0a683fa52e4e60685017b85905451be17bc`, `a881e6703f46b01d8c7d4a5437683546df30449d`, and the pinned tip.
- The six sync bodies contain 168 explicit bullets. Research verified the U001–U168 partition has no missing or duplicate IDs.
- Upstream root→tip touches 872 files; 220 paths overlap changes made by bum. A final endpoint diff is an audit input, not a change inventory by itself.

### Recommendation

Use three comparison legs—publication-root skew, upstream root→pin, and bum root→current—and assign each unit one terminal disposition: **PORT**, **ADAPT**, **ALREADY/SUPERSEDED**, **EXCLUDE**, or **UNAVAILABLE**. Preserve the public SHA and `SOURCE_REV` as separate provenance fields. Freeze the target for the milestone; a newer upstream commit requires restarting inventory rather than moving the pin mid-execution.

## Key Findings

### Recommended Stack

The existing Rust workspace remains the correct platform. There is no framework migration: Rust 1.92.0/edition 2024, Cargo resolver 2/lockfile v4, Tokio 1, ratatui 0.29, crossterm 0.28, ACP 0.10.4, reqwest 0.12 (0.13 only in MCP), rmcp 2.1, tonic/prost 0.14, serde/JSON/TOML remain.

**Required stack additions and pins:**

- `async-openai` 0.33.1 fork — git `https://github.com/our-forks/async-openai.git`, rev **`95b52ebdedf42143083cf3d6f0e0be7c84e9c808`**; adds distinct `ReasoningEffort::Max`.
- `rhai` — manifest 1.25 with `serde`, lock **1.25.1**; powers upstream workflows.
- `xai-workflow` 0.1.0 — new provider-neutral workflow engine, metadata validator, host protocol, journal/replay, and outcomes.
- `dhat` — manifest 0.3, lock **0.3.3**; optional `dhat-heap` lifecycle soak only.
- Adopt upstream target/feature/test dependencies (`shlex`, `xai-tty-utils`, sandbox dependencies, Windows file-system feature, Unix `libc`) with owning source changes.

Critical protocol facts: the checked-in protobuf is unchanged; tool protocol remains 1.0.0; leader protocol remains 1; ACP stays 0.10.4 and gains x.ai extension methods. Do not invent a protocol bump or fabricate unexported schema.

Pinned upstream audit checksums: `Cargo.lock` `6b2eb2c1633bdf2dcfd030208c19a36c52df862522cb72c707982b0bb365e5b8`; `Cargo.toml` `71965098017b6d6c4e228454beab58d8854257afbc648f65190c9fd7891f53bb`; `rust-toolchain.toml` `8632e4f532c10ff7728fdef1bb73f9f391577ca2b23ed133272c714bd371e696`; `default_models.json` `620cfe2b3e4ba8d5e74560ce22a4d42f0f5418a2fb5f192bd38c35b234f3c148`.

### Expected Features

**Must have (table stakes):**

- Complete U001–U168 ledger plus changelog/path/deletion/test coverage, with source, disposition, bum evidence, verification, and rationale.
- P0 security parity: redirect and `web_fetch` SSRF, owner-only credential/artifact files, bearer scoping, plugin SHA/Git operand safety, rollback/replay-resistant managed policy, shell/hook permission hardening, sandbox inheritance/no-TTY Landlock, MCP identity/OAuth issuer safety, prompt/rules/repository trust gates, and auto-mode RCE corpus.
- Session/runtime parity: eager durable persistence, rewind identity, relocation, explicit mirrored import, background status/tasks, scheduler lifecycle/versioning, serialized loops, queue combination, and provider-aware usage.
- TUI/terminal parity: canonical editor, minimal/fullscreen behavior, clipboard and SSH handling, external editor, `bum doctor`, navigation/rendering/status fixes, and PTY coverage.
- Skills/plugins/MCP/hooks parity, including no silent default-skill installation, qualified collisions, non-overwrite, full Markdown reads, setup preferences, stop-hook feedback, and fail-closed matching.
- Workflow runtime imported as upstream infrastructure, disabled by default, after security and persistence; workflow children must reuse bum's provider-aware subagent preflight.
- Worktree auto-GC only with TTL, liveness/CWD, dry-run, stale-registration, and platform guards.
- Grok 4.5 catalog merge without deleting GPT-5.6 Sol/Terra/Luna; distinct `max` only where model capability advertises it.

**Should have / parity differentiators:**

- Explicit provenance/version display separating bum release identity, upstream public SHA, and `SOURCE_REV`.
- Named custom connection profiles and command-backed credentials, safely subordinate to built-in provider identity and endpoint trust.
- Cross-provider workflow acceptance in both directions, preserving bum's core differentiator.
- A repeatable future-sync report, protected-path inventory, egress/home gates, and final-diff allowlist.

**Defer beyond v1.1:**

- A bespoke bum workflow product/design; import upstream parity infrastructure only.
- Arbitrary third-party providers as shipped first-class choices.
- Official x.ai/npm distribution, complete internal crate rename, and unrelated framework/monorepo cleanup.

### Architecture Approach

**Recommendation:** start an integration line from the immutable upstream target, import its coherent tree and new foundations, then port a curated bum overlay. Validate the resolved content before creating any optional two-parent ancestry bridge whose tree is unchanged. This reconciles the research: STACK recommends semantic content porting, while ARCHITECTURE recommends upstream-as-base. Both reject a normal merge and agree that the durable unit is upstream content plus explicit bum contracts. The upstream-derived line is preferable for inclusion completeness; the ledger and endpoint audits are mandatory because root skew and bum overlap prevent trusting ancestry alone.

**Major components:**

1. **Upstream baseline/provenance control plane** — pins public SHA/tree/`SOURCE_REV`, U001–U168 and path ledgers, path maps, invariant owners, and final-diff allowlist.
2. **Leaf foundations** — config/types, managed text, hooks, MCP, sandbox, fast-worktree, diagnostics, relocation, and provider-neutral `xai-workflow`.
3. **Provider runtime** — shell auth/config, catalog, inference ownership, connection profiles, credential precedence, sampler, and Codex/xAI request policies.
4. **State/orchestration** — chat state, persistence, ACP session lifecycle, subagents, workflows, scheduler, prompt queue, and worktree lifecycle.
5. **Presentation/composition** — pager Action→Dispatch→Effect TUI, terminal/render/doctor/workflow UI, and pager-bin wiring that ships `bum` and keeps egress hard-off.

**Key boundary decision:** built-in inference ownership (`Xai`/`Codex`) decides OAuth slot, trusted endpoints, wire policy, and login gate. Upstream named `model_provider` profiles supply connection defaults, and `auth_provider` supplies custom helper credentials. They are separate types with explicit precedence; endpoint trust is checked independently of labels.

### Bum Invariants

Every phase must preserve and test:

- Product/binary/chrome: `bum`, not stock `grok`.
- Storage: `BUM_HOME` and isolated `~/.bum`; no stock credential import/sharing.
- Auth: independent xAI and ChatGPT/Codex OAuth slots, refresh, status, selective logout, and sibling-safe corruption recovery.
- Routing: explicit per-model provider→endpoint→credential→wire resolution; no model-name guessing or global mode.
- Sessions: safe mid-session switching and provider-specific reasoning cleanup.
- Children: Task and Workflow children resolve independently from parents and fail before side effects when credentials are missing.
- Privacy: stock update, feedback transport, Mixpanel, Sentry, internal OTEL, and implicit upload remain unreachable; local tracing/user-owned diagnostics only under explicit policy.

### Critical Pitfalls

1. **False completeness** — reconcile U001–U168, changelogs, all path clusters, additions/deletions/tests, and final differences; zero pending rows at release.
2. **Unsafe unrelated-history operations** — never use an initial merge/bridge as content resolution; freeze and validate the tree first.
3. **Broad replacement erases bum** — protect bum-only auth/routing/privacy/identity/tests; integrate hotspot files semantically.
4. **Security loss in conflicts** — port complete dependency closures and adversarial fixtures first, then re-review after provider/home adaptation.
5. **Provider/auth conflation leaks credentials** — keep inference ownership, connection profiles, helper credentials, and endpoint trust separate.
6. **New `.grok` or egress paths** — extend hermetic filesystem and network audits to workflows, relocation, worktrees, doctor, all startup modes, and shutdown.
7. **Green tests through test deletion** — snapshot discovered tests before integration and block unexplained drops/ignores.
8. **Generated manifest/lock misuse** — use the upstream coherent baseline, settle owning manifests first, regenerate lock state with Cargo, and document the absent public root generator.

## Explicit Unavailable and Excluded Items

### Unavailable at the pinned public snapshot

- Commit summaries mention `ClientToolResult` and `ChatConfig` client-side tool proto work, but the public tree exports no corresponding identifiers/schema and the checked-in proto is unchanged. **Do not fabricate it.**
- U156 names workflow-authoring skills (`create-workflow` / `import-claude-workflow`), but matching `SKILL.md` files are not exposed at the pin. Do not invent or auto-install them.
- U019 `MiniSweAgent:bash` is not trivially traceable by literal fingerprint; determine observable type/schema behavior during planning.
- The root `Cargo.toml` claims generation, but no public generator entry point was identified.
- The upstream initial export lacks `SOURCE_REV`; exact pre-public monorepo lineage cannot be proven.

### Explicitly excluded or not adopted verbatim

- Stock binary/default-run, `GROK_HOME`/`~/.grok`, managed `bin/grok`, or stock credential sharing.
- Stock xAI updater/minimum-version/install/npm behavior.
- xAI feedback identity transport; Mixpanel, Sentry, internal OTEL, trace/codebase/session upload, or remotely enabled product telemetry.
- Replacing bum's mixed catalog, dual OAuth, per-model routing, or cross-provider children with stock single-provider assumptions.
- Treating `[model_providers]` as authorization to use xAI/Codex OAuth or as bum's provider router.
- Sending xAI-only headers/privacy/App Builder/deployment flags on Codex or generic gateways.
- Blind version mass-bumps, stock catalog replacement, invented proto fields, or automatic workflow-skill installation.

Exclusions remain ledger entries and require negative regression tests; security value may not be excluded for convenience.

## Implications for Roadmap

### Phase 1: Immutable Baseline, Provenance, and Completeness Ledger
**Rationale:** No merge base and root skew make trustworthy scope the first dependency.
**Delivers:** frozen public SHA, `SOURCE_REV`, tree/checksums, three-leg inventory, U001–U168 rows, changelog/path/deletion/test clusters, protected bum contracts, validation matrix.
**Addresses:** complete parity and repeatable sync.
**Avoids:** false completeness, moving targets, premature ancestry claims.

### Phase 2: P0 Security Floor
**Rationale:** Known upstream RCE/SSRF/credential/policy fixes cannot wait behind UX work.
**Delivers:** complete security matrix and adapted upstream adversarial tests across hooks, tools, permissions, sandbox, plugins, managed policy, MCP, auth files, and repo trust.
**Addresses:** all U001–U168 security rows classified and verified.
**Avoids:** partial fixes and conflict-induced regression.

### Phase 3: Upstream Foundations and Dependency Closure
**Rationale:** New consumers depend on coherent leaf crates, types, moves, and pinned dependencies.
**Delivers:** upstream-derived integration line; `xai-workflow`; managed text, relocation, diagnostics/editor, auto-GC foundations; exact async-openai/Rhai/dhat pins; test scaffolding.
**Addresses:** structural parity and build topology.
**Avoids:** broad file replacement, deletion mistakes, hand-edited lockfile.

### Phase 4: Product Perimeter and Quiet-Fork Contracts
**Rationale:** New runtime features must use bum identity/home/privacy before they persist or communicate.
**Delivers:** `bum`/`BUM_HOME` adaptation across every new path; updater/telemetry/feedback hard-off; hermetic filesystem and runtime egress gates.
**Addresses:** identity, storage, privacy, distribution constraints.
**Avoids:** `.grok` leakage, stock install repair, hidden telemetry.

### Phase 5: Auth, Catalog, Provider Routing, and Sampling
**Rationale:** This highest-conflict seam must be correct before workflows or imported model UI can spawn requests.
**Delivers:** separate inference ownership vs connection profile/helper types; dual-provider secure auth improvements; Grok 4.5 mixed-catalog merge; distinct supported `max`; captured request goldens and both-provider switching.
**Addresses:** U001/U017/U050/U054/U077/U117/U138/U148/U152 and related auth/model behavior.
**Avoids:** credential crossover, xAI-only defaults, model-name guessing, Codex wire regression.

### Phase 6: Durable Sessions, ACP, Scheduler, and Worktrees
**Rationale:** Workflows depend on stable persistence, cancellation, task lifecycle, and child/worktree ownership.
**Delivers:** eager persistence, rewind IDs, relocation, explicit session state/import, queue combination, usage, scheduler durability, serialized loops, guarded auto-GC.
**Addresses:** runtime/session U-items and 0.2.106–0.2.109 lifecycle behavior.
**Avoids:** orphan state, unsafe GC, implicit remote upload, resume incompatibility.

### Phase 7: Workflow and Cross-Provider Orchestration
**Rationale:** Security, routing, and persistence must exist before enabling another agent-spawn path.
**Delivers:** upstream workflow host/manager/store/tools/scripts/ACP integration, disabled by default; budget/pause/cancel/reconnect/restart tests; Grok→Codex and Codex→Grok workflow children.
**Addresses:** upstream workflow parity without expanding bespoke bum workflow scope.
**Avoids:** workflow-specific auth, inherited parent bearer, pending side effects after missing-provider failure.

### Phase 8: TUI, Terminal, Doctor, Skills, and Documentation
**Rationale:** Presentation should integrate only after backend contracts stabilize, using upstream's split modules and reducer/effect architecture.
**Delivers:** canonical editor, doctor CLI/slash flow, terminal/clipboard/SSH fixes, workflow/task/status UI, skills/plugins/MCP/hooks UX, minimal/fullscreen PTY suites, bum-adapted guides.
**Addresses:** user-visible 0.2.102–0.2.109 parity.
**Avoids:** stock commands/home/install guidance, whole-file UI resolutions, backend-incomplete UI.

### Phase 9: Parity Closure and Optional Ancestry Bridge
**Rationale:** History may record the integration only after content and invariants are proven.
**Delivers:** zero pending ledger rows; final endpoint allowlist; dependency/license/artifact closure; locked builds; v1.0 audit rerun; live dual-provider UAT; future-sync dry run; reviewed tree-preserving two-parent bridge if approved.
**Addresses:** auditable parity and maintainable future synchronization.
**Avoids:** green suite after test loss, ambiguous versions/artifacts, unsafe early bridge.

### Phase Ordering Rationale

- Provenance defines scope; security establishes the minimum safe baseline.
- Leaf/type/dependency foundations precede overlapping product-contract seams.
- Identity/home/privacy are installed before new persistence or network paths execute.
- Provider routing precedes sessions that reconstruct requests and workflows that spawn children.
- Persistence/scheduler/worktree lifecycle precedes workflow orchestration.
- TUI follows backend state contracts; final docs and artifacts describe only verified behavior.
- The ancestry bridge is a final history-recording action, never a merge strategy.

### Research Flags

Phases needing `/gsd-plan-phase --research-phase` or equivalent focused design:

- **Phase 1:** ledger schema, 872-path/change-cluster reconciliation, and exact pin/tree provenance.
- **Phase 3:** generated root manifest maintenance and structural deletion/move mapping.
- **Phase 5:** provider/profile/helper naming, precedence, endpoint trust, Codex wire golden tests.
- **Phase 6:** privacy model for relocation/import and platform-safe worktree GC.
- **Phase 7:** workflow child routing, cancellation ownership, journal/restart semantics, unavailable authoring skills.
- **Phase 9:** exact ancestry-bridge plumbing and proof that the validated tree is unchanged.

Phases using established patterns where separate research can usually be skipped:

- **Phase 2:** upstream code and adversarial tests provide concrete security behavior; planning should focus on adaptation and coverage.
- **Phase 4:** bum's existing identity/home/no-egress contracts and tests are established.
- **Phase 8:** ratatui Action→Dispatch→Effect boundaries and upstream test topology are well documented, though PTY execution remains substantial.

## Major Open Decisions

1. Whether to create the final two-parent ancestry bridge or retain content-lineage synchronization without a bridge. Recommendation: bridge only after full validation because it materially reduces future sync cost.
2. Naming for bum's closed provider concept (`InferenceProvider` recommended) versus upstream connection-profile ID, and exact credential precedence.
3. Default model after merging Grok 4.5; this is a bum product decision, not automatic upstream parity.
4. Compatibility aliases for identity-scoped `GROK_*` variables and project `.grok/workflows` discovery versus bum-primary names/paths.
5. Bum release semver versus upstream 0.2.109 provenance presentation.
6. Whether explicit operator-configured cross-host session mirroring is supported in v1.1; default/implicit remote destinations remain prohibited.
7. Whether authoritative dollar cost can be shown for ChatGPT subscription traffic; recommendation is token counts plus “unknown” rather than estimates.
8. Maintenance procedure for the generated root manifest when the public generator is unavailable.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Exact manifests, lockfile, SHAs, versions, checksums, and isolated upstream builds were inspected. |
| Features | MEDIUM-HIGH | U001–U168, changelogs, docs, tests, and diffs reconcile well; public snapshot omits several named artifacts/contracts. |
| Architecture | HIGH for topology; MEDIUM for intent | No-merge-base, roots, boundaries, and overlaps are verified; feature grouping reconstructs monorepo snapshot intent. |
| Pitfalls | HIGH | Repository-specific conflict surfaces and bum v1 invariants are directly evidenced; external Git/Cargo guidance is supporting only. |

**Overall confidence:** MEDIUM-HIGH. The recommended integration method and safety priorities are strongly supported; a few public-source gaps and product-policy decisions must be resolved during planning.

### Gaps to Address

- Unexported proto/client-tool behavior: keep unavailable until authoritative schema/source appears.
- Workflow authoring skills and MiniSweAgent item: investigate observable behavior without fabrication.
- Root manifest generator: choose a documented reproducible downstream maintenance method.
- Cross-host import/mirroring: privacy review and explicit opt-in design.
- App Builder flags: xAI-only until backend semantics are proven; never leak to Codex.
- Dollar usage: require authoritative provider pricing/account semantics.
- Windows parity: document limits; Unix success is not proof of Windows behavior.
- Upstream may advance: either retain this exact pin or formally restart the inventory.

## Sources

### Primary repository evidence (HIGH confidence)

- Official repository: https://github.com/xai-org/grok-build
- Pinned official commit: https://github.com/xai-org/grok-build/commit/3af4d5d39897855bdcc74f23e690024a5dc05573
- Pinned `SOURCE_REV`, commit bodies, changelog 0.2.102–0.2.109, user guides, manifests, lockfile, schemas, tests, and exact local Git objects.
- bum `.planning/PROJECT.md`, v1.0 milestone audit/requirements, and codebase architecture/concerns.
- Pinned async-openai fork: https://github.com/our-forks/async-openai/commit/95b52ebdedf42143083cf3d6f0e0be7c84e9c808

### Supporting official documentation

- Git merge/unrelated histories, replace, patch-id, range-diff, rerere, and diff documentation: https://git-scm.com/docs
- Cargo manifest/lockfile guidance: https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
- Rhai 1.25.1 feature metadata: https://docs.rs/crate/rhai/1.25.1/features

### Detailed research artifacts

- `.planning/research/STACK.md`
- `.planning/research/FEATURES.md`
- `.planning/research/ARCHITECTURE.md`
- `.planning/research/PITFALLS.md`

---
*Research completed: 2026-07-22*
*Ready for roadmap: yes*
