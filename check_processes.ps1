$processes = Get-Process | Where-Object { $_.ProcessName -like "*tauri*" -or $_.ProcessName -like "*node*" -or $_.ProcessName -like "*cargo*" -or $_.ProcessName -like "*npm*" }
if ($processes) {
    $processes | Select-Object Id, ProcessName, MainWindowTitle | Format-Table -AutoSize
} else {
    Write-Host "No Tauri, Node, Cargo, or NPM processes found running."
}
