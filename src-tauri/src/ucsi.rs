// ─── UCSI (USB Type-C Connector System Software Interface) ─────────
// Placeholder for UCSI support. Full implementation would query
// \\.\UCM#N devices via DeviceIoControl. For now returns unavailable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectorInfo {
    pub available: bool,
    pub connector_id: u32,
    pub power_role: String,
    pub data_role: String,
    pub partner_type: String,
    pub cable_speed: String,
    pub cable_type: String,
    pub usb_capable: bool,
    pub pd_capable: bool,
    pub external_supply: bool,
    pub pd_current_ma: u32,
    pub pd_voltage_mv: u32,
    pub pd_max_watts: u32,
    pub raw_status: u32,
}

pub fn get_connector_status(_connector_id: u32) -> Result<ConnectorInfo, String> {
    // UCSI requires Windows 10 1809+ with the UCSI class driver.
    // Full implementation would:
    // 1. Open \\.\UCM#N or \\.\USB#Connector#N via CreateFileW
    // 2. Issue IOCTL_UCM_QUERY_DATA with GET_CONNECTOR_STATUS command
    // 3. Parse the response for power role, cable type, PD contract, etc.
    //
    // For now, return unavailable to keep the build simple.
    Ok(ConnectorInfo::default())
}
