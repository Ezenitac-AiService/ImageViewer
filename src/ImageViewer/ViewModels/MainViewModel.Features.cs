using System.Collections.ObjectModel;
using System.Globalization;
using System.Windows.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ImageViewer.Helpers;
using ImageViewer.Models;
using ImageViewer.Services;
using ImageViewer.Services.ImageSources;
using Microsoft.Extensions.Logging;

namespace ImageViewer.ViewModels;

/// <summary>
/// 회전 · 반전, 다중 페이지 탐색, 애니메이션 재생, 이미지 정보 패널, 폴더 감시.
/// </summary>
public sealed partial class MainViewModel
{
    private ImageTransform _transform = ImageTransform.Identity;
    private IReadOnlyList<RenderedFrame>? _animationFrames;
    private int _animationIndex;
    private DispatcherTimer? _animationTimer;
    private DispatcherTimer? _watchDebounce;
    private IImageSource? _watchedSource;
    private int _refreshVersion;

    #region 회전 · 반전

    [RelayCommand]
    private void RotateClockwise() => ApplyTransform(_transform.RotateClockwise());

    [RelayCommand]
    private void RotateCounterClockwise() => ApplyTransform(_transform.RotateCounterClockwise());

    [RelayCommand]
    private void FlipHorizontal() => ApplyTransform(_transform.FlipHorizontally());

    [RelayCommand]
    private void FlipVertical() => ApplyTransform(_transform.FlipVertically());

    [RelayCommand]
    private void ResetTransform() => ApplyTransform(ImageTransform.Identity);

    private void ApplyTransform(ImageTransform transform)
    {
        if (_decoded is null)
        {
            return;
        }

        _transform = transform;
        _zoomFactor = 1.0;

        // 폭과 높이가 바뀔 수 있으므로 이전 비트맵을 늘려 보여 주지 않고 새로 렌더링한 뒤 가운데에 다시 배치한다.
        _bitmapImageKey = null;
        _pendingNewImage = true;
        _renderDebounce.Stop();
        _ = RenderCurrentAsync();

        if (CurrentImage is { } info)
        {
            UpdateImageInfo(info, _decoded);
        }

        ShowOsd(transform.Describe());
    }

    #endregion

    #region 다중 페이지

    /// <summary>PageUp: 다중 페이지 이미지면 이전 페이지, 아니면 이전 이미지.</summary>
    [RelayCommand]
    private Task PreviousPageAsync()
    {
        if (_decoded is { PageCount: > 1 } d)
        {
            return d.PageIndex > 0 ? GoToPageAsync(d.PageIndex - 1) : ShowEdge("첫 페이지입니다.");
        }

        return PreviousCommand.ExecuteAsync(null);
    }

    /// <summary>PageDown: 다중 페이지 이미지면 다음 페이지, 아니면 다음 이미지.</summary>
    [RelayCommand]
    private Task NextPageAsync()
    {
        if (_decoded is { PageCount: > 1 } d)
        {
            return d.PageIndex < d.PageCount - 1 ? GoToPageAsync(d.PageIndex + 1) : ShowEdge("마지막 페이지입니다.");
        }

        return NextCommand.ExecuteAsync(null);
    }

    private Task GoToPageAsync(int page)
    {
        _slideShow.RestartTimer();
        return LoadCurrentAsync(page, resetTransform: false);
    }

    private Task ShowEdge(string message)
    {
        ShowOsd(message);
        return Task.CompletedTask;
    }

    #endregion

    #region 애니메이션

    /// <summary>GIF / WebP 애니메이션 재생 여부 (끄면 첫 프레임만 표시).</summary>
    public bool PlayAnimations
    {
        get => Settings.Display.PlayAnimations;
        set
        {
            if (Settings.Display.PlayAnimations == value)
            {
                return;
            }

            Settings.Display.PlayAnimations = value;
            OnPropertyChanged();
            if (_decoded is { IsAnimated: true })
            {
                _bitmapImageKey = null;
                _ = RenderCurrentAsync();
            }

            ShowOsd(value ? "애니메이션 재생" : "애니메이션 정지 (첫 프레임 표시)");
        }
    }

    [RelayCommand]
    private void TogglePlayAnimations() => PlayAnimations = !PlayAnimations;

    private void StartAnimation(IReadOnlyList<RenderedFrame> frames)
    {
        // 같은 프레임 묶음이면(창 크기 변경 · 확대 등) 재생 위치를 유지한다.
        if (ReferenceEquals(frames, _animationFrames))
        {
            return;
        }

        StopAnimation();
        if (frames.Count == 0)
        {
            return;
        }

        _animationFrames = frames;
        _animationIndex = 0;
        _animationDelaysMs = frames.Select(f => (int)f.Delay.TotalMilliseconds).ToArray();
        CurrentBitmap = frames[0].Bitmap;
        UpdateAnimationInfo();

        if (_animationTimer is null)
        {
            _animationTimer = new DispatcherTimer(DispatcherPriority.Render, _dispatcher);
            _animationTimer.Tick += OnAnimationTick;
        }

        _animationTimer.Interval = frames[0].Delay;
        _animationTimer.Start();
    }

    private void OnAnimationTick(object? sender, EventArgs e)
    {
        if (_animationFrames is not { Count: > 0 } frames || _animationTimer is null)
        {
            return;
        }

        _animationIndex = (_animationIndex + 1) % frames.Count;
        var frame = frames[_animationIndex];
        CurrentBitmap = frame.Bitmap;
        UpdateAnimationInfo();
        _animationTimer.Interval = frame.Delay;
    }

    private int[] _animationDelaysMs = [];

    /// <summary>
    /// 왼쪽 아래 반투명 표시. 프레임이 하나뿐인 이미지는 null (표시 안 함).
    /// - 애니메이션: 프레임 n / m · 재생 상태 / 재생 위치 · 전체 길이 · 현재 프레임 지연 / 크기 · 형식
    /// - 다중 페이지: 페이지 n / m / 크기 · 형식
    /// </summary>
    [ObservableProperty]
    private string? _frameInfoText;

    private void UpdateAnimationInfo()
    {
        if (_decoded is { } decoded)
        {
            FrameInfoText = FormatAnimationInfo(_animationIndex, _animationDelaysMs, decoded.Width, decoded.Height, decoded.Format, playing: true);
        }
    }

    internal static string FormatAnimationInfo(
        int index,
        IReadOnlyList<int> delaysMs,
        uint width,
        uint height,
        string format,
        bool playing)
    {
        var count = delaysMs.Count;
        var elapsedMs = delaysMs.Take(index).Sum();
        var totalMs = delaysMs.Sum();
        var delay = index < count ? delaysMs[index] : 0;
        return string.Create(CultureInfo.InvariantCulture,
            $"프레임 {index + 1} / {count} \n" +
            $"{elapsedMs / 1000.0:0.00} / {totalMs / 1000.0:0.00}초  ·  지연 {delay}ms\n" +
            $"{width:N0} × {height:N0}  ·  {format}");
    }

    /// <summary>정지 이미지로 렌더링했을 때의 표시 (재생을 끈 애니메이션 · 다중 페이지).</summary>
    internal static string? BuildStaticFrameInfo(DecodedImage? decoded) => decoded switch
    {
        { IsAnimated: true } d => FormatAnimationInfo(0, d.FrameDelaysMs, d.Width, d.Height, d.Format, playing: false),
        { PageCount: > 1 } d => string.Create(CultureInfo.InvariantCulture,
            $"페이지 {d.PageIndex + 1} / {d.PageCount}\n{d.Width:N0} × {d.Height:N0}  ·  {d.Format}"),
        _ => null,
    };

    private void StopAnimation()
    {
        _animationTimer?.Stop();
        _animationFrames = null;
    }

    #endregion

    #region 이미지 정보 패널

    [ObservableProperty]
    private IReadOnlyList<MetadataItem> _infoItems = [];

    /// <summary>Tab 으로 여닫는 이미지 정보(EXIF) 패널.</summary>
    public bool IsInfoPanelVisible
    {
        get => Settings.Display.InfoPanelVisible;
        set
        {
            if (Settings.Display.InfoPanelVisible == value)
            {
                return;
            }

            Settings.Display.InfoPanelVisible = value;
            OnPropertyChanged();
        }
    }

    [RelayCommand]
    private void ToggleInfoPanel() => IsInfoPanelVisible = !IsInfoPanelVisible;

    private void UpdateImageInfo(ImageInfo info, DecodedImage? decoded)
    {
        InfoItems = BuildInfoItems(info, decoded, _transform);
    }

    internal static IReadOnlyList<MetadataItem> BuildInfoItems(ImageInfo info, DecodedImage? decoded, ImageTransform transform)
    {
        var items = new List<MetadataItem>
        {
            MetadataItem.Header("파일"),
            new("이름", info.DisplayPath),
        };

        if (info.Size >= 0)
        {
            items.Add(new("크기", $"{FileSizeHelper.Format(info.Size)} ({info.Size.ToString("N0", CultureInfo.InvariantCulture)} 바이트)"));
        }

        if (info.ModifiedAt is { } modified)
        {
            items.Add(new("수정 날짜", modified.ToString("yyyy-MM-dd HH:mm:ss", CultureInfo.InvariantCulture)));
        }

        if (decoded is null)
        {
            return items;
        }

        items.Add(MetadataItem.Header("이미지"));
        items.Add(new("해상도", $"{decoded.OriginalWidth:N0} × {decoded.OriginalHeight:N0}"));
        items.Add(new("형식", decoded.Format));
        if (decoded.PageCount > 1)
        {
            items.Add(new("페이지", $"{decoded.PageIndex + 1} / {decoded.PageCount}"));
        }

        if (decoded.IsAnimated)
        {
            items.Add(new("프레임", $"{decoded.FrameCount}개 (애니메이션)"));
        }

        if (!transform.IsIdentity)
        {
            items.Add(new("표시 변환", transform.Describe()));
        }

        if (decoded.Exif.Count > 0)
        {
            items.Add(MetadataItem.Header("EXIF"));
            items.AddRange(decoded.Exif);
        }

        return items;
    }

    #endregion

    #region 빠른 표시: 화면 크기 디코딩 · 원본 업그레이드 · 미리 읽기

    private string? _upgradingKey;

    /// <summary>현재 표시 영역 · 모드에서 필요한 디코딩 크기 정보. 표시 영역을 아직 모르면 null (원본 디코딩).</summary>
    private DecodeTarget? CurrentDecodeTarget() =>
        _viewportWidth > 0 && _viewportHeight > 0
            ? new DecodeTarget(DisplayMode, _viewportWidth * _dpiScale, _viewportHeight * _dpiScale)
            : null;

    /// <summary>
    /// 줄여 읽은 이미지로 그려야 할 크기가 디코딩된 크기보다 크면(원본 크기 모드 · 확대 · 창 확대)
    /// 원본을 백그라운드에서 다시 읽어 바꿔 끼운다. 그동안은 줄여 읽은 이미지를 늘려 보여 준다.
    /// </summary>
    private void RequestFullResolutionIfNeeded(DecodedImage decoded, DisplayLayout layout, ImageTransform transform)
    {
        if (!decoded.IsReduced)
        {
            return;
        }

        var (width, height) = transform.Apply(decoded.Width, decoded.Height);
        if (layout.RenderPixelWidth <= width * 1.02 && layout.RenderPixelHeight <= height * 1.02)
        {
            return;
        }

        _ = UpgradeToFullResolutionAsync(decoded);
    }

    private async Task UpgradeToFullResolutionAsync(DecodedImage reduced)
    {
        var source = _source;
        if (source is null || _upgradingKey == reduced.Key)
        {
            return;
        }

        _upgradingKey = reduced.Key;
        var loadVersion = _loadVersion;
        try
        {
            var full = await _loader.LoadAsync(source, reduced.Info, reduced.PageIndex, target: null, CancellationToken.None);
            if (loadVersion != _loadVersion || !ReferenceEquals(reduced, _decoded))
            {
                full.Release();
                return;
            }

            // 같은 이미지 · 같은 비율이므로 현재 비트맵을 그대로 둔 채 바꿔 끼우고 정확한 크기로 다시 그린다.
            ReplaceDecoded(full);
            _bitmapImageKey = full.Key;
            _renderDebounce.Stop();
            await RenderCurrentAsync();
        }
        catch (Exception ex) when (ex is not OutOfMemoryException)
        {
            _logger.LogDebug(ex, "원본 해상도 읽기 실패: {Name}", reduced.Info.Name);
        }
        finally
        {
            if (_upgradingKey == reduced.Key)
            {
                _upgradingKey = null;
            }
        }
    }

    /// <summary>
    /// 다음에 볼 가능성이 높은 이미지를 우선순위대로 미리 디코딩하고 현재 표시 조건으로 렌더링까지 해 둔다.
    /// 이동하면 캐시에서 바로 꺼내 보여 줄 수 있다. 다른 이미지로 이동하면 취소된다.
    /// - 슬라이드쇼 재생 중: ① 슬라이드쇼의 다음 이미지(랜덤이면 섞은 순서의 다음) ② 오른쪽 ③ 왼쪽
    /// - 그 밖에는: 이동 방향의 다음 두 장, 반대 방향 한 장
    /// </summary>
    private void StartPrefetch()
    {
        var source = _source;
        if (source is null || _images.Count < 2 || CurrentDecodeTarget() is not { } target)
        {
            return;
        }

        var candidates = GetPrefetchCandidates();
        if (candidates.Count == 0)
        {
            return;
        }

        _prefetchCts?.Cancel();
        _prefetchCts = new CancellationTokenSource();
        _ = PrefetchAsync(source, candidates.Select(i => _images[i]).ToList(), target, Filter, _prefetchCts.Token);
    }

    internal List<int> GetPrefetchCandidates()
    {
        var count = _images.Count;
        var candidates = new List<int>();
        if (CurrentIndex < 0 || count < 2)
        {
            return candidates;
        }

        void AddIndex(int index)
        {
            if (index >= 0 && index < count && index != CurrentIndex && !candidates.Contains(index))
            {
                candidates.Add(index);
            }
        }

        void AddOffset(int offset)
        {
            var index = CurrentIndex + offset;
            if (_navigation.WrapAround)
            {
                index = ((index % count) + count) % count;
            }

            AddIndex(index);
        }

        if (IsSlideShowPlaying)
        {
            // 랜덤 재생이면 이 호출에서 섞은 순서가 만들어지고, 다음 타이머가 같은 이미지를 꺼낸다.
            if (_slideShow.PeekNextIndex(CurrentIndex, count) is { } next)
            {
                AddIndex(next);
            }

            AddOffset(+1);
            AddOffset(-1);
            return candidates;
        }

        var forward = _lastDirection >= 0 ? 1 : -1;
        AddOffset(forward);
        AddOffset(forward * 2);
        AddOffset(-forward);
        return candidates;
    }

    private async Task PrefetchAsync(
        IImageSource source,
        IReadOnlyList<ImageInfo> images,
        DecodeTarget target,
        ImageFilter filter,
        CancellationToken ct)
    {
        foreach (var info in images)
        {
            if (ct.IsCancellationRequested || !ReferenceEquals(source, _source))
            {
                return;
            }

            if (!info.IsSupported)
            {
                continue;
            }

            try
            {
                var decoded = await _loader.LoadAsync(source, info, 0, target, ct);
                try
                {
                    if (decoded.IsAnimated && PlayAnimations)
                    {
                        await _renderer.RenderAnimationAsync(decoded, ImageTransform.Identity, ct);
                    }
                    else
                    {
                        // 이동하면 회전 · 반전과 배율은 초기화되므로 그 조건으로 미리 그린다.
                        var (width, height) = (decoded.OriginalWidth, decoded.OriginalHeight);
                        var layout = ImageResizeService.Calculate(
                            DisplayMode, 1.0, width, height,
                            _viewportWidth * _dpiScale, _viewportHeight * _dpiScale);
                        await _renderer.RenderAsync(decoded, layout.RenderPixelWidth, layout.RenderPixelHeight, filter, ImageTransform.Identity, ct);
                    }
                }
                finally
                {
                    decoded.Release();
                }
            }
            catch (OperationCanceledException)
            {
                return;
            }
            catch (Exception ex)
            {
                _logger.LogDebug(ex, "미리 읽기 실패: {Name}", info.Name);
            }
        }
    }

    #endregion

    #region 폴더 감시

    /// <summary>폴더의 이미지 추가 · 삭제 · 이름 변경 · 수정을 감시해 목록을 자동으로 갱신한다.</summary>
    public bool WatchFolder => Settings.Navigation.WatchFolder;

    private void AttachWatcher(IImageSource source)
    {
        if (!Settings.Navigation.WatchFolder || source is not FolderImageSource folder)
        {
            return;
        }

        try
        {
            folder.Changed += OnSourceChanged;
            folder.StartWatching();
            _watchedSource = folder;
        }
        catch (Exception ex) when (ex is IOException or ArgumentException or UnauthorizedAccessException or PlatformNotSupportedException)
        {
            folder.Changed -= OnSourceChanged;
            _logger.LogWarning(ex, "폴더 감시를 시작하지 못했습니다.");
        }
    }

    private void DetachWatcher(IImageSource? source)
    {
        if (source is FolderImageSource folder)
        {
            folder.Changed -= OnSourceChanged;
            folder.StopWatching();
        }

        if (ReferenceEquals(source, _watchedSource))
        {
            _watchedSource = null;
        }

        _watchDebounce?.Stop();
    }

    /// <summary>설정 변경 후 감시를 다시 구성한다.</summary>
    private void RefreshWatcher()
    {
        if (_source is null)
        {
            return;
        }

        DetachWatcher(_source);
        AttachWatcher(_source);
    }

    /// <summary>파일 시스템 이벤트(백그라운드 스레드) → 잠시 모았다가 UI 스레드에서 한 번에 갱신.</summary>
    private void OnSourceChanged(object? sender, EventArgs e)
    {
        _dispatcher.BeginInvoke(() =>
        {
            if (!ReferenceEquals(sender, _source))
            {
                return;
            }

            if (_watchDebounce is null)
            {
                _watchDebounce = new DispatcherTimer(DispatcherPriority.Background, _dispatcher)
                {
                    Interval = TimeSpan.FromMilliseconds(700),
                };
                _watchDebounce.Tick += (_, _) =>
                {
                    _watchDebounce.Stop();
                    _ = RefreshSourceAsync();
                };
            }

            _watchDebounce.Stop();
            _watchDebounce.Start();
        });
    }

    /// <summary>
    /// 이미지 목록을 다시 읽는다. 현재 이미지는 그대로 유지하고,
    /// 현재 이미지가 삭제되었으면 같은 위치의 이미지를, 내용이 바뀌었으면 다시 읽어 표시한다.
    /// </summary>
    internal async Task RefreshSourceAsync()
    {
        var source = _source;
        if (source is null)
        {
            return;
        }

        var version = ++_refreshVersion;
        IReadOnlyList<ImageInfo> images;
        try
        {
            images = await source.GetImagesAsync(CancellationToken.None);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "변경된 폴더를 다시 읽지 못했습니다.");
            return;
        }

        if (version != _refreshVersion || !ReferenceEquals(source, _source))
        {
            return;
        }

        var previous = CurrentImage;
        var previousIndex = CurrentIndex;
        var oldCount = _images.Count;

        // 바뀐 파일의 캐시를 버린다.
        var oldByKey = _images.ToDictionary(i => i.Key);
        foreach (var image in images)
        {
            if (oldByKey.TryGetValue(image.Key, out var old) && IsModified(old, image))
            {
                _loader.Invalidate(image);
                _renderer.Invalidate(image);
            }
        }

        var newIndex = previous is null ? -1 : images.ToList().FindIndex(i => i.Key == previous.Key);
        var currentRemoved = previous is not null && newIndex < 0;
        var currentModified = newIndex >= 0 && IsModified(previous!, images[newIndex]);
        if (newIndex < 0 && images.Count > 0)
        {
            newIndex = Math.Clamp(previousIndex, 0, images.Count - 1);
        }

        _images = images;
        _navigation.Reset(images.Count, Math.Max(0, newIndex));
        _slideShow.ResetOrder();
        RebuildThumbnails(images);

        if (images.Count == 0)
        {
            StopSlideShow(showMessage: false);
            CancelPending();
            ReplaceDecoded(null);
            CurrentIndex = -1;
            CurrentImage = null;
            CurrentBitmap = null;
            ErrorMessage = null;
            InfoItems = [];
            FrameInfoText = null;
            PlaceholderMessage = $"이 {source.Info.Label}에는 표시할 이미지가 없습니다.";
            OnPropertyChanged(nameof(PositionText));
            ShowOsd("폴더에 이미지가 없습니다.");
            return;
        }

        PlaceholderMessage = null;
        SetCurrent(newIndex);
        OnPropertyChanged(nameof(PositionText));

        if (currentRemoved || currentModified || _decoded is null)
        {
            _ = LoadCurrentAsync(resetTransform: currentRemoved);
        }

        var delta = images.Count - oldCount;
        ShowOsd(delta switch
        {
            > 0 => $"폴더 변경: 이미지 {delta}개 추가 (총 {images.Count:N0}개)",
            < 0 => $"폴더 변경: 이미지 {-delta}개 제거 (총 {images.Count:N0}개)",
            _ => currentModified ? "현재 이미지가 변경되어 다시 읽었습니다." : $"폴더 변경 반영 (총 {images.Count:N0}개)",
        });
    }

    private static bool IsModified(ImageInfo before, ImageInfo after) =>
        before.Size != after.Size || before.ModifiedAt != after.ModifiedAt;

    /// <summary>바뀌지 않은 항목은 기존 썸네일을 재사용해 목록을 다시 만든다.</summary>
    private void RebuildThumbnails(IReadOnlyList<ImageInfo> images)
    {
        var existing = new Dictionary<string, ThumbnailItemViewModel>();
        foreach (var item in ThumbnailItems)
        {
            existing.TryAdd(item.Info.Key, item);
        }

        var items = new List<ThumbnailItemViewModel>(images.Count);
        for (var i = 0; i < images.Count; i++)
        {
            var info = images[i];
            if (existing.TryGetValue(info.Key, out var item) && !IsModified(item.Info, info))
            {
                item.Update(i, info);
                item.IsCurrent = false;
            }
            else
            {
                item = new ThumbnailItemViewModel(i, info, RequestThumbnail);
            }

            items.Add(item);
        }

        // 목록에서 빠진 항목은 썸네일 메모리 관리 대상에서도 뺀다.
        var kept = new HashSet<ThumbnailItemViewModel>(items);
        var node = _loadedThumbnails.First;
        while (node is not null)
        {
            var next = node.Next;
            if (!kept.Contains(node.Value))
            {
                _loadedThumbnails.Remove(node);
            }

            node = next;
        }

        // CurrentIndex 가 가리키던 이전 항목의 표시를 SetCurrent 가 지우지 않도록 먼저 목록을 바꾼다.
        CurrentIndex = -1;
        ThumbnailItems = new ObservableCollection<ThumbnailItemViewModel>(items);
    }

    #endregion
}
