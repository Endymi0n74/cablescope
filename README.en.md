# ⚡ CableScope — USB-C Cable & Port Inspector

[![CI](https://github.com/Endymi0n74/cablescope/actions/workflows/release.yml/badge.svg)](https://github.com/Endymi0n74/cablescope/actions/workflows/release.yml)

[🇫🇷 Français](README.md) · **🇬🇧 English**

Inspect your USB-C ports and the connected devices at a glance: hubs, ports,
devices, physical topology and inferred charge state.

> **Version 1.0.0** — developed and tested on **Windows**. The **macOS** and
> **Linux** builds are provided but **untested** (USB scanning relies on Win32 APIs:
> SetupAPI, cfgmgr32, DeviceIoControl — the macOS/Linux adapters will have to be
> validated on real hardware).

---

## ✨ Features

- **Full scan** — enumerates all USB controllers/hubs, their ports and the connected devices
- **Physical topology** — each device is linked to its hub and its actual port
- **Ports tab** — occupied/free ports, speed (Low/Full/High/SuperSpeed), controller failover
- **Devices tab** — devices grouped by hub/port, instant search (name, VID:PID, hub, serial…)
- **Power tab** — badge per device: charger/source, mobile charging, hub, webcam, power consumer
- **Occupancy alerts** — configurable threshold (% of ports occupied per hub), saturation
  notification + return to normal, persistent history in Settings
- **Navigation** — cross-highlight Port ↔ Device, targeted re-scan of a hub
  (double-click on its bar), context menu (right-click), shortcuts (F5 = scan, Ctrl+F = search)
- **VID/PID database** — 240+ known devices (POCO/Xiaomi, Logitech, Creative, Realtek, Anker, Apple…) and 60+ cables/chargers
- **JSON export** — full snapshot (controllers, ports, devices, power_role) from Settings
- **Real-time detection** — plugging/unplugging a cable triggers a re-scan + notification

## 🖥️ Prerequisites (development)

- [Node.js](https://nodejs.org) 18+
- [Rust](https://rustup.rs) stable
- Tauri v2 (CLI included in `node_modules`)

## 🚀 Development

```bash
npm install
npm run tauri:dev        # lance l'app en mode dev avec hot-reload
```

## 📦 Release build

```bash
npm run tauri:build      # produit l'installeur (Windows : NSIS .exe)
```

The artifact is located in `src-tauri/target/release/bundle/`.

> **macOS / Linux**: `tauri build` on these platforms generates a
> `.dmg`/`.app` and a `.deb`/`.AppImage` respectively. **Untested** — the USB scanning logic
> is written for Windows and uses Windows system calls only
> (`SetupDiXxx`, `CM_Get_Parent`, `DeviceIoControl`). On macOS/Linux the app
> launches but the scan will return an empty state until specific
> adapters are implemented.

## 🤖 CI — Multi-platform builds

A **GitHub Actions** workflow (`.github/workflows/release.yml`) builds and attaches the artifacts
to a GitHub release **on every `v*` tag** (e.g. `v1.0.0`):

- **Windows** (tested): NSIS `.exe` installer + `.msi`
- **macOS** (untested): `.dmg` / `.app`
- **Linux** (untested): `.deb` / `.AppImage`

The workflow can also be triggered manually from the Actions tab (`workflow_dispatch`).

## 🧪 Tests

```bash
cd src-tauri && cargo test    # 15 tests : parsing, base VID/PID, filtrage scan_hub
```

## ⌨️ Shortcuts

| Key | Action |
|---|---|
| `F5` | Full scan |
| `Ctrl+F` | Search in Devices |
| `Esc` | Close the context menu |

## 📁 Structure

```
cablescope/
├── index.html            # UI (onglets Ports / Devices / Power / Settings)
├── src/
│   ├── app.js            # logique frontend (rendu, navigation, alertes)
│   └── api.js            # pont vers les commandes Tauri
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs        # commandes Tauri (scan_usb, scan_hub, settings…)
│   │   └── usb/mod.rs    # scan Win32 : hubs, ports, devices, topologie
│   └── tauri.conf.json   # config de build et bundling
```

## ⚠️ Known limitations

- **Real Power Delivery (PD)**: Windows does not expose the negotiated voltage/current through a
  stable public API (UCSI kernel interface) — the Power tab displays an *inferred*
  charge state based on the device identity (charger → source, mobile → charging…).
- **Passive cables/chargers**: without an active chip, they do not enumerate as USB devices —
  their VID/PID entries are ready but only appear if a compatible
  device shows up.
- **macOS/Linux untested**: see the Build section.
