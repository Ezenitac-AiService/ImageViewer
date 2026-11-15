namespace ImageViewer.Models;

/// <summary>현재 열려 있는 이미지 소스(폴더 또는 ZIP) 정보.</summary>
public sealed record ImageSourceInfo(string Path, ImageSourceType SourceType)
{
    public string DisplayName
    {
        get
        {
            var trimmed = System.IO.Path.TrimEndingDirectorySeparator(Path);
            var name = System.IO.Path.GetFileName(trimmed);
            return string.IsNullOrEmpty(name) ? Path : name;
        }
    }

    public string Label => SourceType == ImageSourceType.Zip ? "ZIP" : "폴더";
}
