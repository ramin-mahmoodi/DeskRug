use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
mod tray_catalog;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
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
    pub const SM_SWAPBUTTON: i32 = 23;
    pub const SM_XVIRTUALSCREEN: i32 = 76;
    pub const SM_YVIRTUALSCREEN: i32 = 77;
    pub const SM_CXVIRTUALSCREEN: i32 = 78;
    pub const SM_CYVIRTUALSCREEN: i32 = 79;
    pub const GA_ROOT: UINT = 2;

    pub const SWP_NOACTIVATE: UINT = 0x0010;
    pub const SWP_FRAMECHANGED: UINT = 0x0020;
    pub const SWP_SHOWWINDOW: UINT = 0x0040;

    #[repr(C)]
    pub struct PROCESS_POWER_THROTTLING_STATE {
        pub version: u32,
        pub control_mask: u32,
        pub state_mask: u32,
    }

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
        pub fn GetAsyncKeyState(vKey: i32) -> i16;
        pub fn WindowFromPoint(Point: POINT) -> HWND;
        pub fn GetAncestor(hWnd: HWND, gaFlags: UINT) -> HWND;
        pub fn GetClassNameW(hWnd: HWND, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    }

    #[link(name = "winmm")]
    unsafe extern "system" {
        pub fn timeBeginPeriod(uPeriod: u32) -> u32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetCurrentProcess() -> HWND;
        pub fn SetProcessInformation(
            hProcess: HWND,
            ProcessInformationClass: i32,
            ProcessInformation: *const c_void,
            ProcessInformationSize: u32,
        ) -> BOOL;
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        pub fn RegOpenKeyExW(
            hKey: isize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut isize,
        ) -> i32;
        pub fn RegQueryValueExW(
            hKey: isize,
            lpValueName: *const u16,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        pub fn RegCloseKey(hKey: isize) -> i32;
    }
}

#[cfg(target_os = "windows")]
fn disable_power_throttling() {
    unsafe {
        let pts = win32::PROCESS_POWER_THROTTLING_STATE {
            version: 1,
            control_mask: 0x1, // PROCESS_POWER_THROTTLING_EXECUTION_SPEED
            state_mask: 0,     // 0 = do NOT throttle execution speed
        };
        win32::SetProcessInformation(
            win32::GetCurrentProcess(),
            4, // ProcessPowerThrottling
            &pts as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<win32::PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

#[cfg(target_os = "windows")]
fn left_button_down() -> bool {
    unsafe {
        let swapped = win32::GetSystemMetrics(win32::SM_SWAPBUTTON) != 0;
        let vkey = if swapped { 0x02 } else { 0x01 };
        (win32::GetAsyncKeyState(vkey) as u16 & 0x8000) != 0
    }
}

#[cfg(target_os = "windows")]
fn point_on_desktop(pt: win32::POINT, my_hwnd: win32::HWND) -> bool {
    unsafe {
        let hwnd_at_point = win32::WindowFromPoint(pt);
        if hwnd_at_point.is_null() {
            return true;
        }
        let root = win32::GetAncestor(hwnd_at_point, win32::GA_ROOT);
        if root.is_null() || root == my_hwnd {
            return true;
        }
        let mut buf = [0u16; 64];
        let len = win32::GetClassNameW(root, buf.as_mut_ptr(), 64);
        if len > 0 {
            let cls = String::from_utf16_lossy(&buf[..len as usize]);
            return cls == "Progman" || cls == "WorkerW";
        }
        true
    }
}

// Keep tray icon alive in state
struct TrayState(#[allow(dead_code)] tauri::tray::TrayIcon);

// Global atomic flags
static IS_IGNORING_MOUSE: AtomicBool = AtomicBool::new(true);

static APP_STATE: Mutex<Option<serde_json::Value>> = Mutex::new(None);

type TrayCheckItems = (
    Vec<(&'static str, CheckMenuItem<tauri::Wry>)>,
    Vec<(&'static str, CheckMenuItem<tauri::Wry>)>,
    Vec<(&'static str, CheckMenuItem<tauri::Wry>)>,
);
static TRAY_ITEMS: Mutex<Option<TrayCheckItems>> = Mutex::new(None);

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

fn atomic_save_state(state: &serde_json::Value) {
    let path = get_state_file_path();
    let tmp_path = path.with_extension("tmp");
    if let Ok(json_str) = serde_json::to_string(state) {
        if std::fs::write(&tmp_path, json_str).is_ok() {
            let _ = std::fs::rename(&tmp_path, &path);
        }
    }
}

fn merge_json(target: &mut serde_json::Value, source: &serde_json::Value) {
    if let (Some(t), Some(s)) = (target.as_object_mut(), source.as_object()) {
        for (k, v) in s {
            t.insert(k.clone(), v.clone());
        }
    }
}

fn log_to_file(msg: &str) {
    let mut p = std::env::temp_dir();
    p.push("deskrug-rust.log");
    // Cap log file at 500 KB to prevent unbounded growth
    if let Ok(meta) = std::fs::metadata(&p) {
        if meta.len() > 500 * 1024 {
            let _ = std::fs::remove_file(&p);
        }
    }
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

fn update_tray_checks(state: &serde_json::Value) {
    if let Ok(guard) = TRAY_ITEMS.lock() {
        if let Some((ref styles, ref sizes, ref weights)) = *guard {
            if let Some(s) = state.get("style").and_then(|v| v.as_str()) {
                for (id, item) in styles {
                    let _ = item.set_checked(*id == s);
                }
            }
            if let Some(s) = state.get("size").and_then(|v| v.as_str()) {
                for (id, item) in sizes {
                    let _ = item.set_checked(*id == s);
                }
            }
            if let Some(w) = state.get("weight").and_then(|v| v.as_str()) {
                for (id, item) in weights {
                    let _ = item.set_checked(*id == w);
                }
            }
        }
    }
}

#[tauri::command]
fn log_from_js(msg: String) {
    println!("[JS LOG] {}", msg);
    log_to_file(&format!("[JS] {}", msg));
}

#[tauri::command]
fn set_ignore_mouse(app: tauri::AppHandle, ignore: bool) {
    IS_IGNORING_MOUSE.store(ignore, Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_ignore_cursor_events(ignore);
    }
}

#[tauri::command]
fn save_state(state: serde_json::Value) {
    let to_save = if let Ok(mut s) = APP_STATE.lock() {
        let mut cur = s.take().unwrap_or_else(|| serde_json::json!({}));
        merge_json(&mut cur, &state);
        *s = Some(cur.clone());
        cur
    } else {
        state
    };
    atomic_save_state(&to_save);
}

#[tauri::command]
fn load_state() -> Option<serde_json::Value> {
    let path = get_state_file_path();
    if let Ok(data) = std::fs::read_to_string(path) {
        let val: Option<serde_json::Value> = serde_json::from_str(&data).ok();
        if let Some(ref state) = val {
            if let Ok(mut s) = APP_STATE.lock() {
                let mut cur = s.take().unwrap_or_else(|| serde_json::json!({}));
                merge_json(&mut cur, state);
                *s = Some(cur);
            }
            update_tray_checks(state);
        }
        val
    } else {
        None
    }
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    log_to_file("quit_app() invoked!");
    if let Ok(s) = APP_STATE.lock() {
        if let Some(ref state) = *s {
            atomic_save_state(state);
        }
    }
    let _ = app.exit(0);
    std::process::exit(0);
}

#[tauri::command]
fn report_state(app: tauri::AppHandle, state: serde_json::Value) {
    if let Ok(mut s) = APP_STATE.lock() {
        let mut cur = s.take().unwrap_or_else(|| serde_json::json!({}));
        merge_json(&mut cur, &state);
        *s = Some(cur);
    }
    update_tray_checks(&state);
    let _ = app.emit("carpet_state_changed", state);
}

#[tauri::command]
fn get_current_state() -> Option<serde_json::Value> {
    APP_STATE.lock().ok().and_then(|s| s.clone())
}

fn to_base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn read_reg_sz(sub_key: &str, value_name: &str) -> Option<String> {
    unsafe {
        let sub_key_w: Vec<u16> = sub_key.encode_utf16().chain(std::iter::once(0)).collect();
        let value_w: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut hkey: isize = 0;
        const HKEY_CURRENT_USER: isize = 0x80000001u32 as i32 as isize;
        const KEY_READ: u32 = 0x20019;
        if win32::RegOpenKeyExW(HKEY_CURRENT_USER, sub_key_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
            return None;
        }
        let mut buf_len: u32 = 512;
        let mut buf = vec![0u8; buf_len as usize];
        let mut val_type: u32 = 0;
        let res = win32::RegQueryValueExW(
            hkey,
            value_w.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            buf.as_mut_ptr(),
            &mut buf_len,
        );
        win32::RegCloseKey(hkey);
        if res == 0 && buf_len > 0 {
            let u16_slice: &[u16] = std::slice::from_raw_parts(buf.as_ptr() as *const u16, (buf_len / 2) as usize);
            let s = String::from_utf16_lossy(u16_slice);
            return Some(s.trim_end_matches('\0').trim().to_string());
        }
        None
    }
}

#[cfg(target_os = "windows")]
fn get_wallpaper_mode() -> &'static str {
    let style = read_reg_sz("Control Panel\\Desktop", "WallpaperStyle").unwrap_or_default();
    let tile = read_reg_sz("Control Panel\\Desktop", "TileWallpaper").unwrap_or_default();
    match style.as_str() {
        "10" => "fill",
        "6" => "fit",
        "2" => "stretch",
        "22" => "fill",
        "0" => {
            if tile == "1" {
                "tile"
            } else {
                "center"
            }
        }
        _ => {
            if tile == "1" {
                "tile"
            } else {
                "fill"
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn get_wallpaper_color() -> String {
    if let Some(s) = read_reg_sz("Control Panel\\Colors", "Background") {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            ) {
                return format!("#{:02x}{:02x}{:02x}", r, g, b);
            }
        }
    }
    "#000000".to_string()
}

fn get_screen_bounds() -> (i32, i32, i32, i32) {
    #[cfg(target_os = "windows")]
    unsafe {
        let vx = win32::GetSystemMetrics(win32::SM_XVIRTUALSCREEN);
        let vy = win32::GetSystemMetrics(win32::SM_YVIRTUALSCREEN);
        let vw = win32::GetSystemMetrics(win32::SM_CXVIRTUALSCREEN);
        let vh = win32::GetSystemMetrics(win32::SM_CYVIRTUALSCREEN);
        if vw > 0 && vh > 0 {
            (vx, vy, vw, vh)
        } else {
            (0, 0, win32::GetSystemMetrics(win32::SM_CXSCREEN), win32::GetSystemMetrics(win32::SM_CYSCREEN))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        (0, 0, 1920, 1080)
    }
}

fn get_wallpaper_info() -> Result<serde_json::Value, String> {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        let path = std::path::PathBuf::from(app_data)
            .join("Microsoft")
            .join("Windows")
            .join("Themes")
            .join("TranscodedWallpaper");
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                let (x, y, w, h) = get_screen_bounds();
                #[cfg(target_os = "windows")]
                let mode = get_wallpaper_mode();
                #[cfg(not(target_os = "windows"))]
                let mode = "fill";
                #[cfg(target_os = "windows")]
                let color = get_wallpaper_color();
                #[cfg(not(target_os = "windows"))]
                let color = "#000000".to_string();

                let data_uri = format!("data:image/jpeg;base64,{}", to_base64(&bytes));
                return Ok(serde_json::json!({
                    "dataUri": data_uri,
                    "mode": mode,
                    "color": color,
                    "bounds": { "x": x, "y": y, "width": w, "height": h },
                    "workArea": { "x": x, "y": y, "width": w, "height": h }
                }));
            }
        }
    }
    Err("Wallpaper not found".into())
}

#[tauri::command]
fn get_wallpaper() -> Result<serde_json::Value, String> {
    get_wallpaper_info()
}

const SIZES_LIST: &[(&str, &str)] = &[
    ("kucuk", "Small"),
    ("orta", "Medium"),
    ("buyuk", "Large"),
];

const WEIGHTS_LIST: &[(&str, &str)] = &[
    ("hafif", "Light"),
    ("normal", "Normal"),
    ("agir", "Heavy"),
];

fn build_native_tray_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let flatten = MenuItem::with_id(app, "flatten", "Flatten Rug", true, None::<&str>)?;
    let center = MenuItem::with_id(app, "center", "Center Rug", true, None::<&str>)?;

    let cur = APP_STATE.lock().ok().and_then(|s| s.clone()).unwrap_or_else(|| {
        serde_json::json!({ "style": "tabriz", "size": "orta", "weight": "normal" })
    });
    let cur_style = cur.get("style").and_then(|v| v.as_str()).unwrap_or("tabriz");
    let cur_size = cur.get("size").and_then(|v| v.as_str()).unwrap_or("orta");
    let cur_weight = cur.get("weight").and_then(|v| v.as_str()).unwrap_or("normal");

    let mut style_items = Vec::new();
    let mut root_ptrs = Vec::new();

    // 1. Direct root styles (Tabriz, Classical Red, Indigo, Emerald, Kilim, Invisible)
    for &(id, label) in tray_catalog::ROOT_STYLES.iter() {
        let is_checked = id == cur_style;
        let menu_id = format!("style:{}", id);
        let item = CheckMenuItem::with_id(app, menu_id, label, true, is_checked, None::<&str>)?;
        style_items.push((id, item.clone()));
        root_ptrs.push(item);
    }

    let root_sep = PredefinedMenuItem::separator(app)?;

    // 2. Categories with submenus
    let mut cat_subs = Vec::new();
    for cat in tray_catalog::TRAY_CATEGORIES {
        if !cat.subgroups.is_empty() {
            let mut sub_subs = Vec::new();
            for sub in cat.subgroups.iter() {
                let mut ptrs = Vec::new();
                for &(id, label) in sub.styles.iter() {
                    let is_checked = id == cur_style;
                    let menu_id = format!("style:{}", id);
                    let item = CheckMenuItem::with_id(app, menu_id, label, true, is_checked, None::<&str>)?;
                    style_items.push((id, item.clone()));
                    ptrs.push(item);
                }
                let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = ptrs
                    .iter()
                    .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                    .collect();
                let sg_sub = Submenu::with_items(app, sub.title, true, &refs)?;
                sub_subs.push(sg_sub);
            }
            let sub_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = sub_subs
                .iter()
                .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                .collect();
            let cat_sub = Submenu::with_items(app, cat.title, true, &sub_refs)?;
            cat_subs.push(cat_sub);
        } else {
            let mut ptrs = Vec::new();
            for &(id, label) in cat.styles.iter() {
                let is_checked = id == cur_style;
                let menu_id = format!("style:{}", id);
                let item = CheckMenuItem::with_id(app, menu_id, label, true, is_checked, None::<&str>)?;
                style_items.push((id, item.clone()));
                ptrs.push(item);
            }
            let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = ptrs
                .iter()
                .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
                .collect();
            let cat_sub = Submenu::with_items(app, cat.title, true, &refs)?;
            cat_subs.push(cat_sub);
        }
    }

    let mut pattern_menu_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = Vec::new();
    for item in &root_ptrs {
        pattern_menu_refs.push(item as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    }
    pattern_menu_refs.push(&root_sep as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    for cat in &cat_subs {
        pattern_menu_refs.push(cat as &dyn tauri::menu::IsMenuItem<tauri::Wry>);
    }

    let styles_sub = Submenu::with_items(app, "Pattern", true, &pattern_menu_refs)?;

    let mut size_items = Vec::new();
    let mut size_ptrs = Vec::new();
    for &(id, label) in SIZES_LIST {
        let is_checked = id == cur_size;
        let menu_id = format!("size:{}", id);
        let item = CheckMenuItem::with_id(app, menu_id, label, true, is_checked, None::<&str>)?;
        size_items.push((id, item.clone()));
        size_ptrs.push(item);
    }
    let size_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = size_ptrs
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();
    let sizes_sub = Submenu::with_items(app, "Size", true, &size_refs)?;

    let mut weight_items = Vec::new();
    let mut weight_ptrs = Vec::new();
    for &(id, label) in WEIGHTS_LIST {
        let is_checked = id == cur_weight;
        let menu_id = format!("weight:{}", id);
        let item = CheckMenuItem::with_id(app, menu_id, label, true, is_checked, None::<&str>)?;
        weight_items.push((id, item.clone()));
        weight_ptrs.push(item);
    }
    let weight_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = weight_ptrs
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();
    let weights_sub = Submenu::with_items(app, "Weight", true, &weight_refs)?;

    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

    if let Ok(mut g) = TRAY_ITEMS.lock() {
        *g = Some((style_items, size_items, weight_items));
    }

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
    #[cfg(target_os = "windows")]
    {
        unsafe {
            win32::timeBeginPeriod(1);
        }
        disable_power_throttling();
    }

    unsafe {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--enable-gpu-rasterization --enable-zero-copy --ignore-gpu-blocklist",
        );
    }

    let builder = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            log_from_js,
            set_ignore_mouse,
            save_state,
            load_state,
            report_state,
            get_current_state,
            quit_app,
            get_wallpaper
        ]);

    let app = builder
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Load saved state early so tray reflects it
            let _ = load_state();

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
                        let _ = app.emit("command", serde_json::json!({ "cmd": "save", "arg": null }));
                        let app_handle = app.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(60));
                            quit_app(app_handle);
                        });
                    } else if id == "flatten" {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "flatten", "arg": null }));
                    } else if id == "center" {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "center", "arg": null }));
                    } else if let Some(style) = id.strip_prefix("style:") {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "style", "arg": style }));
                        if let Ok(mut s) = APP_STATE.lock() {
                            let mut val = s.clone().unwrap_or_else(|| serde_json::json!({}));
                            val["style"] = serde_json::json!(style);
                            *s = Some(val.clone());
                            update_tray_checks(&val);
                            atomic_save_state(&val);
                        }
                    } else if let Some(size) = id.strip_prefix("size:") {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "size", "arg": size }));
                        if let Ok(mut s) = APP_STATE.lock() {
                            let mut val = s.clone().unwrap_or_else(|| serde_json::json!({}));
                            val["size"] = serde_json::json!(size);
                            *s = Some(val.clone());
                            update_tray_checks(&val);
                            atomic_save_state(&val);
                        }
                    } else if let Some(weight) = id.strip_prefix("weight:") {
                        let _ = app.emit("command", serde_json::json!({ "cmd": "weight", "arg": weight }));
                        if let Ok(mut s) = APP_STATE.lock() {
                            let mut val = s.clone().unwrap_or_else(|| serde_json::json!({}));
                            val["weight"] = serde_json::json!(weight);
                            *s = Some(val.clone());
                            update_tray_checks(&val);
                            atomic_save_state(&val);
                        }
                    }
                });

            if let Some(ref menu) = tray_menu {
                tray_b = tray_b.menu(menu);
            }

            let tray_icon = app.default_window_icon().cloned()
                .or_else(|| tauri::image::Image::from_app_icon_resource(32).ok());

            if let Some(icon) = tray_icon {
                tray_b = tray_b.icon(icon);
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
                // Initially enable click-through and show window
                let _ = window.set_ignore_cursor_events(true);
                let _ = window.show();

                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd_raw) = window.hwnd() {
                        let hwnd = hwnd_raw.0 as win32::HWND;
                        unsafe {
                            let (x, y, w, h) = get_screen_bounds();
                            win32::SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                x,
                                y,
                                w,
                                h,
                                win32::SWP_NOACTIVATE | win32::SWP_SHOWWINDOW | win32::SWP_FRAMECHANGED,
                            );
                            win32::ShowWindow(hwnd, win32::SW_SHOWNOACTIVATE);
                        }
                    }

                    // Dynamic wallpaper watcher (only checks when invisible pattern is selected)
                    let handle_wallpaper = app_handle.clone();
                    std::thread::spawn(move || {
                        if let Some(app_data) = std::env::var_os("APPDATA") {
                            let path = std::path::PathBuf::from(app_data)
                                .join("Microsoft")
                                .join("Windows")
                                .join("Themes")
                                .join("TranscodedWallpaper");
                            let mut last_mtime = std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());

                            loop {
                                std::thread::sleep(std::time::Duration::from_secs(5));
                                let is_invisible = APP_STATE.lock().ok()
                                    .and_then(|s| s.as_ref().and_then(|v| v.get("style").and_then(|st| st.as_str()).map(|st| st == "invisible")))
                                    .unwrap_or(false);

                                if is_invisible {
                                    if let Ok(meta) = std::fs::metadata(&path) {
                                        if let Ok(mtime) = meta.modified() {
                                            if last_mtime.map_or(true, |last| mtime > last) {
                                                last_mtime = Some(mtime);
                                                if let Ok(info) = get_wallpaper_info() {
                                                    let _ = handle_wallpaper.emit("command", serde_json::json!({ "cmd": "wallpaper", "arg": info }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });

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

                    // Cursor hover and button detection thread (matches upstream electron poller):
                    // When window is in click-through mode, this detects hover over the rug and initial click.
                    // When dragging/hovering, DOM pointer events handle 100% of movement with 0 IPC overhead.
                    let handle_clone2 = app_handle.clone();
                    let window_clone = window.clone();
                    let hwnd_poll_val = window.hwnd().map(|h| h.0 as isize).unwrap_or(0);
                    std::thread::spawn(move || {
                        let mut last_cursor = String::new();

                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(16));

                            unsafe {
                                let mut pt = win32::POINT::default();
                                if win32::GetCursorPos(&mut pt) == 0 {
                                    continue;
                                }

                                let down = left_button_down();
                                let key = format!("{},{},{}", pt.x, pt.y, down);
                                if key == last_cursor {
                                    continue;
                                }

                                let is_ignoring = IS_IGNORING_MOUSE.load(Ordering::Relaxed);
                                // When window is active (hovered or grabbing), DOM events handle movement directly!
                                // We only need to notify if mouse was released (safety net)
                                if !is_ignoring && down {
                                    continue;
                                }

                                let pressed = down && !last_cursor.ends_with(",true");
                                last_cursor = key;

                                let free = if pressed {
                                    point_on_desktop(pt, hwnd_poll_val as win32::HWND)
                                } else {
                                    true
                                };

                                let scale_factor = window_clone.scale_factor().unwrap_or(1.0);
                                let css_x = pt.x as f64 / scale_factor;
                                let css_y = pt.y as f64 / scale_factor;

                                let _ = handle_clone2.emit(
                                    "command",
                                    serde_json::json!({
                                        "cmd": "cursor",
                                        "arg": {
                                            "x": css_x,
                                            "y": css_y,
                                            "down": down,
                                            "free": free
                                        }
                                    }),
                                );
                            }
                        }
                    });
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
            _ => {}
        }
    });
}
