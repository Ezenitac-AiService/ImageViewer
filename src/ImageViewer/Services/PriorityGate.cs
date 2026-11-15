namespace ImageViewer.Services;

/// <summary>
/// 화면에 바로 보일 작업(전경)의 동시 실행 수를 제한하는 대기열.
/// 이미 시작된 디코딩 · 축소는 중간에 멈출 수 없어서, 빠르게 넘기면 "이미 지나간 이미지"의 작업이
/// 자리를 차지해 지금 보려는 이미지가 그 뒤에서 기다리게 된다.
/// 실행 중인 작업 중 아직 필요한(요청이 취소되지 않은) 작업이 <c>capacity</c> 보다 적으면,
/// 지나간 작업이 끝나기를 기다리지 않고 <c>burstCapacity</c> 까지 바로 시작한다.
/// </summary>
internal sealed class PriorityGate
{
    private readonly int _capacity;
    private readonly int _burstCapacity;
    private readonly List<Entry> _running = [];
    private readonly Lock _lock = new();
    private TaskCompletionSource _changed = NewSignal();

    public PriorityGate(int capacity, int burstCapacity)
    {
        _capacity = Math.Max(1, capacity);
        _burstCapacity = Math.Max(_capacity, burstCapacity);
    }

    private sealed class Entry(CancellationToken owner)
    {
        public CancellationToken Owner { get; } = owner;
    }

    /// <summary>현재 실행 중인 작업 수 (테스트용).</summary>
    internal int RunningCount
    {
        get
        {
            lock (_lock)
            {
                return _running.Count;
            }
        }
    }

    /// <summary>
    /// 자리를 얻을 때까지 기다린다. <paramref name="owner"/> 가 취소되면 이 작업은 "지나간 작업"으로 취급되어
    /// 다른 요청이 그 자리를 넘어설 수 있다. 반환된 객체를 해제하면 자리를 돌려준다.
    /// </summary>
    public async Task<IDisposable> EnterAsync(CancellationToken owner)
    {
        while (true)
        {
            Task signal;
            lock (_lock)
            {
                var needed = _running.Count(e => !e.Owner.IsCancellationRequested);
                if (_running.Count < _capacity || (needed < _capacity && _running.Count < _burstCapacity))
                {
                    var entry = new Entry(owner);
                    _running.Add(entry);
                    return new Releaser(this, entry);
                }

                signal = _changed.Task;
            }

            // 자리가 비거나, 실행 중인 작업의 요청이 취소되어 넘어설 수 있게 될 때까지 짧게 기다린다.
            owner.ThrowIfCancellationRequested();
            await Task.WhenAny(signal, Task.Delay(20, owner)).ConfigureAwait(false);
            owner.ThrowIfCancellationRequested();
        }
    }

    private void Release(Entry entry)
    {
        TaskCompletionSource previous;
        lock (_lock)
        {
            _running.Remove(entry);
            previous = _changed;
            _changed = NewSignal();
        }

        previous.TrySetResult();
    }

    private static TaskCompletionSource NewSignal() => new(TaskCreationOptions.RunContinuationsAsynchronously);

    private sealed class Releaser(PriorityGate gate, Entry entry) : IDisposable
    {
        private int _disposed;

        public void Dispose()
        {
            if (Interlocked.Exchange(ref _disposed, 1) == 0)
            {
                gate.Release(entry);
            }
        }
    }
}

internal static class SemaphoreSlimExtensions
{
    /// <summary>세마포어 자리를 얻고, 해제하면 돌려주는 객체를 반환한다.</summary>
    public static async Task<IDisposable> EnterAsync(this SemaphoreSlim semaphore, CancellationToken cancellationToken)
    {
        await semaphore.WaitAsync(cancellationToken).ConfigureAwait(false);
        return new Releaser(semaphore);
    }

    private sealed class Releaser(SemaphoreSlim semaphore) : IDisposable
    {
        private int _disposed;

        public void Dispose()
        {
            if (Interlocked.Exchange(ref _disposed, 1) == 0)
            {
                semaphore.Release();
            }
        }
    }
}
