using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Threading;
using ImageViewer.ViewModels;

namespace ImageViewer.Views;

public partial class ImageListWindow : Window
{
    private readonly ImageListViewModel _vm;

    public ImageListWindow(ImageListViewModel viewModel)
    {
        InitializeComponent();
        _vm = viewModel;
        DataContext = viewModel;
        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        ScrollSelectionIntoView(focusList: true);
    }

    /// <summary>취소 버튼이 없으므로 Esc 로 닫는다.</summary>
    private void OnPreviewKeyDown(object sender, KeyEventArgs e)
    {
        if (e.Key == Key.Escape)
        {
            DialogResult = false;
            e.Handled = true;
        }
    }

    private void OnHeaderClick(object sender, RoutedEventArgs e)
    {
        if (e.OriginalSource is GridViewColumnHeader { Tag: string tag } &&
            Enum.TryParse<ImageListSortColumn>(tag, out var column))
        {
            _vm.SortByCommand.Execute(column);
            ScrollSelectionIntoView(focusList: false);
        }
    }

    private void OnListSelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        // 목록에서 선택하면 썸네일 줄도 해당 위치로 스크롤한다.
        var index = _vm.SelectedThumbnailIndex;
        if (index >= 0 && index < ThumbnailStrip.Items.Count)
        {
            ThumbnailStrip.ScrollIntoView(ThumbnailStrip.Items[index]);
        }

        if (_vm.SelectedItem is { } item && !List.IsKeyboardFocusWithin)
        {
            List.ScrollIntoView(item);
        }
    }

    private void ScrollSelectionIntoView(bool focusList)
    {
        if (_vm.SelectedItem is not { } item)
        {
            if (focusList)
            {
                List.Focus();
            }

            return;
        }

        List.ScrollIntoView(item);
        var index = _vm.SelectedThumbnailIndex;
        if (index >= 0 && index < ThumbnailStrip.Items.Count)
        {
            ThumbnailStrip.ScrollIntoView(ThumbnailStrip.Items[index]);
        }

        if (!focusList)
        {
            return;
        }

        // 컨테이너가 만들어진 뒤 포커스를 줘야 방향키로 바로 이동할 수 있다.
        Dispatcher.BeginInvoke(() =>
        {
            if (List.ItemContainerGenerator.ContainerFromItem(item) is ListViewItem container)
            {
                container.Focus();
            }
            else
            {
                List.Focus();
            }
        }, DispatcherPriority.Loaded);
    }

    private void OnThumbnailMouseWheel(object sender, MouseWheelEventArgs e)
    {
        if (FindScrollViewer(ThumbnailStrip) is { } scroll)
        {
            scroll.ScrollToHorizontalOffset(scroll.HorizontalOffset - e.Delta);
            e.Handled = true;
        }
    }

    /// <summary>
    /// 썸네일 클릭 → 목록에서 선택, 더블클릭 → 열기.
    /// (포커스를 받지 않는 항목은 ListBox 가 클릭을 선택으로 처리하지 않으므로 직접 처리한다)
    /// </summary>
    private void OnThumbnailMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        if (e.OriginalSource is not DependencyObject source ||
            ItemsControl.ContainerFromElement(ThumbnailStrip, source) is not ListBoxItem { DataContext: ThumbnailItemViewModel item })
        {
            return;
        }

        e.Handled = true;
        _vm.SelectedThumbnailIndex = item.Index;
        if (e.ClickCount == 2)
        {
            Accept();
        }
    }

    private void OnListDoubleClick(object sender, MouseButtonEventArgs e)
    {
        if (e.OriginalSource is DependencyObject source &&
            ItemsControl.ContainerFromElement(List, source) is ListViewItem)
        {
            Accept();
        }
    }

    private void OnOk(object sender, RoutedEventArgs e) => Accept();

    private void Accept()
    {
        if (_vm.Accept())
        {
            DialogResult = true;
        }
        else
        {
            DialogResult = false;
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
}
