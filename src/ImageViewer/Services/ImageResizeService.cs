using ImageViewer.Models;

namespace ImageViewer.Services;

/// <summary>배율 계산 결과. 모든 값은 장치 픽셀 기준.</summary>
public readonly record struct DisplayLayout(
    double Scale,
    double DisplayPixelWidth,
    double DisplayPixelHeight,
    uint RenderPixelWidth,
    uint RenderPixelHeight);

/// <summary>
/// 표시 모드·확대 배율·표시 영역으로부터 화면 표시 크기와 렌더링 크기를 계산한다.
/// 순수 계산만 수행하므로 단위 테스트가 가능하다.
/// </summary>
public sealed class ImageResizeService
{
    public const double MinScale = 0.01;
    public const double MaxScale = 32.0;
    public const double ZoomStep = 1.25;

    /// <summary>
    /// 표시 모드에 따른 기본 배율 (이미지 1px → 장치 픽셀 수).
    /// 맞춤 모드는 이미지 크기와 관계없이 표시 영역에 맞춘다 (작은 이미지도 확대).
    /// </summary>
    public static double GetBaseScale(
        DisplayMode mode,
        double imageWidth,
        double imageHeight,
        double viewportWidth,
        double viewportHeight)
    {
        if (imageWidth <= 0 || imageHeight <= 0 || viewportWidth <= 0 || viewportHeight <= 0)
        {
            return 1.0;
        }

        var sx = viewportWidth / imageWidth;
        var sy = viewportHeight / imageHeight;

        var scale = mode switch
        {
            DisplayMode.Original => 1.0,
            DisplayMode.FitWidth => sx,
            DisplayMode.FitHeight => sy,
            // 꽉 차게: 비율을 유지한 채 이미지 전체가 창 안에 들어오는 가장 큰 크기 (잘리는 부분 없음).
            DisplayMode.Fill => Math.Min(sx, sy),
            _ => 1.0,
        };

        return Math.Clamp(scale, MinScale, MaxScale);
    }

    public static DisplayLayout Calculate(
        DisplayMode mode,
        double zoomFactor,
        uint imageWidth,
        uint imageHeight,
        double viewportWidth,
        double viewportHeight)
    {
        var baseScale = GetBaseScale(mode, imageWidth, imageHeight, viewportWidth, viewportHeight);
        var scale = Math.Clamp(baseScale * zoomFactor, MinScale, MaxScale);

        var displayW = imageWidth * scale;
        var displayH = imageHeight * scale;

        // 축소 표시 시에는 표시 크기로 렌더링해 품질을 확보하고 필터를 적용한다.
        // 확대 표시 시에는 원본 크기로 렌더링하고 화면에서 확대한다.
        uint renderW, renderH;
        if (scale < 1.0)
        {
            renderW = (uint)Math.Max(1, Math.Round(displayW));
            renderH = (uint)Math.Max(1, Math.Round(displayH));
        }
        else
        {
            renderW = imageWidth;
            renderH = imageHeight;
        }

        return new DisplayLayout(scale, displayW, displayH, renderW, renderH);
    }

    /// <summary>현재 배율에서 한 단계 확대/축소한 확대 배율(zoomFactor)을 구한다.</summary>
    public static double StepZoom(double baseScale, double zoomFactor, bool zoomIn)
    {
        var current = baseScale * zoomFactor;
        var next = zoomIn ? current * ZoomStep : current / ZoomStep;

        // 100% 를 지나칠 때는 100% 에 멈춘다.
        if ((current < 1.0 && next > 1.0) || (current > 1.0 && next < 1.0))
        {
            next = 1.0;
        }

        next = Math.Clamp(next, MinScale, MaxScale);
        return baseScale <= 0 ? 1.0 : next / baseScale;
    }
}
