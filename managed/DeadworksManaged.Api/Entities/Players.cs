namespace DeadworksManaged.Api;

/// <summary>Static helpers to enumerate all connected player controllers and pawns.</summary>
public static class Players {
	/// <summary>Maximum number of player slots on the server.</summary>
	public const int MaxSlot = 31;

	/// <summary>Returns all currently connected player controllers.</summary>
	public static unsafe IEnumerable<CCitadelPlayerController> GetAll() {
		var list = new List<CCitadelPlayerController>();
		for (int i = 0; i < MaxSlot; i++) {
			var ptr = NativeInterop.GetPlayerController(i);
			if (ptr != null)
				list.Add(new CCitadelPlayerController((nint)ptr));
		}
		return list;
	}

	/// <summary>Returns the hero pawn for every connected player that has one.</summary>
	public static unsafe IEnumerable<CCitadelPlayerPawn> GetAllPawns() {
		var list = new List<CCitadelPlayerPawn>();
		for (int i = 0; i < MaxSlot; i++) {
			var ptr = NativeInterop.GetPlayerController(i);
			if (ptr == null) continue;
			var pawn = NativeInterop.GetHeroPawn(ptr);
			if (pawn != null)
				list.Add(new CCitadelPlayerPawn((nint)pawn));
		}
		return list;
	}

	/// <summary>Returns the player controller in the given slot, or null if the slot is empty.</summary>
	public static unsafe CCitadelPlayerController? FromSlot(int slot) {
		if ((uint)slot >= MaxSlot) return null;
		var ptr = NativeInterop.GetPlayerController(slot);
		return ptr != null ? new CCitadelPlayerController((nint)ptr) : null;
	}
}
