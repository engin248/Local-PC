@echo off
cd /d "%~dp0"
echo Yarbay Emel Hanım — gece devri raporu
git pull origin master 2>nul
if exist "scripts\audio\gece_devri_raporu_tr.wav" (
  start "" "scripts\audio\gece_devri_raporu_tr.wav"
) else (
  powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\GECE_DEVRI_RAPOR_OKU.ps1"
)
pause
