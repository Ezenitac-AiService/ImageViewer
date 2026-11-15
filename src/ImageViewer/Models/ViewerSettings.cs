namespace ImageViewer.Models;

/// <summary>
/// %LOCALAPPDATA%\ImageViewer\settings.json 에 저장되는 사용자 설정.
/// </summary>
public sealed class ViewerSettings
{
    /// <summary>설정 파일 형식 버전. 기본값이 바뀐 항목을 이전 설정에서 옮길 때 사용한다.</summary>
    public const int CurrentVersion = 2;

    /// <summary>파일에 없으면 0 (버전 관리 이전에 저장된 설정).</summary>
    public int Version { get; set; }

    public WindowSettings Window { get; set; } = new();

    public DisplaySettings Display { get; set; } = new();

    public NavigationSettings Navigation { get; set; } = new();

    public SlideShowSettings Slideshow { get; set; } = new();

    public ThumbnailSettings Thumbnail { get; set; } = new();

    public FilterSettings Filter { get; set; } = new();

    public LimitSettings Limits { get; set; } = new();

    /// <summary>읽어 들인 값의 범위를 보정한다.</summary>
    public ViewerSettings Normalize()
    {
        Window ??= new();
        Display ??= new();
        Navigation ??= new();
        Slideshow ??= new();
        Thumbnail ??= new();
        Filter ??= new();
        Limits ??= new();

        if (Version < 2)
        {
            // v2: 마우스 휠 기본 동작을 확대/축소 → 이전/다음 이미지로 변경
            Display.MouseWheel = MouseWheelAction.Navigate;
        }

        Version = CurrentVersion;

        Window.Width = Math.Clamp(Window.Width, 320, 20000);
        Window.Height = Math.Clamp(Window.Height, 240, 20000);
        Slideshow.IntervalSeconds = SlideShowSettings.ClampInterval(Slideshow.IntervalSeconds);
        Thumbnail.Size = Math.Clamp(Thumbnail.Size, 48, 256);
        Filter.SmoothSigma = Math.Clamp(Filter.SmoothSigma, 0.1, 5.0);
        Filter.SharpenSigma = Math.Clamp(Filter.SharpenSigma, 0.1, 5.0);
        Filter.SharpenAmount = Math.Clamp(Filter.SharpenAmount, 0.1, 5.0);
        Limits.MaxImageMegapixels = Math.Clamp(Limits.MaxImageMegapixels, 10, 1000);
        Limits.MaxZipEntryMegabytes = Math.Clamp(Limits.MaxZipEntryMegabytes, 10, 4096);
        return this;
    }
}

public sealed class WindowSettings
{
    public double Width { get; set; } = 1280;
    public double Height { get; set; } = 800;
    public double? Left { get; set; }
    public double? Top { get; set; }
    public bool IsMaximized { get; set; }
}

public sealed class DisplaySettings
{
    public DisplayMode Mode { get; set; } = DisplayMode.FitWidth;

    /// <summary>상단 바(타이틀바 + 메뉴바) 고정(항상 표시) 여부. 해제하면 마우스를 올렸을 때만 표시.</summary>
    public bool TopBarPinned { get; set; } = true;

    /// <summary>하단 바 고정(항상 표시) 여부. 해제하면 마우스를 올렸을 때만 표시.</summary>
    public bool BottomBarPinned { get; set; } = true;

    /// <summary>Tab 으로 여닫는 이미지 정보(EXIF) 패널 표시 여부.</summary>
    public bool InfoPanelVisible { get; set; }

    /// <summary>GIF / WebP 애니메이션 재생 여부.</summary>
    public bool PlayAnimations { get; set; } = true;

    public ViewerBackground Background { get; set; } = ViewerBackground.Black;

    public ImageFilter Filter { get; set; } = ImageFilter.None;

    /// <summary>휠: 기본은 위 = 이전 이미지, 아래 = 다음 이미지 (Ctrl+휠은 확대/축소).</summary>
    public MouseWheelAction MouseWheel { get; set; } = MouseWheelAction.Navigate;
}

public sealed class NavigationSettings
{
    /// <summary>이전·다음 탐색 시 처음과 끝을 연결할지 여부.</summary>
    public bool WrapAround { get; set; } = true;

    /// <summary>폴더의 이미지 추가 · 삭제 · 변경을 감시해 목록을 자동으로 갱신한다.</summary>
    public bool WatchFolder { get; set; } = true;
}

public sealed class ThumbnailSettings
{
    public bool Visible { get; set; } = true;
    public int Size { get; set; } = 96;
}

/// <summary>필터 강도. 추후 설정 화면에서 조절할 수 있도록 분리한다.</summary>
public sealed class FilterSettings
{
    public double SmoothSigma { get; set; } = 0.8;
    public double SharpenSigma { get; set; } = 1.0;
    public double SharpenAmount { get; set; } = 1.0;
}

/// <summary>과도한 메모리 사용을 막기 위한 상한.</summary>
public sealed class LimitSettings
{
    public int MaxImageMegapixels { get; set; } = 150;
    public int MaxZipEntryMegabytes { get; set; } = 512;
}
