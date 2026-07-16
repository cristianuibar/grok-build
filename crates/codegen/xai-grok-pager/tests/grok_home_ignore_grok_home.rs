//! Prove `GROK_HOME` is ignored as product home (D-HOME).
//!
//! Isolated binary: only this OnceLock/env scenario runs in the process.
//! Explicitly removes `BUM_HOME` so an ambient export cannot invalidate the
//! default-home path.

#[test]
fn grok_home_env_is_ignored_for_product_home() {
    let trap = tempfile::tempdir().expect("grok trap tempdir");
    let trap_path = trap.path().to_path_buf();

    // SAFETY: isolated test binary; env must be fixed before first resolve.
    unsafe {
        std::env::remove_var("BUM_HOME");
        std::env::set_var("GROK_HOME", &trap_path);
    }

    let home = xai_grok_config::grok_home();

    // Product home must not be the GROK_HOME trap path.
    assert_ne!(
        home, trap_path,
        "product home must ignore GROK_HOME trap path"
    );
    assert!(
        !home.starts_with(&trap_path),
        "product home must not live under GROK_HOME trap; got {}",
        home.display()
    );

    // Default product home ends with .bum under the resolved user home.
    assert!(
        home.ends_with(".bum"),
        "default product home must end with .bum; got {}",
        home.display()
    );

    // Display labels: default install shows ~/.bum (not $GROK_HOME / trap).
    let prefix = xai_grok_pager::util::display_grok_home_prefix();
    assert_eq!(
        prefix, "~/.bum",
        "with BUM_HOME removed, display prefix should be default ~/.bum"
    );

    // Paths helpers must not resolve under the trap.
    let pager_toml = xai_grok_pager::util::pager_toml_path();
    assert!(
        !pager_toml.starts_with(&trap_path),
        "pager.toml must not be under GROK_HOME trap; got {}",
        pager_toml.display()
    );
    assert_eq!(pager_toml, home.join("pager.toml"));
}
