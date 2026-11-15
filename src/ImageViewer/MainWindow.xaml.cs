using System.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Threading;
using ImageViewer.Helpers;
using ImageViewer.Models;
using ImageViewer.ViewModels;

namespace ImageViewer;

public partial class MainWindow : Window
{
    private readonly MainViewModel _vm;

    // 이미지 좌상단 위치 (표시 영역 기준, DIP)
    private double _offsetX;
    private double _offsetY;

    // 마지막으로 배치에 사용한 크기 (크기 변경 시 화면 중심을 유지하기 위해 보관)
    private double _lastImageWidth;
    private double _lastImageHeight;
    private double _lastViewportWidth;
    private double _lastViewportHeight;

    private Point? _zoomAnchor;
    private Point _dragStart;
    private double _dragStartOffsetX;
    private double _dragStartOffsetY;
    private bool _isDragging;

    // 전체 화면 전환 전 상태
    private WindowState _restoreState;
    private WindowStyle _restoreStyle;
    private ResizeMode _restoreResizeMode;
    private Rect _restoreBounds;

    // 고정 해제된 상단 · 하단 바를 불러내는 영역 = 각 바가 차지하는 높이 (DIP).
    // 숨겨진(Collapsed) 바는 높이가 0 이므로 마지막으로 보였을 때의 높이를 기억해 둔다.
    private const double MinRevealZone = 24;
    private double _topBarExtent = 70;     // 타이틀바 32 + 메뉴바 38
    private double _bottomBarExtent = 170; // 컨트롤 줄 + 썸네일 + 아래 여백
    private bool _menuHover;
    private bool _barHover;

    public MainWindow(MainViewModel viewModel)
    {
        InitializeComponent();
        _vm = viewModel;
        DataContext = viewModel;

        _vm.LayoutChanged += OnLayoutChanged;
        _vm.PropertyChanged += OnViewModelPropertyChanged;

        // 바 높이가 바뀌면(썸네일 표시 전환 등) 마우스 오버 영역과 프레임 정보 위치를 다시 맞춘다.
        TopBar.SizeChanged += (_, _) =>
        {
            if (TopBar.ActualHeight > 0)
            {
                _topBarExtent = TopBar.ActualHeight;
            }
        };
        BottomBar.SizeChanged += (_, _) =>
        {
            if (BottomBar.ActualHeight > 0)
            {
                _bottomBarExtent = BottomBar.ActualHeight + BottomBar.Margin.Bottom;
            }

            UpdateChrome(allowHide: false);
        };

        RestoreWindowPlacement();
        Loaded += (_, _) =>
        {
            UpdateChrome(allowHide: true);
            Viewport.Focus();
            UpdateViewport();
        };
    }

    public MainViewModel ViewModel => _vm;

    #region 창 위치 저장 · 복원

    private void RestoreWindowPlacement()
    {
        var w = _vm.Settings.Window;
        Width = w.Width;
        Height = w.Height;

        if (w.Left is { } left && w.Top is { } top && IsOnScreen(left, top, w.Width, w.Height))
        {
            WindowStartupLocation = WindowStartupLocation.Manual;
            Left = left;
            Top = top;
        }
        else
        {
            WindowStartupLocation = WindowStartupLocation.CenterScreen;
        }

        if (w.IsMaximized)
        {
            WindowState = WindowState.Maximized;
        }
    }

    /// <summary>저장된 위치가 현재 연결된 모니터 영역 안에 일부라도 보이는지 확인한다.</summary>
    private static bool IsOnScreen(double left, double top, double width, double height)
    {
        var virtualScreen = new Rect(
            SystemParameters.VirtualScreenLeft,
            SystemParameters.VirtualScreenTop,
            SystemParameters.VirtualScreenWidth,
            SystemParameters.VirtualScreenHeight);
        var titleBar = new Rect(left + 40, top, Math.Max(1, width - 80), 30);
        return virtualScreen.IntersectsWith(titleBar);
    }

    private void SaveWindowPlacement()
    {
        var w = _vm.Settings.Window;
        var bounds = _vm.IsFullScreen ? _restoreBounds : RestoreBounds;
        var state = _vm.IsFullScreen ? _restoreState : WindowState;

        if (!bounds.IsEmpty && bounds.Width > 0 && bounds.Height > 0)
        {
            w.Left = bounds.Left;
            w.Top = bounds.Top;
            w.Width = bounds.Width;
            w.Height = bounds.Height;
        }

        w.IsMaximized = state == WindowState.Maximized;
    }

    private void OnClosing(object? sender, CancelEventArgs e)
    {
        SaveWindowPlacement();
        _vm.SaveSettings();
        _vm.Dispose();
    }

    #endregion

    #region 전체 화면

    private void OnViewModelPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        switch (e.PropertyName)
        {
            case nameof(MainViewModel.IsFullScreen):
                ApplyFullScreen(_vm.IsFullScreen);
                UpdateChrome(allowHide: true);
                break;
            case nameof(MainViewModel.TopBarPinned):
            case nameof(MainViewModel.BottomBarPinned):
                UpdateChrome(allowHide: true);
                break;
            case nameof(MainViewModel.CurrentIndex):
                ScrollThumbnailIntoView();
                break;
            case nameof(MainViewModel.ThumbnailItems):
                Dispatcher.BeginInvoke(ScrollThumbnailIntoView, DispatcherPriority.Loaded);
                break;
        }
    }

    /// <summary>
    /// 창 테두리 없는 전체 화면. WindowStyle 을 None 으로 바꾼 뒤 최대화하면
    /// 현재 창이 있는 모니터 전체(작업 표시줄 포함)를 덮는다.
    /// </summary>
    private void ApplyFullScreen(bool fullScreen)
    {
        if (fullScreen)
        {
            _restoreState = WindowState;
            _restoreStyle = WindowStyle;
            _restoreResizeMode = ResizeMode;
            _restoreBounds = WindowState == WindowState.Normal ? new Rect(Left, Top, Width, Height) : RestoreBounds;

            // 최대화 상태에서 바로 스타일을 바꾸면 작업 표시줄을 덮지 않으므로 먼저 일반 상태로 돌린다.
            if (WindowState == WindowState.Maximized)
            {
                WindowState = WindowState.Normal;
            }

            WindowStyle = WindowStyle.None;
            ResizeMode = ResizeMode.NoResize;
            WindowState = WindowState.Maximized;
        }
        else
        {
            WindowState = WindowState.Normal;
            WindowStyle = _restoreStyle;
            ResizeMode = _restoreResizeMode;

            if (!_restoreBounds.IsEmpty)
            {
                Left = _restoreBounds.Left;
                Top = _restoreBounds.Top;
                Width = _restoreBounds.Width;
                Height = _restoreBounds.Height;
            }

            WindowState = _restoreState == WindowState.Maximized ? WindowState.Maximized : WindowState.Normal;
        }

        Viewport.Focus();
    }

    #endregion

    #region 상단 메뉴 · 하단 바 표시

    /// <summary>
    /// 상단 바(타이틀바 + 메뉴바)와 하단 바는 모두 이미지 위에 겹쳐 표시한다.
    /// 고정(Pin)된 바는 ` 키 · 전체 화면 · 마우스 위치와 관계없이 항상 표시한다.
    /// 고정을 해제한 바는 마우스를 위쪽 / 아래쪽 가장자리에 올렸을 때만 나타난다.
    /// </summary>

    private bool IsMenuInUse =>
        TopMenu.IsKeyboardFocusWithin ||
        TopMenu.Items.OfType<MenuItem>().Any(m => m.IsSubmenuOpen);

    private void UpdateChrome(bool allowHide)
    {
        var menuVisible = _vm.TopBarPinned || _menuHover || IsMenuInUse || TopBar.IsMouseOver;
        var barVisible = _vm.BottomBarPinned || _barHover || BottomBar.IsKeyboardFocusWithin;

        if (menuVisible)
        {
            TopBar.Visibility = Visibility.Visible;
        }
        else if (allowHide)
        {
            TopBar.Visibility = Visibility.Collapsed;
        }

        if (barVisible)
        {
            BottomBar.Visibility = Visibility.Visible;
        }
        else if (allowHide)
        {
            BottomBar.Visibility = Visibility.Collapsed;
        }

        BottomBar.MaxWidth = Math.Max(200, RootGrid.ActualWidth - 32);

        // 프레임 정보는 하단 바가 보이면 그 위에 둔다.
        var frameBottom = BottomBar.Visibility == Visibility.Visible && BottomBar.ActualHeight > 0
            ? BottomBar.ActualHeight + BottomBar.Margin.Bottom + 10
            : 14;
        if (Math.Abs(FrameInfo.Margin.Bottom - frameBottom) > 0.5)
        {
            FrameInfo.Margin = new Thickness(14, 0, 0, frameBottom);
        }

        // 정보 패널은 고정된 상단 바 아래에 둔다.
        var infoTop = (_vm.TopBarPinned ? TopBar.ActualHeight : 0) + 12;
        if (Math.Abs(InfoPanel.Margin.Top - infoTop) > 0.5)
        {
            InfoPanel.Margin = new Thickness(12, infoTop, 0, 0);
        }
    }

    private void OnWindowPreviewMouseMove(object sender, MouseEventArgs e)
    {
        if (_isDragging)
        {
            return;
        }

        var p = e.GetPosition(RootGrid);
        var height = RootGrid.ActualHeight;

        // 상단 바 높이만큼의 위쪽 영역 / 하단 바 높이만큼의 아래쪽 영역에 마우스가 들어오면 바로 나타나고,
        // 그 영역을 벗어나면 바로 숨긴다.
        _menuHover = p.Y <= Math.Max(MinRevealZone, _topBarExtent);
        _barHover = p.Y >= height - Math.Max(MinRevealZone, _bottomBarExtent);
        UpdateChrome(allowHide: true);
    }

    private void OnWindowMouseLeave(object sender, MouseEventArgs e)
    {
        _menuHover = false;
        _barHover = false;
        UpdateChrome(allowHide: true);
    }

    #endregion

    #region 직접 그린 타이틀바

    private void OnTopBarMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        // 메뉴 항목·버튼은 클릭을 직접 처리하므로 여기에는 빈 영역 클릭만 들어온다.
        e.Handled = true;
        if (e.ClickCount == 2)
        {
            ToggleMaximize();
            return;
        }

        if (_vm.IsFullScreen)
        {
            return;
        }

        try
        {
            DragMove();
        }
        catch (InvalidOperationException)
        {
        }
    }

    /// <summary>타이틀바 우클릭: Windows 시스템 메뉴 (이동 · 크기 조정 · 최소화 등).</summary>
    private void OnTopBarMouseRightButtonUp(object sender, MouseButtonEventArgs e)
    {
        if (e.OriginalSource is DependencyObject source && FindAncestor<MenuItem>(source) is not null)
        {
            return;
        }

        e.Handled = true;
        SystemCommands.ShowSystemMenu(this, PointToScreen(e.GetPosition(this)));
    }

    private void OnMinimizeClick(object sender, RoutedEventArgs e) => SystemCommands.MinimizeWindow(this);

    private void OnMaximizeClick(object sender, RoutedEventArgs e) => ToggleMaximize();

    private void OnCloseClick(object sender, RoutedEventArgs e) => Close();

    private void ToggleMaximize()
    {
        if (_vm.IsFullScreen)
        {
            return;
        }

        WindowState = WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;
    }

    private void OnWindowStateChanged(object? sender, EventArgs e)
    {
        UpdateMaximizedMargin();
        var maximized = WindowState == WindowState.Maximized;
        MaximizeButton.Content = maximized ? "\uE923" : "\uE922";
        MaximizeButton.ToolTip = maximized ? "이전 크기로 복원" : "최대화";
    }

    private static T? FindAncestor<T>(DependencyObject? node) where T : DependencyObject
    {
        while (node is not null)
        {
            if (node is T match)
            {
                return match;
            }

            node = node is Visual or System.Windows.Media.Media3D.Visual3D
                ? VisualTreeHelper.GetParent(node)
                : LogicalTreeHelper.GetParent(node);
        }

        return null;
    }

    /// <summary>
    /// 제목 표시줄이 없는 창(WindowChrome)은 최대화하면 크기 조절 테두리만큼 화면 밖으로 넘친다.
    /// 넘친 만큼 내용에 여백을 줘서 타이틀바 · 창 버튼 · 하단 바가 잘리거나 작업 표시줄에 가리지 않게 한다.
    /// (전체 화면은 테두리 없는 최대화라 넘치지 않는다)
    /// </summary>
    private void UpdateMaximizedMargin()
    {
        var margin = new Thickness(0);
        if (WindowState == WindowState.Maximized && !_vm.IsFullScreen)
        {
            var hwnd = new System.Windows.Interop.WindowInteropHelper(this).Handle;
            if (NativeMethods.GetOverflowBeyondWorkArea(hwnd) is { } px)
            {
                var dpi = VisualTreeHelper.GetDpi(this);
                margin = new Thickness(
                    px.Left / dpi.DpiScaleX,
                    px.Top / dpi.DpiScaleY,
                    px.Right / dpi.DpiScaleX,
                    px.Bottom / dpi.DpiScaleY);
            }
        }

        if (RootGrid.Margin != margin)
        {
            RootGrid.Margin = margin;
        }
    }

    #endregion

    #region 키보드

    private void OnPreviewKeyDown(object sender, KeyEventArgs e)
    {
        // 텍스트 입력·슬라이더·메뉴에 포커스가 있으면 해당 컨트롤에 맡긴다.
        if (KeyboardHelper.IsInputControlFocused() || Keyboard.FocusedElement is MenuItem)
        {
            return;
        }

        var action = KeyboardHelper.GetAction(e.Key, e.SystemKey, Keyboard.Modifiers);
        if (action == ShortcutAction.None)
        {
            return;
        }

        e.Handled = true;
        switch (action)
        {
            case ShortcutAction.ToggleFullScreen:
                _vm.ToggleFullScreenCommand.Execute(null);
                break;
            case ShortcutAction.Exit:
                Close();
                break;
            case ShortcutAction.ToggleMenus:
                _vm.ToggleMenusCommand.Execute(null);
                break;
            case ShortcutAction.PreviousPage:
                _vm.PreviousPageCommand.Execute(null);
                break;
            case ShortcutAction.NextPage:
                _vm.NextPageCommand.Execute(null);
                break;
            case ShortcutAction.RotateClockwise:
                _vm.RotateClockwiseCommand.Execute(null);
                break;
            case ShortcutAction.RotateCounterClockwise:
                _vm.RotateCounterClockwiseCommand.Execute(null);
                break;
            case ShortcutAction.FlipHorizontal:
                _vm.FlipHorizontalCommand.Execute(null);
                break;
            case ShortcutAction.FlipVertical:
                _vm.FlipVerticalCommand.Execute(null);
                break;
            case ShortcutAction.ToggleInfoPanel:
                _vm.ToggleInfoPanelCommand.Execute(null);
                break;
            case ShortcutAction.Previous:
                _vm.PreviousCommand.Execute(null);
                break;
            case ShortcutAction.Next:
                _vm.NextCommand.Execute(null);
                break;
            case ShortcutAction.First:
                _vm.FirstCommand.Execute(null);
                break;
            case ShortcutAction.Last:
                _vm.LastCommand.Execute(null);
                break;
            case ShortcutAction.OpenImageList:
                _vm.ShowImageListCommand.Execute(null);
                break;
            case ShortcutAction.ModeOriginal:
                _vm.SetDisplayModeCommand.Execute(DisplayMode.Original);
                break;
            case ShortcutAction.ModeFitWidth:
                _vm.SetDisplayModeCommand.Execute(DisplayMode.FitWidth);
                break;
            case ShortcutAction.ModeFitHeight:
                _vm.SetDisplayModeCommand.Execute(DisplayMode.FitHeight);
                break;
            case ShortcutAction.ModeFill:
                _vm.SetDisplayModeCommand.Execute(DisplayMode.Fill);
                break;
            case ShortcutAction.FilterNone:
                _vm.SetFilterCommand.Execute(ImageFilter.None);
                break;
            case ShortcutAction.FilterSmooth:
                _vm.SetFilterCommand.Execute(ImageFilter.Smooth);
                break;
            case ShortcutAction.FilterSmoothSharpen:
                _vm.SetFilterCommand.Execute(ImageFilter.SmoothSharpen);
                break;
            case ShortcutAction.ZoomIn:
                ZoomAt(null, zoomIn: true);
                break;
            case ShortcutAction.ZoomOut:
                ZoomAt(null, zoomIn: false);
                break;
            case ShortcutAction.ToggleSlideShow:
                _vm.ToggleSlideShowCommand.Execute(null);
                break;
            default:
                e.Handled = false;
                break;
        }
    }

    private void OnShowShortcuts(object sender, RoutedEventArgs e)
    {
        const string text =
            "Alt + Enter\t전체 화면 전환\n" +
            "Esc\t\t프로그램 종료 (대화 상자에서는 닫기)\n" +
            "`\t\t상단 · 하단 바 고정(Pin) 일괄 전환\n" +
            "← / ↑\t\t이전 이미지\n" +
            "→ / ↓\t\t다음 이미지\n" +
            "Home / End\t처음 / 마지막 이미지\n" +
            "PgUp / PgDn\t이전 / 다음 페이지 (다중 페이지 TIFF, 그 밖에는 이미지)\n" +
            "R / L\t\t시계 / 반시계 방향 90° 회전\n" +
            "H / V\t\t좌우 / 상하 반전\n" +
            "Tab\t\t이미지 정보(EXIF) 패널\n" +
            "Enter\t\t이미지 목록\n" +
            "0\t\t원본 크기\n" +
            "7\t\t폭 맞춤\n" +
            "8\t\t높이 맞춤\n" +
            "9\t\t꽉 차게 보기\n" +
            "U\t\t필터 없음\n" +
            "I\t\t부드럽게\n" +
            "S\t\t부드럽게 + 선명하게\n" +
            "+ / =\t\t확대\n" +
            "-\t\t축소\n" +
            "Ctrl + 0\t\t배율 초기화\n" +
            "Space\t\t슬라이드쇼 재생 / 일시정지\n" +
            "Ctrl + O\t\t파일 / ZIP 열기\n" +
            "Ctrl + Shift + O\t폴더 열기\n\n" +
            "마우스: 휠 위/아래 = 이전/다음 이미지 (설정 변경 가능), Ctrl+휠 = 확대/축소,\n" +
            "Shift+휠 = 이미지 상하 이동, 드래그 = 창 이동 (전체 화면에서는 이미지 이동),\n" +
            "가운데 버튼 드래그 = 이미지 이동,\n" +
            "더블클릭 = 최대화 / 복원,\n" +
            "뒤로/앞으로 버튼 = 이전/다음 이미지";
        MessageBox.Show(this, text, "단축키 안내", MessageBoxButton.OK, MessageBoxImage.Information);
    }

    #endregion

    #region 표시 영역 · 이동(Pan) · 확대

    private void OnViewportSizeChanged(object sender, SizeChangedEventArgs e) => UpdateViewport();

    private void OnDpiChanged(object sender, DpiChangedEventArgs e)
    {
        UpdateMaximizedMargin();
        UpdateViewport();
    }

    private void UpdateViewport()
    {
        var dpi = VisualTreeHelper.GetDpi(this).DpiScaleX;
        _vm.UpdateViewport(Viewport.ActualWidth, Viewport.ActualHeight, dpi);

        // 이미지가 없거나 크기가 그대로여도 중앙 정렬을 다시 계산한다.
        PositionImage(isNewImage: false);
    }

    private void OnLayoutChanged(object? sender, LayoutChangedEventArgs e) => PositionImage(e.IsNewImage);

    /// <summary>
    /// 이미지 위치를 계산한다.
    /// - 새 이미지: 가로는 가운데, 세로는 넘치면 위쪽부터
    /// - 크기 변경: 확대 기준점(마우스 위치 또는 화면 중앙) 아래의 이미지 지점을 유지
    /// - 표시 영역보다 작은 방향은 항상 가운데 정렬, 큰 방향은 빈 공간이 생기지 않도록 제한
    /// </summary>
    private void PositionImage(bool isNewImage)
    {
        var vw = Viewport.ActualWidth;
        var vh = Viewport.ActualHeight;
        var w = _vm.DisplayWidth;
        var h = _vm.DisplayHeight;

        if (isNewImage || _lastImageWidth <= 0 || _lastImageHeight <= 0)
        {
            _offsetX = (vw - w) / 2;
            _offsetY = h > vh ? 0 : (vh - h) / 2;
        }
        else
        {
            var anchor = _zoomAnchor ?? new Point(_lastViewportWidth / 2, _lastViewportHeight / 2);
            var fx = (anchor.X - _offsetX) / _lastImageWidth;
            var fy = (anchor.Y - _offsetY) / _lastImageHeight;

            // 표시 영역 크기가 바뀐 경우 기준점도 새 영역 기준으로 옮긴다.
            var newAnchor = _zoomAnchor ?? new Point(vw / 2, vh / 2);
            _offsetX = newAnchor.X - fx * w;
            _offsetY = newAnchor.Y - fy * h;
        }

        _zoomAnchor = null;
        _lastImageWidth = w;
        _lastImageHeight = h;
        _lastViewportWidth = vw;
        _lastViewportHeight = vh;
        ClampAndApply();
    }

    private void ClampAndApply()
    {
        var vw = Viewport.ActualWidth;
        var vh = Viewport.ActualHeight;
        var w = _vm.DisplayWidth;
        var h = _vm.DisplayHeight;

        _offsetX = w <= vw ? (vw - w) / 2 : Math.Clamp(_offsetX, vw - w, 0);
        _offsetY = h <= vh ? (vh - h) / 2 : Math.Clamp(_offsetY, vh - h, 0);

        Canvas.SetLeft(DisplayImage, _offsetX);
        Canvas.SetTop(DisplayImage, _offsetY);

        Viewport.Cursor = _isDragging ? Cursors.SizeAll : (_vm.IsFullScreen && CanPan ? Cursors.Hand : null);
        UpdateChrome(allowHide: false);
    }

    private void ZoomAt(Point? anchor, bool zoomIn)
    {
        _zoomAnchor = anchor;
        if (zoomIn)
        {
            _vm.ZoomInCommand.Execute(null);
        }
        else
        {
            _vm.ZoomOutCommand.Execute(null);
        }

        _zoomAnchor = null;
    }

    private void OnViewportMouseWheel(object sender, MouseWheelEventArgs e)
    {
        e.Handled = true;
        var modifiers = Keyboard.Modifiers;

        if (modifiers.HasFlag(ModifierKeys.Shift))
        {
            _offsetY += e.Delta / 2.0;
            ClampAndApply();
            return;
        }

        var zoom = _vm.MouseWheelAction == MouseWheelAction.Zoom;
        if (modifiers.HasFlag(ModifierKeys.Control))
        {
            zoom = !zoom;
        }

        if (zoom)
        {
            ZoomAt(e.GetPosition(Viewport), e.Delta > 0);
        }
        else if (e.Delta > 0)
        {
            _vm.PreviousCommand.Execute(null);
        }
        else
        {
            _vm.NextCommand.Execute(null);
        }
    }

    private void OnViewportMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        Viewport.Focus();
        e.Handled = true;

        // 더블클릭: 최대화 / 복원 (전체 화면은 Alt+Enter 로만 전환)
        if (e.ClickCount == 2)
        {
            if (!_vm.IsFullScreen)
            {
                WindowState = WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;
            }

            return;
        }

        // 왼쪽 드래그는 항상 창 이동. 창을 옮길 수 없는 전체 화면에서만 이미지 이동(Pan)으로 쓴다.
        // 이미지 이동은 가운데 버튼 드래그, Shift+휠(상하)로도 할 수 있다.
        if (_vm.IsFullScreen)
        {
            if (CanPan)
            {
                StartPan(e);
            }

            return;
        }

        try
        {
            DragMove();
        }
        catch (InvalidOperationException)
        {
            // 마우스 버튼이 이미 떼어진 경우
        }
    }

    private void StartPan(MouseButtonEventArgs e)
    {
        _dragStart = e.GetPosition(Viewport);
        _dragStartOffsetX = _offsetX;
        _dragStartOffsetY = _offsetY;
        _isDragging = Viewport.CaptureMouse();
        ClampAndApply();
    }

    private bool CanPan =>
        _vm.CurrentBitmap is not null &&
        (_vm.DisplayWidth > Viewport.ActualWidth + 0.5 || _vm.DisplayHeight > Viewport.ActualHeight + 0.5);

    private void OnViewportMouseMove(object sender, MouseEventArgs e)
    {
        if (!_isDragging)
        {
            return;
        }

        var p = e.GetPosition(Viewport);
        _offsetX = _dragStartOffsetX + (p.X - _dragStart.X);
        _offsetY = _dragStartOffsetY + (p.Y - _dragStart.Y);
        ClampAndApply();
    }

    private void OnViewportMouseUp(object sender, MouseButtonEventArgs e)
    {
        if (_isDragging && e.ChangedButton is MouseButton.Left or MouseButton.Middle)
        {
            Viewport.ReleaseMouseCapture();
        }
    }

    private void OnViewportLostMouseCapture(object sender, MouseEventArgs e)
    {
        _isDragging = false;
        ClampAndApply();
    }

    private void OnViewportMouseDown(object sender, MouseButtonEventArgs e)
    {
        switch (e.ChangedButton)
        {
            case MouseButton.XButton1:
                _vm.PreviousCommand.Execute(null);
                e.Handled = true;
                break;
            case MouseButton.XButton2:
                _vm.NextCommand.Execute(null);
                e.Handled = true;
                break;
            case MouseButton.Middle:
                Viewport.Focus();
                if (CanPan)
                {
                    StartPan(e);
                }

                e.Handled = true;
                break;
        }
    }

    #endregion

    #region 썸네일

    private void ScrollThumbnailIntoView()
    {
        var index = _vm.CurrentIndex;
        if (index < 0 || index >= ThumbnailList.Items.Count || !ThumbnailList.IsVisible)
        {
            return;
        }

        ThumbnailList.ScrollIntoView(ThumbnailList.Items[index]);
    }

    /// <summary>
    /// 썸네일 클릭 → 해당 이미지로 이동.
    /// 썸네일 항목은 키보드 단축키를 가로채지 않도록 포커스를 받지 않게 했는데,
    /// ListBox 는 포커스를 받지 못한 항목의 클릭을 선택으로 처리하지 않으므로 직접 처리한다.
    /// </summary>
    private void OnThumbnailMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        if (e.OriginalSource is DependencyObject source &&
            ItemsControl.ContainerFromElement(ThumbnailList, source) is ListBoxItem { DataContext: ThumbnailItemViewModel item })
        {
            e.Handled = true;
            _ = _vm.GoToAsync(item.Index);
            Viewport.Focus();
        }
    }

    /// <summary>썸네일 영역에서는 휠로 좌우 스크롤한다.</summary>
    private void OnThumbnailMouseWheel(object sender, MouseWheelEventArgs e)
    {
        if (FindScrollViewer(ThumbnailList) is { } scroll)
        {
            scroll.ScrollToHorizontalOffset(scroll.HorizontalOffset - e.Delta);
            e.Handled = true;
        }
    }

    private static ScrollViewer? FindScrollViewer(DependencyObject root)
    {
        if (root is ScrollViewer sv)
        {
            return sv;
        }

        for (var i = 0; i < VisualTreeHelper.GetChildrenCount(root); i++)
        {
            if (FindScrollViewer(VisualTreeHelper.GetChild(root, i)) is { } found)
            {
                return found;
            }
        }

        return null;
    }

    #endregion

    #region 드래그 앤 드롭

    private void OnDragOver(object sender, DragEventArgs e)
    {
        e.Effects = e.Data.GetDataPresent(DataFormats.FileDrop) ? DragDropEffects.Link : DragDropEffects.None;
        e.Handled = true;
    }

    private async void OnDrop(object sender, DragEventArgs e)
    {
        if (e.Data.GetData(DataFormats.FileDrop) is string[] { Length: > 0 } paths)
        {
            Activate();
            await _vm.OpenPathAsync(paths[0]);
        }
    }

    #endregion
}
