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
        *MANUAL.lock().unwrap_or_else(|e| e.into_inner()) = Some(m);
        url
    }

    #[tauri::command]
    pub async fn manual_finish(name: String, code: String) -> Result<ProfileInfo, String> {
        let code = code.trim().to_string();
        // Accept a raw session token too — some users land here with one.
        if code.starts_with("devin-session-token") {
            return ops::add_token(&name, &code);
        }
        let m = MANUAL
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .ok_or_else(|| {
                "no manual login in progress — click the link button first"
                    .to_string()
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

    /// Tray popup menu: current sign-in(s) as a disabled header, saved
    /// accounts as check items whose id carries the switch scope
    /// (`sw:<scope>:<name>` — mirrors the card's "only the missing side"
    /// logic), then Show/Quit. Rebuilt on hover and after a switch.
    pub fn tray_menu(
        app: &tauri::AppHandle,
    ) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
        use tauri::menu::{
            CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem,
        };
        let st = ops::status();
        let dsk = st.desktop.as_ref();
        let dsk_in = dsk.map(|d| d.logged_in).unwrap_or(false);
        // Same matching rule as the cards: user_id first, email fallback.
        let dmatch = |user_id: &str, email: &str| {
            dsk_in
                && dsk
                    .map(|d| {
                        (!user_id.is_empty() && user_id == d.user_id)
                            || (!d.email.is_empty()
                                && !email.is_empty()
                                && email == d.email)
                    })
                    .unwrap_or(false)
        };
        let who = |e: &str, n: &str| {
            if !e.is_empty() { e.to_string() } else { n.to_string() }
        };
        let same = st.auth.logged_in && dmatch(&st.auth.user_id, &st.auth.email);

        let mut heads: Vec<MenuItem<tauri::Wry>> = Vec::new();
        if same {
            heads.push(MenuItem::with_id(
                app,
                "cur",
                format!("CLI+桌面端: {}", who(&st.auth.email, &st.auth.name)),
                false,
                None::<&str>,
            )?);
        } else {
            let c = if st.auth.logged_in {
                who(&st.auth.email, &st.auth.name)
            } else {
                "未登录".into()
            };
            heads.push(MenuItem::with_id(
                app,
                "cur_cli",
                format!("CLI: {c}"),
                false,
                None::<&str>,
            )?);
            if let Some(d) = dsk {
                let dl = if d.logged_in {
                    who(&d.email, &d.label)
                } else {
                    "未登录".into()
                };
                heads.push(MenuItem::with_id(
                    app,
                    "cur_desk",
                    format!("桌面端: {dl}"),
                    false,
                    None::<&str>,
                )?);
            }
        }

        let mut accts: Vec<CheckMenuItem<tauri::Wry>> = Vec::new();
        for p in store::list().into_iter().take(12) {
            let m = &p.meta;
            let da = dmatch(&m.user_id, &m.email);
            let (scope, tag) = match (p.is_active, da) {
                (true, true) => ("none", " [CLI+桌面端]"),
                (true, false) => ("desktop", " [CLI]"),
                (false, true) => ("cli", " [桌面端]"),
                (false, false) => ("all", ""),
            };
            // A check only means "live on BOTH sides" — two checked
            // items (one CLI, one Desktop) read as a contradiction.
            // Single-side liveness is conveyed by the [CLI]/[桌面端] tag.
            accts.push(CheckMenuItem::with_id(
                app,
                format!("sw:{scope}:{}", p.name),
                format!("{}{}", who(&m.email, &p.name), tag),
                scope != "none",
                p.is_active && da,
                None::<&str>,
            )?);
        }

        let show = MenuItem::with_id(app, "show", "打开 / Show", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", "退出 / Quit", true, None::<&str>)?;
        let sep1 = PredefinedMenuItem::separator(app)?;
        let sep2 = PredefinedMenuItem::separator(app)?;
        let mut items: Vec<&dyn IsMenuItem<tauri::Wry>> = heads
            .iter()
            .map(|h| h as &dyn IsMenuItem<tauri::Wry>)
            .collect();
        items.push(&sep1);
        items.extend(accts.iter().map(|a| a as &dyn IsMenuItem<tauri::Wry>));
        items.push(&sep2);
        items.push(&show);
        items.push(&quit);
        Menu::with_items(app, &items)
    }

    /// Rebuild the tray menu — throttled for the hover path (status is
    /// cached but still not free), forced after a switch completes.
    pub fn refresh_tray(app: &tauri::AppHandle, force: bool) {
        static LAST: Mutex<Option<std::time::Instant>> = Mutex::new(None);
        {
            let mut g = LAST.lock().unwrap_or_else(|e| e.into_inner());
            if !force
                && g.is_some_and(|t| {
                    t.elapsed() < std::time::Duration::from_secs(3)
                })
            {
                return;
            }
            *g = Some(std::time::Instant::now());
        }
        if let Some(tr) = app.tray_by_id("main-tray") {
            if let Ok(m) = tray_menu(app) {
                let _ = tr.set_menu(Some(m));
            }
        }
    }
}

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Release GUI builds have no console and WER only records the
    // exception code (0xc0000409) — route panics into errors.log so a
    // crash is diagnosable after the fact.
    std::panic::set_hook(Box::new(|info| {
        ops::log_error("panic", &info.to_string());
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Second launch → focus the existing window instead of
            // running a duplicate poller. It may be hidden in the tray.
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            use tauri::{
                tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
                Manager, WindowEvent,
            };
            let raise = |app: &tauri::AppHandle| {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            };
            let menu = gui::tray_menu(app.handle())?;
            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Devin Switch")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, e| {
                    let id = e.id.as_ref();
                    if id == "quit" {
                        app.exit(0);
                    } else if id == "show" {
                        raise(app);
                    } else if let Some((scope, name)) =
                        id.strip_prefix("sw:").and_then(|r| r.split_once(':'))
                    {
                        if scope != "none" {
                            let (app2, scope, name) =
                                (app.clone(), scope.to_string(), name.to_string());
                            std::thread::spawn(move || {
                                if let Err(e) = crate::ops::use_profile(&name, &scope) {
                                    crate::ops::log_error("tray_switch", &e);
                                }
                                gui::refresh_tray(&app2, true);
                            });
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    // Hover refresh keeps the menu's sign-in/account
                    // state current when the popup actually opens.
                    TrayIconEvent::Enter { .. } => {
                        gui::refresh_tray(tray.app_handle(), false)
                    }
                    _ => {}
                })
                .build(app)?;
            if let Some(w) = app.get_webview_window("main") {
                let win = w.clone();
                w.on_window_event(move |event| match event {
                    // Close AND minimize both park the window in the
                    // tray — real Quit lives on the tray menu.
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                    WindowEvent::Resized(_) => {
                        if win.is_minimized().unwrap_or(false) {
                            let _ = win.hide();
                        }
                    }
                    _ => {}
                });
            }
            Ok(())
        })
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
