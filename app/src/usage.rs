use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

// GetUserStatus is the same Connect-RPC call the CLI's /usage reads
// (see magpie internal/provider/devin_usage.go). JSON over Connect needs
// the Connect-Protocol-Version header; the api key rides in metadata.apiKey.
const METHOD: &str = "/exa.seat_management_pb.SeatManagementService/GetUserStatus";
const CLI_VERSION: &str = "3000.11.3";
const CACHE_TTL: Duration = Duration::from_secs(60);

/// Identity + quota as the account's plan reports them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub email: String,
    pub name: String,
    /// `user-<hex>` — needed for Devin Desktop's session record.
    pub user_id: String,
    pub plan: String,
    pub org_id: Option<String>,
    /// Billing-cycle end, RFC3339.
    pub plan_end: Option<String>,
    /// Percent of the daily/weekly quota still remaining (0–100), if the
    /// plan exposes that window.
    pub daily_left: Option<f64>,
    pub daily_reset_unix: Option<i64>,
    pub weekly_left: Option<f64>,
    pub weekly_reset_unix: Option<i64>,
    /// ACU-metered plans.
    pub acu_used: Option<f64>,
    pub acu_limit: Option<f64>,
    /// Extra-usage balance in micros.
    pub overage_micros: Option<f64>,
}

/// Extract (windsurf_api_key, api_server_url) from credentials.toml bytes.
/// The file is machine-written `key = "value"` lines.
pub fn creds_secrets(creds: &[u8]) -> Result<(String, String), String> {
    let text = String::from_utf8_lossy(creds);
    let mut key = None;
    let mut server = None;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = v.trim().trim_matches('"').to_string();
        match k.trim() {
            "windsurf_api_key" => key = Some(v),
            "api_server_url" => server = Some(v),
            _ => {}
        }
    }
    let key = key
        .filter(|k| !k.is_empty())
        .ok_or("credentials.toml has no api key")?;
    let server = server
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "https://server.codeium.com".to_string());
    Ok((key, server))
}

struct Cached {
    at: Instant,
    usage: Usage,
}

static CACHE: Mutex<Option<HashMap<String, Cached>>> = Mutex::new(None);

/// Cached variant for UI polling — same key returns fresh data for 60s.
pub fn user_status_cached(creds: &[u8]) -> Result<Usage, String> {
    let (key, server) = creds_secrets(creds)?;
    {
        let guard = CACHE.lock().unwrap();
        if let Some(map) = guard.as_ref() {
            if let Some(c) = map.get(&key) {
                if c.at.elapsed() < CACHE_TTL {
                    return Ok(c.usage.clone());
                }
            }
        }
    }
    let usage = user_status(&key, &server)?;
    CACHE
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(
            key,
            Cached {
                at: Instant::now(),
                usage: usage.clone(),
            },
        );
    Ok(usage)
}

/// Proto-JSON emits int64s as strings — accept either shape.
fn num(v: &serde_json::Value, key: &str) -> Option<f64> {
    let raw = v.get(key)?;
    if let Some(f) = raw.as_f64() {
        return Some(f);
    }
    raw.as_str()?.trim().parse().ok()
}

pub fn user_status(key: &str, server: &str) -> Result<Usage, String> {
    #[derive(Serialize)]
    struct Meta<'a> {
        #[serde(rename = "ideName")]
        ide_name: &'a str,
        #[serde(rename = "ideVersion")]
        ide_version: &'a str,
        #[serde(rename = "extensionName")]
        extension_name: &'a str,
        #[serde(rename = "extensionVersion")]
        extension_version: &'a str,
        #[serde(rename = "apiKey")]
        api_key: &'a str,
        locale: &'a str,
        os: &'a str,
    }
    #[derive(Serialize)]
    struct Req<'a> {
        metadata: Meta<'a>,
    }
    let url = format!("{}{}", server.trim_end_matches('/'), METHOD);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();
    let mut res = agent
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Connect-Protocol-Version", "1")
        .send_json(Req {
            metadata: Meta {
                ide_name: "devin-cli",
                ide_version: CLI_VERSION,
                extension_name: "devin-cli",
                extension_version: CLI_VERSION,
                api_key: key,
                locale: "en",
                os: std::env::consts::OS,
            },
        })
        .map_err(|e| format!("GetUserStatus failed: {e}"))?;
    let body: serde_json::Value = res
        .body_mut()
        .read_json()
        .map_err(|e| format!("GetUserStatus reply unreadable: {e}"))?;
    if let Some(code) = body.get("code").and_then(|c| c.as_str()) {
        if code == "unauthenticated" {
            return Err("Devin's sign-in has expired — sign in again".into());
        }
        return Err(format!(
            "Devin: {code} {}",
            body.get("message").and_then(|m| m.as_str()).unwrap_or("")
        ));
    }
    let us = &body["userStatus"];
    let st = &us["planStatus"];
    let info = &st["planInfo"];
    let devin = &info["devinInfo"];
    let hide = |k: &str| info.get(k).and_then(|b| b.as_bool()).unwrap_or(false);

    let mut u = Usage {
        email: us
            .get("email")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .into(),
        name: us.get("name").and_then(|e| e.as_str()).unwrap_or("").into(),
        user_id: us
            .get("userId")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .into(),
        plan: info
            .get("planName")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .into(),
        org_id: devin
            .get("orgId")
            .and_then(|e| e.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        plan_end: st
            .get("planEnd")
            .and_then(|e| e.as_str())
            .map(str::to_string),
        acu_used: num(st, "acuConsumed"),
        acu_limit: num(st, "acuLimit"),
        overage_micros: num(st, "overageBalanceMicros"),
        ..Default::default()
    };
    if !hide("hideDailyQuota") {
        u.daily_left = num(st, "dailyQuotaRemainingPercent");
        u.daily_reset_unix = num(st, "dailyQuotaResetAtUnix").map(|f| f as i64);
        // Spent windows lose their percent field entirely — a reset stamp
        // without a remaining % means the window exists and it's empty.
        if u.daily_left.is_none() && u.daily_reset_unix.is_some() {
            u.daily_left = Some(0.0);
        }
    }
    if !hide("hideWeeklyQuota") {
        u.weekly_left = num(st, "weeklyQuotaRemainingPercent");
        u.weekly_reset_unix = num(st, "weeklyQuotaResetAtUnix").map(|f| f as i64);
        if u.weekly_left.is_none() && u.weekly_reset_unix.is_some() {
            u.weekly_left = Some(0.0);
        }
    }
    Ok(u)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_parse() {
        let c = b"windsurf_api_key = \"tok\"\napi_server_url = \"https://s.example\"\ndevin_webapp_host = \"app.devin.ai\"\ndevin_api_url = \"https://api.devin.ai\"\n";
        let (k, s) = creds_secrets(c).unwrap();
        assert_eq!(k, "tok");
        assert_eq!(s, "https://s.example");
    }

    #[test]
    fn secrets_default_server() {
        let c = b"windsurf_api_key = \"tok\"\n";
        let (_, s) = creds_secrets(c).unwrap();
        assert_eq!(s, "https://server.codeium.com");
    }
}
