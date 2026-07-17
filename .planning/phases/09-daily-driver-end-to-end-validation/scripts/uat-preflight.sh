#!/usr/bin/env bash
# Phase 9 UAT preflight — non-secret helper for live dual-login OPS-03..06.
# Prints numbered steps mirroring 09-UAT.md; runs fail-closed secret path + phase-diff gates.
# NEVER stores tokens, cats auth.json, curls live APIs with secrets, or auto-marks UAT PASS.
# Operator (Cristian) executes live login/matrix; this script only prepares and guards.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PHASE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
# Repo root: walk up from phase dir (.planning/phases/09-...)
REPO_ROOT="$(cd "${PHASE_DIR}/../../.." && pwd)"
if [[ ! -f "${REPO_ROOT}/Cargo.toml" ]]; then
  REPO_ROOT="$(git -C "${PHASE_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${REPO_ROOT}")"
fi

PHASE_REL=".planning/phases/09-daily-driver-end-to-end-validation"
UAT_MD="${PHASE_DIR}/09-UAT.md"
BUM_BIN="${REPO_ROOT}/target/debug/bum"
export BUM_HOME="${BUM_HOME:-${HOME}/.bum}"

die() { echo "ERROR: $*" >&2; exit 1; }
ok()  { echo "OK: $*"; }
info(){ echo "→ $*"; }

echo "=============================================="
echo " bum Phase 9 — UAT preflight (non-secret)"
echo "=============================================="
echo "Repo:      ${REPO_ROOT}"
echo "Phase dir: ${PHASE_DIR}"
echo "BUM_HOME:  ${BUM_HOME}  (override with env; optional isolation: mktemp -d)"
echo "Checklist: ${UAT_MD}"
echo "This script NEVER auto-marks UAT PASS (D-15)."
echo

# ── Required secrets hygiene gates (C1-L4, C2-L1, C3-L1) ──────────────────
echo "=== REQUIRED secret gates (fail-closed) ==="

# (a)+(b) Scoped credential/artifact basenames only — NOT bare *token* (C3-L1)
# Matches: auth.json, credentials.json, .oauth, *.pem/*.p12/*.pfx, id_rsa/id_ed25519/id_ecdsa
CRED_PATH_RE='(^|/)(auth\.json|credentials\.json|\.oauth|[^/]+\.pem|[^/]+\.p12|[^/]+\.pfx|id_rsa|id_ed25519|id_ecdsa)($|/)'

if git -C "${REPO_ROOT}" ls-files | rg -q '(^|/)(auth\.json)$'; then
  die "tracked auth.json present in git index (D-12 / C1-L4)"
fi
ok "no tracked auth.json basename"

if git -C "${REPO_ROOT}" ls-files | rg -qi "${CRED_PATH_RE}"; then
  echo "FAIL: scoped credential/artifact path(s) tracked:" >&2
  git -C "${REPO_ROOT}" ls-files | rg -i "${CRED_PATH_RE}" >&2 || true
  die "credential basenames in tree (C3-L1) — remove before UAT commit"
fi
ok "no tracked scoped credential basenames (auth.json, credentials.json, .oauth, *.pem/p12/pfx, id_* keys)"

# (c) Scoped phase-diff content scan (working tree + index) for secret shapes (C2-L1)
SECRET_SHAPE_RE='eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}|BEGIN (RSA |OPENSSH )?PRIVATE|"access_token"[[:space:]]*:[[:space:]]*"[^"]{20,}"|"refresh_token"[[:space:]]*:[[:space:]]*"[^"]{20,}"'
PHASE_DIFF="$(
  {
    git -C "${REPO_ROOT}" diff HEAD -- "${PHASE_REL}"
    git -C "${REPO_ROOT}" diff --cached -- "${PHASE_REL}"
  } 2>/dev/null || true
)"
if [[ -n "${PHASE_DIFF}" ]] && printf '%s\n' "${PHASE_DIFF}" | rg -n "${SECRET_SHAPE_RE}" >/dev/null 2>&1; then
  echo "FAIL: secret-shaped content in phase working-tree or index diff:" >&2
  printf '%s\n' "${PHASE_DIFF}" | rg -n "${SECRET_SHAPE_RE}" >&2 || true
  die "phase-diff secret scan failed (C2-L1) — redact before commit"
fi
ok "phase-diff clean of JWT / private-key / opaque OAuth field dumps"
echo
info "Re-run this script (or the git ls-files / git diff guards in 09-UAT.md) before every commit of UAT evidence."
echo

# ── Binary ────────────────────────────────────────────────────────────────
echo "=== Binary ==="
if [[ ! -x "${BUM_BIN}" ]]; then
  info "Binary missing at ${BUM_BIN} — building xai-grok-pager-bin (optional auto-build)"
  (cd "${REPO_ROOT}" && cargo build -p xai-grok-pager-bin) || die "cargo build failed"
fi
if [[ -x "${BUM_BIN}" ]]; then
  ok "binary: ${BUM_BIN}"
else
  die "binary still missing after build: ${BUM_BIN}"
fi
if command -v git >/dev/null 2>&1; then
  COMMIT="$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  info "git commit: ${COMMIT}"
fi
echo

# ── CLI chrome sample (C1-M4) — print only; never auto-pass ───────────────
echo "=== CLI chrome sample (C1-M4) — operator judges PASS vs BLOCKER DECISION ==="
info "Primary product name should present as bum (not residual stock Grok product chrome)."
if [[ -x "${BUM_BIN}" ]]; then
  set +e
  HELP_HEAD="$("${BUM_BIN}" --help 2>&1 | head -n 40)"
  set -e
  printf '%s\n' "${HELP_HEAD}"
  echo "--- end --help head ---"
else
  info "skip --help (no binary)"
fi
echo
info "Record chrome outcome in 09-UAT.md Preflight — CLI chrome table."
echo

# ── Numbered operator steps (mirror 09-UAT.md) ────────────────────────────
echo "=== Numbered preflight steps (operator executes; script does not login) ==="
cat <<EOF
 1. Disposable workspace (C1-M3 — REQUIRED)
    - Create a disposable git worktree or throwaway fixture dir/file.
    - Do NOT use the primary bum/grok-build checkout as the only edit target.
    - Record path + initial: git status --short (or fixture baseline).
    Example:
      DISPOSABLE=\$(mktemp -d /tmp/bum-uat-ws-XXXXXX)
      git -C ${REPO_ROOT} worktree add "\$DISPOSABLE" HEAD
      cd "\$DISPOSABLE" && git status --short

 2. Product home
    - Default BUM_HOME=${BUM_HOME}
    - Isolation (recommended for clean smoke):
        export BUM_HOME=\$(mktemp -d /tmp/bum-uat-home-XXXXXX)
    - Script refuses to write secrets into the phase dir; credentials stay under BUM_HOME only.

 3. Network + accounts
    - Confirm xAI account and ChatGPT/Codex account usable; network to IdP + APIs.

 4. Dual login (operator only — this script will not run login)
      ${BUM_BIN} login
      ${BUM_BIN} login --provider codex

 5. Auth status without secrets
      ${BUM_BIN} auth status
    - Expect both slots usable. NEVER paste tokens into 09-UAT.md.

 6. CLI chrome (C1-M4)
      ${BUM_BIN} --help
    - Check login / auth status /model labels present as bum daily-driver identity.
    - Residual product-visible Grok chrome → BLOCKER DECISION for Plan 04 (not silent ignore).

 7. Live matrix (fill 09-UAT.md)
    - OPS-03: xAI model (default grok-build) read + edit/shell on disposable workspace
    - OPS-04: GPT-5.6 (default gpt-5.6-sol) read + edit/shell; document capability gaps
    - OPS-05: same process /model switch xAI → GPT (optional reverse); no restart
    - OPS-06: both spawn dirs Grok→Codex and Codex→Grok (NL or Task + model + effort)

 8. Final disposable status + cleanup
    - Final: git status --short (or fixture re-list)
    - Cleanup worktree/fixture; record in 09-UAT.md

 9. Secrets re-check before commit
    - Re-run: bash ${SCRIPT_DIR}/uat-preflight.sh
    - Or manual git ls-files + phase-diff guards from 09-UAT.md § Preflight — secrets

10. Sign-off
    - Operator, date UTC, binary/commit, models, gaps, secrets=No, disposable path, chrome outcome
    - Fixture green does not count (D-16)

Full runbook: ${UAT_MD}
EOF
echo

# Refuse writing secrets into phase dir (no auth write helpers here)
if [[ -e "${PHASE_DIR}/auth.json" ]] || [[ -e "${PHASE_DIR}/credentials.json" ]]; then
  die "credential file present under phase dir — remove immediately (never store tokens in phase tree)"
fi

echo "=== Preflight print complete ==="
echo "Secret path + phase-diff gates: PASSED (this run)."
echo "Live login / OPS matrix: operator only. This script does not mark UAT PASS."
echo "Next: open ${UAT_MD} and execute the checklist."
exit 0
