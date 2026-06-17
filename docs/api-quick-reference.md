# Deadworks API — Quick Reference

Condensed from [docs.deadworks.net](https://docs.deadworks.net/) and `DeadworksManaged.Api` source. For full details, use the official docs.

## Architecture

```
deadworks.exe
  ├── Deadlock engine (server.dll, engine2.dll)
  ├── Native hook layer (C++)
  ├── DeadworksManaged.Api (.NET 10)
  └── Your plugin DLLs (managed/plugins/)
```

Plugins are server-side only. HUD, Panorama UI, and rendering are client-only.

## Base Class Hooks

From `DeadworksPluginBase` / `IDeadworksPlugin`:

| Method | When |
|--------|------|
| `OnPrecacheResources()` | Map load — register resources |
| `OnLoad(bool isReload)` | Plugin load / hot-reload |
| `OnStartupServer()` | New map started |
| `OnUnload()` | Plugin unload |
| `OnGameFrame(simulating, firstTick, lastTick)` | Every server frame |
| `OnTakeDamage(args)` | Before damage applied |
| `OnModifyCurrency(args)` | Before gold change |
| `OnChatMessage(message)` | Player chat |
| `OnClientConCommand(args)` | Client console command |
| `OnClientConnect(args)` | Return `false` to reject |
| `OnClientPutInServer(args)` | Initial slot assignment |
| `OnClientFullConnect(args)` | Player fully in-game |
| `OnClientDisconnect(args)` | Player left |
| `OnEntityCreated/Spawned/Deleted(args)` | Entity lifecycle |
| `OnEntityStartTouch/EndTouch(args)` | Trigger zones |
| `OnAbilityAttempt(args)` | Block abilities via `BlockedButtons` |
| `OnAddModifier(args)` | Before buff/debuff applied |
| `OnCheckTransmit(args)` | Hide entities from a player |
| `OnPawnHeroInitialized(pawn)` | Hero abilities populated |
| `OnConfigReloaded()` | After `dw_reloadconfig` |

Hook methods returning `HookResult`: use `Stop` to block, `Continue` to pass through.

## Commands

```csharp
[Command("boost", Description = "Temporary stamina boost")]
public void CmdBoost(CCitadelPlayerController caller) { }
```

Registers: `/boost`, `!boost`, `dw_boost`.

## Timers

| API | Behavior |
|-----|----------|
| `Timer.Once(duration, callback)` | Fire once |
| `Timer.Every(duration, callback)` | Repeat until cancelled |
| `Timer.Sequence(step => ...)` | Stateful multi-step |
| `Timer.NextTick(callback)` | Next game frame |
| `handle.Cancel()` | Stop a timer |
| `handle.IsFinished` | Check completion |

Durations: `5.Seconds()`, `500.Milliseconds()`, `64.Ticks()`.

## Networking

```csharp
// Send to one player
NetMessages.Send(msg, RecipientFilter.Single(slot));

// Intercept outgoing
[NetMessageHandler]
public HookResult OnChatOutgoing(OutgoingMessageContext<CCitadelUserMsg_ChatMsg> ctx)
{
    return HookResult.Continue; // or Stop to swallow
}
```

## Entities

```csharp
var controller = CBaseEntity.FromIndex<CCitadelPlayerController>(entityIndex);
var pawn = caller.GetHeroPawn();
var state = new EntityData<MyState>(); // per-entity, auto-cleaned
```

Entity index for player controller = player slot + 1.

## ConVars

```csharp
public override void OnStartupServer()
{
    ConVar.Find("citadel_allow_duplicate_heroes")?.SetInt(1);
    ConVar.Find("citadel_player_starting_gold")?.SetInt(0);
}
```

## Precaching

```csharp
public override void OnPrecacheResources()
{
    Precache.AddResource("particles/upgrades/mystical_piano_hit.vpcf");
}
```

## Config

```csharp
public class MyConfig
{
    public int RoundDuration { get; set; } = 300;
}

[PluginConfig]
public MyConfig Config { get; set; } = new();
```

## Common ConVars

| ConVar | Purpose |
|--------|---------|
| `citadel_allow_duplicate_heroes` | Allow same hero on both teams |
| `citadel_player_starting_gold` | Starting gold amount |
| `citadel_trooper_spawn_enabled` | Enable/disable troopers |

Search the codebase and official docs for the full list.

## Console Commands (Deadworks)

| Command | Purpose |
|---------|---------|
| `dw_plugin` | Plugin management |
| `dw_reloadconfig` | Reload plugin configs |

## Thread Safety

**Rule:** After any `await`, `Task.Delay`, `Task.Run`, or file/HTTP I/O, use `Timer.NextTick` before touching entities, players, or game state.
