@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR:~0,-1%"
set "SESSION_SCRIPT=%SCRIPT_DIR%scripts\project_terminal_session.ps1"

if not exist "%SESSION_SCRIPT%" (
  echo HATA: Terminal oturum scripti bulunamadi:
  echo %SESSION_SCRIPT%
  pause
  exit /b 1
)

powershell.exe -NoExit -NoProfile -ExecutionPolicy Bypass -File "%SESSION_SCRIPT%" -ProjectRoot "%PROJECT_DIR%"

endlocal
