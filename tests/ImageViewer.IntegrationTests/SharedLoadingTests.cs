using ImageMagick;
using ImageViewer.Models;
using ImageViewer.Services;
using ImageViewer.Services.ImageSources;

namespace ImageViewer.IntegrationTests;

public class SharedLoadingTests
{
    private static readonly DecodeTarget Target = new(DisplayMode.FitWidth, 1264, 800);

    [Fact]
    public async Task Concurrent_requests_share_one_decode()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("big.jpg"), TestImages.Create(4000, 3000, MagickFormat.Jpeg));
        var settings = new SettingsService(settingsPath: temp.File("s.json"));
        settings.Load();
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(settings);

        var background = loader.LoadAsync(source, info, 0, Target, CancellationToken.None, background: true);
        var foreground = loader.LoadAsync(source, info, 0, Target, CancellationToken.None);
        var results = await Task.WhenAll(background, foreground);

        Assert.Same(results[0], results[1]);
        results[0].Release();
        results[1].Release();
    }

    [Fact]
    public async Task Foreground_gets_result_when_prefetch_requester_cancels()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("big.jpg"), TestImages.Create(4000, 3000, MagickFormat.Jpeg));
        var settings = new SettingsService(settingsPath: temp.File("s.json"));
        settings.Load();
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(settings);

        using var prefetchCts = new CancellationTokenSource();
        var prefetch = loader.LoadAsync(source, info, 0, Target, prefetchCts.Token, background: true);
        var foreground = loader.LoadAsync(source, info, 0, Target, CancellationToken.None);
        prefetchCts.Cancel(); // 사용자가 다른 이미지로 이동해 미리 읽기가 취소됨

        var decoded = await foreground;
        Assert.Equal(4000u, decoded.OriginalWidth);
        decoded.Release();

        try
        {
            (await prefetch).Release();
        }
        catch (OperationCanceledException)
        {
            // 미리 읽기 쪽은 취소되어도 된다.
        }
    }

    [Fact]
    public async Task Concurrent_renders_share_one_result()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("big.jpg"), TestImages.Create(4000, 3000, MagickFormat.Jpeg));
        var settings = new SettingsService(settingsPath: temp.File("s.json"));
        settings.Load();
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(settings);
        var decoded = await loader.LoadAsync(source, info, 0, Target, CancellationToken.None);

        var renderer = new ImageRenderService(new ImageFilterService(settings));
        var a = renderer.RenderAsync(decoded, 1264, 948, ImageFilter.None, ImageTransform.Identity, CancellationToken.None, background: true);
        var b = renderer.RenderAsync(decoded, 1264, 948, ImageFilter.None, ImageTransform.Identity, CancellationToken.None);
        var bitmaps = await Task.WhenAll(a, b);
        Assert.Same(bitmaps[0], bitmaps[1]);
        decoded.Release();
    }
}
