use std::path::Path;
use std::process::{Command, Stdio};

use crate::model::AuthStatus;
use crate::paths;

/// Run `devin` with an optional fake data home (the env override the CLI
/// itself honors: XDG_DATA_HOME on unix, APPDATA on windows).
pub fn devin_cmd(home: Option<&Path>, args: &[&str]) -> Result<Command, String> {
    let bin = paths::devin_bin().ok_or(
        "devin CLI not found on PATH or ~/.local/bin — install Devin first",
    )?;
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(h) = home {
        // Both are set on every platform: the CLI reads whichever applies.
        cmd.env("XDG_DATA_HOME", h).env("APPDATA", h);
    }
    Ok(cmd)
}

/// `devin auth status` — under `home` when given, else the real sign-in.
pub fn auth_status(home: Option<&Path>) -> Result<AuthStatus, String> {
    let out = devin_cmd(home, &["auth", "status"])
        .and_then(|mut c| c.output().map_err(|e| e.to_string()))?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(parse_status(&text))
}

pub fn parse_status(text: &str) -> AuthStatus {
    let mut st = AuthStatus::default();
    if text.contains("Not logged in") {
        return st;
    }
    for line in text.lines() {
        let line = line.trim();
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim().to_string();
        match k.trim() {
            "Name" => st.name = v,
            "Email" => st.email = v,
            "Plan" => st.plan = v,
            "Tier" => st.tier = v,
            _ => {}
        }
    }
    st.logged_in = !st.email.is_empty() || !st.name.is_empty();
    st
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Logged in (via Devin).\n\nCredentials:\n  File:              /x/credentials.toml\n\nUser:\n  Name:              Example User\n  Email:             user@example.com\n  User ID:           user-abc\n\nAccount:\n  Tier:              Devin Pro\n  Plan:              Pro\n";

    #[test]
    fn parses_logged_in() {
        let st = parse_status(SAMPLE);
        assert!(st.logged_in);
        assert_eq!(st.email, "user@example.com");
        assert_eq!(st.name, "Example User");
        assert_eq!(st.plan, "Pro");
        assert_eq!(st.tier, "Devin Pro");
    }

    #[test]
    fn parses_logged_out() {
        let st = parse_status("Not logged in.\n  Credentials path: /x\n");
        assert!(!st.logged_in);
    }
}
