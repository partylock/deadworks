namespace DeadworksManaged.Api;

/// <summary>
/// Fired per-player during CheckTransmit. Use <see cref="Hide"/> to prevent
/// entities from being networked to this player for the current tick.
/// </summary>
public sealed unsafe class CheckTransmitEvent {
    private readonly ulong* _transmitBits;

    internal CheckTransmitEvent(int playerSlot, nint transmitBits) {
        PlayerSlot = playerSlot;
        _transmitBits = (ulong*)transmitBits;
    }

    /// <summary>The player slot this transmit list belongs to.</summary>
    public int PlayerSlot { get; }

    /// <summary>The player controller for this slot, or null if unavailable.</summary>
    public CCitadelPlayerController? Player {
        get {
            var ptr = NativeInterop.GetPlayerController(PlayerSlot);
            return ptr != null ? new CCitadelPlayerController((nint)ptr) : null;
        }
    }

    /// <summary>
    /// Prevents an entity from being networked to this player for the current tick.
    /// Must be called every tick the entity should remain hidden.
    /// </summary>
    public void Hide(CBaseEntity entity) {
        int index = entity.EntityIndex;
        _transmitBits[index >> 6] &= ~(1UL << (index & 63));
    }

    /// <summary>
    /// Checks whether an entity is currently set to be transmitted to this player.
    /// </summary>
    public bool IsTransmitting(CBaseEntity entity) {
        int index = entity.EntityIndex;
        return (_transmitBits[index >> 6] & (1UL << (index & 63))) != 0;
    }
}
