# Plugin Development — PartyLock

## Prerequisites

- [.NET 10 SDK](https://dotnet.microsoft.com/download)
- Deadlock installed (Steam) or a [SteamCMD server install](https://docs.deadworks.net/guides/server-hosting)
- [Deadworks release](https://github.com/Deadworks-net/deadworks/releases) extracted into the Deadlock `game/bin/win64/` folder

## Local Configuration

Create `server/deadworks/local.props` (gitignored):

```xml
<Project>
  <PropertyGroup>
    <DeadlockDir>C:\Program Files (x86)\Steam\steamapps\common\Deadlock\game\bin\win64</DeadlockDir>
  </PropertyGroup>
</Project>
```

`Directory.Build.props` resolves `DeadlockManagedDir` from this path. All example plugins and the managed runtime use it for post-build deploy.

## Creating a Plugin

### 1. New Class Library

Target `net10.0` with dynamic loading enabled:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <EnableDynamicLoading>true</EnableDynamicLoading>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\..\managed\DeadworksManaged.Api\DeadworksManaged.Api.csproj">
      <Private>false</Private>
      <ExcludeAssets>runtime</ExcludeAssets>
    </ProjectReference>
  </ItemGroup>

  <Target Name="DeployToGame" AfterTargets="Build">
    <ItemGroup>
      <DeployFiles Include="$(OutputPath)MyPlugin.dll;$(OutputPath)MyPlugin.pdb" />
    </ItemGroup>
    <Copy SourceFiles="@(DeployFiles)"
          DestinationFolder="$(DeadlockManagedDir)\plugins"
          SkipUnchangedFiles="false" />
  </Target>
</Project>
```

### 2. Plugin Class

```csharp
using DeadworksManaged.Api;

namespace MyPlugin;

public class MyPlugin : DeadworksPluginBase
{
    public override string Name => "MyPlugin";

    public override void OnLoad(bool isReload)
    {
        Console.WriteLine($"[{Name}] {(isReload ? "Reloaded" : "Loaded")}");
    }

    public override void OnUnload()
    {
        Console.WriteLine($"[{Name}] Unloaded");
    }
}
```

### 3. Build & Run

```bash
dotnet build
# DLL lands in {DeadlockDir}/managed/plugins/
```

Start the server from `cmd.exe`:

```batch
deadworks.exe -dedicated -console -insecure +hostport 27015 +map dl_midtown
```

Connect in-game: `connect localhost:27015`

## Lifecycle Cheat Sheet

```
Server Start
  ├── OnPrecacheResources()   ← Precache.AddResource(...)
  ├── OnLoad(isReload)        ← Init state
  ├── OnStartupServer()       ← ConVar.Find(...).SetInt(...)
  │     (server running — hooks fire)
  └── OnUnload()              ← Cancel IHandle, dispose externals
```

See [Plugin Lifecycle](https://docs.deadworks.net/guides/plugin-lifecycle) for the full flow.

## Configuration Files

Use `[PluginConfig]` on a property to load JSON/JSONC from the plugin config directory:

```csharp
[PluginConfig]
public MyConfig Config { get; set; } = new();
```

Reload at runtime with `dw_reloadconfig` — handle changes in `OnConfigReloaded()`.

## Hot-Reload

Editing the plugin DLL while the server runs triggers hot-reload:

1. `OnUnload()` on old instance
2. `OnLoad(isReload: true)` on new instance

**Always cancel** `IHandle` timers, `Timer.Sequence`, sockets, and file watchers in `OnUnload`.

## Async / HTTP

Never touch game objects after `await` directly. Route through `Timer.NextTick`:

```csharp
private async Task FetchDataAsync()
{
    var data = await httpClient.GetStringAsync(url);
    Timer.NextTick(() =>
    {
        // Safe: back on game thread
    });
}
```

## Reference Plugins

Study these in `server/deadworks/examples/plugins/`:

| Plugin | Demonstrates |
|--------|-------------|
| `ExampleTimerPlugin` | Timers (Once, Every, Sequence, NextTick) |
| `ChatRelayPlugin` | `[NetMessageHandler]` interception |
| `DeathmatchPlugin` | `[PluginConfig]`, `EntityData<T>`, hero/item logic |
| `TagPlugin` | Game mode with player tracking |
| `RollTheDicePlugin` | Commands, HUD, sounds, particles, modifiers, timers — [walkthrough](./examples/roll-the-dice.md) |

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Build succeeds, plugin not loaded | Check `local.props` path; verify DLL in `managed/plugins/` |
| `Failed to initialize .NET runtime` | Install/repair .NET 10 SDK |
| No IntelliSense | Ensure `DeadworksManaged.Api.xml` sits next to the API DLL |
| Console logs missing on first boot | Hot-reload once, or log to file |
| `signature not found` | Update Deadworks after a Deadlock patch |

## PartyLock Integration Notes

- **Backend** (`backend/`) handles Steam auth, JWT, and PostgreSQL — out-of-game
- **Deadworks plugins** run in-game on the dedicated server
- Bridge the two via HTTP from plugins (use `Timer.NextTick` after `await`) or a shared message queue
- Keep secrets in server config, never in plugin source or committed JSON
