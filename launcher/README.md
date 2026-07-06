# PartyLock Launcher

Cliente desktop (Tauri) para login Steam, receber partidas e conectar ao Deadlock.

## Desenvolvimento

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
cd server/deadworks/launcher
npm install
npm run tauri dev
```

## Build para distribuição (Windows)

Requisitos: [Node.js](https://nodejs.org/) 20+, [Rust](https://rustup.rs/), WebView2 (já vem no Windows 10/11).

### Só gerar o `.exe` local (sem GitHub)

```powershell
cd server/deadworks/launcher
npm run build:installer
```

### Publicar no GitHub (build + commit + tag + push → CI anexa na Release)

```powershell
cd server/deadworks/launcher
npm run publish
```

Isso faz: bump de versão (patch) → build NSIS → commit → tag `launcher-vX.Y.Z` → push. O workflow em `.github/workflows/launcher-release.yml` publica o instalador.

Republicar a **mesma** versão (tag já existe):

```powershell
.\scripts\release.ps1 -Version 0.1.0 -Tag -Push -Retag -Commit
```

Upload imediato via `gh` (opcional, após `gh auth login`):

```powershell
.\scripts\release.ps1 -Bump patch -Tag -Push -Commit -GhUpload
```

Ou, na pasta `launcher`:

```powershell
.\scripts\build-windows.ps1
```

Se você já estiver dentro de `launcher\scripts`:

```powershell
.\build-windows.ps1
```

O instalador `.exe` (NSIS) sai em:

`src-tauri/target/release/bundle/nsis/PartyLock_<versão>_x64-setup.exe`

Esse arquivo é o que você publica para download.

Link fixo (sempre aponta para a última release):

`https://github.com/partylock/deadworks/releases/latest/download/PartyLock-setup-x64.exe`

Link curto no site (redirect gratuito via `app/public/_redirects` no Cloudflare Pages):

`https://partylock.com.br/download/launcher`

O CI renomeia o instalador para `PartyLock-setup-x64.exe` em cada release — o nome precisa ser sempre o mesmo para o `/latest/download/` funcionar.

### API de produção

Release builds apontam para `https://api.partylock.com.br/api/v1` (via `.env.production` e fallback no código). Para outro endpoint no build:

```powershell
$env:VITE_PARTYLOCK_API_URL = "https://api.partylock.com.br/api/v1"
npm run build:win
```

## Release automático (GitHub Actions)

No repositório `deadworks`, crie uma tag:

```bash
git tag launcher-v0.4.0
git push origin launcher-v0.4.0
```

O workflow `.github/workflows/launcher-release.yml` gera o instalador Windows e anexa na GitHub Release.

## Versão

Mantenha alinhados:

- `package.json` → `version`
- `src-tauri/tauri.conf.json` → `version`
- `src-tauri/Cargo.toml` → `version`
