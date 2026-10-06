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
    use crate::login;
    use crate::model::{ProfileInfo, UseResult};
    use crate::ops::{self, LoginOutcome, ParallelLaunch, Status};
    use crate::store;

    #[tauri::command]
    pub fn get_status() -> Status {
        ops::status()
    }

    #[tauri::command]
    pub fn list_profiles() -> Vec<ProfileInfo> {
        store::list()
    }

    #[tauri::command]
    pub fn save_current(name: String, note: String) -> Result<ProfileInfo, String> {
        ops::save_current(&name, &note)
    }

    #[tauri::command]
    pub fn delete_profile(name: String) -> Result<(), String> {
        store::remove(&name)
    }

    #[tauri::command]
    pub fn rename_profile(from: String, to: String) -> Result<ProfileInfo, String> {
        store::rename(&from, &to)
    }

    #[tauri::command]
    pub fn use_profile(name: String) -> Result<UseResult, String> {
        ops::use_profile(&name)
    }

    #[tauri::command]
    pub fn refresh_usage(name: String) -> Result<ProfileInfo, String> {
        ops::refresh_usage(&name)
    }

    #[tauri::command]
    pub fn start_login() -> Result<login::LoginOffer, String> {
        login::start()
    }

    #[tauri::command]
    pub fn poll_login(id: u64, name: String, note: String) -> Result<LoginOutcome, String> {
        ops::finish_login(id, &name, &note)
    }

    #[tauri::command]
    pub fn cancel_login(id: u64) -> Result<(), String> {
        login::cancel(id);
        Ok(())
    }

    #[tauri::command]
    pub fn add_token(name: String, token: String) -> Result<ProfileInfo, String> {
        ops::add_token(&name, &token)
    }

    #[tauri::command]
    pub fn launch_parallel(name: String, cwd: Option<String>) -> Result<ParallelLaunch, String> {
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
            gui::launch_parallel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running devin-switch");
}
