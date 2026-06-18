@echo off
chcp 65001 >nul
title SES HATTI — Yarbay Emel
color 0B
echo.
echo  ==========================================
echo   SES HATTI BASLAT (Windows System.Speech)
echo   Ayri ag sunucusu YOK — panel icinde TTS
echo  ==========================================
echo.
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\ses_sunucusu.ps1"
if errorlevel 1 pause
echo.
echo  Tamam. Bu pencereyi kapatabilirsiniz; panel ayri acik kalir.
pause
exit /b 0
