param(
    [string]$InstallDir = "C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI",
    [string]$ProjectRoot = ""
)

$ErrorActionPreference = "Continue"

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

$installedExe = Join-Path $InstallDir "lokal_bilgisayar_kontrol_paneli.exe"
$killed = 0

function Stop-ProcessTree {
    param([int]$ProcessId)

    $children = Get-CimInstance Win32_Process -Filter "ParentProcessId=$ProcessId" -ErrorAction SilentlyContinue
    foreach ($child in $children) {
        Stop-ProcessTree -ProcessId $child.ProcessId
    }

    try {
        Stop-Process -Id $ProcessId -Force -ErrorAction Stop
        $script:killed++
        Write-Host "Kapatildi PID $ProcessId"
    } catch {
        Write-Host "PID $ProcessId zaten kapali veya erisilemedi"
    }
}

Write-Host "=== ACIL PANEL KAPATMA ==="
Write-Host "Eski panel sureci kapatilir; F5/yenileme sesi durdurmaz — surec oldurulmali."
Write-Host ""

# 1) Tum kurulu panel exe surecleri (yol fark etmez)
Get-CimInstance Win32_Process -Filter "Name='lokal_bilgisayar_kontrol_paneli.exe'" -ErrorAction SilentlyContinue |
    ForEach-Object {
        Write-Host "Panel exe: PID $($_.ProcessId) — $($_.ExecutablePath)"
        Stop-ProcessTree -ProcessId $_.ProcessId
    }

# 2) Isimle (yedek)
Get-Process -Name "lokal_bilgisayar_kontrol_paneli" -ErrorAction SilentlyContinue |
    ForEach-Object { Stop-ProcessTree -ProcessId $_.Id }

# 3) Gelistirme modu: tauri dev / vite (proje klasoru)
Get-CimInstance Win32_Process -Filter "Name='node.exe' OR Name='cargo.exe'" -ErrorAction SilentlyContinue |
    Where-Object {
        $_.CommandLine -and
        $_.CommandLine.Contains($ProjectRoot) -and
        ($_.CommandLine -match "tauri|vite|lokal_bilgisayar")
    } |
    ForEach-Object {
        Write-Host "Dev sureci: PID $($_.ProcessId)"
        Stop-ProcessTree -ProcessId $_.ProcessId
    }

# 4) target\debug veya target\release altindan calisan kopyalar
$debugExe = Join-Path $ProjectRoot "src-tauri\target\debug\lokal_bilgisayar_kontrol_paneli.exe"
$releaseExe = Join-Path $ProjectRoot "src-tauri\target\release\lokal_bilgisayar_kontrol_paneli.exe"
foreach ($path in @($installedExe, $debugExe, $releaseExe)) {
    if (-not (Test-Path -LiteralPath $path)) { continue }
    $resolved = (Resolve-Path -LiteralPath $path).Path
    Get-CimInstance Win32_Process -Filter "Name='lokal_bilgisayar_kontrol_paneli.exe'" -ErrorAction SilentlyContinue |
        Where-Object { $_.ExecutablePath -eq $resolved } |
        ForEach-Object { Stop-ProcessTree -ProcessId $_.ProcessId }
}

Start-Sleep -Seconds 1

$remaining = @(Get-CimInstance Win32_Process -Filter "Name='lokal_bilgisayar_kontrol_paneli.exe'" -ErrorAction SilentlyContinue)
if ($remaining.Count -gt 0) {
    Write-Host ""
    Write-Host "UYARI: Hala acik panel sureci var. Gorev Yoneticisi'nden elle sonlandirin:"
    $remaining | ForEach-Object { Write-Host "  PID $($_.ProcessId) — $($_.ExecutablePath)" }
    exit 1
}

Write-Host ""
Write-Host "TAMAM: $killed surec kapatildi. Ses durdu."
Write-Host "Guncel surumu kurmak icin: .\KURULU_SURUMU_GUNCELLE.cmd"
exit 0
