# Getting Started

**bum** (Build Using Multiagents) is a terminal-based AI coding assistant built from the Grok Build harness. It runs as a TUI (Terminal User Interface) that understands your codebase, executes shell commands, edits files, searches the web, and manages tasks.

You can use it interactively as a full-screen TUI, run it headlessly for scripting and CI/CD, or integrate it into editors via the Agent Client Protocol (ACP).

---

## Installation

bum is currently distributed as a source build rather than through the stock
x.ai installer. Install the pinned Rust toolchain, then build the product
binary from this repository:

```bash
cargo build -p xai-grok-pager-bin --bin bum --release
./target/release/bum --version
```

For a faster development build:

```bash
cargo build -p xai-grok-pager-bin --bin bum
./target/debug/bum --version
```

Copy or symlink the resulting `bum` executable into a directory on your
`PATH`, then verify the installation:

```bash
bum --version
```

The stock x.ai update channel is disabled in bum. Update the repository and
rebuild when you want a newer version.

---

## First Launch

Start bum by running:

```bash
bum
```

Authenticate the provider slot you intend to use explicitly:

```bash
bum login --provider xai
bum login --provider codex
```

Bare `bum login` remains an xAI-only compatibility shortcut; it does not prompt
you to choose a provider.

On first launch, bum guides you through authentication for the provider you
use. After you sign in, bum stores the provider credentials in
`~/.bum/auth.json` (or `$BUM_HOME/auth.json`), where they persist across
sessions. bum refreshes renewable credentials automatically and prompts you to
sign in again when they can no longer be renewed.

If you prefer API key authentication (e.g., for CI/CD or environments without a browser), set the `XAI_API_KEY` environment variable instead:

```bash
export XAI_API_KEY="xai-..."
bum
```

See [Authentication](02-authentication.md) for the full set of auth options including OIDC, external auth providers, and device code flow.

---

## Basic Interaction

Once authenticated, bum presents a full-screen TUI with two main areas:

- **Scrollback** -- the conversation history showing your prompts, bum's responses, tool calls, file edits, and more.
- **Prompt** -- the input area at the bottom where you type messages.

Type a message and press `Enter` to send it. bum reads files, runs commands, and edits code as needed. Each tool run streams into the scrollback in real time.

Press `Tab` to move focus between the prompt and the scrollback. While a turn is running, `Ctrl+C` cancels it (or clears a non-empty draft first); `Esc` is a no-op mid-turn. Idle, press `Esc` twice within 800ms to clear a non-empty prompt, or (with an empty prompt and conversation messages) to open rewind — see [Keyboard Shortcuts](03-keyboard-shortcuts.md#escape). With the scrollback focused, use the arrow keys to select entries and to collapse or expand them. To navigate with `j`/`k` and fold with `h`/`l` instead, enable Vim mode.

### File References

Use `@` in your prompt to attach files:

```
@src/main.rs              # Attach a file
@src/main.rs:10-50        # Attach lines 10-50
@src/                     # Browse a directory
```

The `@` operator opens a fuzzy file picker. By default it respects `.gitignore` and hides dotfiles. Prefix with `!` to search hidden files:

```
@!.github                 # Search hidden files
@!.env                    # Attach a .env file
```

### Permissions

By default, bum asks for permission before executing shell commands or editing files. You can approve individually or toggle always-approve mode:

- Press `Ctrl+O` to toggle always-approve mode
- Use the `--yolo` flag at launch: `bum --yolo`
- Type `/always-approve` in the prompt to toggle the mode

---

## Key Concepts

### Sessions

Every conversation is a **session**. Sessions are automatically saved to
`~/.bum/sessions/` (or `$BUM_HOME/sessions/`) and can be resumed later. Each
session tracks the full conversation history, tool calls, file edits, and task
state.

- Start a new session: `Ctrl+N` or `/new`
- Resume a previous session: `/resume` in the TUI, or `--resume <ID>` from the CLI
- Continue the most recent session: `bum -c`

### Scrollback

The scrollback is the main display area. It shows:

- **User prompts** -- your messages, rendered as sticky headers
- **Agent messages** -- bum's responses with full markdown rendering and syntax highlighting
- **Thinking blocks** -- the selected model's reasoning process (collapsible when available)
- **Tool calls** -- file edits (with inline diffs), command executions, search results, and more
- **Task lists** -- TODO items tracking progress

Collapse or expand the selected entry with the `Left`/`Right` arrow keys (or `h`/`l` and `e` in Vim mode). In Vim mode, press `y` to copy its content and `Y` to copy its metadata (for example, the command that ran). Press `Enter` to open it in the fullscreen viewer (in any mode).

### Tools

bum has built-in tools for:

| Tool | Description |
|------|-------------|
| `read_file` / `search_replace` | Read and edit files with line-precise changes |
| `grep` | Regex search across your codebase (powered by ripgrep) |
| `list_dir` | List directory contents |
| `run_terminal_command` | Execute shell commands |
| `web_search` / `web_fetch` | Search the web and fetch URLs |
| `todo_write` | Create and manage task lists |
| `spawn_subagent` | Spawn parallel subagent sessions |
| `memory_search` | Search cross-session memory |

Tools can be extended with [MCP servers](05-configuration.md#mcp-servers) for integrations like GitHub, databases, and more.

### Slash Commands

Type `/` in the prompt to access commands. These provide quick actions without writing a full prompt:

```
/model grok-build                 # Switch model
/compact                          # Compress conversation history
/always-approve                   # Toggle always-approve mode
/new                              # Start a new session
```

See [Slash Commands](04-slash-commands.md) for the complete reference.

---

## Common Launch Options

```bash
# Launch the interactive TUI and submit an initial prompt as the first turn
bum "fix the failing auth test and run it"

# Initial prompt in a new git worktree. Use --worktree=<name> (with `=`) so the
# prompt isn't swallowed as the worktree name — `bum -w "refactor module X"`
# would treat "refactor module X" as the worktree label, not the prompt.
bum --worktree=feat "refactor module X"

# Base the worktree on a specific branch (e.g. main) instead of the current HEAD:
bum -w --ref main "implement feature from main"


# Start in a specific project directory
bum --cwd ~/projects/my-app

# Add project-specific rules
bum --rules "Always use TypeScript. Prefer functional components."

# Auto-approve all tool executions
bum --yolo

# Use a specific model
bum -m grok-build

# Resume a previous session
bum --resume <session-id>

# Continue the most recent session
bum -c

# Experimental scrollback-native render mode. Sticky: plain `bum` reopens in
# the mode last chosen via --minimal/--fullscreen (or /minimal//fullscreen).
bum --minimal

# Back to the standard fullscreen TUI (and make it sticky again)
bum --fullscreen

# Headless mode (for scripts)
bum -p "Explain this codebase"
```

---

## Headless Mode

Run bum non-interactively for scripting, CI/CD, and automation:

```bash
bum -p "Your prompt here"
```

Output formats:

| Format | Flag | Description |
|--------|------|-------------|
| `plain` | (default) | Human-readable text |
| `json` | `--output-format json` | Single JSON object with `text`, `stopReason`, `sessionId`, and `requestId` |
| `streaming-json` | `--output-format streaming-json` | NDJSON event stream for real-time processing |

Example CI/CD usage:

```bash
bum -p "Review changes for bugs" --output-format json --yolo | jq -r '.text'
```

---

## Project Rules (AGENTS.md)

Add per-project instructions by creating an `AGENTS.md` file in your repository. bum reads these files and injects their contents as a project-instructions message at the start of the conversation:

```
~/.bum/AGENTS.md            # Global rules (apply to all projects)
<repo-root>/AGENTS.md       # Repository-level rules
<cwd>/AGENTS.md             # Directory-level rules (highest priority)
```

Deeper files take precedence. bum also reads `CLAUDE.md` files for compatibility.

---

## Where to Go Next

| Document | What You Will Learn |
|----------|-------------------|
| [Authentication](02-authentication.md) | Browser login, API keys, OIDC, external auth, device code flow |
| [Keyboard Shortcuts](03-keyboard-shortcuts.md) | Complete reference for all key bindings |
| [Slash Commands](04-slash-commands.md) | All available `/` commands |
| [Configuration](05-configuration.md) | config.toml, pager.toml, environment variables |
