using DeadworksManaged.Api;

namespace DeadworksManaged;

/// <summary>
/// Concrete IHandle implementation wrapping a ScheduledTask.
/// </summary>
internal sealed class TimerHandle : IHandle
{
    private readonly ScheduledTask _task;
    private volatile bool _finished;

    public TimerHandle(ScheduledTask task)
    {
        _task = task;
    }

    public bool IsFinished => _finished;
    public bool ShouldCancelOnMapChange { get; private set; }

    public void Cancel()
    {
        _task.Cancelled = true;
        NotifyFinished();
    }

    public IHandle CancelOnMapChange()
    {
        ShouldCancelOnMapChange = true;
        return this;
    }

    internal void NotifyFinished()
    {
        _finished = true;
    }
}
