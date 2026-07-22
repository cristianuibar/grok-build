#!/usr/bin/env bash
# Phase 12 credential-free regression gate.
#
# Usage:
#   bash .planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md [MODE] [FILES...]
#
# Modes:
#   (none)                    Rust plus static phase gate
#   --rust                    Exact-discovery Rust regression matrix
#   --static                  Inventory, attribution, notice, diff, and fmt gates
#   --docs-files FILE...      Shared identity classifier for supplied files
#   --verify-scaffold         Validate this gate without running phase regressions
#   --validation-pre-final    Audit 17 green rows plus one pending final row
#   --validation-finalize-row Atomically green only row 12-07-03
#   --validation-post-row     Audit all rows green while both flags remain false
#   --validation-set-wave0    Atomically set wave_0_complete after post-row audit
#   --validation-set-nyquist  Atomically set nyquist_compliant after wave_0
#   --validation-final        Read-only final map and flag audit
#
# Latest focused Rust evidence (credential-free):
#   2026-07-21 on buffupmedia — GREEN
#   pager-p12=4, agent-p12=1, apply-patch=41, trusted-reconstruct=1,
#   trusted-wire-headers=1, trusted-header-non-leakage=1,
#   trusted-to-untrusted-switch=1. This is current Phase 12 execution evidence;
#   prior Phase 10 live dual-login evidence is context only and was not rerun.
# Latest static evidence (credential-free):
#   2026-07-21 on buffupmedia — GREEN; exact inventory 22 guides,
#   2 references, and 2 entry points; 35 committed Phase 12 paths allowed;
#   both OpenAI/Codex apply-patch notices verified unchanged; originator=bum;
#   no sampler transport, broad tool rename, competing patch tool, or derived-code
#   notice trigger. Known unrelated workspace rustfmt drift remains explicit;
#   both Phase 12 Rust files pass check-only rustfmt.

set -euo pipefail

readonly PHASE12_BASE_REF=74309d13c79ee98e0d2b0d5f58994bf3481c5ad2
readonly PHASE_DIR=.planning/phases/12-codex-depth-attribution-polish
readonly GATE_PATH="$PHASE_DIR/12-PHASE-GATE.md"
readonly VALIDATION_PATH="$PHASE_DIR/12-VALIDATION.md"
readonly DOCS_RS=crates/codegen/xai-grok-pager/src/docs.rs
readonly AUTH_DOC=crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md
readonly CODEX_AUTH=crates/codegen/xai-grok-shell/src/auth/codex/mod.rs
readonly NO_LIVE_OAUTH_OR_NETWORK=true
readonly -a GUIDE_FILES=(
  crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md
  crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md
  crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md
  crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md
  crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md
  crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md
  crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md
  crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md
  crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md
  crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md
  crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md
  crates/codegen/xai-grok-pager/docs/user-guide/12-project-rules.md
  crates/codegen/xai-grok-pager/docs/user-guide/13-memory.md
  crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md
  crates/codegen/xai-grok-pager/docs/user-guide/15-agent-mode.md
  crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md
  crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md
  crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md
  crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md
  crates/codegen/xai-grok-pager/docs/user-guide/20-background-tasks.md
  crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md
  crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md
)

readonly -a REFERENCE_FILES=(
  crates/codegen/xai-grok-pager/docs/hooks-and-plugins.md
  crates/codegen/xai-grok-pager/docs/custom-hooks.md
)

readonly -a ENTRYPOINT_FILES=(
  README.md
  crates/codegen/xai-grok-pager/docs/user-guide/README.md
)

readonly -a EXPECTED_VALIDATION_IDS=(
  12-01-01 12-01-02 12-01-03
  12-02-01 12-02-02
  12-03-01 12-03-02
  12-04-01 12-04-02
  12-05-01 12-05-02 12-05-03
  12-06-01 12-06-02 12-06-03
  12-07-01 12-07-02 12-07-03
)

fail() {
  echo "PHASE 12 GATE FAIL: $*" >&2
  exit 1
}

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

is_known_doc() {
  local candidate="$1" known
  for known in "${ENTRYPOINT_FILES[@]}" "${GUIDE_FILES[@]}" "${REFERENCE_FILES[@]}"; do
    if [[ "$candidate" == "$known" ]]; then
      return 0
    fi
  done
  return 1
}

# Every standalone Grok/Grok Build reference is denied unless its Markdown line
# establishes one of the documented allowlist contexts. This covers prose,
# headings, links, inline code, and fenced commands without guessing at verbs.
identity_classifier_node() {
  node - "$@" <<'NODE'
const fs = require("fs");

function classify(markdown) {
  const lines = markdown.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (/~\/\.grok(?:\/|[^A-Za-z0-9_-]|$)/i.test(line)) {
      return { line: index + 1, reason: "user-global stock home path" };
    }
    if (/\bbum\s+is\s+(?:the\s+)?(?:stock\s+)?(?:openai\s+)?codex cli\b/i.test(line)) {
      return { line: index + 1, reason: "stock Codex CLI impersonation claim" };
    }

    for (const occurrence of line.matchAll(/\bgrok(?: build)?\b/giu)) {
      let start = occurrence.index;
      let end = start + occurrence[0].length;
      while (start > 0 && /[A-Za-z0-9_./~:$-]/.test(line[start - 1])) start -= 1;
      while (end < line.length && /[A-Za-z0-9_./~:$-]/.test(line[end])) end += 1;
      const token = line.slice(start, end);
      const lower = token.toLowerCase();

      const hostedDomain = lower.includes("grok.com") || lower.includes("grok-proxy.");
      const internalIdentifier = token.startsWith("GROK_")
        || lower.includes("xai-grok-")
        || lower.includes("grok-hooks")
        || lower.includes("-grok-")
        || lower.includes("grok-night")
        || lower.includes("grok-day")
        || lower.includes("--grok-");
      const compatibilityPath = lower.includes(".grok/")
        || lower.startsWith("/etc/grok/")
        || (lower === ".grok" && /\b(?:project|compatibility)\b/i.test(line));
      const modelId = lower === "grok-build" || lower.startsWith("grok-") || lower.endsWith("-grok");
      const explicitModelOrProvider = [
        /(?:xai|x\.ai).{0,40}\bgrok(?: build)?\b/i,
        /\bgrok(?: build)?\b.{0,40}(?:xai|x\.ai)/i,
        /\b(?:model|models|provider|service|family)\b.{0,30}\bgrok(?: build)?\b/i,
        /\bgrok(?: build)?\s+(?:model|models|family)\b/i,
        /^\s*\/m(?:odel)?\s+grok(?: build)?\b/i,
        /^\s*name\s*=\s*"grok(?: build)?\b/i,
      ].some((pattern) => pattern.test(line));
      const lineageOrLegal = /\b(?:fork|forked|lineage|ancestry|upstream|derived|legal|notice|harness)\b/i.test(line);
      const namedInternal = /(?:\b(?:internal|identifier|item name)\b.{0,30}\bgrok\b|\bitems\s*=\s*\[[^\]]*"grok")/i.test(line);

      if (!(hostedDomain || internalIdentifier || compatibilityPath || modelId
          || explicitModelOrProvider || lineageOrLegal || namedInternal)) {
        return { line: index + 1, reason: "stock product or executable lacks an allowed context" };
      }
    }
  }
  return null;
}

const mode = process.argv[2];
if (mode === "files") {
  let failed = false;
  for (const file of process.argv.slice(3)) {
    const violation = classify(fs.readFileSync(file, "utf8"));
    if (violation) {
      console.error(`${file}:${violation.line}: ${violation.reason}`);
      failed = true;
    } else {
      console.log(`identity clean: ${file}`);
    }
  }
  process.exit(failed ? 1 : 0);
}

if (mode === "fixtures") {
  const allowed = [
    "bum supports xAI Grok models and OpenAI Codex models.",
    "## xAI/Grok provider support",
    "Select [Grok models](models.md) from the picker.",
    "```text\n/model grok-build\n```",
    "bum is built from the Grok Build harness lineage.",
    "Authenticate through https://grok.com when using xAI.",
    "The internal identifiers `GROK_HOME` and `xai-grok-shell` remain compatible.",
    "A project-local `.grok/config.toml` is supported for compatibility.",
    "The legal notice records Grok Build ancestry.",
  ];
  const forbidden = [
    "Grok delivers a terminal coding workflow.",
    "Grok offers fast edits.",
    "Grok powers the terminal.",
    "Grok helps with refactors.",
    "Grok works everywhere.",
    "# Grok: a terminal coding agent",
    "Grok—now with more tools.",
    "[Grok](https://example.com) handles the task.",
    "Use `Grok` for this task.",
    "Launch Grok to begin.",
    "Use the Grok CLI for this task.",
    "Run `grok` now.",
    "```bash\ngrok\n```",
    "```bash\n./grok login\n```",
    "Credentials are stored in ~/.grok.",
    "bum is the stock OpenAI Codex CLI.",
  ];
  for (const fixture of allowed) {
    if (classify(fixture)) throw new Error(`allowed identity category was rejected: ${JSON.stringify(fixture)}`);
  }
  for (const fixture of forbidden) {
    if (!classify(fixture)) throw new Error(`forbidden identity category was accepted: ${JSON.stringify(fixture)}`);
  }
  console.log(`identity classifier fixtures: ${allowed.length} allowed, ${forbidden.length} forbidden`);
  process.exit(0);
}

throw new Error(`unknown identity classifier mode: ${mode}`);
NODE
}

classify_identity_file() {
  local file="$1"
  test -f "$file" || fail "missing documentation file: $file"
  is_known_doc "$file" || fail "file is outside the Phase 12 documentation inventory: $file"
  rg -q -i '\bbum\b' "$file" || fail "$file does not identify bum where applicable"
  identity_classifier_node files "$file" || fail "$file contains a forbidden identity reference"
}

identity_fixture_gate() {
  identity_classifier_node fixtures || fail "identity classifier fixtures failed"
}

docs_files_gate() {
  (($# > 0)) || fail "--docs-files requires at least one file"
  local file
  for file in "$@"; do
    classify_identity_file "$file"
  done
}

inventory_gate() {
  ((${#GUIDE_FILES[@]} == 22)) || fail "guide inventory must contain exactly 22 files"
  ((${#REFERENCE_FILES[@]} == 2)) || fail "reference inventory must contain exactly two files"
  ((${#ENTRYPOINT_FILES[@]} == 2)) || fail "entry-point inventory must contain README and guide index"

  local file
  for file in "${ENTRYPOINT_FILES[@]}" "${GUIDE_FILES[@]}" "${REFERENCE_FILES[@]}"; do
    test -f "$file" || fail "inventory file missing: $file"
  done

  local -a registered_guides registered_refs expected_guides expected_refs
  mapfile -t registered_guides < <(
    sed -n '/^pub static USER_GUIDE:/,/^];/p' "$DOCS_RS" |
      rg -o '"[0-9]{2}-[^\"]+\.md"' |
      tr -d '"'
  )
  mapfile -t registered_refs < <(
    sed -n '/^static REFERENCE_DOCS:/,/^];/p' "$DOCS_RS" |
      rg -o '"(hooks-and-plugins|custom-hooks)\.md"' |
      tr -d '"'
  )
  mapfile -t expected_guides < <(printf '%s\n' "${GUIDE_FILES[@]##*/}")
  mapfile -t expected_refs < <(printf '%s\n' "${REFERENCE_FILES[@]##*/}")

  ((${#registered_guides[@]} == 22)) || fail "docs.rs must register exactly 22 guides"
  ((${#registered_refs[@]} == 2)) || fail "docs.rs must register exactly two references"
  diff -u <(printf '%s\n' "${expected_guides[@]}") <(printf '%s\n' "${registered_guides[@]}") ||
    fail "registered guide inventory differs from the gate inventory"
  diff -u <(printf '%s\n' "${expected_refs[@]}") <(printf '%s\n' "${registered_refs[@]}") ||
    fail "registered reference inventory differs from the gate inventory"

  echo "inventory exact: 22 guides, 2 references, 2 entry points"
}

markdown_link_gate() {
  node - "${ENTRYPOINT_FILES[@]}" "${GUIDE_FILES[@]}" "${REFERENCE_FILES[@]}" <<'NODE'
const fs = require("fs");
const path = require("path");

const files = process.argv.slice(2);
let checked = 0;
const failures = [];

for (const file of files) {
  const markdown = fs.readFileSync(file, "utf8");
  const links = markdown.matchAll(/(?<!!)\[[^\]]*\]\(([^)]+)\)/g);
  for (const match of links) {
    let destination = match[1].trim();
    if (destination.startsWith("<") && destination.endsWith(">")) {
      destination = destination.slice(1, -1);
    }
    destination = destination.split(/\s+['\"]/)[0];
    if (/^(?:https?:|mailto:|data:)/i.test(destination) || destination.startsWith("#")) {
      continue;
    }

    const pathname = destination.split("#", 1)[0].split("?", 1)[0];
    if (!pathname) continue;
    checked += 1;
    const resolved = path.resolve(path.dirname(file), decodeURIComponent(pathname));
    if (!fs.existsSync(resolved)) {
      failures.push(`${file}: ${destination} -> ${resolved}`);
    }
  }
}

if (checked === 0) {
  console.error("no local Markdown links were checked");
  process.exit(1);
}
if (failures.length > 0) {
  console.error(`broken local Markdown links:\n${failures.join("\n")}`);
  process.exit(1);
}
console.log(`local Markdown links resolve: ${checked} checked across ${files.length} files`);
NODE
}

# discover_exact LABEL PACKAGE EXPECTED LIST_REGEX TEST_FILTER [CARGO TARGET ARGS...]
discover_exact() {
  local label="$1" package="$2" expected="$3" list_regex="$4" test_filter="$5"
  shift 5
  local -a target_args=("$@")
  local listing count
  listing=$(cargo test -p "$package" "${target_args[@]}" -- --list)
  count=$(awk -F': ' -v pattern="$list_regex" \
    '$1 ~ pattern && $2 == "test" { n++ } END { print n + 0 }' <<< "$listing")
  echo "discover_exact $label: expected=$expected actual=$count"
  [[ "$count" == "$expected" ]] || fail "$label discovery count drifted"
  cargo test -p "$package" "${target_args[@]}" "$test_filter" -- --nocapture
}

rust_gate() {
  [[ "$NO_LIVE_OAUTH_OR_NETWORK" == true ]] || fail "Rust gate must remain credential-free"

  discover_exact pager-p12 xai-grok-pager 4 \
    '::p12_(embedded_docs_use_bum_product_identity|identity_classifier_adversarial_contract|capability_disclosure_is_embedded_and_complete|authentication_documents_exact_provider_commands)$' \
    p12_ --lib
  discover_exact agent-p12 xai-grok-agent 1 \
    '::p12_codex_toolset_identity$' p12_codex_toolset_identity --lib
  discover_exact apply-patch xai-grok-tools 41 \
    'apply_patch' apply_patch --lib
  discover_exact trusted-reconstruct xai-grok-shell 1 \
    '::trusted_codex_reconstruct_enables_profile_and_metadata$' \
    trusted_codex_reconstruct_enables_profile_and_metadata --lib
  discover_exact trusted-wire-headers xai-grok-shell 1 \
    '^trusted_codex_wire_headers_are_sent_and_stable$' \
    trusted_codex_wire_headers_are_sent_and_stable --test model_switch_gate
  discover_exact trusted-header-non-leakage xai-grok-shell 1 \
    '^codex_wire_headers_do_not_leak_to_xai_byok_or_custom$' \
    codex_wire_headers_do_not_leak_to_xai_byok_or_custom --test model_switch_gate
  discover_exact trusted-to-untrusted-switch xai-grok-shell 1 \
    '^trusted_to_untrusted_switch_strips_codex_identity_headers$' \
    trusted_to_untrusted_switch_strips_codex_identity_headers --test model_switch_gate

  echo "PHASE 12 RUST GATE: GREEN (local fixtures only; no OAuth or provider network)"
}

capability_gate() {
  rg -F -q 'bum uses ChatGPT/Codex OAuth and compatible model APIs, but it is not the stock OpenAI Codex CLI.' README.md ||
    fail "README non-stock Codex disclaimer missing"
  rg -F -q '[provider capability contract](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md#provider-capability-contract)' README.md ||
    fail "README capability-contract link missing"
  rg -F -q 'THIRD-PARTY-NOTICES' README.md || fail "root notice link missing from README"
  rg -F -q 'crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md' README.md ||
    fail "crate notice link missing from README"

  rg -F -q '# bum' crates/codegen/xai-grok-pager/docs/user-guide/README.md ||
    fail "guide index bum heading missing"
  rg -F -q '02-authentication.md' crates/codegen/xai-grok-pager/docs/user-guide/README.md ||
    fail "guide index authentication link missing"
  rg -F -q 'Provider capability contract' crates/codegen/xai-grok-pager/docs/user-guide/README.md ||
    fail "guide index capability description missing"
  rg -F -q 'bum -p' crates/codegen/xai-grok-pager/docs/user-guide/README.md ||
    fail "guide index headless bum example missing"

  local row
  for row in \
    "| Transport | Uses bum's provider-routed HTTP path. | Responses are sent with HTTP POST and streamed with SSE. | Responses WebSocket incremental transport is deferred and non-blocking; the supported daily-driver path remains HTTP/SSE. |" \
    '| Request identity metadata | xAI routes do not receive reserved Codex identity metadata. | A trusted Codex OAuth route receives bum-owned `originator` and session metadata. | BYOK and custom endpoints do not inherit that metadata; bum never impersonates the stock Codex CLI. |' \
    "| Patch shape | Uses the edit tools configured for the active bum preset. | \`codex:apply_patch\` provides behavioral patch compatibility through a JSON \`{ patch }\` field. | It is not a promise of exact stock wire format, tool naming, or stronger path containment than bum's existing permission and sandbox checks prove. |" \
    '| Deferred parity | No change to the supported xAI path. | Broad stock Codex tool-name parity is deferred and non-blocking. | Deferred parity does not block productive shell, read, search, or edit workflows. |'; do
    rg -F -q "$row" "$AUTH_DOC" || fail "Authentication capability row drifted: $row"
  done

  local forbidden_claim
  for forbidden_claim in \
    'Responses WebSocket incremental transport is supported' \
    'bum is the stock OpenAI Codex CLI' \
    'uses the stock Codex CLI originator' \
    'provides exact stock tool parity' \
    'provides exact stock wire format'; do
    ! rg -F -q "$forbidden_claim" "$AUTH_DOC" ||
      fail "Authentication guide contains forbidden capability claim: $forbidden_claim"
  done
  echo "capability disclosure rows complete and internally consistent"
}

notice_and_originator_gate() {
  rg -F -q 'openai/codex' THIRD-PARTY-NOTICES || fail "root OpenAI/Codex notice missing"
  rg -F -q 'apply_patch' THIRD-PARTY-NOTICES || fail "root apply_patch notice missing"
  rg -F -q '### openai/codex' crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md ||
    fail "crate OpenAI/Codex notice missing"
  rg -F -q '`apply_patch`' crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md ||
    fail "crate apply_patch notice missing"

  local originator_count
  originator_count=$(rg -c '^pub const CODEX_ORIGINATOR: &str = "bum";$' "$CODEX_AUTH")
  [[ "$originator_count" == 1 ]] || fail "CODEX_ORIGINATOR must be exactly bum once"
  ! rg -n 'CODEX_ORIGINATOR:.*codex_cli_rs|originator.*codex_cli_rs' "$CODEX_AUTH" ||
    fail "stock Codex originator identity detected"

  echo "notices verified unchanged in meaning; CODEX_ORIGINATOR=bum"
}

is_allowed_phase_diff_file() {
  local file="$1"
  if is_known_doc "$file"; then
    return 0
  fi
  case "$file" in
    crates/codegen/xai-grok-pager/src/docs.rs | \
    crates/codegen/xai-grok-agent/src/config.rs | \
    .planning/STATE.md | \
    .planning/ROADMAP.md | \
    .planning/REQUIREMENTS.md | \
    "$PHASE_DIR"/12-CONTEXT.md | \
    "$PHASE_DIR"/12-RESEARCH.md | \
    "$PHASE_DIR"/12-PATTERNS.md | \
    "$PHASE_DIR"/12-VALIDATION.md | \
    "$PHASE_DIR"/12-PHASE-GATE.md | \
    "$PHASE_DIR"/12-0[1-7]-PLAN.md | \
    "$PHASE_DIR"/12-0[1-7]-SUMMARY.md)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

committed_diff_gate() {
  git cat-file -e "$PHASE12_BASE_REF^{commit}" 2>/dev/null || fail "pinned Phase 12 base is missing"
  git merge-base --is-ancestor "$PHASE12_BASE_REF" HEAD || fail "pinned Phase 12 base is not an ancestor of HEAD"

  local -a changed_files
  mapfile -t changed_files < <(git diff --name-only "$PHASE12_BASE_REF"..HEAD)
  ((${#changed_files[@]} > 0)) || fail "committed Phase 12 diff is empty"

  local file
  for file in "${changed_files[@]}"; do
    is_allowed_phase_diff_file "$file" || fail "unplanned file in committed Phase 12 diff: $file"
  done

  local committed_diff
  # The gate contains the forbidden-pattern expressions as data, so exclude only
  # this script from content scanning after its path has passed the allowlist.
  committed_diff=$(git diff --unified=0 "$PHASE12_BASE_REF"..HEAD -- . ":(exclude)$GATE_PATH")
  if rg -n '^\+.*(tokio[_-]tungstenite|responses[_-]websocket|supports[_-]websockets?|OpenAI-Beta)' <<< "$committed_diff"; then
    fail "deferred WebSocket implementation or capability flag found in phase diff"
  fi
  if rg -n '^\+.*originator.*codex_cli_rs' <<< "$committed_diff"; then
    fail "stock Codex originator identity found in phase diff"
  fi
  if rg -n '^\+.*\.with_name\("(apply_patch|exec_command|write_stdin)"\)' <<< "$committed_diff"; then
    fail "broad Codex tool rename found in phase diff"
  fi
  if printf '%s\n' "${changed_files[@]}" | rg -q '^crates/codegen/xai-grok-tools/src/implementations/codex/'; then
    fail "new copied/derived Codex implementation requires explicit notice-scoped review"
  fi
  if printf '%s\n' "${changed_files[@]}" | rg -q '(^|/)THIRD[_-]PARTY[_-]NOTICES'; then
    fail "notice files are verify-only unless a reviewed derived-code trigger exists"
  fi

  echo "committed diff scoped from $PHASE12_BASE_REF through HEAD"
}

format_gate() {
  local fmt_log
  fmt_log=$(mktemp)
  if cargo fmt --all -- --check >"$fmt_log" 2>&1; then
    rm -f "$fmt_log"
    echo "workspace formatting check: GREEN"
    return
  fi

  echo "workspace formatting check: known unrelated drift remains; first affected paths:"
  rg '^Diff in ' "$fmt_log" | sed -n '1,5p'
  rm -f "$fmt_log"
  rustfmt --edition 2024 --check \
    crates/codegen/xai-grok-pager/src/docs.rs \
    crates/codegen/xai-grok-agent/src/config.rs
  echo "Phase 12 Rust formatting check: GREEN (unrelated workspace drift not reformatted)"
}

static_gate() {
  [[ "$NO_LIVE_OAUTH_OR_NETWORK" == true ]] || fail "static gate must remain network-free"
  inventory_gate
  markdown_link_gate
  identity_fixture_gate
  docs_files_gate "${ENTRYPOINT_FILES[@]}" "${GUIDE_FILES[@]}" "${REFERENCE_FILES[@]}"
  capability_gate
  notice_and_originator_gate
  committed_diff_gate
  format_gate
  echo "PHASE 12 STATIC GATE: GREEN (no live OAuth or provider network)"
}

validation_rows() {
  awk -F'|' '
    $2 ~ /^ 12-[0-9][0-9]-[0-9][0-9] / {
      for (i = 2; i <= 13; i++) {
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", $i)
      }
      print $2 "\t" $3 "\t" $4 "\t" $11 "\t" $12 "\t" $13
    }
  ' "$VALIDATION_PATH"
}

flag_count() {
  local key="$1" value="$2" count
  count=$(rg -c "^${key}: ${value}$" "$VALIDATION_PATH" || true)
  echo "${count:-0}"
}

assert_flags() {
  local wave_value="$1" nyquist_value="$2"
  [[ "$(flag_count wave_0_complete "$wave_value")" == 1 ]] ||
    fail "wave_0_complete must occur exactly once as $wave_value"
  [[ "$(flag_count nyquist_compliant "$nyquist_value")" == 1 ]] ||
    fail "nyquist_compliant must occur exactly once as $nyquist_value"
  [[ "$(rg -c '^wave_0_complete:' "$VALIDATION_PATH")" == 1 ]] || fail "duplicate wave_0_complete flags"
  [[ "$(rg -c '^nyquist_compliant:' "$VALIDATION_PATH")" == 1 ]] || fail "duplicate nyquist_compliant flags"
}

validate_row_map() {
  local rows count unique_count id plan wave expected_plan expected_wave occurrences
  rows=$(validation_rows)
  count=$(wc -l <<< "$rows" | tr -d ' ')
  unique_count=$(cut -f1 <<< "$rows" | sort -u | wc -l | tr -d ' ')
  [[ "$count" == 18 && "$unique_count" == 18 ]] || fail "validation map must contain 18 unique rows"

  for id in "${EXPECTED_VALIDATION_IDS[@]}"; do
    occurrences=$(awk -F'\t' -v wanted="$id" '$1 == wanted { n++ } END { print n + 0 }' <<< "$rows")
    [[ "$occurrences" == 1 ]] || fail "validation row $id must occur exactly once"
  done

  while IFS=$'\t' read -r id plan wave _; do
    expected_plan="${id%-*}"
    case "$expected_plan" in
      12-01) expected_wave=1 ;;
      12-02 | 12-03 | 12-04 | 12-05 | 12-06) expected_wave=2 ;;
      12-07) expected_wave=3 ;;
      *) fail "unexpected validation owner: $expected_plan" ;;
    esac
    [[ "$plan" == "$expected_plan" && "$wave" == "$expected_wave" ]] ||
      fail "$id has incorrect plan/wave ownership: $plan/$wave"
  done <<< "$rows"
}

assert_green_row() {
  local id="$1" fields exists evidence status
  fields=$(validation_rows | awk -F'\t' -v wanted="$id" '$1 == wanted { print $4 "\t" $5 "\t" $6 }')
  IFS=$'\t' read -r exists evidence status <<< "$fields"
  [[ "$exists" == "✅" ]] || fail "$id artifact is not checked"
  [[ "$evidence" == PASS\ * ]] || fail "$id lacks dated PASS evidence"
  [[ "$status" == "✅ green" ]] || fail "$id is not green"
}

validation_pre_final() {
  validate_row_map
  local id
  for id in "${EXPECTED_VALIDATION_IDS[@]}"; do
    [[ "$id" == 12-07-03 ]] && continue
    assert_green_row "$id"
  done

  local fields exists evidence status
  fields=$(validation_rows | awk -F'\t' '$1 == "12-07-03" { print $4 "\t" $5 "\t" $6 }')
  IFS=$'\t' read -r exists evidence status <<< "$fields"
  [[ "$exists" == "⬜ pending" && "$evidence" == pending && "$status" == "⬜ pending" ]] ||
    fail "12-07-03 must be the sole pending final row"
  assert_flags false false
  echo "validation pre-final audit: 17 green, 12-07-03 pending, flags false"
}

atomic_rewrite_validation() {
  local tmp
  tmp=$(mktemp "$PHASE_DIR/.12-validation.XXXXXX")
  chmod --reference="$VALIDATION_PATH" "$tmp"
  awk "$@" "$VALIDATION_PATH" > "$tmp"
  cmp -s "$VALIDATION_PATH" "$tmp" && {
    rm -f "$tmp"
    fail "validation transition produced no change"
  }
  mv "$tmp" "$VALIDATION_PATH"
}

validation_finalize_row() {
  validation_pre_final
  local today evidence
  today=$(date -u +%Y-%m-%d)
  evidence="PASS $today: bash $GATE_PATH --validation-pre-final"
  atomic_rewrite_validation -F'|' -v evidence="$evidence" '
    /^\| 12-07-03 \|/ {
      $11 = " ✅ "
      $12 = " " evidence " "
      $13 = " ✅ green "
      print
      next
    }
    { print }
  ' OFS='|'
  echo "validation row 12-07-03 finalized atomically"
}

validation_post_row() {
  validate_row_map
  local id
  for id in "${EXPECTED_VALIDATION_IDS[@]}"; do
    assert_green_row "$id"
  done
  if validation_rows | rg -n 'pending|❌ red|⚠️ flaky'; then
    fail "pending, red, or flaky validation rows remain"
  fi
  assert_flags false false
  echo "validation post-row audit: 18 green, flags false"
}

validation_set_wave0() {
  validation_post_row
  [[ "$(flag_count wave_0_complete false)" == 1 && "$(flag_count wave_0_complete true)" == 0 ]] ||
    fail "wave_0_complete transition is not in its exact initial state"
  [[ "$(flag_count nyquist_compliant false)" == 1 && "$(flag_count nyquist_compliant true)" == 0 ]] ||
    fail "nyquist_compliant must remain false before wave_0 transition"
  atomic_rewrite_validation '
    /^wave_0_complete: false$/ { print "wave_0_complete: true"; next }
    { print }
  '
  assert_flags true false
  echo "wave_0_complete set true atomically"
}

validation_set_nyquist() {
  validate_row_map
  local id
  for id in "${EXPECTED_VALIDATION_IDS[@]}"; do
    assert_green_row "$id"
  done
  [[ "$(flag_count wave_0_complete true)" == 1 && "$(flag_count wave_0_complete false)" == 0 ]] ||
    fail "wave_0_complete must be true before Nyquist transition"
  [[ "$(flag_count nyquist_compliant false)" == 1 && "$(flag_count nyquist_compliant true)" == 0 ]] ||
    fail "Nyquist transition is not in its exact initial state"
  atomic_rewrite_validation '
    /^nyquist_compliant: false$/ { print "nyquist_compliant: true"; next }
    { print }
  '
  assert_flags true true
  echo "nyquist_compliant set true atomically"
}

validation_final() {
  validate_row_map
  local id
  for id in "${EXPECTED_VALIDATION_IDS[@]}"; do
    assert_green_row "$id"
  done
  if validation_rows | rg -n 'pending|❌ red|⚠️ flaky'; then
    fail "final validation contains pending, red, or flaky rows"
  fi
  assert_flags true true
  echo "validation final audit: 18 green, flags true exactly once"
}

verify_scaffold() {
  test -x "$GATE_PATH" || fail "gate is not executable"
  bash -n "$GATE_PATH"
  inventory_gate
  ((${#EXPECTED_VALIDATION_IDS[@]} == 18)) || fail "expected validation ID inventory drifted"
  [[ "$NO_LIVE_OAUTH_OR_NETWORK" == true ]] || fail "no-network invariant is disabled"
  rg -F -q 'discover_exact()' "$GATE_PATH" || fail "discover_exact helper missing"
  rg -F -q 'markdown_link_gate()' "$GATE_PATH" || fail "Markdown link gate missing"
  rg -F -q 'Allowed contextual categories are explicit' "$GATE_PATH" ||
    fail "identity allowlist categories missing"

  local mode
  for mode in \
    --rust \
    --static \
    --docs-files \
    --verify-scaffold \
    --validation-pre-final \
    --validation-finalize-row \
    --validation-post-row \
    --validation-set-wave0 \
    --validation-set-nyquist \
    --validation-final; do
    rg -F -q -- "$mode)" "$GATE_PATH" || fail "dispatch label missing: $mode"
  done

  if rg -n '^[[:space:]]*(curl|wget|gh|glab)[[:space:]]' "$GATE_PATH"; then
    fail "network-capable command found in gate"
  fi
  echo "PHASE 12 GATE SCAFFOLD: VERIFIED (exact inventories; no network)"
}

main() {
  local mode="${1:-}"
  case "$mode" in
    "")
      rust_gate
      static_gate
      ;;
    --rust)
      shift
      (($# == 0)) || fail "--rust accepts no additional arguments"
      rust_gate
      ;;
    --static)
      shift
      (($# == 0)) || fail "--static accepts no additional arguments"
      static_gate
      ;;
    --docs-files)
      shift
      docs_files_gate "$@"
      ;;
    --verify-scaffold)
      shift
      (($# == 0)) || fail "--verify-scaffold accepts no additional arguments"
      verify_scaffold
      ;;
    --validation-pre-final)
      shift
      (($# == 0)) || fail "--validation-pre-final accepts no additional arguments"
      validation_pre_final
      ;;
    --validation-finalize-row)
      shift
      (($# == 0)) || fail "--validation-finalize-row accepts no additional arguments"
      validation_finalize_row
      ;;
    --validation-post-row)
      shift
      (($# == 0)) || fail "--validation-post-row accepts no additional arguments"
      validation_post_row
      ;;
    --validation-set-wave0)
      shift
      (($# == 0)) || fail "--validation-set-wave0 accepts no additional arguments"
      validation_set_wave0
      ;;
    --validation-set-nyquist)
      shift
      (($# == 0)) || fail "--validation-set-nyquist accepts no additional arguments"
      validation_set_nyquist
      ;;
    --validation-final)
      shift
      (($# == 0)) || fail "--validation-final accepts no additional arguments"
      validation_final
      ;;
    *)
      fail "unknown mode: $mode"
      ;;
  esac
}

main "$@"
