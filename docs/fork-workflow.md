# Fork — atualizar com o upstream

`server/deadworks/` é um fork de [Deadworks-net/deadworks](https://github.com/Deadworks-net/deadworks).

| Remote | Repositório | Uso |
|--------|-------------|-----|
| `upstream` | `Deadworks-net/deadworks` | Oficial (somente leitura) |
| `origin` | `partylock/deadworks` | Seu fork no GitHub |

Branch de trabalho: **`partylock`** (plugins e docs PartyLock).

## Estratégia de branches

```
upstream/main  ──merge──►  partylock     ← você faz isso (atualizar)
     │                         │
     │                         └── plugins/, docs/ (PartyLock)
     ▼
  main (fork)                 origin/partylock
  espelho opcional            branch de trabalho permanente
  do oficial
```

| Merge | Faz? | Motivo |
|-------|------|--------|
| `upstream/main` → `partylock` | **Sim** | Trazer novidades do Deadworks oficial |
| `upstream/main` → `main` | Opcional | Manter `main` como cópia limpa do upstream |
| `partylock` → `main` | **Nunca** | Misturaria código PartyLock no espelho do oficial |
| `partylock` → `upstream` | **Nunca** | Você não tem push no repo oficial |

**Você nunca mergeia `partylock` em `main`.** As duas branches divergem de propósito:

- **`partylock`** — onde você trabalha e commita (Deadworks + PartyLock)
- **`main`** — opcional; pode ignorar ou usar só para espelhar o upstream sem suas customizações

No dia a dia: clone, checkout e push sempre em **`partylock`**.

## Atualizar (rotina)

```powershell
cd server/deadworks

git fetch upstream
git checkout partylock
git merge upstream/main
git push origin partylock
```

One-liner:

```powershell
git fetch upstream && git checkout partylock && git merge upstream/main && git push origin partylock
```

## Se houver conflitos

```powershell
# Resolver arquivos marcados pelo Git, depois:
git add .
git commit -m "Merge upstream/main into partylock"
git push origin partylock
```

Conflitos são raros se você só editou `plugins/` e `docs/` (ver regra `deadworks-fork-boundaries`).

## Depois do merge — testar

```powershell
dotnet build managed\DeadworksManaged.sln
dotnet build plugins\PartyLockCorePlugin\PartyLockCorePlugin.csproj
```

Suba o `deadworks.exe` e valide os plugins PartyLock no jogo.

## Quando atualizar

- Nova [release do Deadworks](https://github.com/Deadworks-net/deadworks/releases)
- Patch do Deadlock que quebra hooks (`signature not found` no console)
- Precisa de API ou feature nova do framework

## Opcional: espelhar `main` no fork

Só se quiser uma branch `main` no GitHub igual ao oficial **sem** plugins PartyLock:

```powershell
git checkout main
git merge upstream/main    # só upstream → main, nunca partylock → main
git push origin main
git checkout partylock
```

Se não usar `main`, pode ignorá-la — **`partylock` é suficiente**.

## Setup inicial (referência)

Se clonar de novo:

```powershell
git clone https://github.com/partylock/deadworks.git server/deadworks
cd server/deadworks
git remote add upstream https://github.com/Deadworks-net/deadworks.git
git checkout partylock
```

## O que commitar no fork

| Commitar | Não commitar |
|----------|--------------|
| `plugins/` | `local.props` |
| `docs/` | `bin/`, `obj/` |
| `.gitignore` | configs em `Deadlock/.../configs/` |
