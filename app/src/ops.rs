use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

use serde::Serialize;

use crate::config;
use crate::desktop;
use crate::devincli;
use crate::login::{self, PollState};
use crate::model::{now_stamp, AuthStatus, Meta, ProfileInfo, UseResult};
use crate::paths;
use crate::store;
use crate::usage::{self, Usage};

#[derive(Debug, Serialize)]
pub struct Status {
    pub auth: AuthStatus,
    pub usage: Option<Usage>,
    /// Devin Desktop's own sign-in — independent store from the CLI's.
    /// None when Desktop isn't installed or holds no session.
    pub desktop: Option<DesktopNow>,
    pub running: Vec<String>,
    pub devin_installed: bool,
    pub credentials_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopNow {
    pub logged_in: bool,
    /// Display name straight from the session record.
    pub label: String,
    pub user_id: String,
    /// Filled by the cached quota fetch — may lag a minute.
    pub email: String,
    pub name: String,
    pub usage: Option<Usage>,
}

/// `devin auth status` spawns the CLI — seconds on Windows — so UI polling
/// caches the parsed result keyed by the credentials file's (mtime, len).
/// Any write (switch, external login, delete) changes the key and forces a
/// fresh check.
static AUTH_CACHE: Mutex<Option<(Option<(SystemTime, u64)>, AuthStatus)>> =
    Mutex::new(None);

fn auth_status_cached() -> AuthStatus {
    let key = fs::metadata(paths::credentials_path())
        .and_then(|m| m.modified().map(|t| (t, m.len())))
        .ok();
    let mut g = AUTH_CACHE.lock().unwrap();
    if let Some((k, st)) = g.as_ref() {
        if *k == key {
            return st.clone();
        }
    }
    let st = devincli::auth_status(None).unwrap_or_default();
    *g = Some((key, st.clone()));
    st
}

/// Desktop's session, cached on state.vscdb's (mtime, len) — the decrypt
/// is cheap but polling every 5s shouldn't redo it, and a transient
/// lock while Desktop writes shouldn't flicker the banner.
static DESK_CACHE: Mutex<Option<(Option<(SystemTime, u64)>, Option<DesktopNow>)>> =
    Mutex::new(None);

fn desktop_now() -> Option<DesktopNow> {
    let db = desktop::db_path()?;
    let key = fs::metadata(&db)
        .and_then(|m| m.modified().map(|t| (t, m.len())))
        .ok();
    let mut g = DESK_CACHE.lock().unwrap();
    if let Some((k, d)) = g.as_ref() {
        if *k == key {
            return d.clone();
        }
    }
    let out = Some(match desktop::current_session() {
        Ok(s) => {
            let creds = login::credentials_toml(
                &s.token,
                "https://server.codeium.com",
                "app.devin.ai",
                "https://api.devin.ai",
            )
            .into_bytes();
            let u = usage::user_status_cached(&creds).ok();
            DesktopNow {
                logged_in: true,
                label: s.label,
                user_id: s.user_id,
                email: u.as_ref().map(|x| x.email.clone()).unwrap_or_default(),
                name: u.as_ref().map(|x| x.name.clone()).unwrap_or_default(),
                usage: u,
            }
        }
        Err(_) => DesktopNow {
            logged_in: false,
            label: String::new(),
            user_id: String::new(),
            email: String::new(),
            name: String::new(),
            usage: None,
        },
    });
    *g = Some((key, out.clone()));
    out
}

pub fn status() -> Status {
    let usage = fs::read(paths::credentials_path())
        .ok()
        .and_then(|c| usage::user_status_cached(&c).ok());
    Status {
        auth: auth_status_cached(),
        usage,
        desktop: desktop::installed().then(desktop_now).flatten(),
        running: running_devin_processes(),
        devin_installed: paths::devin_bin().is_some(),
        credentials_path: paths::credentials_path().display().to_string(),
    }
}

/// Kick a background quota refresh for profiles whose snapshot is stale
/// (>5min) — the next poll shows fresh bars. In-flight and recently
/// attempted names are skipped so a dead network can't multiply calls.
static REFRESHING: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
static REFRESH_TRIED: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

pub fn auto_refresh_stale() {
    let now = unix_now();
    for p in store::list() {
        if now - p.meta.usage_at <= 300 {
            continue;
        }
        {
            let mut tried = REFRESH_TRIED.lock().unwrap();
            let map = tried.get_or_insert_with(HashMap::new);
            if map
                .get(&p.name)
                .map(|t| t.elapsed() < Duration::from_secs(300))
                .unwrap_or(false)
            {
                continue;
            }
            map.insert(p.name.clone(), Instant::now());
        }
        {
            let mut inflight = REFRESHING.lock().unwrap();
            if !inflight.insert(p.name.clone()) {
                continue;
            }
        }
        let name = p.name.clone();
        std::thread::spawn(move || {
            let _ = refresh_usage(&name);
            REFRESHING.lock().unwrap().remove(&name);
        });
    }
}

/// "Refresh all" — every profile's quota, fetched in parallel. Blocks
/// until done so the caller's next list shows fresh bars.
pub fn refresh_all() {
    let handles: Vec<_> = store::list()
        .into_iter()
        .map(|p| {
            let name = p.name.clone();
            std::thread::spawn(move || {
                let _ = refresh_usage(&name);
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
}

/// Append an error to %APPDATA%/devin-switch/errors.log — release builds
/// have no devtools, and a toast the user couldn't read in time is
/// otherwise unrecoverable. Best-effort; never fails the caller.
pub fn log_error(ctx: &str, msg: &str) {
    let dir = paths::vault_dir();
    let _ = fs::create_dir_all(&dir);
    let line = format!("{} [{}] {}\n", unix_now(), ctx, msg);
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("errors.log"))
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Identity+quota for a credentials blob; empty Usage when unreachable.
fn account_of(creds: &[u8]) -> Usage {
    usage::user_status_cached(creds).unwrap_or_default()
}

fn meta_from(u: Usage, org: Option<String>, note: &str) -> Meta {
    let ok = !u.email.is_empty() || !u.plan.is_empty();
    Meta {
        email: u.email.clone(),
        display_name: u.name.clone(),
        user_id: u.user_id.clone(),
        plan: u.plan.clone(),
        org_id: u.org_id.clone().or(org),
        note: note.to_string(),
        created_at: now_stamp(),
        usage: ok.then_some(u),
        usage_at: if ok { unix_now() } else { 0 },
    }
}

/// devin CLI sessions and Devin Desktop processes currently running — i.e.
/// what would keep using the old account until restarted. Sessions launched
/// under our fake homes (parallel mode) are excluded.
pub fn running_devin_processes() -> Vec<String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let mut seen = std::collections::BTreeMap::<String, u32>::new();
    #[cfg(unix)]
    let run_root = paths::run_dir();
    for p in sys.processes().values() {
        let name = p.name().to_lowercase();
        let base = name.trim_end_matches(".exe");
        let is_devin = base == "devin"
            || base == "devin desktop"
            || base.starts_with("devin helper")
            || base == "devin.exe";
        if !is_devin || base.contains("devin-switch") {
            continue;
        }
        #[cfg(unix)]
        {
            let isolated = p.environ().iter().any(|e| {
                e.starts_with("XDG_DATA_HOME=") && e.contains(&run_root.display().to_string())
            });
            if isolated {
                continue;
            }
        }
        *seen.entry(p.name().to_string()).or_default() += 1;
    }
    seen.into_iter()
        .map(|(n, c)| if c > 1 { format!("{n} ×{c}") } else { n })
        .collect()
}

/// Profile name from the account's identity: email local part, else
/// display name, else "account". Empty `want` means auto.
fn pick_name(want: &str, u: &Usage) -> Result<String, String> {
    if !want.trim().is_empty() {
        return store::sanitize_name(want);
    }
    let hint = u
        .email
        .split('@')
        .next()
        .filter(|s| !s.is_empty())
        .or(if u.name.is_empty() {
            None
        } else {
            Some(u.name.as_str())
        })
        .map(str::to_string)
        .unwrap_or_else(|| "account".into());
    Ok(store::unique_name(&hint))
}

/// The vault already holds this account: same non-empty email, or
/// byte-identical credentials.
fn find_existing(creds: &[u8], u: &Usage) -> Option<String> {
    for p in store::list() {
        if !u.email.is_empty() && p.meta.email == u.email {
            return Some(p.name);
        }
        if store::creds_of(&p.name)
            .map(|c| c == creds)
            .unwrap_or(false)
        {
            return Some(p.name);
        }
    }
    None
}

/// Shared save path for every add-account flow. With an explicit `want`
/// name we write exactly that profile; on auto-name, adding the same
/// account again refreshes the existing profile's credentials and meta
/// (keeping created_at/note) instead of minting a `-2` duplicate.
fn save_account(
    creds: &[u8],
    u: Usage,
    org: Option<String>,
    want: &str,
    note: &str,
) -> Result<ProfileInfo, String> {
    let name = if want.trim().is_empty() {
        match find_existing(creds, &u) {
            Some(existing) => existing,
            None => pick_name("", &u)?,
        }
    } else {
        store::sanitize_name(want)?
    };
    let mut m = meta_from(u, org, note);
    if let Some(old) = store::list().into_iter().find(|p| p.name == name) {
        m.created_at = old.meta.created_at;
        if m.note.is_empty() {
            m.note = old.meta.note;
        }
    }
    store::save(&name, creds, &m)
}

/// Save whatever the CLI is currently signed in as.
pub fn save_current(name: &str, note: &str) -> Result<ProfileInfo, String> {
    let creds =
        fs::read(paths::credentials_path()).map_err(|_| "Devin isn't signed in".to_string())?;
    save_account(
        &creds,
        account_of(&creds),
        config::current_org_id(),
        name,
        note,
    )
}

/// Finish a login round (PKCE callback or CLI-in-fake-home fallback).
pub fn finish_login(id: u64, name: &str, note: &str) -> Result<LoginOutcome, String> {
    let st = login::poll(id).map_err(|e| {
        eprintln!("[dsw] login poll error: {e}");
        e
    })?;
    match st {
        PollState::Waiting => Ok(LoginOutcome::Waiting),
        PollState::Working => Ok(LoginOutcome::Working),
        PollState::Failed { error } => {
            eprintln!("[dsw] login failed: {error}");
            Err(error)
        }
        PollState::GotCreds { creds, home } => {
            let res = save_account(&creds, account_of(&creds), None, name, note);
            if let Err(e) = &res {
                eprintln!("[dsw] save after login failed: {e}");
            }
            login::cleanup_home(&home);
            res.map(|profile| LoginOutcome::Done { profile })
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LoginOutcome {
    Waiting,
    /// Callback landed; token exchange / account fetch still running.
    Working,
    Done { profile: ProfileInfo },
}

/// Paste-token path: wrap a raw session token into credentials.toml, verify
/// it against GetUserStatus, then save. Unverifiable tokens aren't saved.
pub fn add_token(name: &str, token: &str) -> Result<ProfileInfo, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("token is empty".into());
    }
    add_token_named(name, token)
}

/// Save credentials produced out-of-band (manual code exchange in the
/// CLI). Identity + quota are fetched as usual.
pub fn save_creds(creds: &[u8], name: &str) -> Result<ProfileInfo, String> {
    save_account(creds, account_of(creds), None, name, "")
}

/// Import whatever Devin Desktop is signed in as — decrypts its stored
/// session token, verifies it like a pasted token, then saves. Works
/// while Desktop is running (read-only DB access).
pub fn import_desktop(name: &str) -> Result<ProfileInfo, String> {
    let s = desktop::current_session()?;
    add_token_named(name, &s.token)
}

fn add_token_named(name: &str, token: &str) -> Result<ProfileInfo, String> {
    let creds = login::credentials_toml(
        token,
        "https://server.codeium.com",
        "app.devin.ai",
        "https://api.devin.ai",
    )
    .into_bytes();
    let (key, server) = usage::creds_secrets(&creds)?;
    let u = usage::user_status(&key, &server).map_err(|e| format!("token not recognized: {e}"))?;
    if u.email.is_empty() && u.name.is_empty() {
        return Err("token not recognized: empty user status".into());
    }
    save_account(&creds, u, None, name, "")
}

/// Switch to a stored profile. `scope`: "all" (CLI + Desktop), "cli",
/// or "desktop". When Desktop is part of the scope it must be fully
/// closed — its in-memory copy of state.vscdb wins over a live write,
/// so we refuse rather than half-switch.
pub fn use_profile(name: &str, scope: &str) -> Result<UseResult, String> {
    let name = store::sanitize_name(name)?;
    let creds = store::creds_of(&name)?;
    let meta = store::list()
        .into_iter()
        .find(|p| p.name == name)
        .map(|p| p.meta)
        .unwrap_or_default();
    let want_cli = scope != "desktop";
    let want_desktop = scope != "cli";

    if want_desktop && desktop::is_running() {
        return Err("devin_desktop_running".into());
    }

    // Fresh quota + the account's own org_id, while we still can (a dead
    // network or unparseable profile shouldn't block the swap itself).
    // Also the source of user_id for Desktop's session record.
    let u = usage::creds_secrets(&creds)
        .and_then(|(k, s)| usage::user_status(&k, &s))
        .ok();
    let org = u
        .as_ref()
        .and_then(|x| x.org_id.clone())
        .or(meta.org_id.clone());

    if want_cli {
        let target = paths::credentials_path();
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let tmp = target.with_extension("tmp");
        fs::write(&tmp, &creds).map_err(|e| format!("write credentials: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
        }
        fs::rename(&tmp, &target).map_err(|e| format!("rename credentials: {e}"))?;

        config::apply_org_id(org.as_deref())?;
    }

    // The account we just wrote IS the live sign-in — report it from the
    // fresh fetch (or stored meta) instead of spawning `devin auth
    // status`, which costs seconds on Windows.
    let auth = AuthStatus {
        logged_in: true,
        name: u
            .as_ref()
            .map(|x| x.name.clone())
            .unwrap_or_else(|| meta.display_name.clone()),
        email: u
            .as_ref()
            .map(|x| x.email.clone())
            .unwrap_or_else(|| meta.email.clone()),
        plan: u
            .as_ref()
            .map(|x| x.plan.clone())
            .unwrap_or_else(|| meta.plan.clone()),
        ..Default::default()
    };
    // Refresh the stored meta from whatever we just learned.
    if let Some(u) = &u {
        if !u.email.is_empty() || !u.plan.is_empty() {
            let mut m = meta_from(u.clone(), org.clone(), &meta.note);
            m.created_at = meta.created_at.clone();
            let _ = store::save(&name, &creds, &m);
        }
    }

    let desktop = if !want_desktop {
        "cli_only".to_string()
    } else if !desktop::installed() {
        "unavailable".to_string()
    } else {
        let token = usage::creds_secrets(&creds)
            .map(|(k, _)| k)
            .unwrap_or_default();
        let pick = |f: fn(&usage::Usage) -> String, fb: String| {
            u.as_ref()
                .map(f)
                .filter(|s| !s.is_empty())
                .unwrap_or(fb)
        };
        let label = pick(|x| x.name.clone(), meta.display_name.clone());
        let uid = pick(|x| x.user_id.clone(), meta.user_id.clone());
        let email = pick(|x| x.email.clone(), meta.email.clone());
        match desktop::switch_session(&token, &label, &uid, &email) {
            Ok(()) => "switched".to_string(),
            Err(e) => format!("failed: {e}"),
        }
    };

    Ok(UseResult {
        auth,
        restart_needed: if want_cli {
            running_devin_processes()
        } else {
            Vec::new()
        },
        desktop,
    })
}

/// Force-refresh a profile's cached quota (called by the UI's refresh
/// button; skips the 60s cache).
pub fn refresh_usage(name: &str) -> Result<ProfileInfo, String> {
    let name = store::sanitize_name(name)?;
    let creds = store::creds_of(&name)?;
    let meta = store::list()
        .into_iter()
        .find(|p| p.name == name)
        .map(|p| p.meta)
        .unwrap_or_default();
    let (key, server) = usage::creds_secrets(&creds)?;
    let u = usage::user_status(&key, &server)?;
    let mut m = meta_from(u, meta.org_id, &meta.note);
    m.created_at = meta.created_at;
    store::save(&name, &creds, &m)
}

/// Launch a devin session that runs under an isolated data home, so it can
/// use a different account than the main sign-in — in parallel.
pub fn launch_parallel(name: &str, cwd: Option<&str>) -> Result<ParallelLaunch, String> {
    let name = store::sanitize_name(name)?;
    let creds = store::creds_of(&name)?;
    let home = paths::run_dir().join(format!("par-{name}"));
    let fake_devin = home.join("devin");
    if fake_devin.exists() {
        fs::remove_dir_all(&fake_devin).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&fake_devin).map_err(|e| e.to_string())?;

    // Mirror the real data dir: everything EXCEPT credentials.toml becomes
    // a symlink (binary versions, plugins, sessions db stay shared).
    let real = paths::devin_data_dir();
    if real.is_dir() {
        for e in fs::read_dir(&real).map_err(|e| e.to_string())?.flatten() {
            if e.file_name() == "credentials.toml" {
                continue;
            }
            let link = fake_devin.join(e.file_name());
            link_to(&e.path(), &link)
                .map_err(|err| format!("link {}: {err}", e.path().display()))?;
        }
    }
    fs::write(fake_devin.join("credentials.toml"), &creds)
        .map_err(|e| format!("write isolated credentials: {e}"))?;

    let dir = cwd
        .filter(|c| !c.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home());
    let cmdline = shell_line(&home);
    match open_terminal(&cmdline, &dir) {
        Ok(()) => Ok(ParallelLaunch {
            launched: true,
            command: cmdline,
        }),
        Err(_) => Ok(ParallelLaunch {
            launched: false,
            command: format!("cd \"{}\" && {cmdline}", dir.display()),
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct ParallelLaunch {
    /// True when a terminal window was opened; false → run `command`
    /// manually (also used on platforms without a known terminal).
    pub launched: bool,
    pub command: String,
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn shell_line(home: &Path) -> String {
    if cfg!(windows) {
        format!(
            "set \"APPDATA={}\" && set \"XDG_DATA_HOME={0}\" && devin",
            home.display()
        )
    } else {
        format!("XDG_DATA_HOME=\"{}\" devin", home.display())
    }
}

#[cfg(unix)]
fn link_to(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn link_to(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::os::windows::fs::symlink_dir(src, dst)
    } else {
        std::os::windows::fs::symlink_file(src, dst)
    }
}

#[cfg(not(any(unix, windows)))]
fn link_to(_src: &Path, _dst: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "symlinks unsupported",
    ))
}

#[cfg(target_os = "macos")]
fn open_terminal(cmdline: &str, cwd: &Path) -> Result<(), String> {
    let script = format!(
        "tell application \"Terminal\" to do script \"cd \\\"{}\\\" && {}\"",
        cwd.display(),
        cmdline.replace('"', "\\\"")
    );
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(windows)]
fn open_terminal(cmdline: &str, cwd: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    Command::new("cmd")
        .args(["/c", "start", "", "cmd", "/k"])
        .arg(format!("cd /d \"{}\" && {}", cwd.display(), cmdline))
        .creation_flags(0x0800_0000)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_terminal(cmdline: &str, cwd: &Path) -> Result<(), String> {
    let full = format!("cd \"{}\" && {}; exec $SHELL", cwd.display(), cmdline);
    for term in ["x-terminal-emulator", "gnome-terminal", "konsole", "xterm"] {
        let args: Vec<&str> = match term {
            "gnome-terminal" => vec!["--", "bash", "-c", &full],
            "konsole" => vec!["-e", "bash", "-c", &full],
            _ => vec!["-e", "bash", "-c", &full],
        };
        if Command::new(term).args(&args).spawn().is_ok() {
            return Ok(());
        }
    }
    Err("no known terminal emulator found".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end switch mechanics against sandboxed dirs — never touches
    /// the real sign-in. Windows reads APPDATA, unix reads XDG_* (the same
    /// override devin_cmd applies), so all of them point at a temp dir and
    /// "real" paths are resolved through paths:: itself. Single test fn so
    /// env mutation can't race.
    #[test]
    fn switch_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("dsw-test-{}", std::process::id()));
        let prev: Vec<(&'static str, Option<String>)> =
            ["XDG_DATA_HOME", "XDG_CONFIG_HOME", "APPDATA"]
                .iter()
                .map(|&k| (k, std::env::var(k).ok()))
                .collect();
        std::env::set_var("XDG_DATA_HOME", tmp.join("data"));
        std::env::set_var("XDG_CONFIG_HOME", tmp.join("cfg"));
        std::env::set_var("APPDATA", tmp.join("appdata"));

        // current sign-in "old", a stored account "a" with different creds
        let real_creds = paths::credentials_path();
        fs::create_dir_all(real_creds.parent().unwrap()).unwrap();
        fs::write(&real_creds, b"old-creds").unwrap();
        let real_cfg = paths::config_path();
        fs::create_dir_all(real_cfg.parent().unwrap()).unwrap();
        fs::write(
            &real_cfg,
            br#"{"devin":{"org_id":"org-old"},"agent":{}}"#,
        )
        .unwrap();
        store::save(
            "a",
            b"creds-a",
            &Meta {
                email: "a@example.com".into(),
                org_id: Some("org-a".into()),
                ..Meta::default()
            },
        )
        .unwrap();

        // switch
        let r = use_profile("a", "cli").unwrap();
        assert_eq!(fs::read(&real_creds).unwrap(), b"creds-a");
        let cfg: serde_json::Value =
            serde_json::from_slice(&fs::read(&real_cfg).unwrap()).unwrap();
        assert_eq!(cfg["devin"]["org_id"], "org-a");
        assert!(cfg.get("agent").is_some(), "other keys preserved");
        assert!(
            store::list()
                .iter()
                .find(|p| p.name == "a")
                .unwrap()
                .is_active
        );
        let _ = r; // auth may be empty without a devin-signed fake home

        // rename keeps the account active (byte-compare, not name tracking)
        store::rename("a", "a-renamed").unwrap();
        assert!(
            store::list()
                .iter()
                .find(|p| p.name == "a-renamed")
                .unwrap()
                .is_active
        );
        assert!(store::rename("a-renamed", "a-renamed").is_ok());
        assert!(store::rename("missing", "x").is_err());

        // saving the same sign-in twice refreshes instead of duplicating:
        // current creds == a-renamed's creds, so it lands back there
        save_current("", "").unwrap();
        let p = save_current("", "").unwrap();
        assert_eq!(p.name, "a-renamed");
        assert_eq!(store::list().len(), 1, "no -2 duplicate");

        // an account with no org recorded clears the key entirely
        store::save("b", b"creds-b", &Meta::default()).unwrap();
        use_profile("b", "cli").unwrap();
        let cfg: serde_json::Value =
            serde_json::from_slice(&fs::read(&real_cfg).unwrap()).unwrap();
        assert!(cfg["devin"].get("org_id").is_none());

        for (k, v) in prev {
            match v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}
