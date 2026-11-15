using ImageViewer.Models;
using ImageViewer.Services;

namespace ImageViewer.Tests;

public class ImageResizeServiceTests
{
    [Fact]
    public void Original_is_one_to_one()
    {
        var layout = ImageResizeService.Calculate(DisplayMode.Original, 1, 4000, 3000, 1000, 800);
        Assert.Equal(1.0, layout.Scale);
        Assert.Equal(4000, layout.DisplayPixelWidth);
        Assert.Equal(4000u, layout.RenderPixelWidth);
    }

    [Fact]
    public void FitWidth_matches_viewport_width_and_renders_at_display_size()
    {
        var layout = ImageResizeService.Calculate(DisplayMode.FitWidth, 1, 4000, 3000, 1000, 800);
        Assert.Equal(0.25, layout.Scale, 6);
        Assert.Equal(1000, layout.DisplayPixelWidth, 6);
        Assert.Equal(750, layout.DisplayPixelHeight, 6);
        Assert.Equal(1000u, layout.RenderPixelWidth);
        Assert.Equal(750u, layout.RenderPixelHeight);
    }

    [Fact]
    public void FitHeight_matches_viewport_height()
    {
        var layout = ImageResizeService.Calculate(DisplayMode.FitHeight, 1, 4000, 3000, 1000, 600);
        Assert.Equal(600, layout.DisplayPixelHeight, 6);
        Assert.Equal(800, layout.DisplayPixelWidth, 6);
    }

    [Fact]
    public void Fill_shows_whole_image_as_large_as_possible()
    {
        // 세로로 긴 이미지 → 높이에 맞춤
        var portrait = ImageResizeService.Calculate(DisplayMode.Fill, 1, 1000, 2000, 800, 600);
        Assert.Equal(600, portrait.DisplayPixelHeight, 6);
        Assert.Equal(300, portrait.DisplayPixelWidth, 6);

        // 가로로 긴 이미지 → 폭에 맞춤
        var landscape = ImageResizeService.Calculate(DisplayMode.Fill, 1, 2000, 1000, 800, 600);
        Assert.Equal(800, landscape.DisplayPixelWidth, 6);
        Assert.Equal(400, landscape.DisplayPixelHeight, 6);

        // 폭에 맞추면 넘치는 가로 이미지는 높이에 맞춰 축소 (잘리지 않음)
        var wide = ImageResizeService.Calculate(DisplayMode.Fill, 1, 1000, 900, 800, 600);
        Assert.Equal(600, wide.DisplayPixelHeight, 6);
        Assert.True(wide.DisplayPixelWidth <= 800);

        // 작은 이미지는 창에 맞춰 확대
        var small = ImageResizeService.Calculate(DisplayMode.Fill, 1, 200, 100, 1000, 800);
        Assert.Equal(5.0, small.Scale, 6);
    }

    [Fact]
    public void Small_images_follow_display_mode()
    {
        // 맞춤 모드는 작은 이미지도 표시 영역에 맞춰 확대한다.
        var fitWidth = ImageResizeService.Calculate(DisplayMode.FitWidth, 1, 200, 100, 1000, 800);
        Assert.Equal(5.0, fitWidth.Scale, 6);
        Assert.Equal(200u, fitWidth.RenderPixelWidth); // 확대는 화면에서 처리

        var fitHeight = ImageResizeService.Calculate(DisplayMode.FitHeight, 1, 200, 100, 1000, 800);
        Assert.Equal(8.0, fitHeight.Scale, 6);

        var fill = ImageResizeService.Calculate(DisplayMode.Fill, 1, 200, 100, 1000, 800);
        Assert.Equal(5.0, fill.Scale, 6);

        // 원본 크기는 확대하지 않는다.
        var original = ImageResizeService.Calculate(DisplayMode.Original, 1, 200, 100, 1000, 800);
        Assert.Equal(1.0, original.Scale);
    }

    [Fact]
    public void Zoom_steps_stop_at_100_percent()
    {
        // 기본 배율 0.9 → 확대 시 1.125 가 아니라 1.0 에서 멈춤
        var zoom = ImageResizeService.StepZoom(0.9, 1.0, zoomIn: true);
        Assert.Equal(1.0, 0.9 * zoom, 6);

        var next = ImageResizeService.StepZoom(0.9, zoom, zoomIn: true);
        Assert.Equal(1.25, 0.9 * next, 6);
    }

    [Fact]
    public void Zoom_is_clamped()
    {
        var zoom = 1.0;
        for (var i = 0; i < 100; i++)
        {
            zoom = ImageResizeService.StepZoom(1.0, zoom, zoomIn: false);
        }

        Assert.Equal(ImageResizeService.MinScale, zoom, 6);
    }
}

public class ImageNavigationServiceTests
{
    [Fact]
    public void Wraps_around_when_enabled()
    {
        var nav = new ImageNavigationService { WrapAround = true };
        nav.Reset(3, 2);
        Assert.Equal(0, nav.GetNextIndex());
        nav.MoveTo(0);
        Assert.Equal(2, nav.GetPreviousIndex());
    }

    [Fact]
    public void Stops_at_edges_when_wrap_disabled()
    {
        var nav = new ImageNavigationService { WrapAround = false };
        nav.Reset(3, 2);
        Assert.Equal(-1, nav.GetNextIndex());
        nav.MoveTo(0);
        Assert.Equal(-1, nav.GetPreviousIndex());
        Assert.Equal(1, nav.GetNextIndex());
    }

    [Fact]
    public void Empty_list_has_no_targets()
    {
        var nav = new ImageNavigationService();
        nav.Reset(0);
        Assert.Equal(-1, nav.CurrentIndex);
        Assert.Equal(-1, nav.GetNextIndex());
        Assert.Equal(-1, nav.GetFirstIndex());
        Assert.False(nav.MoveTo(0));
    }
}

public class SlideShowServiceTests
{
    [Fact]
    public void Sequential_repeats_from_start()
    {
        var s = new SlideShowService(new Random(1)) { Mode = SlideShowMode.Sequential, Repeat = true };
        Assert.Equal(1, s.GetNextIndex(0, 3));
        Assert.Equal(0, s.GetNextIndex(2, 3));
    }

    [Fact]
    public void Sequential_ends_without_repeat()
    {
        var s = new SlideShowService(new Random(1)) { Mode = SlideShowMode.Sequential, Repeat = false };
        Assert.Null(s.GetNextIndex(2, 3));
    }

    [Fact]
    public void Random_never_repeats_previous_image()
    {
        var s = new SlideShowService(new Random(42)) { Mode = SlideShowMode.Random, Repeat = true };
        var current = 0;
        for (var i = 0; i < 500; i++)
        {
            var next = s.GetNextIndex(current, 5);
            Assert.NotNull(next);
            Assert.NotEqual(current, next.Value);
            Assert.InRange(next.Value, 0, 4);
            current = next.Value;
        }
    }

    [Fact]
    public void Random_without_repeat_shows_each_image_once()
    {
        var s = new SlideShowService(new Random(7)) { Mode = SlideShowMode.Random, Repeat = false };
        var seen = new HashSet<int> { 3 };
        var current = 3;
        while (s.GetNextIndex(current, 10) is { } next)
        {
            Assert.True(seen.Add(next), $"{next} 이(가) 중복 재생됨");
            current = next;
        }

        Assert.Equal(10, seen.Count);
    }

    [Theory]
    [InlineData(SlideShowMode.Random, true)]
    [InlineData(SlideShowMode.Random, false)]
    [InlineData(SlideShowMode.Sequential, true)]
    public void Peek_predicts_next_tick_without_consuming(SlideShowMode mode, bool repeat)
    {
        var s = new SlideShowService(new Random(3)) { Mode = mode, Repeat = repeat };
        var current = 4;
        for (var i = 0; i < 40; i++)
        {
            var peeked = s.PeekNextIndex(current, 8);
            Assert.Equal(peeked, s.PeekNextIndex(current, 8)); // 여러 번 봐도 같다
            var next = s.GetNextIndex(current, 8);
            Assert.Equal(peeked, next);                       // 다음 타이머가 실제로 같은 이미지를 보여 준다
            if (next is null)
            {
                break;
            }

            current = next.Value;
        }
    }

    [Fact]
    public void Peek_skips_image_reached_by_manual_navigation()
    {
        var s = new SlideShowService(new Random(5)) { Mode = SlideShowMode.Random, Repeat = true };
        var peeked = s.PeekNextIndex(0, 6)!.Value;

        // 사용자가 미리 본 이미지로 직접 이동하면, 다음 타이머는 그 이미지를 다시 보여 주지 않는다.
        var next = s.GetNextIndex(peeked, 6);
        Assert.NotNull(next);
        Assert.NotEqual(peeked, next);
    }

    [Fact]
    public void Interval_is_clamped_to_5_to_90_seconds()
    {
        var s = new SlideShowService { IntervalSeconds = 1 };
        Assert.Equal(5, s.IntervalSeconds);
        s.IntervalSeconds = 1000;
        Assert.Equal(90, s.IntervalSeconds);
    }
}

public class SettingsServiceTests : IDisposable
{
    private readonly string _dir = Path.Combine(Path.GetTempPath(), "iv-settings-" + Guid.NewGuid());

    private string SettingsFile => Path.Combine(_dir, "settings.json");

    [Fact]
    public void Missing_file_uses_defaults()
    {
        var service = new SettingsService(settingsPath: SettingsFile);
        var settings = service.Load();
        Assert.Equal(DisplayMode.FitWidth, settings.Display.Mode);
        Assert.Equal(5, settings.Slideshow.IntervalSeconds);
        Assert.True(settings.Navigation.WrapAround);
    }

    [Fact]
    public void Corrupt_file_uses_defaults()
    {
        Directory.CreateDirectory(_dir);
        File.WriteAllText(SettingsFile, "{ this is not json");
        var settings = new SettingsService(settingsPath: SettingsFile).Load();
        Assert.Equal(96, settings.Thumbnail.Size);
    }

    [Fact]
    public void Round_trips_and_normalizes()
    {
        var service = new SettingsService(settingsPath: SettingsFile);
        service.Load();
        service.Current.Display.Mode = DisplayMode.Fill;
        service.Current.Slideshow.Mode = SlideShowMode.Random;
        service.Current.Slideshow.IntervalSeconds = 30;
        service.Save();

        var json = File.ReadAllText(SettingsFile);
        Assert.Contains("\"mode\": \"Fill\"", json);

        File.WriteAllText(SettingsFile, json.Replace("\"intervalSeconds\": 30", "\"intervalSeconds\": 500"));
        var reloaded = new SettingsService(settingsPath: SettingsFile).Load();
        Assert.Equal(DisplayMode.Fill, reloaded.Display.Mode);
        Assert.Equal(SlideShowMode.Random, reloaded.Slideshow.Mode);
        Assert.Equal(90, reloaded.Slideshow.IntervalSeconds);
    }

    [Fact]
    public void Reads_plan_sample_format()
    {
        Directory.CreateDirectory(_dir);
        File.WriteAllText(SettingsFile, """
            {
              "window": { "width": 1280, "height": 800, "left": 100, "top": 100, "isMaximized": false },
              "display": { "mode": "FitWidth", "upscaleSmallImages": false, "background": "Black" },
              "navigation": { "wrapAround": true },
              "slideshow": { "intervalSeconds": 5, "mode": "Sequential", "repeat": true },
              "thumbnail": { "visible": true, "size": 96 }
            }
            """);
        var settings = new SettingsService(settingsPath: SettingsFile).Load();
        Assert.Equal(100, settings.Window.Left);
        Assert.Equal(ViewerBackground.Black, settings.Display.Background);
        Assert.NotNull(settings.Filter);
    }

    [Fact]
    public void Old_settings_switch_wheel_to_navigation_once()
    {
        Directory.CreateDirectory(_dir);

        // 버전 정보 없는 이전 설정: 휠 기본값이 확대/축소였으므로 이전/다음으로 바꾼다.
        File.WriteAllText(SettingsFile, """{ "display": { "mouseWheel": "Zoom" } }""");
        var migrated = new SettingsService(settingsPath: SettingsFile).Load();
        Assert.Equal(MouseWheelAction.Navigate, migrated.Display.MouseWheel);
        Assert.Equal(ViewerSettings.CurrentVersion, migrated.Version);

        // 새 버전에서 사용자가 직접 고른 확대/축소는 유지한다.
        File.WriteAllText(SettingsFile, """{ "version": 2, "display": { "mouseWheel": "Zoom" } }""");
        var kept = new SettingsService(settingsPath: SettingsFile).Load();
        Assert.Equal(MouseWheelAction.Zoom, kept.Display.MouseWheel);
    }

    public void Dispose()
    {
        if (Directory.Exists(_dir))
        {
            Directory.Delete(_dir, true);
        }
    }
}

public class LruCacheTests
{
    [Fact]
    public void Evicts_least_recently_used_by_count()
    {
        var evicted = new List<string>();
        var cache = new LruCache<string, int>(2, long.MaxValue, _ => 1, (k, _) => evicted.Add(k));
        cache.Add("a", 1);
        cache.Add("b", 2);
        Assert.True(cache.TryGet("a", out _));
        cache.Add("c", 3);

        Assert.Equal(["b"], evicted);
        Assert.False(cache.TryGet("b", out _));
        Assert.True(cache.TryGet("a", out _));
    }

    [Fact]
    public void Evicts_by_cost_but_keeps_newest_item()
    {
        var cache = new LruCache<string, int>(100, 10, v => v);
        cache.Add("a", 6);
        cache.Add("b", 6);
        Assert.Equal(1, cache.Count);
        Assert.True(cache.TryGet("b", out _));

        cache.Add("huge", 50);
        Assert.True(cache.TryGet("huge", out _));
    }
}
