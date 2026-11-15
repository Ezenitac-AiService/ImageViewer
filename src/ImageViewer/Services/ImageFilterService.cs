using ImageMagick;
using ImageViewer.Models;

namespace ImageViewer.Services;

/// <summary>
/// 화면 표시용 이미지에만 필터를 적용한다. 원본 픽셀 데이터는 변경하지 않는다.
/// </summary>
public sealed class ImageFilterService(SettingsService settings)
{
    public void Apply(MagickImage image, ImageFilter filter)
    {
        var options = settings.Current.Filter;
        switch (filter)
        {
            case ImageFilter.Smooth:
                image.GaussianBlur(0, options.SmoothSigma);
                break;

            case ImageFilter.SmoothSharpen:
                image.GaussianBlur(0, options.SmoothSigma);
                image.UnsharpMask(0, options.SharpenSigma, options.SharpenAmount, 0.01);
                break;
        }
    }

    public static string GetDisplayName(ImageFilter filter) => filter switch
    {
        ImageFilter.Smooth => "부드럽게",
        ImageFilter.SmoothSharpen => "부드럽게 + 선명하게",
        _ => "필터 없음",
    };
}
