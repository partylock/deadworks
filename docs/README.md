# Deadworks — PartyLock Server Docs

Deadworks is the .NET plugin framework for **Deadlock** dedicated servers. PartyLock vendors the SDK under `server/deadworks/` and builds in-game plugins against it.

## Official Documentation

Primary source of truth: **[docs.deadworks.net](https://docs.deadworks.net/)**

| Topic | Link |
|-------|------|
| Overview | https://docs.deadworks.net/ |
| Project Setup | https://docs.deadworks.net/getting-started/setup |
| First Plugin | https://docs.deadworks.net/getting-started/first-plugin |
| Plugin Lifecycle | https://docs.deadworks.net/guides/plugin-lifecycle |
| How Deadworks Works | https://docs.deadworks.net/guides/how-deadworks-works |
| Server Hosting | https://docs.deadworks.net/guides/server-hosting |
| Examples | https://docs.deadworks.net/examples/roll-the-dice |

## PartyLock Plugins

| Plugin | Path | Description |
|--------|------|-------------|
| PartyLock Core | `plugins/PartyLockCorePlugin/` | Anti-cheat, bloqueio de troca de herói, herói aleatório |

Build: `dotnet build server/deadworks/plugins/PartyLockCorePlugin/PartyLockCorePlugin.csproj`

## Local Docs

| File | Description |
|------|-------------|
| [fork-workflow.md](./fork-workflow.md) | Como atualizar o fork com o upstream oficial |
| [plugin-development.md](./plugin-development.md) | Setup, build, deploy, and PartyLock conventions |
| [api-quick-reference.md](./api-quick-reference.md) | Condensed API cheat sheet |

## Examples

| Example | Source | Docs |
|---------|--------|------|
| Roll The Dice | `examples/plugins/RollTheDicePlugin/` | [roll-the-dice.md](./examples/roll-the-dice.md) · [official](https://docs.deadworks.net/examples/roll-the-dice/) |

## Repository Layout

```
server/deadworks/
├── managed/
│   ├── DeadworksManaged.Api/   # Public API (entities, timers, net messages)
│   ├── DeadworksManaged/       # Runtime loader
│   └── Directory.Build.props   # Imports local.props for DeadlockDir
├── examples/plugins/           # Reference plugins (Deathmatch, Tag, etc.)
├── docs/                       # This folder
└── local.props                 # Gitignored — set your Deadlock install path
```

## Quick Start

1. Copy `local.props.example` → `local.props` and set `DeadlockDir`
2. Open `examples/ExamplePlugins.slnx` or create a new plugin project
3. Build — DLL auto-deploys to `{DeadlockDir}/managed/plugins/`
4. Run `deadworks.exe` and connect: `connect localhost:27015`

## Cursor AI Rules

Project rules live in `.cursor/rules/` (dentro de `server/deadworks/`):

- `deadworks-fork-boundaries.mdc` — o que pode/não pode editar no fork
- `deadworks-plugins.mdc` — plugin structure and lifecycle

Fork update workflow: [fork-workflow.md](./fork-workflow.md)
