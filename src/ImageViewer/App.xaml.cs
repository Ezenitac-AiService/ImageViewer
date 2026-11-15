using System.Windows;
using System.Windows.Threading;
using ImageMagick;
using ImageViewer.Helpers;
using ImageViewer.Services;
using ImageViewer.Services.ImageSources;
using ImageViewer.ViewModels;
using ImageViewer.Views;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace ImageViewer;

public partial class App : Application
{
    private ServiceProvider? _services;
    private ILogger<App>? _logger;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

        var options = CommandLineHelper.Parse(e.Args);

        // 설치 프로그램에서 사용하는 컨텍스트 메뉴 등록/해제 모드
        if (options.RegisterShell || options.UnregisterShell)
        {
            var exitCode = RunShellCommand(options);
            Shutdown(exitCode);
            return;
        }

        _services = ConfigureServices();
        _logger = _services.GetRequiredService<ILogger<App>>();
        RegisterExceptionHandlers();
        ConfigureMagick();

        _services.GetRequiredService<SettingsService>().Load();

        var window = _services.GetRequiredService<MainWindow>();
        MainWindow = window;
        window.Show();
        _logger.LogInformation("ImageViewer 시작 (Magick.NET {Version})", MagickNET.Version);

        if (options.Error is not null)
        {
            MessageBox.Show(window, options.Error, "ImageViewer", MessageBoxButton.OK, MessageBoxImage.Warning);
        }
        else if (options.Path is not null)
        {
            _ = window.ViewModel.OpenPathAsync(options.Path);
        }
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _services?.Dispose();
        base.OnExit(e);
    }

    private static ServiceProvider ConfigureServices()
    {
        var services = new ServiceCollection();

        services.AddLogging(builder =>
        {
            builder.SetMinimumLevel(LogLevel.Information);
            builder.AddDebug();
            builder.AddProvider(new FileLoggerProvider(AppPaths.LogDirectory));
        });

        services.AddSingleton<SettingsService>(sp => new SettingsService(sp.GetRequiredService<ILogger<SettingsService>>()));
        services.AddSingleton<IImageSourceFactory, ImageSourceFactory>();
        services.AddSingleton<ImageLoaderService>();
        services.AddSingleton<ImageFilterService>();
        services.AddSingleton<ImageRenderService>();
        services.AddSingleton<ThumbnailService>();
        services.AddSingleton<ImageNavigationService>();
        services.AddSingleton<SlideShowService>(_ => new SlideShowService());
        services.AddSingleton<ShellIntegrationService>();
        services.AddSingleton<IDialogService, DialogService>();
        services.AddSingleton<MainViewModel>();
        services.AddSingleton<MainWindow>();

        return services.BuildServiceProvider();
    }

    private static void ConfigureMagick()
    {
        // 지나치게 큰 이미지로 인한 메모리 과다 사용 방지 (픽셀 수 검사는 디코딩 전에 별도로 수행)
        ResourceLimits.Width = 100_000;
        ResourceLimits.Height = 100_000;
        ResourceLimits.ListLength = ImageLoaderService.MaxFrames + 24; // 애니메이션 · 다중 페이지 프레임 수 한도
    }

    private void RegisterExceptionHandlers()
    {
        DispatcherUnhandledException += OnDispatcherUnhandledException;
        TaskScheduler.UnobservedTaskException += (_, args) =>
        {
            _logger?.LogError(args.Exception, "처리되지 않은 비동기 작업 예외");
            args.SetObserved();
        };
        AppDomain.CurrentDomain.UnhandledException += (_, args) =>
        {
            if (args.ExceptionObject is Exception ex)
            {
                _logger?.LogCritical(ex, "처리되지 않은 예외");
            }
        };
    }

    private void OnDispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
    {
        // 개별 이미지 오류 등으로 프로그램 전체가 종료되지 않도록 한다.
        _logger?.LogError(e.Exception, "UI 스레드 예외");
        e.Handled = true;
        try
        {
            MessageBox.Show(
                MainWindow,
                "작업 중 오류가 발생했습니다.\n" + e.Exception.Message,
                "ImageViewer",
                MessageBoxButton.OK,
                MessageBoxImage.Error);
        }
        catch (Exception)
        {
        }
    }

    private static int RunShellCommand(CommandLineOptions options)
    {
        var shell = new ShellIntegrationService();
        try
        {
            if (options.UnregisterShell)
            {
                shell.Unregister();
            }

            if (options.RegisterShell)
            {
                shell.Register();
            }

            return 0;
        }
        catch (Exception)
        {
            return 1;
        }
    }
}
