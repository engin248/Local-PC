param(
    [string]$ProjectRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
    throw "Proje kokunde package.json yok: $ProjectRoot"
}

Set-Location -LiteralPath $ProjectRoot

Write-Host "Gelistirme modu: npm run tauri dev"
Write-Host "Vite: http://localhost:200/ | Tauri penceresi acilacak"
Write-Host "Durdurmak icin bu pencerede Ctrl+C"

npm run tauri dev
