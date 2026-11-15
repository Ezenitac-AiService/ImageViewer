using System.Text.Json;
using System.Text.Json.Serialization;
using ImageViewer.Models;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace ImageViewer.Services;

/// <summary>
/// 사용자 설정을 %LOCALAPPDATA%\ImageViewer\settings.json 에 저장·복원한다.
/// 파일이 없거나 손상되었으면 기본값을 사용한다.
/// </summary>
public sealed class SettingsService
{
    public static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true,
        ReadCommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true,
        Converters = { new JsonStringEnumConverter() },
    };

    private readonly ILogger<SettingsService> _logger;

    public SettingsService(ILogger<SettingsService>? logger = null, string? settingsPath = null)
    {
        _logger = logger ?? NullLogger<SettingsService>.Instance;
        SettingsPath = settingsPath ?? Path.Combine(AppPaths.DataDirectory, "settings.json");
    }

    public string SettingsPath { get; }

    public ViewerSettings Current { get; private set; } = new();

    public ViewerSettings Load()
    {
        try
        {
            if (File.Exists(SettingsPath))
            {
                var json = File.ReadAllText(SettingsPath);
                Current = (JsonSerializer.Deserialize<ViewerSettings>(json, JsonOptions) ?? new ViewerSettings()).Normalize();
                return Current;
            }
        }
        catch (Exception ex) when (ex is JsonException or IOException or UnauthorizedAccessException or NotSupportedException)
        {
            _logger.LogWarning(ex, "설정 파일을 읽을 수 없어 기본값을 사용합니다.");
        }

        Current = new ViewerSettings().Normalize();
        return Current;
    }

    public void Save()
    {
        try
        {
            var dir = Path.GetDirectoryName(SettingsPath);
            if (!string.IsNullOrEmpty(dir))
            {
                Directory.CreateDirectory(dir);
            }

            var json = JsonSerializer.Serialize(Current, JsonOptions);

            // 저장 도중 종료되어도 기존 파일이 깨지지 않도록 임시 파일에 쓴 뒤 교체한다.
            var temp = SettingsPath + ".tmp";
            File.WriteAllText(temp, json);
            File.Move(temp, SettingsPath, overwrite: true);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            _logger.LogWarning(ex, "설정 파일을 저장하지 못했습니다.");
        }
    }
}

public static class AppPaths
{
    public static string DataDirectory { get; } = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "ImageViewer");

    public static string LogDirectory => Path.Combine(DataDirectory, "logs");
}
