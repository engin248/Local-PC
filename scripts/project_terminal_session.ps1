param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot
)

$ErrorActionPreference = "Continue"

$GitDir = "C:\Program Files\Git\cmd"
$GitExe = Join-Path $GitDir "git.exe"
$NodeExeCandidates = @(
    "C:\Program Files\nodejs\node.exe",
    "$env:LOCALAPPDATA\nvm\nodejs\node.exe"
)
$NpmCmdCandidates = @(
    "C:\Program Files\nodejs\npm.cmd",
    "$env:APPDATA\npm\npm.cmd"
)
$CargoExeCandidates = @(
    "$env:USERPROFILE\.cargo\bin\cargo.exe"
)
$SqliteExeCandidates = @(
    "C:\Tools\SQLite\sqlite3.exe"
)
$NodeDirs = @(
    "C:\Program Files\nodejs",
    "$env:LOCALAPPDATA\nvm",
    "$env:APPDATA\npm"
)
$CargoDir = Join-Path $env:USERPROFILE ".cargo\bin"
$SqliteDir = "C:\Tools\SQLite"

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
if ($SafePathPrefix) {
    $env:PATH = "$SafePathPrefix;$env:PATH"
}

Set-Location -LiteralPath $ProjectRoot
Clear-Host

Write-Host "LOKAL BILGISAYAR KONTROL PANELI" -ForegroundColor Green
Write-Host "Dogru proje klasoru:" -ForegroundColor Cyan
Write-Host (Get-Location).Path -ForegroundColor White
Write-Host ""
Write-Host "Arac kontrolu:" -ForegroundColor Cyan

function Get-FirstExistingPath {
    param([string[]]$Candidates)

    foreach ($candidate in $Candidates) {
        if ($candidate -and (Test-Path -LiteralPath $candidate)) {
            return $candidate
        }
    }

    return $null
}

function Write-ToolVersion {
    param(
        [string]$Name,
        [string]$Path,
        [string[]]$Arguments = @("--version")
    )

    if (-not $Path) {
        Write-Host "$Name bulunamadi" -ForegroundColor Yellow
        return
    }

    Write-Host "$Name bulundu: $Path" -ForegroundColor DarkGray
    try {
        $output = & $Path @Arguments 2>$null
        if ($output) {
            $output | ForEach-Object { Write-Host $_ }
        }
    } catch {
        Write-Host "$Name versiyon kontrolu okunamadi; arac yolu mevcut." -ForegroundColor Yellow
    }
}

if (Test-Path -LiteralPath $GitExe) {
    Write-ToolVersion -Name "Git" -Path $GitExe
    try {
        $gitStatus = & $GitExe status --short 2>$null
        if ($gitStatus) {
            $gitStatus | ForEach-Object { Write-Host $_ }
        } else {
            Write-Host "Git calisma agaci temiz." -ForegroundColor Green
        }
    } catch {
        Write-Host "Git status okunamadi; terminal yine dogru proje klasorunde acildi." -ForegroundColor Yellow
    }
} else {
    Write-Host "Git bulunamadi: $GitExe" -ForegroundColor Yellow
}

$NodeExe = Get-FirstExistingPath $NodeExeCandidates
Write-ToolVersion -Name "Node" -Path $NodeExe

$NpmCmd = Get-FirstExistingPath $NpmCmdCandidates
Write-ToolVersion -Name "npm" -Path $NpmCmd

$CargoExe = Get-FirstExistingPath $CargoExeCandidates
Write-ToolVersion -Name "cargo" -Path $CargoExe

$SqliteExe = Get-FirstExistingPath $SqliteExeCandidates
Write-ToolVersion -Name "sqlite3" -Path $SqliteExe

Write-Host ""
Write-Host "Hazir. Bu terminal LOKAL BILGISAYAR KONTROL PANELI icin dogru terminaldir." -ForegroundColor Green
