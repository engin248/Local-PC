@echo off
setlocal
cd /d "%~dp0"
echo LOKAL BILGISAYAR KONTROL PANELI - Gelistirme modu (tauri dev)
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\tauri_dev.ps1"
pause
