using System.Numerics;

namespace DeadworksManaged.Api;

/// <summary>Axis-aligned bounding box helpers.</summary>
public static class BoundingBox {
	/// <summary>Returns true if <paramref name="point"/> lies within the AABB defined by <paramref name="mins"/> and <paramref name="maxs"/> (inclusive).</summary>
	public static bool Contains(Vector3 mins, Vector3 maxs, Vector3 point) =>
		point.X >= mins.X && point.X <= maxs.X &&
		point.Y >= mins.Y && point.Y <= maxs.Y &&
		point.Z >= mins.Z && point.Z <= maxs.Z;
}
