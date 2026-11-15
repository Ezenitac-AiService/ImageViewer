using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Input;

namespace ImageViewer.Helpers;

/// <summary>메인 화면 단축키 정의 (구현 계획서 7장).</summary>
public enum ShortcutAction
{
    None,
    ToggleFullScreen,
    Exit,
    ToggleMenus,
    Previous,
    Next,
    OpenImageList,
    ModeOriginal,
    ModeFitWidth,
    ModeFitHeight,
    ModeFill,
    FilterNone,
    FilterSmooth,
    FilterSmoothSharpen,
    ZoomIn,
    ZoomOut,
    ToggleSlideShow,
    First,
    Last,
    PreviousPage,
    NextPage,
    RotateClockwise,
    RotateCounterClockwise,
    FlipHorizontal,
    FlipVertical,
    ToggleInfoPanel,
}

public static class KeyboardHelper
{
    public static ShortcutAction GetAction(Key key, Key systemKey, ModifierKeys modifiers)
    {
        // Alt + Enter 는 WPF 에서 Key.System + SystemKey.Enter 로 들어온다.
        if (key == Key.System)
        {
            return systemKey is Key.Enter or Key.Return && modifiers.HasFlag(ModifierKeys.Alt)
                ? ShortcutAction.ToggleFullScreen
                : ShortcutAction.None;
        }

        if (modifiers.HasFlag(ModifierKeys.Alt) && key is Key.Enter or Key.Return)
        {
            return ShortcutAction.ToggleFullScreen;
        }

        // Ctrl/Alt 조합은 메뉴 등 다른 기능에 양보한다.
        if (modifiers.HasFlag(ModifierKeys.Control) || modifiers.HasFlag(ModifierKeys.Alt))
        {
            return ShortcutAction.None;
        }

        return key switch
        {
            Key.Escape => ShortcutAction.Exit,
            Key.Oem3 => ShortcutAction.ToggleMenus, // ` (백틱)
            Key.Left or Key.Up or Key.Back => ShortcutAction.Previous,
            Key.Right or Key.Down => ShortcutAction.Next,
            Key.PageUp => ShortcutAction.PreviousPage,   // 다중 페이지면 페이지, 아니면 이미지 이동
            Key.PageDown => ShortcutAction.NextPage,
            Key.R => ShortcutAction.RotateClockwise,
            Key.L => ShortcutAction.RotateCounterClockwise,
            Key.H => ShortcutAction.FlipHorizontal,
            Key.V => ShortcutAction.FlipVertical,
            Key.Tab => ShortcutAction.ToggleInfoPanel,
            Key.Home => ShortcutAction.First,
            Key.End => ShortcutAction.Last,
            Key.Enter or Key.Return => ShortcutAction.OpenImageList,
            Key.D0 or Key.NumPad0 => ShortcutAction.ModeOriginal,
            Key.D7 or Key.NumPad7 => ShortcutAction.ModeFitWidth,
            Key.D8 or Key.NumPad8 => ShortcutAction.ModeFitHeight,
            Key.D9 or Key.NumPad9 => ShortcutAction.ModeFill,
            Key.U => ShortcutAction.FilterNone,
            Key.I => ShortcutAction.FilterSmooth,
            Key.S => ShortcutAction.FilterSmoothSharpen,
            Key.OemPlus or Key.Add => ShortcutAction.ZoomIn,
            Key.OemMinus or Key.Subtract => ShortcutAction.ZoomOut,
            Key.Space => ShortcutAction.ToggleSlideShow,
            _ => ShortcutAction.None,
        };
    }

    /// <summary>
    /// 텍스트 입력이나 값 조절 컨트롤에 포커스가 있으면 메인 단축키를 적용하지 않는다.
    /// </summary>
    public static bool IsInputControlFocused()
    {
        var focused = Keyboard.FocusedElement as DependencyObject;
        return focused is TextBoxBase or PasswordBox or ComboBox or RangeBase ||
               (focused is FrameworkElement fe && fe.TemplatedParent is ComboBox);
    }
}
