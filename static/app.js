// ApexConnect Frontend & Remote Desktop Engine

let currentTab = 'quick-connect';
let hostInfo = null;
let activeWs = null;
let canvas, ctx;
let lastFrameTime = performance.now();
let frameCount = 0;
let fps = 0;
let latency = 0;
let bytesReceived = 0;
let bandwidth = 0;
let pingInterval = null;
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
  loadFileList();
  initDropZone();
});

// WinUI 3 Navigation Tab Switcher
function switchNavTab(tabId) {
  currentTab = tabId;
  document.querySelectorAll('.fluent-view').forEach(el => el.classList.remove('active'));
  document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));

  const targetView = document.getElementById(tabId);
  if (targetView) targetView.classList.add('active');

  const activeBtn = document.querySelector(`.nav-item[data-tab="${tabId}"]`);
  if (activeBtn) activeBtn.classList.add('active');

  if (tabId === 'files-tab') {
    loadFileList();
  }
}

// Backward compatibility
function switchTab(tabId) {
  switchNavTab(tabId);
}

// Fetch Host & QuickSupport Info
async function loadHostInfo() {
  try {
    const res = await fetch('/api/info');
    if (!res.ok) return;
    hostInfo = await res.json();

    const idEl = document.getElementById('my-device-id');
    if (idEl) idEl.innerText = formatApexId(hostInfo.device_id);

    const pinEl = document.getElementById('my-device-pin');
    if (pinEl) pinEl.innerText = hostInfo.pin;

    const ipDisplay = `${hostInfo.local_ip}:${hostInfo.port}`;
    const titlebarIp = document.getElementById('titlebar-ip-status');
    if (titlebarIp) titlebarIp.innerText = `Hazır: ${ipDisplay}`;

    const hostIpDisp = document.getElementById('host-ip-display');
    if (hostIpDisp) hostIpDisp.innerText = ipDisplay;

    const directUrl = document.getElementById('mobile-direct-url');
    if (directUrl) directUrl.innerText = `http://${hostInfo.local_ip}:${hostInfo.port}`;
  } catch (err) {
    console.error('Host info error:', err);
  }
}

async function regeneratePinPrompt() {
  try {
    const res = await fetch('/api/pin/refresh');
    if (res.ok) {
      const data = await res.json();
      const el = document.getElementById('my-device-pin');
      if (el) el.innerText = data.new_pin;
    }
  } catch (e) {
    console.error('PIN refresh error:', e);
  }
}

function copyDeviceId() {
  if (hostInfo && hostInfo.device_id) {
    navigator.clipboard.writeText(hostInfo.device_id.replace(/\s+/g, ''));
    alert(`Apex Cihaz ID panoya kopyalandı: ${formatApexId(hostInfo.device_id)}`);
  }
}

function simulateWindowControl(action) {
  if (action === 'min') {
    alert('Uygulama arka planda ve sistem tepsisinde çalışmaya devam ediyor.');
  } else if (action === 'max') {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(() => {});
    } else {
      document.exitFullscreen().catch(() => {});
    }
  } else if (action === 'close') {
    if (confirm('ApexConnect penceresini kapatmak istiyor musunuz?')) {
      window.close();
    }
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
          <button class="btn btn-primary btn-sm" onclick="connectToTarget('${d.ip}', '${d.device_id || ''}', ${d.port || 8080})">
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

function connectToTarget(ip, deviceId, port) {
  const targetStr = (deviceId && deviceId !== '-') ? formatApexId(deviceId) : ((port && port !== 8080) ? `${ip}:${port}` : `${ip}:${port || 8080}`);
  document.getElementById('target-input').value = targetStr;
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

async function startRemoteSession(target, pin) {
  if (activeWs) {
    activeWs.close();
  }
  if (pingInterval) {
    clearInterval(pingInterval);
  }

  const cleanTarget = target.trim().replace(/\s+/g, '');
  let targetHost = cleanTarget;

  // If target doesn't specify an explicit port or is an ID:
  if (!cleanTarget.includes(':')) {
    try {
      const res = await fetch(`/api/resolve?target=${encodeURIComponent(cleanTarget)}`);
      if (res.ok) {
        const data = await res.json();
        if (data && data.found) {
          targetHost = `${data.ip}:${data.port}`;
          console.log(`Resolved target ${cleanTarget} -> ${targetHost}`);
        } else if (/^\d+\.\d+\.\d+\.\d+$/.test(cleanTarget)) {
          targetHost = `${cleanTarget}:8080`;
        }
      }
    } catch (e) {
      console.warn('Target resolution fallback:', e);
    }
  }

  if (/^\d{6,12}$/.test(targetHost)) {
    alert(`Apex ID (${target}) yerel ağda çözümlenemedi.\nLütfen hedef bilgisayarın açık olduğundan veya IP:PORT (Örn: 192.168.1.50:8080) şeklinde doğrudan bağlanmayı deneyin.`);
    return;
  }

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${targetHost}/ws/client?pin=${encodeURIComponent(pin)}`;
  console.log('Connecting remote WebSocket to:', wsUrl);

  document.getElementById('nav-viewer-btn').style.display = 'inline-flex';
  switchTab('remote-viewer');
  document.getElementById('viewer-target-title').innerText = `Bağlanılıyor: ${targetHost}`;

  activeWs = new WebSocket(wsUrl);
  activeWs.binaryType = 'arraybuffer';

  activeWs.onopen = () => {
    document.getElementById('viewer-target-title').innerText = `Bağlı: ${targetHost}`;
    
    // Heartbeat & Ping RTT loop
    pingInterval = setInterval(() => {
      if (activeWs && activeWs.readyState === WebSocket.OPEN) {
        activeWs.send(JSON.stringify({ type: 'ping', timestamp: Date.now() }));
      }
    }, 2000);
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
        } else if (msg.type === 'pong') {
          latency = Date.now() - msg.timestamp;
          updateStatsDisplay();
        } else if (msg.type === 'file_progress') {
          updateUploadProgress(msg.id, msg.received);
        } else if (msg.type === 'file_complete') {
          finishUploadProgress(msg.name);
        }
      } catch (e) {}
    } else {
      // Binary JPEG frame
      const byteLength = event.data.byteLength || 0;
      bytesReceived += byteLength;

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
          bandwidth = Math.round(bytesReceived / 1024);
          frameCount = 0;
          bytesReceived = 0;
          lastFrameTime = now;
          updateStatsDisplay();
        }
      });
    }
  };

  activeWs.onclose = () => {
    document.getElementById('viewer-target-title').innerText = 'Bağlantı Kapandı';
    document.getElementById('session-stats').innerText = '0 FPS | Offline';
    if (pingInterval) clearInterval(pingInterval);
  };

  activeWs.onerror = (err) => {
    console.error('WS Error:', err);
    document.getElementById('viewer-target-title').innerText = `Bağlantı Hatası: ${targetHost}`;
    alert(`Uzak cihaza (${targetHost}) bağlanırken hata oluştu.\n- Hedef cihazın açık ve erişilebilir olduğunu\n- Girilen PIN kodunun doğruluğunu kontrol edin.`);
  };
}

function updateStatsDisplay() {
  const statsEl = document.getElementById('session-stats');
  if (statsEl) {
    statsEl.innerText = `${fps} FPS | ${latency} ms | ${bandwidth} KB/s`;
  }
}

function closeSession() {
  if (activeWs) {
    activeWs.close();
    activeWs = null;
  }
  if (pingInterval) {
    clearInterval(pingInterval);
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
      const btn = duration > 500 ? 2 : 0;
      activeWs.send(JSON.stringify({ type: 'mouse_click', button: btn }));
    }
  }, { passive: true });
}

// Session Controls
function changeQuality(val) {
  if (activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'set_quality', quality: parseInt(val) }));
  }
}

function changeMonitor(mode) {
  if (activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'set_monitor', monitor: mode }));
  }
}

function sendClipboardPrompt() {
  const text = prompt('Uzak bilgisayarın panosuna (clipboard) yapıştırılacak metin:');
  if (text && activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify({ type: 'clipboard', text }));
    alert('Metin uzak panoya başarıyla aktarıldı.');
  }
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

// File Transfer Engine
async function loadFileList() {
  const tbody = document.getElementById('files-table-body');
  try {
    const res = await fetch('/api/files');
    const files = await res.json();

    if (!files || files.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="4" style="text-align: center; color: var(--text-muted); padding: 2rem;">
            Transfer klasöründe henüz dosya yok. Yukarıdaki alana dosya sürükleyip yükleyebilirsiniz.
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = files.map(f => `
      <tr>
        <td><strong>📄 ${escapeHtml(f.name)}</strong></td>
        <td><code>${formatFileSize(f.size)}</code></td>
        <td style="color: var(--text-muted); font-size: 0.8rem;">${f.modified || '-'}</td>
        <td>
          <a class="btn btn-primary btn-sm" href="/api/files/download/${encodeURIComponent(f.name)}" download>
            ⬇️ İndir
          </a>
        </td>
      </tr>
    `).join('');
  } catch (err) {
    tbody.innerHTML = `<tr><td colspan="4" style="color: #ef4444; text-align: center;">Dosya listesi alınamadı: ${err.message}</td></tr>`;
  }
}

async function openRemoteFolder() {
  try {
    await fetch('/api/files/open-folder');
  } catch (e) {
    console.error(e);
  }
}

function formatFileSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function initDropZone() {
  const dropZone = document.getElementById('file-drop-zone');
  if (!dropZone) return;

  ['dragenter', 'dragover'].forEach(eventName => {
    dropZone.addEventListener(eventName, (e) => {
      e.preventDefault();
      dropZone.classList.add('dragover');
    }, false);
  });

  ['dragleave', 'drop'].forEach(eventName => {
    dropZone.addEventListener(eventName, (e) => {
      e.preventDefault();
      dropZone.classList.remove('dragover');
    }, false);
  });

  dropZone.addEventListener('drop', (e) => {
    const dt = e.dataTransfer;
    const files = dt.files;
    if (files.length > 0) {
      uploadFile(files[0]);
    }
  });
}

function handleFileSelect(event) {
  const files = event.target.files;
  if (files.length > 0) {
    uploadFile(files[0]);
  }
}

let activeUploadFile = null;

function uploadFile(file) {
  if (!activeWs || activeWs.readyState !== WebSocket.OPEN) {
    alert('Dosya yüklemek için önce aktif bir canlı oturum başlatmalısınız veya sunucuya bağlı olmalısınız.');
    return;
  }

  activeUploadFile = file;
  const fileId = Date.now().toString();
  const chunkSize = 64 * 1024; // 64 KB chunks
  let offset = 0;

  document.getElementById('upload-progress-container').style.display = 'block';
  document.getElementById('upload-filename').innerText = `Yükleniyor: ${file.name} (${formatFileSize(file.size)})`;
  document.getElementById('upload-percentage').innerText = '0%';
  document.getElementById('upload-progress-bar').style.width = '0%';

  // Send start packet
  activeWs.send(JSON.stringify({
    type: 'file_start',
    id: fileId,
    name: file.name,
    size: file.size
  }));

  const reader = new FileReader();

  function readNextChunk() {
    if (offset < file.size) {
      const slice = file.slice(offset, offset + chunkSize);
      reader.onload = (e) => {
        const arrayBuffer = e.target.result;
        const bytes = new Uint8Array(arrayBuffer);
        let binary = '';
        for (let i = 0; i < bytes.byteLength; i++) {
          binary += String.fromCharCode(bytes[i]);
        }
        const base64 = btoa(binary);

        activeWs.send(JSON.stringify({
          type: 'file_chunk',
          id: fileId,
          offset: offset,
          data: base64
        }));

        offset += bytes.byteLength;
        const pct = Math.round((offset / file.size) * 100);
        document.getElementById('upload-percentage').innerText = `${pct}%`;
        document.getElementById('upload-progress-bar').style.width = `${pct}%`;

        setTimeout(readNextChunk, 10);
      };
      reader.readAsArrayBuffer(slice);
    } else {
      activeWs.send(JSON.stringify({
        type: 'file_end',
        id: fileId
      }));
    }
  }

  readNextChunk();
}

function updateUploadProgress(fileId, receivedBytes) {
  if (activeUploadFile && activeUploadFile.size > 0) {
    const pct = Math.round((receivedBytes / activeUploadFile.size) * 100);
    document.getElementById('upload-percentage').innerText = `${pct}%`;
    document.getElementById('upload-progress-bar').style.width = `${pct}%`;
  }
}

function finishUploadProgress(fileName) {
  document.getElementById('upload-filename').innerText = `Yükleme Tamamlandı: ${fileName}`;
  document.getElementById('upload-percentage').innerText = '100%';
  document.getElementById('upload-progress-bar').style.width = '100%';
  setTimeout(() => {
    document.getElementById('upload-progress-container').style.display = 'none';
    loadFileList();
  }, 2000);
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// ==================== THEME MANAGEMENT (LIGHT / DARK) ====================
function initTheme() {
  const savedTheme = localStorage.getItem('apex_theme') || 'dark';
  applyTheme(savedTheme);
}

function applyTheme(theme) {
  const html = document.documentElement;
  if (theme === 'light') {
    html.classList.remove('dark');
    html.classList.add('light');
    localStorage.setItem('apex_theme', 'light');
  } else {
    html.classList.remove('light');
    html.classList.add('dark');
    localStorage.setItem('apex_theme', 'dark');
  }
}

function toggleTheme() {
  const isLight = document.documentElement.classList.contains('light');
  applyTheme(isLight ? 'dark' : 'light');
}

// Auto-run theme immediately
initTheme();

// ==================== FRAMELESS WINDOW CONTROLS ====================
async function windowMinimize() {
  try {
    await fetch('/api/window/minimize', { method: 'POST' });
  } catch (e) {
    console.log('Minimize fallback', e);
  }
}

async function windowMaximize() {
  try {
    await fetch('/api/window/maximize', { method: 'POST' });
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(() => {});
    } else {
      document.exitFullscreen().catch(() => {});
    }
  } catch (e) {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(() => {});
    } else {
      document.exitFullscreen().catch(() => {});
    }
  }
}

async function windowClose() {
  try {
    await fetch('/api/window/close', { method: 'POST' });
  } catch (e) {
    window.close();
  }
}

// ==================== SESSION VIDEO RECORDING ENGINE (WebM VP9 / H.264) ====================
let mediaRecorder = null;
let recordedChunks = [];
let recordingStartTime = null;
let recordingTimerInterval = null;
let chosenMimeType = 'video/webm';

function getBestSupportedMimeType() {
  const types = [
    'video/webm; codecs=vp9,opus',
    'video/webm; codecs=vp9',
    'video/webm; codecs=h264,opus',
    'video/webm; codecs=h264',
    'video/webm',
    'video/mp4; codecs=avc1'
  ];
  for (const t of types) {
    if (MediaRecorder.isTypeSupported(t)) {
      return t;
    }
  }
  return 'video/webm';
}

function toggleSessionRecording() {
  if (mediaRecorder && mediaRecorder.state === 'recording') {
    stopSessionRecording();
  } else {
    startSessionRecording();
  }
}

function startSessionRecording() {
  if (!canvas) {
    alert('Kayıt başlatılamadı: Aktif uzak masaüstü ekranı bulunamadı.');
    return;
  }

  try {
    chosenMimeType = getBestSupportedMimeType();
    const stream = canvas.captureStream(30); // 30 FPS smooth video stream
    recordedChunks = [];

    mediaRecorder = new MediaRecorder(stream, {
      mimeType: chosenMimeType,
      videoBitsPerSecond: 2500000 // 2.5 Mbps crisp clarity
    });

    mediaRecorder.ondataavailable = (event) => {
      if (event.data && event.data.size > 0) {
        recordedChunks.push(event.data);
      }
    };

    mediaRecorder.onstop = saveRecordedSession;

    mediaRecorder.start(1000); // 1-second chunks
    recordingStartTime = Date.now();

    const btn = document.getElementById('btn-record-session');
    if (btn) {
      btn.style.background = '#e81123';
      btn.style.color = '#ffffff';
      btn.innerHTML = '⏹ Durdur (00:00)';
    }

    recordingTimerInterval = setInterval(() => {
      const elapsed = Math.floor((Date.now() - recordingStartTime) / 1000);
      const mins = String(Math.floor(elapsed / 60)).padStart(2, '0');
      const secs = String(elapsed % 60).padStart(2, '0');
      const recBtn = document.getElementById('btn-record-session');
      if (recBtn) {
        recBtn.innerHTML = `⏹ Durdur (${mins}:${secs})`;
      }
    }, 1000);

    console.log(`[Video Kaydedici] Kayıt başladı. Format: ${chosenMimeType}`);
  } catch (err) {
    console.error('Kayıt başlatılamadı:', err);
    alert('Ekran kaydı başlatılamadı: ' + err.message);
  }
}

function stopSessionRecording() {
  if (mediaRecorder && mediaRecorder.state === 'recording') {
    mediaRecorder.stop();
  }

  if (recordingTimerInterval) {
    clearInterval(recordingTimerInterval);
    recordingTimerInterval = null;
  }

  const btn = document.getElementById('btn-record-session');
  if (btn) {
    btn.style.background = '';
    btn.style.color = '';
    btn.innerHTML = '🔴 Kaydet';
  }
}

function saveRecordedSession() {
  if (recordedChunks.length === 0) return;

  const blob = new Blob(recordedChunks, { type: chosenMimeType });
  const ext = chosenMimeType.includes('mp4') ? 'mp4' : 'webm';
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const fileName = `ApexConnect_Kayit_${timestamp}.${ext}`;

  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.style.display = 'none';
  a.href = url;
  a.download = fileName;
  document.body.appendChild(a);
  a.click();

  setTimeout(() => {
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
  }, 100);

  alert(`Oturum videosu (${(blob.size / (1024 * 1024)).toFixed(2)} MB) başarıyla kaydedildi:\n${fileName}`);
}

