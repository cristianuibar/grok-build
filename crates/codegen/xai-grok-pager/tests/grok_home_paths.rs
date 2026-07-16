//! `BUM_HOME` override tests in an isolated binary so `grok_home()`'s
//! process-wide `OnceLock` initializes from the overridden env var.
//!
//! Only one env-sensitive scenario lives in this process — do not add a
//! second OnceLock-touching home scenario here.

use std::path::PathBuf;

#[test]
fn bum_home_override_path_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bum_home = tmp.path().to_path_buf();
    // SAFETY: isolated test binary; set before any resolver/display call.
    unsafe {
        std::env::set_var("BUM_HOME", &bum_home);
    }

    assert_eq!(
        xai_grok_pager::util::pager_toml_path(),
        bum_home.join("pager.toml")
    );
    assert_eq!(
        xai_grok_pager::util::display_grok_home_prefix(),
        "$BUM_HOME"
    );
    assert_eq!(
        xai_grok_pager::util::display_user_grok_path("config.toml"),
        "$BUM_HOME/config.toml"
    );

    let memory_path = bum_home.join("memory/MEMORY.md");
    assert_eq!(
        xai_grok_pager::util::abbreviate_path(&memory_path.display().to_string()),
        "$BUM_HOME/memory/MEMORY.md"
    );

    assert!(xai_grok_pager::util::is_under_user_grok_home(&memory_path));
    assert!(!xai_grok_pager::util::is_under_user_grok_home(
        PathBuf::from("/tmp/other").as_path()
    ));
}
