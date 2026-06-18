# Onaylı tüm işlemler — Windows PC (komutan / çocuk tek sefer)
# Cloud Agent bu scripti uzaktan çalıştıramaz; sizin PC'nizde çift tık gerekir.
param(
    [string]$ProjectRoot = "",
    [string]$Branch = "master",
    [switch]$SkipPull,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

$installDir = "C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI"
$installedExe = Join-Path $installDir "lokal_bilgisayar_kontrol_paneli.exe"

function Speak-EmelHatirlatma {
    param([string]$Message)
    try {
        Add-Type -AssemblyName System.Speech
        $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
        $voices = $synth.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo }
        $turkish = $voices | Where-Object { $_.Culture.Name -like "tr-*" }
        $female = $turkish | Where-Object { $_.Gender -eq "Female" } | Select-Object -First 1
        if ($female) { $synth.SelectVoice($female.Name) }
        elseif ($turkish) { $synth.SelectVoice(($turkish | Select-Object -First 1).Name) }
        $synth.Rate = -1
        $synth.Volume = 100
        $synth.Speak($Message)
        $synth.Dispose()
    } catch {
        Write-Host "Ses hatirlatmasi atlandi: $_" -ForegroundColor DarkGray
    }
}

Set-Location -LiteralPath $ProjectRoot

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host " ONAYLI TUM ISLEMLER — LOKAL PANEL" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 1 — Git pull
if (-not $SkipPull) {
    Write-Host "[1/5] Git guncelleniyor (origin/$Branch)..." -ForegroundColor Yellow
    git fetch origin $Branch
    $currentBranch = (git rev-parse --abbrev-ref HEAD 2>$null)
    if ($currentBranch -ne $Branch) {
        git checkout $Branch
    }
    git pull --ff-only origin $Branch
    Write-Host "      Git tamam." -ForegroundColor Green
} else {
    Write-Host "[1/5] Git atlandi (-SkipPull)." -ForegroundColor DarkGray
}

# 2 — Yol kontrolu
Write-Host "[2/5] Veri yollari kontrol ediliyor..." -ForegroundColor Yellow
& (Join-Path $PSScriptRoot "yerel_yollar_kontrol.ps1")

# 3 — Kopru inbox/outbox (yerel, tunel yok)
Write-Host "[3/5] Yerel hazirlik..." -ForegroundColor Yellow
$kopruIn = Join-Path $ProjectRoot "storage\kopru\inbox"
$kopruOut = Join-Path $ProjectRoot "storage\kopru\outbox"
foreach ($d in @($kopruIn, $kopruOut)) {
    if (-not (Test-Path -LiteralPath $d)) {
        New-Item -ItemType Directory -Path $d -Force | Out-Null
    }
}
"ok" | Set-Content -LiteralPath (Join-Path $ProjectRoot "storage\kopru\bridge_running.flag") -Encoding ASCII

# 4 — Kurulu exe guncelle
if (-not $SkipBuild) {
    Write-Host "[4/5] Kurulu panel guncelleniyor (derleme 5-15 dk)..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "update_installed_exe.ps1") `
        -ProjectRoot $ProjectRoot `
        -SkipPull `
        -InstallDir $installDir
} else {
    Write-Host "[4/5] Derleme atlandi (-SkipBuild)." -ForegroundColor DarkGray
    & (Join-Path $PSScriptRoot "stop_panel_processes.ps1") -InstallDir $installDir -ProjectRoot $ProjectRoot
}

# 5 — Panel ac + Emel hatirlatma
Write-Host "[5/5] Panel baslatiliyor..." -ForegroundColor Yellow
if (-not (Test-Path -LiteralPath $installedExe)) {
    throw "Kurulu exe yok: $installedExe — once derleme tamamlanmali."
}
Start-Process -FilePath $installedExe

Start-Sleep -Seconds 3
Speak-EmelHatirlatma "Komutan, panel acildi. Yarbay Emel sekmesinde Emel'i Baslat dugmesine bir kez tiklayin."

Write-Host ""
Write-Host "TAMAM." -ForegroundColor Green
Write-Host "  Panel: $installedExe"
Write-Host "  Son adim: Panelde YARBAY EMEL -> Emel'i Baslat (bir tik)"
Write-Host ""

@{
    ok = $true
    git_head = (git rev-parse --short HEAD 2>$null)
    installed_exe = $installedExe
    emel = "panelde_emel_i_baslat_gerekli"
} | ConvertTo-Json
