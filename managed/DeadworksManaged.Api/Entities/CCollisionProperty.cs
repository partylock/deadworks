using System.Numerics;

namespace DeadworksManaged.Api;

public class CCollisionProperty : NativeEntity {
	private readonly CBaseEntity _owner;

	private static ReadOnlySpan<byte> Class => "CCollisionProperty"u8;
	private static readonly SchemaAccessor<Vector3> _vecMins = new(Class, "m_vecMins"u8);
	private static readonly SchemaAccessor<Vector3> _vecMaxs = new(Class, "m_vecMaxs"u8);
	private static readonly SchemaAccessor<float> _flBoundingRadius = new(Class, "m_flBoundingRadius"u8);

	internal CCollisionProperty(nint handle, CBaseEntity owner) : base(handle) {
		_owner = owner;
	}

	public override bool IsValid => Handle != 0 && _owner.IsValid;

	/// <summary>The owning entity.</summary>
	public CBaseEntity Owner => _owner;

	/// <summary>Lower OBB corner in local space.</summary>
	public Vector3 Mins { get => _vecMins.Get(Handle); set => _vecMins.Set(Handle, value); }

	/// <summary>Upper OBB corner in local space.</summary>
	public Vector3 Maxs { get => _vecMaxs.Get(Handle); set => _vecMaxs.Set(Handle, value); }

	public float BoundingRadius => _flBoundingRadius.Get(Handle);

	/// <summary>Mins in world space (local Mins + owner's AbsOrigin). Assumes identity rotation.</summary>
	public Vector3 WorldMins => Mins + _owner.Position;

	/// <summary>Maxs in world space (local Maxs + owner's AbsOrigin). Assumes identity rotation.</summary>
	public Vector3 WorldMaxs => Maxs + _owner.Position;

	/// <summary>Returns true if <paramref name="point"/> lies within the entity's world-space AABB (inclusive).</summary>
	public bool Contains(Vector3 point) => BoundingBox.Contains(WorldMins, WorldMaxs, point);
}
