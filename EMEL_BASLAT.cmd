@echo off
cd /d "%~dp0"
echo Yarbay Emel Hanim — Windows ses motoru (panodan veya dosyadan)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\GECE_DEVRI_RAPOR_OKU.ps1"
pause
