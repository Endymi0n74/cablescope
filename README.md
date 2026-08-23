# ⚡ CableScope — USB-C Cable & Port Inspector

[![CI](https://github.com/Endymi0n74/cablescope/actions/workflows/release.yml/badge.svg)](https://github.com/Endymi0n74/cablescope/actions/workflows/release.yml)

Inspectez vos ports USB-C et les appareils branchés en un coup d'œil : hubs, ports,
devices, topologie physique et état de charge déduit.

> **Version 1.0.0** — développée et testée sur **Windows**. Les builds **macOS** et
> **Linux** sont fournis mais **non testés** (le scan USB repose sur des API Win32 :
> SetupAPI, cfgmgr32, DeviceIoControl — les adaptateurs macOS/Linux devront être
> validés sur du vrai matériel).

---

## ✨ Fonctionnalités

- **Scan complet** — énumère tous les contrôleurs/hubs USB, leurs ports et les devices connectés
- **Topologie physique** — chaque device est relié à son hub et à son port réel
- **Onglet Ports** — ports occupés/libres, vitesse (Low/Full/High/SuperSpeed), repli des contrôleurs
- **Onglet Devices** — devices groupés par hub/port, recherche instantanée (nom, VID:PID, hub, serial…)
- **Onglet Power** — badge par appareil : chargeur/source, mobile en charge, hub, webcam, consommateur
- **Alertes d'occupation** — seuil configurable (% de ports occupés par hub), notification
  saturation + retour à la normale, historique persistant dans Settings
- **Navigation** — cross-highlight Port ↔ Device, re-scan ciblé d'un hub
  (double-clic sur sa barre), menu contextuel (clic droit), raccourcis (F5 = scan, Ctrl+F = recherche)
- **Base VID/PID** — 240+ appareils connus (POCO/Xiaomi, Logitech, Creative, Realtek, Anker, Apple…) et 60+ câbles/chargeurs
- **Export JSON** — snapshot complet (controllers, ports, devices, power_role) depuis Settings
- **Détection temps réel** — brancher/débrancher un câble déclenche un re-scan + notification

## 🖥️ Prérequis (développement)

- [Node.js](https://nodejs.org) 18+
- [Rust](https://rustup.rs) stable
- Tauri v2 (CLI inclus dans `node_modules`)

## 🚀 Développement

```bash
npm install
npm run tauri:dev        # lance l'app en mode dev avec hot-reload
```

## 📦 Build de release

```bash
npm run tauri:build      # produit l'installeur (Windows : NSIS .exe)
```

L'artefact se trouve dans `src-tauri/target/release/bundle/`.

> **macOS / Linux** : `tauri build` sur ces plateformes génère respectivement un
> `.dmg`/`.app` et un `.deb`/`.AppImage`. **Non testés** — la logique de scan USB
> est écrite pour Windows et utilise des appels système Windows uniquement
> (`SetupDiXxx`, `CM_Get_Parent`, `DeviceIoControl`). Sur macOS/Linux l'app se
> lance mais le scan retournera un état vide tant que des adaptateurs
> spécifiques ne sont pas implémentés.

## 🤖 CI — Builds multi-plateforme

Un workflow **GitHub Actions** (`.github/workflows/release.yml`) build et attache les artefacts
à une release GitHub **à chaque tag `v*`** (ex. `v1.0.0`) :

- **Windows** (testé) : installeur NSIS `.exe` + `.msi`
- **macOS** (non testé) : `.dmg` / `.app`
- **Linux** (non testé) : `.deb` / `.AppImage`

Le workflow est aussi déclenchable manuellement depuis l'onglet Actions (`workflow_dispatch`).

## 🧪 Tests

```bash
cd src-tauri && cargo test    # 15 tests : parsing, base VID/PID, filtrage scan_hub
```

## ⌨️ Raccourcis

| Touche | Action |
|---|---|
| `F5` | Scan complet |
| `Ctrl+F` | Recherche dans Devices |
| `Échap` | Fermer le menu contextuel |

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

## ⚠️ Limites connues

- **Power Delivery (PD) réel** : Windows n'expose pas le voltage/courant négocié via une API
  publique stable (interface noyau UCSI) — l'onglet Power affiche un état de charge *déduit*
  de l'identité du device (chargeur → source, mobile → en charge…).
- **Câbles/chargeurs passifs** : sans puce active, ils ne s'énumèrent pas comme devices
  USB — leurs entrées VID/PID sont prêtes mais ne s'affichent que si un appareil
  compatible se présente.
- **macOS/Linux non testés** : voir la section Build.
