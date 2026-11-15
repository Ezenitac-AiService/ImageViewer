namespace ImageViewer.Helpers;

public sealed record CommandLineOptions(
    string? Path,
    bool RegisterShell,
    bool UnregisterShell,
    string? Error);

public static class CommandLineHelper
{
    public static CommandLineOptions Parse(IReadOnlyList<string> args)
    {
        string? path = null;
        var register = false;
        var unregister = false;

        foreach (var raw in args)
        {
            if (string.IsNullOrWhiteSpace(raw))
            {
                continue;
            }

            switch (raw.Trim().ToLowerInvariant())
            {
                case "--register-shell":
                case "/register-shell":
                    register = true;
                    continue;
                case "--unregister-shell":
                case "/unregister-shell":
                    unregister = true;
                    continue;
            }

            // 첫 번째 경로 인수만 사용한다.
            path ??= raw;
        }

        if (path is null)
        {
            return new CommandLineOptions(null, register, unregister, null);
        }

        var normalized = NormalizePath(path);
        if (normalized is null)
        {
            return new CommandLineOptions(null, register, unregister, "경로 형식이 올바르지 않습니다.");
        }

        if (!File.Exists(normalized) && !Directory.Exists(normalized))
        {
            return new CommandLineOptions(null, register, unregister, $"경로를 찾을 수 없습니다.\n{normalized}");
        }

        return new CommandLineOptions(normalized, register, unregister, null);
    }

    /// <summary>
    /// 명령행 경로를 정규화한다.
    /// 탐색기가 드라이브 루트를 "%1"로 넘기면 <c>D:\"</c> 가 <c>D:"</c> 로 깨지는 문제를 보정한다.
    /// </summary>
    public static string? NormalizePath(string path)
    {
        var p = path.Trim().Trim('"').Trim();
        if (p.Length == 0)
        {
            return null;
        }

        if (p.Length == 2 && char.IsAsciiLetter(p[0]) && p[1] == ':')
        {
            p += Path.DirectorySeparatorChar;
        }

        if (p.IndexOfAny(Path.GetInvalidPathChars()) >= 0)
        {
            return null;
        }

        try
        {
            return Path.GetFullPath(p);
        }
        catch (Exception ex) when (ex is ArgumentException or NotSupportedException or PathTooLongException or System.Security.SecurityException)
        {
            return null;
        }
    }
}
