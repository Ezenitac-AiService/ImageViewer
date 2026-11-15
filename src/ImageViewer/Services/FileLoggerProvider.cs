using System.Text;
using Microsoft.Extensions.Logging;

namespace ImageViewer.Services;

/// <summary>
/// %LOCALAPPDATA%\ImageViewer\logs 에 일자별 로그 파일을 남기는 간단한 로거.
/// 개인 정보 보호를 위해 로그에는 전체 경로 대신 파일명만 기록하도록 호출부에서 주의한다.
/// </summary>
public sealed class FileLoggerProvider : ILoggerProvider
{
    private const int RetentionDays = 7;

    private readonly string _directory;
    private readonly LogLevel _minLevel;
    private readonly Lock _lock = new();

    public FileLoggerProvider(string directory, LogLevel minLevel = LogLevel.Information)
    {
        _directory = directory;
        _minLevel = minLevel;
        TryCleanup();
    }

    public ILogger CreateLogger(string categoryName) => new FileLogger(this, categoryName);

    public void Dispose()
    {
    }

    private void Write(string category, LogLevel level, string message, Exception? exception)
    {
        var sb = new StringBuilder();
        sb.Append(DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff"))
          .Append(" [").Append(level.ToString().ToUpperInvariant()[..4]).Append("] ")
          .Append(category[(category.LastIndexOf('.') + 1)..]).Append(": ")
          .AppendLine(message);

        if (exception is not null)
        {
            // 예외 메시지에 사용자 경로가 포함될 수 있으므로 형식과 메시지만 기록한다.
            sb.Append("    ").Append(exception.GetType().Name).Append(": ").AppendLine(exception.Message);
            if (exception.InnerException is { } inner)
            {
                sb.Append("    inner ").Append(inner.GetType().Name).Append(": ").AppendLine(inner.Message);
            }
        }

        lock (_lock)
        {
            try
            {
                Directory.CreateDirectory(_directory);
                var file = Path.Combine(_directory, $"imageviewer-{DateTime.Now:yyyyMMdd}.log");
                File.AppendAllText(file, sb.ToString());
            }
            catch (Exception)
            {
                // 로그 기록 실패는 무시한다.
            }
        }
    }

    private void TryCleanup()
    {
        try
        {
            if (!Directory.Exists(_directory))
            {
                return;
            }

            var threshold = DateTime.Now.AddDays(-RetentionDays);
            foreach (var file in Directory.EnumerateFiles(_directory, "imageviewer-*.log"))
            {
                if (File.GetLastWriteTime(file) < threshold)
                {
                    File.Delete(file);
                }
            }
        }
        catch (Exception)
        {
        }
    }

    private sealed class FileLogger(FileLoggerProvider provider, string category) : ILogger
    {
        public IDisposable? BeginScope<TState>(TState state) where TState : notnull => null;

        public bool IsEnabled(LogLevel logLevel) => logLevel >= provider._minLevel && logLevel != LogLevel.None;

        public void Log<TState>(LogLevel logLevel, EventId eventId, TState state, Exception? exception, Func<TState, Exception?, string> formatter)
        {
            if (IsEnabled(logLevel))
            {
                provider.Write(category, logLevel, formatter(state, exception), exception);
            }
        }
    }
}
