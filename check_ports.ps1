$connections = Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object { $_.LocalPort -eq 1420 -or $_.LocalPort -eq 8081 -or $_.LocalPort -eq 8082 -or $_.LocalPort -eq 8083 -or $_.LocalPort -eq 8084 -or $_.LocalPort -eq 8085 }
if ($connections) {
    $connections | Select-Object LocalAddress, LocalPort, OwningProcess, State | Format-Table -AutoSize
} else {
    Write-Host "No active connections on ports 1420, 8081-8085 found."
    Write-Host "Listing all listening ports on localhost (127.0.0.1 or 0.0.0.0):"
    Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object { $_.LocalAddress -eq "127.0.0.1" -or $_.LocalAddress -eq "0.0.0.0" } | Select-Object LocalAddress, LocalPort, OwningProcess, State | Sort-Object LocalPort | Format-Table -AutoSize
}
