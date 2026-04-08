using System.Numerics;

namespace DeadworksManaged.Api;

/// <summary>
/// Wraps a native CEntityKeyValues handle. Create with <c>new CEntityKeyValues()</c>,
/// set typed members, pass to <see cref="CBaseEntity.Spawn(CEntityKeyValues)"/>.
/// The engine takes ownership of the underlying allocation - do not dispose manually.
/// </summary>
public sealed unsafe class CEntityKeyValues
{
	internal void* Handle { get; private set; }

	/// <summary><see langword="true"/> if the underlying native handle is still alive.</summary>
	public bool IsValid => Handle != null;

	/// <summary>Allocates a new native CEntityKeyValues object.</summary>
	public CEntityKeyValues()
	{
		Handle = NativeInterop.CreateEntityKeyValues();
	}

	/// <summary>Sets a string value.</summary>
	public void SetString(string key, string value)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		Span<byte> valUtf8 = Utf8.Encode(value, stackalloc byte[Utf8.Size(value)]);
		fixed (byte* keyPtr = keyUtf8, valPtr = valUtf8)
		{
			NativeInterop.EKVSetString(Handle, keyPtr, valPtr);
		}
	}

	/// <summary>Sets a boolean value.</summary>
	public void SetBool(string key, bool value)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		fixed (byte* keyPtr = keyUtf8)
		{
			NativeInterop.EKVSetBool(Handle, keyPtr, value ? (byte)1 : (byte)0);
		}
	}

	/// <summary>Sets a 3D vector value.</summary>
	public void SetVector(string key, Vector3 value)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		fixed (byte* keyPtr = keyUtf8)
		{
			NativeInterop.EKVSetVector(Handle, keyPtr, value.X, value.Y, value.Z);
		}
	}

	/// <summary>Sets a single-precision floating-point value.</summary>
	public void SetFloat(string key, float value)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		fixed (byte* keyPtr = keyUtf8)
		{
			NativeInterop.EKVSetFloat(Handle, keyPtr, value);
		}
	}

	/// <summary>Sets a signed 32-bit integer value.</summary>
	public void SetInt(string key, int value)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		fixed (byte* keyPtr = keyUtf8)
		{
			NativeInterop.EKVSetInt(Handle, keyPtr, value);
		}
	}

	/// <summary>Sets a color value (RGBA).</summary>
	public void SetColor(string key, byte r, byte g, byte b, byte a = 255)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		fixed (byte* keyPtr = keyUtf8)
		{
			NativeInterop.EKVSetColor(Handle, keyPtr, r, g, b, a);
		}
	}

	/// <summary>Sets a string token value (CUtlStringToken). The token hash is computed from <paramref name="tokenString"/>.</summary>
	public void SetStringToken(string key, string tokenString)
	{
		ThrowIfInvalid();
		Span<byte> keyUtf8 = Utf8.Encode(key, stackalloc byte[Utf8.Size(key)]);
		Span<byte> valUtf8 = Utf8.Encode(tokenString, stackalloc byte[Utf8.Size(tokenString)]);
		fixed (byte* keyPtr = keyUtf8, valPtr = valUtf8)
		{
			NativeInterop.EKVSetStringToken(Handle, keyPtr, valPtr);
		}
	}

	private void ThrowIfInvalid()
	{
		if (Handle == null)
			throw new InvalidOperationException("CEntityKeyValues handle is not valid.");
	}
}
