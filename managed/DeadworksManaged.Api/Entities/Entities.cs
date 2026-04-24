namespace DeadworksManaged.Api;

/// <summary>Queries over the server's entity list.</summary>
public static class Entities {
	// MAX_ENTITY_LISTS (64) * MAX_ENTITIES_IN_LIST (512)
	private const int MaxTotalEntities = 32768;

	/// <summary>All valid entities on the server.</summary>
	public static IEnumerable<CBaseEntity> All {
		get {
			var list = new List<CBaseEntity>();
			for (int i = 0; i < MaxTotalEntities; i++) {
				nint ptr;
				unsafe { ptr = (nint)NativeInterop.GetEntityByIndex(i); }
				if (ptr != nint.Zero)
					list.Add(new CBaseEntity(ptr));
			}
			return list;
		}
	}

	/// <summary>All entities whose native DLL class matches <typeparamref name="T"/>.</summary>
	public static IEnumerable<T> ByClass<T>() where T : CBaseEntity {
		var list = new List<T>();
		for (int i = 0; i < MaxTotalEntities; i++) {
			nint ptr;
			unsafe { ptr = (nint)NativeInterop.GetEntityByIndex(i); }
			if (ptr == nint.Zero) continue;
			var entity = new CBaseEntity(ptr);
			if (entity.Is<T>())
				list.Add(NativeEntityFactory.Create<T>(ptr));
		}
		return list;
	}

	/// <summary>All entities whose designer name equals <paramref name="name"/> (ordinal).</summary>
	public static IEnumerable<CBaseEntity> ByDesignerName(string name) {
		var list = new List<CBaseEntity>();
		for (int i = 0; i < MaxTotalEntities; i++) {
			nint ptr;
			unsafe { ptr = (nint)NativeInterop.GetEntityByIndex(i); }
			if (ptr == nint.Zero) continue;
			var entity = new CBaseEntity(ptr);
			if (string.Equals(entity.DesignerName, name, System.StringComparison.Ordinal))
				list.Add(entity);
		}
		return list;
	}

	/// <summary>First entity whose targetname equals <paramref name="name"/> (case-sensitive), or null.</summary>
	public static unsafe CBaseEntity? FirstByName(string name) {
		Span<byte> utf8 = Utf8.Encode(name, stackalloc byte[Utf8.Size(name)]);
		fixed (byte* ptr = utf8) {
			var result = (nint)NativeInterop.FindEntityByName(null, ptr);
			return result != 0 ? new CBaseEntity(result) : null;
		}
	}

	/// <summary>First entity with matching targetname whose native class matches <typeparamref name="T"/>, or null.</summary>
	public static unsafe T? FirstByName<T>(string name) where T : CBaseEntity {
		Span<byte> utf8 = Utf8.Encode(name, stackalloc byte[Utf8.Size(name)]);
		fixed (byte* ptr = utf8) {
			var result = (nint)NativeInterop.FindEntityByName(null, ptr);
			if (result == 0) return null;
			var entity = new CBaseEntity(result);
			return NativeEntityFactory.IsMatch<T>(entity.Classname) ? NativeEntityFactory.Create<T>(result) : null;
		}
	}

	/// <summary>All entities whose targetname equals <paramref name="name"/> (case-sensitive).</summary>
	public static unsafe IEnumerable<CBaseEntity> ByName(string name) {
		var list = new List<CBaseEntity>();
		Span<byte> utf8 = Utf8.Encode(name, stackalloc byte[Utf8.Size(name)]);
		fixed (byte* ptr = utf8) {
			void* cursor = null;
			while ((cursor = NativeInterop.FindEntityByName(cursor, ptr)) != null)
				list.Add(new CBaseEntity((nint)cursor));
		}
		return list;
	}

	/// <summary>All matching entities whose native class matches <typeparamref name="T"/>.</summary>
	public static unsafe IEnumerable<T> ByName<T>(string name) where T : CBaseEntity {
		var list = new List<T>();
		Span<byte> utf8 = Utf8.Encode(name, stackalloc byte[Utf8.Size(name)]);
		fixed (byte* ptr = utf8) {
			void* cursor = null;
			while ((cursor = NativeInterop.FindEntityByName(cursor, ptr)) != null) {
				var entity = new CBaseEntity((nint)cursor);
				if (NativeEntityFactory.IsMatch<T>(entity.Classname))
					list.Add(NativeEntityFactory.Create<T>((nint)cursor));
			}
		}
		return list;
	}
}
