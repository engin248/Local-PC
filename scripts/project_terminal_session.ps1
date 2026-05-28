param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot,

    [string]$LauncherRequestId = "",

    [string]$WindowTitle = "LOKAL BILGISAYAR KONTROL PANELI - Dogru Terminal"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$ProjectSignatureRoot = Join-Path $env:LOCALAPPDATA "LokalBilgisayarKontrolPaneli"
$LogDir = Join-Path $ProjectSignatureRoot "logs"
$LockDir = Join-Path $ProjectSignatureRoot "locks"
$LogPath = Join-Path $LogDir "launcher.log"
$LockPath = Join-Path $LockDir "terminal_open.lock"
$HeartbeatPath = Join-Path $LockDir "terminal_session.heartbeat.json"
$SessionPid = $PID
$SingletonScript = Join-Path $PSScriptRoot "start_panel_singleton.ps1"

$NoExitMessage = "Bu pencere acildiktan sonra kapatilmadikca acik kalir."

try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8
} catch {
    # Encoding setup is best effort; log events stay ASCII.
}

function Ensure-Directory {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        $null = New-Item -ItemType Directory -Path $Path -Force
    }
}

function Write-LauncherLog {
    param(
        [string]$EventName,
        [string]$Message = ""
    )
    Ensure-Directory -Path $LogDir
    $line = [string]::Format(
        "{0:yyyy-MM-ddTHH:mm:ss.fffzzz} [INFO] request={1} event={2} {3}",
        (Get-Date),
        $LauncherRequestId,
        $EventName,
        $Message
    )
    Add-Content -LiteralPath $LogPath -Value $line -Encoding UTF8
}

function Write-Heartbeat {
    Ensure-Directory -Path $LockDir
    $payload = @{
        session_token = $LauncherRequestId
        pid = $SessionPid
        request_id = $LauncherRequestId
        project_root = $ProjectRoot
        last_heartbeat = (Get-Date).ToString("o")
        process_name = (Get-Process -Id $SessionPid -ErrorAction SilentlyContinue).ProcessName
        session_script = "project_terminal_session.ps1"
        launched_by = "open_correct_terminal.ps1"
    }
    $payload | ConvertTo-Json -Compress | Set-Content -LiteralPath $HeartbeatPath -Encoding UTF8
}

function Start-Frontend {
    if (Test-Path -LiteralPath $SingletonScript) {
        try {
            & $SingletonScript
            Write-LauncherLog "terminal_session_log" "frontend_startup_requested"
        } catch {
            Write-LauncherLog "terminal_session_log" "frontend_startup_failed"
        }
    } else {
        Write-LauncherLog "terminal_session_log" "singleton_script_missing=$SingletonScript"
    }
}

function Clear-OwnLock {
    try {
        if (-not (Test-Path -LiteralPath $LockPath)) {
            return
        }
        $lockRaw = Get-Content -Raw -LiteralPath $LockPath -ErrorAction SilentlyContinue
        if (-not $lockRaw) {
            return
        }
        $lock = $lockRaw | ConvertFrom-Json -ErrorAction SilentlyContinue
        if ($lock -and [int]$lock.pid -eq $SessionPid) {
            Remove-Item -LiteralPath $LockPath -Force -ErrorAction SilentlyContinue
            Write-LauncherLog "lock_cleanup" "session_closed pid=$SessionPid"
        }
    } catch {
        Write-LauncherLog "lock_cleanup_error" $_.Exception.Message
    }
}

try {
    Ensure-Directory -Path $LogDir
    Write-LauncherLog "session_started" "pid=$SessionPid request=$LauncherRequestId root=$ProjectRoot"
    Write-Heartbeat
    Write-LauncherLog "initial_heartbeat_written" "pid=$SessionPid"
    try {
        $Host.UI.RawUI.WindowTitle = $WindowTitle
    } catch {
        Write-LauncherLog "session_window_title_warning" $_.Exception.Message
    }

    Set-Location -LiteralPath $ProjectRoot
    Clear-Host

    Write-Host "LOKAL BILGISAYAR KONTROL PANELI" -ForegroundColor Green
    Write-Host "Proje klasoru: $ProjectRoot"
    Write-Host "Terminal Session PID: $SessionPid"
    if ($LauncherRequestId) {
        Write-Host "Launcher Request ID: $LauncherRequestId"
    }
    Write-Host ""
    Write-Host $NoExitMessage -ForegroundColor Yellow

    Start-Frontend

    Write-LauncherLog "session_ready"

    while ($true) {
        Write-Heartbeat
        Write-LauncherLog "heartbeat_written" "pid=$SessionPid"
        Start-Sleep -Seconds 5
    }
} finally {
    Clear-OwnLock
    Write-LauncherLog "session_closed"
}
