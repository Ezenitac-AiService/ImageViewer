namespace ImageViewer.Helpers;

/// <summary>
/// 파일명 정렬용 비교자. 숫자 구간을 숫자로 비교한다 (image2 &lt; image10).
/// Windows 탐색기의 정렬 방식과 유사하며, 대소문자를 구분하지 않는다.
/// </summary>
public sealed class NaturalStringComparer : IComparer<string?>
{
    public static NaturalStringComparer Instance { get; } = new();

    public int Compare(string? x, string? y)
    {
        if (ReferenceEquals(x, y))
        {
            return 0;
        }

        if (x is null)
        {
            return -1;
        }

        if (y is null)
        {
            return 1;
        }

        int ix = 0, iy = 0;
        while (ix < x.Length && iy < y.Length)
        {
            var cx = x[ix];
            var cy = y[iy];

            if (char.IsAsciiDigit(cx) && char.IsAsciiDigit(cy))
            {
                var sx = ix;
                var sy = iy;
                while (ix < x.Length && char.IsAsciiDigit(x[ix])) ix++;
                while (iy < y.Length && char.IsAsciiDigit(y[iy])) iy++;

                var nx = x.AsSpan(sx, ix - sx).TrimStart('0');
                var ny = y.AsSpan(sy, iy - sy).TrimStart('0');

                if (nx.Length != ny.Length)
                {
                    return nx.Length.CompareTo(ny.Length);
                }

                var cmp = nx.SequenceCompareTo(ny);
                if (cmp != 0)
                {
                    return cmp;
                }

                // 숫자 값이 같으면 앞자리 0이 적은 쪽을 먼저 (1 < 01)
                var lenCmp = (ix - sx).CompareTo(iy - sy);
                if (lenCmp != 0)
                {
                    return lenCmp;
                }

                continue;
            }

            var c = string.Compare(
                x, ix, y, iy, 1, StringComparison.CurrentCultureIgnoreCase);
            if (c != 0)
            {
                return c;
            }

            ix++;
            iy++;
        }

        var remain = (x.Length - ix).CompareTo(y.Length - iy);
        return remain != 0 ? remain : string.CompareOrdinal(x, y);
    }
}
