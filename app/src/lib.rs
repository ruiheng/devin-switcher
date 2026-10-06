pub mod config;
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

    #[tauri::command]
    pub async fn get_status() -> Status {
        ops::status()
    }

    #[tauri::command]
    pub async fn list_profiles() -> Vec<ProfileInfo> {
        store::list()
    }

    #[tauri::command]
    pub async fn save_current(name: String, note: String) -> Result<ProfileInfo, String> {
        ops::save_current(&name, &note)
    }

    #[tauri::command]
    pub async fn delete_profile(name: String) -> Result<(), String> {
        store::remove(&name)
    }

    #[tauri::command]
    pub async fn rename_profile(from: String, to: String) -> Result<ProfileInfo, String> {
        store::rename(&from, &to)
    }

    #[tauri::command]
    pub async fn use_profile(name: String) -> Result<UseResult, String> {
        ops::use_profile(&name)
    }

    #[tauri::command]
    pub async fn refresh_usage(name: String) -> Result<ProfileInfo, String> {
        ops::refresh_usage(&name)
    }

    #[tauri::command]
    pub async fn start_login() -> Result<login::LoginOffer, String> {
        login::start()
    }

    #[tauri::command]
    pub async fn poll_login(id: u64, name: String, note: String) -> Result<LoginOutcome, String> {
        ops::finish_login(id, &name, &note)
    }

    #[tauri::command]
    pub async fn cancel_login(id: u64) -> Result<(), String> {
        login::cancel(id);
        Ok(())
    }

    #[tauri::command]
    pub async fn add_token(name: String, token: String) -> Result<ProfileInfo, String> {
        ops::add_token(&name, &token)
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
        let creds = login::manual_finish(&m, &code)?;
        ops::save_creds(&creds, &name)
    }

    #[tauri::command]
    pub async fn launch_parallel(name: String, cwd: Option<String>) -> Result<ParallelLaunch, String> {
        ops::launch_parallel(&name, cwd.as_deref())
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
            gui::start_login,
            gui::poll_login,
            gui::cancel_login,
            gui::add_token,
            gui::manual_start,
            gui::manual_finish,
            gui::launch_parallel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running devin-switch");
}
