use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{Meta, ProfileInfo};
use crate::paths;

/// Profile names map to directory names — keep them boring on purpose.
pub fn sanitize_name(name: &str) -> Result<String, String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("profile name is empty".into());
    }
    if n.len() > 64 {
        return Err("profile name too long (max 64)".into());
    }
    if !n
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'))
    {
        return Err("profile name may only contain letters, digits, - _ . @".into());
    }
    Ok(n.to_string())
}

fn creds_file(dir: &Path) -> PathBuf {
    dir.join("credentials.toml")
}

fn meta_file(dir: &Path) -> PathBuf {
    dir.join("meta.json")
}

/// Write a file so a crash can't leave a torn credentials.toml behind.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    fs::rename(&tmp, path).map_err(|e| format!("rename to {}: {e}", path.display()))
}

pub fn save(name: &str, creds: &[u8], meta: &Meta) -> Result<ProfileInfo, String> {
    let name = sanitize_name(name)?;
    let dir = paths::profile_dir(&name);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    write_atomic(&creds_file(&dir), creds)?;
    write_atomic(
        &meta_file(&dir),
        serde_json::to_string_pretty(meta)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    Ok(info(&name))
}

pub fn remove(name: &str) -> Result<(), String> {
    let name = sanitize_name(name)?;
    let dir = paths::profile_dir(&name);
    if !dir.exists() {
        return Err(format!("no such profile: {name}"));
    }
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())
}

pub fn creds_of(name: &str) -> Result<Vec<u8>, String> {
    fs::read(creds_file(&paths::profile_dir(&sanitize_name(name)?)))
        .map_err(|e| format!("read profile credentials: {e}"))
}

fn meta_of(dir: &Path) -> Meta {
    fs::read(meta_file(dir))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn info(name: &str) -> ProfileInfo {
    let dir = paths::profile_dir(name);
    let meta = meta_of(&dir);
    let active = match (fs::read(creds_file(&dir)), fs::read(paths::credentials_path())) {
        (Ok(saved), Ok(current)) => saved == current,
        _ => false,
    };
    ProfileInfo {
        name: name.to_string(),
        meta,
        is_active: active,
    }
}

/// All profiles, active-flagged by byte-comparing saved credentials against
/// the CLI's current file (no marker file that could desync).
pub fn list() -> Vec<ProfileInfo> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(paths::profiles_dir()) else {
        return out;
    };
    for e in rd.flatten() {
        if !e.path().is_dir() {
            continue;
        }
        let Some(name) = e.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !creds_file(&e.path()).exists() {
            continue;
        }
        out.push(info(&name));
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_rules() {
        assert!(sanitize_name("work@x.com").is_ok());
        assert!(sanitize_name("a-b_c.d").is_ok());
        assert!(sanitize_name("../evil").is_err());
        assert!(sanitize_name("").is_err());
        assert!(sanitize_name("a/b").is_err());
    }
}
