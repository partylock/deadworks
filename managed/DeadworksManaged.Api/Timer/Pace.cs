namespace DeadworksManaged.Api;

/// <summary>
/// Controls the flow of a timer sequence step.
/// Created via <see cref="IStep"/> methods - not instantiated directly.
/// </summary>
public abstract class Pace
{
    internal Pace() { }
}

/// <summary>Execute again after the specified duration.</summary>
internal sealed class WaitPace : Pace
{
    public Duration Delay { get; }
    internal WaitPace(Duration delay) => Delay = delay;
}

/// <summary>The sequence is finished.</summary>
internal sealed class DonePace : Pace
{
    internal static readonly DonePace Instance = new();
    internal DonePace() { }
}
