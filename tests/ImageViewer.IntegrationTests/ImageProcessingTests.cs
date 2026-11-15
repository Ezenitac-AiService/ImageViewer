using System.Windows.Media.Imaging;
using ImageMagick;
using ImageViewer.Models;
using ImageViewer.Services;
using ImageViewer.Services.ImageSources;

namespace ImageViewer.IntegrationTests;

public class ImageLoaderServiceTests
{
    private static SettingsService Settings(TempFolder temp)
    {
        var s = new SettingsService(settingsPath: temp.File("settings.json"));
        s.Load();
        return s;
    }

    [Theory]
    [InlineData(MagickFormat.Jpeg, "x.jpg")]
    [InlineData(MagickFormat.Png, "x.png")]
    [InlineData(MagickFormat.Bmp, "x.bmp")]
    [InlineData(MagickFormat.Gif, "x.gif")]
    [InlineData(MagickFormat.Tiff, "x.tif")]
    [InlineData(MagickFormat.WebP, "x.webp")]
    [InlineData(MagickFormat.Ico, "x.ico")]
    public async Task Decodes_common_formats(MagickFormat format, string name)
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File(name), TestImages.Create(32, 16, format));
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();

        using var loader = new ImageLoaderService(Settings(temp));
        var decoded = await loader.LoadAsync(source, info, CancellationToken.None);
        try
        {
            Assert.Equal(32u, decoded.Width);
            Assert.Equal(16u, decoded.Height);
        }
        finally
        {
            decoded.Release();
        }
    }

    [Fact]
    public async Task Applies_exif_orientation()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("rotated.jpg"), TestImages.Create(40, 20, MagickFormat.Jpeg, orientation: Orientation.RightTop));
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();

        using var loader = new ImageLoaderService(Settings(temp));
        var decoded = await loader.LoadAsync(source, info, CancellationToken.None);
        Assert.Equal(20u, decoded.Width);
        Assert.Equal(40u, decoded.Height);
        decoded.Release();
    }

    [Fact]
    public async Task Corrupt_image_throws_image_load_exception()
    {
        using var temp = new TempFolder();
        var bytes = TestImages.Create(64, 64, MagickFormat.Jpeg);
        File.WriteAllBytes(temp.File("broken.jpg"), bytes[..20]);
        File.WriteAllBytes(temp.File("random.png"), Enumerable.Range(0, 500).Select(i => (byte)(i * 7)).ToArray());
        File.WriteAllBytes(temp.File("empty.gif"), []);
        using var source = new FolderImageSource(temp.Path);
        var images = await source.GetImagesAsync(CancellationToken.None);

        using var loader = new ImageLoaderService(Settings(temp));
        foreach (var info in images)
        {
            await Assert.ThrowsAsync<ImageLoadException>(() => loader.LoadAsync(source, info, CancellationToken.None));
        }
    }

    [Fact]
    public async Task Svg_disguised_as_png_is_rejected()
    {
        using var temp = new TempFolder();
        File.WriteAllText(temp.File("fake.png"), "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"/>");
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(Settings(temp));
        await Assert.ThrowsAsync<ImageLoadException>(() => loader.LoadAsync(source, info, CancellationToken.None));
    }

    [Fact]
    public async Task Mismatched_extension_is_decoded_by_content()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("actually-png.jpg"), TestImages.Png(9, 7));
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(Settings(temp));
        var decoded = await loader.LoadAsync(source, info, CancellationToken.None);
        Assert.Equal("PNG", decoded.Format);
        decoded.Release();
    }

    [Fact]
    public void Pixel_limit_is_enforced_before_decoding()
    {
        using var stream = new MemoryStream(TestImages.Png(100, 100));
        var ex = Assert.Throws<ImageLoadException>(() => ImageLoaderService.Decode(stream, maxPixels: 5_000, null));
        Assert.Contains("너무 큽니다", ex.Message);
    }

    [Fact]
    public async Task Cached_image_survives_until_released()
    {
        using var temp = new TempFolder();
        for (var i = 0; i < 6; i++)
        {
            File.WriteAllBytes(temp.File($"{i}.png"), TestImages.Png());
        }

        using var source = new FolderImageSource(temp.Path);
        var images = await source.GetImagesAsync(CancellationToken.None);
        using var loader = new ImageLoaderService(Settings(temp));

        var first = await loader.LoadAsync(source, images[0], CancellationToken.None);
        foreach (var info in images.Skip(1))
        {
            (await loader.LoadAsync(source, info, CancellationToken.None)).Release();
        }

        // 캐시에서 밀려났어도 호출자가 참조를 가진 동안에는 사용할 수 있다.
        using (var clone = first.CloneImage())
        {
            Assert.NotNull(clone);
        }

        first.Release();
        Assert.True(first.IsDisposed);
        Assert.Null(first.CloneImage());
    }
}

public class ImageRenderServiceTests
{
    [Theory]
    [InlineData(ImageFilter.None)]
    [InlineData(ImageFilter.Smooth)]
    [InlineData(ImageFilter.SmoothSharpen)]
    public async Task Renders_display_size_bitmap_with_filters(ImageFilter filter)
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("a.png"), TestImages.Create(400, 200, MagickFormat.Png, "#3366cc"));
        var settings = new SettingsService(settingsPath: temp.File("s.json"));
        settings.Load();
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(settings);
        var decoded = await loader.LoadAsync(source, info, CancellationToken.None);

        var renderer = new ImageRenderService(new ImageFilterService(settings));
        var bitmap = await renderer.RenderAsync(decoded, 100, 50, filter, ImageTransform.Identity, CancellationToken.None);

        Assert.Equal(100, bitmap.PixelWidth);
        Assert.Equal(50, bitmap.PixelHeight);
        Assert.True(bitmap.IsFrozen);

        // 원본은 변경되지 않는다.
        Assert.Equal(400u, decoded.Width);

        // 같은 요청은 캐시에서 반환된다.
        var again = await renderer.RenderAsync(decoded, 100, 50, filter, ImageTransform.Identity, CancellationToken.None);
        Assert.Same(bitmap, again);
        decoded.Release();
    }

    [Fact]
    public async Task Preserves_transparency()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("t.png"), TestImages.Create(10, 10, MagickFormat.Png, "transparent"));
        var settings = new SettingsService(settingsPath: temp.File("s.json"));
        settings.Load();
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();
        using var loader = new ImageLoaderService(settings);
        var decoded = await loader.LoadAsync(source, info, CancellationToken.None);
        var renderer = new ImageRenderService(new ImageFilterService(settings));
        var bitmap = await renderer.RenderAsync(decoded, 10, 10, ImageFilter.None, ImageTransform.Identity, CancellationToken.None);

        var pixels = new byte[10 * 10 * 4];
        bitmap.CopyPixels(pixels, 40, 0);
        Assert.Equal(0, pixels[3]); // alpha
        decoded.Release();
    }
}

public class ThumbnailServiceTests
{
    [Fact]
    public async Task Creates_small_thumbnail()
    {
        using var temp = new TempFolder();
        File.WriteAllBytes(temp.File("a.jpg"), TestImages.Create(800, 400, MagickFormat.Jpeg));
        var settings = new SettingsService(settingsPath: temp.File("s.json"));
        settings.Load();
        using var source = new FolderImageSource(temp.Path);
        var info = (await source.GetImagesAsync(CancellationToken.None)).Single();

        var service = new ThumbnailService(settings);
        var tcs = new TaskCompletionSource<(BitmapSource?, bool)>();
        service.Request(source, info, 96, (b, done) => tcs.TrySetResult((b, done)));
        var (bitmap, completed) = await tcs.Task.WaitAsync(TimeSpan.FromSeconds(30));

        Assert.True(completed);
        Assert.NotNull(bitmap);
        Assert.Equal(96, bitmap!.PixelWidth);
        Assert.Equal(48, bitmap.PixelHeight);
    }
}
