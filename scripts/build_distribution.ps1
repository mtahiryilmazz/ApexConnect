<#
.SYNOPSIS
    OxiDesk Kurumsal Dağıtım Paketi Oluşturucu
.DESCRIPTION
    Hem IT Yönetim Paketi (oxidesk.exe -> OxiDesk.exe) hem de Son Kullanıcı Mini Hızlı Destek (oxidesk_qs.exe -> OxiQuickSupport.exe)
    dosyalarını optimize release olarak derler ve dist/ klasörüne kopyalar.
#>

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
Set-Location $RepoRoot

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "OxiDesk - Kurumsal Dagitim Paketi Olusturucu" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Release derlemeleri
Write-Host "`nPaketler optimize ediliyor ve derleniyor..." -ForegroundColor Cyan
& cargo build --release --bin oxidesk --bin oxidesk_qs --bin oxidesk_rendezvous --bin oxidesk_service
if ($LASTEXITCODE -ne 0) {
    Write-Error "Release derlemesi basarisiz oldu!"
}

# 2. dist/ ve dist/tools/ dizini olusturma ve kopyalama
$DistDir = Join-Path $RepoRoot "dist"
$ToolsDir = Join-Path $DistDir "tools"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}
if (-not (Test-Path $ToolsDir)) {
    New-Item -ItemType Directory -Path $ToolsDir | Out-Null
}

# Ana Programlar (Kullanıcı için en üst dizinde)
$AdminSrc = Join-Path $RepoRoot "target\release\oxidesk.exe"
$AdminDst = Join-Path $DistDir "OxiDesk.exe"
Copy-Item -Path $AdminSrc -Destination $AdminDst -Force

$QsSrc = Join-Path $RepoRoot "target\release\oxidesk_qs.exe"
$QsDst = Join-Path $DistDir "OxiDesk-QuickSup.exe"
Copy-Item -Path $QsSrc -Destination $QsDst -Force

# Yardımcı ve Sunucu Araçları (tools/ alt dizininde)
$RendezvousSrc = Join-Path $RepoRoot "target\release\oxidesk_rendezvous.exe"
$RendezvousDst = Join-Path $ToolsDir "oxidesk_rendezvous.exe"
Copy-Item -Path $RendezvousSrc -Destination $RendezvousDst -Force

$ServiceSrc = Join-Path $RepoRoot "target\release\oxidesk_service.exe"
$ServiceDst = Join-Path $ToolsDir "oxidesk_service.exe"
Copy-Item -Path $ServiceSrc -Destination $ServiceDst -Force

# 3. Bilgilendirme Notu
$ReadmeContent = @"
OxiDesk Kurumsal Uzak Masaüstü Dağıtım Paketi (v5.0.0 - Pure Win32 Native)
========================================================================

Bu sürüm (v5.0.0), AnyDesk ve RustDesk mimari standartlarında %100 Saf Win32 Native
olarak geliştirilmiştir. Microsoft Edge veya WebView2 gereksinimi tamamen kaldırılmıştır.
Gereksiz klasörler (EBWebView vb.) oluşturulmaz, RAM tüketimi ~15-20 MB seviyesindedir.

ANA PROGRAMLAR:
---------------
1. OxiDesk.exe:
   - IT Departmanı / Sistem Yöneticisi için saf Win32 yerel yönetim konsolu.
   - Doğrudan GDI ve donanım hızlandırmalı ekran aktarımı, anlık sohbet (neon uyarı bildirimli ve küçültülebilir mini dock),
     gelişmiş ayarlar menüsü (kayıt formatı, depolama yolları, kalite), görev yöneticisi ve dosya aktarımı.
   - İstenirse web/mobil tarayıcılardan doğrudan erişim için arka plan HTTP sunucusu da aktiftir.

2. OxiDesk-QuickSup.exe:
   - Son kullanıcılara tek tıkla dağıtılan süper hafif (~2.5 MB) taşınabilir Hızlı Destek aracı.
   - Kurulum gerektirmez, Edge gerektirmez, sıfır artık dosya bırakır.
   - Tek tıkla ID ve PIN üretir, gelen bağlantı onayını yönetir.

YARDIMCI VE SUNUCU ARAÇLARI (tools/ Klasörü):
----------------------------------------------
3. tools\oxidesk_rendezvous.exe:
   - Şirket sunucusunda veya bulutta çalıştırılabilen bağımsız WAN Sinyalleşme ve Relay Köprüsü.
   - NAT ve CGNAT arkasındaki istemcileri birbirine bağlar ve P2P eşleştirme sağlar.

4. tools\oxidesk_service.exe:
   - Windows Hizmeti (Unattended Service) yönetim aracı.
   - Oturum kapalıyken (Winlogon kilit ekranı) ve UAC korumalı pencerelerde tam yetkili uzaktan erişim sağlar.

GitHub: https://github.com/mtahiryilmazz/OxiDesk
"@

$kullanimPath = Join-Path $DistDir "KULLANIM_KILAVUZU.txt"
$utf8WithBom = New-Object System.Text.UTF8Encoding($true)
[System.IO.File]::WriteAllText($kullanimPath, $ReadmeContent, $utf8WithBom)

# 4. Tek Arşiv ZIP Paketi Oluşturma
$ZipPath = Join-Path $DistDir "OxiDesk-v5.0.0-Windows-x64.zip"
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}
$ItemsToZip = @(
    (Join-Path $DistDir "OxiDesk.exe"),
    (Join-Path $DistDir "OxiDesk-QuickSup.exe"),
    (Join-Path $DistDir "KULLANIM_KILAVUZU.txt"),
    $ToolsDir
)
Compress-Archive -Path $ItemsToZip -DestinationPath $ZipPath -Force

Write-Host "`n✅ Dagitim paketleri ve tek ZIP arsivi hazirlandi: $DistDir" -ForegroundColor Green
Get-ChildItem -Path $DistDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
