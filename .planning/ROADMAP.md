# Roadmap: bum

## Overview

Ship **bum** as a full-product fork of Grok Build: isolated `~/.bum` identity, dual OAuth (xAI + ChatGPT/Codex), mixed Grok/GPT-5.6 model picker with per-model routing, cross-provider subagent orchestration, quiet (no phone-home) rebrand, and a daily-driver bar for real coding work. Work proceeds identity → multi-slot auth → catalog → routing → Codex OAuth → session gate/UX → multi-agent → quiet rebrand → e2e daily-driver validation.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3…): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Product identity & isolated home** - Ship `bum` binary and `~/.bum` home cutover
- [ ] **Phase 2: Multi-slot credentials & xAI OAuth** - Provider-scoped auth store with working xAI login under bum
- [ ] **Phase 3: Model catalog & GPT-5.6 entries** - Provider-tagged catalog with Grok + GPT-5.6 family in one list
- [ ] **Phase 4: Provider-aware request routing** - Model → backend + credentials (Grok vs Codex paths)
- [ ] **Phase 5: Codex OAuth & dual auth lifecycle** - ChatGPT login/logout/status + independent refresh
- [ ] **Phase 6: Mid-session switch & missing-provider gate** - Switch anytime; fail closed with login prompt
- [ ] **Phase 7: Cross-provider multi-agent orchestration** - Same-provider regression + parent/child cross-provider spawn
- [ ] **Phase 8: Quiet fork & rebrand polish** - No auto-update/telemetry; full bum chrome/strings
- [ ] **Phase 9: Daily-driver end-to-end validation** - Real sessions both providers, mid-session switch, cross-provider agents

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

**Plans**: 4 plans

Plans:

- [ ] 02-01-PLAN.md — Multi-slot AuthDocument schema + read migrate + merge-safe nested write
- [ ] 02-02-PLAN.md — API key isolation + multi-provider empty-delete / scope-removal prune
- [ ] 02-03-PLAN.md — AuthManager + credential Bearer path proof against multi-slot store
- [ ] 02-04-PLAN.md — bum login CLI/path seal + full auth lib suite gate

### Phase 3: Model catalog & GPT-5.6 entries

**Goal**: Model selector presents a mixed, provider-labeled catalog including GPT-5.6 family options
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: MOD-01, MOD-02
**Success Criteria** (what must be TRUE):

  1. Model selector lists current GPT-5.6 family options (e.g. Sol/Terra/Luna or live Codex IDs), labeled as Codex/OpenAI provider
  2. Model selector lists Grok/xAI models alongside GPT models in one mixed list (not a global provider mode that filters the session)
  3. Every catalog entry carries an explicit provider binding usable by later routing

**Plans**: TBD
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

**Plans**: TBD

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

**Plans**: TBD
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

**Plans**: TBD
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

**Plans**: TBD

### Phase 8: Quiet fork & rebrand polish

**Goal**: Product presents fully as bum and does not phone home or auto-update as stock Grok Build
**Mode:** mvp
**Depends on**: Phase 1 (identity paths); can parallelize polish after Phase 6
**Requirements**: ID-02, OPS-01, OPS-02
**Success Criteria** (what must be TRUE):

  1. Product UI chrome, help text, and user-facing strings present as **bum**, not stock Grok Build
  2. Stock xAI auto-update channel is disabled so bum is not overwritten by official Grok Build updates
  3. Product telemetry / phone-home to xAI analytics is disabled by default for the fork

**Plans**: TBD
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

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9  
(Phase 8 quiet/rebrand polish may start in parallel once Phase 1 identity is stable, but ships before Phase 9 validation.)

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Product identity & isolated home | 5/5 | In Progress|  |
| 2. Multi-slot credentials & xAI OAuth | 0/4 | Not started | - |
| 3. Model catalog & GPT-5.6 entries | 0/TBD | Not started | - |
| 4. Provider-aware request routing | 0/TBD | Not started | - |
| 5. Codex OAuth & dual auth lifecycle | 0/TBD | Not started | - |
| 6. Mid-session switch & missing-provider gate | 0/TBD | Not started | - |
| 7. Cross-provider multi-agent orchestration | 0/TBD | Not started | - |
| 8. Quiet fork & rebrand polish | 0/TBD | Not started | - |
| 9. Daily-driver end-to-end validation | 0/TBD | Not started | - |

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
| OPS-04 | Phase 9 |
| OPS-05 | Phase 9 |
| OPS-06 | Phase 9 |

**Coverage:** 26/26 v1 requirements mapped ✓

---
*Roadmap created: 2026-07-16*
*Granularity: fine · Mode: mvp · Phase IDs: sequential*
