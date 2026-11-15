using ImageViewer.Helpers;

namespace ImageViewer.Services.ImageSources;

public sealed record ImageSourceOpenResult(IImageSource Source, string? InitialFileName);

public interface IImageSourceFactory
{
    /// <summary>
    /// 폴더, ZIP, 또는 이미지 파일 경로로부터 이미지 소스를 만든다.
    /// 이미지 파일이면 해당 파일이 있는 폴더를 열고 그 파일을 처음에 선택한다.
    /// </summary>
    ImageSourceOpenResult Create(string path);
}

public sealed class ImageSourceFactory(SettingsService settingsService) : IImageSourceFactory
{
    public ImageSourceOpenResult Create(string path)
    {
        if (Directory.Exists(path))
        {
            return new ImageSourceOpenResult(new FolderImageSource(path), null);
        }

        if (!File.Exists(path))
        {
            throw new ImageSourceException("경로를 찾을 수 없습니다.");
        }

        if (ImageFormatHelper.IsZipFile(path))
        {
            var maxBytes = (long)settingsService.Current.Limits.MaxZipEntryMegabytes * 1024 * 1024;
            return new ImageSourceOpenResult(new ZipImageSource(path, maxBytes), null);
        }

        if (ImageFormatHelper.IsSupportedExtension(path))
        {
            var folder = Path.GetDirectoryName(path)
                         ?? throw new ImageSourceException("폴더 경로를 확인할 수 없습니다.");
            return new ImageSourceOpenResult(new FolderImageSource(folder), Path.GetFileName(path));
        }

        throw new ImageSourceException("지원하지 않는 파일 형식입니다. 폴더, ZIP 또는 이미지 파일을 열어 주세요.");
    }
}
