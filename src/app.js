// ─── CableScope — Main Application ────────────────────────────────

import * as api from './api.js';

// ─── State ────────────────────────────────────────────────────────

let lastSnapshot = null;
let autoScanTimer = null;
let settings = {
  refreshInterval: 3,
  notifications: true,
  hideEmptyPorts: true,
  autoScan: true,
  occupancyAlert: 80, // 0 = disabled
};
// Hubs already notified (edge-triggered: alert only when crossing above).
const alertedHubs = new Set();
// Alert history entries: { time, hub, pct, occ, total, direction: 'saturation'|'recovery' }
let alertHistory = [];
const ALERT_HISTORY_MAX = 100;

// ─── Init ─────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', async () => {
  initTabs();
  initScanButton();
  initDeviceListener();
  initDeviceSearch();
  initCrossHighlight();
  initControllerToggle();
  initHubContextMenu();
  document.getElementById('btn-back')?.addEventListener('click', goBack);
  document.getElementById('clear-alert-history')?.addEventListener('click', () => {
    alertHistory = [];
    renderAlertHistory();
    persistAlertHistory();
  });
  document.addEventListener('click', (e) => {
    if (e.button !== 0) return;
    const item = e.target.closest('.alert-history-item[data-hub]');
    if (item) openControllerInPorts(item.dataset.hub);
  });
  await loadSettings();
  document.getElementById('export-json')?.addEventListener('click', exportScanJSON);
  applySettings();

  if (settings.autoScan) {
    runScan();
  }
});

// ─── Device Change Listener ───────────────────────────────────────
let deviceChangeDebounce = null;

function initDeviceListener() {
  api.onDeviceChange((evt) => {
    console.log('[CableScope] Device event:', evt.event_type);
    
    // Debounce rapid events (e.g. composite devices triggering multiple events)
    if (deviceChangeDebounce) clearTimeout(deviceChangeDebounce);
    deviceChangeDebounce = setTimeout(() => {
      runScan();
      
      // Show brief notification
      if (settings.notifications) {
        showNotification(
          evt.event_type === 'connected'
            ? '🔌 Device connecté'
            : '🔌 Device déconnecté'
        );
      }
    }, 300);
  });
}

function showNotification(message) {
  // Create a temporary toast notification
  const toast = document.createElement('div');
  toast.className = 'toast-notification';
  toast.textContent = message;
  document.body.appendChild(toast);
  
  // Animate in
  requestAnimationFrame(() => toast.classList.add('show'));
  
  // Remove after 2.5s
  setTimeout(() => {
    toast.classList.remove('show');
    setTimeout(() => toast.remove(), 300);
  }, 2500);
}

// ─── Tabs ─────────────────────────────────────────────────────────

function initTabs() {
  document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
      // Deactivate all
      document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
      document.querySelectorAll('.tab-content').forEach(tc => tc.classList.remove('active'));

      // Activate clicked
      tab.classList.add('active');
      const tabId = `tab-${tab.dataset.tab}`;
      document.getElementById(tabId)?.classList.add('active');

      // Record plain tab switches (skipped for programmatic navigations).
      if (!suppressNavPush) pushNav(tab.dataset.tab, null);
    });
  });
}

// ─── Scan Button ──────────────────────────────────────────────────

function initScanButton() {
  const btn = document.getElementById('btn-scan');
  btn.addEventListener('click', () => runScan());
}

// ─── Device Search / Filter ─────────────────────────────────────
let deviceFilter = '';

function initDeviceSearch() {
  const input = document.getElementById('device-search');
  if (!input) return;
  input.addEventListener('input', () => {
    deviceFilter = input.value.trim().toLowerCase();
    document.getElementById('device-search-clear').hidden = deviceFilter === '';
    if (lastSnapshot) renderDevices(lastSnapshot);
  });
  document.getElementById('device-search-clear').addEventListener('click', () => {
    input.value = '';
    deviceFilter = '';
    document.getElementById('device-search-clear').hidden = true;
    if (lastSnapshot) renderDevices(lastSnapshot);
    input.focus();
  });
}

// ─── Hub context menu (right-click on usage bar / alert badge) ──
let hubCtxName = null;

function initHubContextMenu() {
  const menu = document.getElementById('hub-context-menu');
  if (!menu) return;

  const close = () => { menu.hidden = true; hubCtxName = null; };

  // Right-click on a bar/badge -> show menu at cursor.
  document.addEventListener('contextmenu', (e) => {
    const target = e.target.closest('.hub-usage-bar[data-hub], .hub-header-alert[data-hub]');
    if (!target) return;
    e.preventDefault();
    hubCtxName = target.dataset.hub;

    menu.hidden = false;
    const pad = 6;
    const x = Math.min(e.clientX, window.innerWidth - menu.offsetWidth - pad);
    const y = Math.min(e.clientY, window.innerHeight - menu.offsetHeight - pad);
    menu.style.left = x + 'px';
    menu.style.top = y + 'px';
  });

  // Actions
  menu.addEventListener('click', async (e) => {
    const item = e.target.closest('.hub-ctx-item[data-action]');
    if (!item || !hubCtxName) return;
    const name = hubCtxName;
    close();
    if (item.dataset.action === 'rescan') {
      await rescanHub(name);
    } else if (item.dataset.action === 'copy') {
      try {
        await navigator.clipboard.writeText(name);
        showNotification('📋 Nom du hub copié');
      } catch (err) {
        console.error('Copy failed:', err);
        showNotification('❌ Copie impossible');
      }
    }
  });

  // Close on click elsewhere, Escape, scroll, or tab change.
  document.addEventListener('click', (e) => {
    if (!e.target.closest('#hub-context-menu')) close();
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') close();
  });
  window.addEventListener('scroll', close, true);
}

// ─── Controller collapse (Ports tab) ───────────────────────────
function initControllerToggle() {
  document.addEventListener('click', (e) => {
    // The controller name navigates to Devices; don't also collapse.
    if (e.target.closest('.controller-name-link')) return;
    const header = e.target.closest('.controller-header[data-controller]');
    if (header) toggleController(header.dataset.controller);
  });
}

// ─── Keyboard shortcuts ────────────────────────────────────────
document.addEventListener('keydown', (e) => {
  // Alt+Left = back button (when there is a history to restore).
  if (e.altKey && e.key === 'ArrowLeft' && navHistory.length > 0) {
    e.preventDefault();
    goBack();
    return;
  }
  // F5 = full scan (don't let WebView reload the page).
  if (e.key === 'F5') {
    e.preventDefault();
    runScan();
    return;
  }
  // Ctrl+F = focus the Devices search box.
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
    e.preventDefault();
    const input = document.getElementById('device-search');
    if (input) {
      switchTabByName('devices');
      input.focus();
      input.select();
    }
  }
  // Escape closes the hub context menu (already handled there too).
  if (e.key === 'Escape') {
    const menu = document.getElementById('hub-context-menu');
    if (menu && !menu.hidden) {
      menu.hidden = true;
      hubCtxName = null;
    }
  }
});

// ─── Navigation history (hub back button) ─────────────────────
// Stack of visited locations: { tab, hub: name|null }.
// hub is null for plain tab switches (no specific hub targeted).
const navHistory = [];
// Set while switchTabByName() triggers tab.click() programmatically, so the
// click handler does not push a second entry (callers push their own).
let suppressNavPush = false;

function pushNav(tab, hub) {
  const last = navHistory[navHistory.length - 1];
  // Avoid consecutive duplicates.
  if (last && last.tab === tab && last.hub === hub) return;
  navHistory.push({ tab, hub });
  updateBackButton();
}

function updateBackButton() {
  const btn = document.getElementById('btn-back');
  if (btn) btn.hidden = navHistory.length === 0;
}

function goBack() {
  const prev = navHistory.pop();
  updateBackButton();
  if (!prev) return;
  if (prev.tab === 'ports') {
    openControllerInPorts(prev.hub, true);
  } else if (prev.hub) {
    openHubInDevices(prev.hub, true);
  } else {
    // Plain tab switch with no hub: just restore the tab.
    switchTabByName(prev.tab);
  }
}

// Switch to the Devices tab and flash the hub group matching a controller name.
function openHubInDevices(name, skipHistory) {
  if (!skipHistory) pushNav('devices', name);
  switchTabByName('devices');

  // If a search filter hides this hub, clear it so the group is visible.
  const section = [...document.querySelectorAll('.hub-group[data-hub]')]
    .find(el => el.dataset.hub === name);
  if (!section && deviceFilter && lastSnapshot) {
    const input = document.getElementById('device-search');
    if (input) input.value = '';
    deviceFilter = '';
    document.getElementById('device-search-clear').hidden = true;
    renderDevices(lastSnapshot);
  }

  const target = [...document.querySelectorAll('.hub-group[data-hub]')]
    .find(el => el.dataset.hub === name);
  if (!target) return;

  if (crossHighlightTimer) clearTimeout(crossHighlightTimer);
  document.querySelectorAll('.flash-highlight').forEach(el => el.classList.remove('flash-highlight'));
  target.classList.add('flash-highlight');
  target.scrollIntoView({ behavior: 'smooth', block: 'start' });

  crossHighlightTimer = setTimeout(() => target.classList.remove('flash-highlight'), 2200);
}

// ─── Cross-highlight: Port <-> Device ──────────────────────────
let crossHighlightTimer = null;

function initCrossHighlight() {
  // Click on a port card -> show Devices tab, flash the matching device row.
  document.addEventListener('click', (e) => {
    // Only left-click triggers navigation (right-click opens the hub menu).
    if (e.button !== 0) return;
    const card = e.target.closest('.port-card[data-port-id]');
    if (card) {
      const portId = card.dataset.portId;
      const hubName = lastSnapshot
        ? (lastSnapshot.devices.find(d => d.port_id === portId) || {}).hub_name || ''
        : '';
      if (hubName) pushNav('devices', hubName);
      switchTabByName('devices');
      flashByDataPortId('device-row', portId);
      return;
    }
    // Click on a device row -> show Ports tab, flash the matching port card.
    const row = e.target.closest('.device-row[data-port-id]');
    if (row && row.dataset.portId) {
      const hubName = lastSnapshot
        ? (lastSnapshot.devices.find(d => d.port_id === row.dataset.portId) || {}).hub_name || ''
        : '';
      if (hubName) pushNav('ports', hubName);
      switchTabByName('ports');
      flashByDataPortId('port-card', row.dataset.portId);
      return;
    }
    // Click on a hub usage bar or alert badge -> open Ports tab on that controller.
    const bar = e.target.closest('.hub-usage-bar[data-hub], .hub-header-alert[data-hub]');
    if (bar) {
      openControllerInPorts(bar.dataset.hub);
      return;
    }
    // Click on a controller name in Ports -> open Devices on that hub.
    const link = e.target.closest('.controller-name-link[data-hub]');
    if (link) {
      openHubInDevices(link.dataset.hub);
    }
  });

  // Double-click on a hub usage bar -> targeted re-scan of that hub.
  document.addEventListener('dblclick', (e) => {
    const bar = e.target.closest('.hub-usage-bar[data-hub]');
    if (bar) rescanHub(bar.dataset.hub);
  });
}

// Targeted re-scan of one hub: merges fresh ports + devices into the snapshot.
async function rescanHub(name) {
  const btn = document.getElementById('btn-scan');
  const wasScanning = btn.classList.contains('scanning');
  btn.classList.add('scanning');
  btn.textContent = '⏳ Hub…';
  try {
    const hub = await api.scanHub(name);
    if (!lastSnapshot) return;
    // Replace this controller's ports and its linked devices.
    lastSnapshot.ports = lastSnapshot.ports.filter(p => p.controller_name !== name).concat(hub.ports);
    const kept = lastSnapshot.devices.filter(d => d.hub_name !== name);
    lastSnapshot.devices = kept.concat(hub.devices);
    renderSnapshot(lastSnapshot);
    checkOccupancyAlerts(lastSnapshot);
    showNotification(`🔄 Hub re-scanné : ${name}`);
  } catch (err) {
    console.error('Hub re-scan failed:', err);
    showNotification('❌ Re-scan du hub échoué');
  } finally {
    if (!wasScanning) {
      btn.classList.remove('scanning');
      btn.textContent = '⟳ Scanner';
    }
  }
}

// Switch to the Ports tab, expand (if collapsed) and flash a controller section.
function openControllerInPorts(name, skipHistory) {
  if (!skipHistory) pushNav('ports', name);
  switchTabByName('ports');

  // If collapsed, expand it and re-render so its cards are visible.
  if (collapsedControllers.has(name)) {
    collapsedControllers.delete(name);
    if (lastSnapshot) renderPorts(lastSnapshot);
  }

  const section = [...document.querySelectorAll('.controller-section[data-controller]')]
    .find(el => el.dataset.controller === name);
  if (!section) return;

  if (crossHighlightTimer) clearTimeout(crossHighlightTimer);
  document.querySelectorAll('.flash-highlight').forEach(el => el.classList.remove('flash-highlight'));
  section.classList.add('flash-highlight');
  section.scrollIntoView({ behavior: 'smooth', block: 'start' });

  crossHighlightTimer = setTimeout(() => section.classList.remove('flash-highlight'), 2200);
}

function switchTabByName(name) {
  const tab = document.querySelector(`.tab[data-tab="${name}"]`);
  if (!tab) return;
  suppressNavPush = true;
  tab.click();
  suppressNavPush = false;
}

// Flash every element of a class whose data-port-id matches (port ids contain
// backslashes, so we compare in JS instead of building a CSS selector).
function flashByDataPortId(className, portId) {
  const els = [...document.querySelectorAll('.' + className + '[data-port-id]')]
    .filter(el => el.dataset.portId === portId);
  if (els.length === 0) return;

  if (crossHighlightTimer) clearTimeout(crossHighlightTimer);
  document.querySelectorAll('.flash-highlight').forEach(el => el.classList.remove('flash-highlight'));

  els.forEach(el => {
    el.classList.add('flash-highlight');
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
  });

  crossHighlightTimer = setTimeout(() => {
    els.forEach(el => el.classList.remove('flash-highlight'));
  }, 2200);
}

function deviceMatchesFilter(dev, q) {
  if (!q) return true;
  const vidHex = hex16(dev.vid);
  const pidHex = hex16(dev.pid);
  const haystack = [
    dev.friendly_name, dev.manufacturer, dev.product, dev.serial,
    dev.hub_name, dev.usb_version, dev.speed, dev.power_role,
    `${vidHex}:${pidHex}`, `${vidHex}${pidHex}`,
  ].filter(Boolean).join(' ').toLowerCase();
  return haystack.includes(q);
}

async function runScan() {
  const btn = document.getElementById('btn-scan');
  btn.classList.add('scanning');
  btn.textContent = '⏳ Scan…';

  try {
    lastSnapshot = await api.scanUsb();
    renderSnapshot(lastSnapshot);
    checkOccupancyAlerts(lastSnapshot);
  } catch (err) {
    console.error('Scan failed:', err);
    renderError(err);
  } finally {
    btn.classList.remove('scanning');
    btn.textContent = '⟳ Scanner';
  }
}

// ─── Render Snapshot ──────────────────────────────────────────────

function renderSnapshot(snap) {
  // Update header
  document.getElementById('scan-time').textContent = snap.scan_time || '—';

  // Update summary
  const connected = snap.ports.filter(p => p.connected).length;
  document.getElementById('total-ports').textContent = snap.ports.length;
  document.getElementById('connected-ports').textContent = connected;
  document.getElementById('total-devices').textContent = snap.devices.length;
  document.getElementById('total-controllers').textContent = snap.controllers.length;

  renderPorts(snap);
  renderDevices(snap);
  renderPower(snap);
}

// ─── Render Ports ─────────────────────────────────────────────────

// Collapsed controllers (persist across renders, keyed by controller name).
const collapsedControllers = new Set();

function toggleController(name) {
  if (collapsedControllers.has(name)) collapsedControllers.delete(name);
  else collapsedControllers.add(name);
  if (lastSnapshot) renderPorts(lastSnapshot);
}

// ─── Render Ports ─────────────────────────────────────────────────

function renderPorts(snap) {
  const container = document.getElementById('ports-list');

  if (snap.ports.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <span class="empty-icon">🔌</span>
        <p>Aucun port USB détecté</p>
      </div>`;
    return;
  }

  // Group by controller
  const byController = {};
  for (const port of snap.ports) {
    const ctrl = port.controller_name || 'Hub inconnu';
    if (!byController[ctrl]) byController[ctrl] = [];
    byController[ctrl].push(port);
  }

  let html = '';

  for (const [ctrlName, ports] of Object.entries(byController)) {
    // Filter empty ports if setting enabled
    const displayPorts = settings.hideEmptyPorts
      ? ports.filter(p => p.connected)
      : ports;

    if (displayPorts.length === 0) continue;

    const occupied = ports.filter(p => p.connected).length;
    const freePorts = ports.length - occupied;
    const isCollapsed = collapsedControllers.has(ctrlName);
    html += `<div class="controller-section ${isCollapsed ? 'collapsed' : ''}" data-controller="${escHtml(ctrlName)}">
      <button class="controller-header" data-controller="${escHtml(ctrlName)}" title="Replier/déplier">
        <span class="controller-chevron">▾</span>
        <span class="controller-name controller-name-link" data-hub="${escHtml(ctrlName)}" title="Voir le hub dans Devices">${escHtml(ctrlName)}</span>
        <span class="controller-ports-badge">${occupied}/${ports.length} ports occupés</span>
      </button>
      <div class="controller-body">`;
    if (displayPorts.length !== ports.length) {
      html += `<div class="controller-free-note">${freePorts} port(s) libre(s)</div>`;
    }

    for (const port of displayPorts) {
      const device = snap.devices.find(d => d.port_id === port.id);
      const statusClass = port.connected ? 'connected' : 'disconnected port-empty';
      const dotClass = port.connected ? 'green' : 'gray';
      const speedBadge = getSpeedBadgeClass(port.speed_value);

      html += `
        <div class="port-card ${statusClass}" data-port-id="${port.id}">
          <div class="port-header">
            <div class="port-name">
              <span class="status-dot ${dotClass}"></span>
              Port ${port.port_number}
            </div>
            <span class="port-speed">${escHtml(port.speed)}</span>
          </div>`;

      if (device) {
        const categoryIcon = getCategoryIcon(device.device_class);
        const categoryClass = getCategoryClass(device.device_class);

        html += `
          <div class="port-device">
            <div class="device-name">${escHtml(device.friendly_name)}</div>
            <div class="device-meta">
              <span><span class="meta-label">VID:PID</span> <span class="meta-value">${hex16(device.vid)}:${hex16(device.pid)}</span></span>
              <span><span class="meta-label">USB</span> <span class="meta-value">${escHtml(device.usb_version)}</span></span>
              <span><span class="meta-label">Classe</span> <span class="meta-value">${escHtml(device.device_class_name)}</span></span>
              ${device.serial ? `<span><span class="meta-label">S/N</span> <span class="meta-value">${escHtml(truncStr(device.serial, 16))}</span></span>` : ''}
            </div>
          </div>`;
      } else {
        html += `
          <div class="port-empty-label">
            <span class="port-empty-dot"></span>
            Port vide — aucun device connecté
          </div>`;
      }

      html += `</div>`;
    }

    html += `</div></div>`;
  }

  container.innerHTML = html;
}

// ─── Render Devices ───────────────────────────────────────────────

// Occupancy color by utilization ratio: <50% green, <80% amber, >=80% red.
function occupancyColor(occupied, total) {
  if (!total) return 'var(--text-muted)';
  const ratio = occupied / total;
  if (ratio >= 0.8) return 'var(--accent-red)';
  if (ratio >= 0.5) return 'var(--accent-amber)';
  return 'var(--accent-green)';
}

// Small progress bar for a hub header (occupied/total ports).
// Clicking it opens the Ports tab scrolled to that hub's controller.
function hubUsageBar(occupied, total, hubName, alert) {
  if (!total) return '';
  const pct = Math.min(100, Math.round((occupied / total) * 100));
  const color = occupancyColor(occupied, total);
  const cls = alert ? 'hub-usage-bar clickable alert' : 'hub-usage-bar clickable';
  return `
    <div class="${cls}" data-hub="${escHtml(hubName)}" title="${occupied}/${total} ports occupés (${pct}%) — clic : voir les ports, double-clic : re-scanner">
      <div class="hub-usage-fill" style="width:${pct}%; background:${color};"></div>
    </div>`;
}


function renderDevices(snap) {
  const container = document.getElementById('devices-list');

  if (snap.devices.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <span class="empty-icon">📱</span>
        <p>Aucun device détecté</p>
      </div>`;
    return;
  }

  // Apply the active search filter (name, VID:PID, hub, serial…)
  const filteredDevices = snap.devices.filter(d => deviceMatchesFilter(d, deviceFilter));

  if (filteredDevices.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <span class="empty-icon">🔍</span>
        <p>Aucun device ne correspond à « ${escHtml(deviceFilter)} »</p>
      </div>`;
    return;
  }

  // Group devices by hub, then order by physical port.
  const groups = new Map();
  for (const dev of filteredDevices) {
    const key = dev.hub_name ? dev.hub_name : 'Non localisé';
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(dev);
  }
  const hubNames = [...groups.keys()].sort((x, y) => {
    if (x === 'Non localisé') return 1;
    if (y === 'Non localisé') return -1;
    return x.localeCompare(y);
  });

  const shownCount = filteredDevices.length;
  const totalCount = snap.devices.length;
  const filterNote = deviceFilter && shownCount !== totalCount
    ? ` (${shownCount} sur ${totalCount})`
    : '';
  let html = `
    <div class="device-summary">
      ${shownCount} appareil(s) · ${groups.size} hub(s)${filterNote}
    </div>
    <div class="occupancy-legend" title="Taux de ports occupés par hub">
      <span class="occupancy-legend-title">Occupation des hubs</span>
      <span class="occupancy-legend-item"><span class="occupancy-swatch" style="background:var(--accent-green);"></span>&lt; 50&nbsp;%</span>
      <span class="occupancy-legend-item"><span class="occupancy-swatch" style="background:var(--accent-amber);"></span>50–79&nbsp;%</span>
      <span class="occupancy-legend-item"><span class="occupancy-swatch" style="background:var(--accent-red);"></span>≥ 80&nbsp;%</span>
    </div>
    <div class="device-list">`;

  // Occupied ports per hub: cross-reference scan ports (connected) with the
  // hub's total port count, so each header shows "3/5 ports occupés".
  const portsByController = new Map();
  for (const p of snap.ports) {
    if (!p.connected) continue;
    portsByController.set(p.controller_name, (portsByController.get(p.controller_name) || 0) + 1);
  }
  const totalByController = new Map();
  for (const c of snap.controllers) {
    totalByController.set(c.name, c.port_count);
  }

  for (const hub of hubNames) {
    const devs = groups.get(hub).sort((x, y) => x.port_number - y.port_number);
    const occupied = portsByController.get(hub) || 0;
    const totalPorts = totalByController.get(hub);
    const portsBadge = totalPorts != null
      ? `<span class="hub-header-ports">${occupied}/${totalPorts} ports occupés</span>`
      : '';
    const alertPct = hubOccupancyPct(hub, snap);
    const isAlert = !!(settings.occupancyAlert && alertPct >= settings.occupancyAlert);
    const usageBar = totalPorts != null ? hubUsageBar(occupied, totalPorts, hub, isAlert) : '';
    const alertBadge = isAlert
      ? `<span class="hub-header-alert clickable" data-hub="${escHtml(hub)}" title="Alerte : ${alertPct}% de ports occupés (seuil ${settings.occupancyAlert}%) — cliquer pour voir les ports">⚠️</span>`
      : '';
    html += `
      <div class="hub-group" data-hub="${escHtml(hub)}">
        <div class="hub-header">
          <span class="hub-header-icon">🔌</span>
          <span class="hub-header-name">${escHtml(hub)}</span>
          ${alertBadge}
          ${portsBadge}
          <span class="hub-header-count">${devs.length} device(s)</span>
        </div>
        ${usageBar}`;

    for (const dev of devs) {
      const categoryIcon = getCategoryIcon(dev.device_class);
      const categoryClass = getCategoryClass(dev.device_class);
      const speedBadge = getSpeedBadgeClass(dev.speed_value);
      const portTag = dev.port_number
        ? `<div class="device-port-tag">Port ${dev.port_number}</div>`
        : '<div class="device-port-tag dim">?</div>';
      html += `
        <div class="device-row" data-port-id="${dev.port_id || ''}">
          ${portTag}
          <div class="device-icon ${categoryClass}">${categoryIcon}</div>
          <div class="device-info">
            <div class="device-name">${escHtml(dev.friendly_name)}</div>
            <div class="device-vid-pid">${hex16(dev.vid)}:${hex16(dev.pid)} · USB ${escHtml(dev.usb_version)}</div>
          </div>
          <span class="device-badge ${speedBadge}">${escHtml(dev.speed)}</span>
        </div>`;
    }
    html += '</div>';
  }

  html += '</div>';
  container.innerHTML = html;
}

// ─── Render Power (derived charging status + UCSI fallback) ───

function powerBadge(role) {
  if (role === 'source') return { cls: 'power-source', icon: '⛽', label: 'Source de courant' };
  if (role === 'charging') return { cls: 'power-charging', icon: '🔋', label: 'En charge' };
  if (role === 'hub') return { cls: 'power-source', icon: '🔌', label: 'Hub' };
  if (role === 'camera') return { cls: 'power-neutral', icon: '📷', label: 'Webcam/Caméra' };
  return { cls: 'power-neutral', icon: '◉', label: 'Consommateur' };
}

function renderPower(snap) {
  const container = document.getElementById('ucsi-status');
  const pdContainer = document.getElementById('pd-contracts');

  // Connected devices with their hub/port provide a reliable, charge-relevant view.
  const connected = snap.devices;
  if (connected.length === 0) {
    container.innerHTML = `
      <div class="status-dot gray"></div>
      <span>Aucun device connecté</span>`;
    pdContainer.innerHTML = '';
    return;
  }

  // Group devices by power role: power sources/hubs first, then chargers, then the rest.
  const groups = [
    { title: 'Chargeurs & sources', icon: '⚡', roles: ['source', 'hub'] },
    { title: 'Appareils en charge', icon: '🔋', roles: ['charging'] },
    { title: 'Autres appareils', icon: '◉', roles: ['camera', '', undefined, null] },
  ];

  const item = (dev) => {
    const b = powerBadge(dev.power_role);
    const where = (dev.hub_name && dev.port_number)
      ? `${escHtml(dev.hub_name)} — Port ${dev.port_number}`
      : 'Emplacement inconnu';
    return `
      <div class="pd-item">
        <div class="pd-row">
          <span class="pd-icon">${b.icon}</span>
          <span class="pd-label">${escHtml(dev.friendly_name)}</span>
          <span class="pd-value power-badge ${b.cls}">${b.label}</span>
        </div>
        <div class="pd-row muted" style="font-size:11px;">
          <span>${hex16(dev.vid)}:${hex16(dev.pid)}</span>
          <span style="margin-left:auto;">${where}</span>
        </div>
      </div>`;
  };

  let rows = '';
  for (const g of groups) {
    const members = connected.filter(d => g.roles.includes(d.power_role));
    if (members.length === 0) continue;
    rows += `
      <div class="pd-group-header">
        <span class="pd-group-icon">${g.icon}</span>
        <span>${g.title}</span>
        <span class="pd-group-count">${members.length}</span>
      </div>`;
    rows += members.map(item).join('');
  }

  container.innerHTML = `
    <div class="status-dot green"></div>
    <span>${connected.length} appareil(s) — état de charge déduit de l'identité</span>`;
  pdContainer.innerHTML = rows;
}

// ─── Export scan (JSON) ─────────────────

function exportScanJSON() {
  const status = document.getElementById('export-status');

  const run = async () => {
    let snap = lastSnapshot;
    if (!snap) {
      status.textContent = 'Aucun scan disponible — scan en cours…';
      snap = await api.scanUsb();
      lastSnapshot = snap;
    }

    const payload = {
      app: 'CableScope',
      version: '1.0.0',
      exported_at: new Date().toISOString(),
      scan_time: snap.scan_time,
      controllers: snap.controllers.map(c => ({
        name: c.name,
        hub_path: c.hub_path,
        port_count: c.port_count,
      })),
      ports: snap.ports.map(p => ({
        hub: p.controller_name,
        port_number: p.port_number,
        connected: p.connected,
        speed: p.speed,
        status: p.status,
      })),
      devices: snap.devices.map(d => ({
        name: d.friendly_name,
        vid_pid: hex16(d.vid) + ':' + hex16(d.pid),
        device_class: '0x' + d.device_class.toString(16).padStart(2, '0'),
        usb_version: d.usb_version,
        speed: d.speed,
        hub: d.hub_name,
        port: d.port_number,
        power_role: d.power_role,
        serial: d.serial,
      })),
    };

    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const aEl = document.createElement('a');
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    aEl.href = url;
    aEl.download = 'cablescope-scan-' + ts + '.json';
    document.body.appendChild(aEl);
    aEl.click();
    document.body.removeChild(aEl);
    URL.revokeObjectURL(url);

    status.textContent = `✅ Exporté : ${snap.controllers.length} controllers, ${snap.ports.length} ports, ${snap.devices.length} devices`;
  };

  run().catch(err => {
    console.error('Export failed:', err);
    status.textContent = '❌ Échec de l’export : ' + err.message;
  });
}

// ─── Alert history (Settings tab) ──────────────────────────────

function recordAlert(hub, direction, occ, total, pct) {
  alertHistory.push({
    time: new Date().toLocaleString('fr-FR', {
      day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit',
    }),
    hub,
    direction,
    occ,
    total,
    pct,
  });
  if (alertHistory.length > ALERT_HISTORY_MAX) alertHistory.shift();
  renderAlertHistory();
  persistAlertHistory();
}

// Persist the alert history inside the settings JSON (survives restarts).
function persistAlertHistory() {
  settings.alertHistory = alertHistory;
  api.saveSettings(JSON.stringify(settings)).catch(err => {
    console.error('Failed to persist alert history:', err);
  });
}

function renderAlertHistory() {
  const list = document.getElementById('alert-history-list');
  if (!list) return;
  if (alertHistory.length === 0) {
    list.innerHTML = 'Aucune alerte enregistrée';
    return;
  }
  list.innerHTML = [...alertHistory].reverse().map(e => {
    const cls = e.direction === 'saturation' ? 'alert-h-sat' : 'alert-h-rec';
    const icon = e.direction === 'saturation' ? '⚠️' : '✅';
    const label = e.direction === 'saturation' ? 'Saturation' : 'Retour normal';
    const pctTxt = e.pct != null ? `${e.pct}%` : '—';
    return `
      <div class="alert-history-item clickable" data-hub="${escHtml(e.hub)}" title="Cliquer pour ouvrir le hub dans Ports">
        <span class="alert-h-badge ${cls}">${icon} ${label}</span>
        <span class="alert-h-hub">${escHtml(e.hub)}</span>
        <span class="alert-h-pct">${e.occ}/${e.total} ports (${pctTxt})</span>
        <span class="alert-h-time">${escHtml(e.time)}</span>
      </div>`;
  }).join('');
}

// ─── Occupancy alert ────────────────────────────────────────────

// Notify when a hub crosses above the occupancy threshold (edge-triggered).
function checkOccupancyAlerts(snap) {
  const threshold = settings.occupancyAlert;
  if (!threshold) {
    alertedHubs.clear();
    return;
  }

  const occupiedByCtrl = new Map();
  for (const p of snap.ports) {
    if (!p.connected) continue;
    occupiedByCtrl.set(p.controller_name, (occupiedByCtrl.get(p.controller_name) || 0) + 1);
  }
  const totalByCtrl = new Map();
  for (const c of snap.controllers) {
    totalByCtrl.set(c.name, c.port_count);
  }

  const currentlyOver = new Set();
  for (const [name, occ] of occupiedByCtrl) {
    const total = totalByCtrl.get(name);
    if (!total) continue;
    const pct = Math.round((occ / total) * 100);
    if (pct >= threshold) currentlyOver.add(name);
  }

  // Hubs that dropped back below the threshold -> notify recovery, then re-arm.
  for (const name of [...alertedHubs]) {
    if (currentlyOver.has(name)) continue;
    const occ = occupiedByCtrl.get(name) || 0;
    const total = totalByCtrl.get(name);
    if (total) {
      const pct = Math.round((occ / total) * 100);
      showNotification(`✅ Retour à la normale : ${name} — ${occ}/${total} ports (${pct}%)`);
      recordAlert(name, 'recovery', occ, total, pct);
    } else {
      showNotification(`✅ Retour à la normale : ${name}`);
      recordAlert(name, 'recovery', occ, total, null);
    }
    alertedHubs.delete(name);
  }

  // New crossings above the threshold -> notify once.
  for (const name of currentlyOver) {
    if (alertedHubs.has(name)) continue;
    const occ = occupiedByCtrl.get(name);
    const total = totalByCtrl.get(name);
    const pct = Math.round((occ / total) * 100);
    showNotification(`⚠️ Hub saturé : ${name} — ${occ}/${total} ports (${pct}%)`);
    recordAlert(name, 'saturation', occ, total, pct);
    alertedHubs.add(name);
  }
}

// Occupancy percentage for a controller name (0 if unknown).
function hubOccupancyPct(name, snap) {
  let occ = 0, total = 0;
  for (const p of snap.ports) {
    if (p.controller_name === name && p.connected) occ++;
  }
  for (const c of snap.controllers) {
    if (c.name === name) total = c.port_count;
  }
  if (!total) return 0;
  return Math.round((occ / total) * 100);
}

// ─── Settings ─────────────────────────────────────────────────────

async function loadSettings() {
  try {
    const json = await api.getSettings();
    const parsed = JSON.parse(json);
    settings = { ...settings, ...parsed };
    if (Array.isArray(parsed.alertHistory)) {
      alertHistory = parsed.alertHistory.slice(0, ALERT_HISTORY_MAX);
    }
  } catch {
    // Use defaults
  }
  renderAlertHistory();

  // Apply to UI
  document.getElementById('refresh-interval').value = settings.refreshInterval;
  document.getElementById('notifications').checked = settings.notifications;
  document.getElementById('hide-empty').checked = settings.hideEmptyPorts;
  document.getElementById('auto-scan').checked = settings.autoScan;
  document.getElementById('occupancy-alert').value = settings.occupancyAlert;

  // Save on change
  for (const id of ['refresh-interval', 'notifications', 'hide-empty', 'auto-scan', 'occupancy-alert']) {
    document.getElementById(id)?.addEventListener('change', saveSettingsFromUI);
  }
}

async function saveSettingsFromUI() {
  settings.refreshInterval = parseInt(document.getElementById('refresh-interval').value) || 3;
  settings.notifications = document.getElementById('notifications').checked;
  settings.hideEmptyPorts = document.getElementById('hide-empty').checked;
  settings.autoScan = document.getElementById('auto-scan').checked;
  settings.occupancyAlert = Math.min(100, Math.max(0, parseInt(document.getElementById('occupancy-alert').value) || 0));

  try {
    await api.saveSettings(JSON.stringify(settings));
  } catch (err) {
    console.error('Failed to save settings:', err);
  }

  applySettings();
}

function applySettings() {
  // Auto-scan timer
  if (autoScanTimer) {
    clearInterval(autoScanTimer);
    autoScanTimer = null;
  }

  if (settings.autoScan && lastSnapshot) {
    autoScanTimer = setInterval(runScan, settings.refreshInterval * 1000);
  }

  // Re-render if we have data (to apply hideEmptyPorts)
  if (lastSnapshot) {
    renderSnapshot(lastSnapshot);
  }
}

// ─── Helpers ──────────────────────────────────────────────────────

function escHtml(str) {
  if (!str) return '';
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function hex16(val) {
  return val.toString(16).toUpperCase().padStart(4, '0');
}

function truncStr(str, max) {
  if (!str) return '';
  return str.length > max ? str.substring(0, max) + '…' : str;
}

function getSpeedBadgeClass(speedValue) {
  switch (speedValue) {
    case 1: return 'badge-usb2';      // Low Speed
    case 2: return 'badge-usb2';      // Full Speed
    case 3: return 'badge-usb2';      // High Speed (USB 2.0)
    case 4: return 'badge-ss';        // SuperSpeed (USB 3.0 Gen 1)
    case 5: return 'badge-ss10';      // SuperSpeed+ (USB 3.1 Gen 2)
    case 6: return 'badge-usb4';      // USB4
    default: return 'badge-usb2';
  }
}

function getCategoryIcon(devClass) {
  switch (devClass) {
    case 0x09: return '🔗';  // Hub
    case 0x08: return '💾';  // Storage
    case 0x03: return '⌨️';  // HID
    case 0x0E: return '📷';  // Video
    case 0x01: return '🎵';  // Audio
    case 0x07: return '🖨️';  // Printer
    case 0x0E: return '📺';  // Video/Camera
    case 0xE0: return '📡';  // Wireless
    case 0x11: return '🏷️';  // Billboard
    case 0x12: return '🔌';  // USB-C Bridge
    case 0xFF: return '⚙️';  // Vendor Specific
    default:   return '📱';
  }
}

function getCategoryClass(devClass) {
  switch (devClass) {
    case 0x09: return 'hub';
    case 0x08: return 'storage';
    case 0x01: case 0x0E: return 'display';
    case 0xE0: return 'network';
    case 0xFF: return 'charger';
    case 0x03: case 0x07: return 'periph';
    default:   return 'unknown';
  }
}

function renderError(err) {
  const container = document.getElementById('ports-list');
  container.innerHTML = `
    <div class="empty-state">
      <span class="empty-icon">⚠️</span>
      <p style="color: var(--accent-red);">Erreur lors du scan</p>
      <p class="muted">${escHtml(String(err))}</p>
    </div>`;
}
