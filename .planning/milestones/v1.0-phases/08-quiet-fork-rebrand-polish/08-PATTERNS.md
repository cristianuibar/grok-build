# Phase 8: Quiet fork & rebrand polish - Pattern Map

**Mapped:** 2026-07-17
**Files analyzed:** 22
**Analogs found:** 22 / 22

> Brownfield **edit-in-place** phase. Closest analog for nearly every target is the **same file** (surgical string/default flip). Cross-file analogs supply test harness style (Phase 1 isolation), short-circuit UX (dispatch system blocks), and config-default resolution (shell `BoolFlag`).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `xai-grok-pager/src/app/cli.rs` | config / CLI | request-response | self (`PagerArgs` clap attrs) | exact |
| `xai-grok-pager/src/app/mod.rs` | utility | transform (error context string) | self `"Grok Build TUI"` site | exact |
| `xai-grok-pager/src/views/welcome/hero_box.rs` | component | transform (render const) | self `HERO_SUBTITLE` | exact |
| `xai-grok-pager/src/views/welcome/mod.rs` | component | transform (badge / ZDR / trust copy) | self `VersionBadgeMode` + trust lines | exact |
| `xai-grok-pager/src/project_picker/mod.rs` | component | request-response (QuestionView) | self `build_project_question` | exact |
| `xai-grok-pager/src/app/dispatch/notes.rs` | controller / dispatch | event-driven (Action → Effect) | self `dispatch_enter_feedback_mode` / `dispatch_send_feedback` | exact |
| `xai-grok-pager/src/app/dispatch/billing.rs` | controller / dispatch | request-response | self product string consts + upsell copy | exact |
| `xai-grok-pager/src/app/app_view.rs` | component / store | transform (subscribe message) | self `"Subscribe to use Grok Build"` | exact |
| `xai-grok-pager/src/settings/defs.rs` | config | transform (settings meta) | self `auto_update` SettingMeta | exact |
| `xai-grok-pager/src/settings/registry.rs` | config | transform (effective defaults) | self `unwrap_or(true)` + registry assert | exact |
| `xai-grok-pager/src/brand.rs` (optional new) | utility | transform | surgical string sites if deps hurt; else skip | partial |
| `xai-grok-shell/src/auth/oidc/login.rs` | service / route | request-response (OAuth HTML) | self `callback_response` | exact |
| `xai-grok-shell/src/agent/config.rs` | config | request-response (resolve flags) | self `resolve_feedback` / `resolve_telemetry_mode` | exact |
| `xai-grok-shell/src/extensions/feedback.rs` | service | request-response (ACP ext) | self `handle_feedback` gate | exact |
| `xai-grok-pager-bin/src/main.rs` | controller / composition root | request-response | self `should_check_for_updates`, `run_update_command`, Sentry init | exact |
| `xai-grok-pager-minimal/src/welcome.rs` | component | transform | self product Span | exact |
| `xai-grok-update/src/auto_update.rs` | service | request-response / file-I/O | self `print_update_status` + `run_update_if_available` | exact |
| `xai-grok-update/src/minimum_version.rs` | service | request-response | self `enforce_minimum_version_or_exit` | exact |
| `xai-grok-pager-bin/tests/home_isolation.rs` (extend or sibling) | test | file-I/O + process spawn | self hermetic `bum` spawn + env traps | exact |
| Shell `config.rs` unit tests (`resolve_feedback_*`) | test | request-response | self `resolve_feedback_defaults_to_true_when_unset` | exact |
| Pager billing/dispatch unit tests | test | request-response | `dispatch/tests/billing.rs` string asserts | exact |
| New `p8_*` unit tests (pager / update / bin) | test | request-response | Phase 1 isolation + existing `#[serial]` config tests | role-match |

**Do not modify (anti-analog):** `default_models.json` / model catalog `"Grok Build (xAI)"`, model id `grok-build`, SuperGrok commercial SKU/URL host, internal `GROK_*` env names, crate package names.

## Pattern Assignments

### `xai-grok-pager/src/app/cli.rs` (config, request-response)

**Analog:** same file — clap `PagerArgs` attributes.

**Core clap identity pattern** (lines 418–421):
```rust
#[derive(Debug, Clone, Parser)]
#[command(
    name = "grok",              // → "bum"
    version = version_with_channel(),
    about = "Grok Build TUI",  // → "bum TUI"
    disable_version_flag = true,
    // keep help_template and rest of attrs
)]
pub struct PagerArgs { /* … */ }
```

**Copy target (UI-SPEC):** `name = "bum"`, `about = "bum TUI"`. Do not rewrite full help manuals—product-identity strings only.

**Test analog:** assert clap metadata / help contains `bum` and not product “Grok Build TUI” (new `p8_cli_brand` unit under pager).

---

### `xai-grok-pager/src/views/welcome/hero_box.rs` (component, transform)

**Analog:** same file — `HERO_SUBTITLE` constant.

**Const pattern** (line 31):
```rust
const HERO_SUBTITLE: &str = "Thanks for trying Grok Build, give feedback with /feedback!";
// → "Thanks for using bum."
```

**Rules from UI-SPEC:** one line; do **not** advertise `/feedback`; keep existing `subtitle_rows` hide-when-info behavior (lines 37–39). Geometry (`HERO_BOX_MIN_WIDTH`, pads) unchanged.

---

### `xai-grok-pager/src/views/welcome/mod.rs` (component, transform)

**Analog:** same file — `VersionBadgeMode` span builders + trust/ZDR copy.

**Full badge product + Beta marketing** (lines 437–453):
```rust
VersionBadgeMode::Full { .. } => {
    spans.push(Span::styled(
        "Grok Build  ",   // → "bum  " (or natural "bum" + spacer)
        Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(
        format!("{}{}", xai_grok_version::VERSION, channel),
        Style::default().fg(theme.gray),
    ));
    spans.push(Span::styled(
        " Beta",          // → omit stock Beta marketing chrome
        Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::BOLD),
    ));
}
```

**HeroInline** (lines 466–472):
```rust
spans.push(Span::styled(
    "Grok Build Beta  ",  // → "bum  " without Beta
    Style::default()
        .fg(theme.text_primary)
        .add_modifier(Modifier::BOLD),
));
```

**ZDR / trust product lines** (scouted ~763, ~950):
```rust
"Grok Build is not yet available for this account."
// → "bum is not yet available for this account."

"Grok Build may run or modify contents in this directory,"
// → "bum may run or modify contents in this directory,"
// L2 "posing security risks." unchanged
```

**Style to copy:** existing `Span::styled` + `theme.text_primary` / `Modifier::BOLD` on product badge only—no new colors for “bum”.

---

### `xai-grok-pager/src/project_picker/mod.rs` (component, request-response)

**Analog:** same file — `build_project_question`.

**Question copy pattern** (lines 79–87):
```rust
ProjectQuestion {
    question: Question {
        question: "Run Grok Build in a project directory?\n\n\
             This gives Grok Build full context of your codebase for better results."
            .into(),
        // → UI-SPEC: "Run bum in a project directory?\n\nThis gives bum full context…"
        id: None,
        options,  // option labels UNCHANGED
        multi_select: Some(false),
    },
    resolved_paths,
    dont_ask_index,
}
```

**Test pattern in same module** (lines 98–103): extend or add assert on question string contains `bum` / not product Grok Build.

---

### `xai-grok-pager/src/app/dispatch/notes.rs` (controller, event-driven)

**Analog:** same file — feedback enter/send dispatchers. Cross-pattern: early-return system block without Effect (empty submit already does this).

**Current enter mode** (lines 23–28) — **replace** with disabled short-circuit (stay Normal, no Feedback mode):
```rust
pub(super) fn dispatch_enter_feedback_mode(app: &mut AppView) -> Vec<Effect> {
    with_active_agent(app, |agent| {
        agent.prompt_input_mode = PromptInputMode::Feedback; // → do NOT enter Feedback
        agent.prompt.set_text("");
    });
    vec![]
}
```

**Target pattern (RESEARCH + UI-SPEC):**
```rust
pub(super) fn dispatch_enter_feedback_mode(app: &mut AppView) -> Vec<Effect> {
    with_active_agent(app, |agent| {
        agent.scrollback.push_block(RenderBlock::system(
            "Feedback is disabled in bum (no phone-home).".to_string(),
        ));
        // remain PromptInputMode::Normal — do not collect
    });
    vec![]  // never Effect::SendFeedback
}
```

**Current send path** (lines 43–79) — always thank-you + Effect:
```rust
agent.scrollback.push_block(RenderBlock::system(
    "Thanks for the feedback! The Grok Build team is on it.".to_string(),
));
vec![Effect::SendFeedback {
    agent_id: id,
    session_id,
    feedback_text: trimmed,
}]
```

**Empty-submit short-circuit already in file** (lines 56–61) — copy this shape for disabled path:
```rust
if trimmed.is_empty() {
    agent.scrollback.push_block(RenderBlock::system(
        "Please provide feedback text.".to_string(),
    ));
    return vec![];
}
```

**Imports to keep** (lines 3–9): `with_active_agent`, `Effect`, `RenderBlock`, `PromptInputMode`.

**Router wiring** (unchanged): `router.rs` maps `Action::EnterFeedbackMode` / `SendFeedback` — only handler bodies change.

---

### `xai-grok-pager/src/app/dispatch/billing.rs` + `app_view.rs` (controller / component)

**Analog:** same sites — product-named strings only; keep SuperGrok + grok.com hosts.

**Const + upsell** (`billing.rs` lines 115, 154, 331):
```rust
pub(crate) const FREE_USAGE_USER_MESSAGE: &str =
    "You've reached your free Grok Build usage limit… Get SuperGrok … https://grok.com/supergrok?referrer=grok-build";
// product "Grok Build" → "bum"; keep SuperGrok + URL host

"Purchase credits to keep using Grok Build"  // → "… bum"
"Get the most out of Grok Build. Highest usage limits."  // → "… bum …"
```

**app_view** (~6681):
```rust
message: "Subscribe to use Grok Build".into(),  // → "Subscribe to use bum"
```

**Test analog:** `dispatch/tests/billing.rs` string equality asserts (~260, ~835)—flip expected product word in same PR.

---

### `xai-grok-shell/src/auth/oidc/login.rs` (service, request-response)

**Analog:** same file — browser callback HTML message.

**Callback response pattern** (lines 159–169):
```rust
fn callback_response(result: &CallbackResult) -> (StatusCode, Html<String>) {
    let (title, message) = match result {
        Ok(_) => (
            "Signed in",
            "You can close this window and return to Grok Build.",
            // → "You can close this window and return to bum."
        ),
        Err(_) => ("Access denied", "Close this window and try again."),
    };
    (
        StatusCode::OK,
        Html(callback_page(title, message, result.is_ok())),
    )
}
```

Leave provider IdP branding alone; only product return text.

---

### `xai-grok-pager-bin/src/main.rs` (composition root, request-response)

**Analog:** same file — update gate, CLI update command, version banner, Sentry.

#### 1) Auto-update chokepoint (OPS-01)

**Current gate** (lines 1997–2010):
```rust
/// Centralized gate for all auto-update checks. Add new suppression
/// rules here — not at each call site.
fn should_check_for_updates(no_auto_update_flag: bool) -> bool {
    if cfg!(debug_assertions) {
        return false;
    }
    if no_auto_update_flag {
        return false;
    }
    if std::env::var_os("GROK_DISABLE_AUTOUPDATER").is_some() {
        return false;
    }
    true  // RELEASE DEFAULT ON ← hard-off target
}
```

**Target pattern (RESEARCH):**
```rust
fn should_check_for_updates(_no_auto_update_flag: bool) -> bool {
    // Quiet fork: never probe stock x.ai auto-update channel.
    false
}
```

Call sites already branch on this (startup ~887, leader ~1184, stdio path ~1894)—one flip kills background + leader checker spawn.

**Leader wiring already respects gate** (lines 1184–1199):
```rust
let leader_auto_update = if !should_check_for_updates(no_auto_update || a.no_auto_update) {
    tracing::info!("Leader auto-update disabled");
    None
} else {
    Some(LeaderAutoUpdateConfig { /* ensure_latest_on_disk */ })
};
```
Prefer gate → `None` over rewriting `run_auto_update_checker`.

#### 2) Explicit `bum update` no-op

**Current** (lines 2037–2072) runs `check_update_status` / `run_update` against stock channel.

**Target:** early return before network:
```rust
async fn run_update_command(/* … */) -> Result<()> {
    // Quiet fork: stock channel disabled (UI-SPEC).
    println!("Stock auto-update is disabled in bum. Install or build locally to upgrade.");
    println!("This fork does not download updates from the stock x.ai channel.");
    return Ok(()); // exit 0 preferred
}
```

#### 3) Version banner (ID-02)

```rust
// ~881
"Grok Build (pager) - v{}"  // → "bum (pager) - v{}"
```

#### 4) Sentry (OPS-02 — keep off path)

**Init pattern** (lines 1483–1488):
```rust
let _sentry_guard = xai_grok_telemetry::sentry::init(xai_grok_telemetry::sentry::Config {
    client: "grok-pager",
    client_version: PAGER_CLIENT_VERSION,
    release: env!("VERSION_WITH_COMMIT"),
    disabled: xai_grok_shell::agent::config::is_error_reporting_disabled_sync(),
});
```
**Do not remove Sentry crate.** Ensure `is_error_reporting_disabled_sync` stays true under default Disabled telemetry (already inherits). Optional: force `disabled: true` for fork hard privacy (discretion).

---

### `xai-grok-update/src/auto_update.rs` (service, request-response)

**Analog:** same file — printers + first-run default trap.

**Print status product strings** (lines 60–99):
```rust
println!("Grok Build - v{} [{}]", …);           // → "bum - v{} …"
"A new version of Grok Build is available…"     // eliminate from user path / disabled path
println!("Grok Build - v{} (latest: {}){}", …);
```

**Critical first-run opt-in trap** (lines 481–494):
```rust
// Skip if explicitly disabled.
if current_config.cli.auto_update == Some(false) {
    return Ok(false);
}

// Resolve effective auto_update: None defaults to true (first-run).
let auto_update = current_config.cli.auto_update.unwrap_or(true);  // → unwrap_or(false)

if current_config.cli.auto_update.is_none()
    && let Err(e) = config::update_config(|st| {
        if st.cli.auto_update.is_none() {
            st.cli.auto_update = Some(true);  // → stop persisting true; omit or Some(false)
        }
    })
    .await
{ /* … */ }
```

Also flip other `unwrap_or(true)` auto_update sites in this file (~435, ~2301) in the same plan.

**User-facing install prompt strings** (~526): rebrand or unreachable after hard-off; prefer unreachable via gates.

---

### `xai-grok-update/src/minimum_version.rs` (service, request-response)

**Analog:** same file — `enforce_minimum_version` / `_or_exit`.

**Stock force-upgrade path** (lines 205–224):
```rust
// `None` is "default on"; only explicit `false` opts out.
if cfg.cli.auto_update == Some(false) {
    return Err(MinimumVersionError::AutoUpdateDisabled { current, minimum });
}
// … fetch + run_install_script from stock channel …
eprintln!(
    "This version of Grok ({current}) is no longer supported. Updating to {target}…"
);
```

**Public entry** (lines 267–291):
```rust
pub async fn enforce_minimum_version_or_exit(update_config: &UpdateConfig) {
    let min = match resolve_floor_or_error() {
        Ok(None) => return,
        Ok(Some(m)) => m,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };
    match enforce_minimum_version(Some(&min), update_config).await {
        Ok(EnforcementOutcome::Allowed) => {}
        Ok(EnforcementOutcome::Upgraded) => {
            eprintln!("Update installed. Run `grok` to start.");
            std::process::exit(0);
        }
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}
```

**Target pattern (OPS-01):** hard no-op at entry (ignore floor / never install stock):
```rust
pub async fn enforce_minimum_version_or_exit(_update_config: &UpdateConfig) {
    // Quiet fork: never force-upgrade to stock Grok Build.
}
```
Or always `Allowed` without network. Prefer early return so call sites in pager-bin stay unchanged.

---

### `xai-grok-pager/src/settings/defs.rs` + `registry.rs` (config)

**Analog:** same files — settings meta + effective default mirror.

**defs.rs auto_update** (lines 1220–1231):
```rust
SettingMeta {
    key: "auto_update",
    label: "Auto-update",
    description: "Automatically download and install pager updates on startup. \
                  Restart required.",
    // → UI-SPEC: "Stock update channel (disabled in bum)."
    kind: SettingKind::Bool { default: true },  // → default: false
    restart_required: true,
    …
}
```

**registry effective value** (~669):
```rust
"auto_update" => Some(SettingValue::Bool(pager.auto_update.unwrap_or(true))),
// → unwrap_or(false)
```

**Registry unit assert** (~878–882) must flip with default:
```rust
("auto_update", SettingKind::Bool { default }) => {
    assert!(
        *default,  // → assert!(!*default) or assert_eq!(*default, false)
        "auto_update registry default must be true \
         (matches auto_update.rs's `.unwrap_or(true)`)"
    );
}
```

Update comments at registry lines 272–273 (`None` → default true) in lockstep.

---

### `xai-grok-shell/src/agent/config.rs` (config, request-response)

**Analog:** same file — `resolve_feedback` / `resolve_telemetry_mode` / sync Sentry gate.

**Telemetry default already correct** (lines 2079–2099):
```rust
pub(crate) fn resolve_telemetry_mode(&self) -> Resolved<TelemetryMode> {
    // requirement → env → config → remote …
    Resolved::new(TelemetryMode::Disabled, ConfigSource::Default)
}
```
**Harden (discretion):** for quiet fork, optionally ignore remote telemetry re-enable after default (Pitfall 7)—document decision in plan if applied.

**Feedback default trap** (lines 2167–2178):
```rust
pub(crate) fn resolve_feedback(&self) -> Resolved<bool> {
    let ff = self.remote_settings.as_ref().and_then(|s| s.feedback_enabled);
    BoolFlag::env("GROK_FEEDBACK_ENABLED")
        .requirement(self.requirements.feedback.pinned())
        .config(self.features.feedback)
        .feature_flag(ff)
        .default(true)   // → .default(false)
        .resolve()
}
```

**Mirror pattern for default-false flags** — `resolve_two_pass_compaction` (lines 2179–2188):
```rust
BoolFlag::env("GROK_TWO_PASS_COMPACTION")
    .config(self.features.two_pass_compaction)
    .feature_flag(ff)
    .default(false)
    .resolve()
```

**Sentry inherit** (lines 2936–2941):
```rust
pub fn is_error_reporting_disabled_sync() -> bool {
    !SyncBoolFlag::new(error_reporting_enabled_from_toml)
        .disable_env("DISABLE_ERROR_REPORTING")
        .enable_env(|| env_bool("GROK_ERROR_REPORTING"))
        .inherit(|| !is_telemetry_disabled_sync())
        .resolve()
}
```
Leave structure; verify Disabled telemetry keeps Sentry off.

**Unit test to flip** (lines 8818–8826):
```rust
#[test]
#[serial]
fn resolve_feedback_defaults_to_true_when_unset() {
    // … env clear …
    let r = cfg.resolve_feedback();
    assert!(r.value, "feedback should be true by default");
    // → rename to …_false…; assert!(!r.value, "feedback should be false by default");
}
```

Keep `#[serial]` + env clear pattern for flag tests.

---

### `xai-grok-shell/src/extensions/feedback.rs` (service, request-response)

**Analog:** same file — shell already gates; align copy with UI-SPEC (no easy opt-in primary message).

**Current gate** (lines 82–87):
```rust
if !agent.cfg.borrow().is_feedback_enabled() {
    return Err(acp::Error::internal_error().data(
        "Feedback is disabled. To enable, set GROK_FEEDBACK_ENABLED=true or \
         [features] feedback = true in config.toml.",
    ));
}
```

**Target:** UI-SPEC primary copy without advertising opt-in as UX:
```rust
"Feedback is disabled in bum (no phone-home)."
```

With `resolve_feedback` default false, this path becomes the normal shell refusal.

---

### `xai-grok-pager-minimal/src/welcome.rs` (component, transform)

**Analog:** same file.

**Product span** (lines 73–80):
```rust
Span::styled(
    "Grok Build",  // → "bum"
    Style::default()
        .fg(theme.accent_user)
        .add_modifier(Modifier::BOLD),
),
```

Keep layout (logo + info lines + muted version). PTY e2e that assert `"Grok Build"` → `"bum"`.

---

### Optional `xai-grok-pager/src/brand.rs` (utility)

**Analog:** none required. RESEARCH Pattern 1: surgical edits first; extract only if ≥2 crates share format helpers.

If added:
```rust
//! User-facing product chrome tokens (quiet fork).
pub const PRODUCT_NAME: &str = "bum";
pub fn product_name() -> &'static str { PRODUCT_NAME }
```
Pager-local only unless a leaf crate is already a shared dependency (avoid new pager↔update edges for a 3-letter string).

---

### Tests: hermetic / gate (Phase 1 style)

**Analog:** `xai-grok-pager-bin/tests/home_isolation.rs` + shell `#[serial]` resolve tests.

**Hermetic spawn + privacy env** (home_isolation lines 97–111):
```rust
let mut cmd = Command::new(bum_bin());
cmd.arg("version")
    .env_clear()
    .env("PATH", …)
    .env("HOME", &home)
    .env("BUM_HOME", &bum_home)
    .env("GROK_HOME", &grok_trap)
    .env("GROK_TELEMETRY_ENABLED", "false")
    .env("GROK_FEEDBACK_ENABLED", "false")
    .env("GROK_DISABLE_AUTOUPDATER", "1");
```

**Phase 8 extensions (pattern reuse):**
- Spawn `bum update` / `bum --help` under same hermetic env; assert disabled message / `bum` about; no writes that imply stock channel fetch.
- Pure unit tests preferred for `should_check_for_updates` always false (if made `pub(crate)` or tested via bin filter).
- Filter naming: `p8_*` per RESEARCH Validation Architecture.

**Do not** require live network to x.ai for green tests.

---

## Shared Patterns

### 1. Surgical product string rebrand (not mass sed)

**Source:** UI-SPEC inventory + Phase 1 “user-facing only, crates stay `xai-grok-*`”.
**Apply to:** all ID-02 chrome files listed above.
- Replace product “Grok Build” / CLI `grok` **identity** strings with `bum`.
- **Never** touch model catalog `Grok Build (xAI)`, model id `grok-build`, SuperGrok commercial, rustdoc/crate descriptions unless user-visible chrome.

### 2. Hard-off at chokepoint (OPS-01)

**Source:** `pager-bin` `should_check_for_updates` (main.rs ~1997–2010); `auto_update` defaults; `enforce_minimum_version_or_exit`.
**Apply to:** startup, leader hourly, stdio background, explicit CLI, min-version.
```text
should_check_for_updates → always false
cli.auto_update unwrap_or(false) + no first-run true persist
run_update_command → print UI-SPEC message, Ok(())
enforce_minimum_version_or_exit → immediate return
settings auto_update default false + description
```

### 3. Feedback disabled at entry (OPS-02 adjacent)

**Source:** `notes.rs` empty-submit early return; shell `handle_feedback` gate; UI-SPEC disabled copy.
**Apply to:** pager `dispatch_enter_feedback_mode` / `dispatch_send_feedback` + `resolve_feedback` default false.
```text
System block: "Feedback is disabled in bum (no phone-home)."
No Effect::SendFeedback / no team thank-you
Stay PromptInputMode::Normal
```

### 4. Telemetry / Sentry remain Disabled by default

**Source:** `resolve_telemetry_mode` default `TelemetryMode::Disabled`; Sentry `disabled: is_error_reporting_disabled_sync()`.
**Apply to:** verify-only unless remote re-enable hardening chosen.
- Local tracing under `~/.bum` OK (Phase 1 paths).
- No new phone-home paths; no advertising opt-in in primary UX copy.

### 5. Config flag resolution via BoolFlag

**Source:** `agent/config.rs` `resolve_feedback` / `resolve_two_pass_compaction`.
**Apply to:** feedback default flip only (copy `.default(false)` pattern from two_pass_compaction).

### 6. Unit test conventions

| Concern | Pattern source |
|---------|----------------|
| Env-mutating flag tests | `#[serial]` + `unsafe` env remove/set in `agent/config.rs` tests |
| Settings default assert | `settings/registry.rs` match on `SettingKind::Bool { default }` |
| Hermetic binary | `pager-bin/tests/home_isolation.rs` `env_clear` + `BUM_HOME` + traps |
| Dispatch string | `dispatch/tests/billing.rs` equality on user-facing copy |
| Model label regression | **do not break** existing model catalog tests expecting `Grok Build (xAI)` |

### 7. Action → Effect purity (TUI)

**Source:** architecture — dispatch stays free of network; `Effect::SendFeedback` is the I/O boundary.
**Quiet fork:** refuse **before** Effect so effects/HTTP never run for feedback.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | No greenfield modules required; optional `brand.rs` is trivial const only |

## Metadata

**Analog search scope:**
- `crates/codegen/xai-grok-pager` (cli, welcome, project_picker, dispatch, settings)
- `crates/codegen/xai-grok-pager-bin` (main, tests/home_isolation)
- `crates/codegen/xai-grok-pager-minimal` (welcome)
- `crates/codegen/xai-grok-update` (auto_update, minimum_version)
- `crates/codegen/xai-grok-shell` (agent/config, extensions/feedback, auth/oidc, agent/app leader)
- `.planning/phases/01-product-identity-isolated-home/01-PATTERNS.md` (test/isolation lineage)

**Files scanned:** ~40 primary sites (grep + targeted reads)
**Pattern extraction date:** 2026-07-17

**Key planner guidance:**
1. Wave 0: flip known default assertions (`resolve_feedback`, settings `auto_update`) RED first.
2. Wave 1: ID-02 chrome inventory (UI-SPEC checklist).
3. Wave 2: OPS-01 chokepoints (`should_check_for_updates`, unwrap_or, min-version, update CLI).
4. Wave 3: OPS-02 feedback short-circuit + feedback default false + telemetry/Sentry verify.
5. Wave 4: `p8_*` gates + model-label regression + home_isolation still green.
