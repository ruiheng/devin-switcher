mod config;
mod devincli;
mod login;
mod model;
mod ops;
mod paths;
mod store;

use model::{ProfileInfo, UseResult};
use ops::{LoginOutcome, ParallelLaunch, Status};

#[tauri::command]
fn get_status() -> Status {
    ops::status()
}

#[tauri::command]
fn list_profiles() -> Vec<ProfileInfo> {
    store::list()
}

#[tauri::command]
fn save_current(name: String, note: String) -> Result<ProfileInfo, String> {
    ops::save_current(&name, &note)
}

#[tauri::command]
fn delete_profile(name: String) -> Result<(), String> {
    store::remove(&name)
}

#[tauri::command]
fn use_profile(name: String) -> Result<UseResult, String> {
    ops::use_profile(&name)
}

#[tauri::command]
fn start_login() -> Result<login::LoginOffer, String> {
    login::start()
}

#[tauri::command]
fn poll_login(id: u64, name: String, note: String) -> Result<LoginOutcome, String> {
    ops::finish_login(id, &name, &note)
}

#[tauri::command]
fn cancel_login(id: u64) -> Result<(), String> {
    login::cancel(id);
    Ok(())
}

#[tauri::command]
fn add_token(name: String, token: String) -> Result<ProfileInfo, String> {
    ops::add_token(&name, &token)
}

#[tauri::command]
fn launch_parallel(name: String, cwd: Option<String>) -> Result<ParallelLaunch, String> {
    ops::launch_parallel(&name, cwd.as_deref())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_status,
            list_profiles,
            save_current,
            delete_profile,
            use_profile,
            start_login,
            poll_login,
            cancel_login,
            add_token,
            launch_parallel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running devin-switch");
}
