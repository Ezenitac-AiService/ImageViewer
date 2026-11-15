using ImageMagick;

namespace ImageViewer.IntegrationTests;

/// <summary>테스트용 이미지·폴더를 만든다.</summary>
internal sealed class TempFolder : IDisposable
{
    public TempFolder()
    {
        Path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "iv-test-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Path);
    }

    public string Path { get; }

    public string File(string name) => System.IO.Path.Combine(Path, name);

    public void Dispose()
    {
        try
        {
            Directory.Delete(Path, recursive: true);
        }
        catch (IOException)
        {
        }
    }
}

internal static class TestImages
{
    public static byte[] Create(uint width, uint height, MagickFormat format, string color = "red", Orientation? orientation = null)
    {
        using var image = new MagickImage(new MagickColor(color), width, height);
        if (orientation is { } o)
        {
            var exif = new ExifProfile();
            exif.SetValue(ExifTag.Orientation, (ushort)o);
            image.SetProfile(exif);
            image.Orientation = o;
        }

        return image.ToByteArray(format);
    }

    public static byte[] Png(uint width = 40, uint height = 30, string color = "red") =>
        Create(width, height, MagickFormat.Png, color);
}
