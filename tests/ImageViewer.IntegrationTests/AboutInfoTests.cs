using ImageViewer.Services;

namespace ImageViewer.IntegrationTests;

public class AboutInfoTests
{
    [Fact]
    public void Versions_are_read_from_the_loaded_libraries()
    {
        var info = AboutInfo.Create();

        Assert.Equal("ImageViewer", info.ProductName);
        Assert.Matches(@"^\d+\.\d+\.\d+", info.Version);
        Assert.DoesNotContain('+', info.Version);

        var byName = info.Libraries.ToDictionary(l => l.Name.Split(' ')[0]);
        Assert.StartsWith("15.", byName["Magick.NET"].Version);
        Assert.Matches(@"^7\.\d+\.\d+", byName["ImageMagick"].Version);
        Assert.Matches(@"^3\.\d+\.\d+ \(Skia \d+\.\d+\)$", byName["SkiaSharp"].Version);
        Assert.Equal(["Magick.NET", "ImageMagick", "SkiaSharp", "SharpCompress"], byName.Keys);
        Assert.All(info.Libraries, l =>
        {
            Assert.NotEqual("?", l.Version);
            Assert.StartsWith("https://", l.Url);
        });

        var text = info.ToText();
        Assert.Contains(info.Version, text);
        Assert.Contains(".NET", text);
    }
}
