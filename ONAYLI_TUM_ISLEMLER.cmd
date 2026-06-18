@echo off
chcp 65001 >nul
title ONAYLI TUM ISLEMLER
color 0A
echo.
echo  ====================================================
echo   ONAYLI TUM ISLEMLER
echo   git pull + yol kontrol + kurulu guncelle + panel ac
echo  ====================================================
echo.
echo  Bu dosyayi SIZIN Windows PC'nizde cift tiklayin.
echo  Cloud Agent bu adimi uzaktan yapamaz.
echo.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\onayli_tum_islemler.ps1"
if errorlevel 1 (
  echo.
  echo  HATA. Once ACIL_PANEL_KAPAT.cmd deneyin.
  pause
  exit /b 1
)
echo.
pause
