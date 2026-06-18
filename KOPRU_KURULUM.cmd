@echo off
chcp 65001 >nul
title KOPRU KURULUM (tunel yok)
cd /d "%~dp0"
echo ========================================
echo  YEREL KOPRU - TUNEL GEREKMEZ
echo ========================================
echo.
echo Cursor yerel Agent kullanacaksaniz:
echo   YEREL_HAZIR_BASLAT.cmd yeterli
echo.
start /min "KOPRU" cmd /c "powershell -NoProfile -ExecutionPolicy Bypass -File services\lokal_kopru\bridge.ps1"
timeout /t 2 /nobreak >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\yerel_yollar_kontrol.ps1"
echo.
echo Opsiyonel: her acilista kopru -> KOPRU_OTOMATIK_KAYIT.cmd
echo.
pause
