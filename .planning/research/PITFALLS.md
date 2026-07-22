# Domain Pitfalls: v1.1 Upstream Grok Build Parity

**Domain:** Synchronizing a heavily customized multi-provider Rust fork with official Grok Build
**Researched:** 2026-07-22
**Upstream pin inspected:** public commit `3af4d5d39897855bdcc74f23e690024a5dc05573`, monorepo `SOURCE_REV` `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`
**bum pin inspected:** `128992c5a151e857c728bcbb054c2a8c7c47ce7a`
**Overall confidence:** HIGH for repository-specific findings; LOW for the two external documentation claims because the mandated confidence classifier assigns `webfetch` LOW even when fetching official Git/Cargo pages

## Executive Warning

This is not a normal update merge. `git merge-base HEAD upstream/main` returns no commit. The two public roots are also not content-identical: bum root `c1b5909` and upstream root `c68e39f` differ in 179 files (8,492 insertions and 1,184 deletions). Since their respective roots, bum changed 640 paths and upstream changed 872 paths, with 220 overlapping paths. A direct tip-to-tip diff spans 1,345 files, 138,194 insertions, and 151,932 deletions. Any strategy that treats either root as an automatic common base, or treats upstream tip as a replacement tree, can silently erase v1.0.

The synchronization must therefore be run as an audited content integration with an immutable provenance ledger. Security fixes should be triaged first, structural and low-conflict changes integrated next, and the high-conflict identity/auth/provider/privacy seams adapted deliberately. “Builds successfully” is not parity: stock `.grok` paths, xAI-only routing, updater calls, or telemetry egress can compile and pass upstream tests while violating bum’s product contract.

## Recommended Integration Stages

The roadmap should assign every pitfall to one of these stages.

| Stage | Owner / purpose | Exit condition |
|---|---|---|
| **Stage 0 — Baseline and provenance** | Pin upstream, reconstruct lineage, inventory every delta and disposition | Immutable pins and tree hashes recorded; every upstream sync commit and changed path appears in the ledger; no source integration yet |
| **Stage 1 — Security-first extraction** | Port or equivalently mitigate security changes before feature batches | Each upstream security claim mapped to code and a regression test; no known security fix left merely “for later” |
| **Stage 2 — Structural foundations** | Integrate file splits, new crates/types, durable storage primitives, and low-conflict runtime foundations | Targeted crates compile; structural moves preserve bum-only behavior and tests |
| **Stage 3 — Product-contract adaptation** | Integrate high-conflict home, identity, auth, models, provider routing, subagents, telemetry, and update seams | All static contract gates pass; dual-provider hermetic tests pass; no stock egress path is reachable |
| **Stage 4 — Runtime and TUI parity batches** | Port user-visible upstream features and fixes in reviewable subsystem batches | Each advertised upstream feature/fix is demonstrated or explicitly dispositioned; TUI/PTY suites pass |
| **Stage 5 — Dependency, distribution, and documentation closure** | Reconcile manifests, regenerate lockfile, validate packaging, adapt docs/changelogs | Reproducible locked build produces `bum`; docs/install guidance cannot send users to stock Grok channels |
| **Stage 6 — Parity audit and future-sync seam** | Prove completeness, live contracts, and repeatability | Zero unclassified deltas; v1.0 audit flows re-run; next-upstream dry run produces a bounded review queue |

Security work is deliberately early, but security-sensitive files still receive a second review when Stage 3 adapts product contracts.

## Critical Pitfalls

### Pitfall 1: False provenance completeness

**What goes wrong:** The team pins only `upstream/main`, ports the visible changelog bullets, and declares parity while missing files, tests, deletions, or monorepo changes hidden inside bulk “Synced from monorepo” commits.

**Evidence:** Public upstream has only seven commits from the initial export through the inspected tip. The six sync commits touch 117, 225, 292, 143, 140, and 556 files respectively. `SOURCE_REV` first appears in `8adf901`; the initial public commit has no marker. The latest public commit has 556 changed files and a long embedded change list. Changelogs `0.2.102` through `0.2.109` are useful summaries, not a complete machine-verifiable inventory.

**Consequences:** Applicable fixes disappear without a decision; security patches are assumed present because a release note was read; future maintainers cannot tell whether a difference is intentional or forgotten.

**Detection / warning signs:**
- A parity checklist contains feature names but no upstream commit, path set, or verification.
- Added tests and deleted files have no disposition.
- The ledger count does not reconcile with `git diff --name-status <old-upstream-pin> <new-upstream-pin>`.
- `SOURCE_REV`, public upstream SHA, and bum integration commit are conflated into one identifier.

**Prevention:**
1. Record four immutable identifiers: bum start SHA, upstream public SHA, upstream `SOURCE_REV`, and tree hashes.
2. Build a ledger at both **sync-commit level** and **path/change-cluster level**. Every row must be `ported`, `adapted`, `superseded`, `excluded`, or `not-applicable`.
3. Reconcile the ledger against all 872 paths changed from upstream public root to tip, all upstream additions/deletions, all changelog entries, and each sync commit’s embedded `Changes:` list.
4. Require evidence per row: target bum commit, tests, and rationale. “Already have it” must link the equivalent bum implementation and regression test.
5. Preserve upstream changelogs as provenance data, but write bum-facing release notes separately.

**Stop/go checkpoint:** Do not begin broad integration until the ledger is complete enough to classify all upstream commits and path clusters. Do not close Stage 6 while any path is unclassified.

**Owner:** Stage 0, re-audited in Stage 6.

---

### Pitfall 2: Treating unrelated histories as a normal merge

**What goes wrong:** Running `git merge upstream/main --allow-unrelated-histories`, accepting mass conflict resolutions, or rebasing one root onto the other creates the appearance of a three-way integration without a trustworthy base.

**Evidence:** There is no merge base. The roots differ by 179 files, so even a synthetic assumption that “both roots are the same export” is false. Official Git documentation says merge normally incorporates changes since histories diverged and refuses unrelated histories by default; the override exists as an exceptional safety escape, not a lineage reconstruction tool.

**Consequences:** Whole-file add/add choices replace one side; rename detection invents misleading moves; omissions become hard to distinguish from conflict resolution; future blame and provenance become unreliable.

**Detection / warning signs:**
- Thousands of add/add conflicts resolved with global `ours` or `theirs`.
- A merge commit is created before the integration ledger proves content parity.
- Reviewers see only a giant merge result rather than upstream delta batches.
- The process depends on `git rerere` before resolutions have been independently reviewed.

**Prevention:**
- Reconstruct a **content baseline**, per path where necessary, using bum root, upstream root, upstream sync deltas, and v1.0 commits. Do not assert a common base solely from timestamps or package versions.
- Integrate reviewable change clusters or upstream sync deltas onto an integration branch; preserve source SHAs in metadata.
- Keep a clean worktree and abortable checkpoints. Never integrate into a dirty tree.
- Do not create an ancestry-bridging merge at the start. After full parity, an audited `ours` ancestry bridge may be considered only if its ledger explicitly records exclusions; otherwise continue using prior-upstream-pin → new-upstream-pin diffs for future syncs.

**Stop/go checkpoint:** If a proposed operation cannot show the exact upstream delta it intends to apply and the prior bum behavior it must retain, stop and split it.

**Owner:** Stage 0 strategy; Stage 6 decides future ancestry policy.

---

### Pitfall 3: Broad file replacement erases bum’s product

**What goes wrong:** Copying upstream directories or the entire upstream tree efficiently obtains new code but deletes bum-only modules and overwrites high-value adaptations.

**Evidence:** Direct HEAD-to-upstream-tip comparison reports 335 deletions from bum’s perspective. Upstream lacks bum-only Codex modules (`auth/codex/*`, `codex_refresher.rs`), provider routing/model-switch tests, dual-auth tests, cross-provider subagent tests, home-isolation tests, and `bum` binary wiring. The 220 path overlaps are concentrated in pager (80) and shell (77), exactly where v1.0 made its highest-value changes.

**Consequences:** The build may still work as stock Grok Build while dual OAuth, mixed routing, subagent overrides, quiet defaults, or `~/.bum` disappear.

**Detection / warning signs:**
- A batch has a large deletion count with no deletion review.
- Bum-only integration tests vanish or are “temporarily” disabled.
- Diff review shows upstream files copied wholesale over shell, pager-bin, config, sampler, or model catalog.
- `git diff --stat` is accepted as proof without semantic contract tests.

**Prevention:**
- Maintain a protected bum-only path manifest and fail CI if those files disappear without an approved ledger row.
- Use subsystem batches; inspect both sides’ changes from their own roots before resolving an overlap.
- For high-conflict files, port upstream behavior into bum’s current architecture rather than select a whole side.
- Run contract tests immediately after each high-conflict batch, not only at milestone end.
- Treat large files (`agent/config.rs`, pager dispatch/view modules, settings UI) as mandatory focused-review zones because codebase concerns already identify them as merge-error-prone god files.

**Stop/go checkpoint:** Any batch that unexpectedly removes a bum-only auth, routing, privacy, identity, or test file is a hard stop.

**Owner:** Stages 2–4; protected-path gate owned by Stage 0/6 tooling.

---

### Pitfall 4: Security patches are delayed, partially ported, or lost in conflicts

**What goes wrong:** Feature parity receives attention while security fixes remain buried in bulk sync commits, or a conflict resolution keeps bum behavior but drops the upstream hardening.

**Evidence from actual upstream history:**
- `8adf901`: hook HTTP redirect SSRF bypass fix.
- `98c3b24`: owner-only auth/MCP credentials and crash artifacts; hook matcher fail-closed behavior; plugin SHA pinning and Git operand hardening; signed managed-config downgrade/clock protections; sandbox child-network policy; auth single-flight.
- `7cfcb20`: `web_fetch` non-public-IP blocking, sensitive edit gates, plugin hardening, MCP OAuth RFC 9207 issuer handling.
- `3af4d5d`: prompts for environment-dumping `ps`; blocks `rg --pre` RCE and abused “safe” commands; hardens `kubectl` credential-plugin execution; handles `env -S`; applies Landlock without a controlling TTY.

Several affected files also overlap bum modifications, especially MCP credential home paths and permission code. The codebase concerns already flag permission/sandbox and auth as high-risk.

**Consequences:** Credential exposure, SSRF, permission-gate RCE, unsafe plugin execution, or sandbox bypass can remain exploitable even though the fork claims upstream parity.

**Detection / warning signs:**
- Security is represented by a single “ported security fixes” checkbox.
- A fix’s upstream regression test is absent or only stock-path fixtures pass.
- Security-sensitive diffs are mixed with hundreds of UI changes.
- The implementation preserves behavior but not fail-closed semantics.

**Prevention:**
1. Create a security matrix: upstream SHA, vulnerability class, affected paths, attack precondition, bum overlap, port/equivalent mitigation, regression test.
2. Port the smallest complete security patch dependency closure first. If an exact patch conflicts with bum architecture, implement an equivalent mitigation and preserve/adapt upstream tests.
3. Test redirects at every hop, private/reserved address resolution, credential file mode `0600`, hook compile failures, unsafe shell wrapper parsing, plugin mutable refs, and no-TTY sandbox activation.
4. Re-review security paths after Stage 3, because home/auth/provider adaptation can accidentally undo owner-only writes or token scoping.
5. Never classify a security patch as excluded because it is inconvenient. `Not-applicable` requires a demonstrated unreachable path; `superseded` requires equal-or-stronger tests.

**Stop/go checkpoint:** No general feature integration release candidate while a known upstream security item is unclassified or lacks a test.

**Owner:** Stage 1, with mandatory Stage 3 re-review and Stage 6 security audit.

---

### Pitfall 5: `~/.grok` and stock identity return through new code paths

**What goes wrong:** Existing bum paths remain correct, but newly imported modules use upstream defaults (`GROK_HOME`, `~/.grok`, `grok` managed binary), causing partial storage and identity regression.

**Evidence:** Upstream tip’s `xai-grok-config/src/paths.rs`, `xai-fast-worktree/src/db/mod.rs`, and `xai-grok-mcp/src/credentials.rs` explicitly resolve stock home. Upstream’s pager-bin manifest changes `default-run` and binary name from bum’s `bum` to `xai-grok-pager`; upstream `main.rs` removes bum’s version formatter and restores `.grok` guidance. New upstream worktree auto-GC, session relocation, workflows, managed text, doctor/fix, plugin, and persistence paths create additional storage surfaces that did not exist when v1.0 home isolation was audited.

**Consequences:** Credentials, sessions, worktrees, workflow journals, caches, or MCP tokens can be written into stock Grok state. Stock `grok` and bum can corrupt or expose each other’s data. UI/version/help may again present as Grok Build.

**Detection / warning signs:**
- `BUM_HOME` tests cover old modules only.
- A temp-home run creates `.grok`, reads `GROK_HOME`, or resolves a managed `grok` executable.
- `bum version`, help, doctor output, resume hints, or crash messages say `grok`.
- New path helpers bypass the central bum resolver.

**Prevention:**
- Make the product-home resolver the mandatory dependency for every imported storage feature. No new module may independently read `HOME`, `GROK_HOME`, or append `.grok` for product state.
- Extend the hermetic home audit to auth, MCP, config, sessions, worktrees/auto-GC DB, workflows, relocation journals, plugin cache, memory, crash logs, doctor fixes, and managed application path.
- Add static gates for stock defaults in production code, with explicit allowlists for compatibility documentation, crate names, and test fixtures.
- Preserve `BUM_HOME`-only semantics and the v1.0 out-of-scope decision not to import stock Grok/Codex credentials.
- Re-run `bum version`, CLI help, setup/login/status/logout, doctor, headless, ACP, and crash/recovery identity probes.

**Stop/go checkpoint:** Any unexpected read/write outside the isolated temp bum home blocks the batch.

**Owner:** Stage 3; Stage 4 must extend the audit for every newly enabled feature; Stage 6 runs the full filesystem probe.

---

### Pitfall 6: Upstream auth refactors delete or bypass dual OAuth isolation

**What goes wrong:** Upstream’s new auth single-flight, token-output, or custom model auth-provider work is adopted by replacing bum auth modules, deleting Codex PKCE/device/refresh logic or collapsing provider-scoped records back into a stock xAI credential flow.

**Evidence:** Upstream adds `auth_provider.rs`, `auth_provider_tests.rs`, `single_flight.rs`, and `token_output.rs` but has no bum `auth/codex/*` tree. Tip comparison shows those Codex files as deletions. Upstream auth changes are valuable, but they solve stock/custom-provider token minting, not bum’s ChatGPT/Codex multi-slot lifecycle. v1.0’s audit confirms both provider slots, selective logout, independent refresh, and per-turn Codex refresh as product contracts.

**Consequences:** Last-login-wins behavior, Codex refresh token loss, wrong issuer refresh, logout of both providers, or xAI-only auth UI. Concurrency changes can also reintroduce the historical auth wipe race.

**Detection / warning signs:**
- `auth/codex/*`, `auth_multi_slot.rs`, or `auth_codex_lifecycle.rs` disappear.
- Auth status reports only one provider.
- Upstream’s single-flight is process-global instead of provider/flow scoped.
- A synthetic 403 or failed Codex refresh mutates the xAI slot.
- Session reconstruction uses a stale bearer after upstream lifecycle changes.

**Prevention:**
- Adapt upstream single-flight and secure-file improvements **under** bum’s provider-scoped abstraction; never replace the abstraction with stock auth.
- Preserve independent xAI and Codex login, refresh, expiry, status, and selective logout tests.
- Add concurrency tests for simultaneous xAI/Codex flows, provider-scoped single-flight, cancellation/successor gaps, and refresh-token rotation.
- Preserve the 401-only auth-failure contract; policy/content 403 must not wipe either slot.
- Close the v1.0 debt with a dedicated long-lived Codex child expiry/rotation test while touching this seam.

**Stop/go checkpoint:** No auth refactor proceeds if both providers cannot remain logged in simultaneously in hermetic tests.

**Owner:** Stage 3, security-reviewed with Stage 1 controls.

---

### Pitfall 7: Provider routing regresses to model-name guessing or xAI-only defaults

**What goes wrong:** New upstream model-provider support is assumed equivalent to bum routing and replaces explicit mixed-catalog bindings. Model switching updates a model string but selects the wrong credential/backend, or upstream default-model changes filter away the other provider.

**Evidence:** Upstream 0.2.105 changes the default to Grok 4.5. Upstream 0.2.109 adds `[model_providers.<id>]`, a useful custom-gateway feature but not a substitute for bum’s built-in xAI/Codex routing contract. Upstream lacks bum’s `provider_routing.rs`, `model_switch_gate.rs`, mixed model catalog test, and Codex request-profile modules. The v1.0 audit also records a known reverse-direction catalog-visibility debt for Codex parents.

**Consequences:** Codex bearer sent to xAI, xAI bearer sent to Codex, silent fallback, surprise errors/billing, broken mid-session switching, or subagents constrained to the parent auth mode.

**Detection / warning signs:**
- Provider is inferred from model slug prefixes after integration.
- A default-model update globally filters the mixed picker.
- Host/auth assertions disappear from tests.
- Missing-provider selection sends a network request before prompting login.
- Codex parent cannot see an authenticated xAI child model.

**Prevention:**
- Preserve one authoritative `model -> provider -> endpoint/request adapter -> credential slot` resolution path for main turns and children.
- Integrate upstream `model_providers` as an extension of that route model, not a competing global mode.
- Test exact host, auth slot, headers, request wire profile, and model ID for every built-in route.
- Keep model switching transactional and fail closed before sampling when provider auth is missing.
- Test both cross-provider child directions, explicit child model/effort, default model changes, and provider capability gaps.
- Fix the known candidate-provider visibility bug while upstream model lists and task validation are being reconciled.

**Stop/go checkpoint:** Both directions of hermetic main-turn and subagent route containment must pass before enabling imported model/provider UI.

**Owner:** Stage 3, with picker/TUI behavior in Stage 4.

---

### Pitfall 8: Telemetry and stock auto-update are silently reintroduced

**What goes wrong:** Upstream composition-root and telemetry changes restore Sentry, OTLP, product events, Mixpanel profile sync, update checks, staged updates, or official npm/install guidance.

**Evidence:** Tip comparison removes bum’s `quiet_fork_sentry_disabled`, no-update command path, hard-false update gate, and their tests. Upstream `main.rs` uses config-driven error-reporting disablement and calls real update helpers. Upstream telemetry instrumentation defaults toward configured OTLP export. The README points to `https://x.ai/cli/install.sh`; update code references `@xai-official/grok`. v1.0 debt already notes settings helpers can persist `auto_update=true` out of band.

**Consequences:** Privacy contract breach, secret/context egress, or replacement of bum with stock Grok Build. Config or remote settings can flip behavior that v1.0 intended to be compile/composition-root hard-off.

**Detection / warning signs:**
- Quiet-fork tests are deleted because upstream tests expect update functionality.
- “Disabled by default” replaces “unreachable stock egress.”
- Any startup mode constructs an updater or external telemetry client.
- `bum update` contacts x.ai or invokes npm.
- Remote/config settings can enable Sentry/OTLP/update.

**Prevention:**
- Preserve composition-root hard-off policy for stock updater, minimum-version install, Sentry, external product telemetry, Mixpanel profile sync, and internal OTLP export unless explicitly local/user-owned and approved.
- Keep upstream local tracing and redaction fixes while severing network exporters.
- Add network-deny/loopback-capture tests for interactive, headless, direct stdio, leader, update command, doctor, and shutdown paths.
- Make settings writes normalize `auto_update=false`, closing the v1.0 defense-in-depth debt.
- Maintain a reviewed egress allowlist for model inference and user-configured integrations; all other destinations fail tests.

**Stop/go checkpoint:** Any unexpected socket/DNS attempt during no-network smoke tests blocks release, even if payload is empty or endpoint configuration is absent.

**Owner:** Stage 3; Stage 5 reviews installer/package text; Stage 6 performs runtime egress audit.

---

### Pitfall 9: Generated root `Cargo.toml` and `Cargo.lock` are merged by hand or copied blindly

**What goes wrong:** The generated workspace root is edited as if authoritative, or upstream’s lockfile is copied despite bum-specific package/version/dependency changes.

**Evidence:** Repository instructions mark root `Cargo.toml` generated/read-only. Upstream root manifest changes in the 0.2.105 and 0.2.109 syncs; `Cargo.lock` changes in nearly every sync. Current and upstream toolchains match, but both root manifest and lock differ. Upstream tip switches `async-openai` from crates.io to a pinned Git fork and introduces new dependencies/crates such as `xai-workflow`, `rhai`, `dhat`, and `xai-tty-utils`. Official Cargo guidance says `Cargo.lock` is Cargo-maintained exact resolution and should not be manually edited.

**Consequences:** Missing workspace members, incoherent feature flags, stale checksums, unintended dependency upgrades, irreproducible builds, or loss of bum-specific dev dependencies and binary metadata.

**Detection / warning signs:**
- Conflict markers or manual line edits in `Cargo.lock`.
- Root dependency/version changes land without corresponding per-crate manifest source.
- `cargo check` succeeds only after an unreviewed broad `cargo update`.
- Generated root cannot be reproduced because the generator/source procedure is unknown.

**Prevention:**
1. Adapt per-crate manifests first; preserve bum binary/product metadata explicitly.
2. Identify and document the root-manifest generation source/command before changing generated root. If unavailable in the export, treat that as a Stage 0 blocker requiring an approved reproducible local method—not ad hoc editing.
3. Regenerate `Cargo.lock` from the final manifests with pinned Rust/Cargo, then review package-source, Git SHA, checksum, and duplicate-version deltas.
4. Run `cargo metadata --locked`, targeted checks/tests, and a clean locked release build. Do not run unconstrained `cargo update` as conflict resolution.
5. Audit new Git dependencies and vendored/license notices, especially the upstream `async-openai` fork pin.

**Stop/go checkpoint:** No dependency closure batch is complete until the root manifest and lock are reproducible from reviewed inputs.

**Owner:** Stage 2 for new crate wiring; Stage 5 for final generation/reproducibility.

---

### Pitfall 10: Upstream deletions are copied without semantic adjudication

**What goes wrong:** A deletion is interpreted as “upstream removed it, so bum should too,” even when it is a move, a stock policy decision, or a file bum modified.

**Evidence:** Upstream deletes/splits `settings_modal.rs` into a directory, replaces `diagnostics.rs` with a module tree, retires `terminal_setup.rs` in favor of doctor flows, deletes six built-in skill files, removes several goal tests and signed-managed-config integration files, and deletes old PTY tests. At least four deleted paths are bum-modified: the minimal thinking PTY test, `create-skill`, `help`, `imagine`, plus a signed-managed-config common file. Some deletions clearly correspond to replacement modules/tests; others may reflect upstream product policy rather than obsolete bum functionality.

**Consequences:** Bum capabilities or custom wording vanish; replacement tests omit provider/privacy assertions; duplicate old/new modules remain and compile selectively; stale behavior becomes untested.

**Detection / warning signs:**
- Deletion review asks only whether the old file compiles.
- A moved feature has no mapping from old tests to replacement tests.
- Bum-modified files are deleted without a supersession proof.
- Both legacy and replacement code remain reachable.

**Prevention:**
- Give every upstream deletion one disposition: `accept-delete`, `move/adapt`, `retain-bum`, or `superseded`, with a behavior-level rationale.
- Use rename/similarity evidence only as a hint; verify call sites and tests.
- Port bum assertions into replacement modules before deleting old tests.
- For skills/docs, distinguish stock content policy from technical necessity; retain bum-owned content when still part of the product.
- Ensure only one implementation is reachable after structural migrations.

**Stop/go checkpoint:** No delete-heavy batch is merged while any bum-modified deletion lacks an approved disposition.

**Owner:** Stage 2 for structural moves; Stage 4 for behavior/content decisions; Stage 6 reconciles deletion ledger.

---

### Pitfall 11: Tests go green by losing the tests that define bum

**What goes wrong:** Upstream tests are adopted, old snapshots updated, and the suite passes because bum-specific contract tests were overwritten, removed, ignored, or no longer discovered.

**Evidence:** Bum-only tests include home isolation, dual auth, Codex lifecycle, mixed catalog, provider routing, missing-provider model switch, cross-provider subagents, and quiet-fork tests. Direct tip comparison removes many of these. Upstream adds substantial valuable PTY, auth-provider, workflow, doctor, permission, and security coverage, but stock tests legitimately expect `grok`, `.grok`, updater behavior, and xAI defaults.

**Consequences:** A stock-compatible binary passes while v1.0 regresses. Snapshot updates normalize wrong branding or paths. Compile-only validation misses runtime route and egress failures.

**Detection / warning signs:**
- Test count drops after integration.
- `cargo test` passes only because previously non-ignored contract tests became ignored or unreferenced modules.
- Snapshot review approves bulk updates without checking product strings.
- Upstream and bum suites are treated as mutually exclusive.

**Prevention:**
- Capture a pre-integration test manifest: test binaries, discovered test names, ignored status, and contract category.
- Preserve bum tests and add upstream tests; adapt stock expectations only where product contracts differ.
- Gate on test discovery/count deltas, not only exit code.
- Separate four suites: upstream parity, bum contracts, security, and live/PTY/provider smoke.
- Require manual snapshot review for branding, paths, auth prompts, model/provider labels, telemetry/update copy, and terminal behavior.
- Re-run all eight v1.0 audit flows, including both cross-provider directions, not just unit tests.

**Stop/go checkpoint:** Any unexplained drop in contract-test discovery or increase in ignored tests blocks the phase.

**Owner:** Every integration stage; Stage 6 owns full matrix and v1.0 re-audit.

---

### Pitfall 12: Documentation and changelogs teach stock behavior

**What goes wrong:** Upstream docs are copied verbatim, telling bum users to run `grok`, use `~/.grok`, install from x.ai, expect xAI-only login, enable stock update, or use privacy controls that do not match the quiet fork.

**Evidence:** Upstream README advertises official installers and stock binary; authentication/config/user-guide paths are heavily changed upstream. Changelogs describe `grok` behavior and stock default model. Bum README currently documents dual OAuth, mixed routing, isolated home, and no stock install channel.

**Consequences:** Users overwrite bum, store credentials in the wrong place, misinterpret provider limits, or believe telemetry/update behavior differs from reality. Tests/docs drift can also conceal unsupported provider gaps.

**Detection / warning signs:**
- New docs contain `~/.grok`, `GROK_HOME`, `grok login`, x.ai installers, or `@xai-official/grok` outside a provenance/compatibility allowlist.
- Changelog claims a feature is available before its bum-adapted path is verified.
- Provider-specific limitations disappear from docs.

**Prevention:**
- Treat docs as code: every imported feature batch updates bum-facing docs in the same stage.
- Keep upstream changelog files for traceability, clearly labeled as upstream source history; create a bum parity/adaptation note listing exclusions and provider gaps.
- Add static documentation lint with contextual allowlists.
- Verify all commands in a temp bum home and ensure install/update guidance points only to approved local/team distribution.

**Stop/go checkpoint:** Stage 5 cannot close while docs contain actionable stock install/home/auth guidance presented as bum behavior.

**Owner:** Stage 4 feature owners draft updates; Stage 5 performs global docs audit.

---

### Pitfall 13: Distribution metadata produces the wrong artifact or stock package

**What goes wrong:** Source integration succeeds, but release commands build `xai-grok-pager`, package/update scripts expect `grok`, versions diverge, or npm wrappers fetch official binaries.

**Evidence:** Bum manifest currently sets Rust package version `0.1.220-alpha.4`, `default-run = "bum"`, and binary `bum`; upstream tip sets Rust packages to `0.2.109`, default-run/binary `xai-grok-pager`. Upstream npm package metadata still pins platform packages at `0.1.220-alpha.4`, demonstrating that repository version surfaces are not automatically synchronized. Both trees retain official `@xai-official/grok` wrapper/install code, but official public distribution is explicitly out of scope for bum v1.1.

**Consequences:** CI tests one artifact and ships another; `bum update` installs stock Grok; scripts cannot find the binary; version output lies about parity; licenses/notices lag new dependencies.

**Detection / warning signs:**
- `cargo build -p xai-grok-pager-bin` no longer yields `target/.../bum`.
- Packaging tests invoke or download `@xai-official/grok`.
- Rust package, binary output, changelog, and internal protocol version are bumped as one undifferentiated value.
- Release archive contains both `bum` and an unintended stock executable.

**Prevention:**
- Decide version semantics explicitly: bum release version is independent, while the integrated upstream public SHA and `SOURCE_REV` are provenance fields.
- Preserve `bum` bin/default-run and update every harness/build script that locates the artifact.
- Exclude official npm/x.ai distribution behavior from executable bum paths; retain source only when needed for lineage/tests and mark it excluded.
- Build clean debug and `release-dist` artifacts; execute `bum --help`, `bum version`, login/status, headless, ACP, and TUI smoke against the produced artifact.
- Audit licenses and third-party notices for new crates, Git forks, and workflow runtime dependencies.

**Stop/go checkpoint:** Do not release if any normal bum command can fetch/install a stock package or if artifact name/version/provenance are ambiguous.

**Owner:** Stage 5, with privacy recheck in Stage 6.

---

### Pitfall 14: “Applicable” and “excluded” remain subjective

**What goes wrong:** Incompatible upstream behavior is silently omitted, while difficult but applicable changes are labeled not-applicable. Conversely, stock-only behavior is ported merely to achieve a zero textual diff.

**Consequences:** Parity claims become unauditable; privacy regressions enter under “full parity”; valuable fixes are skipped without scrutiny.

**Explicit exclusions policy:**

| Disposition | Allowed meaning | Required evidence |
|---|---|---|
| **Ported** | Upstream behavior lands materially unchanged | Upstream SHA/path + bum commit + adapted upstream test |
| **Adapted** | Same user/security value, modified for bum contracts | Difference rationale + bum contract tests + upstream parity test |
| **Superseded** | Existing bum implementation is equal or stronger | Code mapping + comparative tests; not merely similar names |
| **Excluded** | Behavior directly conflicts with an approved product constraint | Named constraint, scope boundary, and proof exclusion does not drop security value |
| **Not applicable** | Code path/platform/product capability cannot execute or is absent | Reachability/platform evidence; security items require equivalent-risk analysis |

**Pre-approved exclusion categories:** stock Grok branding and default home; reading/sharing stock credential stores; stock x.ai auto-update/minimum-version installation; xAI product telemetry phone-home; official x.ai/npm distribution for bum; xAI-only global provider assumptions. These are still ledger rows, not silent omissions.

**Never valid by itself:** “too hard,” “large diff,” “tests fail,” “probably internal,” “already works,” or “not in changelog.” Security patches cannot be excluded without an equal-or-stronger mitigation.

**Owner:** Stage 0 defines the ledger; each stage proposes dispositions; Stage 6 approves final exclusions.

---

### Pitfall 15: The next upstream sync is as expensive as this one

**What goes wrong:** v1.1 is completed through one-off manual archaeology, but no durable prior-pin, contract overlay, or sync tooling remains.

**Evidence:** Upstream is a periodic monorepo export with huge aggregate commits; current histories have no ancestry; current bum has no `SOURCE_REV`. Without a recorded checkpoint, future researchers must repeat root comparison and infer which stock differences are intentional.

**Consequences:** Security lag, repeated identity/privacy regressions, unverifiable omissions, and rising fork-maintenance cost.

**Detection / warning signs:**
- Final tree has no machine-readable upstream pin/disposition ledger.
- Future-sync instructions begin with “diff HEAD against upstream/main.”
- Bum customizations remain scattered without contract tests or protected-path/egress gates.
- A new upstream pin cannot be dry-run without modifying source.

**Prevention:**
- Store an immutable sync manifest containing prior/current upstream public SHA, `SOURCE_REV`, tree hash, integrated bum SHA, and ledger schema.
- Future syncs always diff **last integrated upstream pin → new upstream pin**, never bum tip → upstream tip as the source-change inventory.
- Keep bum’s divergence as a small explicit overlay of contracts: product home/identity, provider/auth/routing, cross-provider orchestration, privacy/update, distribution.
- Add a read-only sync report that classifies added/modified/deleted paths, flags protected overlaps, extracts upstream change summaries, and fails on unclassified security keywords/paths.
- Preserve rerunnable contract, egress, home-isolation, and artifact tests independent of upstream file layout.
- Perform a Stage 6 dry run against the same pin (expect zero queue), then against the next available pin or a synthetic upstream commit (expect a bounded classified queue).

**Stop/go checkpoint:** Milestone is not complete until the sync procedure can reproduce the current classification from clean checkout without relying on one researcher’s shell history.

**Owner:** Stage 6.

## Moderate Pitfalls

### Upstream feature dependency slicing
**What goes wrong:** A visible feature is cherry-picked without its type, persistence, protocol, or test-support dependencies. Workflows are the clearest example: upstream adds `xai-workflow`, shell workflow manager/store, pager overlays, tool implementation, ACP ingest, and Rhai assets.
**Prevention:** Build dependency-closure maps per feature; compile and test at each layer. Do not port only UI commands.
**Owner:** Stages 2 and 4.

### Protocol and persistence compatibility drift
**What goes wrong:** New session relocation, durable append, response IDs, workflow journals, or ACP methods make old bum sessions unreadable or alter provider conversation continuity.
**Prevention:** Golden fixtures from v1.0; forward/backward read tests; migrations must be idempotent and remain under `~/.bum`; test both providers after resume/rewind/compaction.
**Owner:** Stages 2–4.

### Config precedence and managed policy regressions
**What goes wrong:** New managed text, model providers, privacy settings, or auto-GC keys change layer precedence or permit remote settings to override bum invariants.
**Prevention:** Table-driven precedence tests; product privacy/home/provider invariants are requirements-level and cannot be remotely flipped. Preserve signed-policy hardening while adapting paths.
**Owner:** Stages 1–3.

### Upstream default changes silently alter product behavior
**What goes wrong:** New Grok 4.5 defaults, `max` effort, queue combining, shell environment inheritance, or workflow defaults apply to Codex models without capability review.
**Prevention:** Defaults are provider-aware; imported defaults require explicit bum decision and migration test. Never infer Codex support from Grok support.
**Owner:** Stages 3–4.

### Performance and lifecycle regressions hidden by functional tests
**What goes wrong:** Larger state, workflow runtime, auto-GC, status watchers, or auth refresh actors introduce startup/turn latency, memory growth, or shutdown races.
**Prevention:** Retain upstream dhat/lifecycle tests, add representative dual-provider sessions, and test leader/direct/headless shutdown. Watch known idle-unload and child-refresh debt.
**Owner:** Stages 2, 4, and 6.

## Phase-Specific Warning Matrix

| Stage topic | Likely pitfall | Required mitigation / proof |
|---|---|---|
| 0: pin and inventory | Wrong base; incomplete provenance | No merge base acknowledged; roots compared; public SHA + `SOURCE_REV` + trees recorded; ledger reconciles counts |
| 1: security | Security fix buried or weakened | Security matrix, attack regression tests, equivalent-mitigation rule |
| 2: structure | Whole-file replacement; deletion mistakes | Protected-path manifest, behavior migration before delete, targeted crate tests |
| 2: generated workspace | Hand-edited generated root | Reproducible generation procedure and reviewed per-crate inputs |
| 3: home/identity | New `.grok` writers | Temp-home filesystem audit covering all new persistence features |
| 3: auth | Stock refactor removes Codex slots | Dual login/logout/refresh/concurrency and 403-no-wipe tests |
| 3: routing | New model providers bypass built-ins | Host/header/slot/wire assertions; both child directions |
| 3: privacy | Config re-enables egress | Composition-root hard-off + runtime socket/DNS capture |
| 4: TUI/features | UI appears complete but backend closure missing | Feature dependency map, PTY tests, provider-specific capability check |
| 4: defaults | Grok default applied to Codex | Provider-aware default/effort/tool tests |
| 5: lockfile | Blind copy or broad update | Cargo-generated lock, source/SHA/checksum review, `--locked` build |
| 5: distribution | Wrong binary/stock installer | Clean artifact smoke; no stock package fetch; independent bum version + provenance |
| 5: docs | Stock commands/home/privacy copied | Static docs lint + command verification |
| 6: audit | Green suite after test loss | Test-discovery comparison, eight v1.0 flows, deletion/exclusion reconciliation |
| 6: maintainability | One-off sync | Clean-checkout sync report and next-pin/synthetic dry run |

## Stop/Go Gates

### Gate A — Inventory ready (after Stage 0)
**GO only if:** upstream pin and `SOURCE_REV` are immutable; roots and no-merge-base are documented; additions/modifications/deletions and security items are ledgered; protected bum paths are known.

### Gate B — Security floor restored (after Stage 1)
**GO only if:** every advertised security change is ported, superseded by tested stronger behavior, or proven unreachable; auth/credential/sandbox/permission tests pass.

### Gate C — Structural integration safe (after Stage 2)
**GO only if:** no unexplained bum-only deletion; new crates and file moves compile; legacy behavior tests migrated; generated workspace method is known.

### Gate D — Product contracts intact (after Stage 3)
**GO only if:** isolated-home, dual OAuth, per-model routing, cross-provider subagents, quiet telemetry/update, and identity tests all pass hermetically.

### Gate E — Feature parity demonstrated (after Stage 4)
**GO only if:** each applicable changelog/change-list item has runnable evidence and provider gaps are documented; PTY/runtime regressions are green.

### Gate F — Release candidate reproducible (after Stage 5)
**GO only if:** clean `--locked` build yields `bum`; dependency and license review is complete; docs/distribution do not point to stock behavior.

### Gate G — Milestone complete (after Stage 6)
**GO only if:** zero unclassified upstream deltas; exclusions approved; v1.0 audit flows still pass; no unexpected egress/home writes; future-sync dry run succeeds.

## Prevention Test Minimums

1. **Provenance:** script verifies current ledger pins, `SOURCE_REV`, expected upstream changed-path count, and zero unknown dispositions.
2. **Identity/home:** temp `HOME` + `BUM_HOME`; assert all product writes stay under bum home and no `.grok`/`.codex` reads or writes occur.
3. **Binary:** clean build emits only intended `bum` product artifact; help/version/setup/doctor/crash/resume text retains bum identity.
4. **Auth:** simultaneous xAI + Codex login state; selective logout; independent refresh; concurrent single-flight per provider; 403 does not wipe; owner-only files.
5. **Routing:** exact endpoint, credential slot, header set, request profile, and model for Grok and GPT; missing-provider preflight sends zero requests.
6. **Switching/subagents:** same-session Grok ↔ GPT; Grok parent → Codex child; Codex parent → xAI child; explicit child effort; refresh across child token expiry.
7. **Privacy/update:** network capture across startup modes and update command; only explicitly allowed inference/user integrations; no Sentry/Mixpanel/product OTLP/x.ai update/npm traffic.
8. **Security:** redirect SSRF, non-public web fetch, hook fail-closed, plugin pin/operands, credential permissions, RFC 9207 MCP callback, `env -S`, `rg --pre`, `ps`, `kubectl`, safe-command RCE corpus, no-TTY sandbox.
9. **Persistence/protocol:** v1.0 sessions resume after integration for each provider; rewind/compaction preserves tool pairing and provider continuity; new relocation/workflow state stays under bum home.
10. **Deletion/test integrity:** protected files and discovered contract tests cannot vanish without approved ledger entries; ignored-test changes are reviewed.
11. **Dependency/release:** `cargo metadata --locked`; targeted crate checks/tests; clean release-dist build; dependency-source and license audit.
12. **Docs:** lint actionable stock home/install/auth/update strings; execute documented bum commands.

## Sources

### Repository evidence — HIGH confidence
- `.planning/PROJECT.md` — non-negotiable v1.1 product contracts and unrelated-history context.
- `.planning/codebase/CONCERNS.md` — generated root manifest, auth wipe history, permission/sandbox fragility, test gaps, and large conflict-prone files.
- `.planning/milestones/v1.0-MILESTONE-AUDIT.md` — 26 validated v1.0 requirements, eight end-to-end flows, and residual debt that synchronization must not erase.
- Local Git objects for bum `128992c5...` and fetched official `upstream/main` `3af4d5d3...`; `git merge-base`, root/tip tree comparisons, path overlap, additions/deletions, sync commit messages, `SOURCE_REV`, and changelogs `0.2.102`–`0.2.109`.
- Actual diffs in config paths, fast-worktree DB, MCP credentials, pager-bin, auth, model/provider, sampler, telemetry, updater, manifests, tests, docs, and packaging.

### Official external documentation — classifier LOW
- Git merge documentation: https://git-scm.com/docs/git-merge — unrelated histories are refused by default and the override is exceptional.
- Cargo Book, `Cargo.toml` vs `Cargo.lock`: https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html — lockfile is Cargo-maintained exact resolution and should not be manually edited.

The external claims are supporting guidance only. The core recommendations are derived from directly inspected repository history and code.

## Research Gaps Requiring Stage-Specific Resolution

- The exported repository states root `Cargo.toml` is generated, but this research did not identify a documented generator entry point in the inspected tree. Stage 0/2 must resolve this before manifest reconciliation.
- The initial upstream public commit lacks `SOURCE_REV`, and its tree differs from bum’s initial root. Exact pre-public monorepo lineage cannot be proven from public Git alone; use content archaeology and record this limitation.
- Official upstream commits after `3af4d5d3...` may appear before execution begins. Stage 0 must either freeze this pin for v1.1 or explicitly restart inventory at a newer pin; never let the target float mid-integration.
- Platform-complete distribution testing (especially Windows) remains constrained by the codebase’s best-effort Windows status; document any platform-specific parity limit rather than inferring success from Unix builds.
