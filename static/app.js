// OxiDesk Frontend & Remote Desktop Engine

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
  startNetworkScan(true);

  // Native Frameless Window Drag Region
  const dragArea = document.querySelector('.titlebar-drag-area');
  if (dragArea) {
    dragArea.addEventListener('mousedown', (e) => {
      if (e.target.closest('button') || e.target.closest('input') || e.target.closest('select')) return;
      if (window.ipc) window.ipc.postMessage('drag');
    });
  }
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
  } else if (tabId === 'network-discovery') {
    startNetworkScan();
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
    alert(`OxiDesk Cihaz ID panoya kopyalandı: ${formatApexId(hostInfo.device_id)}`);
  }
}

function windowMinimize() {
  if (window.ipc) {
    window.ipc.postMessage('minimize');
  } else {
    fetch('/api/window/minimize', { method: 'POST' }).catch(() => {});
  }
}

function windowMaximize() {
  if (window.ipc) {
    window.ipc.postMessage('maximize');
  } else {
    fetch('/api/window/maximize', { method: 'POST' }).catch(() => {});
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(() => {});
    } else {
      document.exitFullscreen().catch(() => {});
    }
  }
}

function windowClose() {
  try {
    if (window.ipc && typeof window.ipc.postMessage === 'function') {
      window.ipc.postMessage('close');
    }
  } catch (e) {}
  try {
    fetch('/api/window/close', { method: 'POST' }).catch(() => {});
  } catch (e) {}
  setTimeout(() => {
    try { window.close(); } catch (e) {}
  }, 50);
}

function formatApexId(idStr) {
  if (!idStr) return '--- --- ---';
  const clean = idStr.replace(/\s+/g, '');
  if (clean.length === 9) {
    return `${clean.slice(0,3)} ${clean.slice(3,6)} ${clean.slice(6,9)}`;
  }
  return idStr;
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
  const target = prompt('OxiDesk ID veya Yerel IP Adresi:');
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
    alert('Lütfen bir hedef OxiDesk ID veya IP adresi girin.');
    return;
  }

  startRemoteSession(target, pin);
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function getActiveWs() {
  const sess = sessionManager.getActiveSession();
  return sess && sess.ws ? sess.ws : activeWs;
}

// ==================== MULTI-SESSION REMOTE DESKTOP ENGINE ====================
const sessionManager = {
  sessions: new Map(), // sessionId -> Session Object
  activeId: null,
  isGridMode: false,

  hasSessions() {
    return this.sessions.size > 0;
  },

  getActiveSession() {
    return this.sessions.get(this.activeId) || null;
  },

  updateTabBar() {
    const dynamicContainer = document.getElementById('dynamic-session-tabs');
    const controlsContainer = document.getElementById('tab-bar-controls');
    const dashboardTabBtn = document.getElementById('tab-btn-dashboard');
    const navViewerBtn = document.getElementById('nav-viewer-btn');

    if (!dynamicContainer) return;

    if (this.sessions.size > 0) {
      if (controlsContainer) controlsContainer.style.display = 'flex';
      if (navViewerBtn) navViewerBtn.style.display = 'inline-flex';
    } else {
      if (controlsContainer) controlsContainer.style.display = 'none';
      if (navViewerBtn) navViewerBtn.style.display = 'none';
      if (dashboardTabBtn) dashboardTabBtn.classList.add('active');
    }

    if (currentTab === 'remote-viewer' && this.activeId) {
      if (dashboardTabBtn) dashboardTabBtn.classList.remove('active');
    } else {
      if (dashboardTabBtn) dashboardTabBtn.classList.add('active');
    }

    dynamicContainer.innerHTML = '';
    this.sessions.forEach((session, id) => {
      const btn = document.createElement('button');
      btn.className = `session-tab-btn ${this.activeId === id && currentTab === 'remote-viewer' ? 'active' : ''}`;
      btn.title = `${session.name} (${session.targetHost})`;
      btn.innerHTML = `
        <span class="tab-icon">🖥️</span>
        <span class="tab-title">${escapeHtml(session.name)}</span>
        <span class="tab-close-btn" onclick="event.stopPropagation(); sessionManager.closeSession('${id}')" title="Bu Bağlantıyı Kapat">✕</span>
      `;
      btn.onclick = () => this.switchToSession(id);
      dynamicContainer.appendChild(btn);
    });

    this.updateGridClasses();
  },

  updateGridClasses() {
    const gridContainer = document.getElementById('multi-sessions-grid');
    if (!gridContainer) return;

    const count = this.sessions.size;
    gridContainer.className = 'sessions-grid-container';

    if (this.isGridMode && count > 0) {
      gridContainer.classList.add('grid-active-mode');
      const capped = Math.min(Math.max(count, 1), 4);
      gridContainer.classList.add(`grid-count-${capped}`);
    } else {
      gridContainer.classList.add('single-active-mode');
    }

    const gridLabel = document.getElementById('grid-mode-label');
    const gridBtn = document.getElementById('btn-grid-toggle');
    const islandGridBtn = document.getElementById('btn-island-grid');
    if (gridLabel) {
      gridLabel.innerText = this.isGridMode ? 'Tek Ekran' : 'Bölünmüş Ekran';
    }
    if (gridBtn) {
      if (this.isGridMode) gridBtn.classList.add('active');
      else gridBtn.classList.remove('active');
    }
    if (islandGridBtn) {
      if (this.isGridMode) islandGridBtn.classList.add('active');
      else islandGridBtn.classList.remove('active');
    }
  },

  switchToSession(id) {
    if (!this.sessions.has(id)) return;
    this.activeId = id;
    const session = this.sessions.get(id);

    // Update global compatibility aliases
    activeWs = session.ws;
    canvas = session.canvas;
    ctx = session.ctx;
    remoteWidth = session.remoteWidth;
    remoteHeight = session.remoteHeight;
    latency = session.latency;
    fps = session.fps;
    bandwidth = session.bandwidth;

    // Update active highlight on session containers
    document.querySelectorAll('.session-view-wrapper').forEach(w => {
      w.classList.remove('active');
    });
    if (session.wrapper) {
      session.wrapper.classList.add('active');
      session.canvas.focus();
    }

    // Switch view to remote viewer
    switchNavTab('remote-viewer');
    this.updateTabBar();
    this.updateViewerHeader(session);
  },

  updateViewerHeader(session) {
    const titleEl = document.getElementById('viewer-target-title');
    if (titleEl && session) {
      titleEl.innerText = session.isConnected ? `Bağlı: ${session.name} (${session.targetHost})` : `Bağlanılıyor: ${session.name}`;
    }
    const statsEl = document.getElementById('session-stats');
    if (statsEl && session) {
      statsEl.innerText = `${session.fps} FPS | ${session.latency} ms | ${session.bandwidth} KB/s`;
    }
  },

  toggleGridMode() {
    this.isGridMode = !this.isGridMode;
    this.updateGridClasses();
    if (!this.isGridMode && this.activeId) {
      this.switchToSession(this.activeId);
    }
  },

  closeSession(id) {
    const session = this.sessions.get(id);
    if (!session) return;

    if (session.ws) {
      try { session.ws.close(); } catch (e) {}
    }
    if (session.pingInterval) {
      clearInterval(session.pingInterval);
    }
    if (session.wrapper && session.wrapper.parentNode) {
      session.wrapper.parentNode.removeChild(session.wrapper);
    }

    this.sessions.delete(id);

    if (this.activeId === id) {
      const remainingIds = Array.from(this.sessions.keys());
      if (remainingIds.length > 0) {
        this.switchToSession(remainingIds[remainingIds.length - 1]);
      } else {
        this.activeId = null;
        activeWs = null;
        canvas = null;
        ctx = null;
        this.updateTabBar();
        switchNavTab('quick-connect');
      }
    } else {
      this.updateTabBar();
    }
  }
};

function switchToSessionTab(tabId) {
  if (tabId === 'dashboard') {
    switchNavTab(currentTab === 'remote-viewer' ? 'quick-connect' : currentTab);
    sessionManager.updateTabBar();
  } else {
    sessionManager.switchToSession(tabId);
  }
}

function openNewConnectionTab() {
  switchNavTab('quick-connect');
  sessionManager.updateTabBar();
  const input = document.getElementById('target-input');
  if (input) {
    input.focus();
    input.select();
  }
}

function toggleMultiGridMode() {
  sessionManager.toggleGridMode();
}

function closeCurrentSession() {
  if (sessionManager.activeId) {
    sessionManager.closeSession(sessionManager.activeId);
  } else {
    closeSession();
  }
}

function closeLiveSession() {
  closeCurrentSession();
}

async function startRemoteSession(target, pin, customName) {
  const cleanTarget = target.trim().replace(/\s+/g, '');
  let targetHost = cleanTarget;
  let resolvedName = customName || cleanTarget;

  // If target doesn't specify an explicit port or is an ID:
  if (!cleanTarget.includes(':')) {
    try {
      const res = await fetch(`/api/resolve?target=${encodeURIComponent(cleanTarget)}`);
      if (res.ok) {
        const data = await res.json();
        if (data && data.found) {
          targetHost = `${data.ip}:${data.port}`;
          if (data.hostname) resolvedName = data.hostname;
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
    alert(`OxiDesk ID (${target}) yerel ağda çözümlenemedi.\nLütfen hedef bilgisayarın açık olduğundan veya IP:PORT (Örn: 192.168.1.50:8080) şeklinde doğrudan bağlanmayı deneyin.`);
    return;
  }

  const sessionId = 'sess_' + Date.now() + '_' + Math.floor(Math.random() * 1000);
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${targetHost}/ws/client?pin=${encodeURIComponent(pin)}`;
  console.log(`[Multi-Session] Starting session ${sessionId} to:`, wsUrl);

  // Create DOM elements for this session inside #multi-sessions-grid
  const gridContainer = document.getElementById('multi-sessions-grid');
  if (!gridContainer) return;

  const wrapper = document.createElement('div');
  wrapper.className = 'session-view-wrapper';
  wrapper.id = `wrapper-${sessionId}`;
  wrapper.onclick = () => {
    if (sessionManager.isGridMode) {
      sessionManager.switchToSession(sessionId);
    }
  };

  wrapper.innerHTML = `
    <div class="session-view-header">
      <span class="sess-name">🖥️ ${escapeHtml(resolvedName)} (${escapeHtml(targetHost)})</span>
      <span class="sess-badge" id="badge-${sessionId}">Bağlanıyor...</span>
    </div>
    <div class="canvas-sub-wrapper">
      <canvas id="canvas-${sessionId}" tabindex="0"></canvas>
    </div>
  `;
  gridContainer.appendChild(wrapper);

  const sessCanvas = wrapper.querySelector('canvas');
  const sessCtx = sessCanvas.getContext('2d');

  const sessionObj = {
    id: sessionId,
    target: cleanTarget,
    targetHost: targetHost,
    name: resolvedName,
    pin: pin,
    ws: null,
    wrapper: wrapper,
    canvas: sessCanvas,
    ctx: sessCtx,
    remoteWidth: 1920,
    remoteHeight: 1080,
    fps: 0,
    latency: 0,
    bandwidth: 0,
    frameCount: 0,
    bytesReceived: 0,
    lastFrameTime: performance.now(),
    pingInterval: null,
    isConnected: false
  };

  attachSessionCanvasEvents(sessionObj);

  // Setup WebSocket connection
  const ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';
  sessionObj.ws = ws;

  ws.onopen = () => {
    sessionObj.isConnected = true;
    const badge = document.getElementById(`badge-${sessionId}`);
    if (badge) {
      badge.innerText = 'Canlı';
      badge.style.background = 'rgba(16, 124, 65, 0.4)';
    }
    if (sessionManager.activeId === sessionId) {
      sessionManager.updateViewerHeader(sessionObj);
    }

    // Ping loop
    sessionObj.pingInterval = setInterval(() => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ping', timestamp: Date.now() }));
      }
    }, 2000);
  };

  ws.onmessage = (event) => {
    if (typeof event.data === 'string') {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'screen_size') {
          sessionObj.remoteWidth = msg.width;
          sessionObj.remoteHeight = msg.height;
          sessCanvas.width = msg.width;
          sessCanvas.height = msg.height;
        } else if (msg.type === 'pong') {
          sessionObj.latency = Date.now() - msg.timestamp;
          if (sessionManager.activeId === sessionId) {
            latency = sessionObj.latency;
            sessionManager.updateViewerHeader(sessionObj);
          }
        } else if (msg.type === 'chat') {
          receiveChatMessage(msg);
        } else if (msg.type === 'file_progress') {
          updateUploadProgress(msg.id, msg.received);
          updateSessionUploadProgress(msg.id, msg.received);
        } else if (msg.type === 'file_complete') {
          finishUploadProgress(msg.name);
          finishSessionUploadProgress(msg.name);
        }
      } catch (e) {}
    } else {
      // Binary JPEG frame
      const byteLen = event.data.byteLength || 0;
      sessionObj.bytesReceived += byteLen;

      const blob = new Blob([event.data], { type: 'image/jpeg' });
      createImageBitmap(blob).then(bitmap => {
        if (sessCanvas.width !== bitmap.width || sessCanvas.height !== bitmap.height) {
          sessCanvas.width = bitmap.width;
          sessCanvas.height = bitmap.height;
          sessionObj.remoteWidth = bitmap.width;
          sessionObj.remoteHeight = bitmap.height;
        }
        sessCtx.drawImage(bitmap, 0, 0);
        bitmap.close();

        // Push frame to recording stream if actively recording this session
        if (sessionManager.activeId === sessionId && recordingStreamTrack && recordingStreamTrack.requestFrame) {
          try { recordingStreamTrack.requestFrame(); } catch (e) {}
        }

        sessionObj.frameCount++;
        const now = performance.now();
        if (now - sessionObj.lastFrameTime >= 1000) {
          sessionObj.fps = Math.round((sessionObj.frameCount * 1000) / (now - sessionObj.lastFrameTime));
          sessionObj.bandwidth = Math.round(sessionObj.bytesReceived / 1024);
          sessionObj.frameCount = 0;
          sessionObj.bytesReceived = 0;
          sessionObj.lastFrameTime = now;
          if (sessionManager.activeId === sessionId) {
            fps = sessionObj.fps;
            bandwidth = sessionObj.bandwidth;
            sessionManager.updateViewerHeader(sessionObj);
          }
        }
      });
    }
  };

  ws.onclose = () => {
    sessionObj.isConnected = false;
    const badge = document.getElementById(`badge-${sessionId}`);
    if (badge) {
      badge.innerText = 'Kapandı';
      badge.style.background = 'rgba(239, 68, 68, 0.4)';
    }
    if (sessionObj.pingInterval) clearInterval(sessionObj.pingInterval);
    if (sessionManager.activeId === sessionId) {
      sessionManager.updateViewerHeader(sessionObj);
    }
  };

  ws.onerror = (err) => {
    console.error(`Session ${sessionId} error:`, err);
    alert(`Uzak cihaza (${resolvedName} - ${targetHost}) bağlanırken hata oluştu.\n- Cihazın açık ve erişilebilir olduğunu\n- Girilen PIN kodunu kontrol edin.`);
  };

  sessionManager.sessions.set(sessionId, sessionObj);
  sessionManager.switchToSession(sessionId);
}

function attachSessionCanvasEvents(session) {
  const c = session.canvas;
  if (!c) return;

  function getNormalizedCoords(e) {
    const rect = c.getBoundingClientRect();
    const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const y = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
    return { x, y };
  }

  c.addEventListener('mousemove', (e) => {
    if (!session.ws || session.ws.readyState !== WebSocket.OPEN) return;
    const { x, y } = getNormalizedCoords(e);
    session.ws.send(JSON.stringify({ type: 'mouse_move', x, y }));
  });

  c.addEventListener('mousedown', (e) => {
    if (!session.ws || session.ws.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    c.focus();
    if (sessionManager.activeId !== session.id) {
      sessionManager.switchToSession(session.id);
    }
    const { x, y } = getNormalizedCoords(e);
    session.ws.send(JSON.stringify({ type: 'mouse_down', button: e.button, x, y }));
  });

  c.addEventListener('mouseup', (e) => {
    if (!session.ws || session.ws.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    const { x, y } = getNormalizedCoords(e);
    session.ws.send(JSON.stringify({ type: 'mouse_up', button: e.button, x, y }));
  });

  c.addEventListener('contextmenu', (e) => e.preventDefault());

  c.addEventListener('wheel', (e) => {
    if (!session.ws || session.ws.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    const delta = e.deltaY < 0 ? 120 : -120;
    session.ws.send(JSON.stringify({ type: 'mouse_wheel', delta }));
  }, { passive: false });

  // Mobile Touch Emulation
  let touchStartTime = 0;
  c.addEventListener('touchstart', (e) => {
    touchStartTime = performance.now();
    if (e.touches.length === 1) {
      const touch = e.touches[0];
      const { x, y } = getNormalizedCoords(touch);
      if (session.ws && session.ws.readyState === WebSocket.OPEN) {
        session.ws.send(JSON.stringify({ type: 'mouse_move', x, y }));
      }
    }
  }, { passive: true });

  c.addEventListener('touchmove', (e) => {
    if (e.touches.length === 1) {
      const touch = e.touches[0];
      const { x, y } = getNormalizedCoords(touch);
      if (session.ws && session.ws.readyState === WebSocket.OPEN) {
        session.ws.send(JSON.stringify({ type: 'mouse_move', x, y }));
      }
    }
  }, { passive: true });

  c.addEventListener('touchend', (e) => {
    const duration = performance.now() - touchStartTime;
    if (session.ws && session.ws.readyState === WebSocket.OPEN) {
      const btn = duration > 500 ? 2 : 0;
      session.ws.send(JSON.stringify({ type: 'mouse_click', button: btn }));
    }
  }, { passive: true });
}

function updateStatsDisplay() {
  const sess = sessionManager.getActiveSession();
  if (sess) {
    sessionManager.updateViewerHeader(sess);
  }
}

function closeSession() {
  if (sessionManager.hasSessions()) {
    sessionManager.closeSession(sessionManager.activeId);
  } else {
    document.getElementById('nav-viewer-btn').style.display = 'none';
    switchNavTab('quick-connect');
  }
}

// Global Canvas Input & Keyboard Events
function initCanvasEvents() {
  window.addEventListener('keydown', (e) => {
    if (currentTab !== 'remote-viewer') return;
    const ws = getActiveWs();
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    if (['F5', 'F12'].includes(e.key)) return;
    e.preventDefault();
    ws.send(JSON.stringify({ type: 'key_down', key: e.key, code: e.code }));
  });

  window.addEventListener('keyup', (e) => {
    if (currentTab !== 'remote-viewer') return;
    const ws = getActiveWs();
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    e.preventDefault();
    ws.send(JSON.stringify({ type: 'key_up', key: e.key, code: e.code }));
  });
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
  const savedTheme = localStorage.getItem('oxidesk_theme') || localStorage.getItem('apex_theme') || 'dark';
  applyTheme(savedTheme);
}

function applyTheme(theme) {
  const html = document.documentElement;
  if (theme === 'light') {
    html.classList.remove('dark');
    html.classList.add('light');
    localStorage.setItem('oxidesk_theme', 'light');
  } else {
    html.classList.remove('light');
    html.classList.add('dark');
    localStorage.setItem('oxidesk_theme', 'dark');
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


// ==================== SESSION VIDEO RECORDING ENGINE (WebM VP9 / H.264) ====================
let mediaRecorder = null;
let recordedChunks = [];
let recordingStartTime = null;
let recordingTimerInterval = null;
let recordingFramePump = null;
let recordingStreamTrack = null;
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
    const stream = canvas.captureStream(30); // 30 FPS stream
    recordedChunks = [];
    recordingStreamTrack = stream.getVideoTracks()[0] || null;

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

    // Active frame pump to ensure video frames continuously reach the encoder even on static desktop
    recordingFramePump = setInterval(() => {
      if (recordingStreamTrack && recordingStreamTrack.requestFrame) {
        try { recordingStreamTrack.requestFrame(); } catch (e) {}
      }
    }, 33);

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
  if (recordingFramePump) {
    clearInterval(recordingFramePump);
    recordingFramePump = null;
  }
  recordingStreamTrack = null;

  if (mediaRecorder && mediaRecorder.state === 'recording') {
    try {
      mediaRecorder.requestData(); // Flush all recorded chunks before stopping
    } catch (e) {}
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
  if (recordedChunks.length === 0) {
    alert('Kayıt dosyası oluşturulamadı: Video karesi alınamadı.');
    return;
  }

  const blob = new Blob(recordedChunks, { type: chosenMimeType });
  if (blob.size === 0) {
    alert('Kayıt dosyası boş (0 byte).');
    return;
  }

  const ext = chosenMimeType.includes('mp4') ? 'mp4' : 'webm';
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const fileName = `OxiDesk_Kayit_${timestamp}.${ext}`;

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

// ==================== IN-SESSION FILE TRANSFER POPUP ====================
function toggleSessionFileModal() {
  const modal = document.getElementById('session-file-modal');
  if (!modal) return;
  modal.classList.toggle('hidden');
  if (!modal.classList.contains('hidden')) {
    loadSessionFilesList();
  }
}

async function loadSessionFilesList() {
  const container = document.getElementById('session-files-items');
  if (!container) return;
  try {
    const res = await fetch('/api/files');
    if (!res.ok) return;
    const files = await res.json();
    if (files.length === 0) {
      container.innerHTML = '<div style="color:var(--text-muted);font-size:0.75rem;padding:6px;">Henüz aktarılan dosya yok.</div>';
      return;
    }
    container.innerHTML = files.slice(0, 8).map(f => `
      <div class="session-file-item">
        <span class="session-file-name" title="${escapeHtml(f.name)}">📄 ${escapeHtml(f.name)}</span>
        <button class="btn btn-subtle btn-xs" onclick="window.open('/api/files/download/${encodeURIComponent(f.name)}')">İndir</button>
      </div>
    `).join('');
  } catch (e) {}
}

function handleSessionFileSelect(event) {
  const file = event.target.files[0];
  if (!file) return;
  uploadSessionFile(file);
}

function uploadSessionFile(file) {
  if (!activeWs || activeWs.readyState !== WebSocket.OPEN) {
    alert('Aktif bir oturum bağlı değil.');
    return;
  }
  const fileId = 'sf_' + Date.now();
  const chunkSize = 64 * 1024; // 64 KB chunks
  let offset = 0;

  const progressCard = document.getElementById('session-upload-progress');
  const filenameEl = document.getElementById('session-upload-filename');
  const percentEl = document.getElementById('session-upload-percentage');
  const barEl = document.getElementById('session-upload-bar');

  if (progressCard) progressCard.style.display = 'block';
  if (filenameEl) filenameEl.innerText = `${file.name} (${(file.size / 1024).toFixed(1)} KB)`;

  activeWs.send(JSON.stringify({
    type: 'file_start',
    id: fileId,
    name: file.name,
    size: file.size
  }));

  const reader = new FileReader();

  function sendNextChunk() {
    if (offset >= file.size) {
      activeWs.send(JSON.stringify({
        type: 'file_end',
        id: fileId
      }));
      if (percentEl) percentEl.innerText = '100% Tamamlandı';
      if (barEl) barEl.style.width = '100%';
      setTimeout(() => {
        if (progressCard) progressCard.style.display = 'none';
        loadSessionFilesList();
      }, 1500);
      return;
    }

    const slice = file.slice(offset, offset + chunkSize);
    reader.onload = (e) => {
      const arrayBuffer = e.target.result;
      const base64Data = arrayBufferToBase64(arrayBuffer);
      activeWs.send(JSON.stringify({
        type: 'file_chunk',
        id: fileId,
        offset: offset,
        data: base64Data
      }));

      offset += chunkSize;
      const pct = Math.min(100, Math.round((offset / file.size) * 100));
      if (percentEl) percentEl.innerText = `${pct}%`;
      if (barEl) barEl.style.width = `${pct}%`;

      setTimeout(sendNextChunk, 10);
    };
    reader.readAsArrayBuffer(slice);
  }

  setTimeout(sendNextChunk, 80);
}

function updateSessionUploadProgress(id, received) {
  // Handled by local chunk loop
}

function finishSessionUploadProgress(name) {
  loadSessionFilesList();
}

// ==================== IN-SESSION CHAT DRAWER ====================
let unreadChatCount = 0;
let chatMessages = [];

function toggleSessionChatModal() {
  const modal = document.getElementById('session-chat-modal');
  if (!modal) return;
  modal.classList.toggle('hidden');
  if (!modal.classList.contains('hidden')) {
    unreadChatCount = 0;
    const badge = document.getElementById('chat-unread-badge');
    if (badge) badge.classList.add('hidden');
    const input = document.getElementById('session-chat-input');
    if (input) input.focus();
    renderChatMessages();
  }
}

function handleChatKeyDown(event) {
  if (event.key === 'Enter') {
    sendChatMessage();
  }
}

function sendChatMessage() {
  const input = document.getElementById('session-chat-input');
  if (!input) return;
  const text = input.value.trim();
  if (!text) return;

  const msg = {
    type: 'chat',
    sender: 'IT Yöneticisi',
    text: text,
    timestamp: Math.floor(Date.now() / 1000)
  };

  if (activeWs && activeWs.readyState === WebSocket.OPEN) {
    activeWs.send(JSON.stringify(msg));
  } else {
    fetch('/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sender: msg.sender, text: msg.text })
    }).catch(() => {});
  }

  chatMessages.push({ ...msg, isMine: true });
  renderChatMessages();
  input.value = '';
}

function receiveChatMessage(msg) {
  const isMine = msg.sender === 'IT Yöneticisi';
  chatMessages.push({ ...msg, isMine });
  const modal = document.getElementById('session-chat-modal');
  if (modal && modal.classList.contains('hidden') && !isMine) {
    unreadChatCount++;
    const badge = document.getElementById('chat-unread-badge');
    if (badge) {
      badge.innerText = unreadChatCount;
      badge.classList.remove('hidden');
    }
  }
  renderChatMessages();
}

function renderChatMessages() {
  const container = document.getElementById('session-chat-messages');
  if (!container) return;
  container.innerHTML = `
    <div class="chat-system-msg">Uzak kullanıcı ile canlı mesajlaşma aktif.</div>
    ${chatMessages.map(m => {
      const timeStr = new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      return `
        <div class="chat-msg ${m.isMine ? 'mine' : 'theirs'}">
          <span style="font-size:0.68rem;opacity:0.85;margin-bottom:2px;">${escapeHtml(m.sender)}</span>
          <span>${escapeHtml(m.text)}</span>
          <span class="chat-meta">${timeStr}</span>
        </div>
      `;
    }).join('')}
  `;
  container.scrollTop = container.scrollHeight;
}

// ==================== NETWORK DISCOVERY & SOS RADAR ====================
async function startNetworkScan(silent = false) {
  const btn = document.getElementById('btn-scan');
  const tbody = document.getElementById('scan-results-body');
  if (!silent && btn) {
    btn.disabled = true;
    btn.innerHTML = '⏳ Taranıyor...';
  }

  try {
    const res = await fetch('/api/scan');
    if (!res.ok) throw new Error('Ağ tarama hatası');
    const data = await res.json();
    const devices = data.devices || [];

    if (!tbody) return;

    if (devices.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="6" class="table-empty">
            Ağda aktif başka bir OxiDesk veya QuickSupport cihazı bulunamadı.
          </td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = devices.map(dev => {
      const isSos = dev.is_sos || (dev.service_type && dev.service_type.includes('SOS'));
      const badgeClass = isSos ? 'badge-offline' : 'badge-success';
      const badgeText = isSos ? '🚨 SOS / YARDIM BEKLİYOR' : '🟢 Aktif';
      const formattedId = dev.device_id ? formatApexId(dev.device_id) : '—';
      const userDisplay = dev.user_name ? ` (${escapeHtml(dev.user_name)})` : '';
      const connectTarget = dev.device_id || `${dev.ip}:${dev.port}`;

      return `
        <tr style="${isSos ? 'background: rgba(239, 68, 68, 0.08);' : ''}">
          <td><span class="badge ${badgeClass}">${badgeText}</span></td>
          <td><strong>${escapeHtml(dev.hostname)}</strong>${userDisplay}</td>
          <td><code>${dev.ip}:${dev.port}</code></td>
          <td><span class="badge badge-tag" style="font-family:monospace;font-size:0.85rem;">${formattedId}</span></td>
          <td>${escapeHtml(dev.service_type)}</td>
          <td>
            <button class="btn btn-fluent-primary btn-xs" onclick="quickConnectTo('${connectTarget}')">
              ⚡ Bağlan
            </button>
          </td>
        </tr>
      `;
    }).join('');
  } catch (err) {
    console.error('Network scan failed:', err);
    if (!silent && tbody) {
      tbody.innerHTML = `<tr><td colspan="6" class="table-empty" style="color:#ef4444;">Tarama sırasında hata oluştu.</td></tr>`;
    }
  } finally {
    if (!silent && btn) {
      btn.disabled = false;
      btn.innerHTML = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/></svg> Ağı Tara`;
    }
  }
}

function quickConnectTo(target) {
  switchNavTab('quick-connect');
  const input = document.getElementById('target-input');
  if (input) {
    input.value = target;
    input.focus();
  }
  const pinInput = document.getElementById('pin-input');
  if (pinInput) {
    pinInput.focus();
  }
}


