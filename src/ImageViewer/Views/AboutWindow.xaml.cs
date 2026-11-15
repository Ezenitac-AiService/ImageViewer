using System.Diagnostics;
using System.Windows;
using System.Windows.Navigation;
using ImageViewer.Services;

namespace ImageViewer.Views;

public partial class AboutWindow : Window
{
    private readonly AboutInfo _info;

    public AboutWindow(AboutInfo info)
    {
        InitializeComponent();
        _info = info;
        DataContext = info;
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
