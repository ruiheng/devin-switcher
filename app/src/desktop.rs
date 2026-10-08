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

/// Electron userData dir for Devin Desktop (`Devin`, capital D — the CLI
/// uses lowercase `devin`; same directory on case-insensitive filesystems,
/// but the Desktop paths below only exist once Desktop has run).
pub fn userdata_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    #[cfg(windows)]
    {
        if let Ok(app) = std::env::var("APPDATA") {
            if !app.is_empty() {
                return Some(PathBuf::from(app).join("Devin"));
            }
        }
        Some(home.join("AppData").join("Roaming").join("Devin"))
    }
    #[cfg(target_os = "macos")]
    {
        Some(
            home.join("Library")
                .join("Application Support")
                .join("Devin"),
        )
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".config"));
        Some(base.join("Devin"))
    }
}

/// The Electron state DB — exposed so callers can cache by its mtime.
pub fn db_path() -> Option<PathBuf> {
    userdata_dir().map(|d| d.join("User").join("globalStorage").join("state.vscdb"))
}

/// Desktop has been run at least once (its state DB exists).
pub fn installed() -> bool {
    db_path().map(|p| p.is_file()).unwrap_or(false)
}

/// Any Devin Desktop process alive right now. Case-sensitive on "Devin"
/// (the CLI's binary is lowercase `devin`), plus an exe-path check so a
/// renamed binary still counts.
pub fn is_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    sys.processes().values().any(|p| {
        if p.name().starts_with("Devin") {
            return true;
        }
        p.exe()
            .and_then(|e| e.file_name())
            .map(|f| f == "Devin.exe")
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
        .map_err(|e| format!("open state.vscdb: {e}"))?;
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
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("open state.vscdb: {e}"))?;
    // Desktop writes this DB while running — wait out short locks instead
    // of failing with SQLITE_BUSY.
    let _ = conn.busy_timeout(std::time::Duration::from_secs(3));
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
    let pt = open(&key, &bytes)?;
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
