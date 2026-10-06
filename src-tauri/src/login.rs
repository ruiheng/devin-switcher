use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::paths;

// The flow mirrors `devin auth login`'s PKCE + localhost callback (see
// magpie internal/provider/signin.go): prompt=select_account keeps an
// existing devin.ai browser session from silently re-using the old account.
const AUTHORIZE_URL: &str = "https://app.devin.ai/auth/cli/continue";
const EXCHANGE_URL: &str =
    "https://server.codeium.com/exa.seat_management_pb.SeatManagementService/ExchangeDevinCLIPKCECode";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(600);

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut v = vec![0u8; n];
    let _ = getrandom::getrandom(&mut v);
    v
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginOffer {
    pub id: u64,
    pub url: String,
    /// Fallback command the user can run in a terminal against the same
    /// fake home — output lands in the same place.
    pub hint: String,
}

/// What a poll found: still waiting, callback received + exchange in
/// flight, credentials acquired (caller finishes the account), or failed.
pub enum PollState {
    Waiting,
    Working,
    GotCreds { creds: Vec<u8>, home: PathBuf },
    Failed { error: String },
}

enum Outcome {
    /// Browser callback arrived; the token exchange is running.
    Exchanging,
    Credentials(Vec<u8>),
    Failed(String),
}

struct Session {
    home: PathBuf,
    created: Instant,
    rx: mpsc::Receiver<Outcome>,
    /// The browser callback landed — polls report Working from here on.
    working: bool,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static SESSIONS: Mutex<Option<HashMap<u64, Session>>> = Mutex::new(None);

fn sessions() -> std::sync::MutexGuard<'static, Option<HashMap<u64, Session>>> {
    SESSIONS.lock().unwrap()
}

/// Exchange the callback code for a session token (same call the CLI makes)
/// and render the credentials.toml `devin auth login` would have written.
fn exchange(code: &str, verifier: &str, redirect: &str) -> Result<Vec<u8>, String> {
    #[derive(serde::Deserialize, Default)]
    struct Res {
        #[serde(rename = "sessionToken")]
        session_token: Option<String>,
        session_token_alt: Option<String>,
        #[serde(rename = "devinWebappHost")]
        webapp: Option<String>,
        #[serde(rename = "devinApiUrl")]
        api: Option<String>,
    }
    #[derive(serde::Serialize)]
    struct Req<'a> {
        code: &'a str,
        code_verifier: &'a str,
        redirect_uri: &'a str,
    }
    let mut res = ureq::post(EXCHANGE_URL)
        .header("Content-Type", "application/json")
        .send_json(Req {
            code,
            code_verifier: verifier,
            redirect_uri: redirect,
        })
        .map_err(|e| format!("token exchange failed: {e}"))?;
    let res: Res = res
        .body_mut()
        .read_json()
        .map_err(|e| format!("token exchange reply unreadable: {e}"))?;
    let key = res
        .session_token
        .or(res.session_token_alt)
        .filter(|s| !s.is_empty())
        .ok_or("Devin's exchange returned no session token")?;
    Ok(credentials_toml(
        &key,
        "https://server.codeium.com",
        res.webapp.as_deref().unwrap_or("app.devin.ai"),
        res.api.as_deref().unwrap_or("https://api.devin.ai"),
    )
    .into_bytes())
}

/// credentials.toml in the exact field order `devin auth login` writes.
pub fn credentials_toml(key: &str, server: &str, webapp: &str, api: &str) -> String {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "windsurf_api_key = \"{}\"\napi_server_url = \"{}\"\ndevin_webapp_host = \"{}\"\ndevin_api_url = \"{}\"\n",
        esc(key),
        esc(server),
        esc(webapp),
        esc(api)
    )
}

fn read_request_line(stream: &TcpStream) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(line)
}

fn query_param(target: &str, key: &str) -> Option<String> {
    let q = target.split('?').nth(1)?.split('#').next()?;
    for pair in q.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if k == key {
            return Some(urldecode(v));
        }
    }
    None
}

fn urldecode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v as char);
                i += 3;
                continue;
            }
        }
        out.push(if b[i] == b'+' { ' ' } else { b[i] as char });
        i += 1;
    }
    out
}

const DONE_PAGE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<!doctype html><title>Devin Switch</title><body style=\"font-family:sans-serif;display:grid;place-items:center;min-height:80vh\"><div><h2>Sign-in complete</h2><p>You can close this tab and return to Devin Switch.</p></div>";

const ERR_PAGE: &str = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<!doctype html><title>Devin Switch</title><body style=\"font-family:sans-serif;display:grid;place-items:center;min-height:80vh\"><div><h2>Sign-in failed</h2><p>Devin Switch didn't recognize this callback — go back and try again.</p></div>";

const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

/// Begin a login round: fake home + PKCE URL + callback listener thread.
pub fn start() -> Result<LoginOffer, String> {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let home = paths::run_dir().join(format!("login-{id}"));
    std::fs::create_dir_all(home.join("devin")).map_err(|e| e.to_string())?;

    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("callback listener: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect = format!("http://127.0.0.1:{port}/callback");

    let verifier = b64url(&rand_bytes(32));
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
    let state = b64url(&rand_bytes(16));

    let url = format!(
        "{AUTHORIZE_URL}?redirect_uri={}&state={}&prompt=select_account&code_challenge={}&code_challenge_method=S256&cli_pkce_marker=1",
        urlencode(&redirect),
        state,
        challenge
    );

    let (tx, rx) = mpsc::channel();
    {
        let verifier = verifier.clone();
        let state = state.clone();
        let redirect = redirect.clone();
        std::thread::spawn(move || {
            let outcome = wait_callback(listener, &state, LOGIN_TIMEOUT).and_then(|code| {
                let _ = tx.send(Outcome::Exchanging);
                exchange(&code, &verifier, &redirect)
            });
            let _ = tx.send(match outcome {
                Ok(creds) => Outcome::Credentials(creds),
                Err(e) => Outcome::Failed(e),
            });
        });
    }

    sessions().get_or_insert_with(HashMap::new).insert(
        id,
        Session {
            home: home.clone(),
            created: Instant::now(),
            rx,
            working: false,
        },
    );

    let hint = if cfg!(windows) {
        format!(
            "set APPDATA={} && set XDG_DATA_HOME={0} && devin auth login",
            home.display()
        )
    } else {
        format!("XDG_DATA_HOME=\"{}\" devin auth login", home.display())
    };

    Ok(LoginOffer { id, url, hint })
}

/// A manual login round — what `devin auth login --force-manual-token-flow`
/// does for SSH/remote sessions: the authorize URL carries no redirect_uri,
/// so after sign-in the page shows a code to paste back instead of hitting
/// a localhost listener.
pub struct ManualLogin {
    pub url: String,
    verifier: String,
}

pub fn manual_start() -> ManualLogin {
    let verifier = b64url(&rand_bytes(32));
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
    let state = b64url(&rand_bytes(16));
    let url = format!(
        "{AUTHORIZE_URL}?state={state}&prompt=select_account&code_challenge={challenge}&code_challenge_method=S256&cli_pkce_marker=1"
    );
    ManualLogin { url, verifier }
}

/// Trade the code the page showed for credentials.toml bytes. The manual
/// flow registered no redirect_uri, so the exchange sends an empty one.
pub fn manual_finish(m: &ManualLogin, code: &str) -> Result<Vec<u8>, String> {
    let code = code.trim();
    if code.is_empty() {
        return Err("code is empty".into());
    }
    exchange(code, &m.verifier, "")
}

/// What one accepted connection turned out to be.
enum Conn {
    /// Our callback with a code — done.
    Code(String),
    /// Our callback but broken (bad state, no code) — terminal failure.
    Fatal(String),
    /// Anything else: favicon requests, speculative sockets that never send
    /// a request line. Answered 404 or dropped; the listener stays up.
    Stray,
}

fn wait_callback(
    listener: TcpListener,
    want_state: &str,
    timeout: Duration,
) -> Result<String, String> {
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() > deadline {
            return Err("login timed out (10 min)".into());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Windows inherits the listener's nonblocking mode on
                // accepted sockets (unix doesn't), and read timeouts are
                // ignored on nonblocking sockets — force blocking or every
                // read can fail instantly with WSAEWOULDBLOCK (10035).
                stream.set_nonblocking(false).ok();
                stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                match handle_conn(&mut stream, want_state) {
                    Conn::Code(code) => return Ok(code),
                    Conn::Fatal(e) => return Err(e),
                    Conn::Stray => continue,
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("callback listener: {e}")),
        }
    }
}

fn handle_conn(stream: &mut TcpStream, want_state: &str) -> Conn {
    let Ok(line) = read_request_line(stream) else {
        return Conn::Stray;
    };
    // drain remaining headers best-effort
    let mut sink = [0u8; 512];
    let _ = stream.read(&mut sink);
    let Some(target) = line.split_whitespace().nth(1) else {
        return Conn::Stray;
    };
    if !target.starts_with("/callback") {
        let _ = stream.write_all(NOT_FOUND.as_bytes());
        return Conn::Stray;
    }
    let code = query_param(target, "code");
    let state = query_param(target, "state");
    if state.as_deref() != Some(want_state) {
        let _ = stream.write_all(ERR_PAGE.as_bytes());
        let _ = stream.flush();
        return Conn::Fatal("callback state mismatch".into());
    }
    match code.filter(|c| !c.is_empty()) {
        Some(code) => {
            let _ = stream.write_all(DONE_PAGE.as_bytes());
            let _ = stream.flush();
            Conn::Code(code)
        }
        None => {
            let _ = stream.write_all(ERR_PAGE.as_bytes());
            let _ = stream.flush();
            Conn::Fatal("callback carried no code".into())
        }
    }
}

/// Poll a login round. On `GotCreds` the caller writes them into the fake
/// home, asks the CLI who signed in, saves the profile, then cleans up.
pub fn poll(id: u64) -> Result<PollState, String> {
    let mut guard = sessions();
    let map = guard.get_or_insert_with(HashMap::new);
    let Some(sess) = map.get_mut(&id) else {
        return Err("no such login session".into());
    };

    // Fallback path: user ran `devin auth login` against the fake home and
    // the CLI wrote credentials.toml itself.
    let cli_creds = sess.home.join("devin").join("credentials.toml");
    if let Ok(b) = std::fs::read(&cli_creds) {
        if !b.is_empty() {
            let home = sess.home.clone();
            map.remove(&id);
            return Ok(PollState::GotCreds { creds: b, home });
        }
    }

    // Drain progress + the terminal outcome if one arrived.
    let mut terminal = None;
    loop {
        match sess.rx.try_recv() {
            Ok(Outcome::Exchanging) => sess.working = true,
            Ok(o) => {
                terminal = Some(o);
                break;
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                terminal = Some(Outcome::Failed("login listener stopped".into()));
                break;
            }
        }
    }

    match terminal {
        Some(Outcome::Credentials(creds)) => {
            let home = sess.home.clone();
            map.remove(&id);
            Ok(PollState::GotCreds { creds, home })
        }
        Some(Outcome::Failed(error)) => {
            map.remove(&id);
            Ok(PollState::Failed { error })
        }
        Some(Outcome::Exchanging) => Ok(PollState::Working),
        None => {
            if sess.working {
                Ok(PollState::Working)
            } else if sess.created.elapsed() > LOGIN_TIMEOUT {
                map.remove(&id);
                Ok(PollState::Failed {
                    error: "login timed out".into(),
                })
            } else {
                Ok(PollState::Waiting)
            }
        }
    }
}

pub fn cancel(id: u64) {
    if let Some(sess) = sessions().get_or_insert_with(HashMap::new).remove(&id) {
        let _ = std::fs::remove_dir_all(&sess.home);
    }
}

pub fn cleanup_home(home: &PathBuf) {
    let _ = std::fs::remove_dir_all(home);
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creds_format() {
        let t = credentials_toml("k", "s", "w", "a");
        assert!(t.starts_with("windsurf_api_key = \"k\""));
        assert!(t.contains("devin_api_url = \"a\""));
        assert_eq!(t.lines().count(), 4);
    }

    #[test]
    fn parses_query() {
        let t = "/callback?code=abc%20x&state=s1";
        assert_eq!(query_param(t, "code").as_deref(), Some("abc x"));
        assert_eq!(query_param(t, "state").as_deref(), Some("s1"));
        assert_eq!(query_param(t, "missing"), None);
    }
}
