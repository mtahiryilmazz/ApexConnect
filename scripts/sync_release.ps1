param (
    [string]$Bump = "none",
    [string]$Message = ""
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
Set-Location $RepoRoot

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "ApexConnect - Otomatik Surum ve GitHub Dagitim Araci" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Cargo.toml surum kontrolu
$CargoTomlPath = Join-Path $RepoRoot "Cargo.toml"
$CargoLines = Get-Content $CargoTomlPath
$CurrentVersion = "1.0.0"

for ($i = 0; $i -lt $CargoLines.Count; $i++) {
    if ($CargoLines[$i] -match '^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"') {
        $Major = [int]$Matches[1]
        $Minor = [int]$Matches[2]
        $Patch = [int]$Matches[3]
        $CurrentVersion = "$Major.$Minor.$Patch"

        if ($Bump -eq "patch") {
            $Patch++
            $NewVersion = "$Major.$Minor.$Patch"
            $CargoLines[$i] = "version = ""$NewVersion"""
            Set-Content -Path $CargoTomlPath -Value $CargoLines
            Write-Host "Surum yukseltildi: v$CurrentVersion -> v$NewVersion" -ForegroundColor Green
            $CurrentVersion = $NewVersion
        } elseif ($Bump -eq "minor") {
            $Minor++
            $Patch = 0
            $NewVersion = "$Major.$Minor.$Patch"
            $CargoLines[$i] = "version = ""$NewVersion"""
            Set-Content -Path $CargoTomlPath -Value $CargoLines
            Write-Host "Surum yukseltildi: v$CurrentVersion -> v$NewVersion" -ForegroundColor Green
            $CurrentVersion = $NewVersion
        } elseif ($Bump -eq "major") {
            $Major++
            $Minor = 0
            $Patch = 0
            $NewVersion = "$Major.$Minor.$Patch"
            $CargoLines[$i] = "version = ""$NewVersion"""
            Set-Content -Path $CargoTomlPath -Value $CargoLines
            Write-Host "Surum yukseltildi: v$CurrentVersion -> v$NewVersion" -ForegroundColor Green
            $CurrentVersion = $NewVersion
        }
        break
    }
}

Write-Host "Mevcut Surum: v$CurrentVersion" -ForegroundColor Yellow

# 2. Test ve Derleme
Write-Host "`nBirim testleri calistiriliyor..." -ForegroundColor Cyan
& cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Error "Birim testleri basarisiz oldu!"
}

Write-Host "`nRelease derlemesi olusturuluyor (apex_connect & apex_qs)..." -ForegroundColor Cyan
& cargo build --release --bin apex_connect --bin apex_qs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Release derlemesi basarisiz oldu!"
}

# 3. Git Senkronizasyonu
Write-Host "`nGit durumu kontrol ediliyor..." -ForegroundColor Cyan
& git add -A

$status = (& git status --porcelain)
if (-not $status) {
    Write-Host "Yeni degisiklik yok. Calisma dizini temiz." -ForegroundColor Yellow
} else {
    $commitMsg = $Message
    if ([string]::IsNullOrWhiteSpace($commitMsg)) {
        $commitMsg = "chore(release): v$CurrentVersion - ApexConnect otomatik guncelleme"
    }

    Write-Host "Commit olusturuluyor: '$commitMsg'..." -ForegroundColor Cyan
    & git commit -m "$commitMsg"

    Write-Host "GitHub uzak deposuna push ediliyor..." -ForegroundColor Cyan
    & git push origin main
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Basariyla GitHub'a yuklendi!" -ForegroundColor Green
    } else {
        Write-Warning "Git push basarisiz oldu veya ag hatasi olustu."
    }
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host "Islem tamamlandi! ApexConnect v$CurrentVersion hazir." -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
