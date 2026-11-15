namespace ImageViewer.Models;

/// <summary>
/// 이미지 목록의 항목 정보. 일반 파일과 ZIP 내부 항목을 동일한 형태로 표현한다.
/// </summary>
public sealed class ImageInfo
{
    public ImageInfo(
        string name,
        ImageSourceType sourceType,
        string? fullPath,
        string? entryPath,
        long size,
        DateTime? modifiedAt,
        bool isSupported = true,
        string? unsupportedReason = null)
    {
        Name = name;
        SourceType = sourceType;
        FullPath = fullPath;
        EntryPath = entryPath;
        Size = size;
        ModifiedAt = modifiedAt;
        IsSupported = isSupported;
        UnsupportedReason = unsupportedReason;
    }

    /// <summary>파일명 (ZIP 항목은 마지막 경로 요소).</summary>
    public string Name { get; }

    /// <summary>일반 파일의 전체 경로. ZIP 항목은 ZIP 파일 경로.</summary>
    public string? FullPath { get; }

    /// <summary>ZIP 내부 항목 경로. 일반 파일은 null.</summary>
    public string? EntryPath { get; }

    /// <summary>파일 크기 (ZIP 항목은 압축 해제 크기).</summary>
    public long Size { get; }

    public DateTime? ModifiedAt { get; }

    public ImageSourceType SourceType { get; }

    public bool IsSupported { get; }

    public string? UnsupportedReason { get; }

    /// <summary>목록·정렬에 사용하는 표시 경로 (ZIP은 내부 경로, 파일은 파일명).</summary>
    public string DisplayPath => EntryPath ?? Name;

    /// <summary>캐시 키로 사용하는 고유 식별자.</summary>
    public string Key => SourceType == ImageSourceType.Zip
        ? $"zip|{FullPath}|{EntryPath}"
        : $"file|{FullPath}";

    public override string ToString() => DisplayPath;
}
