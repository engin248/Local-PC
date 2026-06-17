@echo off
setlocal
cd /d "%~dp0.."
echo Gece devri raporu — Yarbay Emel Hanım seslendiriyor...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
  "$t = Get-Content -LiteralPath '%~dp0gece_devri_emel_oku.txt' -Raw -Encoding UTF8; Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Rate = -1; $s.Speak($t); $s.Dispose()"
echo.
echo Rapor tamamlandi. Iyi geceler.
timeout /t 3 >nul
