$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$textFile = Join-Path $scriptDir "gece_devri_emel_oku.txt"

if (-not (Test-Path -LiteralPath $textFile)) {
    throw "Metin dosyasi bulunamadi: $textFile"
}

$text = Get-Content -LiteralPath $textFile -Raw -Encoding UTF8
if ([string]::IsNullOrWhiteSpace($text)) {
    throw "Okunacak metin bos."
}

Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer

$voices = $synth.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo }
$turkish = $voices | Where-Object { $_.Culture.Name -like "tr-*" }
$female = $turkish | Where-Object { $_.Gender -eq "Female" } | Select-Object -First 1
$pick = if ($female) { $female } elseif ($turkish) { $turkish | Select-Object -First 1 } else { $null }

if ($pick) {
    $synth.SelectVoice($pick.Name)
    Write-Host "Ses: $($pick.Name)"
} else {
    Write-Host "UYARI: Turkce ses bulunamadi, varsayilan ses kullaniliyor."
}

$synth.Rate = -2
$synth.Volume = 100

Write-Host ""
Write-Host "Yarbay Emel Hanim raporu seslendiriyor..."
Write-Host ""

$synth.Speak($text)
$synth.Dispose()

Write-Host ""
Write-Host "Rapor tamamlandi. Iyi geceler."
