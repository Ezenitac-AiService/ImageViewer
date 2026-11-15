namespace ImageViewer.Models;

public sealed class SlideShowSettings
{
    public const int MinIntervalSeconds = 5;
    public const int MaxIntervalSeconds = 90;
    public const int DefaultIntervalSeconds = 5;

    public int IntervalSeconds { get; set; } = DefaultIntervalSeconds;

    public SlideShowMode Mode { get; set; } = SlideShowMode.Sequential;

    /// <summary>마지막 이미지 이후 처음으로 돌아가 계속 재생할지 여부.</summary>
    public bool Repeat { get; set; } = true;

    public static int ClampInterval(int seconds) =>
        Math.Clamp(seconds, MinIntervalSeconds, MaxIntervalSeconds);
}
