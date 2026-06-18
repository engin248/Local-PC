@echo off
chcp 65001 >nul
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\yerel_panel_islem.ps1" -Islem %1
if errorlevel 1 pause
