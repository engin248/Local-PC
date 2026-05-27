$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$GitDir = "C:\Program Files\Git\cmd"
$GitExe = Join-Path $GitDir "git.exe"
$NodeDirs = @(
    "C:\Program Files\nodejs",
    "$env:LOCALAPPDATA\nvm",
    "$env:APPDATA\npm"
)
$CargoDir = Join-Path $env:USERPROFILE ".cargo\bin"
$SqliteDir = "C:\Tools\SQLite"

if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
    throw "HATA: Dogru proje kokunde degiliz. package.json bulunamadi: $ProjectRoot"
}

if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "src-tauri"))) {
    throw "HATA: Dogru proje kokunde degiliz. src-tauri bulunamadi: $ProjectRoot"
}

$PathParts = @()
if (Test-Path -LiteralPath $GitDir) {
    $PathParts += $GitDir
}
foreach ($nodeDir in $NodeDirs) {
    if ($nodeDir -and (Test-Path -LiteralPath $nodeDir)) {
        $PathParts += $nodeDir
    }
}
if (Test-Path -LiteralPath $CargoDir) {
    $PathParts += $CargoDir
}
if (Test-Path -LiteralPath $SqliteDir) {
    $PathParts += $SqliteDir
}

$SafePathPrefix = ($PathParts | Select-Object -Unique) -join ";"

$InnerScript = @"
`$ErrorActionPreference = "Continue"
Set-Location -LiteralPath "$ProjectRoot"
`$env:PATH = "$SafePathPrefix;" + `$env:PATH
Clear-Host
Write-Host "LOKAL BILGISAYAR KONTROL PANELI" -ForegroundColor Green
Write-Host "Dogru proje klasoru:" -ForegroundColor Cyan
Write-Host (Get-Location).Path -ForegroundColor White
Write-Host ""
Write-Host "Arac kontrolu:" -ForegroundColor Cyan
if (Test-Path -LiteralPath "$GitExe") {
    & "$GitExe" --version
    & "$GitExe" status --short
} else {
    Write-Host "Git bulunamadi: $GitExe" -ForegroundColor Yellow
}
if (Get-Command node -ErrorAction SilentlyContinue) { node --version } else { Write-Host "Node bulunamadi" -ForegroundColor Yellow }
if (Get-Command npm -ErrorAction SilentlyContinue) { npm --version } else { Write-Host "npm bulunamadi" -ForegroundColor Yellow }
if (Get-Command cargo -ErrorAction SilentlyContinue) { cargo --version } else { Write-Host "cargo bulunamadi" -ForegroundColor Yellow }
if (Get-Command sqlite3 -ErrorAction SilentlyContinue) { sqlite3 --version } else { Write-Host "sqlite3 bulunamadi" -ForegroundColor Yellow }
Write-Host ""
Write-Host "Hazir. Bu terminal LOKAL BILGISAYAR KONTROL PANELI icin dogru terminaldir." -ForegroundColor Green
"@

$Encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($InnerScript))

$WindowsTerminal = Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps\wt.exe"
if (Test-Path -LiteralPath $WindowsTerminal) {
    Start-Process -FilePath $WindowsTerminal -ArgumentList @(
        "new-tab",
        "--title",
        "LOKAL BILGISAYAR KONTROL PANELI",
        "powershell.exe",
        "-NoExit",
        "-NoProfile",
        "-EncodedCommand",
        $Encoded
    )
} else {
    Start-Process -FilePath "powershell.exe" -ArgumentList @(
        "-NoExit",
        "-NoProfile",
        "-EncodedCommand",
        $Encoded
    )
}
