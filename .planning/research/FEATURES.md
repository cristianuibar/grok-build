# Feature Landscape: v1.1 Upstream Grok Build Parity

**Project:** bum
**Domain:** Rust terminal AI coding agent fork synchronization
**Researched:** 2026-07-22
**Overall confidence:** MEDIUM (official Git/GitHub evidence is direct and cross-checked, but the confidence seam classifies verified web evidence as MEDIUM; the public sync summaries are not a full internal commit history)

## Scope and Pinned Baseline

This milestone should target **official public commit [`3af4d5d39897855bdcc74f23e690024a5dc05573`](https://github.com/xai-org/grok-build/commit/3af4d5d39897855bdcc74f23e690024a5dc05573)**, version **0.2.109**, dated 2026-07-21. Its root `SOURCE_REV` and commit-body `Source-Revision` both pin the corresponding monorepo revision to **`0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`**.

The bum research-time tip is **`128992c5a151e857c728bcbb054c2a8c7c47ce7a`**. `git merge-base HEAD upstream/main` produces no SHA: the histories are disconnected. Bum's root commit, `c1b5909ec707c069f1d21a93917af044e71da0d7`, and upstream's similarly named public root, `c68e39f60462f28d9be5e683d9cbe2c57b1a5027`, are **not tree-equivalent** (179 paths differ). Therefore neither a normal merge nor “everything after upstream's first commit” is a complete scope definition.

The correct parity equation is:

```text
pinned upstream tip tree
− deliberately retained bum contracts
− proven equivalent/superseding bum implementations
− explicit, ledgered exclusions
= v1.1 integration scope
```

### Completeness strategy

The requirements and implementation ledger must reconcile all four evidence layers:

1. **Six public sync commit bodies:** 168 explicit `Changes:` bullets across `8adf901…`, `98c3b24…`, `7cfcb20…`, `ba76b0a…`, `a881e67…`, and `3af4d5d…`.
2. **In-tree release notes:** versions **0.2.102–0.2.109** are newer than bum's imported 0.2.101-era root and include user-visible items absent from generic commit subjects. `crates/codegen/xai-grok-shell/CHANGELOG.md` is authoritative product evidence.
3. **Direct content delta:** bum root → pinned upstream changes 938 paths; upstream public root → pinned upstream changes 872 paths / 127,391 insertions / 54,015 deletions. The final implementation audit must use current bum tip → pinned upstream path/content comparison, not line-count equality.
4. **Tip documentation and tests:** all 25 user-guide files, CLI parsing, slash registry, permission tests, workflow modules, and config schemas at the pinned tree. Commit summaries are indexes, not proof that all supporting behavior was captured.

Each source item needs one terminal disposition: **PORT**, **ADAPT**, **ALREADY/SUPERSEDED**, or **EXCLUDE**, plus evidence and verification. “No code diff remains” is not enough; a changed upstream path may have been rewritten in bum for dual-provider behavior.

## Classification Rules

| Disposition | Meaning for v1.1 | Required evidence |
|---|---|---|
| **PORT (table stake)** | Applicable upstream behavior should match at bum's surface. | Upstream SHA/file, bum implementation, focused automated acceptance. |
| **ADAPT** | Preserve upstream intent while changing names, paths, provider routing, auth, privacy, or product semantics. | Upstream evidence + explicit bum invariant + provider/path-specific tests. |
| **ALREADY / SUPERSEDED** | Bum already has equivalent or stronger behavior. Do not port a second implementation. | Existing bum source/test SHA and an equivalence test against upstream's observable contract. |
| **EXCLUDE** | Intentionally incompatible with bum. | Exact source item, rationale, negative acceptance proving it stays absent/off. |

**Non-negotiable adaptations:** user-facing `grok` → `bum`; `~/.grok`/`GROK_*` product-home defaults → `~/.bum`/`BUM_*` where identity-scoped; keep compatibility identifiers only where protocol/config compatibility requires them; preserve xAI + Codex OAuth slots, mixed model routing, cross-provider children, and provider-specific credentials; no stock updater, feedback transport, internal product analytics, Sentry, or internal OTEL exporter.

## Table Stakes: Security First

These are not optional polish. They prevent credential disclosure, permission bypass, repository code execution, or policy downgrade. Unless an equal-or-stronger bum test already proves the behavior, port them before feature work.

| Security cluster | Upstream evidence | Parity acceptance implication | Complexity / dependency |
|---|---|---|---|
| **Hook HTTP redirect SSRF** | `8adf901` U002; `xai-grok-hooks/src/runner/http.rs` disables redirects and tests that a vetted URL cannot redirect to private/internal targets. | A hook request must not follow redirect targets; private, link-local, metadata, and mapped ranges remain blocked. | MEDIUM; HTTP client policy and integration fixtures. |
| **Bearer scoping and credential files** | `98c3b24` U017, U040–U042; memory bearer restricted to first-party xAI endpoints; auth/MCP credentials, crash dumps, and agent-id cache forced owner-only. | xAI bearer never reaches non-first-party embeddings; `~/.bum` auth/MCP/crash/cache files are `0600` on Unix, including overwrite paths. Codex bearer gets an equivalent first-party boundary. | HIGH; touches dual auth, memory, atomic writes, crash handler. |
| **Plugin supply-chain pinning** | `98c3b24` U025/U027; `require_sha`, shared pin gate; `7cfcb20` U064 hardens Git operands and U069 removes default-skill auto-install/purges legacy installs. | Mutable remote refs are rejected when pinning is required; malicious Git operands cannot become options; no silent marketplace skill install. | HIGH; plugin install, marketplace, migration behavior. |
| **Managed-policy authenticity** | `98c3b24` U026/U046; `a881e67` U122; signed managed claim, rollback-resistant cache, per-fetch nonce/replay probe. | Removing a sidecar cannot downgrade managed state; stale clocks/replays do not revive old policy. Preserve bum's existing signed-policy seam if stronger, but prove equivalence. | HIGH; existing bum signed-policy changes are conflict-prone. |
| **Shell and hook permission gates** | `98c3b24` U024/U029/U030/U035/U037/U038; `3af4d5d` U140/U142/U163. | Unsafe environment prefixes, redirects, sourced scripts, file hooks, inline shell file access, malformed `fail_closed`, opaque/abused safe commands, and `env -S`/`--split-string` payloads cannot bypass ask/deny. Hook matcher recompilation fails closed. | VERY HIGH; parser/policy/manager integration. |
| **Sandbox restrictions** | `98c3b24` U023/U034; `3af4d5d` U158; exact website policy, inherited child network restrictions, Landlock without controlling TTY. | Child agents inherit network limits; headless/no-TTY execution still receives Landlock; website exceptions remain exact. | HIGH; OS-specific tests on Linux plus documented macOS limits. |
| **MCP identity/OAuth safety** | `7cfcb20` U063/U066; ambiguous tool IDs rejected; RFC 9207 `iss` carried through OAuth. | Ambiguous IDs fail instead of dispatching the wrong tool; issuer-bound token exchange succeeds and validates correctly. | MEDIUM. |
| **Prompt/rules injection and repo RCE** | `7cfcb20` U060/U080/U081/U095; project roles/personas gated, System-Reminder XML injection fixed, remote workspace LSP trust no longer hardcoded, sensitive edit targets gated. | Malicious project instructions cannot close/remap the reminder envelope; untrusted repos cannot activate LSP code execution; protected files prompt/deny. | VERY HIGH; must precede broad workflow enablement. |
| **`web_fetch` network boundary** | `7cfcb20` U089; `xai-grok-tools/.../web_fetch/ssrf.rs`. | Non-public and metadata IPs blocked; local opt-in permits explicit loopback hosts only, not private subnets. | MEDIUM. |
| **Auto-mode RCE hardening** | `ba76b0a` U109/U114; `3af4d5d` U134/U135/U140/U142/U144. | Classifier denials continue with bounded retries; unvetted env prefixes classify; environment-dumping `ps`, `kubectl --kubeconfig` credential plugins, packed `env` scripts, safe-command abuse, and `rg --pre` require permission or deny. | VERY HIGH; security phase should retain exact upstream adversarial fixtures. |

**Recommendation:** make this the first implementation phase. A feature-parity release that retains any upstream-fixed RCE/SSRF/policy bypass is not parity.

## User-Visible Release Inventory (0.2.102–0.2.109)

This is the user-centric requirement seed. Items repeated in commit summaries appear once here and are linked to the source-summary ledger later.

### 0.2.102 — navigation, editing, auth, and terminal parity

**PORT:** session-only `--minimal`/`--fullscreen`; `/jump`; clickable `/timeline`; fleet fallback for permission mode; compact edit summaries; normal terminal-style tab completion in `!bash`; requirements-managed voice hiding; minimal-only bold prompts; marketplace-name qualifier; collapsed consecutive edits; shell env/cwd continuity; correct background tool icons; rewind transcript/permission fixes; glob parity; network-home SQLite safety; wrapped-line Home/End; dashboard viewer keys; sandbox-profile conflict warnings; Ctrl+Y recovery; cancelled-turn prompt suppression; PageUp/PageDown placement; VS Code Remote-SSH links; minimal folder trust; qualified colliding skills; headless `--no-wait-for-background` drain; detailed 429 copy; minimal `/copy`; recap/compaction improvements.

**ADAPT:** xAI login adds workspace scopes without altering Codex scopes; live re-login immediately updates only the affected provider; allowed subagent model slugs must include bum's mixed provider catalog; updater/installer self-heal must not reactivate stock xAI installation or update behavior; billing URLs remain xAI-only and honestly labeled.

Evidence: changelog 0.2.102; U001, U006–U014, U028, U031, U039, U045, U047/U050/U052–U055.

### 0.2.103 — plugin integrity, shell continuity, MCP setup, SSH resilience

**PORT:** plugin `require_sha`; named plugin-MCP setup preferences; exit hint with title/last exchange; SSH-wrap tip; GitHub PR color fix; early-cancel turn-slot race fix; complete queued-prompt copy; terminal restoration after child death; confidence-aware clipboard feedback; shell behavior after cwd deletion.

**ADAPT:** model BYOK/API-key voice auth must resolve through the selected model/provider without leaking xAI or Codex credentials; agent-entrypoint self-heal may repair bum binaries only and may never stage/adopt stock updates.

Evidence: changelog 0.2.103; U015, U021/U022, U025/U027/U028, U036/U039, U045/U047/U048.

### 0.2.104–0.2.105 — status, model UX, rules, compaction, rendering

**PORT:** persistent background status; cleaner retry/rate-limit messages; `/btw` in minimal; “Snap prompt to top on send”; `/summarize` alias; login-shell env/aliases/functions snapshot; multiline syntax highlight; home-scope `<root>/rules`; cancelled-task wake suppression; dashboard return-to-previous-agent; MCP RFC 9207 issuer; leader roster; `tool_choice=auto` compaction; bounded scrolling; canonical picker/editor behavior.

**ADAPT:** upstream's Grok 4.5 default and high/medium/low catalog changes are xAI catalog updates, not permission to erase bum's GPT-5.6 rows or mixed list. Merge provider metadata and retain remote-prefetch append protection.

Evidence: changelog 0.2.104/105; U058/U065–U088, U092–U106.

### 0.2.106 — clipboard and scheduler lifecycle

**PORT:** recoverable copy-file fallback; honest SSH/Apple Terminal toasts; light-terminal minimal syntax colors; scheduled task upsert by `task_id`; one-shot retirement; durable delete and lifecycle version clock.

**ADAPT:** expose a bum-named `BUM_CLIPBOARD_NO_OSC52`; optionally continue accepting `GROK_CLIPBOARD_NO_OSC52` as a documented compatibility alias, but do not make the stock name primary.

Evidence: changelog 0.2.106; U111–U115, U136/U168.

### 0.2.107 — durable sessions, hooks, custom auth, minimal UX

**PORT:** stop hooks can return feedback and continue; cross-host mirrored session state/import; eager per-frame persistence; auto-mode deny-and-continue with limits; startup status includes unstaged/untracked files; randomized server-tool descriptions remain accurate; minimal commits full thinking and one-line successful lookups; no-op commands stop spinning; cached-context recap; Ctrl+B backgrounds commands and Ctrl+G opens tasks in fullscreen.

**ADAPT:** named rotating auth providers are reusable custom-model gateway profiles, not replacements for bum's xAI/Codex `ModelProvider` routing enum. Namespace/types must avoid conflating “backend provider” with “credential helper.” OAuth popup fixes must be tested against both providers.

**EXCLUDE:** transmitting feedback/submitter identity to xAI remains off; local diagnostic feedback files may remain if they cause no network traffic.

Evidence: changelog 0.2.107; U107–U123 and upstream `agent/model_providers.rs` at `3af4d5d`.

### 0.2.108 — diagnostics and relocation

**PORT:** read-only `bum doctor`, JSON report, `bum doctor fix terminal.ssh-wrap --yes`, `/doctor`, centralized terminal probes, Voice diagnostics, moved-directory/cross-machine resume, headless-remote image paste, screen-mode-aware actions, minimal external editor.

**ADAPT:** commands/help/config paths must say `bum`, `~/.bum`, and bum's wrapper alias. Ctrl+G is external editor in minimal and tasks pane in fullscreen.

Evidence: changelog 0.2.108; U137/U145/U150/U155/U157/U159–U165.

### 0.2.109 — usage, batching, skills, effort, workflows, worktrees

**PORT:** current-session token/cost in `/usage`; queued follow-up batching via `[ui].combine_queued_prompts`; user skills never overwritten; full Markdown reads under `skills/`; distinct side-call conversation IDs; clearer subagent watcher; duplicate parked markers fixed; non-overlapping `/loop`; scheduler durability; automatic worktree GC with kind TTLs, liveness/CWD guards, throttling, stale Git registration cleanup, optional rebuild.

**ADAPT:** `/usage` needs provider/model attribution where costs are available and must not invent Codex dollar values; `max` becomes a true tier above `xhigh` only when the selected catalog advertises it. Bum currently canonicalizes `max → xhigh`, so U148 is **not already present**. Preserve Codex clamp/catalog semantics while adding the upstream variant. `[model_providers.<id>]` must coexist with bum's provider-bound xAI/Codex model routing.

Evidence: changelog 0.2.109; U124–U168; `xai-fast-worktree/src/auto_gc.rs`, `xai-prompt-queue/src/combine.rs`, `xai-grok-pager/src/doctor_cmd/`, `xai-grok-shell/src/agent/model_providers.rs`.

## Major Operational Feature Areas

### 1. Session durability, relocation, rewind, and cross-host import

| Capability | Disposition | Acceptance implication | Complexity |
|---|---|---|---|
| Durable append + acknowledged persistence | PORT | Updates survive crash boundaries; in-memory state converges to acknowledged disk state without duplicate items. | HIGH |
| Rewind response identity | PORT | User-message chunks carry response IDs; rewind executes against the requested target response, including old/new transcript mixtures. | HIGH |
| Moved working directory resume | PORT | Session search/resume finds a moved project; working-directory switch is persisted and compacted correctly. All state remains under `~/.bum`. | HIGH |
| Mirrored state/import ACP | PORT with privacy gate | `x.ai/session/state` and `x.ai/session/import` interoperate when explicitly configured; no automatic upload or stock cloud dependency. | HIGH |
| Persistent subagent output / bounded long-lived state | PORT + dual-provider regression | Child results survive and memory remains bounded; child provider/model/credential provenance is preserved. | HIGH |
| Side-model conversation IDs | PORT | Compaction/recap/side calls do not pollute the parent conversation ID or cross provider attribution. | MEDIUM |

Evidence: U016/U018/U033/U108/U120/U124/U125/U137/U138/U150/U161/U167; `xai-chat-state`, shell `session/storage/relocation`, `extensions/session_state.rs`.

### 2. Background tasks, scheduler, dashboard, and worktrees

PORT persistent status/elapsed cues, correct icons, cancellation wake suppression, headless drain, dashboard roster/peek lease hardening, Ctrl+B behavior, serialized `/loop`, scheduler upsert/durable delete/version clock, and worktree auto-GC. Worktree GC must never delete a live parent or child workspace; macOS CWD scan and Unix PID checks are required guards, while Linux age cleanup and non-Linux dead-only behavior should follow upstream.

Dependencies: durable persistence before scheduler/workflow lifecycle; task/subagent state before GC; terminal event model before shortcut changes. Complexity is **HIGH** because bum's cross-provider subagents create more live worktrees and credential-gated child states than stock.

### 3. Canonical TUI editing and terminal diagnostics

PORT the canonical ratatui editor across dialogs, search, Persona, extensions, dashboard, settings, pickers, and TextArea; paste/terminal parity; sticky-header PageUp/PageDown fix; bounded presentation latency; multiline highlighting; semantic links/VS Code SSH delegation; clipboard confidence/file fallback/kill switch; external editor in minimal; standalone and slash doctor; image paste over wrapped remote sessions.

Acceptance must include PTY coverage in fullscreen and minimal modes, dark/light polarity, SSH/container clipboard ambiguity, and no persistence of one-session screen flags. Complexity **HIGH**, but this is largely separable from model/provider routing.

### 4. Skills, plugins, MCP, hooks, and project rules

PORT qualified colliding skills, non-overwrite and full Markdown reads, plugin SHA pins, no default-skill auto-install, MCP setup preferences and issuer handling, ambiguous tool rejection, stop-hook feedback, fail-closed matcher compilation, project hook/script gates, global rules discovery, and project role/persona trust gates.

Workflow-authoring docs referenced by U156 must be reconciled with actual tree content: no obvious `create-workflow`/`import-claude-workflow` `SKILL.md` files are present at the pinned public tip, while workflow docs/config/runtime are present. Record this as a **public-source gap**, not as silently completed parity.

### 5. Workflow runtime

The pinned tip adds `xai-workflow`, shell workflow host/manager/registry/store/tracker, a built-in `deep_research.rhai`, workflow tools, `/deep-research`, `/workflow`, `/workflows`, a dashboard, and `[workflows] enabled`/`GROK_WORKFLOWS` (off by default). The commit summary says five runtime bugs were fixed around budget, pause, cancel, and reconnect.

**Recommendation: PORT, but only after security and session durability.** Although custom bum workflows were deferred in v1.0, these are now upstream parity rather than a new bum-designed engine. Preserve upstream's off-by-default gate. Cross-provider adaptation is required: workflow-spawned agents must use the existing child model → provider → credential → backend route and fail closed before side effects. Acceptance must cover agent budget, same-process pause/resume, bounded denial/reconnect, cancel, interrupted-after-restart state, unique display handles, discovery precedence, untrusted symlink/path rejection, and both Grok→Codex and Codex→Grok children.

Complexity: **VERY HIGH**. Dependencies: security gates → durable session state → subagent routing → workflow engine → TUI/dashboard/docs.

### 6. Auth, model catalog, effort, and custom providers

This area has the highest overlap with bum v1 and must be merged semantically, not copied.

- **xAI workspaces scopes (U001):** ADAPT only to xAI login. Codex scopes remain independent.
- **Single-flight/retry auth (U050/U054/U099):** PORT into each provider's independent lifecycle; never serialize or wipe both slots together.
- **Grok 4.5 catalog (U057/U077):** PORT the xAI row/default fields but retain GPT-5.6 Sol/Terra/Luna and provider labels. Remote catalog replacement must still re-append bundled Codex models.
- **Distinct `max` (U148):** upstream supersedes bum's `max → xhigh` alias. Add a true enum tier while retaining per-model supported-level clamp and Codex wire fidelity.
- **Named custom model providers (U117/U152):** ADAPT as reusable gateway/auth-helper profiles. Do not replace bum's explicit xAI/Codex routing authority or allow a helper to shadow first-party OAuth silently.
- **App Builder flags/product selection (U146/U153):** CONDITIONAL ADAPT for xAI Build/App Builder traffic only. Never send xAI deployment semantics on Codex routes.

### 7. Privacy, telemetry, feedback, update, and deployment

| Upstream item | Bum disposition | Rationale / negative acceptance |
|---|---|---|
| SDK agents do not stage unusable self-updates (U009) | **SUPERSEDED / EXCLUDE transport** | Bum's stock updater is hard-off. Keep useful process-safety code only if it cannot contact/stage xAI updates. `bum update`, startup, SDK agents, and remote config make zero stock-update calls. |
| SessionMetrics skips Mixpanel profile sync (U043) | **SUPERSEDED** | Bum disables all product analytics, not merely one profile-sync mode. No Mixpanel traffic in any mode. |
| External OTEL consumer allowlist removal (U051) | **ADAPT / SUPERSEDED** | Internal exporter remains off. User-configured external OTEL may remain only if already part of bum's explicit local operator contract; no remote enable. |
| Coding-sharing opt-out/default/docs (U090/U097/U101) | **ADAPT** | xAI account privacy may be displayed/configured, but it cannot enable bum telemetry, trace upload, feedback, Sentry, or internal OTEL. Codex path must not receive xAI privacy settings. |
| Drop codebase-upload soak item (U110) | **EXCLUDE upload / keep test cleanup** | No automatic codebase/session upload. Remove stale soak assumptions without enabling storage traffic. |
| Feedback identity (U116/U154) | **EXCLUDE network behavior** | `/feedback` stays local/no-phone-home. Do not resolve/transmit author identity to xAI. |
| App Builder flags (U146/U153) | **ADAPT xAI-only** | Applicable only when an explicit xAI deployment path is used; no telemetry inference and no Codex leakage. |
| `x-grok-client-identifier` (U084) | **ADAPT xAI-only** | Treat as required API protocol metadata only on xAI direct tool calls; never attach to Codex or generic third-party gateways. |

## Already Present or Likely Superseded — Do Not Double-Count

These are hypotheses to prove with focused tests before marking ledger entries complete:

| Bum behavior | Upstream overlap | Classification |
|---|---|---|
| Product home, CLI, docs, and version are `bum` / `~/.bum`; stock update and telemetry hard-off. | U009/U043/U051/U097/U110/U116/U154 and all upstream `grok`/`~/.grok` documentation. | **Bum is stronger; adapt/exclude, never overwrite.** |
| Dual auth slots, independent refresh/status/logout, provider-specific OAuth recovery. | U001/U017/U050/U053/U054/U066/U099/U117. | **Partially superseded**, but import upstream races/security fixes per provider. |
| Explicit xAI/Codex `ModelProvider`, mixed picker, route resolver, provider-bound credentials, cross-provider children. | U077/U084/U117/U138/U148/U152. | **Bum architecture supersedes stock single-provider assumptions**, not upstream custom gateway profiles or distinct max tier. |
| `max` accepted as an alias and canonicalized to `xhigh`. | U148 distinct `Max`. | **Not parity. Must adapt/replace alias semantics where catalog advertises true max.** |
| Existing `/usage` billing surface and headless usage fields. | U133 session token/dollar display. | **Partial only.** Add current-session TUI accounting and provider-aware honesty. |
| Existing scheduler, `/loop`, tasks pane, subagents, usage docs. | U111/U136/U139/U143/U149/U168. | **Partial only.** Add lifecycle durability/versioning and whole-work-unit serialization. |
| Existing permissions include some `env -S` blocking tests. | U140 plus broader U134/U135/U142/U144/U163. | **Partial only.** Port complete managed deny/ask and adversarial fixture matrix. |
| Existing hook private-IP validation. | U002 redirect SSRF fix. | **Partial only.** Initial URL validation does not prove redirect safety. |
| Existing owner-only MCP/auth paths and signed policy work. | U026/U040/U046/U122. | **Likely partial/superseding.** Require overwrite, rollback, replay, and dual-slot tests before closing. |
| v1 embedded user guide is bum-branded and provider-aware. | Upstream changes 19 guide files after initial publish. | **Content merge required.** Do not replace with stock docs or assume branding alone equals parity. |

## Source-Summary Ledger Coverage

The following partition assigns **every one of the 168 commit-body summary bullets exactly once**. The implementation ledger should expand each ID to a row with `source_sha`, `source_path`, `disposition`, `bum_evidence`, `verification`, and `notes`.

| Ledger area | Source IDs | Default disposition |
|---|---|---|
| Auth, models, provider/API behavior | U001, U003, U004, U017, U022, U050, U053, U054, U066, U077, U084, U090, U097, U099, U102, U103, U117, U138, U148, U152 | ADAPT; some partial/superseded |
| Security, permissions, sandbox, policy | U002, U005, U023–U027, U029, U030, U034, U035, U037, U038, U040–U042, U046, U051, U060, U063, U064, U080, U081, U089, U095, U109, U114, U122, U134, U135, U140, U142, U144, U158, U163 | PORT/ADAPT; P0 |
| TUI, terminal, clipboard, docs, user UX | U006, U007, U010, U011, U013–U015, U021, U028, U031, U039, U044, U045, U052, U055, U059, U061, U062, U067, U068, U070–U074, U076, U086–U088, U091–U094, U101, U104–U106, U112, U113, U115, U119, U123, U130, U133, U141, U143, U145, U147, U155, U157, U159, U160, U162, U164–U166 | PORT, with bum naming/path adaptation |
| Session/runtime/tools/plugins/workflows | U008, U012, U016, U018–U020, U032, U033, U036, U047–U049, U056–U058, U065, U069, U075, U078, U079, U082, U083, U085, U096, U098, U100, U107, U108, U110, U111, U118, U120, U121, U124, U125, U131, U132, U136, U137, U139, U149–U151, U156, U161, U167, U168 | PORT/ADAPT; workflows very high risk |
| Worktree auto-GC | U126–U129 | PORT after lifecycle safety |
| Quiet-fork/product-specific conflict review | U009, U043, U116, U154 | SUPERSEDE or EXCLUDE with negative tests; U051/U097/U110 are classified in their owning rows and cross-referenced in the privacy section |
| xAI App Builder conditional adaptation | U146, U153 | ADAPT xAI-only |

> Ledger check performed during research: union = U001…U168; no missing or duplicate IDs.

## Feature Dependencies and Recommended Requirement Order

```text
Pinned SHA + machine-readable source ledger
    → security/permission parity
        → durable session/auth primitives
            → canonical TUI + terminal fixes
            → scheduler/background/worktree lifecycle
                → relocation/import
                → workflow runtime
                    → cross-provider workflow acceptance
    → docs/config/changelog reconciliation
        → final path-level no-silent-drop audit
```

1. **Baseline and ledger:** pin both public SHA and `SOURCE_REV`; import 168 summary rows plus changelog-only and path-only rows.
2. **Security P0:** shell/hook/web/MCP/plugin/sandbox/managed-policy hardening.
3. **Core runtime durability:** append/ack, auth races, subagent persistence, cancellation, rewind IDs.
4. **TUI/terminal UX:** canonical editor, status, clipboard, doctor, rendering/navigation.
5. **Tools and lifecycle:** scheduler, loop serialization, worktree GC, skills/plugin behavior.
6. **Relocation and ACP:** moved-directory and explicit mirrored import without automatic upload.
7. **Workflow parity:** off by default, then cross-provider adaptation and UAT.
8. **Catalog/provider reconciliation:** Grok 4.5 + mixed GPT catalog, true max tier, named custom gateway profiles.
9. **Docs and audit:** update all changed guides in bum language; prove every source ledger row has a disposition and test or explicit exclusion.

## Acceptance Model for Requirements

Every user-facing requirement should say what the user can observe; every operational requirement should state the failure boundary. Examples:

- **Security:** “A user can run auto mode without `rg --pre`, kubeconfig exec plugins, environment-dumping `ps`, packed `env -S`, or shell file redirects bypassing permission policy.”
- **Doctor:** “A user can run `bum doctor [--json]`, invoke `/doctor`, and apply only the explicit `terminal.ssh-wrap` fix with confirmation.”
- **Relocation:** “A user can move a repository and resume its bum session with history, cwd, and provider state intact.”
- **Usage:** “A user can view current-session tokens by model/provider; dollar amounts are shown only when authoritative pricing data exists.”
- **Workflows:** “With workflows explicitly enabled, a workflow can spawn children across providers using each child's own OAuth slot; missing credentials fail before worktree/session side effects.”
- **Quiet fork:** “No upstream update, feedback, analytics, Sentry, internal OTEL, or implicit code/session upload behavior can be re-enabled by settings, remote config, workflow, SDK mode, or provider switch.”
- **Ledger:** “A CI/audit artifact maps every U001–U168 item, every 0.2.102–0.2.109 release-note entry, and every changed upstream path cluster to PORT/ADAPT/ALREADY/EXCLUDE with evidence.”

## Explicit Exclusions

1. **Stock xAI auto-update/install adoption** — bum remains manually/private updated; only reusable safety fixes may port.
2. **xAI feedback submission and author identity transport** — `/feedback` remains local/no-phone-home.
3. **Mixpanel, internal OTEL, Sentry, trace/codebase upload, or remotely enabled product telemetry** — user-owned external diagnostics are a separate explicit opt-in if retained.
4. **Replacing isolated `~/.bum` with `~/.grok`, importing stock credentials, or restoring `GROK_HOME` as primary.**
5. **Replacing bum's dual OAuth/mixed routing with upstream single-xAI model assumptions.**
6. **Sending xAI-only headers, privacy controls, App Builder flags, or deployment product selection on Codex requests.**
7. **Blindly installing workflow-authoring skills that are named in U156 but absent from the pinned public tree** — investigate or document the public-source gap first.

These exclusions do not remove their source rows; each must remain in the parity ledger with rationale and a negative regression test.

## Gaps and Phase Research Flags

- **Workflow authoring skill files:** U156 names two skills, but the pinned public tree does not expose matching `SKILL.md` paths. Investigate at implementation time; do not fabricate content.
- **MiniSweAgent:bash:** U019 is named in the summary but not trivially discoverable by literal fingerprint at tip. Trace type/schema behavior rather than assuming deletion or completion.
- **App Builder behavior:** public code shows flags/product selection, but backend applicability to bum and Codex is not fully documented. Restrict to xAI until wire behavior is proven.
- **Dollar usage:** token accounting is local; authoritative dollar cost may depend on xAI deployment metadata and may be unavailable for ChatGPT subscription traffic. Prefer “unknown” to invented cost.
- **Cross-host mirrored sessions:** storage primitives exist, but bum's no-phone-home contract requires explicit operator configuration and no default remote destination.
- **Version/source identity:** add bum-owned upstream metadata rather than making `bum version` present as stock Grok. Suggested fields: bum version, upstream public SHA, upstream `SOURCE_REV`.

## Sources

Primary official evidence:

- [Pinned upstream commit `3af4d5d…`](https://github.com/xai-org/grok-build/commit/3af4d5d39897855bdcc74f23e690024a5dc05573) — complete tip summary and public SHA.
- [Official repository commits](https://github.com/xai-org/grok-build/commits/main) — six sync bodies and initial publish.
- [Pinned `SOURCE_REV`](https://github.com/xai-org/grok-build/blob/3af4d5d39897855bdcc74f23e690024a5dc05573/SOURCE_REV) — `0f4d7c91…`.
- [Pinned in-tree changelog](https://github.com/xai-org/grok-build/blob/3af4d5d39897855bdcc74f23e690024a5dc05573/crates/codegen/xai-grok-shell/CHANGELOG.md) — releases 0.2.101–0.2.109.
- [Pinned user guide](https://github.com/xai-org/grok-build/tree/3af4d5d39897855bdcc74f23e690024a5dc05573/crates/codegen/xai-grok-pager/docs/user-guide) — slash commands, configuration, workflows, terminal, permissions, dashboard, usage.
- Local Git object evidence: `upstream/main`, commit bodies, cumulative diffs, `git merge-base`, root-tree comparison, and content fingerprints at bum `128992c…`.
- Bum v1 contracts: `.planning/PROJECT.md`, `.planning/MILESTONES.md`, `.planning/milestones/v1.0-REQUIREMENTS.md`.

**Evidence note:** web search was used through the required research-plan seam, but recommendations above are grounded in the checked-out official Git objects and GitHub API results rather than third-party reporting. The classifier returns MEDIUM for verified websearch evidence; negative or uncertain claims are explicitly flagged as gaps.
