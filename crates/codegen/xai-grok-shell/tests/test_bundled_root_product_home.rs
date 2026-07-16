//! Product-home isolation for `bundled_root()` (plan 01-05).
//!
//! Isolated integration binary so `grok_home()`'s process-wide OnceLock is
//! initialized from `BUM_HOME` only — no multi-scenario env fights.

use std::path::PathBuf;
use std::sync::OnceLock;

fn product_home() -> &'static PathBuf {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.keep();
        // SAFETY: once per test binary, before any grok_home()/bundled_root() call.
        unsafe {
            std::env::set_var("BUM_HOME", &path);
            std::env::remove_var("GROK_HOME");
        }
        path
    })
}

#[test]
fn bundled_root_lives_under_bum_product_home() {
    let home = product_home();
    let root = xai_grok_shell::bundle::bundled_root();
    assert_eq!(root, home.join("bundled"));
    assert!(
        !root.components().any(|c| c.as_os_str() == ".grok"),
        "bundled_root must not nest under stock .grok: {}",
        root.display()
    );
}

#[test]
fn bundled_root_does_not_use_stock_home_dot_grok() {
    let home = product_home();
    // Even if HOME points at a path with a stock .grok tree, product root wins.
    let decoy = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(decoy.path().join(".grok").join("bundled")).unwrap();
    unsafe {
        std::env::set_var("HOME", decoy.path());
    }
    let root = xai_grok_shell::bundle::bundled_root();
    assert_eq!(root, home.join("bundled"));
    assert_ne!(root, decoy.path().join(".grok").join("bundled"));
}
