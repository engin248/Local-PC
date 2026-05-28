$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$PanelUrl = "http://127.0.0.1:1420/"
$Port = 1420
$MutexName = "Global\LokalBilgisayarKontrolPaneliStartPanel"
$LauncherMutex = [System.Threading.Mutex]::new($false, $MutexName)
$MutexAcquired = $false

function Invoke-WmiQueryWithTimeout {
    param([string]$Filter)
    $ps = $null
    try {
        if ($env:LOKAL_PANEL_FORCE_WMI_TIMEOUT -eq "1") {
            return @()
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
            return @()
        }
        if ($ps.HadErrors) {
            return @()
        }
        return $ps.EndInvoke($asyncResult)
    } catch {
        return @()
    } finally {
        if ($ps) {
            $ps.Dispose()
        }
    }
}

try {
    $MutexAcquired = $LauncherMutex.WaitOne(5000)
    if (-not $MutexAcquired) {
        exit 0
    }

    if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
        throw "HATA: Dogru proje kokunde degiliz. package.json bulunamadi: $ProjectRoot"
    }

    $existingPort = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
        Where-Object { $_.State -eq "Listen" } |
        Select-Object -First 1

    $viteProcess = Invoke-WmiQueryWithTimeout -Filter "Name='node.exe' OR Name='cmd.exe' OR Name='powershell.exe' OR Name='pwsh.exe'" |
        Where-Object {
            $_.CommandLine -and
            $_.CommandLine.Contains($ProjectRoot) -and
            $_.CommandLine.Contains("vite") -and
            $_.CommandLine.Contains("127.0.0.1")
        } |
        Select-Object -First 1

    if ($existingPort -or $viteProcess) {
        exit 0
    }

    $command = "cd /d `"$ProjectRoot`" && npm run dev -- --host 127.0.0.1"
    Start-Process -FilePath "cmd.exe" -WindowStyle Minimized -ArgumentList @("/c", $command)

    $deadline = (Get-Date).AddSeconds(12)
    do {
        Start-Sleep -Milliseconds 500
        $started = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
            Where-Object { $_.State -eq "Listen" } |
            Select-Object -First 1
    } while (-not $started -and (Get-Date) -lt $deadline)

    if ($started) {
        Start-Process $PanelUrl
    }
} finally {
    if ($MutexAcquired) {
        $LauncherMutex.ReleaseMutex()
    }
    $LauncherMutex.Dispose()
}
