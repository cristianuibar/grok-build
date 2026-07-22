//! In-app how-to documentation data (embedded markdown).
//!
//! Single source of truth: two static arrays (`USER_GUIDE`, `REFERENCE_DOCS`)
//! hold every doc. All lookups are zero-allocation; `DocEntry` exists only for
//! backward compatibility with the TUI doc picker.

/// A compile-time document entry. All fields are `&'static str`.
#[derive(Debug)]
pub struct Doc {
    pub filename: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

/// Owned variant for the TUI doc picker (backward compat).
#[derive(Debug, Clone)]
pub struct DocEntry {
    pub title: String,
    pub description: String,
    /// Embedded markdown content.
    pub content: &'static str,
}

impl From<&Doc> for DocEntry {
    fn from(d: &Doc) -> Self {
        Self {
            title: d.title.into(),
            description: d.description.into(),
            content: d.content,
        }
    }
}

// ── Static doc tables ────────────────────────────────────────────────────────

macro_rules! guide {
    ($file:literal, $title:literal, $desc:literal) => {
        Doc {
            filename: $file,
            title: $title,
            description: $desc,
            content: include_str!(concat!("../docs/user-guide/", $file)),
        }
    };
}

pub static USER_GUIDE: &[Doc] = &[
    guide!(
        "01-getting-started.md",
        "Getting Started",
        "Installation, first launch, and basic interaction"
    ),
    guide!(
        "02-authentication.md",
        "Authentication",
        "Browser login, API keys, OIDC, external auth providers"
    ),
    guide!(
        "03-keyboard-shortcuts.md",
        "Keyboard Shortcuts",
        "Complete reference for all TUI key bindings"
    ),
    guide!(
        "04-slash-commands.md",
        "Slash Commands",
        "All / commands for session management, models, memory, hooks"
    ),
    guide!(
        "05-configuration.md",
        "Configuration",
        "config.toml, pager.toml, environment variables, file locations"
    ),
    guide!(
        "06-theming.md",
        "Theming and Appearance",
        "Themes, color support, pager.toml customization"
    ),
    guide!(
        "07-mcp-servers.md",
        "MCP Servers",
        "Setting up external tool integrations via MCP"
    ),
    guide!(
        "08-skills.md",
        "Skills",
        "Creating and using reusable prompt packages"
    ),
    guide!(
        "09-plugins.md",
        "Plugins and Marketplace",
        "Installing, managing, and creating plugin packages"
    ),
    guide!(
        "10-hooks.md",
        "Hooks",
        "Project lifecycle scripts for pre/post tool-use events"
    ),
    guide!(
        "11-custom-models.md",
        "Custom Models",
        "BYOK, Ollama, OpenAI-compatible endpoints"
    ),
    guide!(
        "12-project-rules.md",
        "Project Rules (AGENTS.md)",
        "Per-directory instructions and precedence rules"
    ),
    guide!(
        "13-memory.md",
        "Memory",
        "Cross-session knowledge persistence and search"
    ),
    guide!(
        "14-headless-mode.md",
        "Headless Mode and Scripting",
        "Non-interactive CLI for automation and CI/CD"
    ),
    guide!(
        "15-agent-mode.md",
        "Agent Mode and IDE Integration",
        "ACP stdio transport, WebSocket relay, SDK integration"
    ),
    guide!(
        "16-subagents.md",
        "Subagents and Personas",
        "Spawning parallel child agents with specialized roles"
    ),
    guide!(
        "17-sessions.md",
        "Session Management",
        "Save, load, resume, rewind, and compact sessions"
    ),
    guide!(
        "18-sandbox.md",
        "Sandbox Mode",
        "OS-level filesystem and network isolation"
    ),
    guide!(
        "19-plan-mode.md",
        "Plan Mode",
        "Structured planning with approval dialogs"
    ),
    guide!(
        "20-background-tasks.md",
        "Background Tasks and Monitoring",
        "Background commands, /loop, monitor, scheduler"
    ),
    guide!(
        "21-terminal-support.md",
        "Terminal Support and Troubleshooting",
        "tmux, Byobu, Zellij, SSH, truecolor, clipboard, and diagnostics"
    ),
    guide!(
        "22-permissions-and-safety.md",
        "Permissions and Safety",
        "Tool approval, sandbox, security"
    ),
];

/// Non-user-guide reference docs. Separate from USER_GUIDE because they
/// live under `docs/` (not `docs/user-guide/`), are not extracted to disk,
/// and do not follow the NN-*.md managed naming pattern. Bundled via
/// `include_str!` so they are available at runtime without a docs path.
static REFERENCE_DOCS: &[Doc] = &[
    Doc {
        filename: "hooks-and-plugins.md",
        title: "Hooks & Plugins Guide",
        description: "Using hooks, plugins, and marketplace",
        content: include_str!("../docs/hooks-and-plugins.md"),
    },
    Doc {
        filename: "custom-hooks.md",
        title: "Creating Custom Hooks",
        description: "Writing your own hooks and matchers",
        content: include_str!("../docs/custom-hooks.md"),
    },
];

// ── Public API ───────────────────────────────────────────────────────────────

/// Find a doc by title (case-insensitive). Returns the static entry.
pub fn find_doc(title: &str) -> Option<&'static Doc> {
    USER_GUIDE
        .iter()
        .chain(REFERENCE_DOCS.iter())
        .find(|d| d.title.eq_ignore_ascii_case(title))
}

/// All doc titles, zero allocation.
pub fn all_titles() -> impl Iterator<Item = &'static str> {
    USER_GUIDE
        .iter()
        .chain(REFERENCE_DOCS.iter())
        .map(|d| d.title)
}

/// Returns the content of a how-to document by exact title match (case-insensitive).
pub fn get_howto_doc(title: &str) -> Option<&'static str> {
    find_doc(title).map(|d| d.content)
}

/// Returns a list of available how-to titles for the model to choose from.
pub fn list_howto_titles() -> Vec<String> {
    all_titles().map(String::from).collect()
}

/// Returns all docs as owned `DocEntry` values for the TUI doc picker.
pub fn default_howto_entries() -> Vec<DocEntry> {
    USER_GUIDE
        .iter()
        .chain(REFERENCE_DOCS.iter())
        .map(DocEntry::from)
        .collect()
}

/// Extract user-guide docs to `<grok_home>/docs/user-guide/`.
///
/// Called from the pager binary startup so the model can read them from disk.
pub fn extract_user_guide_docs(grok_home: &std::path::Path) {
    let docs_dir = grok_home.join("docs").join("user-guide");
    if let Err(e) = std::fs::create_dir_all(&docs_dir) {
        tracing::warn!(error = %e, "Failed to create user-guide docs directory");
        return;
    }
    for doc in USER_GUIDE {
        if let Err(e) = std::fs::write(docs_dir.join(doc.filename), doc.content) {
            tracing::debug!(error = %e, filename = doc.filename, "Failed to extract user-guide doc");
        }
    }
    // Clean up stale managed docs (files removed from USER_GUIDE since last run).
    // Only remove files matching the managed naming pattern (NN-*.md).
    if let Ok(entries) = std::fs::read_dir(&docs_dir) {
        let valid: std::collections::HashSet<&str> =
            USER_GUIDE.iter().map(|d| d.filename).collect();
        for dir_entry in entries.flatten() {
            if let Some(name) = dir_entry.file_name().to_str() {
                let is_managed = name.len() > 3
                    && name.as_bytes()[0].is_ascii_digit()
                    && name.as_bytes()[1].is_ascii_digit()
                    && name.as_bytes()[2] == b'-'
                    && name.ends_with(".md");
                if is_managed
                    && !valid.contains(name)
                    && let Err(e) = std::fs::remove_file(dir_entry.path())
                {
                    tracing::debug!(error = %e, filename = name, "Failed to remove stale user-guide doc");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare_markdown_command_lines<'a>(markdown: &'a str, command: &str) -> Vec<(usize, &'a str)> {
        let inline = regex::Regex::new(&format!(r"`\s*{}\s*`", regex::escape(command)))
            .expect("valid inline command regex");
        let fenced = regex::Regex::new(&format!(r"^\s*(?:\$\s*)?{}\s*$", regex::escape(command)))
            .expect("valid fenced command regex");
        let mut in_fence = false;

        markdown
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                if line.trim_start().starts_with("```") {
                    in_fence = !in_fence;
                    return None;
                }
                (inline.is_match(line) || (in_fence && fenced.is_match(line)))
                    .then_some((index + 1, line))
            })
            .collect()
    }

    fn identity_violation(markdown: &str) -> Option<&'static str> {
        let stock_subject = regex::Regex::new(
            r"(?i)\b(?:grok build|grok)\s+(?:is|supports|uses|stores|opens|automatically|prompts|provides|includes|runs|will|can)\b",
        )
        .expect("valid stock-product regex");
        let imperative_stock_name = regex::Regex::new(
            r"(?i)\b(?:launch|invoke|run|start)\s+(?:the\s+)?(?:`|\./)?grok(?:`|\s|$)",
        )
        .expect("valid stock-command regex");
        let inline_stock_executable =
            regex::Regex::new(r"(?i)`(?:\./)?grok(?:`|\s)").expect("valid inline-command regex");
        let stock_cli_name =
            regex::Regex::new(r"(?i)\b(?:the\s+)?grok cli\b").expect("valid stock-CLI regex");
        let stock_home = regex::Regex::new(r"(?i)~/\.grok(?:/|[^[:alnum:]_-]|$)")
            .expect("valid stock-home regex");
        let stock_codex_identity = regex::Regex::new(
            r"(?i)\bbum\s+is\s+(?:the\s+)?(?:stock\s+)?(?:openai\s+)?codex cli\b",
        )
        .expect("valid Codex-identity regex");
        let fenced_stock_command = regex::Regex::new(
            r"(?i)^\s*(?:\$\s*)?(?:\./)?grok(?:\s+(?:-{1,2}[[:alnum:]-]+|login|logout|auth|agent|mcp|plugin|sessions?|inspect|update|workspace|models?))?\s*$",
        )
        .expect("valid fenced-command regex");

        if stock_subject.is_match(markdown) {
            return Some("stock product used as the acting subject");
        }
        if imperative_stock_name.is_match(markdown)
            || inline_stock_executable.is_match(markdown)
            || stock_cli_name.is_match(markdown)
        {
            return Some("stock executable used in command position");
        }
        if stock_home.is_match(markdown) {
            return Some("user-global stock home path");
        }
        if stock_codex_identity.is_match(markdown) {
            return Some("stock Codex CLI impersonation claim");
        }

        let mut in_fence = false;
        for line in markdown.lines() {
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence && fenced_stock_command.is_match(line) {
                return Some("stock executable used in a command block");
            }
        }
        None
    }

    #[test]
    fn user_guide_entries_are_valid() {
        for doc in USER_GUIDE {
            assert!(!doc.content.is_empty(), "Doc {} is empty", doc.filename);
            assert!(
                !doc.title.is_empty(),
                "Doc {} has empty title",
                doc.filename
            );
            assert!(
                !doc.description.is_empty(),
                "Doc {} has empty description",
                doc.filename
            );
            assert!(
                doc.content.starts_with('#'),
                "Doc {} should start with a markdown header",
                doc.filename
            );
        }
    }

    #[test]
    fn user_guide_entries_have_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for doc in USER_GUIDE {
            assert!(
                seen.insert(doc.filename),
                "Duplicate doc in list: {}",
                doc.filename
            );
        }
    }

    #[test]
    fn default_howto_entries_includes_all_user_guide_docs() {
        let entries = default_howto_entries();
        assert_eq!(entries.len(), USER_GUIDE.len() + REFERENCE_DOCS.len());
        for (i, doc) in USER_GUIDE.iter().enumerate() {
            assert_eq!(entries[i].title, doc.title, "Entry {} title mismatch", i);
        }
    }

    #[test]
    fn find_doc_is_case_insensitive() {
        let doc = find_doc("getting started").expect("should find Getting Started");
        assert_eq!(doc.title, "Getting Started");
        assert!(find_doc("nonexistent guide").is_none());
    }

    #[test]
    fn all_titles_covers_both_tables() {
        let titles: Vec<_> = all_titles().collect();
        assert_eq!(titles.len(), USER_GUIDE.len() + REFERENCE_DOCS.len());
    }

    #[test]
    fn get_howto_doc_delegates_to_find_doc() {
        assert!(get_howto_doc("Getting Started").is_some());
        assert!(get_howto_doc("Hooks & Plugins Guide").is_some());
        assert!(get_howto_doc("no such doc").is_none());
    }

    #[test]
    fn p12_embedded_docs_use_bum_product_identity() {
        // These are semantic categories, not a blanket ban on provider words.
        // Provider/model brands (Grok, xAI, Codex, OpenAI), fork lineage, legal
        // notices, internal GROK_*/xai-grok-* identifiers, hosted domains such
        // as grok.com, and project-local .grok/ compatibility paths are valid.
        assert_eq!(
            USER_GUIDE.len(),
            22,
            "registered user-guide inventory drift"
        );
        assert_eq!(
            REFERENCE_DOCS.len(),
            2,
            "registered reference inventory drift"
        );

        for doc in USER_GUIDE.iter().chain(REFERENCE_DOCS.iter()) {
            let lower = doc.content.to_ascii_lowercase();
            assert!(
                lower.contains("bum"),
                "{} must identify the bum product or executable",
                doc.filename
            );

            assert_eq!(
                identity_violation(doc.content),
                None,
                "{} contains a stale stock-product identity claim",
                doc.filename
            );
        }
    }

    #[test]
    fn p12_identity_classifier_adversarial_contract() {
        let allowed = [
            "bum supports xAI Grok models and OpenAI Codex models.",
            "bum is built from the Grok Build harness lineage.",
            "Authenticate through https://grok.com when using xAI.",
            "The internal identifiers `GROK_HOME` and `xai-grok-shell` remain compatible.",
            "A project-local `.grok/config.toml` is supported for compatibility.",
            "The legal notice records Grok Build ancestry.",
        ];
        let forbidden = [
            "Grok provides a terminal coding workflow.",
            "Launch Grok to begin.",
            "Use the Grok CLI for this task.",
            "Run `grok` now.",
            "```bash\ngrok\n```",
            "```bash\n./grok login\n```",
            "Credentials are stored in ~/.grok.",
            "bum is the stock OpenAI Codex CLI.",
        ];

        for fixture in allowed {
            assert_eq!(
                identity_violation(fixture),
                None,
                "allowed identity category was rejected: {fixture:?}"
            );
        }
        for fixture in forbidden {
            assert!(
                identity_violation(fixture).is_some(),
                "forbidden identity category was accepted: {fixture:?}"
            );
        }
    }

    #[test]
    fn p12_capability_disclosure_is_embedded_and_complete() {
        let auth = get_howto_doc("Authentication").expect("embedded Authentication guide");
        let row = |area: &str| {
            auth.lines()
                .find(|line| line.starts_with(&format!("| {area} |")))
                .unwrap_or_else(|| panic!("Authentication capability table is missing {area:?}"))
        };

        assert_eq!(
            row("Transport"),
            "| Transport | Uses bum's provider-routed HTTP path. | Responses are sent with HTTP POST and streamed with SSE. | Responses WebSocket incremental transport is deferred and non-blocking; the supported daily-driver path remains HTTP/SSE. |"
        );
        assert_eq!(
            row("Request identity metadata"),
            "| Request identity metadata | xAI routes do not receive reserved Codex identity metadata. | A trusted Codex OAuth route receives bum-owned `originator` and session metadata. | BYOK and custom endpoints do not inherit that metadata; bum never impersonates the stock Codex CLI. |"
        );
        assert_eq!(
            row("Patch shape"),
            "| Patch shape | Uses the edit tools configured for the active bum preset. | `codex:apply_patch` provides behavioral patch compatibility through a JSON `{ patch }` field. | It is not a promise of exact stock wire format, tool naming, or stronger path containment than bum's existing permission and sandbox checks prove. |"
        );
        assert_eq!(
            row("Deferred parity"),
            "| Deferred parity | No change to the supported xAI path. | Broad stock Codex tool-name parity is deferred and non-blocking. | Deferred parity does not block productive shell, read, search, or edit workflows. |"
        );

        for forbidden_claim in [
            "Responses WebSocket incremental transport is supported",
            "bum is the stock OpenAI Codex CLI",
            "uses the stock Codex CLI originator",
            "provides exact stock tool parity",
            "provides exact stock wire format",
        ] {
            assert!(
                !auth.contains(forbidden_claim),
                "Authentication guide contains forbidden capability claim {forbidden_claim:?}"
            );
        }
    }

    #[test]
    fn p12_authentication_documents_exact_provider_commands() {
        let getting_started =
            get_howto_doc("Getting Started").expect("embedded Getting Started guide");
        let auth = get_howto_doc("Authentication").expect("embedded Authentication guide");

        for command in [
            "bum login --provider xai",
            "bum login --provider codex",
            "bum login --provider codex --device-auth",
            "bum logout --provider xai",
            "bum logout --provider codex",
            "bum logout --all",
        ] {
            assert!(
                auth.contains(command),
                "02-authentication.md is missing exact provider command {command:?}"
            );
        }

        assert!(
            getting_started.contains("bum login --provider xai")
                && getting_started.contains("bum login --provider codex"),
            "01-getting-started.md must present both provider selectors"
        );
        let bare_logout = bare_markdown_command_lines(auth, "bum logout");
        assert_eq!(
            bare_logout,
            vec![(
                62,
                "Bare `bum logout` is rejected and does not mutate credentials. A provider-scoped"
            )],
            "Authentication guide may mention bare logout only in the explicit rejection; command-position occurrences are invalid"
        );
        let bare_login = bare_markdown_command_lines(auth, "bum login");
        assert_eq!(
            bare_login,
            vec![(
                44,
                "Bare `bum login` is retained as an xAI-only compatibility shortcut. Provider"
            )],
            "Authentication guide may mention bare login only as the documented xAI compatibility shortcut"
        );
        assert!(
            !getting_started.contains("run `bum login` and choose"),
            "Getting Started must not claim bare login offers provider selection"
        );
    }

    #[test]
    fn list_howto_titles_returns_all() {
        let titles = list_howto_titles();
        assert_eq!(titles.len(), USER_GUIDE.len() + REFERENCE_DOCS.len());
    }

    #[test]
    fn extract_writes_docs_and_cleans_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let docs_dir = tmp.path().join("docs").join("user-guide");

        std::fs::create_dir_all(&docs_dir).unwrap();
        std::fs::write(docs_dir.join("99-removed.md"), "stale").unwrap();
        std::fs::write(docs_dir.join("notes.md"), "user notes").unwrap();

        extract_user_guide_docs(tmp.path());

        for doc in USER_GUIDE {
            let path = docs_dir.join(doc.filename);
            assert!(path.exists(), "Expected doc {} to exist", doc.filename);
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                doc.content,
                "Content mismatch for {}",
                doc.filename
            );
        }
        assert!(
            !docs_dir.join("99-removed.md").exists(),
            "Stale doc should be cleaned up"
        );
        assert!(
            docs_dir.join("notes.md").exists(),
            "User file should not be deleted"
        );
    }
}
