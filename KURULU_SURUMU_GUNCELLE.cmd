@echo off
setlocal
cd /d "%~dp0"
echo LOKAL BILGISAYAR KONTROL PANELI - Kurulu surum guncelleme
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\update_installed_exe.ps1"
if errorlevel 1 (
  echo.
  echo HATA: Guncelleme basarisiz.
  pause
  exit /b 1
)
echo.
pause
