@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "START_SCRIPT=%SCRIPT_DIR%scripts\start_panel_singleton.ps1"

if not exist "%START_SCRIPT%" (
  echo HATA: Panel acma scripti bulunamadi:
  echo %START_SCRIPT%
  exit /b 1
)

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%START_SCRIPT%"

endlocal
