using ImageMagick;
using ImageViewer.Helpers;
using ImageViewer.Models;
using ImageViewer.Services.ImageSources;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace ImageViewer.Services;

/// <summary>디코딩 실패 등 사용자에게 안내할 이미지 오류.</summary>
public sealed class ImageLoadException(string message, Exception? inner = null) : Exception(message, inner);

/// <summary>
/// 화면 표시에 필요한 크기만큼만 디코딩하기 위한 정보 (장치 픽셀 기준 표시 영역과 표시 모드).
/// JPEG 은 디코딩 단계에서 1/2 · 1/4 · 1/8 등으로 줄여 읽을 수 있어 큰 사진을 훨씬 빨리 연다.
/// 원본 크기 보기 · 확대 등 더 큰 해상도가 필요하면 원본을 다시 디코딩한다.
/// </summary>
public sealed record DecodeTarget(DisplayMode Mode, double ViewportPixelWidth, double ViewportPixelHeight)
{
    /// <summary>
    /// 원본(방향 반영) 크기가 <paramref name="width"/>×<paramref name="height"/> 인 이미지를 표시하는 데 필요한 크기.
    /// 원본에 가까운 크기가 필요하면(축소 효과가 작으면) null = 원본 디코딩.
    /// </summary>
    public (uint Width, uint Height)? RequiredSize(uint width, uint height)
    {
        if (width == 0 || height == 0 || ViewportPixelWidth <= 0 || ViewportPixelHeight <= 0)
        {
            return null;
        }

        var scale = ImageResizeService.GetBaseScale(Mode, width, height, ViewportPixelWidth, ViewportPixelHeight);
        if (scale >= 0.9)
        {
            return null;
        }

        return ((uint)Math.Ceiling(width * scale), (uint)Math.Ceiling(height * scale));
    }

    /// <summary>이미 디코딩된 이미지가 이 표시 조건에 충분한 해상도인지.</summary>
    public bool IsSatisfiedBy(DecodedImage image)
    {
        if (!image.IsReduced)
        {
            return true;
        }

        return RequiredSize(image.OriginalWidth, image.OriginalHeight) is { } required &&
               image.Width >= required.Width && image.Height >= required.Height;
    }
}

/// <summary>애니메이션 한 프레임 (GIF / WebP, 합성(coalesce)된 전체 캔버스).</summary>
public sealed record AnimationFrame(MagickImage Image, int DelayMs);

/// <summary>디코딩 결과. 애니메이션이면 <see cref="Frames"/>, 다중 페이지면 <see cref="PageCount"/> 가 1보다 크다.</summary>
public sealed class DecodeResult(MagickImage image, string format)
{
    public MagickImage Image { get; } = image;

    public string Format { get; } = format;

    /// <summary>원본(방향 반영) 크기. 화면 크기로 줄여 디코딩했으면 <see cref="Image"/> 보다 크다.</summary>
    public uint OriginalWidth { get; init; } = image.Width;

    public uint OriginalHeight { get; init; } = image.Height;

    public int PageIndex { get; init; }

    public int PageCount { get; init; } = 1;

    public IReadOnlyList<AnimationFrame>? Frames { get; init; }

    public IReadOnlyList<MetadataItem> Exif { get; init; } = [];

    public void Deconstruct(out MagickImage image, out string format)
    {
        image = Image;
        format = Format;
    }
}

/// <summary>
/// 디코딩된 원본 이미지 (EXIF 방향 반영, sRGB 변환).
/// 다중 페이지 이미지는 페이지 하나, 애니메이션은 모든 프레임을 가진다.
/// 캐시와 화면이 동시에 참조할 수 있으므로 참조 카운트로 해제 시점을 관리한다.
/// </summary>
public sealed class DecodedImage
{
    private readonly MagickImage _image;
    private readonly IReadOnlyList<AnimationFrame>? _frames;
    private readonly Lock _lock = new();
    private int _refCount = 1;

    internal DecodedImage(ImageInfo info, DecodeResult result)
    {
        Info = info;
        _image = result.Image;
        _frames = result.Frames;
        Width = _image.Width;
        Height = _image.Height;
        OriginalWidth = Math.Max(result.OriginalWidth, Width);
        OriginalHeight = Math.Max(result.OriginalHeight, Height);
        Format = result.Format;
        PageIndex = result.PageIndex;
        PageCount = result.PageCount;
        FrameCount = _frames?.Count ?? 1;
        FrameDelaysMs = _frames?.Select(f => f.DelayMs).ToArray() ?? [];
        Exif = result.Exif;
    }

    public ImageInfo Info { get; }

    /// <summary>디코딩된 픽셀 크기 (화면 크기로 줄여 읽었으면 원본보다 작다).</summary>
    public uint Width { get; }

    public uint Height { get; }

    /// <summary>원본 이미지 크기 (EXIF 방향 반영). 배율 계산과 정보 표시에 쓴다.</summary>
    public uint OriginalWidth { get; }

    public uint OriginalHeight { get; }

    /// <summary>화면 표시용으로 원본보다 작게 디코딩했는지.</summary>
    public bool IsReduced => Width < OriginalWidth || Height < OriginalHeight;

    public string Format { get; }

    /// <summary>다중 페이지 이미지(TIFF)의 현재 페이지 (0부터).</summary>
    public int PageIndex { get; }

    public int PageCount { get; }

    public int FrameCount { get; }

    public bool IsAnimated => FrameCount > 1;

    /// <summary>애니메이션 각 프레임의 표시 시간(ms). 애니메이션이 아니면 비어 있다.</summary>
    public IReadOnlyList<int> FrameDelaysMs { get; }

    public IReadOnlyList<MetadataItem> Exif { get; }

    /// <summary>
    /// 렌더링 캐시 키 (이미지 + 페이지 + 디코딩 해상도).
    /// 줄여 읽은 결과와 원본 결과를 구분해, 원본으로 바꾼 뒤 흐린 렌더링이 재사용되지 않게 한다.
    /// </summary>
    public string Key => ImageLoaderService.CacheKey(Info, PageIndex) + (IsReduced ? $"~{Width}x{Height}" : string.Empty);

    /// <summary>메모리 사용량 추정치 (8bit BGRA 기준).</summary>
    public long Cost => (long)Width * Height * 4 * Math.Max(1, FrameCount);

    public bool IsDisposed
    {
        get
        {
            lock (_lock)
            {
                return _refCount <= 0;
            }
        }
    }

    public bool TryAcquire()
    {
        lock (_lock)
        {
            if (_refCount <= 0)
            {
                return false;
            }

            _refCount++;
            return true;
        }
    }

    public void Release()
    {
        lock (_lock)
        {
            if (_refCount <= 0)
            {
                return;
            }

            if (--_refCount == 0)
            {
                _image.Dispose();
                if (_frames is not null)
                {
                    foreach (var frame in _frames)
                    {
                        frame.Image.Dispose();
                    }
                }
            }
        }
    }

    /// <summary>
    /// 작업용 복제본을 만든다. ImageMagick 의 복제는 픽셀 캐시를 공유(copy-on-write)하므로 가볍다.
    /// 원본이 이미 해제되었으면 null.
    /// </summary>
    public MagickImage? CloneImage()
    {
        lock (_lock)
        {
            return _refCount <= 0 ? null : (MagickImage)_image.Clone();
        }
    }

    /// <summary>애니메이션 프레임 복제본. 애니메이션이 아니거나 해제되었으면 null.</summary>
    public IReadOnlyList<AnimationFrame>? CloneFrames()
    {
        lock (_lock)
        {
            if (_refCount <= 0 || _frames is null)
            {
                return null;
            }

            return _frames.Select(f => new AnimationFrame((MagickImage)f.Image.Clone(), f.DelayMs)).ToList();
        }
    }
}

/// <summary>
/// Magick.NET 으로 이미지를 디코딩한다. 원본 크기의 디코딩 결과는 크기 제한 LRU 캐시에 보관한다.
/// </summary>
public sealed class ImageLoaderService : IDisposable
{
    private readonly ILogger<ImageLoaderService> _logger;
    private readonly SettingsService _settings;
    private readonly LruCache<string, DecodedImage> _cache;

    // 전체 크기 디코딩은 메모리 사용량이 크므로 한 번에 하나만 수행한다.
    // (ImageMagick 자체가 내부적으로 멀티스레드로 처리한다)
    private readonly SemaphoreSlim _decodeGate = new(1, 1);

    public ImageLoaderService(SettingsService settings, ILogger<ImageLoaderService>? logger = null)
    {
        _settings = settings;
        _logger = logger ?? NullLogger<ImageLoaderService>.Instance;
        _cache = new LruCache<string, DecodedImage>(
            maxCount: 8,
            maxCost: 1024L * 1024 * 1024,
            costOf: d => d.Cost,
            onEvicted: (_, d) => d.Release());
    }

    private long MaxPixels => (long)_settings.Current.Limits.MaxImageMegapixels * 1_000_000;

    /// <summary>애니메이션 · 다중 페이지 이미지의 최대 프레임 수.</summary>
    public const int MaxFrames = 1000;

    internal static string CacheKey(ImageInfo info, int pageIndex) => $"{info.Key}#{pageIndex}";

    public Task<DecodedImage> LoadAsync(IImageSource source, ImageInfo info, CancellationToken cancellationToken) =>
        LoadAsync(source, info, 0, target: null, cancellationToken);

    public Task<DecodedImage> LoadAsync(IImageSource source, ImageInfo info, int pageIndex, CancellationToken cancellationToken) =>
        LoadAsync(source, info, pageIndex, target: null, cancellationToken);

    /// <summary>
    /// 이미지(다중 페이지면 지정한 페이지)를 디코딩해 반환한다.
    /// <paramref name="target"/> 이 있으면 화면 표시에 필요한 크기까지만 디코딩할 수 있다 (JPEG).
    /// 캐시에 원본이 있거나 충분히 큰 결과가 있으면 그것을 돌려준다.
    /// 반환된 객체는 호출자가 사용 후 <see cref="DecodedImage.Release"/> 해야 한다.
    /// </summary>
    public async Task<DecodedImage> LoadAsync(
        IImageSource source,
        ImageInfo info,
        int pageIndex,
        DecodeTarget? target,
        CancellationToken cancellationToken)
    {
        var fullKey = CacheKey(info, pageIndex);
        var reducedKey = fullKey + ReducedSuffix;
        if (TryGetSuitable(fullKey, reducedKey, target, out var cached))
        {
            return cached;
        }

        await _decodeGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            // 대기하는 동안 미리 읽기 작업이 같은 이미지를 디코딩했을 수 있다.
            if (TryGetSuitable(fullKey, reducedKey, target, out cached))
            {
                return cached;
            }

            cancellationToken.ThrowIfCancellationRequested();

            var decoded = await Task.Run(async () =>
            {
                await using var stream = await OpenSeekableAsync(source, info, cancellationToken).ConfigureAwait(false);
                cancellationToken.ThrowIfCancellationRequested();
                var result = Decode(stream, MaxPixels, thumbnailHint: null, info.Name, pageIndex, target);
                return new DecodedImage(info, result);
            }, cancellationToken).ConfigureAwait(false);

            // 캐시가 한 참조를, 호출자가 한 참조를 가진다.
            // (범위를 벗어난 페이지를 요청했으면 실제 페이지 키로 저장된다)
            decoded.TryAcquire();
            _cache.Add(CacheKey(info, decoded.PageIndex) + (decoded.IsReduced ? ReducedSuffix : string.Empty), decoded);
            return decoded;
        }
        finally
        {
            _decodeGate.Release();
        }
    }

    private const string ReducedSuffix = "~r";

    /// <summary>원본이 캐시에 있으면 원본을, 없으면 이 표시 조건에 충분한 축소 결과를 찾는다.</summary>
    private bool TryGetSuitable(string fullKey, string reducedKey, DecodeTarget? target, out DecodedImage decoded)
    {
        if (TryGetCached(fullKey, out decoded))
        {
            return true;
        }

        if (target is not null && TryGetCached(reducedKey, out decoded))
        {
            if (target.IsSatisfiedBy(decoded))
            {
                return true;
            }

            decoded.Release();
        }

        decoded = null!;
        return false;
    }

    private bool TryGetCached(string key, out DecodedImage decoded)
    {
        if (_cache.TryGet(key, out decoded!) && decoded.TryAcquire())
        {
            return true;
        }

        decoded = null!;
        return false;
    }

    public void ClearCache() => _cache.Clear();

    /// <summary>파일이 바뀌었을 때 해당 이미지의 모든 페이지를 캐시에서 제거한다.</summary>
    public void Invalidate(ImageInfo info)
    {
        var prefix = info.Key + "#";
        _cache.RemoveWhere(k => k.StartsWith(prefix, StringComparison.Ordinal));
    }

    public void Dispose() => _cache.Clear();

    internal static async Task<Stream> OpenSeekableAsync(IImageSource source, ImageInfo info, CancellationToken cancellationToken)
    {
        if (!info.IsSupported)
        {
            throw new ImageLoadException(info.UnsupportedReason ?? "지원하지 않는 항목입니다.");
        }

        Stream stream;
        try
        {
            stream = await source.OpenImageAsync(info, cancellationToken).ConfigureAwait(false);
        }
        catch (ImageSourceException ex)
        {
            throw new ImageLoadException(ex.Message, ex);
        }

        if (stream.CanSeek)
        {
            return stream;
        }

        await using (stream)
        {
            var memory = new MemoryStream();
            await stream.CopyToAsync(memory, cancellationToken).ConfigureAwait(false);
            memory.Position = 0;
            return memory;
        }
    }

    /// <summary>
    /// 공통 디코딩 절차.
    /// 1) 헤더 검사로 스크립트/문서 포맷 거부 2) Ping 으로 포맷·픽셀 수 검증
    /// 3) 프레임 수 확인 (GIF / WebP 애니메이션, 다중 페이지 TIFF)
    /// 4) 디코딩 5) EXIF 읽기 · 방향 반영 6) sRGB 변환
    /// 내용으로 포맷을 판별할 수 없는 경우(ICO, TGA, RAW 등)에는 파일 확장자를 힌트로 다시 시도한다.
    /// 썸네일(<paramref name="thumbnailHint"/>)은 항상 첫 프레임 / 첫 페이지만 읽는다.
    /// </summary>
    internal static DecodeResult Decode(
        Stream stream,
        long maxPixels,
        uint? thumbnailHint,
        string? fileName = null,
        int pageIndex = 0,
        DecodeTarget? target = null)
    {
        Span<byte> header = stackalloc byte[64];
        var read = stream.Read(header);
        stream.Position = 0;
        if (read == 0)
        {
            throw new ImageLoadException("빈 파일입니다.");
        }

        if (ImageFormatHelper.LooksLikeScriptOrDocument(header[..read]))
        {
            throw new ImageLoadException("지원하지 않는 형식입니다. (텍스트/벡터/문서 형식)");
        }

        var settings = new MagickReadSettings
        {
            FrameIndex = 0,
            FrameCount = 1,
        };

        if (thumbnailHint is { } hint)
        {
            // JPEG 은 디코딩 단계에서 축소해 썸네일 생성 속도를 높인다.
            settings.SetDefine(MagickFormat.Jpeg, "size", $"{hint}x{hint}");
        }

        MagickFormat format;
        uint canvasWidth, canvasHeight;
        uint originalWidth, originalHeight;
        try
        {
            MagickImageInfo info;
            try
            {
                info = new MagickImageInfo(stream, settings);
            }
            catch (MagickMissingDelegateErrorException) when (GetFormatHint(fileName) is { } hinted)
            {
                stream.Position = 0;
                settings.Format = hinted;
                info = new MagickImageInfo(stream, settings);
            }

            stream.Position = 0;
            format = info.Format;

            if (ImageFormatHelper.BlockedFormats.Contains(format))
            {
                throw new ImageLoadException($"지원하지 않는 형식입니다. ({format})");
            }

            canvasWidth = info.Width;
            canvasHeight = info.Height;

            // EXIF 방향이 90° 회전(5~8)이면 화면 기준 폭 · 높이가 바뀐다.
            var swapped = (int)info.Orientation is >= 5 and <= 8;
            (originalWidth, originalHeight) = swapped ? (info.Height, info.Width) : (info.Width, info.Height);

            // JPEG: 화면에 필요한 크기만큼만 디코딩 (libjpeg 의 1/2 · 1/4 · 1/8 등 축소 디코딩)
            if (thumbnailHint is null && target?.RequiredSize(originalWidth, originalHeight) is { } required &&
                format is MagickFormat.Jpeg or MagickFormat.Jpg or MagickFormat.Jpe or MagickFormat.Pjpeg)
            {
                var (hintW, hintH) = swapped ? (required.Height, required.Width) : (required.Width, required.Height);
                settings.SetDefine(MagickFormat.Jpeg, "size", $"{hintW}x{hintH}");
            }
            var pixels = (long)info.Width * info.Height;
            if (info.Width == 0 || info.Height == 0)
            {
                throw new ImageLoadException("이미지 크기 정보를 읽을 수 없습니다.");
            }

            if (pixels > maxPixels)
            {
                throw new ImageLoadException(
                    $"이미지가 너무 큽니다. ({info.Width:N0} × {info.Height:N0}, 허용 한도 {maxPixels / 1_000_000:N0} MP)");
            }
        }
        catch (MagickException ex)
        {
            throw new ImageLoadException(DescribeMagickError(ex), ex);
        }

        var formatName = format.ToString().ToUpperInvariant();
        var animated = formatName is "GIF" or "WEBP";
        var paged = formatName is "TIFF" or "TIF" or "TIFF64" or "PTIF";
        var frameCount = 1;
        if (thumbnailHint is null && (animated || paged))
        {
            frameCount = CountFrames(stream, settings.Format);
        }

        // 애니메이션: 모든 프레임의 픽셀 합이 한도 안이면 전체 프레임을 읽는다 (넘으면 첫 프레임만 표시).
        if (animated && frameCount > 1 && (long)canvasWidth * canvasHeight * frameCount <= maxPixels)
        {
            return ReadAnimation(stream, settings.Format, formatName);
        }

        var page = paged ? Math.Clamp(pageIndex, 0, frameCount - 1) : 0;
        settings.FrameIndex = (uint)page;

        MagickImage? image = null;
        try
        {
            if (format is MagickFormat.Ico or MagickFormat.Cur)
            {
                image = ReadLargestFrame(stream, settings.Format);
            }
            else
            {
                image = new MagickImage(stream, settings);
            }

            var exif = thumbnailHint is null ? ExifReader.Read(image) : [];
            image.AutoOrient();
            NormalizeColor(image);
            return new DecodeResult(image, formatName)
            {
                OriginalWidth = Math.Max(originalWidth, image.Width),
                OriginalHeight = Math.Max(originalHeight, image.Height),
                PageIndex = page,
                PageCount = paged ? frameCount : 1,
                Exif = exif,
            };
        }
        catch (MagickException ex)
        {
            image?.Dispose();
            throw new ImageLoadException(DescribeMagickError(ex), ex);
        }
        catch
        {
            image?.Dispose();
            throw;
        }
    }

    /// <summary>Ping 으로 프레임(페이지) 수를 센다. 실패하면 1.</summary>
    private static int CountFrames(Stream stream, MagickFormat? formatHint)
    {
        try
        {
            var probe = new MagickReadSettings();
            if (formatHint is { } f)
            {
                probe.Format = f;
            }

            return Math.Clamp(MagickImageInfo.ReadCollection(stream, probe).Take(MaxFrames).Count(), 1, MaxFrames);
        }
        catch (MagickException)
        {
            return 1;
        }
        finally
        {
            stream.Position = 0;
        }
    }

    /// <summary>
    /// 애니메이션 전체 프레임을 읽어 합성(coalesce)한다.
    /// 지연 시간이 0 또는 너무 짧은 프레임은 브라우저와 같이 100ms 로 표시한다.
    /// </summary>
    private static DecodeResult ReadAnimation(Stream stream, MagickFormat? formatHint, string formatName)
    {
        var settings = new MagickReadSettings();
        if (formatHint is { } f)
        {
            settings.Format = f;
        }

        var frames = new List<AnimationFrame>();
        try
        {
            using var collection = new MagickImageCollection(stream, settings);
            collection.Coalesce();
            foreach (var frame in collection.Take(MaxFrames))
            {
                var ticksPerSecond = frame.AnimationTicksPerSecond > 0 ? frame.AnimationTicksPerSecond : 100;
                var delayMs = (int)(frame.AnimationDelay * 1000L / ticksPerSecond);
                if (delayMs < 20)
                {
                    delayMs = 100;
                }

                var image = (MagickImage)frame.Clone();
                NormalizeColor(image);
                frames.Add(new AnimationFrame(image, delayMs));
            }

            if (frames.Count == 0)
            {
                throw new ImageLoadException("애니메이션 프레임이 없습니다.");
            }

            return new DecodeResult((MagickImage)frames[0].Image.Clone(), formatName)
            {
                Frames = frames,
            };
        }
        catch (MagickException ex)
        {
            foreach (var frame in frames)
            {
                frame.Image.Dispose();
            }

            throw new ImageLoadException(DescribeMagickError(ex), ex);
        }
    }

    private static MagickFormat? GetFormatHint(string? fileName)
    {
        var ext = Path.GetExtension(fileName)?.TrimStart('.');
        if (string.IsNullOrEmpty(ext) || !ImageFormatHelper.IsSupportedExtension(fileName))
        {
            return null;
        }

        return Enum.TryParse<MagickFormat>(ext, ignoreCase: true, out var format) &&
               !ImageFormatHelper.BlockedFormats.Contains(format)
            ? format
            : null;
    }

    private static MagickImage ReadLargestFrame(Stream stream, MagickFormat? format)
    {
        var settings = new MagickReadSettings();
        if (format is { } f)
        {
            settings.Format = f;
        }

        using var collection = new MagickImageCollection(stream, settings);
        var best = collection.OrderByDescending(f => (long)f.Width * f.Height).First();
        return new MagickImage(best);
    }

    private static void NormalizeColor(MagickImage image)
    {
        try
        {
            var profile = image.GetColorProfile();
            if (image.ColorSpace == ColorSpace.CMYK)
            {
                if (profile is null)
                {
                    image.TransformColorSpace(ColorProfiles.USWebCoatedSWOP, ColorProfiles.SRGB);
                }
                else
                {
                    image.TransformColorSpace(ColorProfiles.SRGB);
                }
            }
            else if (profile is not null && profile.ColorSpace == ColorSpace.sRGB &&
                     !string.Equals(profile.Description, ColorProfiles.SRGB.Description, StringComparison.Ordinal))
            {
                // 광색역 RGB 프로파일(Adobe RGB, Display P3 등)을 sRGB 로 변환
                image.TransformColorSpace(ColorProfiles.SRGB);
            }

            if (image.ColorSpace is not (ColorSpace.sRGB or ColorSpace.Gray or ColorSpace.RGB or ColorSpace.Transparent))
            {
                image.ColorSpace = ColorSpace.sRGB;
            }
        }
        catch (MagickException)
        {
            // 색상 변환 실패는 표시를 막지 않는다.
        }
    }

    internal static string DescribeMagickError(MagickException ex) => ex switch
    {
        MagickCorruptImageErrorException => "이미지 파일이 손상되었습니다.",
        MagickMissingDelegateErrorException => "지원하지 않는 이미지 형식이거나 코덱이 없습니다.",
        MagickResourceLimitErrorException => "이미지를 처리하기 위한 메모리가 부족합니다.",
        MagickCoderErrorException => "이미지를 디코딩할 수 없습니다.",
        _ => "이미지를 읽을 수 없습니다.",
    };
}
