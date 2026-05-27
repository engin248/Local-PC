@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "OPEN_SCRIPT=%SCRIPT_DIR%scripts\open_correct_terminal.ps1"

if not exist "%OPEN_SCRIPT%" (
  echo HATA: Terminal acma scripti bulunamadi:
  echo %OPEN_SCRIPT%
  pause
  exit /b 1
)

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%OPEN_SCRIPT%"

endlocal
