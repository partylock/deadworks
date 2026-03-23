namespace DeadworksManaged.Api;

/// <summary>
/// A handle to a scheduled timer. Allows cancellation and status checking.
/// </summary>
public interface IHandle
{
    /// <summary>Cancel this timer. If already finished, this is a no-op.</summary>
    void Cancel();

    /// <summary>Whether this timer has completed or been cancelled.</summary>
    bool IsFinished { get; }

    /// <summary>
    /// Marks this timer to be automatically cancelled on map change (when OnStartupServer fires).
    /// Returns itself for fluent chaining.
    /// </summary>
    IHandle CancelOnMapChange();
}
