# 처음 실행하기

**언어 / Language**: **[한국어](FIRST_LAUNCH.md)** | [English](FIRST_LAUNCH.en.md)

ImageViewer는 아직 유료 Apple 공증 인증서로 서명하지 않았습니다. 그래서 macOS에서 다운로드한 뒤 **처음 한 번** Gatekeeper 확인을 요청합니다. 아래 절차는 한 번만 하면 되고, 이후에는 평소처럼 실행됩니다.

> 확인 창의 정확한 문구와 화면은 macOS 버전에 따라 다를 수 있습니다. Windows는 Constitution v3.0.0에 따라 지원 대상에서 제외되었으며([`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6) 참고), 모바일(`iOS` / `Android`) 처음 실행 안내는 마일스톤 `007`에서 추가될 예정입니다.

## macOS (Apple Silicon · Intel 공용)

1. [Releases 페이지](https://github.com/Ezenitac-AiService/ImageViewer/releases)에서 `ImageViewer-macos-universal.dmg`를 내려받아 엽니다. Apple Silicon과 Intel Mac 모두 같은 파일을 씁니다.
2. 열린 창에서 **ImageViewer**를 **Applications** 폴더로 끌어다 놓습니다.
3. Applications에서 ImageViewer를 엽니다. "Apple이 확인할 수 없음"과 비슷한 안내가 나오면 **완료**(또는 **취소**)를 누릅니다.
4. **시스템 설정 → 개인정보 보호 및 보안**으로 가서 아래쪽의 ImageViewer 항목 옆 **그래도 열기**를 누르고, 암호나 Touch ID로 확인합니다.
5. 다시 열면 ImageViewer가 실행됩니다.

앱이 **"손상되었기 때문에 열 수 없습니다"**라고 나온다면 그것은 이 안내로 해결되는 문제가 아니라 잘못 만들어진 파일입니다. [이슈](https://github.com/Ezenitac-AiService/ImageViewer/issues)로 알려 주세요.

## Linux (하위 우선순위 빌드 전용 · 미검증)

Linux 빌드는 Constitution v3.0.0 기준 **하위 우선순위 빌드 전용·런타임 미검증** 아카이브로 제공되며, 자동 실행·창 검증을 거치지 않습니다.

```bash
tar -xzf ImageViewer-linux-x64.tar.gz
./ImageViewer/imageviewer
```

실행하려면 GTK 3(파일 대화상자)과 libxkbcommon-x11(X11 키보드) 런타임이 필요합니다. 예: Debian/Ubuntu에서 `sudo apt install libgtk-3-0 libxkbcommon-x11-0`.

## 시작이 안 될 때

ImageViewer는 시작에 실패하면 원인을 알려 주는 창을 띄우고, 진단 기록을 다음 위치에 남깁니다.

| OS | 진단 기록 위치 |
|---|---|
| macOS | `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs` |
| Linux (미검증) | `~/.config/imageviewer/logs` |

이슈를 올릴 때 이 폴더의 최근 `imageviewer-YYYY-MM-DD.log` 내용을 함께 붙여 주세요. 기록에는 전체 폴더 경로가 남지 않고 파일 이름만 남습니다.
