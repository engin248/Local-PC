@echo off
setlocal
cd /d "%~dp0"
echo.
echo ========================================
echo  ACIL SES KES + KURULU SURUM GUNCELLE
echo ========================================
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\stop_panel_processes.ps1"
if errorlevel 1 (
  echo Panel kapatilamadi. Gorev Yoneticisi'nden elle sonlandirin, sonra tekrar deneyin.
  pause
  exit /b 1
)
echo.
echo Kurulu surum guncelleniyor...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\update_installed_exe.ps1"
if errorlevel 1 (
  echo Guncelleme basarisiz.
  pause
  exit /b 1
)
echo.
echo Yeni panel baslatiliyor...
start "" "%LOCALAPPDATA%\LOKAL BILGISAYAR KONTROL PANELI\lokal_bilgisayar_kontrol_paneli.exe"
echo.
pause
