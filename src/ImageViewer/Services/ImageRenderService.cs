using System.Windows.Media;
using System.Windows.Media.Imaging;
using ImageMagick;
using ImageViewer.Models;

namespace ImageViewer.Services;

/// <summary>화면에 표시할 애니메이션 프레임 (비트맵 + 표시 시간).</summary>
public sealed record RenderedFrame(BitmapSource Bitmap, TimeSpan Delay);

/// <summary>
/// 디코딩된 이미지를 화면 표시 크기로 변환하고 필터 · 회전 · 반전을 적용한 뒤 WPF BitmapSource 로 만든다.
/// 결과는 (이미지, 크기, 필터, 변환) 단위로 LRU 캐시한다. 모든 처리는 UI 스레드 밖에서 수행한다.
/// </summary>
public sealed class ImageRenderService(ImageFilterService filterService)
{
    private readonly record struct RenderKey(string ImageKey, uint Width, uint Height, ImageFilter Filter, ImageTransform Transform);

    private readonly record struct AnimationKey(string ImageKey, ImageTransform Transform);

    private readonly LruCache<RenderKey, BitmapSource> _cache = new(
        maxCount: 16,
        maxCost: 384L * 1024 * 1024,
        costOf: b => (long)b.PixelWidth * b.PixelHeight * 4);

    private readonly LruCache<AnimationKey, IReadOnlyList<RenderedFrame>> _animationCache = new(
        maxCount: 4,
        maxCost: 512L * 1024 * 1024,
        costOf: frames => frames.Sum(f => (long)f.Bitmap.PixelWidth * f.Bitmap.PixelHeight * 4));

    private readonly SemaphoreSlim _gate = new(1, 1);

    /// <summary>
    /// 정지 이미지를 렌더링한다. <paramref name="width"/> · <paramref name="height"/> 는 회전 후(화면 기준) 크기.
    /// </summary>
    public async Task<BitmapSource> RenderAsync(
        DecodedImage image,
        uint width,
        uint height,
        ImageFilter filter,
        ImageTransform transform,
        CancellationToken cancellationToken)
    {
        var key = new RenderKey(image.Key, width, height, filter, transform);
        if (_cache.TryGet(key, out var cached))
        {
            return cached;
        }

        await _gate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            if (_cache.TryGet(key, out cached))
            {
                return cached;
            }

            var bitmap = await Task.Run(() =>
            {
                cancellationToken.ThrowIfCancellationRequested();
                using var work = image.CloneImage() ?? throw new ObjectDisposedException(nameof(DecodedImage));

                // 회전 전 기준 크기로 먼저 축소한 뒤 회전한다 (픽셀 수가 적을 때 회전하는 편이 빠르다).
                var (targetW, targetH) = transform.SwapsDimensions ? (height, width) : (width, height);
                if (work.Width != targetW || work.Height != targetH)
                {
                    var geometry = new MagickGeometry(targetW, targetH) { IgnoreAspectRatio = true };
                    var ratio = (double)targetW / work.Width;
                    if (ratio < 0.25)
                    {
                        // 큰 폭으로 축소할 때는 Thumbnail 이 빠르다 (샘플링 후 리샘플링).
                        work.Thumbnail(geometry);
                    }
                    else
                    {
                        work.Resize(geometry);
                    }
                }

                cancellationToken.ThrowIfCancellationRequested();
                filterService.Apply(work, filter);
                transform.Apply(work);
                cancellationToken.ThrowIfCancellationRequested();
                return ToBitmapSource(work);
            }, cancellationToken).ConfigureAwait(false);

            _cache.Add(key, bitmap);
            return bitmap;
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>
    /// 애니메이션 프레임을 원본 해상도로 렌더링한다 (화면 확대 · 축소는 WPF 가 처리).
    /// 필터는 적용하지 않고 회전 · 반전만 적용한다.
    /// </summary>
    public async Task<IReadOnlyList<RenderedFrame>> RenderAnimationAsync(
        DecodedImage image,
        ImageTransform transform,
        CancellationToken cancellationToken)
    {
        var key = new AnimationKey(image.Key, transform);
        if (_animationCache.TryGet(key, out var cached))
        {
            return cached;
        }

        await _gate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            if (_animationCache.TryGet(key, out cached))
            {
                return cached;
            }

            var frames = await Task.Run<IReadOnlyList<RenderedFrame>>(() =>
            {
                var clones = image.CloneFrames() ?? throw new ObjectDisposedException(nameof(DecodedImage));
                var result = new List<RenderedFrame>(clones.Count);
                try
                {
                    foreach (var frame in clones)
                    {
                        cancellationToken.ThrowIfCancellationRequested();
                        transform.Apply(frame.Image);
                        result.Add(new RenderedFrame(ToBitmapSource(frame.Image), TimeSpan.FromMilliseconds(frame.DelayMs)));
                    }
                }
                finally
                {
                    foreach (var frame in clones)
                    {
                        frame.Image.Dispose();
                    }
                }

                return result;
            }, cancellationToken).ConfigureAwait(false);

            _animationCache.Add(key, frames);
            return frames;
        }
        finally
        {
            _gate.Release();
        }
    }

    /// <summary>파일이 바뀌었을 때 해당 이미지의 렌더링 결과를 버린다.</summary>
    public void Invalidate(ImageInfo info)
    {
        var prefix = info.Key + "#";
        _cache.RemoveWhere(k => k.ImageKey.StartsWith(prefix, StringComparison.Ordinal));
        _animationCache.RemoveWhere(k => k.ImageKey.StartsWith(prefix, StringComparison.Ordinal));
    }

    public void ClearCache()
    {
        _cache.Clear();
        _animationCache.Clear();
    }

    /// <summary>
    /// MagickImage 의 픽셀을 BGRA 8bit 배열로 복사해 고정(Freeze)된 BitmapSource 를 만든다.
    /// 반환된 비트맵은 자체 버퍼를 가지므로 MagickImage 를 해제해도 안전하다.
    /// </summary>
    public static BitmapSource ToBitmapSource(MagickImage image)
    {
        var width = (int)image.Width;
        var height = (int)image.Height;
        using var pixels = image.GetPixelsUnsafe();
        var bytes = pixels.ToByteArray(PixelMapping.BGRA)
                    ?? throw new InvalidOperationException("픽셀 데이터를 읽을 수 없습니다.");

        var bitmap = BitmapSource.Create(width, height, 96, 96, PixelFormats.Bgra32, null, bytes, width * 4);
        bitmap.Freeze();
        return bitmap;
    }
}
