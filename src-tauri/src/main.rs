#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
static mut MUTEX_HANDLE: *mut std::ffi::c_void = std::ptr::null_mut();

#[cfg(target_os = "windows")]
fn is_another_instance_running() -> bool {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateMutexW(lpMutexAttributes: *mut c_void, bInitialOwner: i32, lpName: *const u16) -> *mut c_void;
        fn GetLastError() -> u32;
    }
    // Session-local namespace without Global\ prefix so standard user rights work cleanly
    let name: Vec<u16> = "DeskRug_Tauri_Singleton\0".encode_utf16().collect();
    unsafe {
        let handle = CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr());
        if !handle.is_null() && GetLastError() == 183 {
            return true;
        }
        MUTEX_HANDLE = handle;
    }
    false
}

fn cleanup_old_temp_folders() {
    let temp = std::env::temp_dir();
    if let Ok(entries) = std::fs::read_dir(&temp) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("deskrug-wv-") {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    if is_another_instance_running() {
        std::process::exit(0);
    }

    // Clean up any stale temp webview directories from previous runs
    cleanup_old_temp_folders();

    // Give this process its own isolated WebView2 directory to guarantee 0x800700AA never happens
    let pid = std::process::id();
    let wv_dir = std::env::temp_dir().join(format!("deskrug-wv-{}", pid));
    let _ = std::fs::create_dir_all(&wv_dir);
    unsafe {
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &wv_dir);
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--enable-gpu-rasterization --enable-zero-copy --ignore-gpu-blocklist",
        );
    }

    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let mut p = std::env::temp_dir();
        p.push("deskrug-panic.log");
        let _ = std::fs::write(&p, &msg);
        eprintln!("{}", msg);
    }));

    deskrug_lib::run()
}
