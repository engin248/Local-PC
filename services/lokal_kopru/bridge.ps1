# Lokal Komuta Köprüsü — Windows masaüstü HTTP sunucusu
# Cloud Agent (tünel URL ile) veya yerel panel bu API'yi çağırır.
param(
    [string]$ProjectRoot = "",
    [int]$Port = 0,
    [switch]$ProcessFileQueueOnce
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

if ([string]::IsNullOrWhiteSpace($ProjectRoot)) {
    $ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")).Path
}

$configPath = Join-Path $ProjectRoot "config\kopru_bridge.json"
$pathsPath = Join-Path $ProjectRoot "config\yerel_veri_yollari.json"

function Read-JsonFile {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return $null }
    Get-Content -LiteralPath $Path -Raw -Encoding UTF8 | ConvertFrom-Json
}

function Write-JsonResponse {
    param([System.Net.HttpListenerResponse]$Response, [int]$StatusCode, $Body)
    $json = $Body | ConvertTo-Json -Depth 8 -Compress
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
    $Response.StatusCode = $StatusCode
    $Response.ContentType = "application/json; charset=utf-8"
    $Response.ContentLength64 = $bytes.Length
    $Response.OutputStream.Write($bytes, 0, $bytes.Length)
    $Response.OutputStream.Close()
}

function Ensure-KopruToken {
    param($Config)
    if ($Config.token -and $Config.token.ToString().Trim().Length -ge 16) {
        return $Config.token.ToString().Trim()
    }
    $token = [guid]::NewGuid().ToString("N") + [guid]::NewGuid().ToString("N")
    $Config.token = $token
    ($Config | ConvertTo-Json -Depth 6) | Set-Content -LiteralPath $configPath -Encoding UTF8
    Write-Host "Yeni kopru token olusturuldu: config/kopru_bridge.json" -ForegroundColor Yellow
    return $token
}

function Test-KopruAuth {
    param([string]$Provided, [string]$Expected)
    if ([string]::IsNullOrWhiteSpace($Provided)) { return $false }
    return $Provided.Trim() -eq $Expected.Trim()
}

function Get-PathChecks {
    $paths = Read-JsonFile -Path $pathsPath
    if (-not $paths) {
        return @{ ok = $false; error = "yerel_veri_yollari.json bulunamadi"; checks = @() }
    }
    $checks = @()
    $labels = @{
        panel_proje = "Panel proje"
        kurulu_exe = "Kurulu exe"
        uzman_havuzu_json = "UZMAN_HAVUZU.json"
        skill_library_sqlite = "skill_library.sqlite"
        asker_motoru_kok = "Asker Motoru kok"
    }
    foreach ($key in $labels.Keys) {
        $p = $paths.$key
        $checks += [ordered]@{
            key = $key
            label = $labels[$key]
            path = $p
            exists = [bool]($p -and (Test-Path -LiteralPath $p))
        }
    }
    foreach ($alt in @($paths.skill_library_yedek_yollar)) {
        $checks += [ordered]@{
            key = "skill_library_yedek"
            label = "skill_library yedek"
            path = $alt
            exists = [bool](Test-Path -LiteralPath $alt)
        }
    }
    @{
        ok = $true
        timestamp = (Get-Date).ToString("o")
        checks = $checks
    }
}

function Invoke-WhitelistedOp {
    param([string]$Op, [string]$ProjectRoot)
    switch ($Op) {
        "health" {
            return @{ ok = $true; service = "lokal_kopru"; version = 1; project_root = $ProjectRoot }
        }
        "kopru_durum" {
            return @{
                ok = $true
                paths = Get-PathChecks
                pid = $PID
                machine = $env:COMPUTERNAME
            }
        }
        "yol_kontrol" {
            return Get-PathChecks
        }
        "kurulu_guncelle" {
            $script = Join-Path $ProjectRoot "scripts\update_installed_exe.ps1"
            if (-not (Test-Path -LiteralPath $script)) {
                return @{ ok = $false; error = "update_installed_exe.ps1 yok" }
            }
            $log = Join-Path $ProjectRoot "storage\kopru\logs\kurulu_guncelle_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"
            New-Item -ItemType Directory -Force -Path (Split-Path $log) | Out-Null
            & $script -ProjectRoot $ProjectRoot -SkipPull *>&1 | Tee-Object -FilePath $log
            $code = $LASTEXITCODE
            return @{
                ok = ($code -eq 0)
                exit_code = $code
                log = $log
            }
        }
        "panel_kapat" {
            $script = Join-Path $ProjectRoot "scripts\stop_panel_processes.ps1"
            & $script -ProjectRoot $ProjectRoot
            return @{ ok = ($LASTEXITCODE -eq 0); exit_code = $LASTEXITCODE }
        }
        default {
            return @{ ok = $false; error = "Izin verilmeyen islem: $Op" }
        }
    }
}

function Process-FileQueue {
    param($Config, [string]$ProjectRoot, [string]$Token)
    if (-not $Config.file_queue.enabled) { return }
    $inbox = Join-Path $ProjectRoot $Config.file_queue.inbox_dir
    $outbox = Join-Path $ProjectRoot $Config.file_queue.outbox_dir
    New-Item -ItemType Directory -Force -Path $inbox, $outbox | Out-Null
    Get-ChildItem -LiteralPath $inbox -Filter "*.json" -ErrorAction SilentlyContinue | ForEach-Object {
        $job = Get-Content -LiteralPath $_.FullName -Raw -Encoding UTF8 | ConvertFrom-Json
        if ($job.token -and -not (Test-KopruAuth -Provided $job.token -Expected $Token)) {
            $result = @{ ok = $false; error = "token_gecersiz"; job_id = $job.job_id }
        } elseif ($Config.allowed_operations -notcontains $job.op) {
            $result = @{ ok = $false; error = "op_izin_yok"; job_id = $job.job_id }
        } else {
            $result = Invoke-WhitelistedOp -Op $job.op -ProjectRoot $ProjectRoot
            $result.job_id = $job.job_id
        }
        $result.finished_at = (Get-Date).ToString("o")
        $outName = Join-Path $outbox ("{0}_result.json" -f ($job.job_id))
        ($result | ConvertTo-Json -Depth 8) | Set-Content -LiteralPath $outName -Encoding UTF8
        Remove-Item -LiteralPath $_.FullName -Force
    }
}

$config = Read-JsonFile -Path $configPath
if (-not $config) {
    throw "config/kopru_bridge.json bulunamadi"
}
$token = Ensure-KopruToken -Config $config
$port = if ($Port -gt 0) { $Port } else { [int]$config.listen_port }
$hostName = if ($config.listen_host) { $config.listen_host } else { "127.0.0.1" }
$prefix = "http://${hostName}:$port/"

if ($ProcessFileQueueOnce) {
    Process-FileQueue -Config $config -ProjectRoot $ProjectRoot -Token $token
    exit 0
}

$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add($prefix)
$listener.Start()
Write-Host "LOKAL KOMUTA KOPRUSU ACIK: $prefix" -ForegroundColor Green
Write-Host "Token: config/kopru_bridge.json icinde (paylasmayin)" -ForegroundColor DarkYellow

$fileQueueTimer = $null
if ($config.file_queue.enabled) {
    $pollMs = [Math]::Max(1000, [int]$config.file_queue.poll_seconds * 1000)
    $fileQueueTimer = [System.Timers.Timer]::new($pollMs)
    $fileQueueTimer.AutoReset = $true
    $fileQueueTimer.Add_Elapsed({
        try { Process-FileQueue -Config $config -ProjectRoot $ProjectRoot -Token $token } catch { }
    })
    $fileQueueTimer.Start()
}

try {
    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response
        $path = $request.Url.AbsolutePath.TrimEnd("/")
        if ([string]::IsNullOrWhiteSpace($path)) { $path = "/" }

        $auth = $request.Headers["X-Kopru-Token"]
        if (-not $auth) { $auth = $request.QueryString["token"] }

        try {
            if ($path -eq "/" -or $path -eq "/health" -or $path -eq "/v1/health") {
                Write-JsonResponse -Response $response -StatusCode 200 -Body (Invoke-WhitelistedOp -Op "health" -ProjectRoot $ProjectRoot)
                continue
            }

            if (-not (Test-KopruAuth -Provided $auth -Expected $token)) {
                Write-JsonResponse -Response $response -StatusCode 401 -Body @{ ok = $false; error = "token_gerekli" }
                continue
            }

            if ($path -eq "/v1/paths" -or $path -eq "/v1/yol_kontrol") {
                Write-JsonResponse -Response $response -StatusCode 200 -Body (Get-PathChecks)
                continue
            }

            if ($path -eq "/v1/durum") {
                Write-JsonResponse -Response $response -StatusCode 200 -Body (Invoke-WhitelistedOp -Op "kopru_durum" -ProjectRoot $ProjectRoot)
                continue
            }

            if ($path -eq "/v1/run" -and $request.HttpMethod -eq "POST") {
                $reader = New-Object System.IO.StreamReader($request.InputStream, $request.ContentEncoding)
                $raw = $reader.ReadToEnd()
                $reader.Close()
                $body = if ($raw) { $raw | ConvertFrom-Json } else { $null }
                $op = if ($body) { $body.op } else { $request.QueryString["op"] }
                if ($config.allowed_operations -notcontains $op) {
                    Write-JsonResponse -Response $response -StatusCode 403 -Body @{ ok = $false; error = "op_izin_yok"; op = $op }
                    continue
                }
                $result = Invoke-WhitelistedOp -Op $op -ProjectRoot $ProjectRoot
                $code = if ($result.ok) { 200 } else { 500 }
                Write-JsonResponse -Response $response -StatusCode $code -Body $result
                continue
            }

            Write-JsonResponse -Response $response -StatusCode 404 -Body @{ ok = $false; error = "endpoint_yok"; path = $path }
        } catch {
            Write-JsonResponse -Response $response -StatusCode 500 -Body @{ ok = $false; error = $_.Exception.Message }
        }
    }
} finally {
    if ($fileQueueTimer) { $fileQueueTimer.Stop(); $fileQueueTimer.Dispose() }
    $listener.Stop()
}
