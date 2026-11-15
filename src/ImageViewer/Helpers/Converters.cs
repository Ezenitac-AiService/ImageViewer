using System.Globalization;
using System.Windows;
using System.Windows.Data;
using System.Windows.Media;
using ImageViewer.Models;

namespace ImageViewer.Helpers;

/// <summary>열거형 값이 매개변수와 같으면 true (메뉴 선택 상태 표시용).</summary>
public sealed class EnumEqualsConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is null || parameter is null)
        {
            return false;
        }

        var text = parameter.ToString();
        return string.Equals(value.ToString(), text, StringComparison.OrdinalIgnoreCase);
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is true && parameter is not null ? Enum.Parse(targetType, parameter.ToString()!) : Binding.DoNothing;
}

public sealed class BoolToVisibilityConverter : IValueConverter
{
    public bool Invert { get; set; }

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var flag = value switch
        {
            bool b => b,
            string s => !string.IsNullOrEmpty(s),
            null => false,
            _ => true,
        };

        if (Invert)
        {
            flag = !flag;
        }

        return flag ? Visibility.Visible : Visibility.Collapsed;
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        Binding.DoNothing;
}

public sealed class InverseBoolConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) => value is not true;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => value is not true;
}

/// <summary>뷰어 배경 설정을 브러시로 변환한다. 투명 이미지는 이 배경 위에 표시된다.</summary>
public sealed class BackgroundToBrushConverter : IValueConverter
{
    private static readonly Brush Checkerboard = CreateCheckerboard();

    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is ViewerBackground background ? GetBrush(background) : Brushes.Black;

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        Binding.DoNothing;

    public static Brush GetBrush(ViewerBackground background) => background switch
    {
        ViewerBackground.DarkGray => Frozen(new SolidColorBrush(Color.FromRgb(0x20, 0x20, 0x20))),
        ViewerBackground.Gray => Frozen(new SolidColorBrush(Color.FromRgb(0x80, 0x80, 0x80))),
        ViewerBackground.White => Brushes.White,
        ViewerBackground.Checkerboard => Checkerboard,
        _ => Brushes.Black,
    };

    private static Brush Frozen(Brush brush)
    {
        brush.Freeze();
        return brush;
    }

    private static Brush CreateCheckerboard()
    {
        var light = new SolidColorBrush(Color.FromRgb(0xCC, 0xCC, 0xCC));
        var dark = new SolidColorBrush(Color.FromRgb(0x99, 0x99, 0x99));
        var group = new DrawingGroup();
        group.Children.Add(new GeometryDrawing(light, null, new RectangleGeometry(new Rect(0, 0, 16, 16))));
        group.Children.Add(new GeometryDrawing(dark, null, new RectangleGeometry(new Rect(0, 0, 8, 8))));
        group.Children.Add(new GeometryDrawing(dark, null, new RectangleGeometry(new Rect(8, 8, 8, 8))));
        var brush = new DrawingBrush(group)
        {
            TileMode = TileMode.Tile,
            Viewport = new Rect(0, 0, 16, 16),
            ViewportUnits = BrushMappingMode.Absolute,
        };
        brush.Freeze();
        return brush;
    }
}

/// <summary>배경색에 맞춰 안내 문구 색을 정한다.</summary>
public sealed class BackgroundToForegroundConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        value is ViewerBackground.White or ViewerBackground.Checkerboard or ViewerBackground.Gray
            ? Brushes.Black
            : new SolidColorBrush(Color.FromRgb(0xC8, 0xC8, 0xC8));

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
        Binding.DoNothing;
}
