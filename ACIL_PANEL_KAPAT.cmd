@echo off
setlocal
cd /d "%~dp0"
echo.
echo ========================================
echo  ACIL PANEL KAPAT — SESI DURDUR
echo ========================================
echo  F5 veya yenileme sesi durdurmaz.
echo  Eski panel sureci olduruluyor...
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\stop_panel_processes.ps1"
echo.
pause
