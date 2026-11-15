namespace ImageViewer.Services;

/// <summary>
/// 현재 소스의 이미지 목록 안에서 이전·다음 위치를 계산한다.
/// </summary>
public sealed class ImageNavigationService
{
    public int Count { get; private set; }

    public int CurrentIndex { get; private set; } = -1;

    public bool WrapAround { get; set; } = true;

    public void Reset(int count, int initialIndex = 0)
    {
        Count = Math.Max(0, count);
        CurrentIndex = Count == 0 ? -1 : Math.Clamp(initialIndex, 0, Count - 1);
    }

    public bool MoveTo(int index)
    {
        if (index < 0 || index >= Count)
        {
            return false;
        }

        CurrentIndex = index;
        return true;
    }

    /// <summary>다음 이미지 위치. 이동할 수 없으면 -1.</summary>
    public int GetNextIndex() => GetOffsetIndex(+1);

    /// <summary>이전 이미지 위치. 이동할 수 없으면 -1.</summary>
    public int GetPreviousIndex() => GetOffsetIndex(-1);

    public int GetFirstIndex() => Count > 0 ? 0 : -1;

    public int GetLastIndex() => Count > 0 ? Count - 1 : -1;

    private int GetOffsetIndex(int offset)
    {
        if (Count == 0 || CurrentIndex < 0)
        {
            return -1;
        }

        var target = CurrentIndex + offset;
        if (target >= 0 && target < Count)
        {
            return target;
        }

        if (!WrapAround || Count == 1)
        {
            return -1;
        }

        return ((target % Count) + Count) % Count;
    }
}
