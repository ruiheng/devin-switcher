use std::env;
use std::path::PathBuf;

// Devin data dir layout (verified against the installed CLI and magpie's
// provider/devin.go):
//   unix:    $XDG_DATA_HOME/devin  (default ~/.local/share/devin)
//   windows: %APPDATA%/devin
// credentials.toml and config.json live directly inside it.
// Devin Desktop's userData is ~/.devin on every platform.

fn env_nonempty(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

fn home() -> PathBuf {
    env_nonempty("HOME")
        .or_else(|| env_nonempty("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Devin's data root (contains credentials.toml), honoring XDG on unix and
/// APPDATA on windows — the same override the CLI itself uses.
pub fn devin_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(app) = env_nonempty("APPDATA") {
            return PathBuf::from(app).join("devin");
        }
        return home().join("AppData").join("Roaming").join("devin");
    }
    #[cfg(not(windows))]
    {
        let base = env_nonempty("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".local").join("share"));
        base.join("devin")
    }
}

pub fn credentials_path() -> PathBuf {
    devin_data_dir().join("credentials.toml")
}

/// Devin CLI config (~/.config/devin/config.json on unix; %APPDATA%\devin
/// \config.json on windows, per magpie's agent/devin.go).
pub fn config_path() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(app) = env_nonempty("APPDATA") {
            return PathBuf::from(app).join("devin").join("config.json");
        }
        return home()
            .join("AppData")
            .join("Roaming")
            .join("devin")
            .join("config.json");
    }
    #[cfg(not(windows))]
    {
        let base = env_nonempty("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".config"));
        base.join("devin").join("config.json")
    }
}

/// Our own vault. Same convention as devin's data dir so everything stays
/// together; holds profiles/ and run/ subdirectories.
pub fn vault_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(app) = env_nonempty("APPDATA") {
            return PathBuf::from(app).join("devin-switch");
        }
        return home().join("AppData").join("Roaming").join("devin-switch");
    }
    #[cfg(not(windows))]
    {
        let base = env_nonempty("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".local").join("share"));
        base.join("devin-switch")
    }
}

pub fn profiles_dir() -> PathBuf {
    vault_dir().join("profiles")
}

pub fn profile_dir(name: &str) -> PathBuf {
    profiles_dir().join(name)
}

/// Transient fake homes for login flows and parallel launches.
pub fn run_dir() -> PathBuf {
    vault_dir().join("run")
}

/// Locate the devin executable: PATH first, then the usual shim location.
pub fn devin_bin() -> Option<PathBuf> {
    let exe = if cfg!(windows) { "devin.exe" } else { "devin" };
    if let Some(path) = env_nonempty("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir in path.split(sep) {
            let cand = PathBuf::from(dir).join(exe);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    let shim = home().join(".local").join("bin").join(exe);
    if shim.is_file() {
        return Some(shim);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creds_under_data_dir() {
        // env-independent checks only: other tests mutate XDG vars.
        assert_eq!(credentials_path().file_name().unwrap(), "credentials.toml");
        assert!(credentials_path().parent().unwrap().ends_with("devin"));
        assert!(vault_dir().ends_with("devin-switch"));
    }
}
