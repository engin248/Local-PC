param(
    [string]$ProjectRoot = "",
    [string]$LauncherName = "DOGRU_TERMINAL_AC.vbs",
    [int]$LockTtlMinutes = 5
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$requestId = [guid]::NewGuid().Guid
if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

$projectLogRoot = Join-Path $env:LOCALAPPDATA "LokalBilgisayarKontrolPaneli"
$lockDir = Join-Path $projectLogRoot "locks"
$logDir = Join-Path $projectLogRoot "logs"
$lockPath = Join-Path $lockDir "terminal_open.lock"
$heartbeatPath = Join-Path $lockDir "terminal_session.heartbeat.json"
$inProgressPath = Join-Path $lockDir "terminal_open_in_progress.json"
$logPath = Join-Path $logDir "launcher.log"
$sessionScript = Join-Path $PSScriptRoot "project_terminal_session.ps1"
$sessionScriptLeaf = [System.IO.Path]::GetFileName($sessionScript)
$windowTitle = "LOKAL BILGISAYAR KONTROL PANELI - Terminal"
$devPort = 1420
$inProgressTtlSeconds = 20
$startupHeartbeatWaitSeconds = 10
$postStartConfirmSeconds = 10
$pollIntervalMs = 250
$heartbeatFreshSeconds = 45
$launcherMutexName = "Global\LokalBilgisayarKontrolPaneliTerminalLauncher"
$launcherMutex = [System.Threading.Mutex]::new($false, $launcherMutexName)
$launcherMutexAcquired = $false
$errorMessage = ""

function Ensure-Directory {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        $null = New-Item -ItemType Directory -Path $Path -Force
    }
}

function Write-LauncherLog {
    param(
        [string]$EventName,
        [string]$Message = "",
        [string]$Stage = "INFO"
    )
    Ensure-Directory -Path $logDir
    $line = [string]::Format(
        "{0:yyyy-MM-ddTHH:mm:ss.fffzzz} [{1}] request={2} event={3} {4}",
        (Get-Date),
        $Stage,
        $requestId,
        $EventName,
        $Message
    )
    Add-Content -LiteralPath $logPath -Value $line -Encoding UTF8
}

function Invoke-WmiQueryWithTimeout {
    param([string]$Filter)
    $ps = $null
    try {
        if ($env:LOKAL_PANEL_FORCE_WMI_TIMEOUT -eq "1") {
            throw "wmi_query_timeout"
        }
        $ps = [powershell]::Create()
        $null = $ps.AddCommand("Get-CimInstance").AddParameter("ClassName", "Win32_Process").AddParameter("Filter", $Filter).AddParameter("ErrorAction", "Stop")
        $asyncResult = $ps.BeginInvoke()
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        while (-not $asyncResult.IsCompleted -and $sw.ElapsedMilliseconds -lt 1500) {
            Start-Sleep -Milliseconds 10
        }
        if (-not $asyncResult.IsCompleted) {
            $ps.Stop()
            throw "wmi_query_timeout"
        }
        $result = $ps.EndInvoke($asyncResult)
        if ($ps.HadErrors) {
            throw "process_query_failed"
        }
        return $result
    } finally {
        if ($ps) {
            $ps.Dispose()
        }
    }
}

function Get-CimProcess {
    param([int]$ProcessId)
    try {
        return Invoke-WmiQueryWithTimeout -Filter "ProcessId = $ProcessId"
    } catch {
        Write-LauncherLog "wmi_optional_unavailable" "operation=get_process pid=$ProcessId reason=$($_.Exception.Message)" "WARN"
        return $null
    }
}

function Get-ProcessCommandLine {
    param([int]$ProcessId)
    $proc = Get-CimProcess -ProcessId $ProcessId
    if (-not $proc) { return $null }
    return $proc.CommandLine
}

function Is-ProcessAlive {
    param([int]$ProcessId)
    try {
        return [bool](Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
    } catch {
        return $false
    }
}

function Test-SameText {
    param([string]$Left, [string]$Right)
    return ([string]$Left) -ieq ([string]$Right)
}

function Test-ContainsText {
    param([string]$Text, [string]$Needle)
    if ([string]::IsNullOrWhiteSpace($Text) -or [string]::IsNullOrWhiteSpace($Needle)) {
        return $false
    }
    return ($Text.IndexOf($Needle, [System.StringComparison]::OrdinalIgnoreCase) -ge 0)
}

function Read-JsonFile {
    param([string]$Path, [string]$Kind)
    try {
        if (-not (Test-Path -LiteralPath $Path)) {
            return $null
        }
        $raw = Get-Content -Raw -LiteralPath $Path -ErrorAction Stop
        return $raw | ConvertFrom-Json -ErrorAction Stop
    } catch {
        Write-LauncherLog "${Kind}_json_parse_failed" "path=$Path reason=$($_.Exception.Message)" "WARN"
        return $null
    }
}

function Read-LockFile {
    return Read-JsonFile -Path $lockPath -Kind "lock"
}

function Read-HeartbeatState {
    return Read-JsonFile -Path $heartbeatPath -Kind "heartbeat"
}

function Is-HeartbeatFresh {
    param($Payload)
    if (-not $Payload -or -not $Payload.last_heartbeat) {
        return $false
    }
    try {
        $heartbeatAt = [DateTimeOffset]::Parse($Payload.last_heartbeat.ToString())
        return (((Get-Date) - $heartbeatAt.DateTime).TotalSeconds -le $heartbeatFreshSeconds)
    } catch {
        return $false
    }
}

function Is-HeartbeatForProject {
    param($Payload)
    if (-not (Is-HeartbeatFresh -Payload $Payload)) { return $false }
    if (-not $Payload.project_root -or -not (Test-SameText $Payload.project_root $ProjectRoot)) { return $false }
    if (-not $Payload.session_script -or -not (Test-SameText $Payload.session_script $sessionScriptLeaf)) { return $false }
    if (-not $Payload.pid) { return $false }
    try {
        return (Is-ProcessAlive -ProcessId ([int]$Payload.pid))
    } catch {
        return $false
    }
}

function Get-FreshProjectHeartbeat {
    $heartbeat = Read-HeartbeatState
    if (Is-HeartbeatForProject -Payload $heartbeat) {
        return $heartbeat
    }
    return $null
}

function Get-LockState {
    param($Lock)
    if (-not $Lock) { return "missing_lock" }
    if (-not $Lock.pid -or -not $Lock.created_at -or -not $Lock.project_root -or -not $Lock.launcher -or -not $Lock.session_script -or -not $Lock.session_token) {
        return "incomplete_lock"
    }
    if (-not (Test-SameText $Lock.project_root $ProjectRoot)) { return "different_project_root" }
    if (-not (Test-SameText $Lock.session_script $sessionScriptLeaf)) { return "lock_session_mismatch" }

    try { $lockPid = [int]$Lock.pid } catch { return "invalid_pid" }
    $heartbeat = Read-HeartbeatState
    if (Is-HeartbeatForProject -Payload $heartbeat) {
        if ([int]$heartbeat.pid -eq $lockPid -and $heartbeat.session_token -eq $Lock.session_token) {
            return "alive"
        }
        return "heartbeat_token_mismatch"
    }
    if (-not (Is-ProcessAlive -ProcessId $lockPid)) { return "process_not_alive" }

    $commandLine = Get-ProcessCommandLine -ProcessId $lockPid
    if ($commandLine -and (Test-ContainsText $commandLine $ProjectRoot) -and (Test-ContainsText $commandLine $sessionScriptLeaf)) {
        return "alive_without_fresh_heartbeat"
    }
    return "heartbeat_missing_or_stale"
}

function Write-LockFile {
    param(
        [int]$TargetPid,
        [string]$SessionToken,
        [string]$CommandLineSignature = "heartbeat"
    )
    $lockObject = @{
        pid = $TargetPid
        session_token = $SessionToken
        created_at = (Get-Date).ToString("o")
        project_root = $ProjectRoot
        launcher = $LauncherName
        session_script = $sessionScriptLeaf
        heartbeat_path = $heartbeatPath
        command_line_signature = $CommandLineSignature
    }
    $lockObject | ConvertTo-Json -Compress | Set-Content -LiteralPath $lockPath -Encoding UTF8
    return $lockObject
}

function Clear-Lock {
    if (Test-Path -LiteralPath $lockPath) {
        Remove-Item -LiteralPath $lockPath -Force -ErrorAction SilentlyContinue
        Write-LauncherLog "lock_cleanup" "removed_lock=$lockPath"
    }
}

function Remove-LockIfOwned {
    param([int]$ProcessId)
    $existing = Read-LockFile
    if (-not $existing) { return }
    try { $existingPid = [int]$existing.pid } catch { return }
    if ($existingPid -eq $ProcessId) {
        Clear-Lock
    }
}

function Is-StaleLock {
    param($Lock)
    if (-not $Lock) { return $true }
    $state = Get-LockState -Lock $Lock
    if ($state -eq "alive") { return $false }
    if ($state -eq "alive_without_fresh_heartbeat") {
        Write-LauncherLog "stale_lock_by_heartbeat" "state=$state pid=$($Lock.pid)"
        return $true
    }
    try {
        $createdAt = [DateTimeOffset]::Parse($Lock.created_at.ToString())
        if (((Get-Date) - $createdAt.DateTime).TotalMinutes -gt $LockTtlMinutes) {
            Write-LauncherLog "stale_lock_by_ttl" "created_at=$($Lock.created_at)"
            return $true
        }
    } catch {
        return $true
    }
    return $true
}

function Clean-StaleLock {
    $lock = Read-LockFile
    if (-not $lock) { return }
    if (Is-StaleLock -Lock $lock) {
        $state = Get-LockState -Lock $lock
        Write-LauncherLog "stale_lock_detected" "state=$state"
        Clear-Lock
    }
}

function Read-InProgress {
    return Read-JsonFile -Path $inProgressPath -Kind "in_progress"
}

function Set-InProgress {
    $payload = @{
        request_id = $requestId
        launcher = $LauncherName
        created_at = (Get-Date).ToString("o")
    }
    $payload | ConvertTo-Json -Compress | Set-Content -LiteralPath $inProgressPath -Encoding UTF8
}

function Clear-InProgress {
    if (Test-Path -LiteralPath $inProgressPath) {
        Remove-Item -LiteralPath $inProgressPath -Force -ErrorAction SilentlyContinue
    }
}

function Is-InProgressStale {
    param($Payload)
    if (-not $Payload -or -not $Payload.created_at) { return $true }
    try {
        $createdAt = [DateTimeOffset]::Parse($Payload.created_at.ToString())
        return (((Get-Date) - $createdAt.DateTime).TotalSeconds -gt $inProgressTtlSeconds)
    } catch {
        return $true
    }
}

function Is-SessionCommandLine {
    param([string]$CommandLine)
    if ([string]::IsNullOrWhiteSpace($CommandLine)) { return $false }
    return (Test-ContainsText $CommandLine $ProjectRoot) -and (Test-ContainsText $CommandLine $sessionScriptLeaf)
}

function Get-ProjectSessionProcesses {
    try {
        $procs = Invoke-WmiQueryWithTimeout -Filter "Name='powershell.exe' OR Name='pwsh.exe'"
        if (-not $procs) { return @() }
        return $procs | Where-Object { $_.CommandLine -and (Is-SessionCommandLine -CommandLine $_.CommandLine) }
    } catch {
        Write-LauncherLog "wmi_optional_unavailable" "operation=session_scan reason=$($_.Exception.Message)" "WARN"
        return @()
    }
}

function Is-PanelListening {
    try {
        $existing = Get-NetTCPConnection -LocalPort $devPort -ErrorAction SilentlyContinue |
            Where-Object { $_.State -eq "Listen" } |
            Select-Object -First 1
        return [bool]$existing
    } catch {
        return $false
    }
}

function Resolve-SessionCommandLine {
    param([int]$TargetPid)
    $commandLine = Get-ProcessCommandLine -ProcessId $TargetPid
    if (-not $commandLine) { return $null }
    if (-not (Is-SessionCommandLine -CommandLine $commandLine)) { return $null }
    return $commandLine
}

function Wait-ForHeartbeat {
    param(
        [string]$ExpectedToken,
        [int]$TimeoutSeconds
    )
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        $heartbeat = Read-HeartbeatState
        if ((Is-HeartbeatForProject -Payload $heartbeat) -and $heartbeat.session_token -eq $ExpectedToken) {
            return $heartbeat
        }
        Start-Sleep -Milliseconds $pollIntervalMs
    } while ((Get-Date) -lt $deadline)
    return $null
}

function Quote-ProcessArg {
    param([string]$Value)
    return '"' + ($Value -replace '"', '\"') + '"'
}

try {
    Ensure-Directory -Path $lockDir
    Ensure-Directory -Path $logDir

    Write-LauncherLog "launcher_request_received" "launcher=$LauncherName project_root=$ProjectRoot"
    Write-LauncherLog "lock_acquire_attempt" "lock=$lockPath"

    $launcherMutexAcquired = $launcherMutex.WaitOne(15000)
    if (-not $launcherMutexAcquired) {
        Write-LauncherLog "launcher_mutex_timeout" "mutex=$launcherMutexName" "WARN"
        $freshHeartbeat = Get-FreshProjectHeartbeat
        if ($freshHeartbeat) {
            Write-LauncherLog "existing_session_found" "pid=$($freshHeartbeat.pid) heartbeat_without_mutex"
            exit 0
        }
        throw "launcher_mutex_timeout"
    }

    if (-not (Test-Path -LiteralPath $sessionScript)) {
        throw "Session script not found: $sessionScript"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
        throw "Project root validation failed: package.json not found."
    }

    $freshHeartbeat = Get-FreshProjectHeartbeat
    if ($freshHeartbeat) {
        $lock = Write-LockFile -TargetPid ([int]$freshHeartbeat.pid) -SessionToken $freshHeartbeat.session_token -CommandLineSignature "heartbeat"
        Write-LauncherLog "existing_session_found" "pid=$($freshHeartbeat.pid) reused_from_heartbeat lock=$($lock.pid)"
        Clear-InProgress
        exit 0
    }

    $inProgress = Read-InProgress
    if ($inProgress -and -not (Is-InProgressStale -Payload $inProgress)) {
        Write-LauncherLog "terminal_open_in_progress" "request_id=$($inProgress.request_id)"
        Start-Sleep -Seconds 2
        $freshHeartbeat = Get-FreshProjectHeartbeat
        if ($freshHeartbeat) {
            Write-LauncherLog "existing_session_found" "pid=$($freshHeartbeat.pid) heartbeat_after_in_progress"
            exit 0
        }
        throw "Another open operation is in progress."
    }
    Set-InProgress

    Clean-StaleLock

    $existingLock = Read-LockFile
    if ($existingLock) {
        $lockState = Get-LockState -Lock $existingLock
        if ($lockState -eq "alive") {
            Write-LauncherLog "existing_session_found" "pid=$($existingLock.pid) launcher=$($existingLock.launcher)"
            throw "existing_session_found"
        }
        Write-LauncherLog "stale_lock_removed" "state=$lockState"
        Clear-Lock
    }

    $existingSessions = @()
    foreach ($candidate in Get-ProjectSessionProcesses) {
        if (Is-SessionCommandLine -CommandLine $candidate.CommandLine) {
            $existingSessions += $candidate
        }
    }

    if ($existingSessions.Count -gt 0) {
        $existing = $existingSessions | Sort-Object ProcessId | Select-Object -First 1
        $cmd = Resolve-SessionCommandLine -TargetPid ([int]$existing.ProcessId)
        if (-not $cmd) {
            $cmd = "wmi_scan"
        }
        $reusedToken = "wmi-reused-$([guid]::NewGuid().Guid)"
        $lock = Write-LockFile -TargetPid ([int]$existing.ProcessId) -SessionToken $reusedToken -CommandLineSignature $cmd
        Write-LauncherLog "session_pid_detected" "pid=$($existing.ProcessId)"
        Write-LauncherLog "existing_session_found" "pid=$($existing.ProcessId) reused_from_scan lock=$($lock.pid)"
        Clear-InProgress
        exit 0
    }

    Write-LauncherLog "session_start_requested" "starting project_terminal_session.ps1"
    $terminalArgs = @(
        "-NoProfile",
        "-ExecutionPolicy Bypass",
        "-WindowStyle Normal",
        "-WorkingDirectory $(Quote-ProcessArg $ProjectRoot)",
        "-NoExit",
        "-File $(Quote-ProcessArg $sessionScript)",
        "-ProjectRoot $(Quote-ProcessArg $ProjectRoot)",
        "-WindowTitle $(Quote-ProcessArg $windowTitle)",
        "-LauncherRequestId $(Quote-ProcessArg $requestId)"
    ) -join " "
    $startedProcess = Start-Process -FilePath "powershell.exe" -ArgumentList $terminalArgs -PassThru -ErrorAction Stop

    if (-not $startedProcess -or $startedProcess.HasExited) {
        throw "start_process_failed"
    }

    $heartbeat = Wait-ForHeartbeat -ExpectedToken $requestId -TimeoutSeconds $startupHeartbeatWaitSeconds
    if (-not $heartbeat) {
        if ($startedProcess.HasExited) {
            throw "terminal_session_exited_early"
        }
        throw "session_heartbeat_not_confirmed"
    }

    $sessionPid = [int]$heartbeat.pid
    $sessionCmd = Resolve-SessionCommandLine -TargetPid $sessionPid
    if (-not $sessionCmd) {
        $sessionCmd = "heartbeat"
    }
    $lock = Write-LockFile -TargetPid $sessionPid -SessionToken $requestId -CommandLineSignature $sessionCmd
    Write-LauncherLog "session_pid_detected" "pid=$sessionPid lock=$($lock.pid)"

    Start-Sleep -Seconds $postStartConfirmSeconds
    $heartbeat = Read-HeartbeatState
    if (-not (Is-ProcessAlive -ProcessId $sessionPid)) {
        Remove-LockIfOwned -ProcessId $sessionPid
        throw "terminal_session_exited_early"
    }
    if (-not ((Is-HeartbeatForProject -Payload $heartbeat) -and $heartbeat.session_token -eq $requestId)) {
        Remove-LockIfOwned -ProcessId $sessionPid
        throw "session_heartbeat_lost_after_start"
    }

    Write-LauncherLog "session_alive_confirmed" "session_pid=$sessionPid panel_listening=$([bool](Is-PanelListening))"
    Write-LauncherLog "launcher_request_completed" "session_pid=$sessionPid"
    exit 0
} catch {
    $errorMessage = $_.Exception.Message
    if ($errorMessage -eq "existing_session_found") {
        Write-LauncherLog "existing_session_found" "returning_existing"
        exit 0
    }
    Write-LauncherLog "session_start_failed" $errorMessage "ERROR"
    exit 1
} finally {
    Clear-InProgress
    if ($launcherMutexAcquired) {
        $launcherMutex.ReleaseMutex()
    }
    $launcherMutex.Dispose()
    if ($errorMessage -eq "terminal_session_exited_early") {
        if (Test-Path -LiteralPath $lockPath) {
            Clear-Lock
        }
    }
}
