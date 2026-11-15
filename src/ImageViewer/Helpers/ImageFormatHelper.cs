using ImageMagick;

namespace ImageViewer.Helpers;

/// <summary>
/// 지원 확장자 목록과 파일 헤더 검사를 담당한다.
/// </summary>
public static class ImageFormatHelper
{
    /// <summary>
    /// 목록에 포함할 이미지 확장자.
    /// SVG/PDF/PS 계열은 보안 및 렌더링 일관성 검증 전까지 1차 버전에서 제외한다.
    /// </summary>
    public static IReadOnlySet<string> SupportedExtensions { get; } =
        new HashSet<string>(StringComparer.OrdinalIgnoreCase)
        {
            ".jpg", ".jpeg", ".jpe", ".jfif",
            ".png", ".apng",
            ".bmp", ".dib",
            ".gif",
            ".tif", ".tiff",
            ".webp",
            ".heic", ".heif", ".avif",
            ".ico", ".cur",
            ".jxl", ".jp2", ".j2k",
            ".tga", ".psd", ".dds", ".pcx", ".qoi",
            ".ppm", ".pgm", ".pbm", ".pnm",
            ".hdr", ".exr",
            ".dng", ".cr2", ".cr3", ".nef", ".arw", ".orf", ".rw2", ".raf",
        };

    /// <summary>
    /// 확장자가 이미지와 같더라도 실제 내용이 이 포맷으로 판별되면 디코딩을 거부한다.
    /// (스크립트·외부 참조를 처리할 수 있는 포맷)
    /// </summary>
    public static IReadOnlySet<MagickFormat> BlockedFormats { get; } = new HashSet<MagickFormat>
    {
        MagickFormat.Svg, MagickFormat.Msl, MagickFormat.Mvg,
        MagickFormat.Pdf, MagickFormat.Ps, MagickFormat.Eps, MagickFormat.Ept, MagickFormat.Epi,
        MagickFormat.Ai, MagickFormat.Xps, MagickFormat.Text, MagickFormat.Txt,
        MagickFormat.Html, MagickFormat.Http, MagickFormat.Https,
        MagickFormat.Label, MagickFormat.Caption, MagickFormat.Pango,
    };

    public static bool IsSupportedExtension(string? fileName)
    {
        if (string.IsNullOrEmpty(fileName))
        {
            return false;
        }

        var ext = Path.GetExtension(fileName);
        return !string.IsNullOrEmpty(ext) && SupportedExtensions.Contains(ext);
    }

    public static bool IsZipFile(string? path) =>
        !string.IsNullOrEmpty(path) &&
        (path.EndsWith(".zip", StringComparison.OrdinalIgnoreCase) ||
         path.EndsWith(".cbz", StringComparison.OrdinalIgnoreCase));

    /// <summary>
    /// 텍스트(XML/SVG/MSL), PostScript, PDF 로 보이는 헤더를 감지한다.
    /// 확장자가 이미지라도 이런 내용이면 ImageMagick 에 넘기지 않는다.
    /// </summary>
    public static bool LooksLikeScriptOrDocument(ReadOnlySpan<byte> header)
    {
        var span = header;

        // UTF-8 BOM
        if (span.Length >= 3 && span[0] == 0xEF && span[1] == 0xBB && span[2] == 0xBF)
        {
            span = span[3..];
        }

        var i = 0;
        while (i < span.Length && (span[i] == (byte)' ' || span[i] == (byte)'\t' ||
                                   span[i] == (byte)'\r' || span[i] == (byte)'\n'))
        {
            i++;
        }

        span = span[i..];
        if (span.IsEmpty)
        {
            return false;
        }

        return span[0] == (byte)'<' ||
               span.StartsWith("%!"u8) ||
               span.StartsWith("%PDF"u8) ||
               span.StartsWith("push "u8) ||
               span.StartsWith("viewbox"u8);
    }
}
