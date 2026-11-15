using System.Windows.Input;
using ImageViewer.Helpers;
using ImageViewer.Models;
using ImageViewer.Services;

namespace ImageViewer.Tests;

public class ImageTransformTests
{
    [Fact]
    public void Four_clockwise_rotations_return_to_identity()
    {
        var t = ImageTransform.Identity;
        for (var i = 0; i < 4; i++)
        {
            t = t.RotateClockwise();
        }

        Assert.True(t.IsIdentity);
    }

    [Fact]
    public void Opposite_operations_cancel_out()
    {
        var t = ImageTransform.Identity.FlipHorizontally().RotateClockwise();
        Assert.True(t.RotateCounterClockwise().FlipHorizontally().IsIdentity);
        Assert.True(ImageTransform.Identity.FlipVertically().FlipVertically().IsIdentity);
        Assert.True(ImageTransform.Identity.FlipHorizontally().FlipHorizontally().IsIdentity);
    }

    [Fact]
    public void Quarter_turns_swap_dimensions()
    {
        var t = ImageTransform.Identity.RotateClockwise();
        Assert.True(t.SwapsDimensions);
        Assert.Equal((300u, 400u), t.Apply(400, 300));
        Assert.Equal((400u, 300u), t.RotateClockwise().Apply(400, 300));
    }

    [Fact]
    public void Flip_vertical_equals_flip_horizontal_plus_half_turn()
    {
        var t = ImageTransform.Identity.FlipVertically();
        Assert.Equal(new ImageTransform(180, true), t);
    }
}

public class NewShortcutTests
{
    [Theory]
    [InlineData(Key.R, ShortcutAction.RotateClockwise)]
    [InlineData(Key.L, ShortcutAction.RotateCounterClockwise)]
    [InlineData(Key.H, ShortcutAction.FlipHorizontal)]
    [InlineData(Key.V, ShortcutAction.FlipVertical)]
    [InlineData(Key.Tab, ShortcutAction.ToggleInfoPanel)]
    [InlineData(Key.PageUp, ShortcutAction.PreviousPage)]
    [InlineData(Key.PageDown, ShortcutAction.NextPage)]
    public void Maps_feature_shortcuts(Key key, ShortcutAction expected) =>
        Assert.Equal(expected, KeyboardHelper.GetAction(key, Key.None, ModifierKeys.None));
}

public class ExifFormattingTests
{
    [Fact]
    public void Formats_exif_dates() =>
        Assert.Equal("2026-11-15 10:20:30", ExifReader.FormatExifDate("2026:11:15 10:20:30"));

    [Theory]
    [InlineData(1u, 250u, "1/250 초")]
    [InlineData(2u, 1u, "2 초")]
    public void Formats_exposure(uint numerator, uint denominator, string expected) =>
        Assert.Equal(expected, ExifReader.FormatExposure(new ImageMagick.Rational(numerator, denominator)));
}
