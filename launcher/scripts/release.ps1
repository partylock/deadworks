#Requires -Version 5.1
<#
.SYNOPSIS
  Release do PartyLock Launcher: sincroniza versão, build NSIS e (opcional) tag GitHub.

.EXAMPLE
  .\scripts\release.ps1 -Bump patch
  Sobe 0.4.0 -> 0.4.1, atualiza os 3 manifests e gera o instalador.

.EXAMPLE
  .\scripts\release.ps1 -Version 0.5.0 -Tag -Push -Commit
  Define versão, build, commit, tag launcher-v0.5.0 e push (dispara CI).

.EXAMPLE
  .\scripts\release.ps1 -Version 0.4.0 -SkipBuild -Tag -Push -Retag
  Re-dispara release quando a tag launcher-v0.4.0 ja existe.
#>
param(
    [string]$Version,
    [ValidateSet("patch", "minor", "major")]
    [string]$Bump,
    [switch]$SkipBuild,
    [switch]$RegenerateIcons,
    [switch]$Tag,
    [switch]$Push,
    [switch]$Commit,
    [switch]$Retag,
    [switch]$GhUpload,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

$LauncherRoot = Split-Path -Parent $PSScriptRoot
Set-Location $LauncherRoot

function Invoke-Git {
    param(
        [string[]]$GitArgs = @(),
        [switch]$ReadOutput
    )

    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        if ($ReadOutput) {
            return (& git @GitArgs 2>$null | Out-String).Trim()
        }
        & git @GitArgs 2>&1 | Out-Null
        return $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $prev
    }
}

function Get-SemVerParts([string]$Value) {
    if ($Value -match "^(\d+)\.(\d+)\.(\d+)$") {
        return [PSCustomObject]@{
            Major = [int]$matches[1]
            Minor = [int]$matches[2]
            Patch = [int]$matches[3]
        }
    }
    throw "Versão inválida: $Value (use MAJOR.MINOR.PATCH)"
}

function Bump-SemVer([string]$Value, [string]$Part) {
    $semver = Get-SemVerParts $Value
    switch ($Part) {
        "major" { $semver.Major++; $semver.Minor = 0; $semver.Patch = 0 }
        "minor" { $semver.Minor++; $semver.Patch = 0 }
        "patch" { $semver.Patch++ }
    }
    return "$($semver.Major).$($semver.Minor).$($semver.Patch)"
}

function Get-LauncherVersion {
    $pkgPath = Join-Path $LauncherRoot "package.json"
    $pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
    return [string]$pkg.version
}

function Set-LauncherVersion([string]$NewVersion) {
    $files = @(
        @{
            Path    = Join-Path $LauncherRoot "package.json"
            Pattern = '(?m)^  "version": "[^"]+"'
            Replace = "  `"version`": `"$NewVersion`""
        },
        @{
            Path    = Join-Path $LauncherRoot "src-tauri\tauri.conf.json"
            Pattern = '(?m)^  "version": "[^"]+"'
            Replace = "  `"version`": `"$NewVersion`""
        },
        @{
            Path    = Join-Path $LauncherRoot "src-tauri\Cargo.toml"
            Pattern = '(?m)^version = "[^"]+"'
            Replace = "version = `"$NewVersion`""
        }
    )

    foreach ($file in $files) {
        $content = Get-Content $file.Path -Raw
        $updated = [regex]::Replace($content, $file.Pattern, $file.Replace, 1)
        if ($content -eq $updated) {
            throw "Não foi possível atualizar versão em $($file.Path)"
        }
        if ($DryRun) {
            Write-Host "[dry-run] $($file.Path) -> $NewVersion" -ForegroundColor DarkGray
        } else {
            [System.IO.File]::WriteAllText($file.Path, $updated)
            Write-Host "Atualizado: $($file.Path)" -ForegroundColor DarkGray
        }
    }
}

function Get-InstallerFile([string]$SearchDir, [string]$Version = "") {
    $files = @(Get-ChildItem $SearchDir -Filter "*.exe" -ErrorAction SilentlyContinue)
    if ($files.Count -eq 0) { return $null }
    if ($Version) {
        $matched = @($files | Where-Object { $_.Name -like "*_$Version_*" })
        if ($matched.Count -gt 0) { return $matched[0] }
    }
    return ($files | Sort-Object LastWriteTime -Descending)[0]
}

function Resolve-Installer([string]$Version) {
    $bundleDir = Join-Path $LauncherRoot "src-tauri\target\release\bundle\nsis"
    $assetsDir = Join-Path $LauncherRoot "release-assets"
    foreach ($dir in @($bundleDir, $assetsDir)) {
        $file = Get-InstallerFile $dir $Version
        if ($file) { return $file }
    }
    return $null
}

$Script:LauncherAssetName = "PartyLock-setup-x64.exe"
$Script:LauncherDownloadUrl = "https://github.com/partylock/deadworks/releases/latest/download/PartyLock-setup-x64.exe"

function Get-GhRepo {
    $url = Invoke-Git -GitArgs @("-C", $LauncherRoot, "remote", "get-url", "origin") -ReadOutput
    if ($url -match "github\.com[:/](.+?)(?:\.git)?$") {
        return $matches[1]
    }
    return "partylock/deadworks"
}

function Copy-LauncherReleaseAsset($Installer) {
    $assetsDir = Join-Path $LauncherRoot "release-assets"
    New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null
    $assetPath = Join-Path $assetsDir $Script:LauncherAssetName
    Copy-Item $Installer.FullName $assetPath -Force
    return Get-Item $assetPath
}

function Get-GhExe {
    $gh = Get-Command gh -ErrorAction SilentlyContinue
    if ($gh) { return $gh.Source }

    $default = "C:\Program Files\GitHub CLI\gh.exe"
    if (Test-Path $default) { return $default }

    return $null
}

function Invoke-Gh {
    param([string[]]$GhArgs = @())

    $ghExe = Get-GhExe
    if (-not $ghExe) { return @{ Found = $false; ExitCode = 127 } }

    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $ghExe @GhArgs 2>&1 | Out-Null
        return @{ Found = $true; ExitCode = $LASTEXITCODE }
    } finally {
        $ErrorActionPreference = $prev
    }
}

function Invoke-LauncherBuild {
    if ($DryRun) {
        Write-Host "[dry-run] build-windows.ps1" -ForegroundColor DarkGray
        return $null
    }

    & (Join-Path $PSScriptRoot "build-windows.ps1")
    $bundleDir = Join-Path $LauncherRoot "src-tauri\target\release\bundle\nsis"
    $installer = Get-InstallerFile $bundleDir
    if ($installer) {
        Copy-LauncherReleaseAsset $installer | Out-Null
    }
    return $installer
}

function Invoke-GhReleaseUpload([string]$NewVersion, $Installer) {
    if ($Installer -is [System.Array]) { $Installer = $Installer[0] }
    if (-not $Installer) { return }

    $ghExe = Get-GhExe
    if (-not $ghExe) {
        Write-Host "gh CLI nao encontrado. Instale: winget install GitHub.cli" -ForegroundColor Yellow
        return
    }

    $tagName = "launcher-v$NewVersion"
    $repo = Get-GhRepo
    if ($DryRun) {
        Write-Host "[dry-run] gh release upload $tagName --repo $repo" -ForegroundColor DarkGray
        return
    }

    $auth = Invoke-Gh -GhArgs @("auth", "status")
    if ($auth.ExitCode -ne 0) {
        Write-Host "gh nao autenticado. Rode uma vez: gh auth login" -ForegroundColor Yellow
        return
    }

    $asset = Copy-LauncherReleaseAsset $Installer
    Write-Host "Publicando instalador na GitHub Release ($tagName)..." -ForegroundColor Cyan

    $upload = Invoke-Gh -GhArgs @(
        "release", "upload", $tagName, $asset.FullName,
        "--clobber", "--repo", $repo
    )
    if ($upload.ExitCode -eq 0) {
        Write-Host "Instalador publicado na release." -ForegroundColor Green
        Write-Host "Link fixo (latest): $($Script:LauncherDownloadUrl)" -ForegroundColor Cyan
        return
    }

    $title = "PartyLock Launcher v$NewVersion"
    $notes = "Instalador Windows do PartyLock Launcher (PartyLock-setup-x64.exe).`n`nLink fixo: $($Script:LauncherDownloadUrl)"
    $create = Invoke-Gh -GhArgs @(
        "release", "create", $tagName, $asset.FullName,
        "--title", $title,
        "--notes", $notes,
        "--repo", $repo
    )
    if ($create.ExitCode -eq 0) {
        Write-Host "Release criada com instalador local." -ForegroundColor Green
        Write-Host "Link fixo (latest): $($Script:LauncherDownloadUrl)" -ForegroundColor Cyan
        return
    }

    Write-Host "Falha ao publicar via gh. Verifique:" -ForegroundColor Yellow
    Write-Host "  gh release view $tagName --repo $repo" -ForegroundColor DarkGray
}

function Invoke-RegenerateIcons {
    if ($DryRun) {
        Write-Host "[dry-run] tauri icon public/app-icon.png" -ForegroundColor DarkGray
        return
    }

    Write-Host "Regenerando icones..." -ForegroundColor Cyan
    npx tauri icon public/app-icon.png -o src-tauri/icons --ios-color "#0A0A0A"
}

function Remove-LauncherTag([string]$GitRoot, [string]$TagName) {
    if ($DryRun) {
        Write-Host "[dry-run] remover tag $TagName (local + remoto)" -ForegroundColor DarkGray
        return
    }

    Push-Location $GitRoot
    try {
        Invoke-Git -GitArgs @("tag", "-d", $TagName) | Out-Null
        $remoteCode = Invoke-Git -GitArgs @("push", "origin", ":refs/tags/$TagName")
        if ($remoteCode -ne 0) {
            Write-Host "Tag remota $TagName ausente ou ja removida." -ForegroundColor DarkGray
        }
    } finally {
        Pop-Location
    }
}

function Invoke-ReleaseGit([string]$NewVersion) {
    $gitRoot = Invoke-Git -GitArgs @("-C", $LauncherRoot, "rev-parse", "--show-toplevel") -ReadOutput
    if (-not $gitRoot) {
        throw "Repositorio git nao encontrado a partir de $LauncherRoot"
    }

    $tagName = "launcher-v$NewVersion"
    $versionFiles = @(
        "launcher/package.json",
        "launcher/src-tauri/tauri.conf.json",
        "launcher/src-tauri/Cargo.toml"
    )

    if ($Commit) {
        if ($DryRun) {
            Write-Host "[dry-run] git commit versão $NewVersion" -ForegroundColor DarkGray
        } else {
            Push-Location $gitRoot
            try {
                Invoke-Git -Args (@("add") + $versionFiles) | Out-Null
                $commitCode = Invoke-Git -GitArgs @("commit", "-m", "chore(launcher): release v$NewVersion")
                if ($commitCode -ne 0) {
                    Write-Host "Nada para commitar (versao ja commitada?)." -ForegroundColor DarkGray
                }
            } finally {
                Pop-Location
            }
        }
    } elseif ($Tag -or $Push) {
        Write-Host ""
        Write-Host "Aviso: versão alterada mas -Commit não foi passado." -ForegroundColor Yellow
        Write-Host "Faça commit manual antes do push, ou use -Commit." -ForegroundColor Yellow
    }

    if ($Tag) {
        if ($DryRun) {
            Write-Host "[dry-run] git tag $tagName" -ForegroundColor DarkGray
        } else {
            Push-Location $gitRoot
            try {
                $tagExists = Invoke-Git -GitArgs @("rev-parse", "-q", "--verify", "refs/tags/$tagName") -ReadOutput
                if ($tagExists) {
                    if ($Retag) {
                        Write-Host "Tag $tagName ja existe; recriando (-Retag)..." -ForegroundColor Yellow
                        Remove-LauncherTag $gitRoot $tagName
                    } else {
                        Write-Host "Tag $tagName ja existe; pulando criacao (use -Retag para recriar)." -ForegroundColor Yellow
                    }
                }

                if (-not $tagExists -or $Retag) {
                    Invoke-Git -GitArgs @("tag", "-a", $tagName, "-m", "PartyLock Launcher v$NewVersion") | Out-Null
                    Write-Host "Tag criada: $tagName" -ForegroundColor Green
                }
            } finally {
                Pop-Location
            }
        }
    }

    if ($Push) {
        if ($DryRun) {
            Write-Host "[dry-run] git push && git push origin $tagName" -ForegroundColor DarkGray
        } else {
            Push-Location $gitRoot
            try {
                $branchCode = Invoke-Git -GitArgs @("push")
                if ($branchCode -ne 0) { throw "git push falhou (exit $branchCode)" }

                if ($Tag) {
                    if ($Retag) {
                        $tagCode = Invoke-Git -GitArgs @("push", "origin", $tagName, "--force")
                    } else {
                        $tagCode = Invoke-Git -GitArgs @("push", "origin", $tagName)
                    }
                    if ($tagCode -ne 0) {
                        Write-Host "Tag remota ja existe; use -Retag para re-disparar o CI." -ForegroundColor Yellow
                    }
                }
                Write-Host "Push concluido. CI deve publicar a release em breve." -ForegroundColor Green
            } finally {
                Pop-Location
            }
        }
    }
}

if (-not $Version -and -not $Bump) {
    throw "Informe -Version X.Y.Z ou -Bump patch|minor|major"
}
if ($Version -and $Bump) {
    throw "Use apenas -Version ou -Bump, não os dois"
}
if ($Push -and -not $Tag) {
    throw "-Push exige -Tag"
}

$currentVersion = Get-LauncherVersion
$targetVersion = if ($Bump) { Bump-SemVer $currentVersion $Bump } else { $Version }
Get-SemVerParts $targetVersion | Out-Null

Write-Host ""
Write-Host "PartyLock Launcher - release" -ForegroundColor Cyan
Write-Host "  $currentVersion -> $targetVersion"
Write-Host ""

if ($currentVersion -ne $targetVersion) {
    Set-LauncherVersion $targetVersion
} else {
    Write-Host "Versão já é $targetVersion (arquivos inalterados)." -ForegroundColor DarkGray
}

if ($RegenerateIcons) {
    Invoke-RegenerateIcons
}

$installer = $null
if (-not $SkipBuild) {
    $installer = Invoke-LauncherBuild
} else {
    $installer = Resolve-Installer $targetVersion
}
if ($installer -is [System.Array]) { $installer = $installer[0] }

if ($Tag -or $Push -or $Commit) {
    Invoke-ReleaseGit $targetVersion
    if ($GhUpload -and $installer) {
        Invoke-GhReleaseUpload $targetVersion $installer
    } elseif ($Push -and $Tag) {
        Write-Host ""
        Write-Host "CI publica o instalador automaticamente (~10-15 min):" -ForegroundColor Cyan
        Write-Host "https://github.com/partylock/deadworks/actions" -ForegroundColor Cyan
        Write-Host "Link fixo (latest): $($Script:LauncherDownloadUrl)" -ForegroundColor Cyan
        Write-Host "Upload local imediato: adicione -GhUpload (requer gh auth login)" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "Release $targetVersion" -ForegroundColor Green
if ($installer) {
    Write-Host "Instalador: $($installer.FullName)"
    $assetsCopy = Join-Path $LauncherRoot "release-assets\$($installer.Name)"
    if (Test-Path $assetsCopy) {
        Write-Host "Copia local: $assetsCopy"
    }
}
if (-not $Tag) {
    Write-Host ""
    Write-Host "Próximo passo (CI):" -ForegroundColor DarkGray
    Write-Host "  git add launcher/package.json launcher/src-tauri/tauri.conf.json launcher/src-tauri/Cargo.toml"
    Write-Host "  git commit -m `"chore(launcher): release v$targetVersion`""
    Write-Host "  git tag launcher-v$targetVersion"
    Write-Host "  git push && git push origin launcher-v$targetVersion"
    Write-Host ""
    Write-Host "Ou rode de novo com: -Tag -Push -Commit" -ForegroundColor DarkGray
}
