@echo off
chcp 65001 >nul
title YEREL SISTEM HAZIR
cd /d "%~dp0"
echo.
echo  Yerel kopru ve dosya kuyrugu baslatiliyor...
echo  (Tunel GEREKMEZ — Cursor yerel Agent dogrudan calisir)
echo.

if not exist "storage\kopru\inbox" mkdir "storage\kopru\inbox"
if not exist "storage\kopru\outbox" mkdir "storage\kopru\outbox"
echo ok > "storage\kopru\bridge_running.flag"

start /min "KOPRU" cmd /c "powershell -NoProfile -ExecutionPolicy Bypass -File services\lokal_kopru\bridge.ps1"

timeout /t 2 /nobreak >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\yerel_panel_islem.ps1" -Islem yol_kontrol

echo.
echo  HAZIR. Cursor'da bu klasoru acin, Agent modunda yazin:
echo  "kurulu guncelle" veya "yol kontrol"
echo.
pause
