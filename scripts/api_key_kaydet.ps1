# API key'leri terminalden sorup .env dosyasina yazar.
# Komutan: cift tiklayinca sorulan yere key'i yapistir + Enter.
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$envPath = Join-Path $root '.env'
$examplePath = Join-Path $root '.env.example'

if (Test-Path $envPath) {
    $lines = @(Get-Content -Path $envPath)
} elseif (Test-Path $examplePath) {
    $lines = @(Get-Content -Path $examplePath)
} else {
    $lines = @('GEMINI_API_KEY=', 'CURSOR_API_KEY=')
}

Write-Host ''
Write-Host '================================================'
Write-Host '  API KEY KAYDETME (Gemini / Cursor)'
Write-Host '================================================'
Write-Host '  Key''inizi yapistirin (Ctrl+V) ve Enter''a basin.'
Write-Host '  Bos birakip Enter = o key''i atla / degistirme.'
Write-Host '------------------------------------------------'
Write-Host ''

$gemini = Read-Host 'Gemini API key'
$cursor = Read-Host 'Cursor API key'

function Set-EnvLine {
    param([string[]]$Lines, [string]$Key, [string]$Value)
    if ([string]::IsNullOrWhiteSpace($Value)) { return $Lines }
    $Value = $Value.Trim()
    $found = $false
    $out = foreach ($l in $Lines) {
        if ($l -match "^\s*#?\s*$Key\s*=") {
            $found = $true
            "$Key=$Value"
        } else {
            $l
        }
    }
    if (-not $found) { $out = @($out) + "$Key=$Value" }
    return $out
}

$lines = Set-EnvLine -Lines $lines -Key 'GEMINI_API_KEY' -Value $gemini
$lines = Set-EnvLine -Lines $lines -Key 'CURSOR_API_KEY' -Value $cursor

# BOM'suz UTF-8 yaz (Rust yukleyici ilk satiri dogru okusun diye).
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllLines($envPath, $lines, $utf8NoBom)

Write-Host ''
Write-Host "KAYDEDILDI -> $envPath"
if (-not [string]::IsNullOrWhiteSpace($gemini)) { Write-Host '  GEMINI_API_KEY: yazildi' }
if (-not [string]::IsNullOrWhiteSpace($cursor)) { Write-Host '  CURSOR_API_KEY: yazildi' }
Write-Host ''
Write-Host 'Simdi paneli yeniden baslatin. AI Provider Health -> API Key: present'
Write-Host ''
