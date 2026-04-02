namespace DeadworksManaged.Api;

/// <summary>Fired when a client is connecting to the server. Passed to <see cref="IDeadworksPlugin.OnClientConnect"/>.</summary>
public sealed class ClientConnectEvent {
	public required int Slot { get; init; }
	public required string Name { get; init; }
	public required ulong SteamId { get; init; }
	public required string IpAddress { get; init; }
}
