# Cloud Agent veya baska makineden kopru cagrisi (tunel URL + token gerekir)
param(
    [Parameter(Mandatory = $true)]
    [string]$BaseUrl,
    [Parameter(Mandatory = $true)]
    [string]$Token,
    [Parameter(Mandatory = $true)]
    [ValidateSet("health", "yol_kontrol", "kurulu_guncelle", "panel_kapat", "kopru_durum")]
    [string]$Op
)

$ErrorActionPreference = "Stop"
$headers = @{ "X-Kopru-Token" = $Token }

if ($Op -eq "health") {
    $uri = "$BaseUrl/v1/health"
    Invoke-RestMethod -Uri $uri -Method Get
    exit 0
}

if ($Op -eq "yol_kontrol") {
    Invoke-RestMethod -Uri "$BaseUrl/v1/paths" -Headers $headers -Method Get
    exit 0
}

if ($Op -eq "kopru_durum") {
    Invoke-RestMethod -Uri "$BaseUrl/v1/durum" -Headers $headers -Method Get
    exit 0
}

$body = @{ op = $Op } | ConvertTo-Json
Invoke-RestMethod -Uri "$BaseUrl/v1/run" -Headers $headers -Method Post -Body $body -ContentType "application/json"
