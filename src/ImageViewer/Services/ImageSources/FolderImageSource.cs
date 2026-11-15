using ImageViewer.Helpers;
using ImageViewer.Models;

namespace ImageViewer.Services.ImageSources;

/// <summary>일반 폴더의 이미지 파일 (하위 폴더는 포함하지 않음).</summary>
public sealed class FolderImageSource : IImageSource
{
    private readonly string _folder;
    private readonly Lock _watchLock = new();
    private FileSystemWatcher? _watcher;

    /// <summary>
    /// 감시 중인 폴더에서 이미지 파일이 추가 · 삭제 · 이름 변경 · 수정되었을 때 발생한다.
    /// 백그라운드 스레드에서 짧은 간격으로 여러 번 발생할 수 있으므로 받는 쪽에서 모아서 처리한다.
    /// </summary>
    public event EventHandler? Changed;

    public FolderImageSource(string folder)
    {
        _folder = folder;
        Info = new ImageSourceInfo(folder, ImageSourceType.File);
    }

    public ImageSourceInfo Info { get; }

    public Task<IReadOnlyList<ImageInfo>> GetImagesAsync(CancellationToken cancellationToken) =>
        Task.Run<IReadOnlyList<ImageInfo>>(() =>
        {
            if (!Directory.Exists(_folder))
            {
                throw new ImageSourceException("폴더를 찾을 수 없습니다.");
            }

            var list = new List<ImageInfo>();
            try
            {
                var dir = new DirectoryInfo(_folder);
                var options = new EnumerationOptions
                {
                    IgnoreInaccessible = true,
                    RecurseSubdirectories = false,
                    AttributesToSkip = FileAttributes.Hidden | FileAttributes.System,
                };

                foreach (var file in dir.EnumerateFiles("*", options))
                {
                    cancellationToken.ThrowIfCancellationRequested();
                    if (!ImageFormatHelper.IsSupportedExtension(file.Name))
                    {
                        continue;
                    }

                    list.Add(new ImageInfo(
                        file.Name,
                        ImageSourceType.File,
                        file.FullName,
                        entryPath: null,
                        file.Length,
                        file.LastWriteTime));
                }
            }
            catch (UnauthorizedAccessException ex)
            {
                throw new ImageSourceException("폴더에 접근할 권한이 없습니다.", ex);
            }
            catch (IOException ex)
            {
                throw new ImageSourceException("폴더를 읽는 중 오류가 발생했습니다.", ex);
            }

            list.Sort((a, b) => NaturalStringComparer.Instance.Compare(a.Name, b.Name));
            return list;
        }, cancellationToken);

    public Task<Stream> OpenImageAsync(ImageInfo image, CancellationToken cancellationToken)
    {
        if (image.FullPath is null)
        {
            throw new ArgumentException("파일 경로가 없습니다.", nameof(image));
        }

        try
        {
            Stream stream = new FileStream(
                image.FullPath,
                FileMode.Open,
                FileAccess.Read,
                FileShare.ReadWrite | FileShare.Delete,
                bufferSize: 81920,
                FileOptions.SequentialScan);
            return Task.FromResult(stream);
        }
        catch (FileNotFoundException ex)
        {
            throw new ImageSourceException("파일을 찾을 수 없습니다.", ex);
        }
        catch (UnauthorizedAccessException ex)
        {
            throw new ImageSourceException("파일에 접근할 권한이 없습니다.", ex);
        }
        catch (IOException ex)
        {
            throw new ImageSourceException("파일을 열 수 없습니다.", ex);
        }
    }

    /// <summary>폴더 감시를 시작한다 (하위 폴더 제외).</summary>
    public void StartWatching()
    {
        lock (_watchLock)
        {
            if (_watcher is not null)
            {
                return;
            }

            var watcher = new FileSystemWatcher(_folder)
            {
                IncludeSubdirectories = false,
                NotifyFilter = NotifyFilters.FileName | NotifyFilters.LastWrite | NotifyFilters.Size,
                InternalBufferSize = 64 * 1024,
            };
            watcher.Created += OnFileEvent;
            watcher.Deleted += OnFileEvent;
            watcher.Changed += OnFileEvent;
            watcher.Renamed += OnRenamed;
            watcher.Error += OnWatcherError;
            watcher.EnableRaisingEvents = true;
            _watcher = watcher;
        }
    }

    public void StopWatching()
    {
        FileSystemWatcher? watcher;
        lock (_watchLock)
        {
            watcher = _watcher;
            _watcher = null;
        }

        if (watcher is not null)
        {
            watcher.EnableRaisingEvents = false;
            watcher.Dispose();
        }
    }

    private void OnFileEvent(object sender, FileSystemEventArgs e)
    {
        if (ImageFormatHelper.IsSupportedExtension(e.Name))
        {
            Changed?.Invoke(this, EventArgs.Empty);
        }
    }

    private void OnRenamed(object sender, RenamedEventArgs e)
    {
        if (ImageFormatHelper.IsSupportedExtension(e.Name) || ImageFormatHelper.IsSupportedExtension(e.OldName))
        {
            Changed?.Invoke(this, EventArgs.Empty);
        }
    }

    /// <summary>이벤트가 너무 많아 버퍼가 넘친 경우 등: 목록을 전부 다시 읽게 한다.</summary>
    private void OnWatcherError(object sender, ErrorEventArgs e) => Changed?.Invoke(this, EventArgs.Empty);

    public void Dispose() => StopWatching();
}
