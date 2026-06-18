@echo off
chcp 65001 >nul
title KOPRU TUNEL (Cloud erisimi)
cd /d "%~dp0"
echo Once KOPRU_BASLAT.cmd acik olmali.
echo.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0services\lokal_kopru\tunnel_cloudflared.ps1"
pause
