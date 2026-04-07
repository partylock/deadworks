namespace DeadworksManaged.Api;

/// <summary>
/// Per-plugin timer service. Access via <c>this.Timer</c> in any <see cref="IDeadworksPlugin"/>.
/// </summary>
public interface ITimer
{
    /// <summary>Execute a callback once after the specified delay.</summary>
    IHandle Once(Duration delay, Action callback);

    /// <summary>Execute a callback repeatedly at the specified interval.</summary>
    IHandle Every(Duration interval, Action callback);

    /// <summary>
    /// Run a stateful sequence. The callback receives an <see cref="IStep"/> and returns
    /// a <see cref="Pace"/> to control when the next invocation occurs.
    /// </summary>
    IHandle Sequence(Func<IStep, Pace> callback);

    /// <summary>
    /// Defer an action to the next tick. Thread-safe - can be called from any thread.
    /// </summary>
    void NextTick(Action callback);
}
