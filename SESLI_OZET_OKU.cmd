@echo off
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -Command "Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Rate=-1; $s.Volume=100; $t=Get-Content -LiteralPath '%~dp0scripts\emel_sesli_ozet.txt' -Raw -Encoding UTF8; $s.Speak($t); $s.Dispose()"
pause
