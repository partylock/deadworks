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

function Get-InstallerFile([string]$BundleDir) {
    $file = @(Get-ChildItem $BundleDir -Filter "*.exe" -ErrorAction SilentlyContinue)[0]
    if (-not $file) { return $null }
    return $file
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
        $assetsDir = Join-Path $LauncherRoot "release-assets"
        New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null
        Copy-Item $installer.FullName (Join-Path $assetsDir $installer.Name) -Force
    }
    return $installer
}

function Invoke-GhReleaseUpload([string]$NewVersion, $Installer) {
    if ($Installer -is [System.Array]) { $Installer = $Installer[0] }
    if (-not $Installer) { return }
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        Write-Host "gh CLI nao encontrado; o CI anexa o instalador na GitHub Release." -ForegroundColor DarkGray
        return
    }

    $tagName = "launcher-v$NewVersion"
    if ($DryRun) {
        Write-Host "[dry-run] gh release upload $tagName $($Installer.FullName)" -ForegroundColor DarkGray
        return
    }

    Write-Host "Anexando instalador na GitHub Release ($tagName)..." -ForegroundColor Cyan
    $maxAttempts = 12
    for ($i = 1; $i -le $maxAttempts; $i++) {
        gh release upload $tagName $Installer.FullName --clobber 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Instalador publicado na release." -ForegroundColor Green
            return
        }
        if ($i -lt $maxAttempts) {
            Write-Host "Aguardando CI criar a release... ($i/$maxAttempts)" -ForegroundColor DarkGray
            Start-Sleep -Seconds 15
        }
    }

    Write-Host "Nao foi possivel anexar via gh; verifique a release no GitHub apos o CI." -ForegroundColor Yellow
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
    $assetsDir = Join-Path $LauncherRoot "release-assets"
    $installer = Get-InstallerFile $assetsDir
    if (-not $installer) {
        $bundleDir = Join-Path $LauncherRoot "src-tauri\target\release\bundle\nsis"
        $installer = Get-InstallerFile $bundleDir
    }
}
if ($installer -is [System.Array]) { $installer = $installer[0] }

if ($Tag -or $Push -or $Commit) {
    Invoke-ReleaseGit $targetVersion
    if ($Push -and $Tag) {
        Invoke-GhReleaseUpload $targetVersion $installer
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
