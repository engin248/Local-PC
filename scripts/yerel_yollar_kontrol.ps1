# Yolları kontrol et (Windows — komutan veya çocuk bir kez çalıştırır)
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
$configPath = Join-Path $root "config\yerel_veri_yollari.json"
if (-not (Test-Path -LiteralPath $configPath)) {
    Write-Host "HATA: yerel_veri_yollari.json yok" -ForegroundColor Red
    exit 1
}
$paths = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json

function Test-Yol {
    param([string]$Label, [string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        Write-Host "[ATLA] $Label" -ForegroundColor DarkGray
        return
    }
    if (Test-Path -LiteralPath $Path) {
        Write-Host "[VAR]  $Label" -ForegroundColor Green
        Write-Host "       $Path"
    } else {
        Write-Host "[YOK]  $Label" -ForegroundColor Yellow
        Write-Host "       $Path"
    }
}

Write-Host "`n=== YEREL VERI YOLLARI KONTROLU ===`n" -ForegroundColor Cyan
Test-Yol "Panel proje" $paths.panel_proje
Test-Yol "Kurulu exe" $paths.kurulu_exe
Test-Yol "UZMAN_HAVUZU.json" $paths.uzman_havuzu_json
Test-Yol "skill_library.sqlite (ana)" $paths.skill_library_sqlite
foreach ($alt in $paths.skill_library_yedek_yollar) {
    Test-Yol "skill_library (yedek)" $alt
}
Test-Yol "Asker Motoru kok" $paths.asker_motoru_kok
Write-Host "`nBitti.`n"
