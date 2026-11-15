using System.Runtime.InteropServices;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using Microsoft.Win32;

namespace ImageViewer.Services;

/// <summary>
/// Windows 탐색기 컨텍스트 메뉴 "ImageViewer에서 보기" 등록/해제.
/// 관리자 권한 없이 현재 사용자(HKCU\Software\Classes)에 등록한다.
/// 설치 프로그램은 설치 범위에 따라 HKLM/HKCU 에 같은 구조로 등록한다.
///
/// - 폴더:            Directory\shell\ImageViewer            ("%1")
/// - 폴더 빈 공간:    Directory\Background\shell\ImageViewer ("%V")
/// - ZIP/CBZ 파일:    SystemFileAssociations\.zip\shell\ImageViewer ("%1")
///
/// Windows 11 에서는 클래식 메뉴(추가 옵션 표시, Shift+F10)에 표시된다.
/// </summary>
public sealed class ShellIntegrationService
{
    public const string VerbName = "ImageViewer";
    public const string MenuText = "ImageViewer에서 보기";

    private static readonly string[] TargetKeys =
    [
        @"Software\Classes\Directory\shell\" + VerbName,
        @"Software\Classes\Directory\Background\shell\" + VerbName,
        @"Software\Classes\SystemFileAssociations\.zip\shell\" + VerbName,
        @"Software\Classes\SystemFileAssociations\.cbz\shell\" + VerbName,
    ];

    private readonly ILogger<ShellIntegrationService> _logger;

    public ShellIntegrationService(ILogger<ShellIntegrationService>? logger = null)
    {
        _logger = logger ?? NullLogger<ShellIntegrationService>.Instance;
    }

    public static string ExecutablePath =>
        Environment.ProcessPath ?? Path.Combine(AppContext.BaseDirectory, "ImageViewer.exe");

    public static string BuildCommand(string exePath, bool background) =>
        $"\"{exePath}\" \"{(background ? "%V" : "%1")}\"";

    public bool IsRegistered()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(TargetKeys[0] + @"\command");
            var value = key?.GetValue(null) as string;
            return value is not null &&
                   value.Contains(ExecutablePath, StringComparison.OrdinalIgnoreCase);
        }
        catch (Exception ex) when (ex is System.Security.SecurityException or UnauthorizedAccessException or IOException)
        {
            _logger.LogWarning(ex, "컨텍스트 메뉴 등록 상태를 확인하지 못했습니다.");
            return false;
        }
    }

    public void Register()
    {
        var exe = ExecutablePath;
        foreach (var path in TargetKeys)
        {
            var background = path.Contains(@"\Background\", StringComparison.Ordinal);
            using var key = Registry.CurrentUser.CreateSubKey(path, writable: true);
            key.SetValue(null, MenuText);
            key.SetValue("Icon", $"\"{exe}\",0");
            using var command = key.CreateSubKey("command", writable: true);
            command.SetValue(null, BuildCommand(exe, background));
        }

        NotifyShell();
        _logger.LogInformation("탐색기 컨텍스트 메뉴를 등록했습니다.");
    }

    public void Unregister()
    {
        foreach (var path in TargetKeys)
        {
            Registry.CurrentUser.DeleteSubKeyTree(path, throwOnMissingSubKey: false);
        }

        NotifyShell();
        _logger.LogInformation("탐색기 컨텍스트 메뉴를 제거했습니다.");
    }

    private static void NotifyShell()
    {
        const int SHCNE_ASSOCCHANGED = 0x08000000;
        const uint SHCNF_IDLIST = 0x0000;
        try
        {
            SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, IntPtr.Zero, IntPtr.Zero);
        }
        catch (Exception)
        {
            // 알림 실패는 등록 자체에 영향을 주지 않는다.
        }
    }

    [DllImport("shell32.dll")]
    private static extern void SHChangeNotify(int wEventId, uint uFlags, IntPtr dwItem1, IntPtr dwItem2);
}
