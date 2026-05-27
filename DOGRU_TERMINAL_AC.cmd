@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "VBS_SCRIPT=%SCRIPT_DIR%DOGRU_TERMINAL_AC.vbs"

if not exist "%VBS_SCRIPT%" (
  echo HATA: Terminal acma scripti bulunamadi:
  echo %VBS_SCRIPT%
  pause
  exit /b 1
)

wscript.exe "%VBS_SCRIPT%"

endlocal
