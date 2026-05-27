$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$SessionScript = Join-Path $PSScriptRoot "project_terminal_session.ps1"

if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
    throw "HATA: Dogru proje kokunde degiliz. package.json bulunamadi: $ProjectRoot"
}

if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "src-tauri"))) {
    throw "HATA: Dogru proje kokunde degiliz. src-tauri bulunamadi: $ProjectRoot"
}

if (-not (Test-Path -LiteralPath $SessionScript)) {
    throw "HATA: Terminal oturum scripti bulunamadi: $SessionScript"
}

$ArgumentLine = @(
    "-NoExit",
    "-NoProfile",
    "-ExecutionPolicy Bypass",
    "-File `"$SessionScript`"",
    "-ProjectRoot `"$ProjectRoot`""
) -join " "

Start-Process -FilePath "powershell.exe" -WorkingDirectory $ProjectRoot -ArgumentList $ArgumentLine
