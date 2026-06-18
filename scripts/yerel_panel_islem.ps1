# Yerel panel işlemleri — tünel yok, köprü şart değil
# Cursor Yerel Agent veya çocuk bu scripti çalıştırır.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("yol_kontrol", "kurulu_guncelle", "panel_kapat", "kopru_durum")]
    [string]$Islem,
    [string]$ProjectRoot = ""
)

$ErrorActionPreference = "Stop"
if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

switch ($Islem) {
    "yol_kontrol" {
        & (Join-Path $PSScriptRoot "yerel_yollar_kontrol.ps1")
    }
    "kurulu_guncelle" {
        & (Join-Path $PSScriptRoot "update_installed_exe.ps1") -ProjectRoot $ProjectRoot -SkipPull
    }
    "panel_kapat" {
        & (Join-Path $PSScriptRoot "stop_panel_processes.ps1") -ProjectRoot $ProjectRoot
    }
    "kopru_durum" {
        $paths = Join-Path $ProjectRoot "config\yerel_veri_yollari.json"
        $bridge = Join-Path $ProjectRoot "storage\kopru\bridge_running.flag"
        @{
            ok = $true
            project_root = $ProjectRoot
            bridge_flag = (Test-Path -LiteralPath $bridge)
            paths_config = (Test-Path -LiteralPath $paths)
        } | ConvertTo-Json
    }
}
