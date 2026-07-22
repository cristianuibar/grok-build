# Milestones

## v1.0 Multi-provider daily driver (Shipped: 2026-07-22)

**Phases completed:** 13 phases, 65 plans, 137 tasks

**Key accomplishments:**

- Default product home cut over to `~/.bum` with pure `resolve_product_home`, `BUM_HOME`-only override, `bin/bum` managed leaf, and process-isolated path tests
- Worktree twin, leader managed bin, and auto-update activation all target `BUM_HOME` / `~/.bum` / `bin/bum` in lockstep with config SoT — no managed `bin/grok` alias under product home.
- Composition-root `[[bin]]` renamed to `bum` with test-support and PTY harness resolution on `CARGO_BIN_EXE_bum` / `target/debug/bum` (crate names unchanged)
- All inventoried product-home test sandboxes now set `BUM_HOME` / write under `.bum`, matching production isolation so suites no longer re-validate `GROK_HOME` coupling
- Legacy stock-home agent/role/bundle reads gated; hermetic recursive isolation proven; production GROK_HOME bypasses and operational product-home labels cut over to BUM_HOME / ~/.bum
- Locked API-key, logout, and devbox recovery mutations now preserve sibling providers and delete auth.json only when every slot is empty.
- Nested xAI credentials now have automated evidence through AuthManager, ShellAuthCredentialProvider, and the sampler’s real Bearer request path.
- Mixed provider-labeled catalog with `ModelProvider` (xai|codex) on the full type chain and GPT-5.6 Sol/Terra/Luna rows, proven via `tests/model_catalog` integration binary
- Remote prefetch no longer erases GPT-5.6 Sol/Terra/Luna — remove-then-append restores bundled Codex rows after replace with authoritative provider/name and stable end-of-list order
- Skipped — executor environment has no interactive TUI session launch requirement met; plan marks this checkpoint `gate="optional"` advisory only.
- Integration binary encodes MOD-04/MOD-05 dual-route + dual-token + switch contracts with smoke green and intentional behavior-RED until Plans 02–05.
- Public `resolve_provider_route` is the production authority for base_url + credential_slot + session_oauth_allowed, with ChatGPT Codex default endpoint and first-party OAuth host policy.
- Bundled catalog stamps provider-correct base URLs via `resolve_provider_route`, and dual-key `resolve_credentials_for_provider` isolates xAI/Codex session tokens with EndpointsConfig OAuth host provenance.
- Next-sample routing is provider-correct: prepare dual-keys into a PreparedSamplingConfig carrier, model_switch applies transform A without xAI AuthManager re-resolve, and reconstruct attaches live xAI bearer only when ModelAuthFacts.provider is Some(Xai).
- SamplingClient local fail-closed (incl. empty live resolver) with redacted invalid-key logs and mock on-wire Authorization proofs for xAI/Codex fake tokens — full Phase 4 automated gate green.
- Wave 0 RED harness `auth_codex_lifecycle` plus clap parse locks for dual-provider login/logout/status — no production Codex OAuth yet
- Public AuthProvider-parameterized RMW/clear plus pure dual-provider status with correct logged_in/usable semantics (AUTH-03/04 core GREEN)
- In-tree ChatGPT/Codex PKCE + deviceauth OAuth persists only `providers.codex`; `bum login --provider codex` returns before xAI managed-config sync.
- Blocking selective/`--all` logout and paste-safe dual `bum auth status`, with TUI/ACP fail-closed against silent dual wipe (AUTH-03, AUTH-04)
- Per-request Codex ensure_fresh on reconstruct_full_config with lock-held RT rotation, pure CodexRefresher, BYOK/custom gates, and typed CodexAuthMaterial (AUTH-05)
- Full dual OAuth lifecycle + Option C reconstruct seam + multi-slot/routing/clap regressions green; concurrent ensure_fresh IdP single-flight stabilized under full suite
- Shell fail-closes mid-session model switch with typed `MODEL_SWITCH_MISSING_PROVIDER` before prepare/SetSessionModel, reusing Phase 5 usable-token semantics (refreshable OK; BYOK skip).
- Pager maps `MODEL_SWITCH_MISSING_PROVIDER` to a QuestionView (Login now / Keep current) with transactional settings switch — rejected targets never flash as current or persist as default.
- Login now reuses the Plan 02 gate-open DeferredModelSwitch (including persist_default), arms provider-scoped recovery (xAI OAuth vs Codex CLI-primary), and auto-retries SwitchModel when AuthComplete or external status refresh marks the required provider usable — with bounded poll generation cancel for stale refreshes.
- Authoritative dual-slot usable producer (AuthMeta + disk refresh) feeds an AppView cache refreshed on startup/login/logout/focus; unusable non-BYOK models show exact ` · needs login` on slash `/model` and settings DynamicEnum without filtering the mixed catalog.
- Session-level fixture tests prove MOD-03 free Grok↔GPT movement with dual usable tokens, D-06 history preserve + mid-turn non-cancel, BYOK skip, and next-sample target-provider routing — not pure routing substitutes.
- Phase 6 closed with a greened per-subgroup cargo gate (shell `model_switch_gate` + pager `--lib` p6_ filters) and a complete VALIDATION map for MOD-03/MOD-06 — no product scope expansion.
- Compile-safe Nyquist scaffold: tools + shell `p7_` green filters with dual-token fixtures and Task effort-None gap lock (no product schema/gate yet).
- Task tool exposes optional reasoning_effort, wires canonical tokens into SubagentRuntimeOverrides (max→xhigh), rejects invalid effort fail-closed, and documents NL model+effort guidance for parent models.
- Authoritative shell spawn path: missing-provider credential gate runs after effective child model preflight and before insert_pending/worktree; pure oauth_provider_slot_usable shared with model_switch; Tool effort fail-closed with supported/unsupported fixtures.
- Background Task path awaits async `SubagentBackend::preflight_spawn` (live shell effective-model resolve + credential gate) and fails closed with `bum login --provider` before returning the started notice — not a sync slug-only Fn.
- In-crate minimal harness drives production child resolve → one mock HTTP sample and proves Grok↔Codex Authorization isolation both ways, parent model stability after real spawn, and missing-slot fail-closed with zero outbound traffic.
- Green-only phase gate closes Phase 7: VALIDATION greened AGENT-01..06 filter map, per-subgroup discover+execute all pass under fixture tokens, same-provider regression + dual Authorization + async preflight proven.
- Compile-safe Nyquist scaffold: shell `p8_telemetry` (OPS-02 Disabled default) + pager `p8_wave1` discovery smoke; VALIDATION inventory for ID-02/OPS-01/OPS-02 without product rebrand or update hard-off.
- Pager clap, welcome/hero, project picker, billing product strings, and headless/plugin_cmd residual CLI present as bum; SuperGrok commercial SKU and model brands preserved; green p8_cli_brand / p8_welcome / p8_runtime_cli proofs.
- OAuth return, shell recovery CLI, minimal welcome, and pager-bin banner/crash/server identity present as bum with green p8_ proofs (C1-H1 / ID-02).
- OPS-01 hard-off: composition-root gate always false, bum update / Ctrl+U finish no-op with locked UI-SPEC message and hermetic zero stock-helper calls, auto_update None→false without first-run true persist, min-version entry no-op, settings default off.
- Default path never phones home: /feedback short-circuits, remote cannot re-enable telemetry/feedback, internal OTLP exporter off, Sentry hard-off.
- Phase 8 closed green: VALIDATION maps all ID-02/OPS-01/OPS-02 filters; PHASE-GATE discover+execute (incl. unconditional p8_sentry/p8_internal_otel + residual greps + model/isolation) all pass with no intentional-red.
- Single differentiated green `p9_` composition residual plus finalized dual auto/human VALIDATION inventory for OPS-03..06
- Hybrid 09-PHASE-GATE composed and automated half greened for full P0/P1/p9_ residual inventory without live OAuth
- Required OPS-03..06 dual-login checklist plus non-secret preflight with fail-closed scoped credential path and phase-diff secret gates
- OPS-03 through OPS-06 passed on live xAI and Codex backends, including cross-provider child tasks in both directions after four scoped Codex wire/stream fixes.
- The automated residual half and signed live UAT half are both GREEN; Phase 9 is Nyquist-compliant and consumable by strict GSD completion tooling.
- A disabled typed Codex Responses profile now reaches the sampler and produces trust-gated, tool-aware wire JSON without altering generic conversion behavior.
- Visible Responses SSE text now persists through an output-less completed terminal event, preventing post-success Empty retries while preserving retries for actually empty responses.
- Provider transitions now strip only foreign encrypted reasoning payloads, preserving useful session history and same-provider Codex continuity.
- Trusted Codex OAuth now owns bum's Responses profile and actor-lifetime UUID request identity, while encrypted-history mismatch 400s fail once without compaction or resubmission.
- Focused Phase 10 automated suite is green (26/26 behavior commands); live OPS-04 and OPS-05 PASS on rebuilt bum (operator 2026-07-18).
- Every planned shell sampler configuration path now keeps the typed Responses wire profile explicitly disabled, preserving routing and header behavior until the trusted OAuth boundary activates it.
- All planned shell test and fixture sampler configurations now explicitly preserve the disabled Responses wire profile, keeping trusted Codex behavior opt-in.
- Full embedded-doc identity RED contracts, canonical Codex patch-tool protection, and a credential-free exact-discovery phase gate
- An embedded xAI/Grok and ChatGPT/Codex capability matrix with honest bum identity, supported HTTP/SSE boundaries, and direct entry-point discovery
- bum-first onboarding and TUI documentation with truthful source installation, isolated `~/.bum` state, and explicitly preserved compatibility identifiers
- bum-owned MCP, skills, plugins, hooks, custom-model, and project-rule guidance with explicit compatibility boundaries
- bum-owned memory, automation, ACP, subagent, and session guidance with isolated state and honest compatibility attribution
- bum-owned safety, terminal, permission, and compiled reference guidance with isolated global paths and bounded patch-tool containment claims
- Exact local-fixture Rust coverage and a fail-closed 18-row Nyquist audit now prove bum attribution, provider isolation, notice integrity, and deferred-scope honesty without live credentials or provider traffic.
- Literal review/Plan 08 allowances, adversarial boundary fixtures, and SHA-identified credential-free evidence restore reproducible Phase 12 closure without changing product behavior.
- `bum version` now prints `bum {version}` via `VERSION_CMD_PRODUCT_NAME` + pure `format_version_cmd_line`, with unit gate and live binary smoke green; JSON unchanged.
- Hermetic `bum version` spawn asserts product token `bum ` (not stock `grok `) with version body present; scoped static residual greps clean; VALIDATION Nyquist map green without dual-login UAT.

---
