//! Devin Desktop keeps its sign-in in Electron's state DB —
//! `User/globalStorage/state.vscdb` under the userData dir — as the
//! `codeium.windsurf` extension's `windsurf_auth.sessions` secret:
//!
//!   value = {"type":"Buffer","data":[...]}  (JSON byte array)
//!   bytes = b"v10" || nonce(12) || AES-256-GCM(secret) ciphertext||tag(16)
//!   key   = DPAPI-decrypt(b64decode(Local State .os_crypt.encrypted_key)[5:])
//!
//! Two plaintext mirrors ride along: `windsurfAuthStatus.apiKey` and
//! `codeium.windsurf.lastLoginEmail`. All three are patched on switch.
//!
//! Desktop MUST be fully closed when this runs — it holds state.vscdb
//! open and would overwrite (or clobber) a concurrent write.
//!
//! Only implemented on Windows: the os_crypt key is DPAPI-wrapped there.
//! macOS keeps it in the Keychain, Linux in a different store entirely.

use std::path::PathBuf;

const SESSIONS_KEY: &str =
    r#"secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.sessions"}"#;

/// Product dirs Electron uses for Devin Desktop — "Devin" on current
/// builds, "Windsurf" on Code-OSS-lineage installs (confirmed in the
/// field: %APPDATA%\Windsurf\User\globalStorage\state.vscdb).
const PRODUCT_DIRS: [&str; 2] = ["Devin", "Windsurf"];
/// Main executable names under those install dirs.
const EXE_NAMES: [&str; 2] = ["devin.exe", "windsurf.exe"];

fn looks_like_userdata(d: &std::path::Path) -> bool {
    d.join("Local State").is_file()
        || d.join("User")
            .join("globalStorage")
            .join("state.vscdb")
            .is_file()
}

/// Every directory Desktop's userData could live at, in preference
/// order — the first one holding Electron state wins.
fn candidate_dirs() -> Vec<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    let mut bases: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        for var in ["APPDATA", "LOCALAPPDATA"] {
            if let Ok(v) = std::env::var(var) {
                if !v.is_empty() {
                    bases.push(PathBuf::from(v));
                }
            }
        }
        if bases.is_empty() {
            bases.push(home.join("AppData").join("Roaming"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        bases.push(home.join("Library").join("Application Support"));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        bases.push(
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".config")),
        );
    }
    let mut out = Vec::new();
    for b in bases {
        for n in PRODUCT_DIRS {
            out.push(b.join(n));
        }
    }
    out
}

/// The actual userData dir. Resolve by the state DB itself — a
/// skeleton dir holding only `Local State` (left behind by an older
/// install or a short-lived run) must not shadow a product dir that
/// has the real `state.vscdb`, and the os_crypt key lives in the SAME
/// product dir as the db, so product_name/vault_key stay consistent.
pub fn userdata_dir() -> Option<PathBuf> {
    let has_db = |d: &PathBuf| {
        d.join("User")
            .join("globalStorage")
            .join("state.vscdb")
            .is_file()
    };
    candidate_dirs()
        .into_iter()
        .find(has_db)
        .or_else(|| candidate_dirs().into_iter().find(|d| looks_like_userdata(d)))
}

/// Install dirs where a Desktop exe could live — used to detect
/// "installed but never launched" (no userData exists yet).
fn install_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(l) = std::env::var("LOCALAPPDATA") {
        for n in PRODUCT_DIRS {
            let d = PathBuf::from(&l).join("Programs").join(n);
            if d.is_dir() {
                v.push(d);
            }
        }
    }
    for var in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Ok(pf) = std::env::var(var) {
            for n in PRODUCT_DIRS {
                let d = PathBuf::from(&pf).join(n);
                if d.is_dir() {
                    v.push(d);
                }
            }
        }
    }
    v
}

/// The Electron state DB — exposed so callers can cache by its mtime.
/// Only returned when the file itself exists.
pub fn db_path() -> Option<PathBuf> {
    userdata_dir()
        .map(|d| d.join("User").join("globalStorage").join("state.vscdb"))
        .filter(|p| p.is_file())
}

/// Desktop is installed: state/userData exists, or an exe does (a
/// fresh install that never ran has no userData yet).
pub fn installed() -> bool {
    if userdata_dir().is_some() {
        return true;
    }
    install_dirs().iter().any(|d| {
        std::fs::read_dir(d)
            .map(|rd| {
                rd.flatten().any(|e| {
                    let n = e.file_name().to_string_lossy().to_lowercase();
                    EXE_NAMES.contains(&n.as_str())
                })
            })
            .unwrap_or(false)
    })
}

/// The product name owning the resolved userData ("Devin" or
/// "Windsurf") — the processes that could hold state.vscdb open are
/// that product's, not any devin-named binary.
pub fn product_name() -> String {
    userdata_dir()
        .and_then(|d| d.file_name().map(|f| f.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Devin".to_string())
}

/// Any process of the Desktop product alive right now. Case-sensitive —
/// the CLI's binary is lowercase `devin` and must NOT match; a real
/// Windsurf IDE only counts when the userData resolved to Windsurf
/// (then it genuinely holds that same state file open).
pub fn is_running() -> bool {
    let product = product_name();
    let exe = format!("{product}.exe");
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    sys.processes().values().any(|p| {
        if p.name().starts_with(&product) {
            return true;
        }
        p.exe()
            .and_then(|e| e.file_name())
            .map(|f| f.to_string_lossy() == exe)
            .unwrap_or(false)
    })
}

/// Rewrite Desktop's stored session as `token` belonging to
/// (label, user_id, email). Errors when state can't be reached or the
/// session row is absent (Desktop never signed in).
pub fn switch_session(
    token: &str,
    label: &str,
    user_id: &str,
    email: &str,
) -> Result<(), String> {
    let db = db_path().ok_or("Devin Desktop data dir not found")?;
    if !db.is_file() {
        return Err("Devin Desktop data dir not found".into());
    }
    let key = vault_key()?;

    let sessions = serde_json::json!([{
        "id": uuid4(),
        "accessToken": token,
        "account": { "label": label, "id": user_id },
        "scopes": [],
    }]);
    let sealed = seal(&key, sessions.to_string().as_bytes())?;
    let bufval = buffer_json(&sealed);

    let conn = rusqlite::Connection::open(&db)
        .map_err(|e| format!("open {}: {e}", db.display()))?;
    let _ = conn.busy_timeout(std::time::Duration::from_secs(3));
    let n = conn
        .execute(
            "UPDATE ItemTable SET value=?1 WHERE key=?2",
            rusqlite::params![bufval, SESSIONS_KEY],
        )
        .map_err(|e| format!("write sessions: {e}"))?;
    if n == 0 {
        return Err("no Desktop session stored — sign in to Devin Desktop once first".into());
    }

    patch_json(&conn, "windsurfAuthStatus", |v| {
        v["apiKey"] = serde_json::json!(token)
    });
    patch_json(&conn, "codeium.windsurf", |v| {
        v["lastLoginEmail"] = serde_json::json!(email)
    });
    Ok(())
}

/// SELECT → mutate one JSON row → UPDATE. Missing/unparseable rows are
/// skipped silently: they're caches Desktop rebuilds anyway.
fn patch_json(
    conn: &rusqlite::Connection,
    key: &str,
    f: impl FnOnce(&mut serde_json::Value),
) {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM ItemTable WHERE key=?1", [key], |r| r.get(0))
        .ok();
    let Some(raw) = raw else { return };
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    f(&mut v);
    let _ = conn.execute(
        "UPDATE ItemTable SET value=?1 WHERE key=?2",
        rusqlite::params![v.to_string(), key],
    );
}

/// Read-side connection to state.vscdb. Desktop keeps it in WAL mode,
/// and a pure READ_ONLY open dies with SQLITE_CANTOPEN once the -shm
/// file is gone (a read-only connection can neither create it nor run
/// recovery) — the file can be right there and still "fail to open".
/// Roaming-profile redirection produces the same symptom. Fall back to
/// copying the db (+ -wal/-shm sidecars) to %TEMP% and opening the
/// copy: WAL recovery runs normally there and every "exists but won't
/// open" cause is sidestepped. Returns the temp dir to clean up.
fn open_state_db(
    db: &std::path::Path,
) -> Result<(rusqlite::Connection, Option<PathBuf>), String> {
    match rusqlite::Connection::open_with_flags(
        db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(conn) => {
            let _ = conn.busy_timeout(std::time::Duration::from_secs(3));
            Ok((conn, None))
        }
        Err(e) => {
            let tmp = std::env::temp_dir().join("devin-switch");
            let _ = std::fs::create_dir_all(&tmp);
            let dst = tmp.join("state.vscdb");
            std::fs::copy(db, &dst).map_err(|e2| {
                format!("open {}: {e} (temp copy: {e2})", db.display())
            })?;
            for side in ["wal", "shm"] {
                let src = PathBuf::from(format!("{}-{side}", db.display()));
                if src.is_file() {
                    let _ = std::fs::copy(&src, tmp.join(format!("state.vscdb-{side}")));
                }
            }
            let conn = rusqlite::Connection::open(&dst).map_err(|e2| {
                format!("open {}: {e} (temp copy: {e2})", db.display())
            })?;
            let _ = conn.busy_timeout(std::time::Duration::from_secs(3));
            Ok((conn, Some(tmp)))
        }
    }
}

/// What Desktop is currently signed in as — the first entry of the
/// decrypted sessions secret. Read-only; safe while Desktop runs.
#[derive(Clone)]
pub struct DesktopSession {
    pub token: String,
    pub label: String,
    pub user_id: String,
}

pub fn current_session() -> Result<DesktopSession, String> {
    let db = db_path().ok_or("Devin Desktop data dir not found")?;
    let key = vault_key()?;
    let (conn, tmp) = open_state_db(&db)?;
    let res = session_from(&conn, &key);
    drop(conn);
    if let Some(d) = tmp {
        let _ = std::fs::remove_dir_all(d);
    }
    res
}

fn session_from(
    conn: &rusqlite::Connection,
    key: &[u8; 32],
) -> Result<DesktopSession, String> {
    let raw: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key=?1",
            [SESSIONS_KEY],
            |r| r.get(0),
        )
        .map_err(|_| "no Desktop session stored".to_string())?;
    let v: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("sessions row unreadable: {e}"))?;
    let bytes: Vec<u8> = v
        .get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_u64().map(|n| n as u8)).collect())
        .ok_or("sessions row isn't a Buffer")?;
    let pt = open(key, &bytes)?;
    let arr: Vec<serde_json::Value> =
        serde_json::from_slice(&pt).map_err(|e| format!("sessions JSON bad: {e}"))?;
    let s = arr.first().ok_or("Desktop has no session")?;
    Ok(DesktopSession {
        token: s
            .get("accessToken")
            .and_then(|t| t.as_str())
            .ok_or("session has no accessToken")?
            .to_string(),
        label: s
            .pointer("/account/label")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        user_id: s
            .pointer("/account/id")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

/// Reverse of `seal`: strip `v10`, AES-256-GCM open.
fn open(key: &[u8; 32], bytes: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let body = bytes
        .strip_prefix(b"v10")
        .ok_or("secret isn't a v10 blob")?;
    if body.len() < 12 + 16 {
        return Err("secret blob too short".into());
    }
    Aes256Gcm::new(key.into())
        .decrypt(Nonce::from_slice(&body[..12]), &body[12..])
        .map_err(|e| format!("secret won't decrypt: {e}"))
}

/// `v10` + AES-256-GCM(nonce, plaintext), nonce freshly random.
fn seal(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::aead::{Aead, KeyInit, OsRng};
    use aes_gcm::{AeadCore, Aes256Gcm};

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = Aes256Gcm::new(key.into())
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("seal: {e}"))?;
    let mut out = Vec::with_capacity(3 + 12 + ct.len());
    out.extend_from_slice(b"v10");
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Wrap raw bytes as Electron's stored `{"type":"Buffer","data":[...]}`.
fn buffer_json(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 4 + 32);
    s.push_str(r#"{"type":"Buffer","data":["#);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&b.to_string());
    }
    s.push_str("]}");
    s
}

fn uuid4() -> String {
    let mut b = [0u8; 16];
    let _ = getrandom::getrandom(&mut b);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

/// The 32-byte AES key hiding in `Local State`'s DPAPI-wrapped os_crypt.
#[cfg(windows)]
fn vault_key() -> Result<[u8; 32], String> {
    use base64::Engine;
    let ls_path = userdata_dir()
        .map(|d| d.join("Local State"))
        .ok_or("no Desktop dir")?;
    let ls: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&ls_path).map_err(|e| format!("read Local State: {e}"))?,
    )
    .map_err(|e| format!("parse Local State: {e}"))?;
    let b64 = ls
        .pointer("/os_crypt/encrypted_key")
        .and_then(|v| v.as_str())
        .ok_or("Local State has no os_crypt key")?;
    let wrapped = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("os_crypt key not base64: {e}"))?;
    let blob = wrapped
        .strip_prefix(b"DPAPI")
        .ok_or("unexpected os_crypt key prefix")?;
    let key = dpapi_unprotect(blob)?;
    key.try_into()
        .map_err(|_| "os_crypt key isn't 32 bytes".to_string())
}

#[cfg(not(windows))]
fn vault_key() -> Result<[u8; 32], String> {
    Err("Devin Desktop switching is only supported on Windows".into())
}

/// CryptUnprotectData — same-user DPAPI unwrap, no elevated entropy needed.
#[cfg(windows)]
fn dpapi_unprotect(data: &[u8]) -> Result<Vec<u8>, String> {
    #[repr(C)]
    struct Blob {
        cb: u32,
        pb: *mut u8,
    }
    extern "system" {
        fn CryptUnprotectData(
            input: *const Blob,
            desc: *mut *mut u16,
            entropy: *const Blob,
            reserved: *mut std::ffi::c_void,
            prompt: *mut std::ffi::c_void,
            flags: u32,
            out: *mut Blob,
        ) -> i32;
        fn LocalFree(p: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }
    let mut inbuf = data.to_vec();
    let input = Blob {
        cb: inbuf.len() as u32,
        pb: inbuf.as_mut_ptr(),
    };
    let mut out = Blob {
        cb: 0,
        pb: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            &mut out,
        )
    };
    if ok == 0 {
        return Err(format!("CryptUnprotectData: {}", std::io::Error::last_os_error()));
    }
    let v = unsafe { std::slice::from_raw_parts(out.pb, out.cb as usize).to_vec() };
    unsafe { LocalFree(out.pb.cast()) };
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_json_roundtrip_shape() {
        let s = buffer_json(b"v10abc");
        assert_eq!(s, r#"{"type":"Buffer","data":[118,49,48,97,98,99]}"#);
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["data"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn uuid_shape() {
        let u = uuid4();
        assert_eq!(u.len(), 36);
        assert_eq!(&u[14..15], "4");
    }

    #[test]
    fn seal_produces_v10_blob() {
        let key = [7u8; 32];
        let sealed = seal(&key, b"hello").unwrap();
        assert_eq!(&sealed[..3], b"v10");
        assert!(sealed.len() > 3 + 12 + 5 + 16 - 1);
    }

    #[test]
    fn seal_then_open() {
        use aes_gcm::aead::{Aead, KeyInit};
        use aes_gcm::{Aes256Gcm, Nonce};
        let key = [3u8; 32];
        let sealed = seal(&key, b"secret-payload").unwrap();
        let nonce = Nonce::from_slice(&sealed[3..15]);
        let pt = Aes256Gcm::new((&key).into())
            .decrypt(nonce, &sealed[15..])
            .unwrap();
        assert_eq!(pt, b"secret-payload");
    }
}
