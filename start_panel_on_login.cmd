@echo off
setlocal

set "PROJECT_DIR=C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli"
set "PANEL_URL=http://127.0.0.1:1420/"

powershell.exe -NoProfile -ExecutionPolicy Bypass -Command ^
  "$existing = Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue; ^
   if (-not $existing) { ^
     Start-Process -FilePath 'cmd.exe' -WindowStyle Minimized -ArgumentList '/c', 'cd /d ""%PROJECT_DIR%"" && npm run dev -- --host 127.0.0.1'; ^
   }"

timeout /t 5 /nobreak >nul
start "" "%PANEL_URL%"

endlocal
