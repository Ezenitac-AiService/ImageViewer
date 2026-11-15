using ImageViewer.Services;

namespace ImageViewer.Tests;

public class PriorityGateTests
{
    [Fact]
    public async Task Waits_when_full_of_needed_work()
    {
        var gate = new PriorityGate(capacity: 2, burstCapacity: 4);
        using var a = await gate.EnterAsync(CancellationToken.None);
        using var b = await gate.EnterAsync(CancellationToken.None);

        var third = gate.EnterAsync(CancellationToken.None);
        await Task.Delay(100);
        Assert.False(third.IsCompleted);

        a.Dispose();
        using var c = await third.WaitAsync(TimeSpan.FromSeconds(2));
        Assert.Equal(2, gate.RunningCount);
    }

    [Fact]
    public async Task New_request_bypasses_stale_work()
    {
        var gate = new PriorityGate(capacity: 2, burstCapacity: 4);
        using var stale1 = new CancellationTokenSource();
        using var stale2 = new CancellationTokenSource();
        using var a = await gate.EnterAsync(stale1.Token);
        using var b = await gate.EnterAsync(stale2.Token);

        // 사용자가 다른 이미지로 이동: 실행 중인 두 작업은 더 이상 필요 없다.
        stale1.Cancel();
        stale2.Cancel();

        using var fresh = await gate.EnterAsync(CancellationToken.None).WaitAsync(TimeSpan.FromSeconds(2));
        Assert.Equal(3, gate.RunningCount);
    }

    [Fact]
    public async Task Burst_is_bounded()
    {
        var gate = new PriorityGate(capacity: 2, burstCapacity: 3);
        var stale = new CancellationTokenSource();
        var slots = new List<IDisposable>
        {
            await gate.EnterAsync(stale.Token),
            await gate.EnterAsync(stale.Token),
        };
        stale.Cancel();
        slots.Add(await gate.EnterAsync(CancellationToken.None)); // 3번째: 넘어서기 허용

        var fourth = gate.EnterAsync(CancellationToken.None);      // 상한(3) 초과 → 대기
        await Task.Delay(100);
        Assert.False(fourth.IsCompleted);

        slots[0].Dispose();
        slots.Add(await fourth.WaitAsync(TimeSpan.FromSeconds(2)));
        foreach (var s in slots)
        {
            s.Dispose();
        }

        Assert.Equal(0, gate.RunningCount);
    }

    [Fact]
    public async Task Cancelled_waiter_gives_up()
    {
        var gate = new PriorityGate(capacity: 1, burstCapacity: 1);
        using var a = await gate.EnterAsync(CancellationToken.None);
        using var cts = new CancellationTokenSource();
        var waiting = gate.EnterAsync(cts.Token);
        cts.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => waiting);
        Assert.Equal(1, gate.RunningCount);
    }
}
