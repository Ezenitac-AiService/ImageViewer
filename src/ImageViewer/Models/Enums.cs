namespace ImageViewer.Models;

/// <summary>이미지 크기 조절 모드 (단축키 0 / 7 / 8 / 9).</summary>
public enum DisplayMode
{
    Original,
    FitWidth,
    FitHeight,
    /// <summary>비율을 유지한 채 이미지 전체를 창에 맞춰 최대한 크게 표시 (잘리지 않음).</summary>
    Fill,
}

/// <summary>화면 표시용 이미지 필터 (단축키 U / I / S).</summary>
public enum ImageFilter
{
    None,
    Smooth,
    SmoothSharpen,
}

public enum SlideShowMode
{
    Sequential,
    Random,
}

public enum MouseWheelAction
{
    Zoom,
    Navigate,
}

public enum ImageSourceType
{
    File,
    Zip,
}

public enum ViewerBackground
{
    Black,
    DarkGray,
    Gray,
    White,
    Checkerboard,
}
