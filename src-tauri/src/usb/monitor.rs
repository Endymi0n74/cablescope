// ─── USB Device Change Monitor ─────────────────────────────────────
// Real-time monitoring via hidden HWND + RegisterDeviceNotificationW.
// Emits Tauri events when devices are plugged/unplugged.

use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub event_type: String, // "connected" | "disconnected"
    pub timestamp: String,
}

// ─── Win32 FFI Types ──────────────────────────────────────────────

type HWND = *mut c_void;
type HDEVNOTIFY = *mut c_void;
type LRESULT = isize;
type WPARAM = usize;
type LPARAM = isize;

const WM_DEVICECHANGE: u32 = 0x0219;
const DBT_DEVICEARRIVAL: u32 = 0x8000;
const DBT_DEVICEREMOVECOMPLETE: u32 = 0x8004;
const DBT_DEVTYP_DEVICEINTERFACE: u32 = 0x00000005;
const DEVICE_NOTIFY_WINDOW_HANDLE: u32 = 0x00000000;

#[repr(C)]
struct DevBroadcastDeviceInterfaceW {
    dbcc_size: u32,
    dbcc_devicetype: u32,
    dbcc_reserved: u16,
    dbcc_classguid: [u8; 16],
    dbcc_name: [u16; 1],
}

// GUID_NULL: receive notifications for ALL device interface classes.
// (Some machines/VMs register non-standard USB interface GUIDs, so we
// cannot rely on a hardcoded USB class GUID here.)
const USB_DEVICE_INTERFACE_GUID: [u8; 16] = [0u8; 16];

#[cfg(windows)]
extern "system" {
    fn RegisterClassExW(lpWndClass: *const WndClassExW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16,
        dwStyle: u32, x: i32, y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND, hMenu: *mut c_void, hInstance: *mut c_void,
        lpParam: *mut c_void,
    ) -> HWND;
    fn DestroyWindow(hWnd: HWND) -> i32;
    fn DefWindowProcW(hWnd: HWND, msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn RegisterDeviceNotificationW(
        hRecipient: HANDLE, lpNotificationFilter: *const c_void,
        dwFlags: u32,
    ) -> HDEVNOTIFY;
    fn UnregisterDeviceNotification(hHandle: HDEVNOTIFY) -> i32;
    fn GetMessageW(lpMsg: *mut Msg, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const Msg) -> i32;
    fn DispatchMessageW(lpMsg: *const Msg) -> LRESULT;
    fn GetLastError() -> u32;
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut c_void;
}

type HANDLE = *mut c_void;

#[repr(C)]
struct WndClassExW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: WndProc,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut c_void,
    h_icon: HWND,
    h_cursor: HWND,
    hbr_background: *mut c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: HWND,
}

type WndProc = extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
struct Msg {
    hwnd: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
    time: u32,
    pt: Point,
}

#[repr(C)]
#[derive(Default)]
struct Point {
    x: i32,
    y: i32,
}

// ─── Event Storage ────────────────────────────────────────────────

static EVENT_TX: OnceLock<std::sync::mpsc::Sender<DeviceEvent>> = OnceLock::new();

/// Start the device monitor in a background thread.
/// Returns a receiver that yields DeviceEvents whenever a USB device is
/// connected or disconnected. The Tauri app should poll / crossbeam-recv.
pub fn start_monitor() -> std::sync::mpsc::Receiver<DeviceEvent> {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        unsafe { monitor_thread_main(tx) };
    });

    rx
}

// ─── Monitor Thread Implementation ────────────────────────────────

unsafe fn monitor_thread_main(tx: std::sync::mpsc::Sender<DeviceEvent>) {
    // Store sender for window_proc access
    let _ = EVENT_TX.set(tx);

    let class_name = wide("CableScopeDeviceMonitor");

    let h_instance = GetModuleHandleW(std::ptr::null());

    let wnd_class = WndClassExW {
        cb_size: std::mem::size_of::<WndClassExW>() as u32,
        style: 0,
        lpfn_wnd_proc: window_proc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance,
        h_icon: std::ptr::null_mut(),
        h_cursor: std::ptr::null_mut(),
        hbr_background: std::ptr::null_mut(),
        lpsz_menu_name: std::ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
        h_icon_sm: std::ptr::null_mut(),
    };

    let atom = RegisterClassExW(&wnd_class);
    if atom == 0 {
        let err = GetLastError();
        eprintln!("[CableScope] RegisterClassExW failed: {}", err);
        return;
    }

    // Create hidden window
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        wide("CableScope Monitor").as_ptr(),
        0,
        0, 0, 0, 0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        h_instance,
        std::ptr::null_mut(),
    );

    if hwnd.is_null() {
        let err = GetLastError();
        eprintln!("[CableScope] CreateWindowExW failed: {}", err);
        return;
    }

    // Register for USB device interface notifications
    let filter_size = std::mem::size_of::<DevBroadcastDeviceInterfaceW>() as u32;
    let mut filter: DevBroadcastDeviceInterfaceW = std::mem::zeroed();
    filter.dbcc_size = filter_size;
    filter.dbcc_devicetype = DBT_DEVTYP_DEVICEINTERFACE;
    filter.dbcc_classguid = USB_DEVICE_INTERFACE_GUID;

    let h_notify = RegisterDeviceNotificationW(
        hwnd,
        &filter as *const _ as *const c_void,
        DEVICE_NOTIFY_WINDOW_HANDLE,
    );

    if h_notify.is_null() {
        let err = GetLastError();
        eprintln!("[CableScope] RegisterDeviceNotificationW failed: {}", err);
        DestroyWindow(hwnd);
        return;
    }

    log_info(&format!(
        "[CableScope] Device monitor started (thread {})",
        std::thread::current().name().unwrap_or("?")
    ));

    // Message loop
    let mut msg: Msg = std::mem::zeroed();
    while GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    // Cleanup
    UnregisterDeviceNotification(h_notify);
    DestroyWindow(hwnd);
    log_info("[CableScope] Device monitor stopped");
}

extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    _w_param: WPARAM,
    _l_param: LPARAM,
) -> LRESULT {
    if msg == WM_DEVICECHANGE {
        let w = _w_param as u32;
        if w == DBT_DEVICEARRIVAL || w == DBT_DEVICEREMOVECOMPLETE {
            let event_type = if w == DBT_DEVICEARRIVAL {
                "connected"
            } else {
                "disconnected"
            };

            let event = DeviceEvent {
                event_type: event_type.to_string(),
                timestamp: timestamp_str(),
            };

            log_info(&format!("[CableScope] Device {}", event.event_type));

            // Send event to the main thread via channel
            if let Some(tx) = EVENT_TX.get() {
                let _ = tx.send(event);
            }
        }
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, _w_param, _l_param) }
}

// ─── Helpers ──────────────────────────────────────────────────────

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn timestamp_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn log_info(msg: &str) {
    // Write to our log file
    super::super::logger::log_file(msg);
}
