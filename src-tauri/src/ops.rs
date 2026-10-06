use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::config;
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
    pub running: Vec<String>,
    pub devin_installed: bool,
    pub credentials_path: String,
}

pub fn status() -> Status {
    let usage = fs::read(paths::credentials_path())
        .ok()
        .and_then(|c| usage::user_status_cached(&c).ok());
    Status {
        auth: devincli::auth_status(None).unwrap_or_default(),
        usage,
        running: running_devin_processes(),
        devin_installed: paths::devin_bin().is_some(),
        credentials_path: paths::credentials_path().display().to_string(),
    }
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

/// Save whatever the CLI is currently signed in as.
pub fn save_current(name: &str, note: &str) -> Result<ProfileInfo, String> {
    let creds =
        fs::read(paths::credentials_path()).map_err(|_| "Devin isn't signed in".to_string())?;
    store::save(
        name,
        &creds,
        &meta_from(account_of(&creds), config::current_org_id(), note),
    )
}

/// Finish a login round (PKCE callback or CLI-in-fake-home fallback).
pub fn finish_login(id: u64, name: &str, note: &str) -> Result<LoginOutcome, String> {
    match login::poll(id)? {
        PollState::Waiting => Ok(LoginOutcome::Waiting),
        PollState::Failed { error } => Err(error),
        PollState::GotCreds { creds, home } => {
            let res = store::save(name, &creds, &meta_from(account_of(&creds), None, note));
            login::cleanup_home(&home);
            res.map(|profile| LoginOutcome::Done { profile })
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LoginOutcome {
    Waiting,
    Done { profile: ProfileInfo },
}

/// Paste-token path: wrap a raw session token into credentials.toml, verify
/// it against GetUserStatus, then save. Unverifiable tokens aren't saved.
pub fn add_token(name: &str, token: &str) -> Result<ProfileInfo, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("token is empty".into());
    }
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
    store::save(name, &creds, &meta_from(u, None, ""))
}

/// Switch the CLI (and thereby Desktop) to a stored profile.
pub fn use_profile(name: &str) -> Result<UseResult, String> {
    let name = store::sanitize_name(name)?;
    let creds = store::creds_of(&name)?;
    let meta = store::list()
        .into_iter()
        .find(|p| p.name == name)
        .map(|p| p.meta)
        .unwrap_or_default();

    // Fresh quota + the account's own org_id, while we still can (a dead
    // network or unparseable profile shouldn't block the swap itself).
    let u = usage::creds_secrets(&creds)
        .and_then(|(k, s)| usage::user_status(&k, &s))
        .ok();
    let org = u
        .as_ref()
        .and_then(|x| x.org_id.clone())
        .or(meta.org_id.clone());

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

    let auth = devincli::auth_status(None).unwrap_or_default();
    // Refresh the stored meta from whatever we just learned.
    if let Some(u) = u {
        if !u.email.is_empty() || !u.plan.is_empty() {
            let mut m = meta_from(u, org.clone(), &meta.note);
            m.created_at = meta.created_at.clone();
            let _ = store::save(&name, &creds, &m);
        }
    }

    Ok(UseResult {
        auth,
        restart_needed: running_devin_processes(),
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
    Command::new("cmd")
        .args(["/c", "start", "", "cmd", "/k"])
        .arg(format!("cd /d \"{}\" && {}", cwd.display(), cmdline))
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

    /// End-to-end switch mechanics against sandboxed XDG dirs — never
    /// touches the real sign-in. Single test fn so env mutation can't race.
    #[test]
    fn switch_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("dsw-test-{}", std::process::id()));
        let data = tmp.join("data");
        let cfgd = tmp.join("cfg");
        let prev_data = std::env::var("XDG_DATA_HOME").ok();
        let prev_cfg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", &data);
        std::env::set_var("XDG_CONFIG_HOME", &cfgd);

        // current sign-in "old", a stored profile "a" with different creds
        let real_creds = data.join("devin").join("credentials.toml");
        fs::create_dir_all(real_creds.parent().unwrap()).unwrap();
        fs::write(&real_creds, b"old-creds").unwrap();
        fs::create_dir_all(&cfgd.join("devin")).unwrap();
        fs::write(
            cfgd.join("devin").join("config.json"),
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
        let r = use_profile("a").unwrap();
        assert_eq!(fs::read(&real_creds).unwrap(), b"creds-a");
        let cfg: serde_json::Value =
            serde_json::from_slice(&fs::read(cfgd.join("devin").join("config.json")).unwrap())
                .unwrap();
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

        // a profile with no org recorded clears the key entirely
        store::save("b", b"creds-b", &Meta::default()).unwrap();
        use_profile("b").unwrap();
        let cfg: serde_json::Value =
            serde_json::from_slice(&fs::read(cfgd.join("devin").join("config.json")).unwrap())
                .unwrap();
        assert!(cfg["devin"].get("org_id").is_none());

        match prev_data {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match prev_cfg {
            Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}
