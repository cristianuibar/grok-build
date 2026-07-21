<div align="center">

# bum — Build Using Multiagents

**bum** is a full-product fork of the Grok Build terminal AI coding agent.
It is a multi-provider daily driver: one CLI (`bum`), dual OAuth (xAI for Grok
models + ChatGPT/Codex for GPT), per-model routing, and an isolated product
home at `~/.bum` (`$BUM_HOME`).

[Building from source](#building-from-source) ·
[Authentication](#authentication) ·
[Documentation](#documentation) ·
[Repository layout](#repository-layout) ·
[Development](#development) ·
[License](#license)

</div>

---

## What this is

This repository is the Rust workspace for the **bum** CLI/TUI and agent
runtime. It keeps the Grok Build harness lineage (crates remain `xai-grok-*`,
model catalog still includes **Grok Build (xAI)** / `grok-build`) and evolves
it into a private multi-provider product:

bum uses ChatGPT/Codex OAuth and compatible model APIs, but it is not the stock OpenAI Codex CLI.
See the [provider capability contract](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md#provider-capability-contract) for the supported transport, identity, continuity, and tool boundaries.

| Concern | bum |
|---------|-----|
| CLI binary | `bum` |
| Product name | **bum** / **BUM** (*Build Using Multiagents*) |
| Product home | `~/.bum` or `$BUM_HOME` |
| Auth | xAI OAuth (Grok) + ChatGPT/Codex OAuth (GPT), dual slots |
| Routing | Per-model provider selection (mixed picker) |
| Telemetry / auto-update | Quiet fork — no stock x.ai phone-home as primary path |

> [!NOTE]
> This is **not** the stock x.ai Grok client install path. Prefer building
> from this tree. Official x.ai CLI docs remain useful for upstream lineage
> behavior, but are not the primary install channel for bum.

## Building from source

Requirements:

- **Rust** — toolchain pinned by [`rust-toolchain.toml`](rust-toolchain.toml);
  `rustup` installs it on first build.
- **protoc** — resolves [`bin/protoc`](bin/protoc) (a
  [dotslash](https://dotslash-cli.com) launcher) or `$PROTOC` / `PATH`.
- macOS and Linux supported; Windows is best-effort.

```sh
# Build + launch the TUI
cargo run -p xai-grok-pager-bin --bin bum

# Debug binary
cargo build -p xai-grok-pager-bin --bin bum
./target/debug/bum

# Release
cargo build -p xai-grok-pager-bin --bin bum --release
./target/release/bum

# Fast validation
cargo check -p xai-grok-pager-bin
```

On first launch, authenticate with the providers you need (see below). Config
and credentials live under `~/.bum` (override with `BUM_HOME`).

## Authentication

Dual OAuth is the primary path:

```sh
bum login                  # xAI / Grok (default)
bum login --provider codex # ChatGPT / Codex
bum auth status            # both slots, never prints secrets
bum logout --provider xai  # selective logout
bum logout --all
```

Then pick models mid-session; routing follows each model’s provider binding.

## Documentation

In-tree user guide (pager crate):

[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)

Getting started, keyboard shortcuts, slash commands, configuration, theming,
MCP, skills, plugins, hooks, headless mode, sandboxing, and more. The
[provider capability contract](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md#provider-capability-contract)
documents provider-specific behavior without equating bum with a stock CLI.

## Repository layout

| Path | Contents |
|------|----------|
| `crates/codegen/xai-grok-pager-bin` | Composition root; binary artifact **`bum`** |
| `crates/codegen/xai-grok-pager` | TUI: scrollback, prompt, modals, rendering |
| `crates/codegen/xai-grok-shell` | Agent runtime + leader/stdio/headless |
| `crates/codegen/xai-grok-tools` | Tool implementations (terminal, edit, search, …) |
| `crates/codegen/xai-grok-workspace` | Host filesystem, VCS, execution, checkpoints |
| `crates/codegen/...` | Config, MCP, markdown, sandbox, auth, … |
| `crates/common/`, `crates/build/`, `prod/mc/` | Shared leaf crates |
| `third_party/` | Vendored upstream (Mermaid diagram stack) |

Crate package names keep the `xai-grok-*` prefix (fork lineage). Product-facing
identity is **bum** only.

> [!IMPORTANT]
> The root `Cargo.toml` (workspace members, dependency versions, lints,
> profiles) is **generated** — treat it as read-only. Prefer editing per-crate
> `Cargo.toml` files.

## Development

```sh
cargo check -p <crate>        # target specific crates; full workspace is slow
cargo test -p xai-grok-config
cargo clippy -p <crate>       # clippy.toml at repo root
cargo fmt --all               # rustfmt.toml at repo root
```

## Contributing

> [!NOTE]
> External contributions are not accepted. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

First-party code in this repository is licensed under the **Apache License,
Version 2.0** — see [`LICENSE`](LICENSE).

Third-party and vendored code remains under its original licenses. See:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) — crates.io / git dependencies,
  bundled UI themes, and **in-tree source ports** (including openai/codex and
  sst/opencode tool implementations)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
  — crate-local notice for the codex and opencode ports
- [`third_party/NOTICE`](third_party/NOTICE) — vendored Mermaid-stack index
