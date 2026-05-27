$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$SessionScript = Join-Path $PSScriptRoot "project_terminal_session.ps1"
$WindowTitle = "LOKAL BILGISAYAR KONTROL PANELI - Dogru Terminal"
$MutexName = "Global\LokalBilgisayarKontrolPaneliDogruTerminalLauncher"
$LauncherMutex = [System.Threading.Mutex]::new($false, $MutexName)
$MutexAcquired = $false

try {
    $MutexAcquired = $LauncherMutex.WaitOne(5000)
    if (-not $MutexAcquired) {
        exit 0
    }

    if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "package.json"))) {
        throw "HATA: Dogru proje kokunde degiliz. package.json bulunamadi: $ProjectRoot"
    }

    if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot "src-tauri"))) {
        throw "HATA: Dogru proje kokunde degiliz. src-tauri bulunamadi: $ProjectRoot"
    }

    if (-not (Test-Path -LiteralPath $SessionScript)) {
        throw "HATA: Terminal oturum scripti bulunamadi: $SessionScript"
    }

    function Show-ExistingWindow {
        param([IntPtr]$Handle)

        if ($Handle -eq [IntPtr]::Zero) {
            return
        }

        $typeName = "LokalPanelTerminalWindow"
        if (-not ($typeName -as [type])) {
            Add-Type @"
using System;
using System.Runtime.InteropServices;

public static class LokalPanelTerminalWindow {
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
}
"@
        }

        [LokalPanelTerminalWindow]::ShowWindow($Handle, 9) | Out-Null
        [LokalPanelTerminalWindow]::SetForegroundWindow($Handle) | Out-Null
    }

    $ExistingTerminal = Get-Process powershell -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowTitle -eq $WindowTitle } |
        Sort-Object StartTime -Descending |
        Select-Object -First 1

    if ($ExistingTerminal) {
        Show-ExistingWindow -Handle $ExistingTerminal.MainWindowHandle
        exit 0
    }

    $ExistingTerminalsByCommandLine = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -eq "powershell.exe" -and
            $_.CommandLine -and
            $_.CommandLine.Contains($SessionScript) -and
            $_.CommandLine.Contains($ProjectRoot)
        } |
        Sort-Object CreationDate -Descending)

    if ($ExistingTerminalsByCommandLine.Count -gt 0) {
        $terminalToKeep = $ExistingTerminalsByCommandLine[0]
        $duplicateTerminals = $ExistingTerminalsByCommandLine | Select-Object -Skip 1

        foreach ($duplicate in $duplicateTerminals) {
            Stop-Process -Id $duplicate.ProcessId -Force -ErrorAction SilentlyContinue
        }

        $existingProcess = Get-Process -Id $terminalToKeep.ProcessId -ErrorAction SilentlyContinue
        if ($existingProcess) {
            Show-ExistingWindow -Handle $existingProcess.MainWindowHandle
        }
        exit 0
    }

    $ArgumentLine = @(
        "-NoExit",
        "-NoProfile",
        "-ExecutionPolicy Bypass",
        "-File `"$SessionScript`"",
        "-ProjectRoot `"$ProjectRoot`"",
        "-WindowTitle `"$WindowTitle`""
    ) -join " "

    Start-Process -FilePath "powershell.exe" -WorkingDirectory $ProjectRoot -ArgumentList $ArgumentLine
} finally {
    if ($MutexAcquired) {
        $LauncherMutex.ReleaseMutex()
    }
    $LauncherMutex.Dispose()
}
