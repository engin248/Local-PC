@echo off
chcp 65001 >nul
title API KEY KAYDET
cd /d "%~dp0"

echo.
echo  ================================================
echo   API KEY KAYDETME (Gemini / Cursor)
echo  ================================================
echo.

if not exist ".env.example" (
  echo  HATA: .env.example bulunamadi. Once "kurulu guncelle" yapin.
  echo.
  pause
  exit /b 1
)

if exist ".env" (
  echo  .env dosyasi zaten var. Not Defteri ile aciliyor...
) else (
  copy /y ".env.example" ".env" >nul
  echo  .env dosyasi olusturuldu. Not Defteri ile aciliyor...
)

echo.
echo  ------------------------------------------------
echo   YAPMANIZ GEREKEN:
echo   1) Acilan Not Defteri penceresinde
echo      GEMINI_API_KEY= satirinin SONUNA keyinizi yapistirin.
echo   2) CURSOR_API_KEY= satirinin SONUNA keyinizi yapistirin.
echo   3) Ctrl+S ile KAYDEDIN, pencereyi kapatin.
echo  ------------------------------------------------
echo.

start "" notepad ".env"

echo  Kaydettikten sonra paneli yeniden baslatin.
echo.
pause
