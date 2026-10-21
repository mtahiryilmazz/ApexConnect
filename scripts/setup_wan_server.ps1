<#
.SYNOPSIS
    OxiDesk v4.0.0 - Otomatik WAN Rendezvous / Relay Sunucusu Kurulum ve Yapılandırma Aracı (Windows)
.DESCRIPTION
    Bu araç, uzak ağlardaki (farklı şirketler, evler, NAT/Güvenlik Duvarı arkası)
    bilgisayarların OxiDesk ile doğrudan ve kesintisiz haberleşebilmesi için
    merkezi sinyal ve aktarım sunucusunu otomatik olarak kurar, yapılandırır ve Windows Güvenlik Duvarı kurallarını açar.
#>

param(
    [int]$Port = 8095,
    [switch]$InstallService,
    [switch]$StartDocker
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   OxiDesk v4.0.0 - WAN Rendezvous & Relay Server Kurulumu  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Administrator Yetkisi Kontrolü
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warning "UYARI: Güvenlik Duvarı portunu açmak ve Servis kurmak için Yönetici (Administrator) yetkisi önerilir."
}

# 2. Windows Güvenlik Duvarı (Firewall) Kuralı Oluşturma
Write-Host "[1/3] Windows Güvenlik Duvarı Kuralı Yapılandırılıyor..." -ForegroundColor Yellow
try {
    $existingRule = Get-NetFirewallRule -DisplayName "OxiDesk Rendezvous WAN (Port $Port)" -ErrorAction SilentlyContinue
    if (-not $existingRule) {
        New-NetFirewallRule -DisplayName "OxiDesk Rendezvous WAN (Port $Port)" `
            -Direction Inbound `
            -Action Allow `
            -Protocol TCP `
            -LocalPort $Port `
            -Description "OxiDesk v4.0.0 WAN Sinyal ve NAT Traversal Sunucusu" | Out-Null
        Write-Host "  -> Firewall kuralı başarıyla eklendi (TCP $Port Inbound)." -ForegroundColor Green
    } else {
        Write-Host "  -> Firewall kuralı zaten mevcut." -ForegroundColor DarkGray
    }
} catch {
    Write-Warning "  -> Firewall kuralı eklenemedi (Yönetici olarak çalıştırmayı deneyin): $_"
}

# 3. Yerel ve Dış IP Adresini Belirleme
Write-Host ""
Write-Host "[2/3] Sunucu Ağ Bilgileri Tespit Ediliyor..." -ForegroundColor Yellow
$localIps = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.InterfaceAlias -notmatch "Loopback" -and $_.IPAddress -notmatch "^169\.254" }).IPAddress
$primaryIp = if ($localIps) { $localIps[0] } else { "127.0.0.1" }

$publicIp = "Bilinmiyor"
try {
    $publicIp = (Invoke-RestMethod -Uri "https://api.ipify.org" -TimeoutSec 3).Trim()
} catch {
    try {
        $publicIp = (Invoke-RestMethod -Uri "https://icanhazip.com" -TimeoutSec 3).Trim()
    } catch {
        $publicIp = "Dış IP tespit edilemedi (İnternet bağlantınızı kontrol edin)"
    }
}

Write-Host "  -> Yerel IP   : $primaryIp" -ForegroundColor Cyan
Write-Host "  -> Dış (WAN) IP: $publicIp" -ForegroundColor Cyan
Write-Host "  -> Port       : $Port" -ForegroundColor Cyan

# 4. Sunucu Çalıştırma / Yapılandırma
Write-Host ""
Write-Host "[3/3] Sunucu Dağıtım Seçenekleri:" -ForegroundColor Yellow

$dockerInstalled = Get-Command docker -ErrorAction SilentlyContinue

if ($dockerInstalled) {
    Write-Host "  [A] Docker tespit edildi. Konteyner olarak başlatmak ister misiniz?" -ForegroundColor White
    $dockerCmd = "docker run -d --restart always -p ${Port}:${Port} --name oxidesk-relay oxidesk/relay:latest"
    Write-Host "      Komut: $dockerCmd" -ForegroundColor DarkGray
    if ($StartDocker) {
        Write-Host "      Docker konteyneri başlatılıyor..." -ForegroundColor Green
        Invoke-Expression $dockerCmd
    }
}

# 5. Sunucu Başlatma Scripti Oluşturma (Stand-Alone)
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$binPath = Join-Path $rootDir "target\release\oxidesk_rendezvous.exe"

$startBatPath = Join-Path $rootDir "start_wan_server.bat"
$startBatContent = @"
@echo off
title OxiDesk WAN Rendezvous Server (Port $Port)
set OXIDESK_RENDEZVOUS_PORT=$Port
echo ========================================================
echo   OxiDesk WAN Rendezvous Server Baslatiliyor...
echo   Port: $Port
echo   Yerel URL: ws://127.0.0.1:$Port/ws/signaling
echo   WAN URL  : ws://$publicIp:$Port/ws/signaling
echo ========================================================
if exist "$binPath" (
    "$binPath"
) else (
    echo [BILGI] Cargo ile derleniyor...
    cargo run --release --bin oxidesk_rendezvous
)
pause
"@

Set-Content -Path $startBatPath -Value $startBatContent -Encoding UTF8
Write-Host "  -> Stand-alone başlatma aracı oluşturuldu: start_wan_server.bat" -ForegroundColor Green

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "   KURULUM VE YAPILANDIRMA TAMAMLANDI!                     " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "OxiDesk İstemcileri için WAN Sunucu Adresi:" -ForegroundColor White
Write-Host "  ws://${publicIp}:${Port}/ws/signaling" -ForegroundColor Yellow -BackgroundColor Black
Write-Host ""
Write-Host "Bu adresi OxiDesk -> Ayarlar -> Dış Ağlar / WAN Sunucusu alanına yapıştırabilirsiniz." -ForegroundColor Cyan
Write-Host ""
