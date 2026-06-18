@echo off
chcp 65001 >nul
title API KEY KAYDET
cd /d "%~dp0"

if not exist "scripts\api_key_kaydet.ps1" (
  echo  HATA: scripts\api_key_kaydet.ps1 bulunamadi. Once "kurulu guncelle" yapin.
  echo.
  pause
  exit /b 1
)

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\api_key_kaydet.ps1"

pause
