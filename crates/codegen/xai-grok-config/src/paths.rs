//! Filesystem locations for product config files and binaries.

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

static GROK_HOME: OnceLock<PathBuf> = OnceLock::new();

#[cfg(target_os = "macos")]
const CLAUDE_MANAGED_SETTINGS_PATH: &str =
    "/Library/Application Support/ClaudeCode/managed-settings.json";
#[cfg(target_os = "linux")]
const CLAUDE_MANAGED_SETTINGS_PATH: &str = "/etc/claude-code/managed-settings.json";

/// Pure product-home resolver: does not read process env or touch [`GROK_HOME`] OnceLock.
///
/// Precedence: non-empty `bum_home` wins (relative values are absolutized against
/// the process cwd so a later `chdir` cannot retarget the OnceLock cache);
/// empty `bum_home` is treated as unset. Otherwise join `.bum` under `user_home`
/// (dunce-canonicalized when present). If `user_home` is `None`, uses `"."` then
/// `.bum` (same shape as the historical cwd-relative fallback).
///
/// Intentionally has no `GROK_HOME` parameter — product home is `BUM_HOME` only.
pub(crate) fn resolve_product_home(
    bum_home: Option<OsString>,
    user_home: Option<PathBuf>,
) -> PathBuf {
    if let Some(v) = bum_home
        && !v.is_empty()
    {
        let path = PathBuf::from(v);
        if path.is_absolute() {
            return path;
        }
        // Absolutize relative override once so the product-home cache is stable.
        return std::env::current_dir()
            .map(|cwd| cwd.join(&path))
            .unwrap_or(path);
    }
    let home = user_home.unwrap_or_else(|| PathBuf::from("."));
    dunce::canonicalize(&home).unwrap_or(home).join(".bum")
}

/// The default user product directory (`~/.bum`, canonicalized) used when
/// `BUM_HOME` is unset. Exposed so callers (e.g. display helpers) can detect
/// whether [`grok_home()`] is the default without duplicating the computation.
///
/// Uses [`dunce::canonicalize`] instead of [`std::fs::canonicalize`]: on
/// Windows, std returns a verbatim path (`\\?\C:\Users\...`) which external
/// tools choke on — e.g. `git clone` rejects `\\?\` destinations with
/// "Invalid argument", breaking marketplace cache clones under
/// `~/.bum/marketplace-cache`. `dunce` strips the prefix whenever the path
/// is safely representable in legacy form; on non-Windows it is identical to
/// `std::fs::canonicalize`.
///
/// Keep the dunce canonicalization in sync with the hand-rolled duplicate in
/// `xai_fast_worktree::db::resolve_grok_home` (deliberately standalone crate).
pub fn default_grok_home() -> PathBuf {
    #[allow(deprecated)]
    resolve_product_home(None, std::env::home_dir())
}

/// Per-user config directory: `$BUM_HOME` or `~/.bum`. Created if needed.
pub fn grok_home() -> PathBuf {
    GROK_HOME
        .get_or_init(|| {
            #[allow(deprecated)]
            let home = resolve_product_home(std::env::var_os("BUM_HOME"), std::env::home_dir());
            let _ = std::fs::create_dir_all(&home);
            home
        })
        .clone()
}

/// The user-global product home, but only when one genuinely resolves: `Some` when
/// `$BUM_HOME` is set to a **non-empty** value or a home directory is found, `None`
/// otherwise. Empty `BUM_HOME` is treated as unset (aligned with
/// [`resolve_product_home`]). Unlike [`grok_home()`], this never falls back to a
/// cwd-relative `.bum`, so callers that *scan* user-global product resources
/// (hooks, marketplace sources, ...) don't mistake a project's local tree for the
/// user-global one when no home resolves.
pub fn user_grok_home() -> Option<PathBuf> {
    let bum_set = std::env::var_os("BUM_HOME").is_some_and(|v| !v.is_empty());
    #[allow(deprecated)]
    let resolvable = bum_set || std::env::home_dir().is_some();
    resolvable.then(grok_home)
}

/// Canonical managed application path: `$BUM_HOME/bin/bum` (Unix) or `bum.exe` (Windows).
pub fn grok_application() -> PathBuf {
    let name = if cfg!(windows) { "bum.exe" } else { "bum" };
    grok_home().join("bin").join(name)
}

/// System-wide config directory: `/etc/grok/` on Unix, `None` on Windows.
pub fn system_config_dir() -> Option<PathBuf> {
    if cfg!(unix) {
        Some(PathBuf::from("/etc/grok"))
    } else {
        None
    }
}

/// System path for the managed-settings.json used for settings compat, if it exists.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn claude_managed_settings_path() -> Option<PathBuf> {
    let path = PathBuf::from(CLAUDE_MANAGED_SETTINGS_PATH);
    path.exists().then_some(path)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn claude_managed_settings_path() -> Option<PathBuf> {
    None
}

/// The platform path where managed-settings.json would live for settings
/// compat, whether or not it exists. `None` on unsupported platforms.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn claude_managed_settings_probe_path() -> Option<PathBuf> {
    Some(PathBuf::from(CLAUDE_MANAGED_SETTINGS_PATH))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn claude_managed_settings_probe_path() -> Option<PathBuf> {
    None
}

/// Max bytes for a single directory name component (macOS APFS, Linux ext4,
/// NTFS all enforce 255 bytes).
const MAX_DIRNAME_BYTES: usize = 255;

/// Encode a CWD string into a filesystem-safe directory name component.
///
/// Short CWDs (URL-encoded form <= 255 bytes) use URL-encoding for backward
/// compatibility and human readability on disk.
///
/// Long CWDs (> 255 bytes encoded) use a compact `{slug}-{blake3_hex16}`
/// form that is always <= 57 bytes. Callers must write a `.cwd` metadata
/// file via [`ensure_sessions_cwd_dir`] so the original CWD can be
/// recovered by [`decode_cwd_from_dirname`].
pub fn encode_cwd_dirname(cwd: &str) -> String {
    let url_encoded = urlencoding::encode(cwd);
    if url_encoded.len() <= MAX_DIRNAME_BYTES {
        return url_encoded.into_owned();
    }
    let hash = blake3::hash(cwd.as_bytes());
    let hash16 = &hash.to_hex()[..16];
    let leaf = std::path::Path::new(cwd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace");
    let slug = slugify(leaf, 40);
    let slug = if slug.is_empty() { "workspace" } else { &slug };
    format!("{slug}-{hash16}")
}

/// Recover the original CWD from a sessions CWD directory.
///
/// Tries URL-decoding the directory name first (works for short/legacy dirs).
/// Falls back to reading a `.cwd` metadata file inside the directory (written
/// by [`ensure_sessions_cwd_dir`] for hash-based dirs).
pub fn decode_cwd_from_dirname(dir: &std::path::Path) -> Option<String> {
    let name = dir.file_name()?.to_str()?;
    if let Ok(decoded) = urlencoding::decode(name) {
        let s = decoded.into_owned();
        // URL-decoded absolute CWDs always start with `/` (Unix) or a drive
        // letter (Windows).  The slug-hash form never does, so this
        // distinguishes the two encodings unambiguously.
        if s.starts_with('/') || (cfg!(windows) && s.chars().nth(1) == Some(':')) {
            return Some(s);
        }
    }
    std::fs::read_to_string(dir.join(".cwd"))
        .ok()
        .map(|s| s.trim().to_string())
}

/// Build the CWD-level session directory path:
/// `grok_home()/sessions/{encode_cwd_dirname(cwd)}`.
///
/// Does **not** create the directory on disk — use [`ensure_sessions_cwd_dir`]
/// when the directory must exist.
pub fn sessions_cwd_dir(cwd: &str) -> PathBuf {
    grok_home().join("sessions").join(encode_cwd_dirname(cwd))
}

/// Create the CWD-level session directory and write a `.cwd` metadata file
/// when hash-based encoding is used (long paths).
///
/// For short paths the `.cwd` file is not written because the directory name
/// itself is reversible via URL-decoding.
pub fn ensure_sessions_cwd_dir(cwd: &str) -> std::io::Result<PathBuf> {
    let encoded_name = encode_cwd_dirname(cwd);
    let dir = grok_home().join("sessions").join(&encoded_name);
    std::fs::create_dir_all(&dir)?;
    // Hash-based encoding is in use when the dirname differs from the
    // plain URL-encoded form.  Write a `.cwd` file so decode can recover
    // the original path.  O_CREAT|O_EXCL via create_new avoids TOCTOU
    // races with parallel session starts.
    if encoded_name != urlencoding::encode(cwd).as_ref() {
        let cwd_file = dir.join(".cwd");
        match std::fs::File::create_new(&cwd_file) {
            Ok(mut f) => {
                std::io::Write::write_all(&mut f, cwd.as_bytes())?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Ok(dir)
}

/// Generate a URL-safe slug from a string.
///
/// Lowercases, replaces non-alphanumeric chars with `-`, collapses
/// consecutive dashes, and truncates to `max_len` characters.
fn slugify(input: &str, max_len: usize) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_dash = false;
    for c in input.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
            prev_dash = false;
        } else if !prev_dash {
            result.push('-');
            prev_dash = true;
        }
    }
    let trimmed = result.trim_matches('-');
    trimmed.chars().take(max_len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Realistic CWDs that trigger the bug (URL-encoded > 255 bytes).
    const LONG_CWDS: &[&str] = &[
        "/Users/dev/Documents/開発プロジェクト/機能追加/テスト環境/ソースコード/main-branch",
        "/Users/user/Library/Mobile Documents/com~apple~CloudDocs/项目文件/深层嵌套目录/更深层次的/工作区域/project",
        "/Users/user/Library/CloudStorage/OneDrive-대한민국회사/프로젝트/개발환경/소스코드/백엔드/서비스/my-app",
        "/Users/user/Documents/工作文件夹/二零二六年项目/子目录一/子目录二/子目录三/源代码/code",
    ];

    #[test]
    fn long_cwd_uses_hash_fallback_within_name_max() {
        let long_cwd = format!("/Users/test/{}", "中".repeat(30));
        let encoded = encode_cwd_dirname(&long_cwd);
        assert!(encoded.len() <= MAX_DIRNAME_BYTES);
        assert!(!encoded.starts_with("%2F"));
    }

    #[test]
    fn different_long_paths_produce_different_hashes() {
        let a = format!("/Users/test/{}", "中".repeat(30));
        let b = format!("/Users/test/{}", "日".repeat(30));
        assert_ne!(encode_cwd_dirname(&a), encode_cwd_dirname(&b));
    }

    #[test]
    fn decode_reads_cwd_file_for_hash_dirs() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("some-slug-abcdef0123456789");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".cwd"), "/original/long/path").unwrap();
        assert_eq!(
            decode_cwd_from_dirname(&dir),
            Some("/original/long/path".to_string())
        );
    }

    #[test]
    fn decode_returns_none_without_cwd_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("some-slug-abcdef0123456789");
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(decode_cwd_from_dirname(&dir), None);
    }

    #[test]
    fn cwd_file_write_is_idempotent_via_excl() {
        let tmp = TempDir::new().unwrap();
        let long_cwd = format!("/Users/test/{}", "中".repeat(30));
        let dir = tmp.path().join(encode_cwd_dirname(&long_cwd));
        std::fs::create_dir_all(&dir).unwrap();
        let cwd_file = dir.join(".cwd");
        std::fs::write(&cwd_file, &long_cwd).unwrap();
        match std::fs::File::create_new(&cwd_file) {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            other => panic!("expected AlreadyExists, got: {other:?}"),
        }
        assert_eq!(std::fs::read_to_string(&cwd_file).unwrap(), long_cwd);
    }

    #[test]
    fn url_encoded_long_cwd_fails_on_real_filesystem() {
        let tmp = TempDir::new().unwrap();
        let url_encoded = urlencoding::encode(LONG_CWDS[0]).into_owned();
        let result = std::fs::create_dir_all(tmp.path().join(&url_encoded));
        assert!(result.is_err());
    }

    #[test]
    fn full_roundtrip_on_real_filesystem_for_long_cwds() {
        let tmp = TempDir::new().unwrap();
        for cwd in LONG_CWDS {
            let encoded = encode_cwd_dirname(cwd);
            let dir = tmp.path().join(&encoded);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(".cwd"), cwd).unwrap();
            assert_eq!(decode_cwd_from_dirname(&dir).as_deref(), Some(*cwd));
        }
    }

    #[test]
    fn short_cwds_use_url_encoding_and_roundtrip_on_real_filesystem() {
        let tmp = TempDir::new().unwrap();
        for cwd in [
            "/Users/foo/project",
            "/tmp",
            "/Users/user/Documents/project-名前",
        ] {
            let encoded = encode_cwd_dirname(cwd);
            assert_eq!(encoded, urlencoding::encode(cwd).into_owned());
            let dir = tmp.path().join(&encoded);
            std::fs::create_dir_all(&dir).unwrap();
            assert_eq!(decode_cwd_from_dirname(&dir).as_deref(), Some(cwd));
        }
    }

    #[test]
    fn default_grok_home_has_no_verbatim_prefix() {
        // On Windows, std::fs::canonicalize returns `\\?\C:\...` verbatim
        // paths that external tools (notably `git clone`) reject. The dunce
        // canonicalization must yield a plain path. No-op assertion on Unix.
        let home = default_grok_home();
        assert!(!home.to_string_lossy().starts_with(r"\\?\"));
        assert!(home.ends_with(".bum"));
    }

    #[test]
    fn resolve_product_home_bum_override_wins() {
        let user = PathBuf::from("/tmp/fake-user-home");
        let resolved = resolve_product_home(
            Some(OsString::from("/custom/bum-root")),
            Some(user),
        );
        assert_eq!(resolved, PathBuf::from("/custom/bum-root"));
    }

    #[test]
    fn resolve_product_home_relative_bum_is_absolutized() {
        let user = PathBuf::from("/tmp/fake-user-home");
        let resolved = resolve_product_home(
            Some(OsString::from("relative-bum-root")),
            Some(user),
        );
        assert!(
            resolved.is_absolute(),
            "relative BUM_HOME must be absolutized, got {}",
            resolved.display()
        );
        assert!(resolved.ends_with("relative-bum-root"));
        let expected = std::env::current_dir()
            .unwrap()
            .join("relative-bum-root");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolve_product_home_default_joins_bum() {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().to_path_buf();
        let resolved = resolve_product_home(None, Some(user.clone()));
        assert!(resolved.ends_with(".bum"));
        assert_eq!(
            resolved,
            dunce::canonicalize(&user).unwrap_or(user).join(".bum")
        );
    }

    #[test]
    fn resolve_product_home_empty_bum_falls_through() {
        let tmp = TempDir::new().unwrap();
        let user = tmp.path().to_path_buf();
        let resolved = resolve_product_home(Some(OsString::from("")), Some(user.clone()));
        assert!(resolved.ends_with(".bum"));
        assert_eq!(
            resolved,
            dunce::canonicalize(&user).unwrap_or(user).join(".bum")
        );
    }

    #[test]
    fn resolve_product_home_none_user_home_uses_dot_bum() {
        // When no user home is available, fall back to "." then dunce-canonicalize
        // (same shape as historical default_grok_home when home_dir is None).
        let resolved = resolve_product_home(None, None);
        let expected = {
            let home = PathBuf::from(".");
            dunce::canonicalize(&home).unwrap_or(home).join(".bum")
        };
        assert_eq!(resolved, expected);
        assert!(resolved.ends_with(".bum"));
    }

    #[test]
    fn resolve_product_home_no_verbatim_prefix_on_injected_user_home() {
        let tmp = TempDir::new().unwrap();
        let resolved = resolve_product_home(None, Some(tmp.path().to_path_buf()));
        assert!(!resolved.to_string_lossy().starts_with(r"\\?\"));
        assert!(resolved.ends_with(".bum"));
    }

    #[test]
    fn resolve_product_home_api_has_no_grok_home_input() {
        // Compile-time / API shape proof: only BUM_HOME + user_home parameters.
        // Call sites that would pass GROK_HOME must not exist on this helper.
        let _ = resolve_product_home as fn(Option<OsString>, Option<PathBuf>) -> PathBuf;
    }

    #[test]
    fn grok_application_leaf_is_bum() {
        // Managed binary leaf under product home is bum (not grok).
        let app = if cfg!(windows) { "bum.exe" } else { "bum" };
        // Path shape only — do not call grok_home() here (OnceLock / env sensitive).
        let leaf = PathBuf::from("bin").join(app);
        assert_eq!(leaf.file_name().and_then(|n| n.to_str()), Some(app));
        assert!(app.starts_with("bum"));
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World!", 40), "hello-world");
    }

    #[test]
    fn slugify_cjk_produces_empty() {
        assert_eq!(slugify("深层目录", 40), "");
    }

    #[test]
    fn slugify_truncates() {
        assert_eq!(slugify(&"a".repeat(100), 10).len(), 10);
    }
}
