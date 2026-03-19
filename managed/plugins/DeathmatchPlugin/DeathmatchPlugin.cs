using System.Numerics;
using System.Text.Json.Serialization;
using DeadworksManaged.Api;

namespace DeathmatchPlugin;

public class SpawnPoint {
	[JsonPropertyName("pos")]
	public float[] Pos { get; set; } = [0, 0, 0];

	[JsonPropertyName("ang")]
	public float[] Ang { get; set; } = [0, 0, 0];
}

public class DeathmatchConfig {
	public Dictionary<string, Dictionary<string, SpawnPoint[]>> SpawnPoints { get; set; } = new();
}

public class DeathmatchPlugin : DeadworksPluginBase {
	public override string Name => "Deathmatch";

	[PluginConfig]
	public DeathmatchConfig Config { get; set; } = new();

	private readonly Dictionary<int, string> _playerNames = new();
	private bool _rebroadcasting;

	public override void OnLoad(bool isReload) {
		Console.WriteLine(isReload ? "Deathmatch reloaded!" : "Deathmatch loaded!");
	}

	public override void OnClientPutInServer(ClientPutInServerEvent args) {
		_playerNames[args.Slot] = args.Name;
	}

	[ChatCommand("pos")]
	public HookResult CmdPos(ChatCommandContext ctx) {
		var pawn = ctx.Controller?.GetHeroPawn()?.As<CCitadelPlayerPawn>();
		if (pawn == null) return HookResult.Handled;
		var pos = pawn.Position;
		var ang = pawn.CameraAngles;
		Console.WriteLine($@"{{ ""pos"": [{pos.X}, {pos.Y}, {pos.Z}], ""ang"": [{ang.X}, {ang.Y}, {ang.Z}] }}");
		return HookResult.Handled;
	}

	[GameEventHandler("player_respawned")]
	public HookResult OnPlayerRespawned(PlayerRespawnedEvent args) {
		var pawn = args.Userid;
		if (pawn == null) return HookResult.Continue;

		var teamKey = pawn.TeamNum.ToString();
		if (Config.SpawnPoints.TryGetValue(Server.MapName, out var teams)
			&& teams.TryGetValue(teamKey, out var spawns)
			&& spawns.Length > 0) {
			var spawn = spawns[Random.Shared.Next(spawns.Length)];
			var pos = spawn.Pos.Length >= 3 ? new Vector3(spawn.Pos[0], spawn.Pos[1], spawn.Pos[2]) : (Vector3?)null;
			var ang = spawn.Ang.Length >= 3 ? new Vector3(spawn.Ang[0], spawn.Ang[1], spawn.Ang[2]) : (Vector3?)null;
			pawn.Teleport(position: pos, angles: ang);
		}

		return HookResult.Continue;
	}

	public override HookResult OnClientConCommand(ClientConCommandEvent e) {
		Console.WriteLine($"[ConCmd] {e.Command} (args: {string.Join(", ", e.Args)})");
		if (e.Command == "selecthero") {
			var pawn = e.Controller?.GetHeroPawn()?.As<CCitadelPlayerPawn>();
			// Health is a silly heuristic
			if (pawn != null && !pawn.InRegenerationZone && pawn.Health > 0) {
				var controller = e.Controller;
				if (controller != null) {
					int slot = controller.EntityIndex - 1;
					var msg = new CCitadelUserMsg_ChatMsg {
						PlayerSlot = slot,
						Text = "[server] You can only change heroes while in spawn",
						AllChat = true,
					};
					NetMessages.Send(msg, RecipientFilter.Single(slot));
				}
				return HookResult.Stop;
			}
		}
		return HookResult.Continue;
	}

	/// <summary>
	/// Rebroadcasts chat messages so each recipient sees the message as coming from
	/// their own player slot (guaranteed to have a portrait), with the actual sender's
	/// name prefixed in the text. This works around Deadlock's 12-slot portrait limit.
	/// </summary>
	[NetMessageHandler]
	public HookResult OnChatMsgOutgoing(OutgoingMessageContext<CCitadelUserMsg_ChatMsg> ctx) {
		// Reentrancy guard — our own Send calls trigger this hook again
		if (_rebroadcasting) return HookResult.Continue;

		var senderSlot = ctx.Message.PlayerSlot;

		// Let system/server messages pass through unchanged
		if (senderSlot < 0) return HookResult.Continue;

		var text = ctx.Message.Text;
		var allChat = ctx.Message.AllChat;
		var laneColor = ctx.Message.LaneColor;
		var originalMask = ctx.Recipients.Mask;

		var senderName = _playerNames.GetValueOrDefault(senderSlot, $"Player {senderSlot}");

		// Rebroadcast to each recipient individually with their own slot
		_rebroadcasting = true;
		try {
			for (int slot = 0; slot < 64; slot++) {
				if ((originalMask & (1UL << slot)) == 0) continue;

				var msg = new CCitadelUserMsg_ChatMsg {
					PlayerSlot = slot,
					Text = slot == senderSlot ? text : $"[{senderName}]: {text}",
					AllChat = allChat,
					LaneColor = laneColor
				};
				NetMessages.Send(msg, RecipientFilter.Single(slot));
			}
		} finally {
			_rebroadcasting = false;
		}

		// Suppress the original broadcast
		return HookResult.Stop;
	}

	public override void OnUnload() => Console.WriteLine("Deathmatch unloaded!");

	public override void OnPrecacheResources() {
	}

	public override void OnStartupServer() {
		ConVar.Find("citadel_active_lane")?.SetInt(4);
		ConVar.Find("citadel_player_spawn_time_max_respawn_time")?.SetInt(5);
		ConVar.Find("citadel_allow_purchasing_anywhere")?.SetInt(1);
		ConVar.Find("citadel_item_sell_price_ratio")?.SetFloat(1.0f);
		ConVar.Find("citadel_voice_all_talk")?.SetInt(1);
		ConVar.Find("citadel_player_starting_gold")?.SetInt(0);
		ConVar.Find("citadel_trooper_spawn_enabled")?.SetInt(0);
		ConVar.Find("citadel_npc_spawn_enabled")?.SetInt(0);
		ConVar.Find("citadel_start_players_on_zipline")?.SetInt(0);
		ConVar.Find("citadel_allow_duplicate_heroes")?.SetInt(1);
	}

	public override HookResult OnTakeDamage(TakeDamageEvent args) {
		if (args.Entity.DesignerName == "npc_boss_tier3" || args.Entity.DesignerName == "npc_boss_tier2" || args.Entity.DesignerName == "npc_trooper_boss")
			return HookResult.Stop;
		return HookResult.Continue;
	}

	public override HookResult OnModifyCurrency(ModifyCurrencyEvent args) {
		if (args.CurrencyType == ECurrencyType.EGold) {
			if (args.Source == ECurrencySource.EStartingAmount) {
				// Trigger boons by reissuing as non-starting amount
				args.Pawn.ModifyCurrency(ECurrencyType.EGold, 15_000, ECurrencySource.ECheats);
				args.Pawn.ModifyCurrency(ECurrencyType.EAbilityPoints, 17, ECurrencySource.ECheats);
				return HookResult.Stop;
			}
			if (args.Source != ECurrencySource.ECheats && args.Source != ECurrencySource.EItemPurchase && args.Source != ECurrencySource.EItemSale)
				return HookResult.Stop;
		}
		return HookResult.Continue;
	}

	[GameEventHandler("player_hero_changed")]
	public HookResult OnPlayerHeroChanged(PlayerHeroChangedEvent args) {
		var pawn = args.Userid?.As<CCitadelPlayerPawn>();
		if (pawn != null) {
			// Otherwise AP carries
			pawn.ResetHero();
			pawn.Heal(pawn.GetMaxHealth());
		}

		return HookResult.Continue;
	}

	public override void OnEntitySpawned(EntitySpawnedEvent e) {
		var designerNamesToRemove = new HashSet<string>() { "npc_trooper_boss" };
		var namesToRemove = new HashSet<string>() {
		};
		if (designerNamesToRemove.Contains(e.Entity.DesignerName) || namesToRemove.Contains(e.Entity.Name)) {
			e.Entity.Remove();
		}
	}

	public override void OnClientFullConnect(ClientFullConnectEvent args) {
		var controller = args.Controller;
		if (controller == null) return;

		int team2 = 0, team3 = 0;
		for (int i = 0; i < 64; i++) {
			var ent = CBaseEntity.FromIndex(i + 1);
			if (ent == null) continue;
			if (ent.TeamNum == 2) team2++;
			else if (ent.TeamNum == 3) team3++;
		}
		int team = team2 < team3 ? 2 : team3 < team2 ? 3 : Random.Shared.Next(2) == 0 ? 2 : 3;
		Console.WriteLine($"Assigning {args.Slot} to team {team}");
		controller.ChangeTeam(team);

		var heroes = Enum.GetValues<Heroes>()
			.Where(h => h.GetHeroData()?.AvailableInGame == true)
			.ToArray();
		var hero = heroes[Random.Shared.Next(heroes.Length)];

		Console.WriteLine($"Assigning {args.Slot} to hero {hero.ToHeroName()}");
		controller.SelectHero(hero);
	}

	public override void OnClientDisconnect(ClientDisconnectedEvent args) {
		_playerNames.Remove(args.Slot);

		var controller = args.Controller;
		if (controller == null) return;

		var pawn = controller.GetHeroPawn();
		if (pawn == null) return;

		pawn.Remove();
	}
}
