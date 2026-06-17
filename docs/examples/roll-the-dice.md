# Example: Roll The Dice

Official docs: https://docs.deadworks.net/examples/roll-the-dice/

Source: `server/deadworks/examples/plugins/RollTheDicePlugin/`

## What It Does

- Player runs `/rtd` or `!rtd`
- Plugin rolls a random effect and shows a HUD announcement
- **Mystical Piano Strike**: warning sound → 1.7s delay → particle + impact sound + 3s knockdown
- Particle auto-cleans after 5 seconds

## Key Concepts

| Concept | API |
|---------|-----|
| Command | `[Command("rtd")]` |
| HUD announcement | `CCitadelUserMsg_HudGameAnnouncement` + `NetMessages.Send` |
| Sound | `pawn.EmitSound(...)` |
| Particles | `CParticleSystem.Create(...).AtPosition(...).Spawn()` |
| Modifier | `pawn.AddModifier("modifier_citadel_knockdown", kv)` |
| Delayed work | `Timer.Once(...)` |
| Precaching | `Precache.AddResource` in `OnPrecacheResources` |

## Build & Deploy

```bash
cd server/deadworks/examples/plugins/RollTheDicePlugin
dotnet build
```

Requires `server/deadworks/local.props` with a valid `DeadlockDir`. DLL deploys to `{DeadlockDir}/managed/plugins/`.

## Extending

Add more rolls to the `effects` array:

```csharp
var effects = new (string Name, Action<CCitadelPlayerPawn> Apply)[] {
    ("Mystical Piano Strike", ApplyPianoStrike),
    ("Your Effect", ApplyYourEffect),
};
```
