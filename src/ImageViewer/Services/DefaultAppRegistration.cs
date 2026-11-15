using System.Diagnostics;
using System.Runtime.InteropServices;
using ImageViewer.Helpers;
using Microsoft.Win32;

namespace ImageViewer.Services;

/// <summary>
/// Windows "기본 앱" 등록.
///
/// Windows 10 / 11 은 앱이 스스로 기본 앱이 되는 것을 막는다 (사용자가 설정에서 직접 골라야 한다).
/// 그래서 ImageViewer 를 기본 앱 후보로 등록한 뒤, 설정 &gt; 앱 &gt; 기본 앱의 ImageViewer 화면을 연다.
/// 모두 현재 사용자(HKCU) 아래에 쓰므로 관리자 권한이 필요 없다.
///
/// - Software\Classes\ImageViewer.Image · ImageViewer.Archive   열기 명령 ("exe" "%1") 과 아이콘 (ProgID)
/// - Software\Classes\.jpg\OpenWithProgids\ImageViewer.Image      "연결 프로그램" 목록에 표시
/// - Software\ImageViewer\Capabilities\FileAssociations           기본 앱 설정에서 고를 수 있는 형식
/// - Software\RegisteredApplications\ImageViewer                  기본 앱 목록에 앱 표시
/// </summary>
public static class DefaultAppRegistration
{
    public const string AppName = "ImageViewer";
    public const string ImageProgId = "ImageViewer.Image";
    public const string ArchiveProgId = "ImageViewer.Archive";
    public const string CapabilitiesPath = @"Software\ImageViewer\Capabilities";

    /// <summary>기본 앱으로 고를 수 있는 확장자와 ProgID (이미지 + ZIP / CBZ).</summary>
    public static IReadOnlyList<(string Extension, string ProgId)> Associations { get; } =
        ImageFormatHelper.SupportedExtensions
            .Order(StringComparer.OrdinalIgnoreCase)
            .Select(e => (e.ToLowerInvariant(), ImageProgId))
            .Concat([(".zip", ArchiveProgId), (".cbz", ArchiveProgId)])
            .ToList();

    /// <param name="root">보통 <see cref="Registry.CurrentUser"/>.</param>
    /// <param name="softwarePath">보통 "Software" (테스트에서는 별도 키 아래에 쓴다).</param>
    public static void Register(RegistryKey root, string exePath, string softwarePath = "Software")
    {
        var classes = softwarePath + @"\Classes\";
        WriteProgId(root, classes + ImageProgId, "ImageViewer 이미지", exePath);
        WriteProgId(root, classes + ArchiveProgId, "ImageViewer 이미지 묶음 (ZIP)", exePath);

        foreach (var (ext, progId) in Associations)
        {
            using var openWith = root.CreateSubKey(classes + ext + @"\OpenWithProgids", writable: true);
            openWith.SetValue(progId, Array.Empty<byte>(), RegistryValueKind.None);
        }

        using (var caps = root.CreateSubKey(Rebase(softwarePath, CapabilitiesPath), writable: true))
        {
            caps.SetValue("ApplicationName", AppName);
            caps.SetValue("ApplicationDescription", "폴더 · ZIP 이미지 뷰어");
            caps.SetValue("ApplicationIcon", $"\"{exePath}\",0");
            using var files = caps.CreateSubKey("FileAssociations", writable: true);
            foreach (var (ext, progId) in Associations)
            {
                files.SetValue(ext, progId);
            }
        }

        using (var apps = root.CreateSubKey(softwarePath + @"\RegisteredApplications", writable: true))
        {
            apps.SetValue(AppName, CapabilitiesPath);
        }
    }

    public static void Unregister(RegistryKey root, string softwarePath = "Software")
    {
        var classes = softwarePath + @"\Classes\";
        foreach (var (ext, progId) in Associations)
        {
            using var openWith = root.OpenSubKey(classes + ext + @"\OpenWithProgids", writable: true);
            openWith?.DeleteValue(progId, throwOnMissingValue: false);
        }

        root.DeleteSubKeyTree(classes + ImageProgId, throwOnMissingSubKey: false);
        root.DeleteSubKeyTree(classes + ArchiveProgId, throwOnMissingSubKey: false);
        root.DeleteSubKeyTree(Rebase(softwarePath, CapabilitiesPath), throwOnMissingSubKey: false);
        var appKey = Rebase(softwarePath, @"Software\ImageViewer");
        using (var app = root.OpenSubKey(appKey))
        {
            if (app is { SubKeyCount: 0, ValueCount: 0 })
            {
                app.Dispose();
                root.DeleteSubKey(appKey, throwOnMissingSubKey: false);
            }
        }

        using var apps = root.OpenSubKey(softwarePath + @"\RegisteredApplications", writable: true);
        apps?.DeleteValue(AppName, throwOnMissingValue: false);
    }

    /// <summary>지금 실행 파일로 등록되어 있는지 (열기 명령이 이 exe 를 가리키는지).</summary>
    public static bool IsRegistered(RegistryKey root, string exePath, string softwarePath = "Software")
    {
        using var apps = root.OpenSubKey(softwarePath + @"\RegisteredApplications");
        using var command = root.OpenSubKey(softwarePath + @"\Classes\" + ImageProgId + @"\shell\open\command");
        return apps?.GetValue(AppName) is string &&
               command?.GetValue(null) is string value &&
               value.Contains(exePath, StringComparison.OrdinalIgnoreCase);
    }

    internal static string OpenCommand(string exePath) => $"\"{exePath}\" \"%1\"";

    private static void WriteProgId(RegistryKey root, string path, string friendlyName, string exePath)
    {
        using var key = root.CreateSubKey(path, writable: true);
        key.SetValue(null, friendlyName);
        using (var icon = key.CreateSubKey("DefaultIcon", writable: true))
        {
            icon.SetValue(null, $"\"{exePath}\",0");
        }

        using var command = key.CreateSubKey(@"shell\open\command", writable: true);
        command.SetValue(null, OpenCommand(exePath));
    }

    /// <summary>"Software\..." 경로를 다른 기준 경로(테스트용) 아래로 옮긴다.</summary>
    private static string Rebase(string softwarePath, string path) =>
        softwarePath == "Software" ? path : softwarePath + path["Software".Length..];

    /// <summary>
    /// 설정 &gt; 앱 &gt; 기본 앱의 ImageViewer 화면을 연다 (Windows 11).
    /// 그 화면을 지원하지 않는 Windows 에서는 기본 앱 첫 화면이 열린다.
    /// </summary>
    public static void OpenSettings()
    {
        try
        {
            Process.Start(new ProcessStartInfo($"ms-settings:defaultapps?registeredAppUser={AppName}") { UseShellExecute = true });
        }
        catch (Exception ex) when (ex is System.ComponentModel.Win32Exception or InvalidOperationException)
        {
            Process.Start(new ProcessStartInfo("ms-settings:defaultapps") { UseShellExecute = true });
        }
    }

    /// <summary>연결 정보가 바뀌었음을 탐색기에 알린다.</summary>
    public static void NotifyShell()
    {
        try
        {
            SHChangeNotify(0x08000000, 0, IntPtr.Zero, IntPtr.Zero);
        }
        catch (Exception)
        {
        }
    }

    [DllImport("shell32.dll")]
    private static extern void SHChangeNotify(int wEventId, uint uFlags, IntPtr dwItem1, IntPtr dwItem2);
}
