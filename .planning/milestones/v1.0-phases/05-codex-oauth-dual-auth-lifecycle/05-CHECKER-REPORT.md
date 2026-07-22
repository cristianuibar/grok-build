# Phase 5 Plan Check — Codex OAuth & dual auth lifecycle

**Checked:** 2026-07-16 (re-verify after revision)  
**Plans:** 05-01 … 05-06 (6)  
**Verdict:** **PASS**  
**Issues:** 0 blocker(s), 1 residual warning (non-blocking)

---

## Prior blockers — disposition

| ID | Issue | Status |
|----|-------|--------|
| **B1** | `05-VALIDATION.md` missing | **FIXED** — present, complete (test map, per-task verify, Wave 0, cargo hygiene, AUTH-05 prepare-path proof, sign-off) |
| **B2** | RESEARCH Open Questions not RESOLVED | **FIXED** — `## Open Questions (RESOLVED)` with Q1–Q5 each marked RESOLVED to plan locks |
| **B3** | AUTH-05 production prepare-time call site missing | **FIXED** — 05-05 Task 2 wires `ensure_fresh_codex_session_key` on prepare path; `must_haves.key_links` prepare → CodexRefresher; `codex_pre_request_refresh_updates_snapshot` in Wave 0 + Task 2 `<automated>` |

---

## Prior warnings — disposition

| ID | Issue | Status |
|----|-------|--------|
| **W1** | ChatGPT-Account-ID not in automated verify | **FIXED** — 05-05 Task 3: `chatgpt_account_id_header_on_codex` + regression `no_proxy_headers_on_codex` |
| **W2** | Authorize URL unit test not gated | **FIXED** — 05-03 Task 1 chains `codex_authorize_url_includes_pkce_and_localhost_callback && codex_login_persists_slot` |
| **W3** | Device-code automated proof soft | **FIXED** — Wave 0 + 05-03 Task 2: `codex_device_endpoints_use_deviceauth` |
| **W4** | `bum auth status` handler vs pure format | **FIXED** — 05-04 Task 2: `run_cli_auth_status` + pure formatter + clap |
| **W5** | Plan 03 file surface high | **ACK** — residual non-blocking; split already Task 1/2; do not expand further |

---

## Goal-backward summary

| Success criterion / requirement | Planned delivery | Status |
|---------------------------------|------------------|--------|
| AUTH-02 ChatGPT OAuth PKCE + device; bum store only | 05-03 + Wave 0 authorize/device/login contracts | Covered |
| AUTH-03 selective logout | 05-02 clear API + 05-04 CLI | Covered |
| AUTH-04 per-provider status | 05-02 pure format + 05-04 `run_cli_auth_status` | Covered |
| AUTH-05 independent refresh, no cross-wipe | 05-05 pure isolation + **prepare-time invoker** + cache invalidate | Covered |
| Phase goal “keep both sessions healthy” | prepare ensure-fresh spends RT, persists codex only, invalidates snapshot | Covered |
| Deferred Phase 6/7/import/Platform key | 05-06 audit + explicit exclusions | OK |

---

## Dimension results

| # | Dimension | Result |
|---|-----------|--------|
| 1 | Requirement coverage | PASS — AUTH-02..05 in frontmatter + tasks |
| 2 | Task completeness | PASS — all auto tasks have files/action/verify/done |
| 3 | Dependency correctness | PASS — acyclic 01→02→03; 04←02+03; 05←03+04; 06←03+04+05 |
| 4 | Key links planned | PASS — prepare → CodexRefresher → mutate_provider_store_or_prune(PROVIDER_CODEX) → invalidate cache; login/logout/status wired |
| 5 | Scope sanity | WARNING residual — 05-03 ~11 files / 05-05 3 tasks (~target max); acceptable |
| 6 | Verification derivation | PASS — user-observable truths; call-site + header + device + status handler proves |
| 7 | Context compliance | PASS — D-01..D-13 / CONTEXT decisions honored; deferred excluded |
| 7b | Scope reduction | PASS — no fake “v1 static” of locks; Task 2/3 cache-stub is inter-task order only with disk+bearer still required |
| 7c | Architectural tier | PASS — matches RESEARCH responsibility map |
| 8 | Nyquist compliance | PASS — VALIDATION.md exists; all tasks have `<automated>`; sampling continuity OK; Wave 0 lists prepare + account header + deviceauth + authorize + status handler |
| 9 | Cross-plan data contracts | PASS — provider-slot RMW shared by login/logout/refresh |
| 10 | CLAUDE.md / AGENTS | PASS — Rust brownfield, no stock import, ChatGPT OAuth primary |
| 11 | Research resolution | PASS — Open Questions (RESOLVED) |
| 12 | Pattern compliance | PASS — plans cite PATTERNS analogs |

### Dimension 8: Nyquist Compliance

| Task | Plan | Wave | Automated Command | Status |
|------|------|------|-------------------|--------|
| 01-01 | 01 | 1 | `auth_codex_lifecycle --list` + smoke | ✅ |
| 01-02 | 01 | 1 | pager clap defaults | ✅ |
| 02-01 | 02 | 2 | mutate_provider + auth_multi_slot | ✅ |
| 02-02 | 02 | 2 | auth_status_format_paste_safe | ✅ |
| 03-01 | 03 | 3 | authorize_url + login_persists | ✅ |
| 03-02 | 03 | 3 | provider clap + deviceauth + login | ✅ |
| 04-01 | 04 | 4 | selective / bare / --all logout | ✅ |
| 04-02 | 04 | 4 | auth status clap + run_cli_auth_status + format | ✅ |
| 05-01 | 05 | 5 | refresh_isolates + invalid_grant | ✅ |
| 05-02 | 05 | 5 | **codex_pre_request_refresh_updates_snapshot** | ✅ |
| 05-03 | 05 | 5 | chatgpt_account_id_header + no_proxy regression | ✅ |
| 06-01 | 06 | 6 | full lifecycle suite | ✅ |
| 06-02 | 06 | 6 | multi_slot + provider_routing + check | ✅ |

Sampling: all waves ≥2/3 automated → ✅  
Wave 0: `auth_codex_lifecycle.rs` + clap scaffolds listed → ✅  
Overall: ✅ PASS

---

## Residual (non-blocking)

### W5. [scope_sanity] Plan 03 / 05 surface
- **Severity:** warning (info-level residual)
- Plan 03 still ~11 files; Plan 05 has 3 tasks and ~10 files. Within execute budget if not expanded.
- **Action:** none required before execute.

---

## What remains strong (do not regress)

- Cargo hygiene: one TESTNAME, `&&` chains, no shell `--lib` phase gate
- Threat models + STRIDE on every plan
- COVERAGE.md INTEGRATE/OPT-OUT matches CONTEXT
- Dual-slot isolation contracts named end-to-end including prepare-time refresh
- UI-SPEC copy locks in 05-03/04
- Deferred Phase 6/7/import explicitly excluded

---

## Structured issues

```yaml
issues: []
```

---

## Recommendation

**PASS.** Prior B1–B3 and W1–W4 are addressed. Plans are ready for `/gsd:execute-phase 5`.

No return to planner required for blockers.
