using ImageViewer.ViewModels;

namespace ImageViewer.Services;

/// <summary>ViewModel 이 창·대화 상자에 직접 의존하지 않도록 분리한 인터페이스.</summary>
public interface IDialogService
{
    string? PickFolder();

    string? PickFile();

    /// <summary>이미지 목록 대화 상자. 열기를 선택하면 true.</summary>
    bool ShowImageList(ImageListViewModel viewModel);

    bool ShowSettings(SettingsViewModel viewModel);

    void ShowMessage(string message, bool isError = false);

    bool Confirm(string message);
}
