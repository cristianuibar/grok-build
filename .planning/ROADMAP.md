# Roadmap: bum

## Overview

Ship **bum** as a full-product fork of Grok Build: isolated `~/.bum` identity, dual OAuth (xAI + ChatGPT/Codex), mixed Grok/GPT-5.6 model picker with per-model routing, cross-provider subagent orchestration, quiet (no phone-home) rebrand, and a daily-driver bar for real coding work. Work proceeds identity → multi-slot auth → catalog → routing → Codex OAuth → session gate/UX → multi-agent → quiet rebrand → e2e daily-driver validation.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3…): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Product identity & isolated home** - Ship `bum` binary and `~/.bum` home cutover
- [x] **Phase 2: Multi-slot credentials & xAI OAuth** - Provider-scoped auth store with working xAI login under bum
- [x] **Phase 3: Model catalog & GPT-5.6 entries** - Provider-tagged catalog with Grok + GPT-5.6 family in one list (completed 2026-07-16)
- [x] **Phase 4: Provider-aware request routing** - Model → backend + credentials (Grok vs Codex paths) (completed 2026-07-16)
- [x] **Phase 5: Codex OAuth & dual auth lifecycle** - ChatGPT login/logout/status + independent refresh
- [x] **Phase 6: Mid-session switch & missing-provider gate** - Switch anytime; fail closed with login prompt (completed 2026-07-17)
- [x] **Phase 7: Cross-provider multi-agent orchestration** - Same-provider regression + parent/child cross-provider spawn (completed 2026-07-17)
- [x] **Phase 8: Quiet fork & rebrand polish** - No auto-update/telemetry; full bum chrome/strings (completed 2026-07-17)
- [ ] **Phase 9: Daily-driver end-to-end validation** - Real sessions both providers, mid-session switch, cross-provider agents
- [ ] **Phase 10: Codex Responses wire parity** - Align bum request shape with official Codex CLI (headers, reasoning defaults, continuity)
- [ ] **Phase 11: Codex effort & catalog fidelity** - GPT-5.6 effort levels, soft-clamp UX, catalog capability flags from Codex models.json patterns
- [ ] **Phase 12: Codex depth & attribution polish** - Optional tool/WS/originator depth; honest product attribution (bum ≠ Codex CLI)

## Phase Details

### Phase 1: Product identity & isolated home

**Goal**: User launches `bum` and all product state lives under isolated `~/.bum` (or `BUM_HOME`)
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: ID-01, ID-03
**Success Criteria** (what must be TRUE):

  1. User can invoke the primary CLI as `bum` (not `grok` / `xai-grok-pager` as the shipped command name)
  2. Fresh run creates/uses config, auth, and session paths under `~/.bum` (or `BUM_HOME`), not `~/.grok` or stock Codex paths
  3. Running with a temporary home shows no writes under `~/.grok` / `~/.codex` for product state

**Plans**: 5/5 plans executed

Plans:

- [x] 01-01-PLAN.md — SoT home cutover (`~/.bum` / `BUM_HOME`) + pure resolver + process-isolated env tests + path display labels
- [x] 01-02-PLAN.md — Twin resolver, leader managed bin, updater `bin/bum`, workspace fallback lockstep
- [x] 01-03-PLAN.md — Ship `[[bin]]` as `bum` + harness binary resolution
- [x] 01-04-PLAN.md — Full test sandbox cutover (PTY flows/leader/scripted, shell inventory, lock/log)
- [x] 01-05-PLAN.md — Bundle + roles/personas + stock-home gate + hermetic isolation proof + shell-inclusive grep

### Phase 2: Multi-slot credentials & xAI OAuth

**Goal**: Auth storage is provider-scoped so dual OAuth is safe; xAI login still works end-to-end under bum
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: AUTH-01
**Success Criteria** (what must be TRUE):

  1. User can complete xAI OAuth (browser and/or device-code) with credentials stored only under the bum auth store
  2. Auth store is structured for multiple provider slots (xAI first; second slot reserved without overwriting xAI)
  3. After xAI login, user can start an agent turn that authenticates successfully against the xAI path

**Plans**: 4/4 plans executed

Plans:

- [x] 02-01-PLAN.md — Multi-slot AuthDocument + lock-scoped mutate + concurrency + version fail-closed
- [x] 02-02-PLAN.md — Locked API keys + multi-provider prune + try_devbox_recovery
- [x] 02-03-PLAN.md — AuthManager + ShellAuthCredentialProvider + sampling_config.api_key Bearer seam
- [x] 02-04-PLAN.md — GROK_AUTH_PATH isolation + mock multi-slot login asserts + phase gate

### Phase 3: Model catalog & GPT-5.6 entries

**Goal**: Model selector presents a mixed, provider-labeled catalog including GPT-5.6 family options
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: MOD-01, MOD-02
**Success Criteria** (what must be TRUE):

  1. Model selector lists current GPT-5.6 family options (e.g. Sol/Terra/Luna or live Codex IDs), labeled as Codex/OpenAI provider
  2. Model selector lists Grok/xAI models alongside GPT models in one mixed list (not a global provider mode that filters the session)
  3. Every catalog entry carries an explicit provider binding usable by later routing

**Plans:** 3/3 plans complete

Plans:

- [x] 03-01-PLAN.md — Wave 0 compiling harness + ModelProvider schema/override chain + embedded mixed catalog
- [x] 03-02-PLAN.md — Prefetch Codex union-append + collision authority + dual-auth visibility
- [x] 03-03-PLAN.md — ACP meta.provider + CLI format_cli_model_row + required /model UAT

**UI hint**: yes

### Phase 4: Provider-aware request routing

**Goal**: Selecting a model routes each turn to the correct backend with that provider’s credentials
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: MOD-04, MOD-05
**Success Criteria** (what must be TRUE):

  1. Requests for Grok/xAI models use the xAI / cli-chat-proxy path with xAI credentials (not Codex tokens or Platform API semantics)
  2. Requests for GPT/Codex models use the OpenAI/Codex (ChatGPT backend) path with ChatGPT OAuth credentials — not the xAI proxy
  3. Switching the active model changes resolved base URL / credential slot for the next sample (verifiable in logs or tests with fake tokens)

**Plans:** 5/5 plans complete

Plans:

- [x] 04-01-PLAN.md — Wave 0 harness + RED dual-route; dual-token never_cross_slot scaffold; switch contract reserved for production transforms A/B
- [x] 04-02-PLAN.md — Codex endpoints + resolve_provider_route production authority + first-party session_oauth_allowed
- [x] 04-03-PLAN.md — default_models via resolver + rebind + dual-key credentials with EndpointsConfig trust provenance
- [x] 04-04-PLAN.md — PreparedSamplingConfig/auth_type carrier + Option ModelAuthFacts.provider + production transforms A/B + reconstruct
- [x] 04-05-PLAN.md — Local fail-closed (incl. empty live resolver) + mock Authorization + secret log fix + phase gate

### Phase 5: Codex OAuth & dual auth lifecycle

**Goal**: User can log into both xAI and ChatGPT/Codex, manage them independently, and keep both sessions healthy
**Mode:** mvp
**Depends on**: Phase 2 (store), Phase 4 (routing ready for live GPT turns)
**Requirements**: AUTH-02, AUTH-03, AUTH-04, AUTH-05
**Success Criteria** (what must be TRUE):

  1. User can log in to ChatGPT/Codex via ChatGPT OAuth (PKCE browser flow and device-code where applicable); credentials only under bum auth store
  2. User can log out of one provider without clearing the other provider’s credentials
  3. User can inspect per-provider auth status (which providers are logged in / usable)
  4. Access tokens for each provider refresh independently without wiping or invalidating the other provider’s session

**Plans:** 6/6 plans complete

Plans:

- [x] 05-01-PLAN.md — Wave 0 RED harness (Option C reconstruct + BYOK/custom + identity names + clap)
- [x] 05-02-PLAN.md — Public AuthProvider RMW + clear_with_lock + usable status semantics
- [x] 05-03-PLAN.md — Codex OAuth (PKCE + device multi-step) + login branch before xAI sync
- [x] 05-04-PLAN.md — Blocking dual logout + status Write + TUI/ACP dual-safe
- [x] 05-05-PLAN.md — Option C reconstruct + SessionToken/OAuth gates + lock-held clear + identity preserve
- [x] 05-06-PLAN.md — Phase gate (Option C seam + concurrent + BYOK/custom + regressions + fmt)

**UI hint**: yes

### Phase 6: Mid-session switch & missing-provider gate

**Goal**: User freely switches models mid-session; missing credentials fail closed with the correct login prompt
**Mode:** mvp
**Depends on**: Phase 4, Phase 5
**Requirements**: MOD-03, MOD-06
**Success Criteria** (what must be TRUE):

  1. User can switch models mid-session anytime; the next turn uses the newly selected model without restarting the CLI
  2. Selecting a model whose provider has no usable credentials blocks the switch and prompts that provider’s login (no silent mid-turn 401 as primary UX)
  3. With both providers logged in, user can move between Grok and GPT-5.6 models in one continuous session

**Plans**: 6/6 plans executed

Plans:

- [x] 06-01-PLAN.md — Shell typed MODEL_SWITCH_MISSING_PROVIDER + apply gate + session side-effect harness
- [x] 06-02-PLAN.md — QuestionView + transactional default switch (no optimistic current/persist); wave-2 Effect owner
- [x] 06-03-PLAN.md — Provider-aware deferred (persist_default) + AuthComplete + external CLI poll/FocusGained + generation cancel
- [x] 06-04-PLAN.md — AuthMeta dual-slot + lifecycle refresh triggers (H3) + slash/settings needs-login badge + BYOK
- [x] 06-05-PLAN.md — Dual-login free switch + named history/mid-turn session observables
- [x] 06-06-PLAN.md — Validation map + phase gate with per-subgroup p6_ discovery asserts

**UI hint**: yes

### Phase 7: Cross-provider multi-agent orchestration

**Goal**: Parent on one provider can spawn a child on another with correct model, effort, credentials, and backend routing
**Mode:** mvp
**Depends on**: Phase 6
**Requirements**: AGENT-01, AGENT-02, AGENT-03, AGENT-04, AGENT-05, AGENT-06
**Success Criteria** (what must be TRUE):

  1. Existing same-provider subagent spawn/resume/roles/personas still work after rebrand (no regression when parent and child share a provider)
  2. User or parent agent can spawn a subagent with an explicit model on a different provider than the parent (e.g. Grok parent → `gpt-5.6-sol` child)
  3. Subagent launch accepts reasoning effort (or equivalent Codex effort); child runs with that effort
  4. Cross-provider child turns use the child’s model → provider → credentials → backend (not the parent’s bearer or base URL)
  5. Natural-language orchestration works: e.g. main on Grok, “start a Codex Sol medium-effort subagent to research X” yields a Sol medium-effort child that returns results; missing child-provider login fails closed with a clear login prompt

**Plans**: 6/6 plans executed

Plans:

- [x] 07-01-PLAN.md — Wave 0 RED p7_ harness (tools effort contracts + shell cross_provider_subagent)
- [x] 07-02-PLAN.md — Task schema + reasoning_effort wire + NL description (AGENT-03/06)
- [x] 07-03-PLAN.md — Authoritative spawn gate + Tool effort/unknown-model fail-closed (AGENT-02/03/04/05)
- [x] 07-04-PLAN.md — Eager TaskProviderCredentialGate preflight for background default (AGENT-05)
- [x] 07-05-PLAN.md — Dual fake-token isolation + no wrong-backend proofs (AGENT-02/04/05/06)
- [x] 07-06-PLAN.md — AGENT-01 regression + VALIDATION/PHASE-GATE for AGENT-01..06

### Phase 8: Quiet fork & rebrand polish

**Goal**: Product presents fully as bum and does not phone home or auto-update as stock Grok Build
**Mode:** mvp
**Depends on**: Phase 1 (identity paths); can parallelize polish after Phase 6
**Requirements**: ID-02, OPS-01, OPS-02
**Success Criteria** (what must be TRUE):

  1. Product UI chrome, help text, and user-facing strings present as **bum**, not stock Grok Build
  2. Stock xAI auto-update channel is disabled so bum is not overwritten by official Grok Build updates
  3. Product telemetry / phone-home to xAI analytics is disabled by default for the fork

**Plans**: 6/6 plans executed

Plans:

- [x] 08-01-PLAN.md — Wave 1 green p8_ harness + VALIDATION inventory (telemetry default + model/isolation baseline)
- [x] 08-02-PLAN.md — ID-02 pager TUI chrome (clap, hero, picker, billing)
- [x] 08-03-PLAN.md — ID-02 OAuth return + pager-minimal + bin banner
- [x] 08-04-PLAN.md — OPS-01 hard-off auto-update (gates, defaults, CLI no-op, min-version, settings)
- [x] 08-05-PLAN.md — OPS-02 feedback short-circuit + telemetry/Sentry quiet defaults
- [x] 08-06-PLAN.md — Phase gate: VALIDATION + PHASE-GATE green discovery for ID-02/OPS-01/OPS-02

**UI hint**: yes

### Phase 9: Daily-driver end-to-end validation

**Goal**: bum is usable as the default coding agent for real work across both providers and cross-provider agents
**Mode:** mvp
**Depends on**: Phase 6, Phase 7, Phase 8
**Requirements**: OPS-03, OPS-04, OPS-05, OPS-06
**Success Criteria** (what must be TRUE):

  1. User can complete a real coding session on an xAI model after xAI login (tools, edit, shell as in stock daily use)
  2. User can complete a real coding session on a GPT-5.6 model after Codex login (tools, edit, shell as supported on that path)
  3. User can switch between Grok and GPT-5.6 in one session and continue productive work without restarting the CLI
  4. Parent-on-Grok + child-on-Codex and parent-on-Codex + child-on-Grok subagent research/coding tasks complete successfully when both providers are logged in

**Plans**: 3/5 plans executed

Plans:

- [x] 09-01-PLAN.md — VALIDATION dual map + thin green p9_ fixture residual smoke
- [x] 09-02-PLAN.md — Automated PHASE-GATE (prior p6/p7/p8 subset + p9_) discover+execute
- [x] 09-03-PLAN.md — Required 09-UAT.md dual-login checklist + non-secret preflight script
- [ ] 09-04-PLAN.md — Live dual-login UAT execute (human) + in-phase blocker fixes only
- [ ] 09-05-PLAN.md — Hybrid gate close: re-run auto + 09-VERIFICATION + nyquist sign-off

### Phase 10: Codex Responses wire parity

**Goal**: bum’s ChatGPT Codex HTTP path matches official Codex CLI wire contracts beyond the system→instructions fix, so productive GPT turns stay green under real dual login  
**Mode:** mvp  
**Depends on**: Phase 9 (blocker fix + live UAT re-verify)  
**Requirements**: OPS-04, OPS-05 (deepen); research: `.planning/research/CODEX-RESPONSES-WIRE.md`  
**Success Criteria** (what must be TRUE):

  1. Codex Responses requests set `store`/`stream`/`include`/`tool_choice` consistent with ChatGPT path (not Azure-store assumptions)
  2. Optional Codex-compatible headers (`session-id` / `thread-id` / originator as **bum**, not `codex_cli_rs`) documented and applied where they improve reliability
  3. Reasoning summary defaults omit or match GPT-5.6 catalog (not always force `concise` if server rejects)
  4. Automated tests assert no `role: system` in Responses `input` and non-empty `instructions` when system history present
  5. Live OPS-04 re-run stays PASS after wire tweaks

**Plans**: TBD (plan after Phase 9 hybrid close or in parallel once 09-04 system fix re-UAT green)

### Phase 11: Codex effort & catalog fidelity

**Goal**: GPT-5.6 effort levels work like official Codex (soft-clamp, supported menus) — no false “effort not supported” on catalog models  
**Mode:** mvp  
**Depends on**: Phase 10 (or Phase 9 if effort is the only remaining UX gap)  
**Requirements**: MOD-01/MOD-02 deepen; OPS-04  
**Success Criteria** (what must be TRUE):

  1. GPT-5.6 catalog entries advertise non-empty supported effort levels (or equivalent meta) aligned with Codex `models.json` patterns
  2. Mid-session switch Grok→Codex soft-clamps effort to a supported level; does not hard-fail the turn
  3. Wire sends `reasoning.effort` for supported levels; ultra maps to max if exposed
  4. UAT notes no longer list effort as a product blocker when levels are supported

**Plans**: TBD

### Phase 12: Codex depth & attribution polish

**Goal**: Optional deeper Codex parity (WS incremental, tool naming notes) and clear product attribution that bum is not stock Codex CLI  
**Mode:** standard  
**Depends on**: Phase 10–11  
**Requirements**: product honesty / D-10 gaps; optional AGENT/OPS polish  
**Success Criteria** (what must be TRUE):

  1. Documented decision: which Codex-only features are in-scope vs deferred (WS, freeform apply_patch, originator)
  2. Product chrome never claims “Codex CLI” identity; model brands remain GPT-5.6 / provider labels
  3. THIRD_PARTY / notices updated if any Codex-derived constants or substantial ports land
  4. Capability gaps vs stock Codex remain honest in docs (D-10)

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12  
(Phase 8 quiet/rebrand polish may start in parallel once Phase 1 identity is stable, but ships before Phase 9 validation.  
Phases 10–12 deepen Codex after Phase 9 daily-driver bar; Phase 9 in-phase fix for system→instructions is required for OPS-04 green.)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Product identity & isolated home | 5/5 | Complete    | 2026-07-16 |
| 2. Multi-slot credentials & xAI OAuth | 4/4 | Complete    | 2026-07-16 |
| 3. Model catalog & GPT-5.6 entries | 3/3 | Complete    | 2026-07-16 |
| 4. Provider-aware request routing | 5/5 | Complete    | 2026-07-16 |
| 5. Codex OAuth & dual auth lifecycle | 6/6 | Complete    | 2026-07-16 |
| 6. Mid-session switch & missing-provider gate | 6/6 | Complete    | 2026-07-17 |
| 7. Cross-provider multi-agent orchestration | 6/6 | Complete    | 2026-07-17 |
| 8. Quiet fork & rebrand polish | 6/6 | Complete    | 2026-07-17 |
| 9. Daily-driver end-to-end validation | 3/5 | In Progress|  |
| 10. Codex Responses wire parity | 0/TBD | Planned |  |
| 11. Codex effort & catalog fidelity | 0/TBD | Planned |  |
| 12. Codex depth & attribution polish | 0/TBD | Planned |  |

## Coverage map

| Requirement | Phase |
|-------------|-------|
| ID-01 | Phase 1 |
| ID-02 | Phase 8 |
| ID-03 | Phase 1 |
| AUTH-01 | Phase 2 |
| AUTH-02 | Phase 5 |
| AUTH-03 | Phase 5 |
| AUTH-04 | Phase 5 |
| AUTH-05 | Phase 5 |
| MOD-01 | Phase 3 |
| MOD-02 | Phase 3 |
| MOD-03 | Phase 6 |
| MOD-04 | Phase 4 |
| MOD-05 | Phase 4 |
| MOD-06 | Phase 6 |
| AGENT-01 | Phase 7 |
| AGENT-02 | Phase 7 |
| AGENT-03 | Phase 7 |
| AGENT-04 | Phase 7 |
| AGENT-05 | Phase 7 |
| AGENT-06 | Phase 7 |
| OPS-01 | Phase 8 |
| OPS-02 | Phase 8 |
| OPS-03 | Phase 9 |
| OPS-04 | Phase 9 (+ Phase 10–11 deepen) |
| OPS-05 | Phase 9 (+ Phase 10 deepen) |
| OPS-06 | Phase 9 |

**Coverage:** 26/26 v1 requirements mapped ✓ (Phases 10–12 deepen OPS/MOD without new requirement IDs)

---
*Roadmap created: 2026-07-16*  
*Updated: 2026-07-18 — Phase 10–12 Codex wire/effort/attribution after official codex-rs research*  
*Granularity: fine · Mode: mvp · Phase IDs: sequential*
