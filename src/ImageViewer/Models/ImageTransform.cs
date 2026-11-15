using ImageMagick;

namespace ImageViewer.Models;

/// <summary>
/// 화면 표시용 회전 · 반전 상태. 원본 파일은 변경하지 않는다.
/// 변환은 "시계 방향 <see cref="Rotation"/>도 회전 후, 필요하면 좌우 반전" 순서로 정규화해 보관한다.
/// 사용자가 화면 기준으로 한 조작(회전 · 좌우 반전 · 상하 반전)을 이 형태로 합성한다.
/// </summary>
public readonly record struct ImageTransform(int Rotation, bool FlipHorizontal)
{
    public static ImageTransform Identity => default;

    public bool IsIdentity => Rotation == 0 && !FlipHorizontal;

    /// <summary>90° / 270° 회전이면 표시 폭과 높이가 바뀐다.</summary>
    public bool SwapsDimensions => Rotation is 90 or 270;

    /// <summary>화면 기준 시계 방향 90° 회전.</summary>
    public ImageTransform RotateClockwise() =>
        this with { Rotation = Normalize(Rotation + (FlipHorizontal ? -90 : 90)) };

    /// <summary>화면 기준 반시계 방향 90° 회전.</summary>
    public ImageTransform RotateCounterClockwise() =>
        this with { Rotation = Normalize(Rotation + (FlipHorizontal ? 90 : -90)) };

    /// <summary>화면 기준 좌우 반전.</summary>
    public ImageTransform FlipHorizontally() => this with { FlipHorizontal = !FlipHorizontal };

    /// <summary>화면 기준 상하 반전 (= 좌우 반전 + 180° 회전).</summary>
    public ImageTransform FlipVertically() => new(Normalize(Rotation + 180), !FlipHorizontal);

    /// <summary>회전 · 반전을 적용한 뒤의 (폭, 높이).</summary>
    public (uint Width, uint Height) Apply(uint width, uint height) =>
        SwapsDimensions ? (height, width) : (width, height);

    public void Apply(MagickImage image)
    {
        if (Rotation != 0)
        {
            image.Rotate(Rotation);
        }

        if (FlipHorizontal)
        {
            image.Flop();
        }
    }

    public string Describe()
    {
        if (IsIdentity)
        {
            return "회전 · 반전 없음";
        }

        var parts = new List<string>();
        if (Rotation != 0)
        {
            parts.Add($"{Rotation}° 회전");
        }

        if (FlipHorizontal)
        {
            parts.Add("좌우 반전");
        }

        return string.Join(" · ", parts);
    }

    private static int Normalize(int degrees) => ((degrees % 360) + 360) % 360;
}
