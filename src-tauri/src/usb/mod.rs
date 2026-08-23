// ─── USB Port & Device Enumeration for Windows ─────────────────────
// Uses raw Win32 FFI for SetupAPI + DeviceIoControl.

pub mod monitor;

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

// ─── Data Types ───────────────────────────────────────────────────

/// Filter a snapshot down to one controller: its ports plus the devices
/// linked to those ports (matched by port_id). Pure function, used by the
/// scan_hub command and covered by unit tests.
pub fn filter_scan_for_hub(snap: UsbSnapshot, name: &str) -> HubScan {
    let ports: Vec<UsbPort> = snap
        .ports
        .into_iter()
        .filter(|p| p.controller_name == name)
        .collect();
    let port_ids: std::collections::HashSet<String> =
        ports.iter().map(|p| p.id.clone()).collect();
    let devices: Vec<UsbDevice> = snap
        .devices
        .into_iter()
        .filter(|d| port_ids.contains(&d.port_id))
        .collect();
    HubScan { ports, devices }
}

/// Subset of a snapshot limited to one controller (targeted re-scan).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubScan {
    pub ports: Vec<UsbPort>,
    pub devices: Vec<UsbDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsbSnapshot {
    pub controllers: Vec<UsbController>,
    pub ports: Vec<UsbPort>,
    pub devices: Vec<UsbDevice>,
    pub scan_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbController {
    pub name: String,
    pub hub_path: String,
    pub port_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsbPort {
    pub id: String,
    pub controller_name: String,
    pub hub_path: String,
    pub port_number: u32,
    pub connected: bool,
    pub speed: String,
    pub speed_value: u32,
    pub device_address: u32,
    pub open_pipes: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsbDevice {
    pub port_id: String,
    pub power_role: String,
    pub vid: u16,
    pub pid: u16,
    pub bcd_usb: u16,
    pub bcd_device: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub manufacturer: String,
    pub product: String,
    pub serial: String,
    pub hub_name: String,
    pub port_number: u32,
    pub driver_key: String,
    pub friendly_name: String,
    pub device_class_name: String,
    pub usb_version: String,
    pub speed: String,
}

// ─── Raw Win32 FFI ────────────────────────────────────────────────

type HANDLE = *mut core::ffi::c_void;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const NULL: *mut core::ffi::c_void = core::ptr::null_mut();

const GENERIC_READ: u32 = 0x80000000;
const OPEN_EXISTING: u32 = 3;
const DIGCF_PRESENT: u32 = 0x02;
const DIGCF_ALLCLASSES: u32 = 0x04;
const DIGCF_DEVICEINTERFACE: u32 = 0x10;

// IOCTL codes probed empirically on this system's USB stack
const IOCTL_USB_GET_NODE_INFORMATION: u32 = 0x220408;
const IOCTL_USB_GET_PORT_STATUS: u32 = 0x220010;


#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct SpDeviceInterfaceData {
    cb_size: u32,
    interface_class_guid: GUID,
    _flags: u32,
    _reserved: usize,
}

// SP_DEVINFO_DATA: cbSize(4) + ClassGuid(16) + DevInst(4) + Reserved(8) = 32 on x64
#[repr(C)]
struct SpDevInfoData {
    cb_size: u32,
    class_guid: GUID,
    _dev_inst: u32,
    _reserved: usize,
}

#[repr(C)]
struct SpDeviceInfoDetailData {
    cb_size: u32,
    _dev_path: [u16; 520], // MAX_PATH + some
}

// USB Node Information
const NODE_INFO_BUF_SIZE: usize = 512;

#[cfg(windows)]
extern "system" {
    fn CreateFileW(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: *mut core::ffi::c_void,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: HANDLE,
    ) -> HANDLE;

    fn CloseHandle(h_object: HANDLE) -> i32;

    fn CM_Get_Parent(pdn_dev_inst: *mut u32, dn_dev_inst: u32, ul_flags: u32) -> u32;
    fn CM_Get_Device_IDW(dn_dev_inst: u32, buffer: *mut u16, buffer_len: u32, ul_flags: u32) -> u32;

    fn DeviceIoControl(
        h_device: HANDLE,
        dw_io_control_code: u32,
        lp_in_buffer: *const core::ffi::c_void,
        n_in_buffer_size: u32,
        lp_out_buffer: *mut core::ffi::c_void,
        n_out_buffer_size: u32,
        lp_bytes_returned: *mut u32,
        lp_overlapped: *mut core::ffi::c_void,
    ) -> i32;

    fn SetupDiGetClassDevsW(
        class_guid: *const GUID,
        enumerator: *const u16,
        hwnd_parent: HANDLE,
        flags: u32,
    ) -> HANDLE;

    fn SetupDiEnumDeviceInterfaces(
        device_info_set: HANDLE,
        device_info_data: *mut core::ffi::c_void,
        interface_class_guid: *const GUID,
        member_index: u32,
        device_interface_data: *mut SpDeviceInterfaceData,
    ) -> i32;

    fn SetupDiGetDeviceInterfaceDetailW(
        device_info_set: HANDLE,
        device_interface_data: *const SpDeviceInterfaceData,
        device_interface_detail_data: *mut SpDeviceInfoDetailData,
        device_interface_detail_data_size: u32,
        required_size: *mut u32,
        device_info_data: *mut core::ffi::c_void,
    ) -> i32;

    fn SetupDiEnumDeviceInfo(
        device_info_set: HANDLE,
        member_index: u32,
        device_info_data: *mut SpDevInfoData,
    ) -> i32;

    fn SetupDiGetDeviceInstanceIdW(
        device_info_set: HANDLE,
        device_info_data: *const SpDevInfoData,
        device_instance_id: *mut u16,
        device_instance_id_size: u32,
        required_size: *mut u32,
    ) -> i32;

    fn SetupDiGetDeviceRegistryPropertyW(
        device_info_set: HANDLE,
        device_info_data: *const SpDevInfoData,
        property: u32,
        property_reg_data_type: *mut u32,
        property_buffer: *mut u8,
        property_buffer_size: u32,
        required_size: *mut u32,
    ) -> i32;

    fn RegOpenKeyExW(
        h_key: HANDLE,
        lp_sub_key: *const u16,
        ul_options: u32,
        sam_desired: u32,
        phk_result: *mut HANDLE,
    ) -> i32;

    fn RegEnumKeyExW(
        h_key: HANDLE,
        dw_index: u32,
        lp_name: *mut u16,
        lpcch_name: *mut u32,
        lp_reserved: *mut u32,
        lp_class: *mut u16,
        lpcch_class: *mut u32,
        lpft_last_write_time: *mut core::ffi::c_void,
    ) -> i32;

    fn RegCloseKey(h_key: HANDLE) -> i32;

    fn SetupDiDestroyDeviceInfoList(device_info_set: HANDLE) -> i32;

    fn GetLastError() -> u32;
}

unsafe fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn open_hub(hub_path: &str) -> Result<HANDLE> {
    let wide = wide_string(hub_path);
    let h = CreateFileW(
        wide.as_ptr(),
        GENERIC_READ,
        0,
        NULL,
        OPEN_EXISTING,
        0,
        NULL,
    );
    if h == INVALID_HANDLE_VALUE || h.is_null() {
        let err = GetLastError();
        anyhow::bail!("Failed to open hub {}: Win32 error {}", hub_path, err);
    }
    Ok(h)
}

unsafe fn close_handle(h: HANDLE) {
    if !h.is_null() && h != INVALID_HANDLE_VALUE {
        CloseHandle(h);
    }
}

// ─── SetupAPI Enumeration ─────────────────────────────────────────



fn enumerate_controllers() -> Result<Vec<UsbController>> {
    // Discover the hub interface class GUIDs this system actually uses
    // (some machines/VMs register non-standard GUIDs), then enumerate via
    // SetupAPI which returns proper device interface paths.
    let mut hub_guids = discover_hub_interface_guids();
    super::logger::log_file(&format!(
        "[usb] Discovered {} hub interface GUIDs",
        hub_guids.len()
    ));

    if hub_guids.is_empty() {
        // Fallback: standard Microsoft USB hub interface GUID
        hub_guids.push(GUID {
            data1: 0xf18a0e88,
            data2: 0xc30c,
            data3: 0x11d0,
            data4: [0x81, 0xe9, 0x00, 0xa0, 0xc9, 0x1e, 0xeb, 0x34],
        });
    }

    let mut hub_paths: Vec<String> = Vec::new();
    for guid in &hub_guids {
        unsafe {
            let h_dev_info = SetupDiGetClassDevsW(
                guid,
                core::ptr::null(),
                NULL,
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            );
            if h_dev_info == INVALID_HANDLE_VALUE || h_dev_info.is_null() {
                super::logger::log_file("[usb] GetClassDevs failed for a hub GUID");
                continue;
            }
            let paths = enumerate_hub_paths_for_guid(h_dev_info, guid);
            SetupDiDestroyDeviceInfoList(h_dev_info);
            for p in paths {
                if !hub_paths.contains(&p) {
                    hub_paths.push(p);
                }
            }
        }
    }

    super::logger::log_file(&format!(
        "[usb] Found {} hub interface paths",
        hub_paths.len()
    ));

    let mut controllers = Vec::new();
    for (i, path) in hub_paths.iter().enumerate() {
        let port_count = count_hub_ports(path);
        super::logger::log_file(&format!("[usb] Hub {} ports = {}", i, port_count));
        if port_count == 0 {
            continue;
        }
        // Path format: \.\usb#root_hub30#5&239a9e6d&0&0#{guid}
        let path_parts: Vec<&str> = path.split('#').collect();
        let hub_name = if path_parts.len() >= 3 {
            format!("{} ({})", path_parts[1], path_parts[2])
        } else {
            path.clone()
        };
        controllers.push(UsbController {
            name: hub_name,
            hub_path: path.clone(),
            port_count,
        });
    }

    Ok(controllers)
}

/// Enumerate all device interfaces of the given class GUID and return their paths
unsafe fn enumerate_hub_paths_for_guid(h_dev_info: HANDLE, guid: &GUID) -> Vec<String> {
    let mut paths = Vec::new();
    let mut index = 0u32;

    loop {
        let mut iface_data = SpDeviceInterfaceData {
            cb_size: std::mem::size_of::<SpDeviceInterfaceData>() as u32,
            interface_class_guid: GUID {
                data1: 0,
                data2: 0,
                data3: 0,
                data4: [0; 8],
            },
            _flags: 0,
            _reserved: 0,
        };

        if SetupDiEnumDeviceInterfaces(
            h_dev_info,
            core::ptr::null_mut(),
            guid,
            index,
            &mut iface_data,
        ) == 0 {
            break;
        }

        let mut required_size = 0u32;
        SetupDiGetDeviceInterfaceDetailW(
            h_dev_info,
            &iface_data,
            core::ptr::null_mut(),
            0,
            &mut required_size,
            core::ptr::null_mut(),
        );

        if required_size == 0 {
            index += 1;
            continue;
        }

        let alloc_size = required_size.max(528);
        let mut detail_buf = SpDeviceInfoDetailData {
            cb_size: 8, // sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA) on x64
            _dev_path: [0u16; 520],
        };

        if SetupDiGetDeviceInterfaceDetailW(
            h_dev_info,
            &iface_data,
            &mut detail_buf,
            alloc_size,
            &mut required_size,
            core::ptr::null_mut(),
        ) == 0 {
            index += 1;
            continue;
        }

        let len = detail_buf
            ._dev_path
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(520);
        let p = String::from_utf16_lossy(&detail_buf._dev_path[..len]);
        if !p.is_empty() && !paths.contains(&p) {
            paths.push(p);
        }
        index += 1;
    }

    paths
}

/// Scan the registry DeviceClasses tree and return the interface class GUIDs
/// that contain USB root hub interfaces (works with non-standard GUIDs too).
fn discover_hub_interface_guids() -> Vec<GUID> {
    const HKEY_LOCAL_MACHINE: HANDLE = 0x80000002usize as HANDLE;
    const KEY_READ: u32 = 0x20019;
    const ERROR_NO_MORE_ITEMS: i32 = 259;
    const DEVICE_CLASSES_ROOT: &str = "SYSTEM\\CurrentControlSet\\Control\\DeviceClasses";

    let mut guids: Vec<GUID> = Vec::new();

    unsafe {
        let mut h_root: HANDLE = std::ptr::null_mut();
        let root_wide = wide_string(DEVICE_CLASSES_ROOT);
        let open_res = RegOpenKeyExW(HKEY_LOCAL_MACHINE, root_wide.as_ptr(), 0, KEY_READ, &mut h_root);
        super::logger::log_file(&format!(
            "[usb] RegOpenKeyExW(DeviceClasses) -> {}",
            open_res
        ));
        if open_res != 0 {
            return guids;
        }

        let mut class_index = 0u32;
        loop {
            let mut guid_buf = [0u16; 64];
            let mut guid_len = 64u32;
            let r = RegEnumKeyExW(
                h_root,
                class_index,
                guid_buf.as_mut_ptr(),
                &mut guid_len,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if r == ERROR_NO_MORE_ITEMS {
                break;
            }
            if r != 0 {
                if class_index == 0 {
                    super::logger::log_file(&format!(
                        "[usb] RegEnumKeyExW class err idx=0 -> {}",
                        r
                    ));
                }
                class_index += 1;
                continue;
            }

            let guid_name = String::from_utf16_lossy(&guid_buf[..guid_len as usize]);

            // Open this interface class key and look for a ROOT_HUB subkey
            let mut h_class: HANDLE = std::ptr::null_mut();
            let class_path = format!("{}\\{}", DEVICE_CLASSES_ROOT, guid_name);
            let class_wide = wide_string(&class_path);
            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, class_wide.as_ptr(), 0, KEY_READ, &mut h_class) != 0 {
                class_index += 1;
                continue;
            }

            let mut is_hub_class = false;
            let mut iface_index = 0u32;
            loop {
                let mut name_buf = [0u16; 1024];
                let mut name_len = 1024u32;
                let r2 = RegEnumKeyExW(
                    h_class,
                    iface_index,
                    name_buf.as_mut_ptr(),
                    &mut name_len,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if r2 == ERROR_NO_MORE_ITEMS {
                    break;
                }
                if r2 != 0 {
                    iface_index += 1;
                    continue;
                }
                let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                if name.contains("USB#ROOT_HUB") || name.contains("usb#root_hub") {
                    is_hub_class = true;
                    break;
                }
                iface_index += 1;
            }
            RegCloseKey(h_class);

            if is_hub_class {
                super::logger::log_file(&format!("[usb] HUB class guid = {}", guid_name));
                match parse_guid(&guid_name) {
                    Some(g) => guids.push(g),
                    None => super::logger::log_file(&format!(
                        "[usb] parse_guid failed for {}",
                        guid_name
                    )),
                }
            }
            class_index += 1;
        }
        RegCloseKey(h_root);
    }

    guids
}

/// Parse a GUID string like {xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}
fn parse_guid(s: &str) -> Option<GUID> {
    let s = s.trim().trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    let data1 = u32::from_str_radix(parts[0], 16).ok()?;
    let data2 = u16::from_str_radix(parts[1], 16).ok()?;
    let data3 = u16::from_str_radix(parts[2], 16).ok()?;
    let hex = format!("{}{}", parts[3], parts[4]);
    if hex.len() != 16 {
        return None;
    }
    let mut data4 = [0u8; 8];
    for i in 0..8 {
        data4[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(GUID {
        data1,
        data2,
        data3,
        data4,
    })
}



fn count_hub_ports(hub_path: &str) -> u32 {
    unsafe {
        let h = match open_hub(hub_path) {
            Ok(h) => h,
            Err(e) => {
                super::logger::log_file(&format!(
                    "[usb] open_hub FAILED: {}",
                    e
                ));
                return 0;
            }
        };

        let mut buf = [0u8; NODE_INFO_BUF_SIZE];
        let mut returned = 0u32;

        let ok = DeviceIoControl(
            h,
            IOCTL_USB_GET_NODE_INFORMATION,
            NULL,
            0,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            buf.len() as u32,
            &mut returned,
            NULL,
        );

        let ioctl_err = GetLastError();
        close_handle(h);

        if ok != 0 {
            // Empirically: 4-byte header, then USB_HUB_DESCRIPTOR with
            // bNumberOfPorts at byte 6 (verified via hexdump on real hardware)
            let ports = buf[6] as u32;
            super::logger::log_file(&format!(
                "[usb] NODE_INFO ok: returned={} bNumPorts={}",
                returned, ports
            ));
            ports
        } else {
            super::logger::log_file(&format!(
                "[usb] NODE_INFO failed: Win32={}",
                ioctl_err
            ));
            0
        }
    }
}

/// Query each port on a hub for connected devices
fn query_hub_ports(
    hub_path: &str,
    port_count: u32,
    controller_name: &str,
) -> Result<(Vec<UsbPort>, Vec<UsbDevice>)> {
    unsafe {
        let h = open_hub(hub_path)?;

        let mut ports = Vec::new();

        for port_num in 1..=port_count {
            let mut buf = [0u8; 16];
            buf[0..4].copy_from_slice(&port_num.to_ne_bytes());

            let mut returned = 0u32;
            let ok = DeviceIoControl(
                h,
                IOCTL_USB_GET_PORT_STATUS,
                buf.as_ptr() as *const core::ffi::c_void,
                buf.len() as u32,
                buf.as_mut_ptr() as *mut core::ffi::c_void,
                buf.len() as u32,
                &mut returned,
                NULL,
            );

            if ok == 0 {
                continue;
            }

            // Port status: ConnectionIndex(4) + StatusFlags(4)
            let status = u32::from_ne_bytes([buf[4], buf[5], buf[6], buf[7]]);

            let connected = (status & 0x0001) != 0;

            let (speed_val, speed_str): (u32, String) = if status & 0x1000 != 0 || status & 0x2000 != 0 {
                (4, "SuperSpeed".into())
            } else if status & 0x0400 != 0 {
                (3, "High Speed".into())
            } else if status & 0x0200 != 0 {
                (1, "Low Speed".into())
            } else if connected {
                (2, "Full Speed".into())
            } else {
                (0, "Unknown".into())
            };

            let status_str: String = if connected {
                "Connected".into()
            } else {
                "No Device".into()
            };

            let port_id = format!("{}:{}", hub_path, port_num);

            ports.push(UsbPort {
                id: port_id.clone(),
                controller_name: controller_name.to_string(),
                hub_path: hub_path.to_string(),
                port_number: port_num,
                connected,
                speed: speed_str.clone(),
                speed_value: speed_val,
                device_address: 0,
                open_pipes: 0,
                status: status_str,
            });
        }

        close_handle(h);
        Ok((ports, Vec::new()))
    }
}

// ─── USB Device Enumeration (SetupAPI) ────────────────────────────

const SPDRP_DEVICEDESC: u32 = 0x00000000;
const SPDRP_COMPATIBLEIDS: u32 = 0x00000002;
const SPDRP_SERVICE: u32 = 0x00000004;
const SPDRP_MFG: u32 = 0x0000000B;
const SPDRP_FRIENDLYNAME: u32 = 0x0000000C;
const SPDRP_LOCATION_INFORMATION: u32 = 0x0000000D;

/// Read a string registry property for a device
unsafe fn get_dev_string(h: HANDLE, dev: &SpDevInfoData, prop: u32) -> String {
    let mut buf = [0u16; 256];
    let ok = SetupDiGetDeviceRegistryPropertyW(
        h,
        dev,
        prop,
        core::ptr::null_mut(),
        buf.as_mut_ptr() as *mut u8,
        (buf.len() * 2) as u32,
        core::ptr::null_mut(),
    );
    if ok == 0 {
        return String::new();
    }
    let len = buf.iter().position(|&ch| ch == 0).unwrap_or(256);
    String::from_utf16_lossy(&buf[..len])
}

/// Enumerate all USB devices (instance IDs like USB\VID_xxxx&PID_xxxx) via SetupAPI
/// True when the device's parent is itself a USB device (a hub or a composite
/// parent). Hubs and host controllers hang off PCI/ACPI instead, so this
/// separates real peripherals from hubs.
fn parent_is_usb_device(dev_inst: u32) -> bool {
    let mut parent: u32 = 0;
    if unsafe { CM_Get_Parent(&mut parent, dev_inst, 0) } != 0 {
        return true; // can't determine — keep the device
    }
    let mut buf = [0u16; 512];
    if unsafe { CM_Get_Device_IDW(parent, buf.as_mut_ptr(), 512, 0) } != 0 {
        return true;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(512);
    let id = String::from_utf16_lossy(&buf[..len]);
    id.starts_with("USB\\")
}

/// Instance ID of the device's immediate parent (the hub it is attached to).
fn parent_hub_instance(dev_inst: u32) -> String {
    let mut parent: u32 = 0;
    if unsafe { CM_Get_Parent(&mut parent, dev_inst, 0) } != 0 {
        return String::new();
    }
    let mut buf = [0u16; 512];
    if unsafe { CM_Get_Device_IDW(parent, buf.as_mut_ptr(), 512, 0) } != 0 {
        return String::new();
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(512);
    String::from_utf16_lossy(&buf[..len])
}

/// Physical port number on the immediate hub, from the location string
/// (e.g. "Port_#0004.Hub_#0010" -> 4). Uses the LAST Port_# segment, which
/// corresponds to the direct parent hub for nested topologies.
fn device_location_port(h: HANDLE, dev: &SpDevInfoData) -> u32 {
    let loc = unsafe { get_dev_string(h, dev, SPDRP_LOCATION_INFORMATION) };
    let mut port = 0u32;
    let mut from = 0usize;
    let marker = "Port_#";
    while let Some(pos) = loc[from..].find(marker) {
        let start = from + pos + marker.len();
        let end = loc[start..]
            .find(|c: char| !c.is_ascii_digit())
            .map(|i| start + i)
            .unwrap_or(loc.len());
        if let Ok(p) = loc[start..end].parse::<u32>() {
            port = p;
        }
        from = end;
    }
    port
}

/// Canonical key of a hub from its device-interface path.
/// Path: "\\?\usb#vid_2109&pid_0822#6&1b2b7d3c&0&0#{guid}"
/// Key:  "vid_2109&pid_0822#6&1b2b7d3c&0&0"
fn hub_key_from_service_path(path: &str) -> Option<String> {
    let tokens: Vec<&str> = path.split('#').collect();
    // tokens[0] = "\\?\usb"; drop it, then stop at the "{guid}" token.
    let inner: Vec<&str> = tokens[1..]
        .iter()
        .copied()
        .take_while(|t| !t.starts_with('{'))
        .collect();
    if inner.is_empty() {
        None
    } else {
        Some(inner.join("#").to_lowercase())
    }
}

/// Canonical key of a hub from a device instance ID.
/// Instance: "USB\VID_2109&PID_0822\6&1B2B7D3C&0&0"
/// Key:      "vid_2109&pid_0822#6&1b2b7d3c&0&0"
fn hub_key_from_dev_instance(instance: &str) -> String {
    let s = if let Some(stripped) = instance.strip_prefix("USB\\") {
        stripped
    } else {
        instance
    };
    s.replace('\\', "#").to_lowercase()
}

/// Read the real USB device class (bDeviceClass) from the device's
/// Compatible IDs (e.g. "USBClass_09" for hubs, "DevClass_00" for composites).
fn parse_device_class(h: HANDLE, dev: &SpDevInfoData) -> u8 {
    let comp = unsafe { get_dev_string(h, dev, SPDRP_COMPATIBLEIDS) };
    for token in comp.split(['\\', '&']) {
        let t = token.trim();
        let rest = t
            .strip_prefix("DevClass_")
            .or_else(|| t.strip_prefix("Class_"));
        if let Some(hex) = rest {
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                return v;
            }
        }
    }
    0
}

fn enumerate_usb_devices() -> Vec<(UsbDevice, String, u32)> {
    let mut devices: Vec<(UsbDevice, String, u32)> = Vec::new();
    unsafe {
        let h = SetupDiGetClassDevsW(
            core::ptr::null(),
            core::ptr::null(),
            NULL,
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        );
        if h == INVALID_HANDLE_VALUE || h.is_null() {
            return devices;
        }

        let mut idx = 0u32;
        loop {
            let mut dev = SpDevInfoData {
                cb_size: std::mem::size_of::<SpDevInfoData>() as u32,
                class_guid: GUID {
                    data1: 0,
                    data2: 0,
                    data3: 0,
                    data4: [0; 8],
                },
                _dev_inst: 0,
                _reserved: 0,
            };

            if SetupDiEnumDeviceInfo(h, idx, &mut dev) == 0 {
                break;
            }

            let mut id_buf = [0u16; 512];
            let mut req = 0u32;
            if SetupDiGetDeviceInstanceIdW(h, &dev, id_buf.as_mut_ptr(), 512, &mut req) == 0 {
                idx += 1;
                continue;
            }
            let id_len = id_buf.iter().position(|&ch| ch == 0).unwrap_or(512);
            let instance_id = String::from_utf16_lossy(&id_buf[..id_len]);

            // USB devices: USB\VID_xxxx&PID_xxxx[&MI_nn]\serial
            if let Some(rest) = instance_id.strip_prefix("USB\\VID_") {
                let parts: Vec<&str> = rest.split('&').collect();
                let vid = parts
                    .first()
                    .and_then(|p| u16::from_str_radix(p, 16).ok());
                // PID_xxxx may run straight into "\serial" (simple devices) or
                // "&MI_nn" (composite functions) — parse only the hex digits.
                let pid = parts
                    .iter()
                    .find(|p| p.starts_with("PID_"))
                    .and_then(|p| {
                        let hex = p[4..].split(['&', '\\']).next().unwrap_or("");
                        u16::from_str_radix(hex, 16).ok()
                    });

                if let (Some(vid), Some(pid)) = (vid, pid) {
                    // Hubs hang off PCI/ACPI host controllers; peripherals hang off
                    // hubs. Keeping only devices attached to the USB bus skips the
                    // hubs that are already listed in the Ports tab.
                    if !parent_is_usb_device(dev._dev_inst) {
                        idx += 1;
                        continue;
                    }

                    // Hubs run under the USBHUB3/usbhub driver — skip them, they
                    // are already listed in the Ports tab.
                    let service = get_dev_string(h, &dev, SPDRP_SERVICE).to_lowercase();
                    if service == "usbhub3" || service == "usbhub" {
                        idx += 1;
                        continue;
                    }

                    let hub_inst = parent_hub_instance(dev._dev_inst);
                    let hub_port = device_location_port(h, &dev);

                    let mfg = get_dev_string(h, &dev, SPDRP_MFG);
                    let desc = get_dev_string(h, &dev, SPDRP_DEVICEDESC);
                    let friendly = get_dev_string(h, &dev, SPDRP_FRIENDLYNAME);
                    let serial = instance_id
                        .rsplit('\\')
                        .next()
                        .unwrap_or("")
                        .to_string();

                    let friendly_name = if !friendly.is_empty() {
                        friendly
                    } else if !desc.is_empty()
                        && desc != "USB Composite Device"
                        && desc != "Périphérique USB composite"
                    {
                        desc.clone()
                    } else {
                        super::devices::lookup(vid, pid).name
                    };

                    let info = UsbDevice {
                        port_id: String::new(),
                        power_role: String::new(),
                        vid,
                        pid,
                        bcd_usb: 0,
                        bcd_device: 0,
                        device_class: parse_device_class(h, &dev),
                        device_subclass: 0,
                        device_protocol: 0,
                        max_packet_size: 0,
                        manufacturer: mfg,
                        product: desc,
                        serial,
                        hub_name: String::new(),
                        port_number: 0,
                        driver_key: String::new(),
                        friendly_name,
                        device_class_name: String::new(),
                        usb_version: String::new(),
                        speed: String::new(),
                    };

                    // Composite functions (MI_nn / IG_ / LAMPARRAY...) share the
                    // parent's VID:PID. Keep the composite parent node — its name
                    // is descriptive — and drop the duplicate interface entries.
                    let is_interface = parts.len() > 2;
                    match devices.iter_mut().find(|(d, _, _)| d.vid == vid && d.pid == pid) {
                        Some((existing, h, p)) if !is_interface => {
                            *existing = info;
                            *h = hub_inst.clone();
                            *p = hub_port;
                        }
                        None => devices.push((info, hub_inst, hub_port)),
                        _ => {}
                    }
                }
            }

            idx += 1;
        }

        SetupDiDestroyDeviceInfoList(h);
    }
    devices
}

// ─── Public API ───────────────────────────────────────────────────

pub fn full_scan() -> Result<UsbSnapshot> {
    let controllers = enumerate_controllers().context("Failed to enumerate USB hubs")?;

    let mut all_ports = Vec::new();

    for ctrl in &controllers {
        let (ports, _devs) =
            query_hub_ports(&ctrl.hub_path, ctrl.port_count, &ctrl.name).unwrap_or_else(|e| {
                log::warn!("Failed to query hub {}: {}", ctrl.hub_path, e);
                (Vec::new(), Vec::new())
            });

        all_ports.extend(ports);
    }

    // Connected devices come from SetupAPI (reliable VID/PID/name source)
    // Link each device to the hub and physical port it is plugged into.
    // The hub's device-interface path and its device node instance ID share a
    // canonical "vid_pid#serial" key, so we match on that and fill the port.
    let hub_index: Vec<(String, String, String)> = controllers
        .iter()
        .filter_map(|c| {
            hub_key_from_service_path(&c.hub_path)
                .map(|k| (k, c.name.clone(), c.hub_path.clone()))
        })
        .collect();

    let mut all_devices = Vec::new();
    for (mut dev, hub_instance, port) in enumerate_usb_devices() {
        let key = hub_key_from_dev_instance(&hub_instance);
        if let Some((_, cname, chpath)) = hub_index.iter().find(|(k, _, _)| *k == key) {
            dev.hub_name = cname.clone();
            dev.port_number = port;
            // Matches UsbPort.id ("{hub_path}:{port}") so the UI can pair them.
            dev.port_id = format!("{}:{}", chpath, port);
        }
        dev.power_role = super::devices::power_role(dev.vid, dev.pid, dev.device_class).to_string();
        all_devices.push(dev);
    }

    let now = chrono_free();

    Ok(UsbSnapshot {
        controllers,
        ports: all_ports,
        devices: all_devices,
        scan_time: now,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────

fn chrono_free() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02} UTC", h, m, s)
}

#[cfg(test)]
fn format_usb_version(bcd: u16) -> String {
    let major = (bcd >> 8) & 0xFF;
    let minor = (bcd >> 4) & 0x0F;
    let patch = bcd & 0x0F;
    if patch > 0 {
        format!("{}.{}.{}", major, minor, patch)
    } else {
        format!("{}.{}", major, minor)
    }
}



// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn scan_devices_contain_no_hubs() {
        // Live hardware test: if the machine has USB devices, none of them may
        // be hubs (they run under the USBHUB3/usbhub driver and are excluded).
        let Ok(snap) = full_scan() else { return };
        if snap.devices.is_empty() {
            return; // no USB devices on this machine — nothing to assert
        }
        for d in &snap.devices {
            let lower = d.friendly_name.to_lowercase();
            assert!(
                !lower.contains("concentrateur") && !lower.contains("generic usb hub"),
                "hub leaked into devices: {}",
                d.friendly_name
            );
        }
    }

    #[test]
    fn parse_guid_valid() {
        let g = parse_guid("{f18a0e88-c30c-11d0-81e9-00a0c91eeb34}").expect("should parse");
        assert_eq!(g.data1, 0xf18a0e88);
        assert_eq!(g.data2, 0xc30c);
        assert_eq!(g.data3, 0x11d0);
        assert_eq!(g.data4, [0x81, 0xe9, 0x00, 0xa0, 0xc9, 0x1e, 0xeb, 0x34]);
    }

    #[test]
    fn parse_guid_accepts_no_braces() {
        let g = parse_guid("00000000-0000-0000-0000-000000000000").expect("should parse");
        assert_eq!(g.data1, 0);
        assert_eq!(g.data4, [0u8; 8]);
    }

    #[test]
    fn parse_guid_rejects_invalid() {
        assert!(parse_guid("").is_none());
        assert!(parse_guid("not-a-guid").is_none());
        assert!(parse_guid("{1234}").is_none());
        assert!(parse_guid("{f18a0e88-c30c-11d0-81e9-00a0c91eeb3}").is_none()); // too short
    }

    #[test]
    fn format_usb_version_cases() {
        assert_eq!(format_usb_version(0x0200), "2.0");
        assert_eq!(format_usb_version(0x0310), "3.1");
        assert_eq!(format_usb_version(0x0201), "2.0.1");
        assert_eq!(format_usb_version(0x0110), "1.1");
    }

    #[test]
    fn filter_scan_for_hub_keeps_only_target_controller() {
        // Simulated snapshot: two controllers with ports, and devices linked
        // to ports on both controllers.
        let make_port = |ctrl: &str, hub_path: &str, port: u32, connected: bool| UsbPort {
            id: format!("{}:{}", hub_path, port),
            controller_name: ctrl.to_string(),
            hub_path: hub_path.to_string(),
            port_number: port,
            connected,
            speed: String::new(),
            speed_value: 0,
            device_address: 0,
            open_pipes: 0,
            status: String::new(),
        };
        let make_device = |port_id: &str, friendly: &str, hub: &str| UsbDevice {
            port_id: port_id.to_string(),
            hub_name: hub.to_string(),
            friendly_name: friendly.to_string(),
            ..Default::default()
        };

        let snap = UsbSnapshot {
            controllers: vec![
                UsbController { name: "hub_a".into(), hub_path: "path_a".into(), port_count: 3 },
                UsbController { name: "hub_b".into(), hub_path: "path_b".into(), port_count: 2 },
            ],
            ports: vec![
                make_port("hub_a", "path_a", 1, true),
                make_port("hub_a", "path_a", 2, false),
                make_port("hub_b", "path_b", 1, true),
            ],
            devices: vec![
                make_device("path_a:1", "POCO X7", "hub_a"),
                make_device("path_a:2", "Caméra (port vide mais device lié)", "hub_a"),
                make_device("path_b:1", "Souris", "hub_b"),
            ],
            scan_time: String::new(),
        };

        // Filter for hub_a: keeps both of its ports, plus both devices whose
        // port_id belongs to hub_a — even the one on a disconnected port.
        let result = filter_scan_for_hub(snap.clone(), "hub_a");
        assert_eq!(result.ports.len(), 2);
        assert_eq!(result.devices.len(), 2);
        assert!(result.ports.iter().all(|p| p.controller_name == "hub_a"));
        assert!(result.devices.iter().all(|d| d.hub_name == "hub_a"));

        // Filter for hub_b: only its port and its device.
        let result_b = filter_scan_for_hub(snap.clone(), "hub_b");
        assert_eq!(result_b.ports.len(), 1);
        assert_eq!(result_b.devices.len(), 1);
        assert_eq!(result_b.devices[0].friendly_name, "Souris");

        // Unknown controller: empty result, no panic.
        let result_empty = filter_scan_for_hub(snap.clone(), "hub_unknown");
        assert!(result_empty.ports.is_empty());
        assert!(result_empty.devices.is_empty());
    }
}
