@echo off
setlocal
cd /d "%~dp0.."
echo.
echo ========================================
echo  YARBAY EMEL HANIM — GECe DEVRI RAPORU
echo ========================================
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0GECE_DEVRI_RAPOR_OKU.ps1"
if errorlevel 1 (
  echo.
  echo PowerShell sesi calismadi. WAV dosyasini deneyin:
  echo   scripts\audio\gece_devri_raporu_tr.wav
  start "" "%~dp0audio\gece_devri_raporu_tr.wav" 2>nul
)
echo.
pause
