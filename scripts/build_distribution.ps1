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
& cargo build --release --bin apex_connect --bin apex_qs --bin apex_rendezvous
if ($LASTEXITCODE -ne 0) {
    Write-Error "Release derlemesi basarisiz oldu!"
}

# 2. dist/ dizini olusturma ve kopyalama
$DistDir = Join-Path $RepoRoot "dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}

$AdminSrc = Join-Path $RepoRoot "target\release\apex_connect.exe"
$AdminDst = Join-Path $DistDir "apex_connect.exe"
Copy-Item -Path $AdminSrc -Destination $AdminDst -Force

$QsSrc = Join-Path $RepoRoot "target\release\apex_qs.exe"
$QsDst = Join-Path $DistDir "apex_qs.exe"
Copy-Item -Path $QsSrc -Destination $QsDst -Force

$RendezvousSrc = Join-Path $RepoRoot "target\release\apex_rendezvous.exe"
$RendezvousDst = Join-Path $DistDir "apex_rendezvous.exe"
Copy-Item -Path $RendezvousSrc -Destination $RendezvousDst -Force

# 3. Bilgilendirme Notu
$ReadmeContent = @"
ApexConnect Kurumsal Uzak Masaüstü Dağıtım Paketi (v1.2.0)
==========================================================

1. apex_connect.exe:
   - IT Departmanı / Bilgi İşlem personeli için tam yetkili yönetim konsolu.
   - Ağ tarayıcısı, adres defteri, dosya aktarımı ve WinUI 3 Fluent arayüz içerir.

2. apex_qs.exe:
   - Şirket çalışanlarına dağıtılacak taşınabilir (portable) Mini Hızlı Destek aracı.
   - Kurulum gerektirmez, açıldığı anda 9 haneli Apex ID üretir ve IT masasına otomatik SOS anonsu gönderir.

3. apex_rendezvous.exe:
   - Şirket sunucusunda veya bulutta çalıştırılabilen bağımsız WAN Sinyalleşme ve Relay Köprüsü.
   - İnternet üzerindeki NAT ve CGNAT arkasındaki istemcileri birbirine bağlar ve P2P eşleştirme sağlar.

GitHub: https://github.com/mtahiryilmazz/ApexConnect
"@

Set-Content -Path (Join-Path $DistDir "KULLANIM_KILAVUZU.txt") -Value $ReadmeContent -Encoding UTF8

Write-Host "`n✅ Dagitim paketleri hazirlandi: $DistDir" -ForegroundColor Green
Get-ChildItem -Path $DistDir | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
