use serde::{Deserialize, Serialize};

use crate::usage::Usage;

/// Per-profile metadata kept beside credentials.toml in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Meta {
    pub email: String,
    pub display_name: String,
    pub plan: String,
    /// Last known devin.org_id for this account (from config.json).
    pub org_id: Option<String>,
    pub note: String,
    pub created_at: String,
    /// Cached GetUserStatus quota snapshot + when it was taken (unix secs).
    pub usage: Option<Usage>,
    pub usage_at: i64,
}

/// A profile as the frontend sees it.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileInfo {
    pub name: String,
    #[serde(flatten)]
    pub meta: Meta,
    /// credentials.toml bytes equal the CLI's current file.
    pub is_active: bool,
}

/// Parsed `devin auth status`.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(default)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub name: String,
    pub email: String,
    pub plan: String,
    pub tier: String,
}

/// Result of switching.
#[derive(Debug, Clone, Serialize)]
pub struct UseResult {
    pub auth: AuthStatus,
    /// devin/Desktop processes still running the old sign-in; restart them.
    pub restart_needed: Vec<String>,
}

pub fn now_stamp() -> String {
    // YYYY-MM-DD HH:MM, good enough for a "created" label.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}Z",
        y,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60
    )
}

/// Days since unix epoch → (year, month, day). Public for the CLI's date
/// formatting.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_shape() {
        let s = now_stamp();
        assert_eq!(s.len(), 17, "{}", s);
        assert!(s.ends_with('Z'));
    }
}
