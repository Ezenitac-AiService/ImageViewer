using ImageViewer.Models;
using ImageViewer.Services;
using ImageViewer.ViewModels;

namespace ImageViewer.Tests;

public class PlaceholderTests
{
    [Theory]
    [InlineData(DisplayMode.FitWidth)]
    [InlineData(DisplayMode.FitHeight)]
    [InlineData(DisplayMode.Fill)]
    public void Placeholder_lands_at_final_image_size_in_fit_modes(DisplayMode mode)
    {
        // 원본 6000×4000, 썸네일 96×64 (같은 비율)
        var final = ImageResizeService.Calculate(mode, 1.0, 6000, 4000, 2560, 1440);
        var (w, h) = MainViewModel.PlaceholderDisplaySize(mode, 96, 64, 2560, 1440);

        Assert.Equal(final.DisplayPixelWidth, w, 1);
        Assert.Equal(final.DisplayPixelHeight, h, 1);
    }

    [Fact]
    public void Placeholder_waits_long_enough_for_prefetched_images()
    {
        // 미리 읽어 둔 이미지는 수십 ms 안에 표시되므로 그보다 충분히 길어야 한다.
        Assert.Equal(TimeSpan.FromMilliseconds(250), MainViewModel.PlaceholderDelay);
    }
}
