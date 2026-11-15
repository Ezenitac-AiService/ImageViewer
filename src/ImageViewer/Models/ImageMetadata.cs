namespace ImageViewer.Models;

/// <summary>정보 패널의 한 줄. <see cref="IsHeader"/> 이면 구역 제목.</summary>
public sealed record MetadataItem(string Label, string Value, bool IsHeader = false)
{
    public static MetadataItem Header(string title) => new(title, string.Empty, IsHeader: true);
}
