using System.Runtime.InteropServices;

namespace ImageViewer.Helpers;

internal static class NativeMethods
{
    private const uint MONITOR_DEFAULTTONEAREST = 2;

    [StructLayout(LayoutKind.Sequential)]
    private struct RECT
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MONITORINFO
    {
        public int cbSize;
        public RECT rcMonitor;
        public RECT rcWork;
        public uint dwFlags;
    }

    [DllImport("user32.dll")]
    private static extern IntPtr MonitorFromWindow(IntPtr hwnd, uint dwFlags);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetMonitorInfo(IntPtr hMonitor, ref MONITORINFO lpmi);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);

    /// <summary>
    /// 창이 있는 모니터의 작업 영역(작업 표시줄 제외) 밖으로 창이 넘친 크기(물리 픽셀).
    /// </summary>
    public static (int Left, int Top, int Right, int Bottom)? GetOverflowBeyondWorkArea(IntPtr hwnd)
    {
        if (hwnd == IntPtr.Zero || !GetWindowRect(hwnd, out var window))
        {
            return null;
        }

        var monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        var info = new MONITORINFO { cbSize = Marshal.SizeOf<MONITORINFO>() };
        if (monitor == IntPtr.Zero || !GetMonitorInfo(monitor, ref info))
        {
            return null;
        }

        var work = info.rcWork;
        return (
            Math.Max(0, work.Left - window.Left),
            Math.Max(0, work.Top - window.Top),
            Math.Max(0, window.Right - work.Right),
            Math.Max(0, window.Bottom - work.Bottom));
    }
}
