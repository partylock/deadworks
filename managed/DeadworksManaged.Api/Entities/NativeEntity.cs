namespace DeadworksManaged.Api;

/// <summary>Base class for all managed wrappers around native C++ entity/object pointers.</summary>
public abstract class NativeEntity {
	private readonly nint _handle;

	/// <summary>Raw pointer to the native object. Subclasses that use handle-based identity override this to re-resolve each access.</summary>
	public virtual nint Handle => _handle;

	/// <summary>True if the pointer is non-null.</summary>
	public virtual bool IsValid => Handle != 0;

	protected NativeEntity(nint handle) => _handle = handle;

	/// <summary>For subclasses that resolve Handle dynamically and do not store a raw pointer.</summary>
	protected NativeEntity() { }
}
