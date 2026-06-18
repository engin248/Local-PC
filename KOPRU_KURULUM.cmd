@echo off
chcp 65001 >nul
title KOPRU KURULUM
cd /d "%~dp0"
echo ========================================
echo  LOKAL KOMUTA KOPRUSU - ILK KURULUM
echo ========================================
echo.
echo 1) Kopru sunucusu baslatiliyor (yeni pencere)...
start "KOPRU" cmd /k "%~dp0KOPRU_BASLAT.cmd"
timeout /t 3 /nobreak >nul
echo.
echo 2) Yol kontrolu (yerel)...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\yerel_yollar_kontrol.ps1"
echo.
echo 3) Cloud Agent erisimi icin (istege bagli):
echo    KOPRU_TUNEL_BASLAT.cmd calistirin
echo    Cikan https URL'yi config\kopru_bridge.json icine yazin
echo.
echo Token: config\kopru_bridge.json dosyasinda (ilk calistirmada olusur)
echo.
pause
