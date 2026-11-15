namespace ImageViewer.Services;

/// <summary>
/// 같은 키의 작업이 진행 중이면 새로 시작하지 않고 그 작업을 함께 기다리게 한다.
/// 예: 미리 읽기가 디코딩 중인 이미지로 사용자가 이동하면, 처음부터 다시 디코딩하지 않고 그 결과를 이어받는다.
/// 작업이 끝나면 목록에서 빠진다.
/// </summary>
internal sealed class SharedWork<TKey, TResult> where TKey : notnull
{
    private readonly Dictionary<TKey, Task<TResult>> _running = new();
    private readonly Lock _lock = new();

    public Task<TResult> GetOrStart(TKey key, Func<Task<TResult>> start)
    {
        lock (_lock)
        {
            if (_running.TryGetValue(key, out var existing))
            {
                return existing;
            }

            var task = start();
            if (!task.IsCompleted)
            {
                _running[key] = task;
                _ = task.ContinueWith(
                    completed =>
                    {
                        lock (_lock)
                        {
                            if (_running.TryGetValue(key, out var current) && ReferenceEquals(current, completed))
                            {
                                _running.Remove(key);
                            }
                        }
                    },
                    CancellationToken.None,
                    TaskContinuationOptions.ExecuteSynchronously,
                    TaskScheduler.Default);
            }

            return task;
        }
    }

    /// <summary>
    /// 공유 작업을 기다린다. 함께 기다리던 작업이 시작 전에 (다른 요청자의 취소로) 취소되었으면
    /// 이 요청자의 취소 토큰으로 다시 시작한다.
    /// </summary>
    public async Task<TResult> RunAsync(TKey key, Func<Task<TResult>> start, CancellationToken cancellationToken)
    {
        for (var attempt = 0; ; attempt++)
        {
            var task = GetOrStart(key, start);
            try
            {
                return await task.WaitAsync(cancellationToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException) when (!cancellationToken.IsCancellationRequested && attempt < 3)
            {
                // 다른 요청자가 시작한 작업이 시작 전에 취소됨 → 다시 시도
            }
        }
    }
}
