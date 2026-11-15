using ImageViewer.Helpers;
using ImageViewer.Services;

namespace ImageViewer.Tests;

public class ShellIntegrationTests
{
    [Fact]
    public void Registers_folders_archives_and_every_image_extension()
    {
        var keys = ShellIntegrationService.TargetKeys.Select(k => k.Path).ToList();

        Assert.Contains(@"Software\Classes\Directory\shell\ImageViewer", keys);
        Assert.Contains(@"Software\Classes\Directory\Background\shell\ImageViewer", keys);
        Assert.Contains(@"Software\Classes\SystemFileAssociations\.zip\shell\ImageViewer", keys);
        Assert.Contains(@"Software\Classes\SystemFileAssociations\.cbz\shell\ImageViewer", keys);

        foreach (var ext in ImageFormatHelper.SupportedExtensions)
        {
            Assert.Contains($@"Software\Classes\SystemFileAssociations\{ext.ToLowerInvariant()}\shell\ImageViewer", keys);
        }

        Assert.Equal(4 + ImageFormatHelper.SupportedExtensions.Count, keys.Distinct(StringComparer.OrdinalIgnoreCase).Count());
    }

    [Fact]
    public void Only_folder_background_uses_directory_placeholder()
    {
        var background = ShellIntegrationService.TargetKeys.Where(k => k.Background).Select(k => k.Path).ToList();
        Assert.Equal([@"Software\Classes\Directory\Background\shell\ImageViewer"], background);

        Assert.Equal("\"C:\\App\\ImageViewer.exe\" \"%V\"", ShellIntegrationService.BuildCommand(@"C:\App\ImageViewer.exe", background: true));
        Assert.Equal("\"C:\\App\\ImageViewer.exe\" \"%1\"", ShellIntegrationService.BuildCommand(@"C:\App\ImageViewer.exe", background: false));
    }
}
