using ImageViewer.Models;

namespace ImageViewer.Services.ImageSources;

/// <summary>
/// 일반 폴더와 ZIP 을 동일하게 다루기 위한 이미지 소스 추상화.
/// </summary>
public interface IImageSource : IDisposable
{
    ImageSourceInfo Info { get; }

    /// <summary>소스 내부의 이미지 목록을 파일명 오름차순으로 반환한다.</summary>
    Task<IReadOnlyList<ImageInfo>> GetImagesAsync(CancellationToken cancellationToken);

    /// <summary>
    /// 이미지 데이터를 읽을 수 있는 스트림을 연다. 호출자가 스트림을 소유하고 해제한다.
    /// 반환되는 스트림은 탐색(Seek) 가능하다.
    /// </summary>
    Task<Stream> OpenImageAsync(ImageInfo image, CancellationToken cancellationToken);
}

/// <summary>이미지 소스를 열거나 항목을 읽는 중 발생한 사용자 안내용 오류.</summary>
public sealed class ImageSourceException : Exception
{
    public ImageSourceException(string message, Exception? inner = null)
        : base(message, inner)
    {
    }
}
