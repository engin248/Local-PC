param(
    [string]$ConfigPath = (Join-Path $PSScriptRoot "..\config\asker_motoru.json"),
    [string]$RootA = "",
    [string]$RootB = ""
)

$ErrorActionPreference = "Continue"

function Resolve-ConfiguredPath {
    param([string]$Path)
    $projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
    $parentRoot = Split-Path -Parent $projectRoot

    if ($Path.StartsWith('$PROJECT_ROOT')) {
        $suffix = $Path.Substring('$PROJECT_ROOT'.Length).TrimStart('\', '/')
        return Join-Path $projectRoot $suffix
    }
    if ($Path.StartsWith('$PARENT_DIR')) {
        $suffix = $Path.Substring('$PARENT_DIR'.Length).TrimStart('\', '/')
        return Join-Path $parentRoot $suffix
    }
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }
    return Join-Path $projectRoot $Path
}

if ([string]::IsNullOrWhiteSpace($RootA) -or [string]::IsNullOrWhiteSpace($RootB)) {
    $config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace($RootA)) {
        $RootA = Resolve-ConfiguredPath $config.roots[0].path
    }
    if ([string]::IsNullOrWhiteSpace($RootB)) {
        $RootB = Resolve-ConfiguredPath $config.roots[1].path
    }
}

Write-Host "==========================================================================" -ForegroundColor Cyan
Write-Host "            ASKER MOTORU YENİ NESİL VS LEGACY PARİTE DENETİMİ             " -ForegroundColor Cyan
Write-Host "==========================================================================" -ForegroundColor Cyan
Write-Host "KÖKEN A (Aktif Sunucu): $RootA" -ForegroundColor Yellow
Write-Host "KÖKEN B (Kök Klasör):   $RootB" -ForegroundColor Yellow
Write-Host "--------------------------------------------------------------------------"

# Dosya eşleştirmeleri: [Dizin A Yolu], [Dizin B Yolu], [Açıklama]
$mappings = @(
    @{ PathA = "runtime\indexes\KUTUPHANE_INDEX.json"; PathB = "runtime\indexes\MUTLAK_KUTUPHANE_INDEX.json"; Desc = "Kütüphane Beceri İndeksi" },
    @{ PathA = "runtime\indexes\UZMAN_HAVUZU.json"; PathB = "runtime\indexes\L5_UZMAN_HAVUZU.json"; Desc = "Uzman Havuzu Veritabanı" },
    @{ PathA = "runtime\indexes\PLANLAMA_UZMANLIK_ALANI_VERITABANI_HARITASI.json"; PathB = "runtime\indexes\PLANLAMA_UZMANLIK_ALANI_VERITABANI_HARITASI.json"; Desc = "Planlama Haritası" },
    @{ PathA = "Panel\chatbot\src\app\page.tsx"; PathB = "Panel\strategy.js"; Desc = "Komuta Arayüzü Çekirdeği" },
    @{ PathA = "Panel\electron_main.js"; PathB = "Panel\electron_main.js"; Desc = "Electron Giriş Noktası" },
    @{ PathA = "Panel\tsconfig.json"; PathB = ".gitignore"; Desc = "Tip / Git Yapılandırması" }
)

function Get-FileHashSha256 {
    param([string]$Path)
    if (Test-Path -LiteralPath $Path) {
        return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash
    }
    return $null
}

function Get-FileLineCount {
    param([string]$Path)
    if (Test-Path -LiteralPath $Path) {
        try {
            return (Get-Content -LiteralPath $Path).Count
        } catch {
            return "N/A"
        }
    }
    return $null
}

function Get-FileSize {
    param([string]$Path)
    if (Test-Path -LiteralPath $Path) {
        return (Get-Item -LiteralPath $Path).Length
    }
    return $null
}

$results = @()

foreach ($map in $mappings) {
    $pathA = Join-Path $RootA $map.PathA
    $pathB = Join-Path $RootB $map.PathB

    $existsA = Test-Path -LiteralPath $pathA
    $existsB = Test-Path -LiteralPath $pathB

    $hashA = if ($existsA) { Get-FileHashSha256 $pathA } else { "EKSİK" }
    $hashB = if ($existsB) { Get-FileHashSha256 $pathB } else { "EKSİK" }

    $sizeA = if ($existsA) { Get-FileSize $pathA } else { 0 }
    $sizeB = if ($existsB) { Get-FileSize $pathB } else { 0 }

    $linesA = if ($existsA) { Get-FileLineCount $pathA } else { 0 }
    $linesB = if ($existsB) { Get-FileLineCount $pathB } else { 0 }

    $status = "OK"
    if ($hashA -eq "EKSİK" -or $hashB -eq "EKSİK") {
        $status = "DOSYA EKSİK"
    } elseif ($hashA -eq $hashB) {
        $status = "TAM PARİTE (HALEF-SELEF AYNI)"
    } else {
        $status = "MİMARİ FARK (İŞLEVSEL GÜNCELLEME)"
    }

    $results += [PSCustomObject]@{
        "Bileşen / Tanım" = $map.Desc
        "Köken A (Masaüstü Aktif)" = if ($existsA) { "$sizeA B / $linesA satır / $($hashA.Substring(0, 8))" } else { "EKSİK" }
        "Köken B (Legacy/Yedek)" = if ($existsB) { "$sizeB B / $linesB satır / $($hashB.Substring(0, 8))" } else { "EKSİK" }
        "Parite Durumu" = $status
    }
}

$results | Format-Table -AutoSize

# Markdown Raporu oluştur
$markdown = @"
# ASKER MOTORU İSTASYON PARİTE RAPORU
**Zaman Damgası**: $((Get-Date).ToString("yyyy-MM-dd HH:mm:ss"))
**Aktif Merkez İstasyonu (Köken A - Next.js)**: $RootA
**Yedek/Legacy Kök İstasyonu (Köken B - Electron)**: $RootB

| Bileşen / Tanım | Köken A (Masaüstü Aktif) | Köken B (Legacy/Yedek) | Parite Durumu |
| :--- | :--- | :--- | :--- |
"@

foreach ($row in $results) {
    $markdown += "`n| " + $row."Bileşen / Tanım" + " | " + $row."Köken A (Masaüstü Aktif)" + " | " + $row."Köken B (Legacy/Yedek)" + " | " + $row."Parite Durumu" + " |"
}

$markdownPath = Join-Path $PSScriptRoot "asker_motoru_parity_report.md"
$markdown | Set-Content -LiteralPath $markdownPath -Encoding UTF8
Write-Host "--------------------------------------------------------------------------"
Write-Host "Markdown Parite Raporu başarıyla kaydedildi: $markdownPath" -ForegroundColor Green
Write-Host "==========================================================================" -ForegroundColor Cyan
