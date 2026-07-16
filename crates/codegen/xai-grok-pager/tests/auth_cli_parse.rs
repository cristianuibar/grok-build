//! Phase 5 Wave 0 — dual-provider clap parse contracts (D-01 / D-05 / D-06).
//!
//! Integration binary so named filters run without compiling the full
//! `xai-grok-pager` lib-test target (pre-existing pager-render `cfg(test)`
//! helper visibility breaks `cargo test -p xai-grok-pager --lib`).
//!
//! Prefer:
//! ```text
//! cargo test -p xai-grok-pager --test auth_cli_parse bum_login_provider_codex_parses
//! ```

use clap::Parser;
use xai_grok_pager::app::{AuthCommand, AuthProviderArg, Command, PagerArgs};

/// D-01: bare `bum login` leaves provider=None (xAI default).
#[test]
fn bum_login_defaults_to_xai_without_provider_argument() {
    let args = PagerArgs::try_parse_from(["bum", "login"]).expect("bare bum login parses");
    let Some(Command::Login {
        oauth,
        device_auth,
        provider,
        ..
    }) = args.command
    else {
        panic!("expected login command");
    };
    assert!(!oauth);
    assert!(!device_auth);
    assert_eq!(
        provider, None,
        "bare bum login must leave provider=None (xAI default)"
    );

    let oauth = PagerArgs::try_parse_from(["bum", "login", "--oauth"]).expect("--oauth");
    assert!(matches!(
        oauth.command,
        Some(Command::Login {
            oauth: true,
            device_auth: false,
            provider: None,
            ..
        })
    ));

    let device =
        PagerArgs::try_parse_from(["bum", "login", "--device-auth"]).expect("--device-auth");
    assert!(matches!(
        device.command,
        Some(Command::Login {
            oauth: false,
            device_auth: true,
            provider: None,
            ..
        })
    ));

    let conflict =
        PagerArgs::try_parse_from(["bum", "login", "--oauth", "--device-auth"]).unwrap_err();
    assert_eq!(
        conflict.kind(),
        clap::error::ErrorKind::ArgumentConflict,
        "login transport flags must remain mutually exclusive"
    );
}

/// D-01: `bum login --provider codex` parses.
#[test]
fn bum_login_provider_codex_parses() {
    let args = PagerArgs::try_parse_from(["bum", "login", "--provider", "codex"])
        .expect("login --provider codex parses");
    let Some(Command::Login {
        provider,
        oauth,
        device_auth,
        ..
    }) = args.command
    else {
        panic!("expected login command");
    };
    assert_eq!(provider, Some(AuthProviderArg::Codex));
    assert!(!oauth);
    assert!(!device_auth);

    let with_device =
        PagerArgs::try_parse_from(["bum", "login", "--provider", "codex", "--device-auth"])
            .expect("codex + device-auth parses");
    assert!(matches!(
        with_device.command,
        Some(Command::Login {
            provider: Some(AuthProviderArg::Codex),
            device_auth: true,
            oauth: false,
            ..
        })
    ));
}

/// D-01: `bum login --provider xai` parses.
#[test]
fn bum_login_provider_xai_parses() {
    let args = PagerArgs::try_parse_from(["bum", "login", "--provider", "xai"])
        .expect("login --provider xai parses");
    let Some(Command::Login { provider, .. }) = args.command else {
        panic!("expected login command");
    };
    assert_eq!(provider, Some(AuthProviderArg::Xai));
}

/// D-05: bare logout parse shape (handler fail-closed is Plan 04).
#[test]
fn bum_logout_requires_provider_or_all() {
    let bare = PagerArgs::try_parse_from(["bum", "logout"]).expect("bare logout parses");
    let Some(Command::Logout { provider, all }) = bare.command else {
        panic!("expected logout command");
    };
    assert_eq!(provider, None);
    assert!(!all);

    let selective =
        PagerArgs::try_parse_from(["bum", "logout", "--provider", "codex"]).expect("selective");
    assert!(matches!(
        selective.command,
        Some(Command::Logout {
            provider: Some(AuthProviderArg::Codex),
            all: false,
        })
    ));

    let xai = PagerArgs::try_parse_from(["bum", "logout", "--provider", "xai"]).expect("logout xai");
    assert!(matches!(
        xai.command,
        Some(Command::Logout {
            provider: Some(AuthProviderArg::Xai),
            all: false,
        })
    ));

    let conflict =
        PagerArgs::try_parse_from(["bum", "logout", "--provider", "codex", "--all"]).unwrap_err();
    assert_eq!(
        conflict.kind(),
        clap::error::ErrorKind::ArgumentConflict,
        "--provider and --all must conflict"
    );
}

/// D-05: `bum logout --all` parses.
#[test]
fn bum_logout_all_parses() {
    let args = PagerArgs::try_parse_from(["bum", "logout", "--all"]).expect("logout --all parses");
    assert!(matches!(
        args.command,
        Some(Command::Logout {
            provider: None,
            all: true,
        })
    ));
}

/// D-06: `bum auth status` parses.
#[test]
fn bum_auth_status_parses() {
    let args = PagerArgs::try_parse_from(["bum", "auth", "status"]).expect("auth status parses");
    assert!(matches!(
        args.command,
        Some(Command::Auth {
            command: AuthCommand::Status,
        })
    ));
}
