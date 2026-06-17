param(
    [string]$ProjectRoot = "",
    [string]$InstallDir = "C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI",
    [string]$Branch = "master",
    [switch]$SkipPull,
    [switch]$SkipVerify,
    [switch]$DirectCopy
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

function Stop-PanelProcesses {
    param([string]$ExePath)

    Get-Process -Name "lokal_bilgisayar_kontrol_paneli" -ErrorAction SilentlyContinue |
        ForEach-Object {
            Write-Host "Kapatiliyor: PID $($_.Id)"
            Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        }

    if (Test-Path -LiteralPath $ExePath) {
        $resolved = (Resolve-Path -LiteralPath $ExePath).Path
        Get-CimInstance Win32_Process -Filter "Name='lokal_bilgisayar_kontrol_paneli.exe'" -ErrorAction SilentlyContinue |
            Where-Object { $_.ExecutablePath -and $_.ExecutablePath -eq $resolved } |
            ForEach-Object {
                Write-Host "Kapatiliyor (kurulu yol): PID $($_.ProcessId)"
                Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue
            }
    }

    Start-Sleep -Seconds 1
}

function Backup-InstalledExe {
    param([string]$InstalledExe)

    if (-not (Test-Path -LiteralPath $InstalledExe)) {
        return $null
    }

    $stamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $backup = "$InstalledExe.pre_update_$stamp"
    Copy-Item -LiteralPath $InstalledExe -Destination $backup -Force
    Write-Host "Yedek: $backup"
    return $backup
}

if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
    throw "Proje kokunde package.json yok: $ProjectRoot"
}

Set-Location -LiteralPath $ProjectRoot

if (-not $SkipPull) {
    Write-Host "Git guncelleniyor: origin/$Branch"
    git fetch origin $Branch
    $currentBranch = (git rev-parse --abbrev-ref HEAD 2>$null)
    if ($currentBranch -ne $Branch) {
        git checkout $Branch
    }
    git pull --ff-only origin $Branch
}

Write-Host "Bagimliliklar kuruluyor..."
npm install

Write-Host "Release build baslatiliyor (npm run tauri build)..."
npm run tauri build

$targetExe = Join-Path $ProjectRoot "src-tauri\target\release\lokal_bilgisayar_kontrol_paneli.exe"
$nsisInstaller = Join-Path $ProjectRoot "src-tauri\target\release\bundle\nsis\LOKAL BILGISAYAR KONTROL PANELI_0.1.0_x64-setup.exe"
$installedExe = Join-Path $InstallDir "lokal_bilgisayar_kontrol_paneli.exe"

if (-not (Test-Path -LiteralPath $targetExe)) {
    throw "Build cikti exe bulunamadi: $targetExe"
}

Stop-PanelProcesses -ExePath $installedExe

$useInstaller = -not $DirectCopy

if ($useInstaller) {
    if (-not (Test-Path -LiteralPath $nsisInstaller)) {
        throw "NSIS installer bulunamadi: $nsisInstaller"
    }

    Write-Host "NSIS sessiz kurulum calistiriliyor (onerilen yontem)..."
    $installProc = Start-Process -FilePath $nsisInstaller -ArgumentList "/S" -Wait -PassThru
    if ($installProc.ExitCode -ne 0) {
        throw "NSIS kurulumu basarisiz. ExitCode=$($installProc.ExitCode)"
    }
    Start-Sleep -Seconds 2
} else {
    if (-not (Test-Path -LiteralPath $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $null = Backup-InstalledExe -InstalledExe $installedExe
    Copy-Item -LiteralPath $targetExe -Destination $installedExe -Force
    Write-Host "Kurulu exe dogrudan kopyalandi: $installedExe"
}

if (-not $SkipVerify) {
    & (Join-Path $PSScriptRoot "verify_installed_release.ps1") `
        -ProjectRoot $ProjectRoot `
        -InstallDir $InstallDir
} else {
    Write-Host "Dogrulama atlandi (-SkipVerify)."
}

Write-Host ""
Write-Host "TAMAM: Kurulu surum guncellendi."
Write-Host "  Yol: $installedExe"
Write-Host "  Gelistirme modu icin: npm run tauri dev"
