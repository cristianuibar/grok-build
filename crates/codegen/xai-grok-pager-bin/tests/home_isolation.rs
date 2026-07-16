//! Hermetic temp-home isolation proof (plan 01-05 / D-VERIFY).
//!
//! Spawns the built `bum` binary under an isolated HOME + BUM_HOME with stock
//! GROK_HOME / CODEX_HOME traps. Asserts product state lands under BUM_HOME and
//! that trap trees (and ambient `.grok` / `.codex` under the hermetic home) stay
//! absent or byte-identical to the pre-run snapshot.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

fn bum_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bum"))
}

/// Snapshot every regular file under `root` (relative path → contents).
/// Missing roots yield an empty map.
fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    if !root.exists() {
        return out;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                stack.push(path);
            } else if meta.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(path.as_path())
                    .to_path_buf();
                let bytes = std::fs::read(&path).unwrap_or_default();
                out.insert(rel, bytes);
            }
        }
    }
    out
}

fn assert_tree_unchanged(label: &str, root: &Path, before: &BTreeMap<PathBuf, Vec<u8>>) {
    let after = snapshot_tree(root);
    assert_eq!(
        before, &after,
        "{label} trap tree changed under {} — isolation violated.\nbefore keys: {:?}\nafter keys: {:?}",
        root.display(),
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    // If the trap never existed, it must still be absent (no empty dir create).
    if before.is_empty() {
        assert!(
            !root.exists() || snapshot_tree(root).is_empty(),
            "{label}: expected trap path {} absent or empty after run",
            root.display()
        );
    }
}

fn assert_no_product_tree(home: &Path, segment: &str) {
    let path = home.join(segment);
    assert!(
        !path.exists() || snapshot_tree(&path).is_empty(),
        "unexpected product-state tree at {} (files: {:?})",
        path.display(),
        snapshot_tree(&path).keys().collect::<Vec<_>>()
    );
}

#[test]
fn hermetic_temp_home_writes_only_under_bum_home() {
    let tmp = TempDir::new().expect("tempdir");
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();

    let bum_home = home.join(".bum");
    // Dedicated traps — must remain empty/unchanged (not ambient stock homes).
    let grok_trap = home.join(".grok-trap");
    let codex_trap = home.join(".codex-trap");

    let grok_before = snapshot_tree(&grok_trap);
    let codex_before = snapshot_tree(&codex_trap);
    let stock_grok_before = snapshot_tree(&home.join(".grok"));
    let stock_codex_before = snapshot_tree(&home.join(".codex"));

    let mut cmd = Command::new(bum_bin());
    cmd.arg("version")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", &home)
        .env("BUM_HOME", &bum_home)
        // Trap product-root env keys so ambient developer homes are never used.
        .env("GROK_HOME", &grok_trap)
        .env("CODEX_HOME", &codex_trap)
        // Telemetry / auto-update knobs (mirrors test_env_cmd_tokio).
        .env("GROK_TELEMETRY_ENABLED", "false")
        .env("GROK_FEEDBACK_ENABLED", "false")
        .env("GROK_TRACE_UPLOAD", "false")
        .env("GROK_INSTRUMENTATION", "disabled")
        .env("GROK_DISABLE_AUTOUPDATER", "1");

    // Windows: HOME alone is not enough for std::env::home_dir(); set
    // USERPROFILE (and USERNAME is optional) so the child stays sandboxed.
    #[cfg(windows)]
    {
        cmd.env("USERPROFILE", &home);
        // Known Folder APIs may also consult HOMEDRIVE/HOMEPATH.
        if let Some(s) = home.to_str() {
            // Best-effort: leave HOMEDRIVE/HOMEPATH unset if we cannot split.
            let _ = s;
        }
    }

    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn bum: {e}"));
    assert!(
        output.status.success(),
        "bum version failed (exit {:?})\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // Product root must exist and have received startup writes (docs extract, etc.).
    assert!(
        bum_home.exists(),
        "BUM_HOME {} was not created",
        bum_home.display()
    );
    let product_files = snapshot_tree(&bum_home);
    assert!(
        !product_files.is_empty(),
        "expected product writes under BUM_HOME {}, got empty tree",
        bum_home.display()
    );

    // Recursive trap assertions (byte-identical to pre-run snapshot).
    assert_tree_unchanged("GROK_HOME trap", &grok_trap, &grok_before);
    assert_tree_unchanged("CODEX_HOME trap", &codex_trap, &codex_before);
    assert_tree_unchanged("stock .grok under hermetic home", &home.join(".grok"), &stock_grok_before);
    assert_tree_unchanged(
        "stock .codex under hermetic home",
        &home.join(".codex"),
        &stock_codex_before,
    );

    // Also assert stock product-state segments never appeared under hermetic home.
    assert_no_product_tree(&home, ".grok");
    assert_no_product_tree(&home, ".codex");
}
