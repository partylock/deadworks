namespace DeadworksManaged;

/// <summary>
/// Central index of everything a plugin has registered (commands, convars, chat commands,
/// event handlers, net message handlers). Each subsystem calls <see cref="Add"/> at
/// registration time; <see cref="Remove"/> bulk-clears on unload. Dispatch stays in the
/// owning subsystem - this is metadata only.
/// </summary>
internal static class PluginRegistrationTracker
{
    public readonly record struct Entry(string Kind, string Name, string Description);

    private static readonly Dictionary<string, List<Entry>> _entries = new(StringComparer.OrdinalIgnoreCase);
    private static readonly Lock _lock = new();

    public static void Add(string normalizedPath, string kind, string name, string description = "")
    {
        lock (_lock)
        {
            if (!_entries.TryGetValue(normalizedPath, out var list))
            {
                list = [];
                _entries[normalizedPath] = list;
            }
            list.Add(new Entry(kind, name, description));
        }
    }

    public static void Remove(string normalizedPath)
    {
        lock (_lock)
        {
            _entries.Remove(normalizedPath);
        }
    }

    public static List<Entry> GetEntries(string normalizedPath)
    {
        lock (_lock)
        {
            return _entries.TryGetValue(normalizedPath, out var list) ? [.. list] : [];
        }
    }

    public static void Clear()
    {
        lock (_lock)
        {
            _entries.Clear();
        }
    }
}
