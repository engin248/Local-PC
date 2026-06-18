@echo off
chcp 65001 >nul
title LOKAL KOMUTA KOPRUSU
cd /d "%~dp0"
echo.
echo  Kopru baslatiliyor (pencere acik kalsin)...
echo  Port: 19200  —  Kapatmak icin bu pencereyi kapatin.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0services\lokal_kopru\bridge.ps1"
pause
