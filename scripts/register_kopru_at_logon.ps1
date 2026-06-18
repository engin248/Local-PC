# Windows acilista kopru otomatik baslat (tunel yok)
param([string]$ProjectRoot = "")

$ErrorActionPreference = "Stop"
if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
}

$taskName = "LokalPanelKopru"
$cmd = "powershell.exe"
$args = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$ProjectRoot\services\lokal_kopru\bridge.ps1`" -ProjectRoot `"$ProjectRoot`""

$action = New-ScheduledTaskAction -Execute $cmd -Argument $args -WorkingDirectory $ProjectRoot
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Settings $settings -Force | Out-Null

Write-Host "Gorev kaydedildi: $taskName (her oturum acilisinda kopru)" -ForegroundColor Green
