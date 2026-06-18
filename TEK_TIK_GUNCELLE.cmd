@echo off
chcp 65001 >nul
title LOKAL PANEL - TEK TIK GUNCELLEME
color 0E
echo.
echo  ================================================
echo   KURULU PANEL GUNCELLENIYOR
echo   (Git yok - sadece yerel derleme)
echo  ================================================
echo.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\update_installed_exe.ps1" -SkipPull
if errorlevel 1 (
  echo.
  echo  HATA oldu. ACIL_PANEL_KAPAT.cmd deneyin sonra tekrar.
  pause
  exit /b 1
)
echo.
echo  TAMAM - Panel guncellendi.
echo  Simdi paneli acin, YARBAY EMEL sekmesinde Emel'i Baslat.
echo.
pause
