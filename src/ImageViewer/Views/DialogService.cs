using System.Windows;
using ImageViewer.Services;
using ImageViewer.ViewModels;
using Microsoft.Win32;

namespace ImageViewer.Views;

public sealed class DialogService : IDialogService
{
    private const string Caption = "ImageViewer";

    private static Window? Owner => Application.Current?.MainWindow is { IsLoaded: true } w ? w : null;

    public string? PickFolder()
    {
        var dialog = new OpenFolderDialog { Title = "이미지 폴더 선택" };
        return dialog.ShowDialog(Owner) == true ? dialog.FolderName : null;
    }

    public string? PickFile()
    {
        var images = string.Join(";", Helpers.ImageFormatHelper.SupportedExtensions.Select(e => "*" + e));
        var dialog = new OpenFileDialog
        {
            Title = "이미지 또는 ZIP 파일 열기",
            Filter = $"이미지 및 ZIP|{images};*.zip;*.cbz|ZIP 파일|*.zip;*.cbz|이미지 파일|{images}|모든 파일|*.*",
            CheckFileExists = true,
        };
        return dialog.ShowDialog(Owner) == true ? dialog.FileName : null;
    }

    public bool ShowImageList(ImageListViewModel viewModel)
    {
        var window = new ImageListWindow(viewModel) { Owner = Owner };
        return window.ShowDialog() == true;
    }

    public bool ShowSettings(SettingsViewModel viewModel)
    {
        var window = new SettingsWindow(viewModel) { Owner = Owner };
        return window.ShowDialog() == true;
    }

    public void ShowMessage(string message, bool isError = false)
    {
        var icon = isError ? MessageBoxImage.Warning : MessageBoxImage.Information;
        if (Owner is { } owner)
        {
            MessageBox.Show(owner, message, Caption, MessageBoxButton.OK, icon);
        }
        else
        {
            MessageBox.Show(message, Caption, MessageBoxButton.OK, icon);
        }
    }

    public bool Confirm(string message)
    {
        var owner = Application.Current?.Windows.OfType<Window>().FirstOrDefault(w => w.IsActive) ?? Owner;
        var result = owner is null
            ? MessageBox.Show(message, Caption, MessageBoxButton.YesNo, MessageBoxImage.Question)
            : MessageBox.Show(owner, message, Caption, MessageBoxButton.YesNo, MessageBoxImage.Question);
        return result == MessageBoxResult.Yes;
    }
}
