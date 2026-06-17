param(
    [string]$Text = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Text)) {
    $Text = Get-Clipboard -TextFormatType Text
}

if ([string]::IsNullOrWhiteSpace($Text)) {
    Write-Host "Okunacak metin yok. Metni parametre olarak verin veya panoya kopyalayin."
    exit 1
}

Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer

$voice = $synth.GetInstalledVoices() |
    Where-Object { $_.VoiceInfo.Culture.Name -like "tr-*" } |
    Select-Object -First 1

if ($voice) {
    $synth.SelectVoice($voice.VoiceInfo.Name)
}

$synth.Rate = -1
$synth.Volume = 100
Write-Host "Yarbay Emel Hanım seslendiriyor..."
$synth.Speak($Text)
$synth.Dispose()
Write-Host "Tamam."
