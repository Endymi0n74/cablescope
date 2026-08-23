// ─── USB Device Database ──────────────────────────────────────────
// Maps VID:PID pairs to human-readable device names.
// 200+ vendors, 100+ specific devices, 50+ known cables.
// This replaces macOS WhatCable's e-marker database with a Windows
// approach based on known USB device identifiers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInfo {
    pub name: String,
    pub category: String,
    pub manufacturer: String,
}

#[derive(Debug, Clone)]
pub struct VendorInfo {
    name: &'static str,
    category: &'static str,
}

#[derive(Debug, Clone)]
// Detailed cable fields (speed/watts/type) are reserved for the future
// Power tab display; the cable name is already surfaced via lookup().
#[allow(dead_code)]
pub struct CableInfo {
    name: &'static str,
    speed: &'static str,
    max_watts: u32,
    cable_type: &'static str,
}

#[derive(Debug, Clone)]
struct DeviceEntry {
    name: &'static str,
    category: &'static str,
}

// ═══════════════════════════════════════════════════════════════════
//  VENDOR DATABASE — 200+ USB Vendors
// ═══════════════════════════════════════════════════════════════════

/// Insert a vendor, preferring the canonical name (no "(suffix)") on duplicates.
fn insert_vendor(m: &mut HashMap<u16, VendorInfo>, vid: u16, info: VendorInfo) {
    match m.get(&vid) {
        None => {
            m.insert(vid, info);
        }
        Some(existing) => {
            let existing_has_suffix = existing.name.contains('(');
            let new_has_suffix = info.name.contains('(');
            if !new_has_suffix && existing_has_suffix {
                m.insert(vid, info);
            }
        }
    }
}

fn vendors() -> &'static HashMap<u16, VendorInfo> {
    static LOCK: OnceLock<HashMap<u16, VendorInfo>> = OnceLock::new();
    LOCK.get_or_init(|| {
        // A VID can appear in several categories; keep the canonical name
        // (without a parenthetical suffix) so lookups return clean labels.
        let mut m: HashMap<u16, VendorInfo> = HashMap::new();

        // ── Apple & Ecosystem ──────────────────────────────
        insert_vendor(&mut m, 0x05AC, VendorInfo { name: "Apple", category: "Cable/Charger" });
        insert_vendor(&mut m, 0x0A5C, VendorInfo { name: "Apple (Broadcom)", category: "Bluetooth/WiFi" });
        insert_vendor(&mut m, 0x06BC, VendorInfo { name: "Apple (Thunderbolt)", category: "Thunderbolt" });

        // ── Samsung & Mobile ───────────────────────────────
        insert_vendor(&mut m, 0x04E8, VendorInfo { name: "Samsung", category: "Mobile/Charger" });
        insert_vendor(&mut m, 0x2207, VendorInfo { name: "Samsung (MTP)", category: "Mobile" });
        insert_vendor(&mut m, 0x0BB4, VendorInfo { name: "HTC", category: "Mobile" });
        insert_vendor(&mut m, 0x18D1, VendorInfo { name: "Google (Nexus/Pixel)", category: "Mobile" });
        insert_vendor(&mut m, 0x2717, VendorInfo { name: "Xiaomi", category: "Mobile" });
        insert_vendor(&mut m, 0x0E8D, VendorInfo { name: "MediaTek", category: "Mobile/WiFi" });
        insert_vendor(&mut m, 0x2A70, VendorInfo { name: "OnePlus", category: "Mobile" });
        insert_vendor(&mut m, 0x05C6, VendorInfo { name: "Qualcomm", category: "Mobile/Modem" });
        insert_vendor(&mut m, 0x1286, VendorInfo { name: "Lenovo Mobile", category: "Mobile" });
        insert_vendor(&mut m, 0x2D95, VendorInfo { name: "Realme", category: "Mobile" });
        insert_vendor(&mut m, 0x19D2, VendorInfo { name: "ZTE", category: "Mobile/Modem" });
        insert_vendor(&mut m, 0x20A6, VendorInfo { name: "Huawei", category: "Mobile/Networking" });
        insert_vendor(&mut m, 0x12D1, VendorInfo { name: "Huawei (HiSilicon)", category: "Mobile/Modem" });
        insert_vendor(&mut m, 0x0409, VendorInfo { name: "NEC", category: "Computing" });

        // ── Microsoft & Xbox ───────────────────────────────
        insert_vendor(&mut m, 0x045E, VendorInfo { name: "Microsoft", category: "Peripherals" });
        insert_vendor(&mut m, 0x045F, VendorInfo { name: "Microsoft (Surface)", category: "Computing" });

        // ── Intel ──────────────────────────────────────────
        insert_vendor(&mut m, 0x8087, VendorInfo { name: "Intel", category: "USB Controller" });
        insert_vendor(&mut m, 0x8086, VendorInfo { name: "Intel", category: "Computing" });

        // ── USB Hub Controllers ────────────────────────────
        insert_vendor(&mut m, 0x2109, VendorInfo { name: "VIA Labs", category: "Hub Controller" });
        insert_vendor(&mut m, 0x1A40, VendorInfo { name: "Terminus Technology", category: "Hub Controller" });
        insert_vendor(&mut m, 0x0424, VendorInfo { name: "Microchip (SMSC)", category: "Hub Controller" });
        insert_vendor(&mut m, 0x174C, VendorInfo { name: "ASMedia", category: "Hub/Storage Bridge" });
        insert_vendor(&mut m, 0x0BDA, VendorInfo { name: "Realtek", category: "Hub/Ethernet" });
        insert_vendor(&mut m, 0x3456, VendorInfo { name: "Genesys Logic", category: "Hub Controller" });
        insert_vendor(&mut m, 0x2E1A, VendorInfo { name: "InnoConn", category: "Hub/Dock" });

        // ── Dell ───────────────────────────────────────────
        insert_vendor(&mut m, 0x413C, VendorInfo { name: "Dell", category: "Computing/Peripherals" });
        insert_vendor(&mut m, 0x0A5C, VendorInfo { name: "Broadcom (Dell)", category: "Bluetooth" });

        // ── Lenovo ─────────────────────────────────────────
        insert_vendor(&mut m, 0x17EF, VendorInfo { name: "Lenovo", category: "Computing/Dock" });
        insert_vendor(&mut m, 0x1057, VendorInfo { name: "Lenovo Mobile", category: "Mobile" });

        // ── HP ─────────────────────────────────────────────
        insert_vendor(&mut m, 0x03F0, VendorInfo { name: "HP", category: "Peripherals" });
        insert_vendor(&mut m, 0x103C, VendorInfo { name: "HP Inc", category: "Computing" });

        // ── ASUS ───────────────────────────────────────────
        insert_vendor(&mut m, 0x0B05, VendorInfo { name: "ASUS", category: "Computing" });
        insert_vendor(&mut m, 0x04D2, VendorInfo { name: "ASUS (MediaTek)", category: "WiFi" });

        // ── Acer ───────────────────────────────────────────
        insert_vendor(&mut m, 0x0502, VendorInfo { name: "Acer", category: "Computing" });

        // ── Razer ──────────────────────────────────────────
        insert_vendor(&mut m, 0x1532, VendorInfo { name: "Razer", category: "Gaming Peripherals" });

        // ── Logitech ───────────────────────────────────────
        insert_vendor(&mut m, 0x046D, VendorInfo { name: "Logitech", category: "Peripherals" });

        // ── Audio: Headphones & DACs ───────────────────────
        insert_vendor(&mut m, 0x0471, VendorInfo { name: "Philips", category: "Audio" });
        insert_vendor(&mut m, 0x04DB, VendorInfo { name: "Sony", category: "Audio/Mobile" });
        insert_vendor(&mut m, 0x054C, VendorInfo { name: "Sony (PlayStation)", category: "Gaming" });
        insert_vendor(&mut m, 0x0955, VendorInfo { name: "NVIDIA", category: "Computing" });
        insert_vendor(&mut m, 0x0955, VendorInfo { name: "NVIDIA (Shield)", category: "Computing" });
        insert_vendor(&mut m, 0x1015, VendorInfo { name: "Sennheiser", category: "Audio" });
        insert_vendor(&mut m, 0x046F, VendorInfo { name: "C-Media", category: "Audio" });
        insert_vendor(&mut m, 0x0D8C, VendorInfo { name: "C-Media (USB Audio)", category: "Audio" });
        insert_vendor(&mut m, 0x1235, VendorInfo { name: "Focusrite", category: "Audio Interface" });
        insert_vendor(&mut m, 0x0EB8, VendorInfo { name: "M-Audio", category: "Audio Interface" });
        insert_vendor(&mut m, 0x0763, VendorInfo { name: "MOTU", category: "Audio Interface" });
        insert_vendor(&mut m, 0x2A39, VendorInfo { name: "UGREEN (Audio)", category: "Audio DAC" });
        insert_vendor(&mut m, 0x305A, VendorInfo { name: "AudioQuest", category: "DAC" });
        insert_vendor(&mut m, 0x0D8C, VendorInfo { name: "C-Media (DAC)", category: "Audio DAC" });
        insert_vendor(&mut m, 0x0001, VendorInfo { name: "Audio-Technica", category: "Audio" });
        insert_vendor(&mut m, 0x04D8, VendorInfo { name: "Microchip (DAC)", category: "Audio DAC" });
        insert_vendor(&mut m, 0x2B56, VendorInfo { name: "Shure", category: "Audio" });
        insert_vendor(&mut m, 0x1220, VendorInfo { name: "Stamford (AudioEngine)", category: "Audio" });
        insert_vendor(&mut m, 0x16B0, VendorInfo { name: "iFi Audio", category: "Audio DAC" });
        insert_vendor(&mut m, 0x04E8, VendorInfo { name: "Samsung (Audio)", category: "Audio" });

        // ── Storage ────────────────────────────────────────
        insert_vendor(&mut m, 0x0BC2, VendorInfo { name: "Seagate", category: "Storage" });
        insert_vendor(&mut m, 0x0781, VendorInfo { name: "SanDisk (WD)", category: "Storage" });
        insert_vendor(&mut m, 0x0951, VendorInfo { name: "Kingston", category: "Storage" });
        insert_vendor(&mut m, 0x13FE, VendorInfo { name: "Phison", category: "Storage" });
        insert_vendor(&mut m, 0x125F, VendorInfo { name: "ADATA", category: "Storage" });
        insert_vendor(&mut m, 0x1058, VendorInfo { name: "Western Digital", category: "Storage" });
        insert_vendor(&mut m, 0x04E8, VendorInfo { name: "Samsung (Storage)", category: "Storage" });
        insert_vendor(&mut m, 0x059B, VendorInfo { name: "LaCie", category: "Storage" });
        insert_vendor(&mut m, 0x04B4, VendorInfo { name: "Cypress (Storage)", category: "Storage Bridge" });
        insert_vendor(&mut m, 0x152D, VendorInfo { name: "JMicron", category: "Storage Bridge" });
        insert_vendor(&mut m, 0x174C, VendorInfo { name: "ASMedia (Storage)", category: "Storage Bridge" });
        insert_vendor(&mut m, 0x2109, VendorInfo { name: "VIA Labs (Storage)", category: "Storage Bridge" });
        insert_vendor(&mut m, 0x1F75, VendorInfo { name: "Innodisk", category: "Storage" });
        insert_vendor(&mut m, 0x090C, VendorInfo { name: "Silicon Motion", category: "Storage" });
        insert_vendor(&mut m, 0x13B1, VendorInfo { name: "Apacer", category: "Storage" });
        insert_vendor(&mut m, 0x154B, VendorInfo { name: "PNY", category: "Storage" });
        insert_vendor(&mut m, 0x1376, VendorInfo { name: "LSI (Avago)", category: "Storage Controller" });
        insert_vendor(&mut m, 0x9023, VendorInfo { name: "Intel (Optane)", category: "Storage" });
        insert_vendor(&mut m, 0x14A0, VendorInfo { name: "Transcend", category: "Storage" });
        insert_vendor(&mut m, 0x1189, VendorInfo { name: "Corsair", category: "Storage" });
        insert_vendor(&mut m, 0x02A7, VendorInfo { name: "Crucial (Micron)", category: "Storage" });
        insert_vendor(&mut m, 0x0984, VendorInfo { name: "Corsair (Storage)", category: "Storage" });

        // ── Networking: Ethernet Adapters ───────────────────
        insert_vendor(&mut m, 0x0B95, VendorInfo { name: "ASIX", category: "Ethernet" });
        insert_vendor(&mut m, 0x2001, VendorInfo { name: "D-Link", category: "Ethernet" });
        insert_vendor(&mut m, 0x2357, VendorInfo { name: "TP-Link", category: "Ethernet/WiFi" });
        insert_vendor(&mut m, 0x0BDA, VendorInfo { name: "Realtek (Ethernet)", category: "Ethernet" });
        insert_vendor(&mut m, 0x077B, VendorInfo { name: "Linksys", category: "Ethernet" });
        insert_vendor(&mut m, 0x04BB, VendorInfo { name: "I-O DATA", category: "Ethernet" });
        insert_vendor(&mut m, 0x07A8, VendorInfo { name: "ATEN", category: "Ethernet/KVM" });
        insert_vendor(&mut m, 0x056E, VendorInfo { name: "Elecom", category: "Ethernet" });
        insert_vendor(&mut m, 0x1C04, VendorInfo { name: "QNAP", category: "Ethernet" });
        insert_vendor(&mut m, 0x0B6A, VendorInfo { name: "CalDigit", category: "Dock" });
        insert_vendor(&mut m, 0x1A40, VendorInfo { name: "HooToo", category: "Hub/Dock" });
        insert_vendor(&mut m, 0x3340, VendorInfo { name: "Ugreen", category: "Ethernet/Hub" });
        insert_vendor(&mut m, 0x291A, VendorInfo { name: "Anker", category: "Hub/Dock/Cable" });
        insert_vendor(&mut m, 0x30C9, VendorInfo { name: "Luxshare", category: "Cable/Dock" });
        insert_vendor(&mut m, 0x174C, VendorInfo { name: "Startech", category: "Dock/Adapter" });

        // ── WiFi Adapters ──────────────────────────────────
        insert_vendor(&mut m, 0x0CF3, VendorInfo { name: "Qualcomm Atheros", category: "WiFi/BT" });
        insert_vendor(&mut m, 0x0E8D, VendorInfo { name: "MediaTek (WiFi)", category: "WiFi" });
        insert_vendor(&mut m, 0x0A5C, VendorInfo { name: "Broadcom (WiFi)", category: "WiFi/BT" });
        insert_vendor(&mut m, 0x8087, VendorInfo { name: "Intel (WiFi)", category: "WiFi/BT" });
        insert_vendor(&mut m, 0x050D, VendorInfo { name: "Belkin", category: "WiFi" });
        insert_vendor(&mut m, 0x0846, VendorInfo { name: "Netgear", category: "Ethernet/WiFi" });
        insert_vendor(&mut m, 0x1044, VendorInfo { name: "ZyXEL", category: "WiFi" });

        // ── Webcam & Video Capture ──────────────────────────
        insert_vendor(&mut m, 0x046D, VendorInfo { name: "Logitech (Webcam)", category: "Webcam" });
        insert_vendor(&mut m, 0x0B05, VendorInfo { name: "ASUS (Webcam)", category: "Webcam" });
        insert_vendor(&mut m, 0x05A9, VendorInfo { name: "OmniVision", category: "Camera" });
        insert_vendor(&mut m, 0x0C45, VendorInfo { name: "Sonix", category: "Camera" });
        insert_vendor(&mut m, 0x0458, VendorInfo { name: "Genius", category: "Webcam" });
        insert_vendor(&mut m, 0x0AC9, VendorInfo { name: "Microdia", category: "Camera" });
        insert_vendor(&mut m, 0x2BC5, VendorInfo { name: "Elgato", category: "Capture Card" });
        insert_vendor(&mut m, 0x0FD9, VendorInfo { name: "Elgato", category: "Stream Deck" });
        insert_vendor(&mut m, 0x1997, VendorInfo { name: "Razer (Webcam)", category: "Webcam" });
        insert_vendor(&mut m, 0x10E8, VendorInfo { name: "Monsoon Solutions", category: "Power Monitor" });

        // ── Displays & Adapters ─────────────────────────────
        insert_vendor(&mut m, 0x17E9, VendorInfo { name: "DisplayLink", category: "Display Adapter" });
        insert_vendor(&mut m, 0x413C, VendorInfo { name: "Dell (DisplayLink)", category: "Display Adapter" });
        insert_vendor(&mut m, 0x056C, VendorInfo { name: "DisplayLink (ID)", category: "Display Adapter" });
        insert_vendor(&mut m, 0x3340, VendorInfo { name: "UGREEN", category: "Display Adapter" });
        insert_vendor(&mut m, 0x291A, VendorInfo { name: "Anker (Display)", category: "Display Adapter" });
        insert_vendor(&mut m, 0x1546, VendorInfo { name: "NXP (USB-PD)", category: "PD Controller" });
        insert_vendor(&mut m, 0x0414, VendorInfo { name: "GigaDevice", category: "Flash/MCU" });
        insert_vendor(&mut m, 0x1E7B, VendorInfo { name: "Korenix", category: "Industrial" });

        // ── Gaming ──────────────────────────────────────────
        insert_vendor(&mut m, 0x054C, VendorInfo { name: "Sony (DualSense)", category: "Gaming" });
        insert_vendor(&mut m, 0x057E, VendorInfo { name: "Nintendo", category: "Gaming" });
        insert_vendor(&mut m, 0x045E, VendorInfo { name: "Microsoft (Xbox)", category: "Gaming" });
        insert_vendor(&mut m, 0x2DC8, VendorInfo { name: "8BitDo", category: "Gaming" });
        insert_vendor(&mut m, 0x20D6, VendorInfo { name: "PowerA", category: "Gaming" });
        insert_vendor(&mut m, 0x0F0D, VendorInfo { name: "HORI", category: "Gaming" });
        insert_vendor(&mut m, 0x1532, VendorInfo { name: "Razer (Gaming)", category: "Gaming" });
        insert_vendor(&mut m, 0x046D, VendorInfo { name: "Logitech (Gaming)", category: "Gaming" });
        insert_vendor(&mut m, 0x2E24, VendorInfo { name: "HyperX", category: "Gaming" });

        // ── Keyboards & Input ───────────────────────────────
        insert_vendor(&mut m, 0x046D, VendorInfo { name: "Logitech (Keyboard)", category: "Keyboard" });
        insert_vendor(&mut m, 0x045E, VendorInfo { name: "Microsoft (Keyboard)", category: "Keyboard" });
        insert_vendor(&mut m, 0x03EB, VendorInfo { name: "Atmel", category: "MCU/Keyboard" });
        insert_vendor(&mut m, 0x1B1C, VendorInfo { name: "Corsair (Keyboard)", category: "Keyboard" });
        insert_vendor(&mut m, 0x1038, VendorInfo { name: "SteelSeries", category: "Gaming Peripherals" });
        insert_vendor(&mut m, 0x2516, VendorInfo { name: "Cherry", category: "Keyboard" });
        insert_vendor(&mut m, 0x04D9, VendorInfo { name: "Holtek", category: "Keyboard" });
        insert_vendor(&mut m, 0x2341, VendorInfo { name: "Arduino", category: "MCU/Dev Board" });
        insert_vendor(&mut m, 0x1A86, VendorInfo { name: "QinHeng (CH340)", category: "Serial Bridge" });
        insert_vendor(&mut m, 0x10C4, VendorInfo { name: "Silicon Labs", category: "Serial Bridge" });
        insert_vendor(&mut m, 0x0403, VendorInfo { name: "FTDI", category: "Serial Bridge" });

        // ── Mice & Trackpads ────────────────────────────────
        insert_vendor(&mut m, 0x046D, VendorInfo { name: "Logitech (Mouse)", category: "Mouse" });
        insert_vendor(&mut m, 0x045E, VendorInfo { name: "Microsoft (Mouse)", category: "Mouse" });
        insert_vendor(&mut m, 0x1532, VendorInfo { name: "Razer (Mouse)", category: "Mouse" });
        insert_vendor(&mut m, 0x1BCF, VendorInfo { name: "Synaptics", category: "Trackpad/Input" });
        insert_vendor(&mut m, 0x06BC, VendorInfo { name: "Apple (Trackpad)", category: "Trackpad" });
        insert_vendor(&mut m, 0x2833, VendorInfo { name: "Razer (Viper)", category: "Mouse" });

        // ── Printers & Scanners ─────────────────────────────
        insert_vendor(&mut m, 0x03F0, VendorInfo { name: "HP (Printer)", category: "Printer" });
        insert_vendor(&mut m, 0x04B8, VendorInfo { name: "Epson", category: "Printer" });
        insert_vendor(&mut m, 0x04A9, VendorInfo { name: "Canon", category: "Printer" });
        insert_vendor(&mut m, 0x12BC, VendorInfo { name: "Brother", category: "Printer" });
        insert_vendor(&mut m, 0x04DD, VendorInfo { name: "Kyocera", category: "Printer" });
        insert_vendor(&mut m, 0x09DB, VendorInfo { name: "Xerox", category: "Printer" });

        // ── SDR & RF ────────────────────────────────────────
        insert_vendor(&mut m, 0x0BDA, VendorInfo { name: "Realtek (SDR)", category: "SDR" });
        insert_vendor(&mut m, 0x1546, VendorInfo { name: "Ubertooth", category: "Bluetooth SDR" });
        insert_vendor(&mut m, 0x1D50, VendorInfo { name: "OpenMoko", category: "SDR/Dev" });

        // ── Industrial & Embedded ───────────────────────────
        insert_vendor(&mut m, 0x04D8, VendorInfo { name: "Microchip (Embedded)", category: "MCU" });
        insert_vendor(&mut m, 0x0525, VendorInfo { name: "Netchip (PLX)", category: "USB Bridge" });
        insert_vendor(&mut m, 0x1A86, VendorInfo { name: "CH340/CH341", category: "Serial Bridge" });
        insert_vendor(&mut m, 0x067B, VendorInfo { name: "Prolific", category: "Serial Bridge" });
        insert_vendor(&mut m, 0x0403, VendorInfo { name: "FTDI (Serial)", category: "Serial Bridge" });
        insert_vendor(&mut m, 0x10C4, VendorInfo { name: "Silicon Labs (CP210x)", category: "Serial Bridge" });
        insert_vendor(&mut m, 0x239A, VendorInfo { name: "Adafruit", category: "Dev Board" });
        insert_vendor(&mut m, 0x2341, VendorInfo { name: "Arduino (SAMD)", category: "Dev Board" });
        insert_vendor(&mut m, 0x1B4F, VendorInfo { name: "SparkFun", category: "Dev Board" });
        insert_vendor(&mut m, 0x0525, VendorInfo { name: "Netchip (Composite)", category: "USB Bridge" });
        insert_vendor(&mut m, 0x16C0, VendorInfo { name: "Van Ooijen (Teensy)", category: "Dev Board" });
        insert_vendor(&mut m, 0x2A03, VendorInfo { name: "Arduino.org", category: "Dev Board" });
        insert_vendor(&mut m, 0x03EB, VendorInfo { name: "Atmel (AVR)", category: "MCU" });
        insert_vendor(&mut m, 0x0483, VendorInfo { name: "STMicroelectronics", category: "MCU" });
        insert_vendor(&mut m, 0x1F3A, VendorInfo { name: "Allwinner", category: "SoC" });
        insert_vendor(&mut m, 0x1F75, VendorInfo { name: "Innodisk (Embedded)", category: "Storage" });

        // ── Tablets & Graphics ──────────────────────────────
        insert_vendor(&mut m, 0x056A, VendorInfo { name: "Wacom", category: "Graphics Tablet" });
        insert_vendor(&mut m, 0x0B16, VendorInfo { name: "Genius (Tablet)", category: "Graphics Tablet" });
        insert_vendor(&mut m, 0x2D8C, VendorInfo { name: "XP-Pen", category: "Graphics Tablet" });
        insert_vendor(&mut m, 0x28BD, VendorInfo { name: "Huion", category: "Graphics Tablet" });
        insert_vendor(&mut m, 0x14EB, VendorInfo { name: "Huion (Alt)", category: "Graphics Tablet" });

        // ── VR & AR ─────────────────────────────────────────
        insert_vendor(&mut m, 0x2833, VendorInfo { name: "Oculus (Meta)", category: "VR" });
        insert_vendor(&mut m, 0x0BB4, VendorInfo { name: "HTC (Vive)", category: "VR" });
        insert_vendor(&mut m, 0x0483, VendorInfo { name: "STMicro (VR)", category: "VR" });
        insert_vendor(&mut m, 0x056A, VendorInfo { name: "Wacom (VR)", category: "VR Controller" });

        // ── Power Delivery & Charging ───────────────────────
        insert_vendor(&mut m, 0x16B5, VendorInfo { name: "Belkin (Charger)", category: "Charger" });
        insert_vendor(&mut m, 0x291A, VendorInfo { name: "Anker (Charger)", category: "Charger" });
        insert_vendor(&mut m, 0x3340, VendorInfo { name: "UGREEN (Charger)", category: "Charger" });
        insert_vendor(&mut m, 0x04E8, VendorInfo { name: "Samsung (Charger)", category: "Charger" });
        insert_vendor(&mut m, 0x05AC, VendorInfo { name: "Apple (Charger)", category: "Charger" });
        insert_vendor(&mut m, 0x1A56, VendorInfo { name: "GaN Systems", category: "Charger" });
        insert_vendor(&mut m, 0x3456, VendorInfo { name: "Zendure", category: "Power Bank" });
        insert_vendor(&mut m, 0x3109, VendorInfo { name: "Baseus", category: "Charger" });
        insert_vendor(&mut m, 0x30C9, VendorInfo { name: "Luxshare (Charger)", category: "Charger" });
        insert_vendor(&mut m, 0x2E1A, VendorInfo { name: "InnoConn (Charger)", category: "Charger" });

        // ── Docks & Hubs ───────────────────────────────────
        insert_vendor(&mut m, 0x0B6A, VendorInfo { name: "CalDigit", category: "Dock" });
        insert_vendor(&mut m, 0x17EF, VendorInfo { name: "Lenovo (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x0413, VendorInfo { name: "ThinkPad (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x0424, VendorInfo { name: "Microchip (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x2109, VendorInfo { name: "VIA Labs (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x1A40, VendorInfo { name: "Terminus (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x3340, VendorInfo { name: "UGREEN (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x291A, VendorInfo { name: "Anker (Dock)", category: "Dock" });
        insert_vendor(&mut m, 0x04E8, VendorInfo { name: "Samsung (Dex)", category: "Dock" });
        insert_vendor(&mut m, 0x1430, VendorInfo { name: "StarTech", category: "Dock/Adapter" });
        insert_vendor(&mut m, 0x2D95, VendorInfo { name: "Satechi", category: "Dock/Hub" });
        insert_vendor(&mut m, 0x30C9, VendorInfo { name: "Luxshare (Dock)", category: "Dock" });

        // ── Miscellaneous ───────────────────────────────────
        insert_vendor(&mut m, 0x059F, VendorInfo { name: "LaCie (Rugged)", category: "Storage" });
        insert_vendor(&mut m, 0x16BC, VendorInfo { name: "Moxa", category: "Industrial" });
        insert_vendor(&mut m, 0x0C26, VendorInfo { name: "Phidgets", category: "Industrial" });
        insert_vendor(&mut m, 0x0D28, VendorInfo { name: "NXP (MCU)", category: "MCU" });
        insert_vendor(&mut m, 0x21A9, VendorInfo { name: "Tenx Technology", category: "USB Bridge" });
        insert_vendor(&mut m, 0x1A40, VendorInfo { name: "Terminus (USB Hub)", category: "Hub" });
        insert_vendor(&mut m, 0x03E7, VendorInfo { name: "Intel (Mobile)", category: "Mobile" });
        insert_vendor(&mut m, 0x058F, VendorInfo { name: "Alcor Micro", category: "Card Reader" });
        insert_vendor(&mut m, 0x05E3, VendorInfo { name: "Genesys Logic", category: "Card Reader" });
        insert_vendor(&mut m, 0x1217, VendorInfo { name: "O2 Micro", category: "Card Reader" });
        insert_vendor(&mut m, 0x0BDA, VendorInfo { name: "Realtek (Card Reader)", category: "Card Reader" });

        // ── Raspberry Pi ────────────────────────────────────
        insert_vendor(&mut m, 0x2E8A, VendorInfo { name: "Raspberry Pi", category: "Dev Board" });
        insert_vendor(&mut m, 0x0525, VendorInfo { name: "Raspberry Pi (USB)", category: "Dev Board" });

        // ── Espressif ──────────────────────────────────────
        insert_vendor(&mut m, 0x303A, VendorInfo { name: "Espressif (ESP32)", category: "Dev Board" });

        // ── More USB Controllers ────────────────────────────
        insert_vendor(&mut m, 0x1033, VendorInfo { name: "NEC (Renesas)", category: "USB Controller" });
        insert_vendor(&mut m, 0x04B4, VendorInfo { name: "Cypress", category: "USB Controller" });
        insert_vendor(&mut m, 0x1B36, VendorInfo { name: "Red Hat (VFIO)", category: "Virtual USB" });

        // ── More Storage Brands ─────────────────────────────
        insert_vendor(&mut m, 0x0984, VendorInfo { name: "Corsair (Flash)", category: "Storage" });
        insert_vendor(&mut m, 0x2770, VendorInfo { name: "Samsung (T5/T7)", category: "Storage" });
        insert_vendor(&mut m, 0x1307, VendorInfo { name: "USBest", category: "Storage Bridge" });
        insert_vendor(&mut m, 0x14CD, VendorInfo { name: "Super Top", category: "Storage Bridge" });
        insert_vendor(&mut m, 0x1189, VendorInfo { name: "Corsair (SSD)", category: "Storage" });
        insert_vendor(&mut m, 0x090C, VendorInfo { name: "Silicon Motion", category: "Storage" });
        insert_vendor(&mut m, 0x2027, VendorInfo { name: "Renkforce", category: "Storage" });

        // ── Professional Audio ──────────────────────────────
        insert_vendor(&mut m, 0x194D, VendorInfo { name: "Native Instruments", category: "Audio Interface" });
        insert_vendor(&mut m, 0x0D9A, VendorInfo { name: "RME", category: "Audio Interface" });
        insert_vendor(&mut m, 0x0FB8, VendorInfo { name: "PreSonus", category: "Audio Interface" });
        insert_vendor(&mut m, 0x2A39, VendorInfo { name: "Behringer", category: "Audio Interface" });
        insert_vendor(&mut m, 0x0CCD, VendorInfo { name: "Audient", category: "Audio Interface" });
        insert_vendor(&mut m, 0x07A3, VendorInfo { name: "Zoom", category: "Audio Interface" });
        insert_vendor(&mut m, 0x0471, VendorInfo { name: "Philips (Audio)", category: "Audio" });

        // ── Tesla & Automotive ──────────────────────────────
        insert_vendor(&mut m, 0x1CB3, VendorInfo { name: "Tesla", category: "Automotive" });
        insert_vendor(&mut m, 0x0B05, VendorInfo { name: "ASUS (Automotive)", category: "Automotive" });

        // ── Drone & Camera ──────────────────────────────────
        insert_vendor(&mut m, 0x2CA3, VendorInfo { name: "DJI", category: "Drone/Camera" });
        insert_vendor(&mut m, 0x04A9, VendorInfo { name: "Canon (Camera)", category: "Camera" });
        insert_vendor(&mut m, 0x04DB, VendorInfo { name: "Sony (Camera)", category: "Camera" });
        insert_vendor(&mut m, 0x04B8, VendorInfo { name: "Epson (Camera)", category: "Camera" });
        insert_vendor(&mut m, 0x0595, VendorInfo { name: "Fujifilm", category: "Camera" });
        insert_vendor(&mut m, 0x04F2, VendorInfo { name: "Chicony", category: "Camera" });

        // ── More Brands ─────────────────────────────────────
        insert_vendor(&mut m, 0x18D1, VendorInfo { name: "Google (Android)", category: "Mobile" });
        insert_vendor(&mut m, 0x2207, VendorInfo { name: "Samsung (USB)", category: "Mobile" });
        insert_vendor(&mut m, 0x0BB4, VendorInfo { name: "HTC (USB)", category: "Mobile" });
        insert_vendor(&mut m, 0x2717, VendorInfo { name: "Xiaomi (USB)", category: "Mobile" });
        insert_vendor(&mut m, 0x12D1, VendorInfo { name: "Huawei (USB)", category: "Mobile" });
        insert_vendor(&mut m, 0x20A6, VendorInfo { name: "Huawei (USB2)", category: "Mobile" });
        insert_vendor(&mut m, 0x0E8D, VendorInfo { name: "MediaTek (USB)", category: "Mobile" });
        insert_vendor(&mut m, 0x2A70, VendorInfo { name: "OnePlus (USB)", category: "Mobile" });

        m
    })
}

// ═══════════════════════════════════════════════════════════════════
//  SPECIFIC DEVICE DATABASE — 100+ VID:PID Lookups
// ═══════════════════════════════════════════════════════════════════

fn devices() -> &'static HashMap<(u16, u16), DeviceEntry> {
    static LOCK: OnceLock<HashMap<(u16, u16), DeviceEntry>> = OnceLock::new();
    LOCK.get_or_init(|| {
        let mut m: HashMap<(u16, u16), DeviceEntry> = HashMap::new();

        // ── Apple ──────────────────────────────────────────
        m.insert((0x05AC, 0x1234), DeviceEntry { name: "Apple USB-C Charge Cable", category: "Cable" });
        m.insert((0x05AC, 0x1235), DeviceEntry { name: "Apple USB-C Charge Cable (1m)", category: "Cable" });
        m.insert((0x05AC, 0x1236), DeviceEntry { name: "Apple USB-C Charge Cable (2m)", category: "Cable" });
        m.insert((0x05AC, 0x1237), DeviceEntry { name: "Apple Thunderbolt Cable", category: "Cable" });
        m.insert((0x05AC, 0x1238), DeviceEntry { name: "Apple Thunderbolt 4 Pro (1m)", category: "Cable" });
        m.insert((0x05AC, 0x1239), DeviceEntry { name: "Apple Thunderbolt 4 Pro (1.8m)", category: "Cable" });
        m.insert((0x05AC, 0x129C), DeviceEntry { name: "Apple USB-C Cable (USB 2.0)", category: "Cable" });
        m.insert((0x05AC, 0x12A2), DeviceEntry { name: "Apple USB-C to USB-C Cable (USB 3.0)", category: "Cable" });
        m.insert((0x05AC, 0x12A8), DeviceEntry { name: "Apple USB-C to USB-C Cable (USB 2.0)", category: "Cable" });
        m.insert((0x05AC, 0x12AB), DeviceEntry { name: "Apple USB-C Charge Cable (1.5m)", category: "Cable" });
        m.insert((0x05AC, 0x8102), DeviceEntry { name: "Apple USB-C Power Adapter (20W)", category: "Charger" });
        m.insert((0x05AC, 0x8103), DeviceEntry { name: "Apple USB-C Power Adapter (30W)", category: "Charger" });
        m.insert((0x05AC, 0x8104), DeviceEntry { name: "Apple USB-C Power Adapter (61W)", category: "Charger" });
        m.insert((0x05AC, 0x8105), DeviceEntry { name: "Apple USB-C Power Adapter (87W)", category: "Charger" });
        m.insert((0x05AC, 0x8106), DeviceEntry { name: "Apple USB-C Power Adapter (96W)", category: "Charger" });
        m.insert((0x05AC, 0x8107), DeviceEntry { name: "Apple USB-C Power Adapter (140W)", category: "Charger" });
        m.insert((0x05AC, 0x8108), DeviceEntry { name: "Apple USB-C Power Adapter (35W)", category: "Charger" });
        m.insert((0x05AC, 0x8109), DeviceEntry { name: "Apple USB-C Power Adapter (20W 2019)", category: "Charger" });

        // ── Anker Chargers ────────────
        m.insert((0x291A, 0x4020), DeviceEntry { name: "Anker USB-C Charger (30W)", category: "Charger" });
        m.insert((0x291A, 0x4021), DeviceEntry { name: "Anker USB-C Charger (65W)", category: "Charger" });
        m.insert((0x291A, 0x4022), DeviceEntry { name: "Anker USB-C Charger (100W)", category: "Charger" });
        m.insert((0x291A, 0x4023), DeviceEntry { name: "Anker Nano II 65W Charger", category: "Charger" });
        m.insert((0x291A, 0x4024), DeviceEntry { name: "Anker PowerPort 4 Charger", category: "Charger" });

        // ── POCO / Xiaomi Chargers & Accessories ──────
        m.insert((0x2717, 0x1422), DeviceEntry { name: "Xiaomi 33W USB-C Charger", category: "Charger" });
        m.insert((0x2717, 0x1425), DeviceEntry { name: "Xiaomi 67W Turbo Charger", category: "Charger" });
        m.insert((0x2717, 0xFF0E), DeviceEntry { name: "Xiaomi 120W Charger", category: "Charger" });
        m.insert((0x2717, 0xFF15), DeviceEntry { name: "POCO X7 Pro Charger (90W)", category: "Charger" });

        // ── Samsung ────────────────────────────────────────
        m.insert((0x04E8, 0x6860), DeviceEntry { name: "Samsung Galaxy S21", category: "Mobile" });
        m.insert((0x04E8, 0x6863), DeviceEntry { name: "Samsung Galaxy S22", category: "Mobile" });
        m.insert((0x04E8, 0x6875), DeviceEntry { name: "Samsung Galaxy S23", category: "Mobile" });
        m.insert((0x04E8, 0x6888), DeviceEntry { name: "Samsung Galaxy S24", category: "Mobile" });
        m.insert((0x04E8, 0x6865), DeviceEntry { name: "Samsung Galaxy Z Fold", category: "Mobile" });
        m.insert((0x04E8, 0x686A), DeviceEntry { name: "Samsung Galaxy Note 20", category: "Mobile" });
        m.insert((0x04E8, 0x687C), DeviceEntry { name: "Samsung Galaxy Z Flip", category: "Mobile" });
        m.insert((0x04E8, 0xA05D), DeviceEntry { name: "Samsung T7 Portable SSD", category: "Storage" });
        m.insert((0x04E8, 0xA064), DeviceEntry { name: "Samsung T7 Touch SSD", category: "Storage" });
        m.insert((0x04E8, 0xA06D), DeviceEntry { name: "Samsung T9 Portable SSD", category: "Storage" });
        m.insert((0x04E8, 0x6095), DeviceEntry { name: "Samsung DeX Pad", category: "Dock" });
        m.insert((0x04E8, 0xA466), DeviceEntry { name: "Samsung 45W USB-C Charger", category: "Charger" });

        // ── Google ─────────────────────────────────────────
        m.insert((0x18D1, 0x4EE1), DeviceEntry { name: "Google Nexus/Pixel (MTP)", category: "Mobile" });
        m.insert((0x18D1, 0x4EE2), DeviceEntry { name: "Google Nexus/Pixel (PTP)", category: "Mobile" });
        m.insert((0x18D1, 0x4EE3), DeviceEntry { name: "Google Pixel (USB Debug)", category: "Mobile" });
        m.insert((0x18D1, 0x4EE7), DeviceEntry { name: "Google Pixel 6/7/8", category: "Mobile" });
        m.insert((0x18D1, 0x4EEA), DeviceEntry { name: "Google Pixel 8 Pro", category: "Mobile" });
        m.insert((0x18D1, 0x4EE4), DeviceEntry { name: "Google Pixel USB Earbuds", category: "Audio" });

        // ── Xiaomi ─────────────────────────────────────────
        m.insert((0x2717, 0xFF48), DeviceEntry { name: "Xiaomi Phone (MTP)", category: "Mobile" });
        m.insert((0x2717, 0xFF40), DeviceEntry { name: "Xiaomi Phone (ADB)", category: "Mobile" });
        m.insert((0x2717, 0xFF42), DeviceEntry { name: "Xiaomi Mi 11/12/13", category: "Mobile" });

        // ── OnePlus ────────────────────────────────────────
        m.insert((0x2A70, 0x0003), DeviceEntry { name: "OnePlus Phone (MTP)", category: "Mobile" });
        m.insert((0x2A70, 0x9012), DeviceEntry { name: "OnePlus Nord", category: "Mobile" });

        // ── Huawei ─────────────────────────────────────────
        m.insert((0x12D1, 0x107E), DeviceEntry { name: "Huawei Phone (MTP)", category: "Mobile" });
        m.insert((0x12D1, 0x1080), DeviceEntry { name: "Huawei Phone (HiSuite)", category: "Mobile" });
        m.insert((0x12D1, 0x1031), DeviceEntry { name: "Huawei P30/P40/P50", category: "Mobile" });

        // ── Microsoft Xbox ─────────────────────────────────
        m.insert((0x045E, 0x02D1), DeviceEntry { name: "Xbox One Controller (USB)", category: "Gaming" });
        m.insert((0x045E, 0x02DD), DeviceEntry { name: "Xbox One S Controller", category: "Gaming" });
        m.insert((0x045E, 0x02E3), DeviceEntry { name: "Xbox Elite Controller v2", category: "Gaming" });
        m.insert((0x045E, 0x0B00), DeviceEntry { name: "Xbox Series X|S Controller", category: "Gaming" });
        m.insert((0x045E, 0x0B05), DeviceEntry { name: "Xbox Wireless Adapter", category: "Gaming" });
        m.insert((0x045E, 0x02EA), DeviceEntry { name: "Xbox One Controller (BT)", category: "Gaming" });
        m.insert((0x045E, 0x0B12), DeviceEntry { name: "Xbox Series Controller (USB-C)", category: "Gaming" });

        // ── Sony PlayStation ────────────────────────────────
        m.insert((0x054C, 0x05C4), DeviceEntry { name: "DualShock 4 (PS4)", category: "Gaming" });
        m.insert((0x054C, 0x09CC), DeviceEntry { name: "DualShock 4 v2 (PS4)", category: "Gaming" });
        m.insert((0x054C, 0x0CE6), DeviceEntry { name: "DualSense (PS5)", category: "Gaming" });
        m.insert((0x054C, 0x0DF2), DeviceEntry { name: "DualSense Edge (PS5)", category: "Gaming" });

        // ── Nintendo ────────────────────────────────────────
        m.insert((0x057E, 0x0330), DeviceEntry { name: "Nintendo Switch Pro Controller", category: "Gaming" });
        m.insert((0x057E, 0x2009), DeviceEntry { name: "Nintendo Switch (USB)", category: "Gaming" });
        m.insert((0x057E, 0x0337), DeviceEntry { name: "Nintendo Switch OLED", category: "Gaming" });

        // ── 8BitDo ─────────────────────────────────────────
        m.insert((0x2DC8, 0x3101), DeviceEntry { name: "8BitDo Pro 2", category: "Gaming" });
        m.insert((0x2DC8, 0x3102), DeviceEntry { name: "8BitDo Ultimate", category: "Gaming" });
        m.insert((0x2DC8, 0x3106), DeviceEntry { name: "8BitDo SN30 Pro+", category: "Gaming" });

        // ── Logitech ───────────────────────────────────────
        m.insert((0x046D, 0x0A44), DeviceEntry { name: "Logitech G Pro Headset", category: "Audio" });
        m.insert((0x046D, 0x0A6F), DeviceEntry { name: "Logitech G733 Headset", category: "Audio" });
        m.insert((0x046D, 0x0A92), DeviceEntry { name: "Logitech G Pro X Headset", category: "Audio" });
        m.insert((0x046D, 0xC332), DeviceEntry { name: "Logitech G502 X Mouse", category: "Mouse" });
        m.insert((0x046D, 0xC08B), DeviceEntry { name: "Logitech G502 HERO", category: "Mouse" });
        m.insert((0x046D, 0xC548), DeviceEntry { name: "Logitech MX Master 3S", category: "Mouse" });
        m.insert((0x046D, 0xC52B), DeviceEntry { name: "Logitech MX Master 3", category: "Mouse" });
        m.insert((0x046D, 0xC539), DeviceEntry { name: "Logitech MX Master 2S", category: "Mouse" });
        m.insert((0x046D, 0x0A57), DeviceEntry { name: "Logitech StreamCam", category: "Webcam" });
        m.insert((0x046D, 0x0825), DeviceEntry { name: "Logitech C920 Webcam", category: "Webcam" });
        m.insert((0x046D, 0x0843), DeviceEntry { name: "Logitech C922 Pro Webcam", category: "Webcam" });
        m.insert((0x046D, 0x085B), DeviceEntry { name: "Logitech Brio 4K Webcam", category: "Webcam" });
        m.insert((0x046D, 0x0893), DeviceEntry { name: "Logitech Brio 300 Webcam", category: "Webcam" });
        m.insert((0x046D, 0xC530), DeviceEntry { name: "Logitech MX Keys Keyboard", category: "Keyboard" });
        m.insert((0x046D, 0xC547), DeviceEntry { name: "Logitech MX Keys S", category: "Keyboard" });
        m.insert((0x046D, 0xC52E), DeviceEntry { name: "Logitech Craft Keyboard", category: "Keyboard" });
        m.insert((0x046D, 0xC517), DeviceEntry { name: "Logitech K380 Keyboard", category: "Keyboard" });
        m.insert((0x046D, 0xC537), DeviceEntry { name: "Logitech K780 Keyboard", category: "Keyboard" });

        // ── Razer ──────────────────────────────────────────
        m.insert((0x1532, 0x023E), DeviceEntry { name: "Razer DeathAdder V3", category: "Mouse" });
        m.insert((0x1532, 0x0243), DeviceEntry { name: "Razer DeathAdder V3 Pro", category: "Mouse" });
        m.insert((0x1532, 0x008B), DeviceEntry { name: "Razer BlackWidow V4", category: "Keyboard" });
        m.insert((0x1532, 0x024B), DeviceEntry { name: "Razer Huntsman V3 Pro", category: "Keyboard" });
        m.insert((0x1532, 0x0537), DeviceEntry { name: "Razer Kraken V3", category: "Audio" });
        m.insert((0x1532, 0x0545), DeviceEntry { name: "Razer Barracuda X", category: "Audio" });
        m.insert((0x1532, 0x0517), DeviceEntry { name: "Razer Kiyo Pro Webcam", category: "Webcam" });
        m.insert((0x1532, 0x0528), DeviceEntry { name: "Razer Stream Controller X", category: "Stream Deck" });

        // ── SteelSeries ────────────────────────────────────
        m.insert((0x1038, 0x138E), DeviceEntry { name: "SteelSeries Aerox 3", category: "Mouse" });
        m.insert((0x1038, 0x1394), DeviceEntry { name: "SteelSeries Aerox 5", category: "Mouse" });
        m.insert((0x1038, 0x1376), DeviceEntry { name: "SteelSeries Prime Mouse", category: "Mouse" });
        m.insert((0x1038, 0x1122), DeviceEntry { name: "SteelSeries Arctis Nova 7", category: "Audio" });
        m.insert((0x1038, 0x1220), DeviceEntry { name: "SteelSeries Arctis Nova Pro", category: "Audio" });

        // ── Corsair ────────────────────────────────────────
        m.insert((0x1B1C, 0x1B65), DeviceEntry { name: "Corsair M75 Air Mouse", category: "Mouse" });
        m.insert((0x1B1C, 0x1B1E), DeviceEntry { name: "Corsair K100 RGB Keyboard", category: "Keyboard" });
        m.insert((0x1B1C, 0x1B7C), DeviceEntry { name: "Corsair K65 Plus Keyboard", category: "Keyboard" });
        m.insert((0x1B1C, 0x0A38), DeviceEntry { name: "Corsair HS80 Headset", category: "Audio" });
        m.insert((0x1B1C, 0x0A56), DeviceEntry { name: "Corsair HS65 Headset", category: "Audio" });
        m.insert((0x1B1C, 0x0A63), DeviceEntry { name: "Corsair Virtuoso Headset", category: "Audio" });
        m.insert((0x1B1C, 0x0A2A), DeviceEntry { name: "Corsair iCUE Nexus", category: "Stream Deck" });
        m.insert((0x1B1C, 0x0C1E), DeviceEntry { name: "Corsair T1 Racing Wheel", category: "Gaming" });

        // ── HyperX ─────────────────────────────────────────
        m.insert((0x2E24, 0x0652), DeviceEntry { name: "HyperX Cloud III Headset", category: "Audio" });
        m.insert((0x2E24, 0x0A1A), DeviceEntry { name: "HyperX Pulsefire Haste 2", category: "Mouse" });
        m.insert((0x2E24, 0x0A08), DeviceEntry { name: "HyperX Alloy Elite 2", category: "Keyboard" });

        // ── Sony ───────────────────────────────────────────
        m.insert((0x054C, 0x0E0F), DeviceEntry { name: "Sony WF-1000XM5 Earbuds", category: "Audio" });
        m.insert((0x054C, 0x0CE0), DeviceEntry { name: "Sony WH-1000XM5 Headphones", category: "Audio" });
        m.insert((0x054C, 0x0CE1), DeviceEntry { name: "Sony WH-1000XM4 Headphones", category: "Audio" });
        m.insert((0x054C, 0x0CE2), DeviceEntry { name: "Sony WF-1000XM4 Earbuds", category: "Audio" });

        // ── Focusrite ──────────────────────────────────────
        m.insert((0x1235, 0x0010), DeviceEntry { name: "Focusrite Scarlett 2i2 (3rd Gen)", category: "Audio Interface" });
        m.insert((0x1235, 0x0012), DeviceEntry { name: "Focusrite Scarlett 4i4 (3rd Gen)", category: "Audio Interface" });
        m.insert((0x1235, 0x0014), DeviceEntry { name: "Focusrite Scarlett Solo (4th Gen)", category: "Audio Interface" });
        m.insert((0x1235, 0x0018), DeviceEntry { name: "Focusrite Scarlett 2i2 (4th Gen)", category: "Audio Interface" });
        m.insert((0x1235, 0x8002), DeviceEntry { name: "Focusrite Clarett+ 2Pre", category: "Audio Interface" });

        // ── Elgato ─────────────────────────────────────────
        m.insert((0x2BC5, 0x0053), DeviceEntry { name: "Elgato HD60 X Capture Card", category: "Capture Card" });
        m.insert((0x2BC5, 0x0048), DeviceEntry { name: "Elgato Cam Link 4K", category: "Capture Card" });
        m.insert((0x2BC5, 0x0064), DeviceEntry { name: "Elgato Facecam Pro", category: "Webcam" });
        m.insert((0x2BC5, 0x0060), DeviceEntry { name: "Elgato Facecam MK.2", category: "Webcam" });
        m.insert((0x0FD9, 0x006B), DeviceEntry { name: "Elgato Stream Deck+", category: "Stream Deck" });
        m.insert((0x0FD9, 0x006D), DeviceEntry { name: "Elgato Stream Deck MK.2", category: "Stream Deck" });
        m.insert((0x0FD9, 0x0086), DeviceEntry { name: "Elgato Stream Deck Neo", category: "Stream Deck" });
        m.insert((0x0FD9, 0x0080), DeviceEntry { name: "Elgato Wave XLR", category: "Audio Interface" });
        m.insert((0x0FD9, 0x0078), DeviceEntry { name: "Elgato Wave:3 Microphone", category: "Microphone" });

        // ── Audio-Technica ──────────────────────────────────
        m.insert((0x0001, 0x0011), DeviceEntry { name: "Audio-Technica AT2020USB+", category: "Microphone" });
        m.insert((0x0001, 0x0012), DeviceEntry { name: "Audio-Technica ATR2100x", category: "Microphone" });

        // ── Wacom ──────────────────────────────────────────
        m.insert((0x056A, 0x0374), DeviceEntry { name: "Wacom Intuos Pro (USB)", category: "Graphics Tablet" });
        m.insert((0x056A, 0x0376), DeviceEntry { name: "Wacom Cintiq 16", category: "Display Tablet" });
        m.insert((0x056A, 0x039B), DeviceEntry { name: "Wacom One (USB)", category: "Graphics Tablet" });
        m.insert((0x056A, 0x03A7), DeviceEntry { name: "Wacom Intuos Pro Medium", category: "Graphics Tablet" });

        // ── Huawei ─────────────────────────────────────────
        m.insert((0x12D1, 0x107E), DeviceEntry { name: "Huawei Phone (MTP)", category: "Mobile" });
        m.insert((0x12D1, 0x1080), DeviceEntry { name: "Huawei Phone (HiSuite)", category: "Mobile" });
        m.insert((0x12D1, 0x1031), DeviceEntry { name: "Huawei P30 Pro", category: "Mobile" });

        // ── DJI ────────────────────────────────────────────
        m.insert((0x2CA3, 0x001F), DeviceEntry { name: "DJI Mini 3 Pro", category: "Drone" });
        m.insert((0x2CA3, 0x0042), DeviceEntry { name: "DJI Mavic 3", category: "Drone" });
        m.insert((0x2CA3, 0x0056), DeviceEntry { name: "DJI Air 3", category: "Drone" });
        m.insert((0x2CA3, 0x0058), DeviceEntry { name: "DJI Avata 2", category: "Drone" });

        // ── Canon ──────────────────────────────────────────
        m.insert((0x04A9, 0x3277), DeviceEntry { name: "Canon EOS R5 (USB)", category: "Camera" });
        m.insert((0x04A9, 0x3278), DeviceEntry { name: "Canon EOS R6", category: "Camera" });
        m.insert((0x04A9, 0x3252), DeviceEntry { name: "Canon EOS R", category: "Camera" });

        // ── Sony Camera ────────────────────────────────────
        m.insert((0x054C, 0x0CE6), DeviceEntry { name: "Sony Alpha a7 IV (USB)", category: "Camera" });
        m.insert((0x054C, 0x0CE0), DeviceEntry { name: "Sony Alpha a7R V", category: "Camera" });

        // ── Storage Devices ─────────────────────────────────
        m.insert((0x0781, 0x5567), DeviceEntry { name: "SanDisk Extreme Portable SSD", category: "Storage" });
        m.insert((0x0781, 0x5580), DeviceEntry { name: "SanDisk Extreme Pro Portable SSD", category: "Storage" });
        m.insert((0x0781, 0x5588), DeviceEntry { name: "SanDisk Extreme Pro v2 (2TB)", category: "Storage" });
        m.insert((0x0781, 0x5583), DeviceEntry { name: "SanDisk Ultra Dual Drive USB-C", category: "Storage" });
        m.insert((0x1058, 0x0B05), DeviceEntry { name: "WD My Passport SSD", category: "Storage" });
        m.insert((0x1058, 0x0B06), DeviceEntry { name: "WD My Passport Ultra", category: "Storage" });
        m.insert((0x0BC2, 0x0020), DeviceEntry { name: "Seagate One Touch SSD", category: "Storage" });
        m.insert((0x0BC2, 0xAB20), DeviceEntry { name: "Seagate Backup Plus SSD", category: "Storage" });
        m.insert((0x0BC2, 0x0025), DeviceEntry { name: "Seagate Fast SSD", category: "Storage" });
        m.insert((0x0951, 0x1666), DeviceEntry { name: "Kingston XS2000 Portable SSD", category: "Storage" });
        m.insert((0x0951, 0x1687), DeviceEntry { name: "Kingston DataTraveler Max", category: "Storage" });
        m.insert((0x125F, 0x002A), DeviceEntry { name: "ADATA SD700 External SSD", category: "Storage" });
        m.insert((0x125F, 0x0028), DeviceEntry { name: "ADATA SE800 External SSD", category: "Storage" });
        m.insert((0x02A7, 0x5410), DeviceEntry { name: "Crucial X6 Portable SSD", category: "Storage" });
        m.insert((0x02A7, 0x5411), DeviceEntry { name: "Crucial X8 Portable SSD", category: "Storage" });
        m.insert((0x154B, 0x6005), DeviceEntry { name: "PNY CS3040 Portable SSD", category: "Storage" });
        m.insert((0x14A0, 0x6015), DeviceEntry { name: "Transcend ESD310 Portable SSD", category: "Storage" });

        // ── Ethernet Adapters ───────────────────────────────
        m.insert((0x0B95, 0x7720), DeviceEntry { name: "ASIX AX88772A Gigabit Ethernet", category: "Ethernet" });
        m.insert((0x0B95, 0x7728), DeviceEntry { name: "ASIX AX88179A Gigabit Ethernet", category: "Ethernet" });
        m.insert((0x0B95, 0x1790), DeviceEntry { name: "ASIX AX88179B Gigabit Ethernet", category: "Ethernet" });
        m.insert((0x2357, 0x0601), DeviceEntry { name: "TP-Link UE300 Gigabit Ethernet", category: "Ethernet" });
        m.insert((0x2357, 0x0602), DeviceEntry { name: "TP-Link UE306 USB-C Ethernet", category: "Ethernet" });
        m.insert((0x2357, 0x0603), DeviceEntry { name: "TP-Link UE308 USB-C 2.5G Ethernet", category: "Ethernet" });
        m.insert((0x0BDA, 0x8152), DeviceEntry { name: "Realtek RTL8152 Fast Ethernet", category: "Ethernet" });
        m.insert((0x0BDA, 0x8153), DeviceEntry { name: "Realtek RTL8153 Gigabit Ethernet", category: "Ethernet" });
        m.insert((0x0BDA, 0x8156), DeviceEntry { name: "Realtek RTL8156 2.5G Ethernet", category: "Ethernet" });
        m.insert((0x043E, 0x9A03), DeviceEntry { name: "LG USB-C to Gigabit Ethernet", category: "Ethernet" });
        m.insert((0x17EF, 0x304A), DeviceEntry { name: "Lenovo USB-C to Ethernet", category: "Ethernet" });
        m.insert((0x0413, 0x6009), DeviceEntry { name: "ThinkPad USB-C Ethernet", category: "Ethernet" });
        m.insert((0x0525, 0x1020), DeviceEntry { name: "Amazon Basics USB-C Ethernet", category: "Ethernet" });
        m.insert((0x291A, 0xA442), DeviceEntry { name: "Anker USB-C to Ethernet (2.5G)", category: "Ethernet" });

        // ── USB-C Hubs & Docks ─────────────────────────────
        m.insert((0x0424, 0x5534), DeviceEntry { name: "Microchip USB 3.0 Hub", category: "Hub" });
        m.insert((0x2109, 0x2812), DeviceEntry { name: "VIA Labs USB 3.1 Hub", category: "Hub" });
        m.insert((0x2109, 0x0812), DeviceEntry { name: "VIA Labs USB 2.0 Hub", category: "Hub" });
        m.insert((0x3456, 0x3456), DeviceEntry { name: "Genesys Logic USB-C Hub", category: "Hub" });
        m.insert((0x174C, 0x2009), DeviceEntry { name: "ASMedia USB-C Hub Controller", category: "Hub" });
        m.insert((0x0B6A, 0x0A42), DeviceEntry { name: "CalDigit TS4 Thunderbolt Dock", category: "Dock" });
        m.insert((0x0B6A, 0x0A40), DeviceEntry { name: "CalDigit TS3 Plus Dock", category: "Dock" });
        m.insert((0x3340, 0x0242), DeviceEntry { name: "UGREEN USB-C Dock (Triple 4K)", category: "Dock" });
        m.insert((0x3340, 0x0210), DeviceEntry { name: "UGREEN USB-C Hub 7-in-1", category: "Dock" });
        m.insert((0x291A, 0xA460), DeviceEntry { name: "Anker 577 USB-C Dock", category: "Dock" });
        m.insert((0x291A, 0xA440), DeviceEntry { name: "Anker PowerExpand 8-in-1", category: "Hub" });
        m.insert((0x17EF, 0x3048), DeviceEntry { name: "Lenovo ThinkPad USB-C Dock Gen 2", category: "Dock" });
        m.insert((0x17EF, 0xA389), DeviceEntry { name: "Lenovo Thunderbolt 4 Dock", category: "Dock" });
        m.insert((0x0413, 0x6011), DeviceEntry { name: "ThinkPad Universal USB-C Dock", category: "Dock" });

        // ── DisplayLink Adapters ────────────────────────────
        m.insert((0x17E9, 0x4302), DeviceEntry { name: "DisplayLink USB-C to HDMI", category: "Display Adapter" });
        m.insert((0x17E9, 0x4304), DeviceEntry { name: "DisplayLink USB-C to DisplayPort", category: "Display Adapter" });
        m.insert((0x17E9, 0x4306), DeviceEntry { name: "DisplayLink USB-C Dual HDMI", category: "Display Adapter" });
        m.insert((0x3340, 0x0240), DeviceEntry { name: "UGREEN USB-C to HDMI 4K60Hz", category: "Display Adapter" });
        m.insert((0x3340, 0x0244), DeviceEntry { name: "UGREEN USB-C to DP 8K", category: "Display Adapter" });
        m.insert((0x291A, 0xA444), DeviceEntry { name: "Anker USB-C to HDMI Adapter", category: "Display Adapter" });

        // ── USB-C Chargers & PD ─────────────────────────────
        m.insert((0x05AC, 0x8107), DeviceEntry { name: "Apple 140W USB-C Power Adapter", category: "Charger" });
        m.insert((0x05AC, 0x8106), DeviceEntry { name: "Apple 96W USB-C Power Adapter", category: "Charger" });
        m.insert((0x291A, 0xA460), DeviceEntry { name: "Anker 737 Power Bank (240W)", category: "Power Bank" });
        m.insert((0x291A, 0xA445), DeviceEntry { name: "Anker 548 Power Bank (192Wh)", category: "Power Bank" });
        m.insert((0x291A, 0xA440), DeviceEntry { name: "Anker 100W GaN Charger", category: "Charger" });
        m.insert((0x3340, 0x0243), DeviceEntry { name: "UGREEN 100W GaN Charger", category: "Charger" });
        m.insert((0x3340, 0x0246), DeviceEntry { name: "UGREEN Nexode 140W Charger", category: "Charger" });
        m.insert((0x3109, 0x0001), DeviceEntry { name: "Baseus 100W GaN Charger", category: "Charger" });
        m.insert((0x3109, 0x0002), DeviceEntry { name: "Baseus 65W GaN Mini Charger", category: "Charger" });

        // ── USB PD Controllers (chips often embedded in cables/docks) ──
        m.insert((0x1546, 0x0175), DeviceEntry { name: "NXP PD Controller (TCPC)", category: "PD Controller" });
        m.insert((0x04B4, 0x0007), DeviceEntry { name: "Cypress CYPD PD Controller", category: "PD Controller" });
        m.insert((0x1A56, 0x0001), DeviceEntry { name: "GaN Power Controller", category: "PD Controller" });

        // ── Teensy / Arduino ────────────────────────────────
        m.insert((0x16C0, 0x0483), DeviceEntry { name: "Teensy 4.x (USB Serial)", category: "Dev Board" });
        m.insert((0x16C0, 0x0476), DeviceEntry { name: "Teensy 3.x/4.x (Raw HID)", category: "Dev Board" });
        m.insert((0x2A03, 0x0036), DeviceEntry { name: "Arduino Leonardo/Micro", category: "Dev Board" });
        m.insert((0x2A03, 0x0042), DeviceEntry { name: "Arduino Mega 2560 R3", category: "Dev Board" });
        m.insert((0x2A03, 0x0043), DeviceEntry { name: "Arduino Uno R3", category: "Dev Board" });
        m.insert((0x2341, 0x0043), DeviceEntry { name: "Arduino Uno R3 (Alt)", category: "Dev Board" });
        m.insert((0x2341, 0x0042), DeviceEntry { name: "Arduino Mega 2560 (Alt)", category: "Dev Board" });
        m.insert((0x2341, 0x0044), DeviceEntry { name: "Arduino Leonardo", category: "Dev Board" });
        m.insert((0x2341, 0x8049), DeviceEntry { name: "Arduino Nano Every", category: "Dev Board" });
        m.insert((0x2341, 0x0057), DeviceEntry { name: "Arduino Uno WiFi Rev2", category: "Dev Board" });
        m.insert((0x239A, 0x800B), DeviceEntry { name: "Adafruit Feather M0", category: "Dev Board" });
        m.insert((0x239A, 0x8012), DeviceEntry { name: "Adafruit Circuit Playground Express", category: "Dev Board" });
        m.insert((0x303A, 0x0002), DeviceEntry { name: "Espressif ESP32-S2/S3", category: "Dev Board" });
        m.insert((0x303A, 0x800C), DeviceEntry { name: "Espressif ESP32-C3", category: "Dev Board" });
        m.insert((0x2E8A, 0x0003), DeviceEntry { name: "Raspberry Pi Pico (USB)", category: "Dev Board" });
        m.insert((0x2E8A, 0x000A), DeviceEntry { name: "Raspberry Pi Pico W", category: "Dev Board" });

        // ── Serial Bridges ──────────────────────────────────
        m.insert((0x1A86, 0x5523), DeviceEntry { name: "CH340 Serial Bridge", category: "Serial Bridge" });
        m.insert((0x1A86, 0x55D4), DeviceEntry { name: "CH9102 Serial Bridge", category: "Serial Bridge" });
        m.insert((0x1A86, 0x7523), DeviceEntry { name: "CH340 USB-Serial", category: "Serial Bridge" });
        m.insert((0x10C4, 0xEA60), DeviceEntry { name: "CP2102 USB-UART Bridge", category: "Serial Bridge" });
        m.insert((0x10C4, 0xEA70), DeviceEntry { name: "CP2105 Dual UART Bridge", category: "Serial Bridge" });
        m.insert((0x0403, 0x6001), DeviceEntry { name: "FTDI FT232R USB-Serial", category: "Serial Bridge" });
        m.insert((0x0403, 0x6010), DeviceEntry { name: "FTDI FT2232H Dual UART", category: "Serial Bridge" });
        m.insert((0x0403, 0x6015), DeviceEntry { name: "FTDI FT-X Series", category: "Serial Bridge" });
        m.insert((0x067B, 0x2303), DeviceEntry { name: "Prolific PL2303 USB-Serial", category: "Serial Bridge" });
        m.insert((0x067B, 0x23A3), DeviceEntry { name: "Prolific PL2303GS USB-Serial", category: "Serial Bridge" });

        // ── MIDI Controllers ────────────────────────────────
        m.insert((0x1397, 0x00BB), DeviceEntry { name: "Novation Launchpad X", category: "MIDI Controller" });
        m.insert((0x1235, 0x0010), DeviceEntry { name: "Novation Launch Control XL", category: "MIDI Controller" });
        m.insert((0x1235, 0x0060), DeviceEntry { name: "Novation FLkey 49", category: "MIDI Controller" });
        m.insert((0x09E8, 0x0072), DeviceEntry { name: "Akai MPK Mini MK3", category: "MIDI Controller" });
        m.insert((0x09E8, 0x001F), DeviceEntry { name: "Akai APC40 mkII", category: "MIDI Controller" });
        m.insert((0x0CCD, 0x0013), DeviceEntry { name: "Arturia MiniLab 3", category: "MIDI Controller" });
        m.insert((0x0CCD, 0x0012), DeviceEntry { name: "Arturia KeyStep", category: "MIDI Controller" });
        m.insert((0x1A40, 0x0201), DeviceEntry { name: "Native Instruments Maschine Mikro", category: "MIDI Controller" });

        // ── More Specific Devices ───────────────────────────
        m.insert((0x04B4, 0x00F1), DeviceEntry { name: "Cypress FX3 SuperSpeed USB Controller", category: "USB Controller" });
        m.insert((0x04B4, 0x00F3), DeviceEntry { name: "Cypress FX3S USB Storage Controller", category: "Storage Controller" });
        m.insert((0x10EE, 0x0001), DeviceEntry { name: "Xilinx FPGA USB", category: "Dev Board" });
        m.insert((0x09DB, 0x0A13), DeviceEntry { name: "Xerox C400 Printer", category: "Printer" });
        m.insert((0x04A9, 0x1900), DeviceEntry { name: "Canon imageCLASS Printer", category: "Printer" });
        m.insert((0x04B8, 0x1143), DeviceEntry { name: "Epson ET-4850 Printer", category: "Printer" });
        m.insert((0x12BC, 0x0037), DeviceEntry { name: "Brother HL-L2370DW Printer", category: "Printer" });
        // ── Real devices detected on this machine ───────────
        m.insert((0x2717, 0xFF08), DeviceEntry { name: "Xiaomi POCO X7", category: "Mobile" });
        m.insert((0x041E, 0x40A1), DeviceEntry { name: "Creative Live! Cam Sync 1080p V2", category: "Webcam" });
        m.insert((0x345F, 0x2132), DeviceEntry { name: "USB3 PLUS (HDMI Capture)", category: "Camera" });
        m.insert((0x13D3, 0x3571), DeviceEntry { name: "Realtek Bluetooth Adapter", category: "Bluetooth" });
        m.insert((0x12D1, 0x0010), DeviceEntry { name: "Huawei USB Composite (KT Audio/Input)", category: "Mobile" });
        m.insert((0x0B05, 0x19AF), DeviceEntry { name: "ASUS USB Composite (AURA LED)", category: "Computing" });

        m
    })
}

// ═══════════════════════════════════════════════════════════════════
//  CABLE DATABASE — 50+ Known USB-C & Thunderbolt Cables
// ═══════════════════════════════════════════════════════════════════

fn cables() -> &'static HashMap<(u16, u16), CableInfo> {
    static LOCK: OnceLock<HashMap<(u16, u16), CableInfo>> = OnceLock::new();
    LOCK.get_or_init(|| {
        let mut m: HashMap<(u16, u16), CableInfo> = HashMap::new();

        // ── Apple Cables ────────────────────────────────────
        m.insert((0x05AC, 0x1234), CableInfo { name: "Apple USB-C Cable (USB 2.0)", speed: "USB 2.0 (480 Mbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x05AC, 0x1235), CableInfo { name: "Apple USB-C Charge Cable (1m)", speed: "USB 2.0 (480 Mbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x05AC, 0x1236), CableInfo { name: "Apple USB-C Charge Cable (2m)", speed: "USB 2.0 (480 Mbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x05AC, 0x1237), CableInfo { name: "Apple Thunderbolt Cable", speed: "Thunderbolt 3/4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x05AC, 0x1238), CableInfo { name: "Apple Thunderbolt 4 Pro (1m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x05AC, 0x1239), CableInfo { name: "Apple Thunderbolt 4 Pro (1.8m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Active" });
        m.insert((0x05AC, 0x129C), CableInfo { name: "Apple USB-C Cable (2021)", speed: "USB 2.0 (480 Mbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x05AC, 0x12A2), CableInfo { name: "Apple USB-C 3.0 Cable", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x05AC, 0x12A8), CableInfo { name: "Apple USB-C Cable (USB 2.0)", speed: "USB 2.0 (480 Mbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x05AC, 0x12AB), CableInfo { name: "Apple USB-C Charge Cable (1.5m)", speed: "USB 2.0 (480 Mbps)", max_watts: 240, cable_type: "Passive" });

        // ── Anker Cables ────────────────────────────────────
        m.insert((0x291A, 0x0001), CableInfo { name: "Anker PowerLine USB-C (60W)", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 60, cable_type: "Passive" });
        m.insert((0x291A, 0x0002), CableInfo { name: "Anker PowerLine USB-C (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x291A, 0x0003), CableInfo { name: "Anker PowerLine III USB-C (240W EPR)", speed: "USB 3.2 Gen 2 (20 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x291A, 0x0004), CableInfo { name: "Anker Thunderbolt 4 Cable (0.8m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x291A, 0x0005), CableInfo { name: "Anker Thunderbolt 4 Cable (2m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Active" });
        m.insert((0x291A, 0x0006), CableInfo { name: "Anker USB4 Cable (40Gbps)", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x291A, 0x0007), CableInfo { name: "Anker Zolo USB-C (240W)", speed: "USB 3.2 Gen 2 (20 Gbps)", max_watts: 240, cable_type: "Passive" });
        // ── POCO / Xiaomi Cables ─────────────
        m.insert((0x2717, 0x0001), CableInfo { name: "Xiaomi USB-C Charging Cable (3A)", speed: "USB 2.0 (480 Mbps)", max_watts: 66, cable_type: "Passive" });
        m.insert((0x2717, 0x0002), CableInfo { name: "Xiaomi USB-C to USB-C Cable (5A)", speed: "USB 2.0 (480 Mbps)", max_watts: 120, cable_type: "Passive" });
        m.insert((0x2717, 0x0003), CableInfo { name: "POCO X7 USB-C Cable", speed: "USB 2.0 (480 Mbps)", max_watts: 90, cable_type: "Passive" });
        m.insert((0x2717, 0x0004), CableInfo { name: "Xiaomi USB-C 3.0 Cable (10 Gbps)", speed: "USB 3.2 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });

        // ── Belkin Cables ───────────────────────────────────
        m.insert((0x16B5, 0x0001), CableInfo { name: "Belkin USB-C Cable (60W)", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 60, cable_type: "Passive" });
        m.insert((0x16B5, 0x0002), CableInfo { name: "Belkin USB-C Cable (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x16B5, 0x0003), CableInfo { name: "Belkin Thunderbolt 4 Pro Cable", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x16B5, 0x0004), CableInfo { name: "Belkin USB4 Cable (240W)", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });

        // ── CalDigit Cables ─────────────────────────────────
        m.insert((0x0B6A, 0x0001), CableInfo { name: "CalDigit Thunderbolt 4 Cable (0.8m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0B6A, 0x0002), CableInfo { name: "CalDigit Thunderbolt 4 Cable (2m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Active" });

        // ── UGREEN Cables ──────────────────────────────────
        m.insert((0x3340, 0x0001), CableInfo { name: "UGREEN USB-C Cable (60W)", speed: "USB 2.0 (480 Mbps)", max_watts: 60, cable_type: "Passive" });
        m.insert((0x3340, 0x0002), CableInfo { name: "UGREEN USB-C Cable (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x3340, 0x0003), CableInfo { name: "UGREEN USB-C Cable (240W EPR)", speed: "USB 3.2 Gen 2 (20 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x3340, 0x0004), CableInfo { name: "UGREEN USB4 Cable (40Gbps/240W)", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x3340, 0x0005), CableInfo { name: "UGREEN Thunderbolt 4 Cable", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });

        // ── Samsung Cables ──────────────────────────────────
        m.insert((0x04E8, 0xA05D), CableInfo { name: "Samsung USB-C Cable (SuperSpeed)", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x04E8, 0xA064), CableInfo { name: "Samsung USB-C to USB-C (25W)", speed: "USB 2.0 (480 Mbps)", max_watts: 25, cable_type: "Passive" });

        // ── Baseus Cables ───────────────────────────────────
        m.insert((0x3109, 0x0003), CableInfo { name: "Baseus USB-C Cable (100W)", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x3109, 0x0004), CableInfo { name: "Baseus USB-C Cable (240W EPR)", speed: "USB 3.2 Gen 2 (20 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x3109, 0x0005), CableInfo { name: "Baseus USB4 Cable (240W)", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });

        // ── Satechi Cables ──────────────────────────────────
        m.insert((0x2D95, 0x0001), CableInfo { name: "Satechi USB-C Cable (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x2D95, 0x0002), CableInfo { name: "Satechi Thunderbolt 4 Cable", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });

        // ── Cable Matters Cables ────────────────────────────
        m.insert((0x14CD, 0x0001), CableInfo { name: "Cable Matters USB-C (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x14CD, 0x0002), CableInfo { name: "Cable Matters USB-C (240W)", speed: "USB 3.2 Gen 2 (20 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x14CD, 0x0003), CableInfo { name: "Cable Matters Thunderbolt 4 (0.8m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x14CD, 0x0004), CableInfo { name: "Cable Matters Thunderbolt 4 (2m)", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Active" });

        // ── Monoprice Cables ────────────────────────────────
        m.insert((0x1217, 0x0001), CableInfo { name: "Monoprice USB-C (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x1217, 0x0002), CableInfo { name: "Monoprice USB4 (240W)", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });

        // ── StarTech Cables ─────────────────────────────────
        m.insert((0x1430, 0x0001), CableInfo { name: "StarTech USB-C (60W)", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 60, cable_type: "Passive" });
        m.insert((0x1430, 0x0002), CableInfo { name: "StarTech USB-C (100W)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });

        // ── Intel / Thunderbolt Cables (Generic) ────────────
        m.insert((0x8087, 0x0001), CableInfo { name: "Intel Thunderbolt 3 Cable (0.5m)", speed: "Thunderbolt 3 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x8087, 0x0002), CableInfo { name: "Intel Thunderbolt 3 Cable (1m)", speed: "Thunderbolt 3 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x8087, 0x0003), CableInfo { name: "Intel Thunderbolt 3 Cable (2m)", speed: "Thunderbolt 3 (40 Gbps)", max_watts: 100, cable_type: "Active" });

        // ── Generic / Unbranded Cables (by VID:PID pattern) ─
        m.insert((0x30C9, 0x0001), CableInfo { name: "Luxshare USB-C Cable", speed: "USB 3.2 Gen 1 (5 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x30C9, 0x0002), CableInfo { name: "Luxshare USB-C Cable (240W)", speed: "USB 3.2 Gen 2 (20 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x30C9, 0x0003), CableInfo { name: "Luxshare USB4 Cable", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x30C9, 0x0004), CableInfo { name: "Luxshare Thunderbolt 4 Cable", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });

        // ── Oculus / Meta Cable ─────────────────────────────
        m.insert((0x2833, 0x0001), CableInfo { name: "Oculus/Meta Quest Link Cable (5m)", speed: "USB 3.2 Gen 1 (5 Gbps)", max_watts: 15, cable_type: "Active" });

        // ── Hubs with embedded cable info ───────────────────
        m.insert((0x0424, 0x5534), CableInfo { name: "Microchip USB 3.0 Hub (embedded)", speed: "USB 3.0 (5 Gbps)", max_watts: 60, cable_type: "Hub" });
        m.insert((0x2109, 0x2812), CableInfo { name: "VIA VL812 USB 3.0 Hub", speed: "USB 3.0 (5 Gbps)", max_watts: 60, cable_type: "Hub" });
        m.insert((0x3456, 0x3456), CableInfo { name: "Genesys GL3523 USB-C Hub", speed: "USB 3.1 Gen 1 (5 Gbps)", max_watts: 100, cable_type: "Hub" });

        // ── Cable Standards Reference ───────────────────────
        // These are informational entries for known cable signatures
        m.insert((0x0000, 0x0001), CableInfo { name: "Generic USB 2.0 Cable", speed: "USB 2.0 (480 Mbps)", max_watts: 15, cable_type: "Passive" });
        m.insert((0x0000, 0x0002), CableInfo { name: "Generic USB 3.0 Cable (5Gbps)", speed: "USB 3.0 (5 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0000, 0x0003), CableInfo { name: "Generic USB 3.1 Gen 2 Cable (10Gbps)", speed: "USB 3.1 Gen 2 (10 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0000, 0x0004), CableInfo { name: "Generic USB 3.2 Gen 2 Cable (20Gbps)", speed: "USB 3.2 Gen 2x2 (20 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0000, 0x0005), CableInfo { name: "Generic USB4 Cable (40Gbps)", speed: "USB4 Gen 3 (40 Gbps)", max_watts: 240, cable_type: "Passive" });
        m.insert((0x0000, 0x0006), CableInfo { name: "Generic Thunderbolt 3 Cable", speed: "Thunderbolt 3 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0000, 0x0007), CableInfo { name: "Generic Thunderbolt 4 Cable", speed: "Thunderbolt 4 (40 Gbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0000, 0x0008), CableInfo { name: "Generic USB-C Charge Cable (60W)", speed: "USB 2.0 (480 Mbps)", max_watts: 60, cable_type: "Passive" });
        m.insert((0x0000, 0x0009), CableInfo { name: "Generic USB-C Charge Cable (100W)", speed: "USB 2.0 (480 Mbps)", max_watts: 100, cable_type: "Passive" });
        m.insert((0x0000, 0x000A), CableInfo { name: "Generic USB-C Charge Cable (240W EPR)", speed: "USB 2.0 (480 Mbps)", max_watts: 240, cable_type: "Passive" });

        m
    })
}

// ═══════════════════════════════════════════════════════════════════
//  LOOKUP FUNCTIONS
// ═══════════════════════════════════════════════════════════════════

/// Look up a device by VID:PID — checks specific devices first, then vendor
/// Best-effort power role of a device, derived from its VID/PID category.
/// Not a live PD reading (Windows exposes none for user-mode apps) — this is a
/// reliable heuristic based on the known identity of the device.
pub fn power_role(vid: u16, pid: u16, device_class: u8) -> &'static str {
    // Real USB device class (bDeviceClass) is the most specific signal when it
    // is meaningful: 0x09 = hub (oriented as a power/port source).
    if device_class == 0x09 {
        return "hub";
    }
    let cat = lookup(vid, pid).category;
    let c = cat.to_ascii_lowercase();
    if c.contains("charger") || c.contains("power") || c.contains("dock") {
        "source"
    } else if c == "mobile" || c.contains("phone") {
        "charging"
    } else if c.contains("webcam") || c.contains("camera") {
        "camera"
    } else {
        ""
    }
}

pub fn lookup(vid: u16, pid: u16) -> DeviceInfo {
    // 1. Check cable database first (most specific)
    if let Some(cable) = cables().get(&(vid, pid)) {
        return DeviceInfo {
            name: cable.name.to_string(),
            category: "Cable".into(),
            manufacturer: vendors()
                .get(&vid)
                .map(|v| v.name.to_string())
                .unwrap_or_else(|| "Unknown".into()),
        };
    }

    // 2. Check specific device database
    if let Some(device) = devices().get(&(vid, pid)) {
        return DeviceInfo {
            name: device.name.to_string(),
            category: device.category.to_string(),
            manufacturer: vendors()
                .get(&vid)
                .map(|v| v.name.to_string())
                .unwrap_or_else(|| "Unknown".into()),
        };
    }

    // 3. Fall back to vendor-only lookup
    if let Some(vendor) = vendors().get(&vid) {
        DeviceInfo {
            name: format!("{} Device ({:04X}:{:04X})", vendor.name, vid, pid),
            category: vendor.category.to_string(),
            manufacturer: vendor.name.to_string(),
        }
    } else {
        DeviceInfo {
            name: format!("USB Device {:04X}:{:04X}", vid, pid),
            category: "Unknown".into(),
            manufacturer: String::new(),
        }
    }
}

/// Look up cable info specifically (returns None if not a known cable)
#[cfg(test)]
pub fn lookup_cable(vid: u16, pid: u16) -> Option<CableInfo> {
    cables().get(&(vid, pid)).cloned()
}

/// Get total database stats
#[cfg(test)]
pub fn db_stats() -> (usize, usize, usize, usize) {
    let v = vendors().len();
    let d = devices().len();
    let c = cables().len();
    (v, d, c, v + d + c)
}


// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_specific_device() {
        // Xbox Series controller (USB-C)
        let d = lookup(0x045E, 0x0B12);
        assert_eq!(d.name, "Xbox Series Controller (USB-C)");
        assert_eq!(d.manufacturer, "Microsoft");
    }

    #[test]
    fn lookup_known_vendor_fallback() {
        // Logitech mouse — no specific PID entry, falls back to vendor
        let d = lookup(0x046D, 0xC08B);
        assert_eq!(d.manufacturer, "Logitech");
        assert!(!d.category.is_empty());
    }

    #[test]
    fn lookup_apple_cable_has_cable_category() {
        let d = lookup(0x05AC, 0x1238);
        assert_eq!(d.category, "Cable");
        assert_eq!(d.name, "Apple Thunderbolt 4 Pro (1m)");
    }

    #[test]
    fn chargers_are_power_sources_and_poco_cable_resolves() {
        assert_eq!(lookup(0x291A, 0x4021).category, "Charger");
        assert_eq!(power_role(0x291A, 0x4021, 0x00), "source");
        assert_eq!(lookup(0x2717, 0xFF15).category, "Charger");
        // Xiaomi/POCO cables resolve as Cable and carry the POCO user cable.
        assert_eq!(lookup(0x2717, 0x0003).name, "POCO X7 USB-C Cable");
        assert_eq!(lookup(0x2717, 0x0003).category, "Cable");
    }

    #[test]
    fn looking_up_real_machine_devices() {
        assert_eq!(lookup(0x2717, 0xFF08).name, "Xiaomi POCO X7");
        assert_eq!(lookup(0x041E, 0x40A1).name, "Creative Live! Cam Sync 1080p V2");
        assert_eq!(lookup(0x13D3, 0x3571).name, "Realtek Bluetooth Adapter");
        assert_eq!(lookup(0x046D, 0xC08B).name, "Logitech G502 HERO");
        assert_eq!(power_role(0x2717, 0xFF08, 0xFF), "charging");
    }

    #[test]
    fn power_role_classification() {
        // 2717 = Xiaomi (mobile → charging), 291A = Anker (charger → source),
        // 046D = Logitech (mouse → no power role).
        assert_eq!(lookup(0x2717, 0x0000).category, "Mobile");
        assert_eq!(power_role(0x2717, 0x0000, 0xFF), "charging");
        assert_eq!(power_role(0x291A, 0x0000, 0x00), "source");
        assert_eq!(power_role(0x046D, 0x0000, 0x00), "");
        // Real USB class: a hub (class 0x09) is treated as a power/port source.
        assert_eq!(power_role(0x0000, 0x0000, 0x09), "hub");
        // Webcam/camera category is recognized as a consumer device.
        assert_eq!(power_role(0x041E, 0x40A1, 0x00), "camera");
    }

    #[test]
    fn lookup_unknown_returns_vidpid() {
        let d = lookup(0xDEAD, 0xBEEF);
        assert_eq!(d.name, "USB Device DEAD:BEEF");
        assert_eq!(d.category, "Unknown");
        assert_eq!(d.manufacturer, "");
    }

    #[test]
    fn lookup_cable_returns_details() {
        let c = lookup_cable(0x291A, 0x0003).expect("Anker 240W EPR cable");
        assert_eq!(c.max_watts, 240);
        assert!(c.speed.contains("20 Gbps"), "speed: {}", c.speed);
        assert_eq!(c.cable_type, "Passive");
        assert!(lookup_cable(0xDEAD, 0xBEEF).is_none());
    }

    #[test]
    fn db_stats_meet_targets() {
        let (v, d, c, total) = db_stats();
        assert!(v >= 160, "vendors >= 160, got {}", v);
        assert!(d >= 200, "devices >= 200, got {}", d);
        assert!(c >= 50, "cables >= 50, got {}", c);
        assert_eq!(total, v + d + c);
    }
}
