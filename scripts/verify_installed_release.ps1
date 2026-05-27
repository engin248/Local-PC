param(
    [string]$ProjectRoot = "C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli",
    [string]$InstallDir = "C:\Users\Esisya\AppData\Local\LOKAL BILGISAYAR KONTROL PANELI"
)

$ErrorActionPreference = "Stop"

function Get-RequiredItem {
    param([string]$Path, [string]$Label)

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "$Label bulunamadi: $Path"
    }

    Get-Item -LiteralPath $Path
}

function Get-Sha256 {
    param([string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
}

function Get-DifferentByteCount {
    param([string]$Left, [string]$Right)

    $leftBytes = [System.IO.File]::ReadAllBytes($Left)
    $rightBytes = [System.IO.File]::ReadAllBytes($Right)

    if ($leftBytes.Length -ne $rightBytes.Length) {
        return [Math]::Abs($leftBytes.Length - $rightBytes.Length) + [Math]::Min($leftBytes.Length, $rightBytes.Length)
    }

    $count = 0
    for ($i = 0; $i -lt $leftBytes.Length; $i++) {
        if ($leftBytes[$i] -ne $rightBytes[$i]) {
            $count++
        }
    }

    $count
}

function Test-BinaryMarker {
    param([string]$Path, [string]$Marker)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    $text.Contains($Marker)
}

$targetExe = Join-Path $ProjectRoot "src-tauri\target\release\lokal_bilgisayar_kontrol_paneli.exe"
$nsisInstaller = Join-Path $ProjectRoot "src-tauri\target\release\bundle\nsis\LOKAL BILGISAYAR KONTROL PANELI_0.1.0_x64-setup.exe"
$msiInstaller = Join-Path $ProjectRoot "src-tauri\target\release\bundle\msi\LOKAL BILGISAYAR KONTROL PANELI_0.1.0_x64_en-US.msi"
$installedExe = Join-Path $InstallDir "lokal_bilgisayar_kontrol_paneli.exe"

$targetItem = Get-RequiredItem $targetExe "Target release exe"
$installedItem = Get-RequiredItem $installedExe "Kurulu exe"
$nsisItem = Get-RequiredItem $nsisInstaller "NSIS installer"
$msiItem = Get-RequiredItem $msiInstaller "MSI installer"

$installedVersion = $installedItem.VersionInfo
$targetVersion = $targetItem.VersionInfo
$differentByteCount = Get-DifferentByteCount $installedItem.FullName $targetItem.FullName

$requiredMarkers = @(
    "CREATE TABLE IF NOT EXISTS operation_steps",
    "CREATE TABLE IF NOT EXISTS operation_monitor_logs",
    "CREATE TABLE IF NOT EXISTS principle_evaluations",
    "COUNT(DISTINCT approver_id)",
    "selected_best_option_reason",
    "accepted_correct_approach_reason"
)

$missingMarkers = @()
foreach ($marker in $requiredMarkers) {
    if (-not (Test-BinaryMarker $installedItem.FullName $marker)) {
        $missingMarkers += $marker
    }
}

$result = [PSCustomObject]@{
    InstalledExe = $installedItem.FullName
    InstalledExeLength = $installedItem.Length
    InstalledExeSha256 = Get-Sha256 $installedItem.FullName
    TargetReleaseExe = $targetItem.FullName
    TargetReleaseExeLength = $targetItem.Length
    TargetReleaseExeSha256 = Get-Sha256 $targetItem.FullName
    DifferentByteCount = $differentByteCount
    InstalledProductName = $installedVersion.ProductName
    TargetProductName = $targetVersion.ProductName
    InstalledProductVersion = $installedVersion.ProductVersion
    TargetProductVersion = $targetVersion.ProductVersion
    NsisInstaller = $nsisItem.FullName
    NsisInstallerSha256 = Get-Sha256 $nsisItem.FullName
    MsiInstaller = $msiItem.FullName
    MsiInstallerSha256 = Get-Sha256 $msiItem.FullName
    RequiredMarkersMissing = $missingMarkers
}

$result | Format-List

if ($installedItem.Length -ne $targetItem.Length) {
    throw "Kurulu exe ve target/release exe boyutu farkli. Bu beklenen paketleme farkindan daha risklidir."
}

if ($installedVersion.ProductName -ne "LOKAL BILGISAYAR KONTROL PANELI") {
    throw "Kurulu exe product name hatali: $($installedVersion.ProductName)"
}

if ($installedVersion.ProductVersion -ne $targetVersion.ProductVersion) {
    throw "Kurulu exe ve target/release exe versiyonu farkli."
}

if ($missingMarkers.Count -gt 0) {
    throw "Kurulu exe guncel production markerlarini tasimiyor: $($missingMarkers -join ', ')"
}

if ($differentByteCount -gt 16) {
    throw "Kurulu exe ile target/release exe arasinda beklenenden fazla binary fark var: $differentByteCount bayt."
}

Write-Host "PASS: Kurulu exe installer cikisi olarak dogrulandi. Hash hedefi kurulu exe + installer hashleridir; target/release ham build hash'i kabul kriteri degildir."
