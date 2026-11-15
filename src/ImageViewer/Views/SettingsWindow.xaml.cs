using System.Windows;
using ImageViewer.ViewModels;

namespace ImageViewer.Views;

public partial class SettingsWindow : Window
{
    public SettingsWindow(SettingsViewModel viewModel)
    {
        InitializeComponent();
        DataContext = viewModel;
    }

    private void OnOk(object sender, RoutedEventArgs e) => DialogResult = true;
}
