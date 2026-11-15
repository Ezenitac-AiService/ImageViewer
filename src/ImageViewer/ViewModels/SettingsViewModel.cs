using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ImageViewer.Models;
using ImageViewer.Services;

namespace ImageViewer.ViewModels;

public sealed record OptionItem<T>(T Value, string Label);

/// <summary>설정 대화 상자. 확인을 누르면 <see cref="ApplyTo"/> 로 반영한다.</summary>
public sealed partial class SettingsViewModel : ObservableObject
{
    private readonly ShellIntegrationService _shell;
    private readonly IDialogService _dialogs;

    public SettingsViewModel(ViewerSettings settings, ShellIntegrationService shell, IDialogService dialogs)
    {
        _shell = shell;
        _dialogs = dialogs;

        _mouseWheel = settings.Display.MouseWheel;
        _background = settings.Display.Background;
        _wrapAround = settings.Navigation.WrapAround;
        _watchFolder = settings.Navigation.WatchFolder;
        _slideShowRepeat = settings.Slideshow.Repeat;
        _thumbnailVisible = settings.Thumbnail.Visible;
        _thumbnailSize = settings.Thumbnail.Size;
        _thumbnailScrollBar = settings.Thumbnail.ShowScrollBar;
        _smoothSigma = settings.Filter.SmoothSigma;
        _sharpenSigma = settings.Filter.SharpenSigma;
        _sharpenAmount = settings.Filter.SharpenAmount;
        _maxImageMegapixels = settings.Limits.MaxImageMegapixels;
        _isShellRegistered = shell.IsRegistered();
    }

    public IReadOnlyList<OptionItem<MouseWheelAction>> MouseWheelOptions { get; } =
    [
        new(MouseWheelAction.Navigate, "위: 이전 / 아래: 다음 이미지 (Ctrl+휠: 확대/축소)"),
        new(MouseWheelAction.Zoom, "확대 / 축소 (Ctrl+휠: 이전/다음 이미지)"),
    ];

    public IReadOnlyList<OptionItem<ViewerBackground>> BackgroundOptions { get; } =
    [
        new(ViewerBackground.Black, "검정"),
        new(ViewerBackground.DarkGray, "어두운 회색"),
        new(ViewerBackground.Gray, "회색"),
        new(ViewerBackground.White, "흰색"),
        new(ViewerBackground.Checkerboard, "체크무늬 (투명 영역 확인)"),
    ];

    [ObservableProperty] private MouseWheelAction _mouseWheel;
    [ObservableProperty] private ViewerBackground _background;
    [ObservableProperty] private bool _wrapAround;
    [ObservableProperty] private bool _watchFolder;
    [ObservableProperty] private bool _slideShowRepeat;
    [ObservableProperty] private bool _thumbnailVisible;
    [ObservableProperty] private int _thumbnailSize;
    [ObservableProperty] private bool _thumbnailScrollBar;
    [ObservableProperty] private double _smoothSigma;
    [ObservableProperty] private double _sharpenSigma;
    [ObservableProperty] private double _sharpenAmount;
    [ObservableProperty] private int _maxImageMegapixels;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(ShellStatusText))]
    private bool _isShellRegistered;

    public string ShellStatusText => IsShellRegistered
        ? "현재 사용자에 등록되어 있습니다."
        : "등록되어 있지 않습니다.";

    [RelayCommand]
    private void RegisterShell()
    {
        if (!_dialogs.Confirm(
                "폴더 및 ZIP 파일의 탐색기 우클릭 메뉴에 'ImageViewer에서 보기'를 추가합니다.\n" +
                "(현재 사용자 레지스트리 HKCU\\Software\\Classes 에 등록)\n\n계속하시겠습니까?"))
        {
            return;
        }

        try
        {
            _shell.Register();
        }
        catch (Exception ex)
        {
            _dialogs.ShowMessage("컨텍스트 메뉴를 등록하지 못했습니다.\n" + ex.Message, isError: true);
        }

        IsShellRegistered = _shell.IsRegistered();
    }

    [RelayCommand]
    private void UnregisterShell()
    {
        try
        {
            _shell.Unregister();
        }
        catch (Exception ex)
        {
            _dialogs.ShowMessage("컨텍스트 메뉴를 제거하지 못했습니다.\n" + ex.Message, isError: true);
        }

        IsShellRegistered = _shell.IsRegistered();
    }

    public void ApplyTo(ViewerSettings settings)
    {
        settings.Display.MouseWheel = MouseWheel;
        settings.Display.Background = Background;
        settings.Navigation.WrapAround = WrapAround;
        settings.Navigation.WatchFolder = WatchFolder;
        settings.Slideshow.Repeat = SlideShowRepeat;
        settings.Thumbnail.Visible = ThumbnailVisible;
        settings.Thumbnail.Size = ThumbnailSize;
        settings.Thumbnail.ShowScrollBar = ThumbnailScrollBar;
        settings.Filter.SmoothSigma = SmoothSigma;
        settings.Filter.SharpenSigma = SharpenSigma;
        settings.Filter.SharpenAmount = SharpenAmount;
        settings.Limits.MaxImageMegapixels = MaxImageMegapixels;
        settings.Normalize();
    }
}
