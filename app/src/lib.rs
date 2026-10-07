pub mod config;
pub mod desktop;
pub mod devincli;
pub mod login;
pub mod model;
pub mod ops;
pub mod paths;
pub mod store;
pub mod usage;

#[cfg(feature = "gui")]
mod gui {
    use std::sync::Mutex;

    use crate::login::{self, ManualLogin};
    use crate::model::{ProfileInfo, UseResult};
    use crate::ops::{self, LoginOutcome, ParallelLaunch, Status};
    use crate::store;

    static MANUAL: Mutex<Option<ManualLogin>> = Mutex::new(None);

    /// Log a command failure to errors.log before returning it — release
    /// builds have no devtools, so a missed toast is otherwise lost.
    fn logged<T>(ctx: &str, r: Result<T, String>) -> Result<T, String> {
        r.map_err(|e| {
            ops::log_error(ctx, &e);
            e
        })
    }

    #[tauri::command]
    pub async fn get_status() -> Status {
        ops::status()
    }

    #[tauri::command]
    pub async fn list_profiles() -> Vec<ProfileInfo> {
        ops::auto_refresh_stale();
        store::list()
    }

    #[tauri::command]
    pub async fn save_current(name: String, note: String) -> Result<ProfileInfo, String> {
        logged("save_current", ops::save_current(&name, &note))
    }

    #[tauri::command]
    pub async fn delete_profile(name: String) -> Result<(), String> {
        logged("delete_profile", store::remove(&name))
    }

    #[tauri::command]
    pub async fn rename_profile(from: String, to: String) -> Result<ProfileInfo, String> {
        logged("rename_profile", store::rename(&from, &to))
    }

    #[tauri::command]
    pub async fn use_profile(name: String, scope: Option<String>) -> Result<UseResult, String> {
        logged("use_profile", ops::use_profile(&name, scope.as_deref().unwrap_or("all")))
    }

    #[tauri::command]
    pub async fn refresh_usage(name: String) -> Result<ProfileInfo, String> {
        logged("refresh_usage", ops::refresh_usage(&name))
    }

    #[tauri::command]
    pub async fn refresh_all() -> Vec<ProfileInfo> {
        ops::refresh_all();
        store::list()
    }

    #[tauri::command]
    pub async fn start_login() -> Result<login::LoginOffer, String> {
        logged("start_login", login::start())
    }

    #[tauri::command]
    pub async fn poll_login(id: u64, name: String, note: String) -> Result<LoginOutcome, String> {
        logged("poll_login", ops::finish_login(id, &name, &note))
    }

    #[tauri::command]
    pub async fn cancel_login(id: u64) -> Result<(), String> {
        login::cancel(id);
        Ok(())
    }

    #[tauri::command]
    pub async fn add_token(name: String, token: String) -> Result<ProfileInfo, String> {
        logged("add_token", ops::add_token(&name, &token))
    }

    /// Save Devin Desktop's current sign-in as an account.
    #[tauri::command]
    pub async fn import_desktop(name: String) -> Result<ProfileInfo, String> {
        logged("import_desktop", ops::import_desktop(&name))
    }

    /// Manual (paste-code) login for the GUI's token tab: same PKCE round
    /// as dsw login — the URL is shown for copy/open, the user pastes the
    /// code the page displays.
    #[tauri::command]
    pub async fn manual_start() -> String {
        let m = login::manual_start();
        let url = m.url.clone();
        *MANUAL.lock().unwrap() = Some(m);
        url
    }

    #[tauri::command]
    pub async fn manual_finish(name: String, code: String) -> Result<ProfileInfo, String> {
        let code = code.trim().to_string();
        // Accept a raw session token too — some users land here with one.
        if code.starts_with("devin-session-token") {
            return ops::add_token(&name, &code);
        }
        let m = MANUAL.lock().unwrap().take().ok_or_else(|| {
            "no manual login in progress — click the link button first".to_string()
        })?;
        let creds = login::manual_finish(&m, &code).map_err(|e| {
            ops::log_error("manual_finish", &e);
            e
        })?;
        logged("manual_finish", ops::save_creds(&creds, &name))
    }

    #[tauri::command]
    pub async fn launch_parallel(name: String, cwd: Option<String>) -> Result<ParallelLaunch, String> {
        logged("launch_parallel", ops::launch_parallel(&name, cwd.as_deref()))
    }
}

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            gui::get_status,
            gui::list_profiles,
            gui::save_current,
            gui::delete_profile,
            gui::rename_profile,
            gui::use_profile,
            gui::refresh_usage,
            gui::refresh_all,
            gui::start_login,
            gui::poll_login,
            gui::cancel_login,
            gui::add_token,
            gui::import_desktop,
            gui::manual_start,
            gui::manual_finish,
            gui::launch_parallel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running devin-switch");
}
