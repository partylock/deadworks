#Requires -Version 5.1
$ErrorActionPreference = "Stop"

$LauncherRoot = Split-Path -Parent $PSScriptRoot
Set-Location $LauncherRoot

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "npm não encontrado no PATH"
}

$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
if (Test-Path $cargo) {
    $env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
}

Write-Host "Instalando dependências…" -ForegroundColor Cyan
npm ci

Write-Host "Gerando instalador Windows (NSIS)…" -ForegroundColor Cyan
npm run build:win

$bundleDir = Join-Path $LauncherRoot "src-tauri\target\release\bundle\nsis"
if (-not (Test-Path $bundleDir)) {
    throw "Build concluiu mas pasta NSIS não encontrada: $bundleDir"
}

$installer = Get-ChildItem $bundleDir -Filter "*.exe" | Select-Object -First 1
if (-not $installer) {
    throw "Nenhum .exe encontrado em $bundleDir"
}

Write-Host ""
Write-Host "Instalador pronto:" -ForegroundColor Green
Write-Host $installer.FullName
Write-Host ""
Write-Host "So build local. Para publicar no GitHub (tag + CI):" -ForegroundColor DarkGray
Write-Host "  npm run publish" -ForegroundColor Cyan
