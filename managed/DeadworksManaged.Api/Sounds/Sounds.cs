namespace DeadworksManaged.Api.Sounds;

/// <summary>Convenience wrappers for the most common <see cref="SoundEvent"/> usages.</summary>
public static class Sounds
{
	/// <summary>Plays a soundevent at the listener's own position (non-positional from the client's view).</summary>
	/// <returns>The soundevent GUID returned by <see cref="SoundEvent.Emit"/>.</returns>
	public static uint Play(string name, RecipientFilter recipients, float volume = 1f, float pitch = 1f)
	{
		return new SoundEvent(name)
		{
			Volume = volume,
			Pitch = pitch,
		}.Emit(recipients);
	}

	/// <summary>Plays a soundevent anchored to an entity (uses that entity's world position for spatialization).</summary>
	/// <returns>The soundevent GUID returned by <see cref="SoundEvent.Emit"/>.</returns>
	public static uint PlayAt(string name, int sourceEntityIndex, RecipientFilter recipients, float volume = 1f, float pitch = 1f)
	{
		return new SoundEvent(name)
		{
			SourceEntityIndex = sourceEntityIndex,
			Volume = volume,
			Pitch = pitch,
		}.Emit(recipients);
	}
}
