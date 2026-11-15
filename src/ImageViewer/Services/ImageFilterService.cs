using ImageMagick;
using ImageViewer.Models;

namespace ImageViewer.Services;

/// <summary>
/// 화면 표시용 이미지에만 필터를 적용한다. 원본 픽셀 데이터는 변경하지 않는다.
/// </summary>
public sealed class ImageFilterService(SettingsService settings)
{
    /// <summary>이보다 작은 sigma 는 눈에 띄는 차이가 없어 건너뛴다.</summary>
    private const double MinEffectiveSigma = 0.05;

    /// <summary>
    /// 필터를 적용한다. 설정의 sigma 는 <b>화면 픽셀</b> 기준이다.
    /// <paramref name="displayScale"/> 은 이 이미지의 한 픽셀이 화면에서 몇 픽셀로 보이는지
    /// (작은 이미지를 키워 보이면 1보다 크고, WPF 가 줄여 보이면 1보다 작다).
    /// 이미지 픽셀 기준으로 sigma 를 그대로 쓰면, 작은 이미지를 확대해 볼 때 흐림도 같이 확대되어 과하게 뭉개진다.
    /// </summary>
    public void Apply(MagickImage image, ImageFilter filter, double displayScale = 1.0)
    {
        if (filter == ImageFilter.None)
        {
            return;
        }

        var options = settings.Current.Filter;
        var scale = displayScale > 0 && double.IsFinite(displayScale) ? displayScale : 1.0;
        var smoothSigma = options.SmoothSigma / scale;
        var sharpenSigma = options.SharpenSigma / scale;

        switch (filter)
        {
            case ImageFilter.Smooth:
                Blur(image, smoothSigma);
                break;

            case ImageFilter.SmoothSharpen:
                Blur(image, smoothSigma);
                if (sharpenSigma >= MinEffectiveSigma)
                {
                    image.UnsharpMask(0, sharpenSigma, options.SharpenAmount, 0.01);
                }

                break;
        }
    }

    private static void Blur(MagickImage image, double sigma)
    {
        if (sigma >= MinEffectiveSigma)
        {
            image.GaussianBlur(0, sigma);
        }
    }

    public static string GetDisplayName(ImageFilter filter) => filter switch
    {
        ImageFilter.Smooth => "부드럽게",
        ImageFilter.SmoothSharpen => "부드럽게 + 선명하게",
        _ => "필터 없음",
    };
}
