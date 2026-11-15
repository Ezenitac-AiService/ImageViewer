namespace ImageViewer.Services;

/// <summary>
/// 개수와 비용(바이트) 상한을 함께 적용하는 스레드 안전 LRU 캐시.
/// </summary>
public sealed class LruCache<TKey, TValue> where TKey : notnull
{
    private readonly int _maxCount;
    private readonly long _maxCost;
    private readonly Func<TValue, long> _costOf;
    private readonly Action<TKey, TValue>? _onEvicted;
    private readonly LinkedList<(TKey Key, TValue Value, long Cost)> _order = new();
    private readonly Dictionary<TKey, LinkedListNode<(TKey Key, TValue Value, long Cost)>> _map = new();
    private readonly Lock _lock = new();
    private long _totalCost;

    public LruCache(int maxCount, long maxCost, Func<TValue, long> costOf, Action<TKey, TValue>? onEvicted = null)
    {
        _maxCount = Math.Max(1, maxCount);
        _maxCost = Math.Max(1, maxCost);
        _costOf = costOf;
        _onEvicted = onEvicted;
    }

    public int Count
    {
        get
        {
            lock (_lock)
            {
                return _map.Count;
            }
        }
    }

    public long TotalCost
    {
        get
        {
            lock (_lock)
            {
                return _totalCost;
            }
        }
    }

    public bool TryGet(TKey key, out TValue value)
    {
        lock (_lock)
        {
            if (_map.TryGetValue(key, out var node))
            {
                _order.Remove(node);
                _order.AddFirst(node);
                value = node.Value.Value;
                return true;
            }
        }

        value = default!;
        return false;
    }

    public void Add(TKey key, TValue value)
    {
        List<(TKey, TValue)>? evicted = null;
        lock (_lock)
        {
            if (_map.TryGetValue(key, out var existing))
            {
                _order.Remove(existing);
                _map.Remove(key);
                _totalCost -= existing.Value.Cost;
                if (!ReferenceEquals(existing.Value.Value, value))
                {
                    (evicted ??= []).Add((key, existing.Value.Value));
                }
            }

            var cost = _costOf(value);
            var node = _order.AddFirst((key, value, cost));
            _map[key] = node;
            _totalCost += cost;

            // 방금 넣은 항목은 남겨 두고, 오래된 항목부터 제거
            while ((_map.Count > _maxCount || _totalCost > _maxCost) && _order.Last is { } last && last != node)
            {
                _order.RemoveLast();
                _map.Remove(last.Value.Key);
                _totalCost -= last.Value.Cost;
                (evicted ??= []).Add((last.Value.Key, last.Value.Value));
            }
        }

        if (evicted is not null && _onEvicted is not null)
        {
            foreach (var (k, v) in evicted)
            {
                _onEvicted(k, v);
            }
        }
    }

    /// <summary>조건에 맞는 키의 항목을 제거한다.</summary>
    public void RemoveWhere(Func<TKey, bool> predicate)
    {
        List<(TKey, TValue)> removed = [];
        lock (_lock)
        {
            foreach (var key in _map.Keys.Where(predicate).ToList())
            {
                var node = _map[key];
                _order.Remove(node);
                _map.Remove(key);
                _totalCost -= node.Value.Cost;
                removed.Add((key, node.Value.Value));
            }
        }

        if (_onEvicted is not null)
        {
            foreach (var (k, v) in removed)
            {
                _onEvicted(k, v);
            }
        }
    }

    public void Clear()
    {
        List<(TKey, TValue)> evicted;
        lock (_lock)
        {
            evicted = _order.Select(e => (e.Key, e.Value)).ToList();
            _order.Clear();
            _map.Clear();
            _totalCost = 0;
        }

        if (_onEvicted is not null)
        {
            foreach (var (k, v) in evicted)
            {
                _onEvicted(k, v);
            }
        }
    }
}
