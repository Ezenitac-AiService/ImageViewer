using System.Globalization;

namespace ImageViewer.Helpers;

public static class FileSizeHelper
{
    private static readonly string[] Units = ["B", "KB", "MB", "GB", "TB"];

    /// <summary>바이트 수를 사람이 읽기 쉬운 문자열로 변환한다. (예: 2.1 MB, 850 KB)</summary>
    public static string Format(long bytes)
    {
        if (bytes < 0)
        {
            return "-";
        }

        if (bytes < 1024)
        {
            return string.Create(CultureInfo.InvariantCulture, $"{bytes} B");
        }

        double value = bytes;
        var unit = 0;
        while (value >= 1024 && unit < Units.Length - 1)
        {
            value /= 1024;
            unit++;
        }

        // KB 단위는 정수, 그 이상은 소수점 한 자리
        var format = unit == 1 ? "0" : "0.0";
        return value.ToString(format, CultureInfo.InvariantCulture) + " " + Units[unit];
    }
}
