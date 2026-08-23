import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/** Full USB scan — enumerates all hubs, ports, and connected devices */
export async function scanUsb() {
  return invoke('scan_usb');
}

/** Targeted re-scan of a single hub/controller (ports + linked devices) */
export async function scanHub(name) {
  return invoke('scan_hub', { name });
}

/** Query UCSI connector info for a specific port (if driver supports it) */
export async function getConnectorInfo(portIndex) {
  return invoke('get_connector_info', { portIndex });
}

/** Look up VID:PID in the built-in device database */
export async function lookupDevice(vid, pid) {
  return invoke('lookup_device', { vid, pid });
}

/** Get saved app settings (JSON string) */
export async function getSettings() {
  return invoke('get_settings');
}

/** Save app settings (JSON string) */
export async function saveSettings(json) {
  return invoke('save_settings', { json });
}

/** Get today's log file contents */
export async function getLogs() {
  return invoke('get_logs');
}

/** Listen for real-time device change events from the backend monitor */
export function onDeviceChange(callback) {
  return listen('device-change', (event) => {
    callback(event.payload);
  });
}
