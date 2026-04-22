using System.Buffers.Binary;
using Google.Protobuf;

namespace DeadworksManaged.Api.Sounds;

/// <summary>
/// Builds and emits an SOS sound event to clients.
/// </summary>
/// <example>
/// <code>
/// new SoundEvent("UI.SomeSoundName") { Volume = 0.8f }
///     .Emit(RecipientFilter.Single(playerSlot));
/// </code>
/// </example>
public sealed class SoundEvent
{
	private readonly Dictionary<string, SoundEventField> _fields = new(StringComparer.Ordinal);

	/// <summary>The soundevent name as defined in a <c>.vsndevts</c> manifest.</summary>
	public string Name { get; set; }

	/// <summary>Entity index the sound emits from. <c>-1</c> plays at the listener's own position.</summary>
	public int SourceEntityIndex { get; set; } = -1;

	/// <summary>Optional start-time offset for the event.</summary>
	public float StartTime { get; set; }

	/// <summary>Shortcut for <c>SetFloat("public.volume", ...)</c>.</summary>
	public float Volume
	{
		set => SetFloat("public.volume", value);
	}

	/// <summary>Shortcut for <c>SetFloat("public.pitch", ...)</c>.</summary>
	public float Pitch
	{
		set => SetFloat("public.pitch", value);
	}

	public SoundEvent(string name)
	{
		if (string.IsNullOrEmpty(name))
			throw new ArgumentException("Soundevent name must not be empty.", nameof(name));
		Name = name;
	}

	public SoundEvent SetBool(string fieldName, bool value)
	{
		_fields[fieldName] = SoundEventField.FromBool(value);
		return this;
	}

	public SoundEvent SetInt32(string fieldName, int value)
	{
		_fields[fieldName] = SoundEventField.FromInt32(value);
		return this;
	}

	public SoundEvent SetUInt32(string fieldName, uint value)
	{
		_fields[fieldName] = SoundEventField.FromUInt32(value);
		return this;
	}

	public SoundEvent SetUInt64(string fieldName, ulong value)
	{
		_fields[fieldName] = SoundEventField.FromUInt64(value);
		return this;
	}

	public SoundEvent SetFloat(string fieldName, float value)
	{
		_fields[fieldName] = SoundEventField.FromFloat(value);
		return this;
	}

	public SoundEvent SetFloat3(string fieldName, float x, float y, float z)
	{
		_fields[fieldName] = SoundEventField.FromFloat3(x, y, z);
		return this;
	}

	public bool HasField(string fieldName) => _fields.ContainsKey(fieldName);

	public bool TryGetFloat(string fieldName, out float value)
	{
		if (_fields.TryGetValue(fieldName, out var f) && f.Type == SosFieldType.Float)
		{
			value = f.AsFloat;
			return true;
		}
		value = 0f;
		return false;
	}

	public bool TryGetInt32(string fieldName, out int value)
	{
		if (_fields.TryGetValue(fieldName, out var f) && f.Type == SosFieldType.Int32)
		{
			value = f.AsInt32;
			return true;
		}
		value = 0;
		return false;
	}

	public bool TryGetBool(string fieldName, out bool value)
	{
		if (_fields.TryGetValue(fieldName, out var f) && f.Type == SosFieldType.Bool)
		{
			value = f.AsBool;
			return true;
		}
		value = false;
		return false;
	}

	/// <summary>
	/// Emits the sound to the given recipients. Returns the allocated GUID
	/// </summary>
	public unsafe uint Emit(RecipientFilter recipients)
	{
		uint guid = NativeInterop.TakeSoundEventGuid != null
			? NativeInterop.TakeSoundEventGuid()
			: 0u;

		byte[] packed = PackFields();

		var msg = new CMsgSosStartSoundEvent
		{
			SoundeventHash = MurmurHash2.HashLowerCase(Name, SosHashSeeds.SoundeventName),
			SoundeventGuid = unchecked((int)guid),
			Seed = unchecked((int)guid),
			SourceEntityIndex = SourceEntityIndex,
			StartTime = StartTime,
			PackedParams = ByteString.CopyFrom(packed),
		};

		NetMessages.Send(msg, recipients);
		return guid;
	}

	/// <summary>
	/// Updates the params of a playing sound identified by <paramref name="guid"/>
	/// with this builder's current fields.
	/// </summary>
	public void SetParams(uint guid, RecipientFilter recipients)
	{
		byte[] packed = PackFields();
		var msg = new CMsgSosSetSoundEventParams
		{
			SoundeventGuid = unchecked((int)guid),
			PackedParams = ByteString.CopyFrom(packed),
		};
		NetMessages.Send(msg, recipients);
	}

	/// <summary>Stops a specific playing sound by its GUID.</summary>
	public static void Stop(uint guid, RecipientFilter recipients)
	{
		var msg = new CMsgSosStopSoundEvent
		{
			SoundeventGuid = unchecked((int)guid),
		};
		NetMessages.Send(msg, recipients);
	}

	/// <summary>Stops all playing instances of the named soundevent for the given source entity.</summary>
	public static void StopByName(string soundeventName, int sourceEntityIndex, RecipientFilter recipients)
	{
		if (string.IsNullOrEmpty(soundeventName))
			throw new ArgumentException("Soundevent name must not be empty.", nameof(soundeventName));

		var msg = new CMsgSosStopSoundEventHash
		{
			SoundeventHash = MurmurHash2.HashLowerCase(soundeventName, SosHashSeeds.SoundeventName),
			SourceEntityIndex = sourceEntityIndex,
		};
		NetMessages.Send(msg, recipients);
	}

	/// <summary>
	/// Serializes the current fields into the SOS packed-params wire format.
	/// Per field: <c>[4B LE hash][1B type][1B payload size][1B pad=0][N B LE payload]</c>.
	/// </summary>
	internal byte[] PackFields()
	{
		int total = 0;
		foreach (var kv in _fields)
			total += 7 + kv.Value.PayloadSize;

		if (total == 0)
			return Array.Empty<byte>();

		byte[] buf = new byte[total];
		Span<byte> span = buf;
		int pos = 0;

		foreach (var (fieldName, field) in _fields)
		{
			uint nameHash = MurmurHash2.HashLowerCase(fieldName, SosHashSeeds.FieldName);
			BinaryPrimitives.WriteUInt32LittleEndian(span.Slice(pos, 4), nameHash);
			pos += 4;

			span[pos++] = (byte)field.Type;
			span[pos++] = (byte)field.PayloadSize;
			span[pos++] = 0; // pad

			field.WritePayload(span.Slice(pos, field.PayloadSize));
			pos += field.PayloadSize;
		}

		return buf;
	}
}
