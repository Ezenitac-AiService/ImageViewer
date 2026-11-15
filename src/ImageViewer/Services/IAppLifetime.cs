using System.Diagnostics;

namespace ImageViewer.Services;

/// <summary>앱 종료 (업데이트를 적용할 때 사용).</summary>
public interface IAppLifetime
{
    /// <summary>앱을 정상 종료(설정 저장 포함)한 뒤 <paramref name="afterExit"/> 프로세스를 실행한다.</summary>
    void ExitAndRun(ProcessStartInfo afterExit);
}
