using System.Reflection;
using System.Runtime.InteropServices;
using ImageMagick;
using SharpCompress.Archives;
using SkiaSharp;

namespace ImageViewer.Services;

/// <summary>정보 창에 표시하는 라이브러리 한 줄.</summary>
public sealed record LibraryInfo(string Name, string Version, string Purpose, string License, string Url);

/// <summary>
/// 프로그램 · 사용 라이브러리 · 실행 환경 버전.
/// 라이브러리 목록은 외부(서드파티) 라이브러리만 둔다. .NET · Microsoft 가 제공하는 라이브러리
/// (Microsoft.Extensions.*, CommunityToolkit.Mvvm 등)는 넣지 않는다.
/// 버전은 실제로 불러온 어셈블리(네이티브 라이브러리는 그 라이브러리가 알려 주는 값)에서 읽으므로
/// 패키지를 올리면 따로 고칠 필요가 없다.
/// </summary>
public sealed class AboutInfo
{
    public required string ProductName { get; init; }
    public required string Version { get; init; }
    public required string Description { get; init; }
    public required IReadOnlyList<LibraryInfo> Libraries { get; init; }
    public required string Runtime { get; init; }
    public required string OperatingSystem { get; init; }

    /// <summary>GitHub 프로젝트 페이지 (프로젝트 파일의 UpdateRepository 값으로 만든다, 없으면 null).</summary>
    public string? ProjectUrl { get; init; }

    public static AboutInfo Create()
    {
        var app = typeof(AboutInfo).Assembly;
        return new AboutInfo
        {
            ProductName = app.GetCustomAttribute<AssemblyProductAttribute>()?.Product ?? "ImageViewer",
            Version = VersionOf(app),
            Description = "폴더 · ZIP 이미지 뷰어",
            Libraries =
            [
                new("Magick.NET (Q8, OpenMP x64)", VersionOf(typeof(MagickImage).Assembly),
                    "이미지 디코딩 · 리사이즈 · 필터", "Apache-2.0", "https://github.com/dlemstra/Magick.NET"),
                new("ImageMagick", NativeImageMagickVersion(),
                    "Magick.NET 의 처리 엔진", "ImageMagick License", "https://imagemagick.org"),
                new("SkiaSharp", $"{VersionOf(typeof(SKCodec).Assembly)} (Skia {SkiaSharpVersion.Native.ToString(2)})",
                    "GIF · WebP 애니메이션 디코딩", "MIT", "https://github.com/mono/SkiaSharp"),
                new("SharpCompress", VersionOf(typeof(IArchive).Assembly),
                    "ZIP · CBZ 압축 파일 읽기", "MIT", "https://github.com/adamhathcock/sharpcompress"),
            ],
            Runtime = $"{RuntimeInformation.FrameworkDescription} ({RuntimeInformation.ProcessArchitecture.ToString().ToLowerInvariant()})",
            OperatingSystem = RuntimeInformation.OSDescription,
            ProjectUrl = Updates.UpdateService.Repository is { Length: > 0 } repo ? $"https://github.com/{repo}" : null,
        };
    }

    /// <summary>버그 신고 등에 붙여 넣을 수 있는 텍스트.</summary>
    public string ToText()
    {
        var lines = new List<string>
        {
            $"{ProductName} {Version}",
            "",
            "라이브러리:",
        };
        lines.AddRange(Libraries.Select(l => $"  {l.Name} {l.Version} ({l.License})"));
        lines.Add("");
        lines.Add($"런타임: {Runtime}");
        lines.Add($"운영체제: {OperatingSystem}");
        if (ProjectUrl is not null)
        {
            lines.Add($"프로젝트: {ProjectUrl}");
        }
        return string.Join(Environment.NewLine, lines);
    }

    /// <summary>InformationalVersion (빌드 메타데이터 "+커밋" 은 뺀다), 없으면 어셈블리 버전.</summary>
    internal static string VersionOf(Assembly assembly)
    {
        var informational = assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion;
        if (!string.IsNullOrWhiteSpace(informational))
        {
            var plus = informational.IndexOf('+');
            return plus > 0 ? informational[..plus] : informational;
        }

        return assembly.GetName().Version?.ToString(3) ?? "?";
    }

    private static string NativeImageMagickVersion()
    {
        try
        {
            // 예: "ImageMagick 7.1.2-8 Q8 x64 ..." 에서 버전 부분만.
            var text = MagickNET.ImageMagickVersion;
            var parts = text.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            return parts.Length >= 2 && parts[0] == "ImageMagick" ? parts[1] : text;
        }
        catch (Exception)
        {
            return "?";
        }
    }
}
