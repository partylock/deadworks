namespace DeadworksManaged.Api;

/// <summary>Wraps CEntitySubclassVDataBase - the VData subclass pointer stored on an entity, providing its design-time name.</summary>
public sealed unsafe class CEntitySubclassVDataBase : NativeEntity {
	internal CEntitySubclassVDataBase(nint handle) : base(handle) { }

	public string Name {
		get {
			nint pName = *(nint*)((byte*)Handle + 0x10);
			return pName != 0 ? System.Runtime.InteropServices.Marshal.PtrToStringUTF8(pName) ?? "" : "";
		}
	}
}
