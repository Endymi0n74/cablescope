mod usb;
mod devices;
mod ucsi;
mod logger;

use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter,
    Manager,
};
use tokio::sync::RwLock;
use usb::UsbSnapshot;

struct AppState {
    last_snapshot: Arc<RwLock<UsbSnapshot>>,
}

// ─── Tauri Commands ───────────────────────────────────────────────

/// Full scan of all USB ports and connected devices
#[tauri::command]
fn scan_usb(state: tauri::State<'_, AppState>) -> Result<UsbSnapshot, String> {
    logger::log_file("CMD scan_usb");
    let result = usb::full_scan();
    match &result {
        Ok(snap) => {
            // Cache the latest snapshot in shared state (used by notifications/reports)
            let mut guard = state.last_snapshot.blocking_write();
            *guard = snap.clone();
            drop(guard);
            logger::log_file(&format!(
                "CMD scan_usb: {} controllers, {} ports, {} devices",
                snap.controllers.len(),
                snap.ports.len(),
                snap.devices.len()
            ));
        }
        Err(e) => logger::log_file(&format!("CMD scan_usb ERROR: {}", e)),
    }
    result.map_err(|e| e.to_string())
}

/// Targeted re-scan of a single hub/controller (ports + linked devices)
#[tauri::command]
fn scan_hub(name: String) -> Result<usb::HubScan, String> {
    logger::log_file(&format!("CMD scan_hub: {}", name));
    let snap = usb::full_scan().map_err(|e| e.to_string())?;
    let hub_scan = usb::filter_scan_for_hub(snap, &name);
    logger::log_file(&format!(
        "CMD scan_hub: {} ports, {} devices for {}",
        hub_scan.ports.len(),
        hub_scan.devices.len(),
        name
    ));
    Ok(hub_scan)
}

/// Get UCSI connector info for a specific port (if supported)
#[tauri::command]
fn get_connector_info(port_index: u32) -> Result<ucsi::ConnectorInfo, String> {
    logger::log_file(&format!("CMD get_connector_info: port {}", port_index));
    ucsi::get_connector_status(port_index).map_err(|e| e.to_string())
}

/// Look up a device by VID:PID in the database
#[tauri::command]
fn lookup_device(vid: u16, pid: u16) -> Result<devices::DeviceInfo, String> {
    Ok(devices::lookup(vid, pid))
}

/// Get app settings from local storage
#[tauri::command]
fn get_settings() -> Result<String, String> {
    let path = settings_path();
    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!({
            "refreshInterval": 3,
            "notifications": true,
            "hideEmptyPorts": true,
            "launchAtStartup": false
        }).to_string())
    }
}

/// Save app settings
#[tauri::command]
fn save_settings(json: String) -> Result<(), String> {
    let path = settings_path();
    let dir = path.parent().unwrap();
    let _ = std::fs::create_dir_all(dir);
    std::fs::write(&path, &json).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_logs() -> Result<String, String> {
    Ok(logger::read_today_log())
}

// ─── App Entry ────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Single-instance check
    {
        let lock_path = lock_path();
        if lock_path.exists() {
            if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if is_process_alive(pid) {
                        logger::init();
                        logger::log_file("Another instance already running, exiting.");
                        return;
                    }
                }
            }
            let _ = std::fs::remove_file(&lock_path);
        }
        let _ = std::fs::write(&lock_path, std::process::id().to_string());
    }

    logger::init();
    logger::log_file("CableScope starting...");

    let last_snapshot = Arc::new(RwLock::new(UsbSnapshot::default()));

    tauri::Builder::default()
        .manage(AppState {
            last_snapshot: last_snapshot.clone(),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            scan_usb,
            scan_hub,
            get_connector_info,
            lookup_device,
            get_settings,
            save_settings,
            get_logs,
        ])
        .setup(move |app| {
            // ── System Tray ──
            let show_i = MenuItemBuilder::with_id("show", "Afficher CableScope")
                .build(app)?;
            let quit_i = MenuItemBuilder::with_id("quit", "Quitter")
                .build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_i)
                .separator()
                .item(&quit_i)
                .build()?;

            let icon_bytes = include_bytes!("../icons/icon.png");
            let tray_icon = Image::from_bytes(icon_bytes)
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("CableScope — USB-C Inspector")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        let lock = lock_path();
                        let _ = std::fs::remove_file(&lock);
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.destroy();
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // X button hides the window
            if let Some(window) = app.get_webview_window("main") {
                let w_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w_clone.hide();
                    }
                });
            }

            // ── Start USB Device Monitor ──
            {
                let rx = usb::monitor::start_monitor();
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    logger::log_file("Device monitor thread started");
                    while let Ok(evt) = rx.recv() {
                        logger::log_file(&format!(
                            "Device event: {} at {}",
                            evt.event_type, evt.timestamp
                        ));
                        // Emit event to frontend so it can auto-scan
                        let _ = app_handle.emit("device-change", &evt);
                    }
                    logger::log_file("Device monitor thread ended");
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running CableScope");
}

// ─── Helpers ──────────────────────────────────────────────────────

fn lock_path() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(base)
        .join("CableScope")
        .join("app.lock")
}

fn settings_path() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(base)
        .join("CableScope")
        .join("settings.json")
}

#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    extern "system" {
        fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> *mut core::ffi::c_void;
        fn CloseHandle(h: *mut core::ffi::c_void) -> i32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !h.is_null() && h != -1isize as *mut core::ffi::c_void {
            CloseHandle(h);
            return true;
        }
    }
    false
}

#[cfg(not(windows))]
fn is_process_alive(_pid: u32) -> bool {
    false
}
