<#
.SYNOPSIS
    ApexConnect Kurumsal Dağıtım Paketi Oluşturucu
.DESCRIPTION
    Hem IT Yönetim Paketi (apex_connect.exe) hem de Son Kullanıcı Mini Hızlı Destek (apex_qs.exe)
    dosyalarını optimize release olarak derler ve dist/ klasörüne kopyalar.
#>

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
Set-Location $RepoRoot

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "ApexConnect - Kurumsal Dagitim Paketi Olusturucu" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Release derlemeleri
Write-Host "`nPaketler optimize ediliyor ve derleniyor..." -ForegroundColor Cyan
& cargo build --release --bin apex_connect --bin apex_qs --bin apex_rendezvous --bin apex_service
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
$AdminSrc = Join-Path $RepoRoot "target\release\apex_connect.exe"
$AdminDst = Join-Path $DistDir "ApexConnect.exe"
Copy-Item -Path $AdminSrc -Destination $AdminDst -Force

$QsSrc = Join-Path $RepoRoot "target\release\apex_qs.exe"
$QsDst = Join-Path $DistDir "ApexQuickSupport.exe"
Copy-Item -Path $QsSrc -Destination $QsDst -Force

# Yardımcı ve Sunucu Araçları (tools/ alt dizininde)
$RendezvousSrc = Join-Path $RepoRoot "target\release\apex_rendezvous.exe"
$RendezvousDst = Join-Path $ToolsDir "apex_rendezvous.exe"
Copy-Item -Path $RendezvousSrc -Destination $RendezvousDst -Force

$ServiceSrc = Join-Path $RepoRoot "target\release\apex_service.exe"
$ServiceDst = Join-Path $ToolsDir "apex_service.exe"
Copy-Item -Path $ServiceSrc -Destination $ServiceDst -Force

# 3. Bilgilendirme Notu
$ReadmeContent = @"
ApexConnect Kurumsal Uzak Masaüstü Dağıtım Paketi (v2.0.0)
==========================================================

ANA PROGRAMLAR:
---------------
1. ApexConnect.exe:
   - IT Departmanı / Sistem Yöneticisi için tam yetkili masaüstü yönetim konsolu.
   - Ağ tarayıcısı, adres defteri, dosya aktarımı, denetim logları ve WinUI 3 Fluent arayüz içerir.

2. ApexQuickSupport.exe:
   - Şirket personeline dağıtılacak taşınabilir (portable) Mini Hızlı Destek aracı.
   - Kurulum gerektirmez, açıldığı anda 9 haneli Apex ID üretir ve IT masasına otomatik SOS anonsu gönderir.

YARDIMCI VE SUNUCU ARAÇLARI (tools/ Klasörü):
----------------------------------------------
3. tools\apex_rendezvous.exe:
   - Şirket sunucusunda veya bulutta çalıştırılabilen bağımsız WAN Sinyalleşme ve Relay Köprüsü.
   - İnternet üzerindeki NAT ve CGNAT arkasındaki istemcileri birbirine bağlar ve P2P eşleştirme sağlar.

4. tools\apex_service.exe:
   - Windows Hizmeti (Unattended Service) yönetim aracı.
   - Oturum kapalıyken (Winlogon kilit ekranı) ve UAC korumalı pencerelerde tam yetkili uzaktan erişim sağlar.

GitHub: https://github.com/mtahiryilmazz/ApexConnect
"@

$kullanimPath = Join-Path $DistDir "KULLANIM_KILAVUZU.txt"
$utf8WithBom = New-Object System.Text.UTF8Encoding($true)
[System.IO.File]::WriteAllText($kullanimPath, $ReadmeContent, $utf8WithBom)

# 4. Tek Arşiv ZIP Paketi Oluşturma
$ZipPath = Join-Path $DistDir "ApexConnect-v2.0.0-Windows-x64.zip"
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}
$ItemsToZip = @(
    (Join-Path $DistDir "ApexConnect.exe"),
    (Join-Path $DistDir "ApexQuickSupport.exe"),
    (Join-Path $DistDir "KULLANIM_KILAVUZU.txt"),
    $ToolsDir
)
Compress-Archive -Path $ItemsToZip -DestinationPath $ZipPath -Force

Write-Host "`n✅ Dagitim paketleri ve tek ZIP arsivi hazirlandi: $DistDir" -ForegroundColor Green
Get-ChildItem -Path $DistDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
