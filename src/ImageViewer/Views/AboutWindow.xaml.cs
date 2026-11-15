using System.Diagnostics;
using System.Windows;
using System.Windows.Navigation;
using ImageViewer.Services;
using ImageViewer.ViewModels;

namespace ImageViewer.Views;

public partial class AboutWindow : Window
{
    private readonly AboutInfo _info;

    public AboutWindow(AboutInfo info, UpdateViewModel? update = null)
    {
        InitializeComponent();
        _info = info;
        DataContext = info;
        if (update is null)
        {
            UpdateGroup.Visibility = Visibility.Collapsed;
        }
        else
        {
            UpdateGroup.DataContext = update;
        }
    }

    private void OnRequestNavigate(object sender, RequestNavigateEventArgs e)
    {
        try
        {
            Process.Start(new ProcessStartInfo(e.Uri.AbsoluteUri) { UseShellExecute = true });
        }
        catch (Exception)
        {
            // 기본 브라우저가 없으면 무시
        }

        e.Handled = true;
    }

    private void OnOpenProject(object sender, RoutedEventArgs e)
    {
        if (_info.ProjectUrl is not { } url)
        {
            return;
        }

        try
        {
            Process.Start(new ProcessStartInfo(url) { UseShellExecute = true });
        }
        catch (Exception)
        {
            // 기본 브라우저가 없으면 무시
        }
    }

    private void OnCopy(object sender, RoutedEventArgs e)
    {
        try
        {
            Clipboard.SetText(_info.ToText());
            CopiedText.Visibility = Visibility.Visible;
        }
        catch (Exception)
        {
            // 다른 프로그램이 클립보드를 잡고 있으면 실패할 수 있다
        }
    }
}
