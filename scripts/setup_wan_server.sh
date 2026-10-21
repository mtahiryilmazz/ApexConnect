#!/usr/bin/env bash
# ==============================================================================
# OxiDesk v4.0.0 - Automated Linux / VPS WAN Server Setup Tool
# Sets up OxiDesk WAN Rendezvous & Relay Server on Ubuntu/Debian/CentOS/Alpine
# ==============================================================================

set -e

PORT=${1:-8095}

echo "=========================================================="
echo "   OxiDesk v4.0.0 - Linux WAN Server Setup & Config Tool   "
echo "=========================================================="

# 1. Determine Public IP
WAN_IP=$(curl -s -4 https://api.ipify.org || curl -s -4 https://icanhazip.com || echo "YOUR_SERVER_IP")
echo "[+] Detected WAN Public IP: $WAN_IP"
echo "[+] Listening Port        : $PORT"

# 2. Check Firewall (UFW or Firewalld)
if command -v ufw >/dev/null 2>&1; then
    echo "[+] Configuring UFW firewall for port $PORT..."
    ufw allow "$PORT/tcp" comment "OxiDesk Rendezvous WAN" || true
elif command -v firewall-cmd >/dev/null 2>&1; then
    echo "[+] Configuring Firewalld for port $PORT..."
    firewall-cmd --permanent --add-port="$PORT/tcp" || true
    firewall-cmd --reload || true
fi

# 3. Check Docker or systemd deployment
if command -v docker >/dev/null 2>&1; then
    echo "[+] Docker detected. Running container..."
    docker stop oxidesk-relay 2>/dev/null || true
    docker rm oxidesk-relay 2>/dev/null || true
    docker run -d \
      --name oxidesk-relay \
      --restart always \
      -p "${PORT}:${PORT}" \
      -e PORT="${PORT}" \
      oxidesk/relay:latest || echo "[-] Docker pull fallback..."
fi

# 4. Create systemd Service file for native deployment
cat <<EOF > /etc/systemd/system/oxidesk-wan.service
[Unit]
Description=OxiDesk WAN Rendezvous & Relay Server v4.0.0
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/oxidesk
ExecStart=/opt/oxidesk/oxidesk_rendezvous
Restart=always
RestartSec=3
Environment=OXIDESK_RENDEZVOUS_PORT=$PORT

[Install]
WantedBy=multi-user.target
EOF

echo "[+] Systemd service template written to /etc/systemd/system/oxidesk-wan.service"

echo ""
echo "=========================================================="
echo "   SUCCESS! OxiDesk WAN Server is Configured!             "
echo "=========================================================="
echo "Client Configuration URL (Enter in OxiDesk Settings):"
echo "  ws://${WAN_IP}:${PORT}/ws/signaling"
echo "=========================================================="
