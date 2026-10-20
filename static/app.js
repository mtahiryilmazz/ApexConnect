// ApexConnect Frontend & Remote Desktop Engine

let currentTab = 'quick-connect';
let hostInfo = null;
let activeWs = null;
let canvas, ctx;
let lastFrameTime = performance.now();
let frameCount = 0;
let fps = 0;
let remoteWidth = 1920;
let remoteHeight = 1080;

document.addEventListener('DOMContentLoaded', () => {
  canvas = document.getElementById('remote-canvas');
  if (canvas) {
    ctx = canvas.getContext('2d');
    initCanvasEvents();
  }
  loadHostInfo();
  loadAddressBook();
});

// Tab Switcher
function switchTab(tabId) {
  currentTab = tabId;
  document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
  document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));

  const targetPane = document.getElementById(tabId);
  if (targetPane) targetPane.classList.add('active');

  const activeBtn = Array.from(document.querySelectorAll('.tab-btn')).find(b => b.getAttribute('onclick')?.includes(tabId));
  if (activeBtn) activeBtn.classList.add('active');
}

// Fetch Host & QuickSupport Info
async function loadHostInfo() {
  try {
    const res = await fetch('/api/info');
    if (!res.ok) return;
    hostInfo = await res.json();

    document.getElementById('my-device-id').innerText = formatApexId(hostInfo.device_id);
    document.getElementById('my-device-pin').innerText = hostInfo.pin;
    document.getElementById('host-ip-display').innerText = `${hostInfo.local_ip}:${hostInfo.port}`;
    document.getElementById('mobile-direct-url').innerText = `http://${hostInfo.local_ip}:${hostInfo.port}`;
  } catch (err) {
    console.error('Host info error:', err);
  }
}

function formatApexId(idStr) {
  if (!idStr) return '--- --- ---';
  const clean = idStr.replace(/\s+/g, '');
  if (clean.length === 9) {
    return `${clean.slice(0,3)} ${clean.slice(3,6)} ${clean.slice(6,9)}`;
  }
  return idStr;
}

// Network Discovery Scanner
async function startNetworkScan() {
  const tbody = document.getElementById('scan-results-body');
  const btn = document.getElementById('btn-scan');
  btn.disabled = true;
  btn.innerText = 'Taranıyor...';

  tbody.innerHTML = `
    <tr>
      <td colspan="6" style="text-align: center; color: #60a5fa; padding: 2rem;">
        Yerel ağ cihazları ve aktif ApexConnect istemcileri taranıyor, lütfen bekleyin...
      </td>
    </tr>
  `;

  try {
    const res = await fetch('/api/scan');
    const data = await res.json();

    if (!data.devices || data.devices.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="6" style="text-align: center; color: var(--text-muted); padding: 2rem;">
            Ağda aktif cihaz bulunamadı.
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = data.devices.map(d => `
      <tr>
        <td><span class="badge ${d.is_online ? 'badge-online' : 'badge-offline'}">${d.is_online ? 'Online' : 'Offline'}</span></td>
        <td><strong>${escapeHtml(d.hostname)}</strong></td>
        <td><code>${d.ip}</code></td>
        <td><strong style="color: #60a5fa;">${formatApexId(d.device_id || '-')}</strong></td>
        <td><span class="badge badge-tag">${d.service_type || 'LAN Host'}</span></td>
        <td>
          <button class="btn btn-primary btn-sm" onclick="connectToTarget('${d.ip}', '${d.device_id}')">
            Bağlan
          </button>
          <button class="btn btn-secondary btn-sm" onclick="quickAddToBook('${escapeHtml(d.hostname)}', '${d.ip}', '${d.device_id}')">
            Kaydet
          </button>
        </td>
      </tr>
    `).join('');
  } catch (err) {
    tbody.innerHTML = `
      <tr>
        <td colspan="6" style="text-align: center; color: #ef4444; padding: 2rem;">
          Tarama hatası: ${err.message}
        </td>
      </tr>
    `;
  } finally {
    btn.disabled = false;
    btn.innerText = 'Ağı Tara';
  }
}

// Address Book (Adres Defteri)
async function loadAddressBook() {
  const tbody = document.getElementById('address-book-body');
  try {
    const res = await fetch('/api/address-book');
    const items = await res.json();

    if (!items || items.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="7" style="text-align: center; color: var(--text-muted); padding: 2rem;">
            Henüz kayıtlı cihaz yok. "Yeni Cihaz Ekle" butonunu kullanarak sık bağlandığınız bilgisayarları ekleyebilirsiniz.
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = items.map(item => `
      <tr>
        <td><span class="badge ${item.is_online ? 'badge-online' : 'badge-offline'}">${item.is_online ? 'Online' : 'Offline'}</span></td>
        <td><strong>${escapeHtml(item.name)}</strong></td>
        <td><span class="badge badge-tag">${escapeHtml(item.group || 'Genel')}</span></td>
        <td><code>${item.target}</code></td>
        <td style="color: var(--text-muted); font-size: 0.8rem;">${escapeHtml(item.notes || '-')}</td>
        <td style="font-size: 0.78rem; color: var(--text-muted);">${item.last_connected || 'Hiç'}</td>
        <td>
          <button class="btn btn-primary btn-sm" onclick="connectToAddressBook('${item.target}', '${item.saved_pin || ''}')">
            Bağlan
          </button>
          <button class="btn btn-danger btn-sm" onclick="deleteAddressBookItem('${item.id}')">
            Sil
          </button>
        </td>
      </tr>
    `).join('');
  } catch (err) {
    console.error('Address book error:', err);
  }
}

async function quickAddToBook(hostname, ip, device_id) {
  const name = prompt('Cihaz için bir takma ad girin:', hostname || ip);
  if (!name) return;
  const group = prompt('Departman / Grup adı (Örn: Muhasebe, Sistem, Şube-1):', 'Ofis') || 'Genel';

  await fetch('/api/address-book', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      id: Date.now().toString(),
      name: name,
      group: group,
      target: device_id || ip,
      notes: `IP: ${ip}`,
      saved_pin: ''
    })
  });
  loadAddressBook();
  switchTab('address-book');
}

function showAddDeviceModal() {
  const name = prompt('Cihaz Adı (Örn: Muhasebe-PC1):');
  if (!name) return;
  const target = prompt('Apex ID veya Yerel IP Adresi:');
  if (!target) return;
  const group = prompt('Departman/Grup (Örn: Muhasebe, Satış, Sunucu):', 'Ofis') || 'Genel';
  const pin = prompt('Sabit Bağlantı Parolası (Opsiyonel):', '');

  fetch('/api/address-book', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      id: Date.now().toString(),
      name,
      group,
      target,
      notes: '',
      saved_pin: pin
    })
  }).then(() => loadAddressBook());
}

async function deleteAddressBookItem(id) {
  if (!confirm('Bu cihazı adres defterinden silmek istediğinize emin misiniz?')) return;
  await fetch(`/api/address-book/${id}`, { method: 'DELETE' });
  loadAddressBook();
}

// Remote Connection Manager
function connectToAddressBook(target, pin) {
  document.getElementById('target-input').value = target;
  document.getElementById('pin-input').value = pin;
  initiateConnection();
}

function connectToTarget(ip, deviceId) {
  document.getElementById('target-input').value = deviceId || ip;
  switchTab('quick-connect');
}

function initiateConnection() {
  const target = document.getElementById('target-input').value.trim();
  const pin = document.getElementById('pin-input').value.trim();

  if (!target) {
    alert('Lütfen bir hedef Apex ID veya IP adresi girin.');
    return;
  }

  startRemoteSession(target, pin);
}

function startRemoteSession(target, pin) {
  if (activeWs) {
    activeWs.close();
  }

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws/client?target=${encodeURIComponent(target)}&pin=${encodeURIComponent(pin)}`;

  document.getElementById('nav-viewer-btn').style.display = 'inline-flex';
  switchTab('remote-viewer');
  document.getElementById('viewer-target-title').innerText = `Bağlanılıyor: ${target}`;

  activeWs = new WebSocket(wsUrl);
  activeWs.binaryType = 'arraybuffer';

  activeWs.onopen = () => {
    document.getElementById('viewer-target-title').innerText = `Bağlı: ${target}`;
  };

  activeWs.onmessage = (event) => {
    if (typeof event.data === 'string') {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'screen_size') {
          remoteWidth = msg.width;
          remoteHeight = msg.height;
          canvas.width = remoteWidth;
          canvas.height = remoteHeight;
        }
      } catch (e) {}
    } else {
      // Binary JPEG frame
      const blob = new Blob([event.data], { type: 'image/jpeg' });
      createImageBitmap(blob).then(bitmap => {
        if (canvas.width !== bitmap.width || canvas.height !== bitmap.height) {
          canvas.width = bitmap.width;
          canvas.height = bitmap.height;
          remoteWidth = bitmap.width;
          remoteHeight = bitmap.height;
        }
        ctx.drawImage(bitmap, 0, 0);
        bitmap.close();

        frameCount++;
        const now = performance.now();
        if (now - lastFrameTime >= 1000) {
          fps = Math.round((frameCount * 1000) / (now - lastFrameTime));
          frameCount = 0;
          lastFrameTime = now;
          document.getElementById('session-stats').innerText = `${fps} FPS | Canlı`;
        }
      });
    }
  };

  activeWs.onclose = () => {
    document.getElementById('viewer-target-title').innerText = 'Bağlantı Kapandı';
    document.getElementById('session-stats').innerText = '0 FPS | Offline';
  };

  activeWs.onerror = (err) => {
    console.error('WS Error:', err);
    alert('Uzak oturuma bağlanırken hata oluştu. PIN veya adres kontrolü yapın.');
  };
}

function closeSession() {
  if (activeWs) {
    activeWs.close();
    activeWs = null;
  }
  document.getElementById('nav-viewer-btn').style.display = 'none';
  switchTab('quick-connect');
}

// Canvas Input & Mouse Events
function initCanvasEvents() {
  if (!canvas) return;

  function getNormalizedCoords(e) {
    const rect = canvas.getBoundingClientRect();
    const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const y = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
    return { x, y };
  }

  canvas.addEventListener('mousemove', (e) => {
    if (!activeWs || activeWs.readyState !== WebSocket.OPEN) return;
    const { x, y } = getNormalizedCoords(e);
    activeWs.send(JSON.stringify({ type: 'mouse_move', x, y }));
  });

  canvas.addEventListener('mousedown', (e) => {
    if (!activeWs || activeWs.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    canvas.focus();
    const { x, y } = getNormalizedCoords(e);
    activeWs.send(JSON.stringify({ type: 'mouse_down', button: e.button, x, y }));
  });

  canvas.addEventListener('mouseup', (e) => {
    if (!activeWs || activeWs.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    const { x, y } = getNormalizedCoords(e);
    activeWs.send(JSON.stringify({ type: 'mouse_up', button: e.button, x, y }));
  });

  canvas.addEventListener('contextmenu', (e) => e.preventDefault());

  canvas.addEventListener('wheel', (e) => {
    if (!activeWs || activeWs.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    const delta = e.deltaY < 0 ? 120 : -120;
    activeWs.send(JSON.stringify({ type: 'mouse_wheel', delta }));
  }, { passive: false });

  // Keyboard events
  window.addEventListener('keydown', (e) => {
    if (currentTab !== 'remote-viewer' || !activeWs || activeWs.readyState !== WebSocket.OPEN) return;
    if (['F5', 'F12'].includes(e.key)) return;
    e.preventDefault();
    activeWs.send(JSON.stringify({ type: 'key_down', key: e.key, code: e.code }));
  });

  window.addEventListener('keyup', (e) => {
    if (currentTab !== 'remote-viewer' || !activeWs || activeWs.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    activeWs.send(JSON.stringify({ type: 'key_up', key: e.key, code: e.code }));
  });

  // Mobile Touch Emulation
  let touchStartTime = 0;
  canvas.addEventListener('touchstart', (e) => {
    touchStartTime = performance.now();
    if (e.touches.length === 1) {
      const touch = e.touches[0];
      const { x, y } = getNormalizedCoords(touch);
      if (activeWs && activeWs.readyState === WebSocket.OPEN) {
        activeWs.send(JSON.stringify({ type: 'mouse_move', x, y }));
      }
    }
  }, { passive: true });

  canvas.addEventListener('touchmove', (e) => {
    if (e.touches.length === 1) {
      const touch = e.touches[0];
      const { x, y } = getNormalizedCoords(touch);
      if (activeWs && activeWs.readyState === WebSocket.OPEN) {
        activeWs.send(JSON.stringify({ type: 'mouse_move', x, y }));
      }
    }
  }, { passive: true });

  canvas.addEventListener('touchend', (e) => {
    const duration = performance.now() - touchStartTime;
    if (activeWs && activeWs.readyState === WebSocket.OPEN) {
      // Short tap = left click, long tap = right click
      const btn = duration > 500 ? 2 : 0;
      activeWs.send(JSON.stringify({ type: 'mouse_click', button: btn }));
    }
  }, { passive: true });
}

function sendCtrlAltDel() {
  if (activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'hotkey', action: 'ctrl_alt_del' }));
  }
}

function sendLeftClick() {
  if (activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'mouse_click', button: 0 }));
  }
}

function sendRightClick() {
  if (activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'mouse_click', button: 2 }));
  }
}

function toggleVirtualKeyboard() {
  const text = prompt('Uzak bilgisayara gönderilecek metni yazın:');
  if (text && activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'type_text', text }));
  }
}

function toggleFullscreen() {
  const container = document.getElementById('viewer-container');
  if (!document.fullscreenElement) {
    container.requestFullscreen().catch(err => alert(err.message));
  } else {
    document.exitFullscreen();
  }
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
