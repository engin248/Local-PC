# Cloudflare quick tunnel — kopru'yu internete acar (tek seferlik URL)
param(
    [string]$ProjectRoot = "",
    [int]$Port = 19200
)

$ErrorActionPreference = "Stop"
if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")).Path
}

$configPath = Join-Path $ProjectRoot "config\kopru_bridge.json"
$cf = Get-Command cloudflared -ErrorAction SilentlyContinue
if (-not $cf) {
    $binDir = Join-Path $ProjectRoot "storage\kopru\bin"
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    $cfExe = Join-Path $binDir "cloudflared.exe"
    if (-not (Test-Path -LiteralPath $cfExe)) {
        Write-Host "cloudflared indiriliyor..." -ForegroundColor Cyan
        $url = "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe"
        Invoke-WebRequest -Uri $url -OutFile $cfExe -UseBasicParsing
    }
    $cf = $cfExe
}

Write-Host "Tunel baslatiliyor: http://127.0.0.1:$Port" -ForegroundColor Cyan
Write-Host "Asagida https://....trycloudflare.com URL gorunecek." -ForegroundColor Yellow
Write-Host "Bu URL'yi config/kopru_bridge.json -> tunnel_public_url alanina yapistirin." -ForegroundColor Yellow
Write-Host ""

& $cf tunnel --url "http://127.0.0.1:$Port"
