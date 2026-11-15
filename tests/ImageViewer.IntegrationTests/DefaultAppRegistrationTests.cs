using ImageViewer.Services;
using Microsoft.Win32;

namespace ImageViewer.IntegrationTests;

/// <summary>
/// 기본 앱 등록을 실제 Software\Classes 대신 HKCU\Software\ImageViewer.Tests\{guid} 아래에 써서 확인한다.
/// (테스트가 끝나면 지운다)
/// </summary>
public sealed class DefaultAppRegistrationTests : IDisposable
{
    private const string Exe = @"C:\Apps\ImageViewer\ImageViewer.exe";
    private readonly string _base = @"Software\ImageViewer.Tests\" + Guid.NewGuid().ToString("N");

    public void Dispose()
    {
        Registry.CurrentUser.DeleteSubKeyTree(_base, throwOnMissingSubKey: false);
        using (var parent = Registry.CurrentUser.OpenSubKey(@"Software\ImageViewer.Tests"))
        {
            if (parent is not { SubKeyCount: 0, ValueCount: 0 })
            {
                return; // 다른 테스트가 아직 쓰는 중
            }
        }

        Registry.CurrentUser.DeleteSubKey(@"Software\ImageViewer.Tests", throwOnMissingSubKey: false);
    }

    private RegistryKey? Open(string relative) => Registry.CurrentUser.OpenSubKey(_base + relative);

    [Fact]
    public void Registers_progids_capabilities_and_open_with_entries()
    {
        Assert.False(DefaultAppRegistration.IsRegistered(Registry.CurrentUser, Exe, _base));
        DefaultAppRegistration.Register(Registry.CurrentUser, Exe, _base);
        Assert.True(DefaultAppRegistration.IsRegistered(Registry.CurrentUser, Exe, _base));
        Assert.False(DefaultAppRegistration.IsRegistered(Registry.CurrentUser, @"D:\other\ImageViewer.exe", _base));

        using (var command = Open(@"\Classes\ImageViewer.Image\shell\open\command"))
        {
            Assert.Equal($"\"{Exe}\" \"%1\"", command!.GetValue(null));
        }

        using (var archive = Open(@"\Classes\ImageViewer.Archive\shell\open\command"))
        {
            Assert.Equal($"\"{Exe}\" \"%1\"", archive!.GetValue(null));
        }

        using (var files = Open(@"\ImageViewer\Capabilities\FileAssociations"))
        {
            Assert.Equal("ImageViewer.Image", files!.GetValue(".jpg"));
            Assert.Equal("ImageViewer.Image", files.GetValue(".webp"));
            Assert.Equal("ImageViewer.Archive", files.GetValue(".cbz"));
            Assert.Equal(DefaultAppRegistration.Associations.Count, files.ValueCount);
        }

        using (var apps = Open(@"\RegisteredApplications"))
        {
            Assert.Equal(@"Software\ImageViewer\Capabilities", apps!.GetValue("ImageViewer"));
        }

        using (var openWith = Open(@"\Classes\.png\OpenWithProgids"))
        {
            Assert.Contains("ImageViewer.Image", openWith!.GetValueNames());
        }
    }

    [Fact]
    public void Unregister_removes_only_imageviewer_entries()
    {
        // 다른 프로그램의 "연결 프로그램" 항목과 RegisteredApplications 값은 남아야 한다.
        using (var other = Registry.CurrentUser.CreateSubKey(_base + @"\Classes\.jpg\OpenWithProgids"))
        {
            other.SetValue("Other.Viewer", Array.Empty<byte>(), RegistryValueKind.None);
        }

        using (var apps = Registry.CurrentUser.CreateSubKey(_base + @"\RegisteredApplications"))
        {
            apps.SetValue("OtherApp", @"Software\Other\Capabilities");
        }

        DefaultAppRegistration.Register(Registry.CurrentUser, Exe, _base);
        DefaultAppRegistration.Unregister(Registry.CurrentUser, _base);

        Assert.False(DefaultAppRegistration.IsRegistered(Registry.CurrentUser, Exe, _base));
        Assert.Null(Open(@"\Classes\ImageViewer.Image"));
        Assert.Null(Open(@"\Classes\ImageViewer.Archive"));
        Assert.Null(Open(@"\ImageViewer")); // 비어 있으면 앱 키도 지운다

        using var openWith = Open(@"\Classes\.jpg\OpenWithProgids");
        Assert.Equal(["Other.Viewer"], openWith!.GetValueNames());
        using var registered = Open(@"\RegisteredApplications");
        Assert.Equal(["OtherApp"], registered!.GetValueNames());
    }
}
