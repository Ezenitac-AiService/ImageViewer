using System.Runtime.InteropServices;
using ImageViewer.Helpers;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using Microsoft.Win32;

namespace ImageViewer.Services;

/// <summary>
/// Windows 탐색기 컨텍스트 메뉴 "ImageViewer에서 보기" 등록/해제.
/// 관리자 권한 없이 현재 사용자(HKCU\Software\Classes)에 등록한다.
/// 설치 프로그램도 이 기능(<c>--register-shell</c> / <c>--unregister-shell</c>)을 실행해 같은 키를 쓴다.
///
/// - 폴더:            Directory\shell\ImageViewer                    ("%1")
/// - 폴더 빈 공간:    Directory\Background\shell\ImageViewer         ("%V")
/// - ZIP/CBZ 파일:    SystemFileAssociations\.zip\shell\ImageViewer  ("%1")
/// - 이미지 파일:     SystemFileAssociations\.jpg\shell\ImageViewer  ("%1") — 지원하는 모든 확장자
///                    (이미지 파일을 열면 그 폴더를 열고 해당 이미지를 선택한다)
///
/// 여러 항목을 선택했을 때 창이 여러 개 뜨지 않도록 단일 선택에서만 메뉴를 보인다.
/// Windows 11 에서는 클래식 메뉴(추가 옵션 표시, Shift+F10)에 표시된다.
/// </summary>
public sealed class ShellIntegrationService
{
    public const string VerbName = "ImageViewer";
    public const string MenuText = "ImageViewer에서 보기";

    private const string Classes = @"Software\Classes\";

    private readonly ILogger<ShellIntegrationService> _logger;

    public ShellIntegrationService(ILogger<ShellIntegrationService>? logger = null)
    {
        _logger = logger ?? NullLogger<ShellIntegrationService>.Instance;
    }

    /// <summary>등록할 메뉴 키 (HKCU 기준 경로) 와 폴더 빈 공간 여부.</summary>
    internal static IReadOnlyList<(string Path, bool Background)> TargetKeys { get; } = BuildTargetKeys();

    private static List<(string, bool)> BuildTargetKeys()
    {
        var keys = new List<(string, bool)>
        {
            (Classes + @"Directory\shell\" + VerbName, false),
            (Classes + @"Directory\Background\shell\" + VerbName, true),
            (Classes + @"SystemFileAssociations\.zip\shell\" + VerbName, false),
            (Classes + @"SystemFileAssociations\.cbz\shell\" + VerbName, false),
        };

        foreach (var ext in ImageFormatHelper.SupportedExtensions.Order(StringComparer.OrdinalIgnoreCase))
        {
            keys.Add((Classes + @"SystemFileAssociations\" + ext.ToLowerInvariant() + @"\shell\" + VerbName, false));
        }

        return keys;
    }

    public static string ExecutablePath =>
        Environment.ProcessPath ?? Path.Combine(AppContext.BaseDirectory, "ImageViewer.exe");

    public static string BuildCommand(string exePath, bool background) =>
        $"\"{exePath}\" \"{(background ? "%V" : "%1")}\"";

    public bool IsRegistered()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(TargetKeys[0].Path + @"\command");
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
        foreach (var (path, background) in TargetKeys)
        {
            using var key = Registry.CurrentUser.CreateSubKey(path, writable: true);
            key.SetValue(null, MenuText);
            key.SetValue("Icon", $"\"{exe}\",0");
            key.SetValue("MultiSelectModel", "Single");
            using var command = key.CreateSubKey("command", writable: true);
            command.SetValue(null, BuildCommand(exe, background));
        }

        NotifyShell();
        _logger.LogInformation("탐색기 컨텍스트 메뉴를 등록했습니다. ({Count}개 항목)", TargetKeys.Count);
    }

    public void Unregister()
    {
        foreach (var (path, _) in TargetKeys)
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
