use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type UINT = u32;

    #[repr(C)]
    #[derive(Copy, Clone, Default, Debug)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    pub const SW_SHOWNOACTIVATE: i32 = 4;
    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;

    pub const SWP_NOACTIVATE: UINT = 0x0010;
    pub const SWP_FRAMECHANGED: UINT = 0x0020;
    pub const SWP_SHOWWINDOW: UINT = 0x0040;

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn SetWindowPos(
            hWnd: HWND,
            hWndInsertAfter: HWND,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: UINT,
        ) -> BOOL;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
        pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    }
}

// Keep tray icon alive in state
struct TrayState(#[allow(dead_code)] tauri::tray::TrayIcon);

// Global atomic flags
static IS_IGNORING_MOUSE: AtomicBool = AtomicBool::new(true);
static IS_MENU_OPEN: AtomicBool = AtomicBool::new(false);

static APP_STATE: Mutex<Option<serde_json::Value>> = Mutex::new(None);

fn get_state_file_path() -> std::path::PathBuf {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        let mut p = std::path::PathBuf::from(app_data);
        p.push("deskrug");
        std::fs::create_dir_all(&p).ok();
        p.push("carpet-state.json");
        return p;
    }
    std::path::PathBuf::from("carpet-state.json")
}

fn log_to_file(msg: &str) {
    let mut p = std::env::temp_dir();
    p.push("deskrug-rust.log");
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
        let _ = writeln!(f, "[{}] {}", chrono_or_simple(), msg);
    }
}

fn chrono_or_simple() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[tauri::command]
fn log_from_js(msg: String) {
    println!("[JS LOG] {}", msg);
    log_to_file(&format!("[JS] {}", msg));
}

#[tauri::command]
fn set_ignore_mouse(app: tauri::AppHandle, ignore: bool) {
    if IS_MENU_OPEN.load(Ordering::Relaxed) && ignore {
        // Do NOT ignore mouse while Telegram menu is open!
        return;
    }
    IS_IGNORING_MOUSE.store(ignore, Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_ignore_cursor_events(ignore);
    }
}

#[tauri::command]
fn save_state(state: serde_json::Value) {
    let path = get_state_file_path();
    if let Ok(json_str) = serde_json::to_string(&state) {
        std::fs::write(path, json_str).ok();
    }
}

#[tauri::command]
fn load_state() -> Option<serde_json::Value> {
    let path = get_state_file_path();
    if let Ok(data) = std::fs::read_to_string(path) {
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    log_to_file("quit_app() invoked!");
    let _ = app.exit(0);
    std::process::exit(0);
}

#[tauri::command]
fn report_state(app: tauri::AppHandle, state: serde_json::Value) {
    if let Ok(mut s) = APP_STATE.lock() {
        *s = Some(state.clone());
    }
    let _ = app.emit("carpet_state_changed", state);
}

#[tauri::command]
fn get_current_state() -> Option<serde_json::Value> {
    APP_STATE.lock().ok().and_then(|s| s.clone())
}

#[tauri::command]
fn set_menu_mode(app: tauri::AppHandle, open: bool) {
    IS_MENU_OPEN.store(open, Ordering::Relaxed);
    IS_IGNORING_MOUSE.store(!open, Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_ignore_cursor_events(!open);
    }
}

#[tauri::command]
fn show_menu(app: tauri::AppHandle) {
    set_menu_mode(app.clone(), true);
    let _ = app.emit("open_custom_menu", serde_json::json!({ "is_tray": false }));
}

fn build_native_tray_menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let flatten = MenuItem::with_id(app, "flatten", "Flatten Rug", true, None::<&str>)?;
    let center = MenuItem::with_id(app, "center", "Center Rug", true, None::<&str>)?;

    // Submenu for styles
    let style_tabriz = MenuItem::with_id(app, "style:tabriz", "Tabriz", true, None::<&str>)?;
    let style_klasik = MenuItem::with_id(app, "style:klasik", "Classic", true, None::<&str>)?;
    let style_lacivert = MenuItem::with_id(app, "style:lacivert", "Navy", true, None::<&str>)?;
    let style_zumrut = MenuItem::with_id(app, "style:zumrut", "Emerald", true, None::<&str>)?;
    let style_kilim = MenuItem::with_id(app, "style:kilim", "Kilim", true, None::<&str>)?;
    let style_gorunmez = MenuItem::with_id(app, "style:gorunmez", "Invisible", true, None::<&str>)?;
    let style_hereke = MenuItem::with_id(app, "style:hereke", "Hereke", true, None::<&str>)?;
    let style_iznik = MenuItem::with_id(app, "style:iznik", "Iznik", true, None::<&str>)?;
    let style_herati = MenuItem::with_id(app, "style:herati", "Herati", true, None::<&str>)?;
    let style_heriz = MenuItem::with_id(app, "style:heriz", "Heriz", true, None::<&str>)?;
    let style_nain = MenuItem::with_id(app, "style:nain", "Nain", true, None::<&str>)?;
    let style_turkmen = MenuItem::with_id(app, "style:turkmen", "Turkmen", true, None::<&str>)?;
    let style_bahtiyari = MenuItem::with_id(app, "style:bahtiyari", "Bakhtiari", true, None::<&str>)?;
    let style_gabbe = MenuItem::with_id(app, "style:gabbe", "Gabbeh", true, None::<&str>)?;

    let styles_sub = Submenu::with_items(
        app,
        "Pattern",
        true,
        &[
            &style_tabriz,
            &style_klasik,
            &style_lacivert,
            &style_zumrut,
            &style_kilim,
            &style_gorunmez,
            &style_hereke,
            &style_iznik,
            &style_herati,
            &style_heriz,
            &style_nain,
            &style_turkmen,
            &style_bahtiyari,
            &style_gabbe,
        ],
    )?;

    // Submenu for sizes
    let size_kucuk = MenuItem::with_id(app, "size:kucuk", "Small", true, None::<&str>)?;
    let size_orta = MenuItem::with_id(app, "size:orta", "Medium", true, None::<&str>)?;
    let size_buyuk = MenuItem::with_id(app, "size:buyuk", "Large", true, None::<&str>)?;
    let sizes_sub = Submenu::with_items(
        app,
        "Size",
        true,
        &[&size_kucuk, &size_orta, &size_buyuk],
    )?;

    // Submenu for weights
    let weight_hafif = MenuItem::with_id(app, "weight:hafif", "Light", true, None::<&str>)?;
    let weight_normal = MenuItem::with_id(app, "weight:normal", "Normal", true, None::<&str>)?;
    let weight_agir = MenuItem::with_id(app, "weight:agir", "Heavy", true, None::<&str>)?;
    let weights_sub = Submenu::with_items(
        app,
        "Weight",
        true,
        &[&weight_hafif, &weight_normal, &weight_agir],
    )?;

    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[
            &flatten,
            &center,
            &styles_sub,
            &sizes_sub,
            &weights_sub,
            &sep,
            &quit,
        ],
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            log_from_js,
            set_ignore_mouse,
            save_state,
            load_state,
            report_state,
            get_current_state,
            show_menu,
            set_menu_mode,
            quit_app
        ]);

    let app = builder
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Create native Windows tray menu
            let tray_menu = match build_native_tray_menu(&app_handle) {
                Ok(m) => Some(m),
                Err(e) => {
                    log_to_file(&format!("Error building native tray menu: {:?}", e));
                    None
                }
            };

            let mut tray_b = TrayIconBuilder::with_id("carpet-tray")
                .tooltip("DeskRug")
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let id = event.id().as_ref();
                    if id == "quit" {
                        let _ = app.exit(0);
                        std::process::exit(0);
                    } else if id == "flatten" {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "flatten", "arg": null }));
                    } else if id == "center" {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "center", "arg": null }));
                    } else if let Some(style) = id.strip_prefix("style:") {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "style", "arg": style }));
                    } else if let Some(size) = id.strip_prefix("size:") {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "size", "arg": size }));
                    } else if let Some(weight) = id.strip_prefix("weight:") {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "weight", "arg": weight }));
                    }
                });

            if let Some(ref menu) = tray_menu {
                tray_b = tray_b.menu(menu);
            }

            if let Some(icon) = app.default_window_icon() {
                tray_b = tray_b.icon(icon.clone());
            }

            match tray_b.build(&app_handle) {
                Ok(tray) => {
                    log_to_file("Tray icon built successfully");
                    app.manage(TrayState(tray));
                }
                Err(e) => {
                    log_to_file(&format!("Failed to build tray icon: {:?}", e));
                }
            }

            // Setup main window
            if let Some(window) = app.get_webview_window("main") {
                // Initially enable click-through
                let _ = window.set_ignore_cursor_events(true);

                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd_raw) = window.hwnd() {
                        let hwnd = hwnd_raw.0 as win32::HWND;
                        unsafe {
                            // Match full screen size
                            let screen_w = win32::GetSystemMetrics(win32::SM_CXSCREEN);
                            let screen_h = win32::GetSystemMetrics(win32::SM_CYSCREEN);
                            win32::SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                0,
                                0,
                                screen_w,
                                screen_h,
                                win32::SWP_NOACTIVATE | win32::SWP_SHOWWINDOW | win32::SWP_FRAMECHANGED,
                            );

                            // Show without activating/stealing focus
                            win32::ShowWindow(hwnd, win32::SW_SHOWNOACTIVATE);
                        }

                        // Send initial show & layout commands after page loads
                        let handle_clone = app_handle.clone();
                        std::thread::spawn(move || {
                            for ms in [200, 600, 1500] {
                                std::thread::sleep(std::time::Duration::from_millis(ms));
                                let _ = handle_clone.emit("command", serde_json::json!({ "cmd": "show", "arg": null }));
                                let _ = handle_clone.emit("command", serde_json::json!({ "cmd": "layout", "arg": null }));
                                let _ = handle_clone.emit("command", serde_json::json!({ "cmd": "redraw", "arg": null }));
                                let _ = handle_clone.emit("command", serde_json::json!({ "cmd": "reset-input", "arg": null }));
                            }
                        });

                        // Smart hover detection thread:
                        // ONLY runs when window is click-through (ignoring mouse).
                        // As soon as mouse enters carpet, bundle.js calls setIgnoreMouse(false),
                        // and this thread goes SILENT so native pointer events run with 0 latency!
                        let handle_clone2 = app_handle.clone();
                        let scale_factor = window.scale_factor().unwrap_or(1.0);
                        std::thread::spawn(move || {
                            let mut last_x = -9999;
                            let mut last_y = -9999;

                            loop {
                                std::thread::sleep(std::time::Duration::from_millis(40));

                                // When user is over carpet or dragging, DO NOT emit anything! Let native events handle it!
                                if !IS_IGNORING_MOUSE.load(Ordering::Relaxed) {
                                    continue;
                                }

                                unsafe {
                                    let mut pt = win32::POINT::default();
                                    if win32::GetCursorPos(&mut pt) != 0 {
                                        if (pt.x - last_x).abs() >= 2 || (pt.y - last_y).abs() >= 2 {
                                            last_x = pt.x;
                                            last_y = pt.y;

                                            let css_x = pt.x as f64 / scale_factor;
                                            let css_y = pt.y as f64 / scale_factor;

                                            let _ = handle_clone2.emit(
                                                "command",
                                                serde_json::json!({
                                                    "cmd": "cursor",
                                                    "arg": {
                                                        "x": css_x,
                                                        "y": css_y
                                                    }
                                                }),
                                            );
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    log_to_file("Starting app.run...");
    app.run(|_app_handle, event| {
        match &event {
            tauri::RunEvent::ExitRequested { code, .. } => {
                log_to_file(&format!("ExitRequested code: {:?}", code));
            }
            tauri::RunEvent::WindowEvent { label, event, .. } => {
                log_to_file(&format!("WindowEvent [{}] {:?}", label, event));
            }
            tauri::RunEvent::WebviewEvent { label, event, .. } => {
                log_to_file(&format!("WebviewEvent [{}] {:?}", label, event));
            }
            _ => {}
        }
    });
}
