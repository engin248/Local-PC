# Yarbay Emel — Windows ses hattı (ayrı TCP sunucu YOK; System.Speech + panel)
param(
    [string]$ProjectRoot = "",
    [switch]$PanelOnly,
    [switch]$TestOnly
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

$installDir = "C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI"
$installedExe = Join-Path $installDir "lokal_bilgisayar_kontrol_paneli.exe"
$statusDir = Join-Path $ProjectRoot "storage\ses_sunucusu"
$statusFile = Join-Path $statusDir "durum.json"

New-Item -ItemType Directory -Path $statusDir -Force | Out-Null

function Write-SesDurum {
    param([hashtable]$Data)
    $payload = @{
        ok = $true
        mod = "windows_system_speech"
        not = "Ayri ses sunucusu yok — WebView2 speechSynthesis panel icinde"
        updated_at = (Get-Date).ToString("o")
    }
    foreach ($k in $Data.Keys) { $payload[$k] = $Data[$k] }
    $payload | ConvertTo-Json | Set-Content -LiteralPath $statusFile -Encoding UTF8
}

function Start-EmelSesHatti {
    Add-Type -AssemblyName System.Speech
    $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
    $turkish = $synth.GetInstalledVoices() |
        Where-Object { $_.VoiceInfo.Culture.Name -like "tr-*" } |
        Select-Object -First 1
    if ($turkish) {
        $synth.SelectVoice($turkish.VoiceInfo.Name)
        $voiceName = $turkish.VoiceInfo.Name
    } else {
        $voiceName = "varsayilan"
    }
    $synth.Rate = -1
    $synth.Volume = 100
    $phrase = "Yarbay Emel Hanım görevde. Ses hattı açıldı. Komutanım, panelde Emel'i Başlat düğmesine bir kez tıklayın."
    $synth.Speak($phrase)
    $synth.Dispose()
    return $voiceName
}

Write-Host ""
Write-Host "=== SES HATTI BASLATILIYOR ===" -ForegroundColor Cyan
Write-Host ""

try {
    $voiceName = Start-EmelSesHatti
    Write-SesDurum @{
        durum = "hazir"
        voice = $voiceName
        panel = $installedExe
    }
    Write-Host "Ses hatti: HAZIR ($voiceName)" -ForegroundColor Green
} catch {
    Write-SesDurum @{ durum = "hata"; error = $_.Exception.Message }
    Write-Host "Ses hatti HATA: $($_.Exception.Message)" -ForegroundColor Red
    throw
}

if ($TestOnly) {
    Write-Host "TestOnly — panel acilmadi."
    exit 0
}

if (-not $PanelOnly) {
    if (Test-Path -LiteralPath $installedExe) {
        Write-Host "Panel aciliyor: $installedExe"
        Start-Process -FilePath $installedExe
    } else {
        Write-Host "UYARI: Kurulu exe yok. Once ONAYLI_TUM_ISLEMLER.cmd calistirin." -ForegroundColor Yellow
        Write-Host "       Beklenen: $installedExe"
    }
}

Write-Host ""
Write-Host "Panel acilinca: YARBAY EMEL -> Emel'i Baslat (bir tik)" -ForegroundColor Yellow
Write-Host "Durum dosyasi: $statusFile"
Write-Host ""
