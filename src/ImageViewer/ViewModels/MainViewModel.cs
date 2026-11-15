using System.Collections.ObjectModel;
using System.Windows;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ImageViewer.Models;
using ImageViewer.Services;
using ImageViewer.Services.ImageSources;
using Microsoft.Extensions.Logging;

namespace ImageViewer.ViewModels;

/// <summary>표시 크기가 바뀌었음을 View 에 알린다. 새 이미지이면 이동(Pan) 위치를 초기화한다.</summary>
public sealed class LayoutChangedEventArgs(bool isNewImage) : EventArgs
{
    public bool IsNewImage { get; } = isNewImage;
}

public sealed partial class MainViewModel : ObservableObject, IDisposable
{
    private const int MaxLoadedThumbnails = 300;

    private readonly SettingsService _settingsService;
    private readonly IImageSourceFactory _sourceFactory;
    private readonly ImageLoaderService _loader;
    private readonly ImageRenderService _renderer;
    private readonly ThumbnailService _thumbnails;
    private readonly ImageNavigationService _navigation;
    private readonly SlideShowService _slideShow;
    private readonly ShellIntegrationService _shell;
    private readonly IDialogService _dialogs;
    private readonly ILogger<MainViewModel> _logger;
    private readonly Dispatcher _dispatcher;
    private readonly DispatcherTimer _renderDebounce;
    private readonly DispatcherTimer _osdTimer;
    private readonly DispatcherTimer _loadingIndicatorTimer;
    private readonly LinkedList<ThumbnailItemViewModel> _loadedThumbnails = new();

    private IImageSource? _source;
    private IReadOnlyList<ImageInfo> _images = [];
    private DecodedImage? _decoded;
    private string? _bitmapImageKey;
    private bool _pendingNewImage;
    private double _zoomFactor = 1.0;
    private double _viewportWidth;
    private double _viewportHeight;
    private double _dpiScale = 1.0;
    private int _lastDirection = 1;
    private int _loadVersion;
    private int _renderVersion;
    private int _openVersion;
    private CancellationTokenSource? _loadCts;
    private CancellationTokenSource? _renderCts;
    private CancellationTokenSource? _prefetchCts;
    private CancellationTokenSource? _openCts;

    public MainViewModel(
        SettingsService settingsService,
        IImageSourceFactory sourceFactory,
        ImageLoaderService loader,
        ImageRenderService renderer,
        ThumbnailService thumbnails,
        ImageNavigationService navigation,
        SlideShowService slideShow,
        ShellIntegrationService shell,
        IDialogService dialogs,
        ILogger<MainViewModel> logger)
    {
        _settingsService = settingsService;
        _sourceFactory = sourceFactory;
        _loader = loader;
        _renderer = renderer;
        _thumbnails = thumbnails;
        _navigation = navigation;
        _slideShow = slideShow;
        _shell = shell;
        _dialogs = dialogs;
        _logger = logger;
        _dispatcher = Dispatcher.CurrentDispatcher;

        _renderDebounce = new DispatcherTimer(DispatcherPriority.Background, _dispatcher)
        {
            Interval = TimeSpan.FromMilliseconds(120),
        };
        _renderDebounce.Tick += (_, _) =>
        {
            _renderDebounce.Stop();
            _ = RenderCurrentAsync();
        };

        _osdTimer = new DispatcherTimer(DispatcherPriority.Background, _dispatcher)
        {
            Interval = TimeSpan.FromSeconds(1.6),
        };
        _osdTimer.Tick += (_, _) =>
        {
            _osdTimer.Stop();
            OsdMessage = null;
        };

        // 짧은 로딩에는 표시하지 않아 깜박임을 줄인다.
        _loadingIndicatorTimer = new DispatcherTimer(DispatcherPriority.Background, _dispatcher)
        {
            Interval = TimeSpan.FromMilliseconds(250),
        };
        _loadingIndicatorTimer.Tick += (_, _) =>
        {
            _loadingIndicatorTimer.Stop();
            IsLoading = true;
        };

        _slideShow.Tick += OnSlideShowTick;
        ApplySettingsToServices();
    }

    /// <summary>표시 크기 변경 알림 (View 가 이동 위치를 조정).</summary>
    public event EventHandler<LayoutChangedEventArgs>? LayoutChanged;

    public ViewerSettings Settings => _settingsService.Current;

    #region 바인딩 속성

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(WindowTitle))]
    private ImageSourceInfo? _sourceInfo;

    [ObservableProperty]
    private ObservableCollection<ThumbnailItemViewModel> _thumbnailItems = [];

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(PositionText), nameof(ThumbnailSelectedIndex), nameof(HasImages))]
    private int _currentIndex = -1;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(WindowTitle))]
    private ImageInfo? _currentImage;

    [ObservableProperty]
    private BitmapSource? _currentBitmap;

    [ObservableProperty]
    private double _displayWidth;

    [ObservableProperty]
    private double _displayHeight;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(ScaleText))]
    private double _scale = 1.0;

    [ObservableProperty]
    private string? _errorMessage;

    [ObservableProperty]
    private string? _placeholderMessage = "폴더, ZIP 또는 이미지 파일을 열어 주세요.\n(파일 메뉴 · 드래그 앤 드롭 · 탐색기 우클릭 메뉴)";

    [ObservableProperty]
    private bool _isLoading;

    [ObservableProperty]
    private string? _osdMessage;

    [ObservableProperty]
    private bool _isFullScreen;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(SlideShowButtonText))]
    private bool _isSlideShowPlaying;

    public bool HasImages => _images.Count > 0;

    public string WindowTitle
    {
        get
        {
            if (CurrentImage is null)
            {
                return SourceInfo is null ? "ImageViewer" : $"{SourceInfo.DisplayName} - ImageViewer";
            }

            return SourceInfo is null
                ? $"{CurrentImage.Name} - ImageViewer"
                : $"{CurrentImage.Name} - {SourceInfo.DisplayName} - ImageViewer";
        }
    }

    public string PositionText
    {
        get
        {
            if (_images.Count == 0)
            {
                return "0 / 0";
            }

            var text = $"{CurrentIndex + 1:N0} / {_images.Count:N0}";
            return _decoded is { PageCount: > 1 } d ? $"{text}  ·  {d.PageIndex + 1}/{d.PageCount}쪽" : text;
        }
    }

    public string ScaleText => $"{Scale * 100:0}%";

    /// <summary>재생 / 일시정지 아이콘 글리프 (Segoe Fluent Icons).</summary>
    public string SlideShowButtonText => IsSlideShowPlaying ? PauseGlyph : PlayGlyph;

    private const string PlayGlyph = "\uE768";
    private const string PauseGlyph = "\uE769";

    /// <summary>썸네일 목록 선택 위치. 사용자가 선택하면 해당 이미지로 이동한다.</summary>
    public int ThumbnailSelectedIndex
    {
        get => CurrentIndex;
        set
        {
            if (value >= 0 && value != CurrentIndex)
            {
                _ = GoToAsync(value);
            }
        }
    }

    public DisplayMode DisplayMode
    {
        get => Settings.Display.Mode;
        set
        {
            if (Settings.Display.Mode == value)
            {
                return;
            }

            Settings.Display.Mode = value;
            OnPropertyChanged();
            _zoomFactor = 1.0;
            ShowOsd(GetModeName(value));
            UpdateLayoutNow(renderImmediately: true);
        }
    }

    public ImageFilter Filter
    {
        get => Settings.Display.Filter;
        set
        {
            if (Settings.Display.Filter == value)
            {
                return;
            }

            Settings.Display.Filter = value;
            OnPropertyChanged();
            ShowOsd("필터: " + ImageFilterService.GetDisplayName(value));
            _ = RenderCurrentAsync();
        }
    }

    public int SlideShowInterval
    {
        get => Settings.Slideshow.IntervalSeconds;
        set
        {
            var v = SlideShowSettings.ClampInterval(value);
            if (Settings.Slideshow.IntervalSeconds == v)
            {
                return;
            }

            Settings.Slideshow.IntervalSeconds = v;
            _slideShow.IntervalSeconds = v;
            OnPropertyChanged();
            OnPropertyChanged(nameof(SlideShowIntervalText));
        }
    }

    public string SlideShowIntervalText => $"{SlideShowInterval}초";

    public SlideShowMode SlideShowMode
    {
        get => Settings.Slideshow.Mode;
        set
        {
            if (Settings.Slideshow.Mode == value)
            {
                return;
            }

            Settings.Slideshow.Mode = value;
            _slideShow.Mode = value;
            OnPropertyChanged();
            ShowOsd(value == SlideShowMode.Random ? "랜덤 재생" : "순차 재생");
            if (IsSlideShowPlaying)
            {
                StartPrefetch(); // 재생 순서가 바뀌었으므로 다음 이미지를 다시 미리 읽는다
            }
        }
    }

    public bool SlideShowRepeat
    {
        get => Settings.Slideshow.Repeat;
        set
        {
            if (Settings.Slideshow.Repeat == value)
            {
                return;
            }

            Settings.Slideshow.Repeat = value;
            _slideShow.Repeat = value;
            OnPropertyChanged();
            if (IsSlideShowPlaying)
            {
                StartPrefetch();
            }
        }
    }

    public bool ThumbnailsVisible
    {
        get => Settings.Thumbnail.Visible;
        set
        {
            if (Settings.Thumbnail.Visible == value)
            {
                return;
            }

            Settings.Thumbnail.Visible = value;
            OnPropertyChanged();
        }
    }

    public double ThumbnailSize => Settings.Thumbnail.Size;

    public ViewerBackground Background => Settings.Display.Background;

    /// <summary>상단 바 고정 여부. 해제하면 마우스를 위쪽에 올렸을 때만 나타난다.</summary>
    public bool TopBarPinned
    {
        get => Settings.Display.TopBarPinned;
        set
        {
            if (Settings.Display.TopBarPinned == value)
            {
                return;
            }

            Settings.Display.TopBarPinned = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(BothBarsPinned));
        }
    }

    /// <summary>상단 · 하단 바가 모두 고정된 상태 (` 키 메뉴 항목 표시용).</summary>
    public bool BothBarsPinned => TopBarPinned && BottomBarPinned;

    /// <summary>하단 바 고정 여부. 해제하면 마우스를 올렸을 때만 나타난다.</summary>
    public bool BottomBarPinned
    {
        get => Settings.Display.BottomBarPinned;
        set
        {
            if (Settings.Display.BottomBarPinned == value)
            {
                return;
            }

            Settings.Display.BottomBarPinned = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(BothBarsPinned));
        }
    }

    public MouseWheelAction MouseWheelAction => Settings.Display.MouseWheel;

    #endregion

    #region 열기

    [RelayCommand]
    private async Task OpenFolderAsync()
    {
        var path = _dialogs.PickFolder();
        if (path is not null)
        {
            await OpenPathAsync(path);
        }
    }

    [RelayCommand]
    private async Task OpenFileAsync()
    {
        var path = _dialogs.PickFile();
        if (path is not null)
        {
            await OpenPathAsync(path);
        }
    }

    /// <summary>폴더, ZIP 또는 이미지 파일 경로를 연다.</summary>
    public async Task OpenPathAsync(string path)
    {
        var version = ++_openVersion;
        _openCts?.Cancel();
        _openCts = new CancellationTokenSource();
        var ct = _openCts.Token;

        ImageSourceOpenResult result;
        IReadOnlyList<ImageInfo> images;
        try
        {
            result = _sourceFactory.Create(path);
        }
        catch (ImageSourceException ex)
        {
            _dialogs.ShowMessage(ex.Message, isError: true);
            return;
        }

        try
        {
            images = await result.Source.GetImagesAsync(ct);
        }
        catch (OperationCanceledException)
        {
            result.Source.Dispose();
            return;
        }
        catch (Exception ex)
        {
            result.Source.Dispose();
            _logger.LogWarning(ex, "이미지 소스를 열지 못했습니다: {Name}", Path.GetFileName(path));
            _dialogs.ShowMessage(ex is ImageSourceException ? ex.Message : "이미지 목록을 읽지 못했습니다.", isError: true);
            return;
        }

        if (version != _openVersion)
        {
            result.Source.Dispose();
            return;
        }

        var initialIndex = 0;
        if (result.InitialFileName is { } fileName)
        {
            var found = images.ToList().FindIndex(i => string.Equals(i.Name, fileName, StringComparison.OrdinalIgnoreCase));
            initialIndex = Math.Max(0, found);
        }

        SetSource(result.Source, images, initialIndex);
    }

    private void SetSource(IImageSource source, IReadOnlyList<ImageInfo> images, int initialIndex)
    {
        StopSlideShow(showMessage: false);
        CancelPending();
        _thumbnails.CancelAll();
        _loadedThumbnails.Clear();

        var old = _source;
        DetachWatcher(old);
        _source = source;
        _images = images;
        SourceInfo = source.Info;
        _navigation.Reset(images.Count, initialIndex);
        _slideShow.ResetOrder();
        _loader.ClearCache();
        _renderer.ClearCache();
        ReplaceDecoded(null);

        // 이전 소스는 진행 중인 백그라운드 작업이 끝난 뒤 해제한다.
        if (old is not null)
        {
            _ = Task.Run(old.Dispose);
        }

        ThumbnailItems = new ObservableCollection<ThumbnailItemViewModel>(
            images.Select((info, i) => new ThumbnailItemViewModel(i, info, RequestThumbnail)));

        _logger.LogInformation("{Type} 열기: 이미지 {Count}개", source.Info.Label, images.Count);
        AttachWatcher(source);

        if (images.Count == 0)
        {
            CurrentIndex = -1;
            CurrentImage = null;
            CurrentBitmap = null;
            ErrorMessage = null;
            InfoItems = [];
            FrameInfoText = null;
            PlaceholderMessage = $"이 {source.Info.Label}에는 표시할 이미지가 없습니다.";
            OnPropertyChanged(nameof(PositionText));
            return;
        }

        PlaceholderMessage = null;
        _lastDirection = 1;
        SetCurrent(_navigation.CurrentIndex);
        _ = LoadCurrentAsync();
    }

    #endregion

    #region 탐색

    [RelayCommand]
    private Task NextAsync() => NavigateAsync(_navigation.GetNextIndex(), +1, "마지막 이미지입니다.");

    [RelayCommand]
    private Task PreviousAsync() => NavigateAsync(_navigation.GetPreviousIndex(), -1, "첫 번째 이미지입니다.");

    [RelayCommand]
    private Task FirstAsync() => NavigateAsync(_navigation.GetFirstIndex(), +1, null);

    [RelayCommand]
    private Task LastAsync() => NavigateAsync(_navigation.GetLastIndex(), -1, null);

    public Task GoToAsync(int index) =>
        NavigateAsync(index, index >= CurrentIndex ? +1 : -1, null);

    private Task NavigateAsync(int index, int direction, string? edgeMessage, bool fromSlideShow = false)
    {
        if (_images.Count == 0)
        {
            return Task.CompletedTask;
        }

        if (index < 0)
        {
            if (edgeMessage is not null)
            {
                ShowOsd(edgeMessage);
            }

            return Task.CompletedTask;
        }

        if (!fromSlideShow)
        {
            // 정책: 슬라이드쇼 중 수동으로 이동하면 타이머를 다시 시작한다.
            _slideShow.RestartTimer();
        }

        if (index == CurrentIndex && _decoded is not null)
        {
            return Task.CompletedTask;
        }

        _lastDirection = direction;
        _navigation.MoveTo(index);
        SetCurrent(index);
        return LoadCurrentAsync();
    }

    private void SetCurrent(int index)
    {
        if (CurrentIndex >= 0 && CurrentIndex < ThumbnailItems.Count)
        {
            ThumbnailItems[CurrentIndex].IsCurrent = false;
        }

        CurrentIndex = index;
        CurrentImage = index >= 0 && index < _images.Count ? _images[index] : null;

        if (index >= 0 && index < ThumbnailItems.Count)
        {
            ThumbnailItems[index].IsCurrent = true;
        }
    }

    #endregion

    #region 이미지 로딩 및 렌더링

    /// <summary>현재 이미지를 읽는다. 다중 페이지면 <paramref name="pageIndex"/> 페이지를 읽는다.</summary>
    /// <param name="resetTransform">다른 이미지로 이동할 때 회전 · 반전을 초기화한다 (페이지 이동은 유지).</param>
    private async Task LoadCurrentAsync(int pageIndex = 0, bool resetTransform = true)
    {
        var source = _source;
        var info = CurrentImage;
        if (source is null || info is null)
        {
            return;
        }

        var version = ++_loadVersion;
        _loadCts?.Cancel();
        _prefetchCts?.Cancel();
        _loadCts = new CancellationTokenSource();
        var ct = _loadCts.Token;

        _loadingIndicatorTimer.Stop();
        _loadingIndicatorTimer.Start();

        try
        {
            var decoded = await _loader.LoadAsync(source, info, pageIndex, CurrentDecodeTarget(), ct);
            if (version != _loadVersion)
            {
                decoded.Release();
                return;
            }

            ReplaceDecoded(decoded);
            StopAnimation();
            if (resetTransform)
            {
                _transform = ImageTransform.Identity;
            }

            _zoomFactor = 1.0;
            _pendingNewImage = true;
            UpdateImageInfo(info, decoded);
            OnPropertyChanged(nameof(PositionText));
            await RenderCurrentAsync();
            if (version == _loadVersion)
            {
                StartPrefetch();
            }
        }
        catch (OperationCanceledException)
        {
        }
        catch (Exception ex)
        {
            if (version != _loadVersion)
            {
                return;
            }

            if (ex is ImageLoadException)
            {
                _logger.LogInformation("이미지 표시 실패: {Name} - {Message}", info.Name, ex.Message);
            }
            else
            {
                _logger.LogError(ex, "이미지 표시 중 예외: {Name}", info.Name);
            }

            ShowError(info, ex is ImageLoadException ? ex.Message : "이미지를 표시할 수 없습니다.");
        }
        finally
        {
            if (version == _loadVersion)
            {
                _loadingIndicatorTimer.Stop();
                IsLoading = false;
            }
        }
    }

    private void ShowError(ImageInfo info, string message)
    {
        StopAnimation();
        FrameInfoText = null;
        ReplaceDecoded(null);
        UpdateImageInfo(info, null);
        OnPropertyChanged(nameof(PositionText));
        CurrentBitmap = null;
        _bitmapImageKey = null;
        ErrorMessage = $"{info.Name}\n\n{message}";
        Scale = 1.0;

        if (IsSlideShowPlaying)
        {
            // 슬라이드쇼 중에는 오류 이미지를 오래 보여 주지 않는다.
            _slideShow.RestartTimer();
        }
    }

    /// <summary>현재 이미지를 현재 배율·필터로 다시 렌더링한다. 마지막 요청의 결과만 화면에 반영한다.</summary>
    private async Task RenderCurrentAsync()
    {
        var decoded = _decoded;
        if (decoded is null || _viewportWidth <= 0 || _viewportHeight <= 0)
        {
            return;
        }

        var loadVersion = _loadVersion;
        var version = ++_renderVersion;
        _renderCts?.Cancel();
        _renderCts = new CancellationTokenSource();
        var ct = _renderCts.Token;

        var layout = ComputeLayout(decoded);
        var transform = _transform;
        try
        {
            // 애니메이션: 프레임을 원본 해상도로 만들어 재생한다 (확대 · 축소는 화면에서 처리).
            if (decoded.IsAnimated && PlayAnimations)
            {
                var frames = await _renderer.RenderAnimationAsync(decoded, transform, ct);
                if (version != _renderVersion || loadVersion != _loadVersion || !ReferenceEquals(decoded, _decoded))
                {
                    return;
                }

                StartAnimation(frames);
                _bitmapImageKey = decoded.Key;
                ErrorMessage = null;
                ApplyLayout(ComputeLayout(decoded));
                return;
            }

            // 화면 크기로 줄여 읽은 이미지로는 해상도가 부족하면(원본 크기 · 확대) 원본을 백그라운드에서 읽는다.
            RequestFullResolutionIfNeeded(decoded, layout, transform);

            var bitmap = await _renderer.RenderAsync(decoded, layout.RenderPixelWidth, layout.RenderPixelHeight, Filter, transform, ct);
            if (version != _renderVersion || loadVersion != _loadVersion || !ReferenceEquals(decoded, _decoded))
            {
                return;
            }

            // 렌더링 중에 배율이 바뀌었으면 최신 크기를 적용하고 다시 렌더링을 예약한다.
            var latest = ComputeLayout(decoded);
            StopAnimation();
            CurrentBitmap = bitmap;
            FrameInfoText = BuildStaticFrameInfo(decoded);
            _bitmapImageKey = decoded.Key;
            ErrorMessage = null;
            ApplyLayout(latest);
            if (latest.RenderPixelWidth != layout.RenderPixelWidth || latest.RenderPixelHeight != layout.RenderPixelHeight)
            {
                ScheduleRender();
            }
        }
        catch (OperationCanceledException)
        {
        }
        catch (ObjectDisposedException)
        {
            // 캐시에서 해제된 경우 다시 읽는다.
            if (loadVersion == _loadVersion && ReferenceEquals(decoded, _decoded))
            {
                ReplaceDecoded(null);
                _ = LoadCurrentAsync();
            }
        }
        catch (Exception ex)
        {
            if (version == _renderVersion && CurrentImage is { } info)
            {
                _logger.LogError(ex, "렌더링 실패: {Name}", info.Name);
                ShowError(info, "이미지를 표시용으로 변환하지 못했습니다.");
            }
        }
    }

    /// <summary>회전 · 반전을 적용한 화면 기준 원본 이미지 크기 (축소 디코딩과 관계없이 원본 기준).</summary>
    private (uint Width, uint Height) DisplayedSize(DecodedImage decoded) =>
        _transform.Apply(decoded.OriginalWidth, decoded.OriginalHeight);

    private DisplayLayout ComputeLayout(DecodedImage decoded)
    {
        var (width, height) = DisplayedSize(decoded);
        return ImageResizeService.Calculate(
            DisplayMode,
            _zoomFactor,
            width,
            height,
            _viewportWidth * _dpiScale,
            _viewportHeight * _dpiScale);
    }

    private void ApplyLayout(DisplayLayout layout)
    {
        DisplayWidth = layout.DisplayPixelWidth / _dpiScale;
        DisplayHeight = layout.DisplayPixelHeight / _dpiScale;
        Scale = layout.Scale;

        var isNew = _pendingNewImage;
        _pendingNewImage = false;
        LayoutChanged?.Invoke(this, new LayoutChangedEventArgs(isNew));
    }

    /// <summary>
    /// 배율이나 표시 영역이 바뀌면 현재 비트맵을 즉시 늘리거나 줄여 보여 주고,
    /// 정확한 크기의 렌더링은 잠시 뒤에 수행한다.
    /// </summary>
    private void UpdateLayoutNow(bool renderImmediately)
    {
        var decoded = _decoded;
        if (decoded is null)
        {
            return;
        }

        if (_bitmapImageKey == decoded.Key && !_pendingNewImage)
        {
            ApplyLayout(ComputeLayout(decoded));
        }

        if (renderImmediately)
        {
            _renderDebounce.Stop();
            _ = RenderCurrentAsync();
        }
        else
        {
            ScheduleRender();
        }
    }

    private void ScheduleRender()
    {
        _renderDebounce.Stop();
        _renderDebounce.Start();
    }

    /// <summary>View 의 표시 영역 크기(DIP)와 DPI 배율을 전달받는다.</summary>
    public void UpdateViewport(double width, double height, double dpiScale)
    {
        if (width <= 0 || height <= 0)
        {
            return;
        }

        var first = _viewportWidth <= 0;
        if (Math.Abs(width - _viewportWidth) < 0.5 && Math.Abs(height - _viewportHeight) < 0.5 &&
            Math.Abs(dpiScale - _dpiScale) < 0.001)
        {
            return;
        }

        _viewportWidth = width;
        _viewportHeight = height;
        _dpiScale = dpiScale <= 0 ? 1.0 : dpiScale;
        UpdateLayoutNow(renderImmediately: first);
    }

    private void ReplaceDecoded(DecodedImage? decoded)
    {
        var old = _decoded;
        _decoded = decoded;
        old?.Release();
    }

    private void CancelPending()
    {
        StopAnimation();
        _loadVersion++;
        _renderVersion++;
        _loadCts?.Cancel();
        _renderCts?.Cancel();
        _prefetchCts?.Cancel();
        _renderDebounce.Stop();
        _loadingIndicatorTimer.Stop();
        IsLoading = false;
    }

    #endregion

    #region 썸네일

    private void RequestThumbnail(ThumbnailItemViewModel item)
    {
        var source = _source;
        if (source is null)
        {
            return;
        }

        var pixelSize = (uint)Math.Clamp(Math.Round(Settings.Thumbnail.Size * _dpiScale), 32, 512);
        _thumbnails.Request(source, item.Info, pixelSize, (bitmap, completed) =>
        {
            _dispatcher.BeginInvoke(DispatcherPriority.Background, () =>
            {
                if (!ReferenceEquals(source, _source))
                {
                    return;
                }

                if (!completed)
                {
                    item.ResetRequest();
                    return;
                }

                item.SetThumbnail(bitmap);
                if (bitmap is null)
                {
                    return;
                }

                _loadedThumbnails.AddLast(item);
                while (_loadedThumbnails.Count > MaxLoadedThumbnails && _loadedThumbnails.First is { } oldest)
                {
                    _loadedThumbnails.RemoveFirst();
                    oldest.Value.ReleaseThumbnail();
                }
            });
        });
    }

    #endregion

    #region 표시 모드 · 확대 · 필터

    [RelayCommand]
    private void SetDisplayMode(DisplayMode mode)
    {
        if (DisplayMode == mode)
        {
            // 같은 모드를 다시 선택하면 확대 배율만 초기화한다.
            _zoomFactor = 1.0;
            ShowOsd(GetModeName(mode));
            UpdateLayoutNow(renderImmediately: true);
            return;
        }

        DisplayMode = mode;
    }

    [RelayCommand]
    private void SetFilter(ImageFilter filter) => Filter = filter;

    [RelayCommand]
    private void ZoomIn() => Zoom(true);

    [RelayCommand]
    private void ZoomOut() => Zoom(false);

    private void Zoom(bool zoomIn)
    {
        var decoded = _decoded;
        if (decoded is null)
        {
            return;
        }

        var (displayedWidth, displayedHeight) = DisplayedSize(decoded);
        var baseScale = ImageResizeService.GetBaseScale(
            DisplayMode, displayedWidth, displayedHeight,
            _viewportWidth * _dpiScale, _viewportHeight * _dpiScale);
        _zoomFactor = ImageResizeService.StepZoom(baseScale, _zoomFactor, zoomIn);
        UpdateLayoutNow(renderImmediately: false);
        ShowOsd(ScaleText);
    }

    [RelayCommand]
    private void ResetZoom()
    {
        _zoomFactor = 1.0;
        UpdateLayoutNow(renderImmediately: true);
        ShowOsd(ScaleText);
    }

    private static string GetModeName(DisplayMode mode) => mode switch
    {
        DisplayMode.Original => "원본 크기",
        DisplayMode.FitWidth => "폭 맞춤",
        DisplayMode.FitHeight => "높이 맞춤",
        DisplayMode.Fill => "꽉 차게 보기",
        _ => mode.ToString(),
    };

    #endregion

    #region 슬라이드쇼

    [RelayCommand]
    private void ToggleSlideShow()
    {
        if (IsSlideShowPlaying)
        {
            StopSlideShow(showMessage: true);
        }
        else
        {
            StartSlideShow();
        }
    }

    [RelayCommand]
    private void SetSlideShowMode(SlideShowMode mode) => SlideShowMode = mode;

    private void StartSlideShow()
    {
        if (_images.Count == 0)
        {
            ShowOsd("표시할 이미지가 없습니다.");
            return;
        }

        _slideShow.Start();
        IsSlideShowPlaying = true;
        ShowOsd($"슬라이드쇼 재생 ({SlideShowInterval}초, {(SlideShowMode == SlideShowMode.Random ? "랜덤" : "순차")})");

        // 재생 순서(랜덤이면 섞은 목록)를 바로 만들고 첫 번째 다음 이미지와 좌우 이미지를 미리 읽는다.
        StartPrefetch();
    }

    private void StopSlideShow(bool showMessage)
    {
        if (!IsSlideShowPlaying)
        {
            return;
        }

        _slideShow.Stop();
        IsSlideShowPlaying = false;
        if (showMessage)
        {
            ShowOsd("슬라이드쇼 일시정지");
        }
    }

    private void OnSlideShowTick(object? sender, EventArgs e)
    {
        var next = _slideShow.GetNextIndex(CurrentIndex, _images.Count);
        if (next is null)
        {
            StopSlideShow(showMessage: false);
            ShowOsd("슬라이드쇼가 끝났습니다.");
            return;
        }

        _ = NavigateAsync(next.Value, +1, null, fromSlideShow: true);
    }

    #endregion

    #region 창 · 대화 상자

    [RelayCommand]
    private void ToggleFullScreen() => IsFullScreen = !IsFullScreen;

    /// <summary>
    /// ` 키: 상단 · 하단 바의 고정(Pin)을 한꺼번에 전환한다.
    /// 하나라도 고정되어 있으면 모두 해제(숨김)하고, 모두 해제되어 있으면 모두 고정(표시)한다.
    /// </summary>
    [RelayCommand]
    private void ToggleMenus()
    {
        var pin = !(TopBarPinned || BottomBarPinned);
        TopBarPinned = pin;
        BottomBarPinned = pin;
        ShowOsd(pin
            ? "상단 · 하단 바 고정"
            : "상단 · 하단 바 고정 해제 (마우스를 위 / 아래 가장자리에 올리면 표시)");
    }

    [RelayCommand]
    private void ToggleTopBarPin() => TopBarPinned = !TopBarPinned;

    [RelayCommand]
    private void ToggleBottomBarPin() => BottomBarPinned = !BottomBarPinned;

    [RelayCommand]
    private void ToggleThumbnails() => ThumbnailsVisible = !ThumbnailsVisible;

    [RelayCommand]
    private async Task ShowImageListAsync()
    {
        if (_images.Count == 0)
        {
            ShowOsd("표시할 이미지가 없습니다.");
            return;
        }

        var vm = new ImageListViewModel(_images, CurrentIndex, SourceInfo, ThumbnailItems);
        if (_dialogs.ShowImageList(vm) && vm.ResultIndex is { } index)
        {
            await GoToAsync(index);
        }
    }

    [RelayCommand]
    private void ShowSettings()
    {
        var vm = new SettingsViewModel(Settings, _shell, _dialogs);
        if (!_dialogs.ShowSettings(vm))
        {
            return;
        }

        var thumbSizeChanged = vm.ThumbnailSize != Settings.Thumbnail.Size;
        var limitsChanged = vm.MaxImageMegapixels != Settings.Limits.MaxImageMegapixels;
        vm.ApplyTo(Settings);
        _settingsService.Save();
        ApplySettingsToServices();
        RefreshWatcher();

        OnPropertyChanged(nameof(SlideShowRepeat));
        OnPropertyChanged(nameof(ThumbnailsVisible));
        OnPropertyChanged(nameof(ThumbnailSize));
        OnPropertyChanged(nameof(Background));
        OnPropertyChanged(nameof(MouseWheelAction));

        _renderer.ClearCache();
        if (limitsChanged && _decoded is null && CurrentImage is not null)
        {
            _ = LoadCurrentAsync();
        }
        else
        {
            _zoomFactor = 1.0;
            UpdateLayoutNow(renderImmediately: true);
        }

        if (thumbSizeChanged && _source is not null)
        {
            _thumbnails.CancelAll();
            _loadedThumbnails.Clear();
            ThumbnailItems = new ObservableCollection<ThumbnailItemViewModel>(
                _images.Select((info, i) => new ThumbnailItemViewModel(i, info, RequestThumbnail) { IsCurrent = i == CurrentIndex }));
        }
    }

    [RelayCommand]
    private void Exit() => Application.Current?.MainWindow?.Close();

    private void ApplySettingsToServices()
    {
        _navigation.WrapAround = Settings.Navigation.WrapAround;
        _slideShow.IntervalSeconds = Settings.Slideshow.IntervalSeconds;
        _slideShow.Mode = Settings.Slideshow.Mode;
        _slideShow.Repeat = Settings.Slideshow.Repeat;
    }

    public void ShowOsd(string message)
    {
        OsdMessage = message;
        _osdTimer.Stop();
        _osdTimer.Start();
    }

    #endregion

    /// <summary>종료 시 설정을 저장한다.</summary>
    public void SaveSettings() => _settingsService.Save();

    public void Dispose()
    {
        DetachWatcher(_source);
        CancelPending();
        _slideShow.Dispose();
        _thumbnails.CancelAll();
        ReplaceDecoded(null);
        _loader.Dispose();
        _renderer.ClearCache();
        _source?.Dispose();
        _source = null;
    }
}
