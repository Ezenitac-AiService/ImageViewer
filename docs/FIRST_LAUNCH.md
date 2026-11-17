# 처음 실행하기

**언어 / Language**: **[한국어](FIRST_LAUNCH.md)** | [English](FIRST_LAUNCH.en.md)

ImageViewer는 아직 유료 게시자 인증서로 서명하지 않았습니다. 그래서 다운로드한 뒤 **처음 한 번** 운영체제가 확인을 요청합니다. 아래 절차는 한 번만 하면 되고, 이후에는 평소처럼 실행됩니다.

> 확인 창의 정확한 문구와 화면은 OS 버전에 따라 다를 수 있습니다. 릴리스 전 깨끗한 환경 검증에서 찍은 화면을 이 문서에 추가할 예정입니다(작업 T110).

## Windows 10 / 11

1. [Releases 페이지](https://github.com/Ezenitac-AiService/ImageViewer/releases)에서 `ImageViewer-windows-x64.exe`를 내려받습니다. 설치 없이 바로 실행하는 단일 파일입니다.
2. 파일을 실행하면 **"Windows의 PC 보호"** 창이 나타날 수 있습니다.
3. **추가 정보**를 누른 뒤 **실행**을 누릅니다.
4. ImageViewer 창이 열립니다. 별도의 검은 콘솔 창은 나타나지 않습니다.

추가로 설치할 런타임(Visual C++ 재배포 패키지 등)은 없습니다.

**스마트 앱 컨트롤**이 켜져 있는 PC에서는 서명되지 않은 앱을 실행할 방법이 없습니다. 이 경우 게시자 서명이 있는 릴리스가 나올 때까지 기다려 주세요.

## macOS (Apple Silicon · Intel 공용)

1. `ImageViewer-macos-universal.dmg`를 내려받아 엽니다. Apple Silicon과 Intel Mac 모두 같은 파일을 씁니다.
2. 열린 창에서 **ImageViewer**를 **Applications** 폴더로 끌어다 놓습니다.
3. Applications에서 ImageViewer를 엽니다. "Apple이 확인할 수 없음"과 비슷한 안내가 나오면 **완료**(또는 **취소**)를 누릅니다.
4. **시스템 설정 → 개인정보 보호 및 보안**으로 가서 아래쪽의 ImageViewer 항목 옆 **그래도 열기**를 누르고, 암호나 Touch ID로 확인합니다.
5. 다시 열면 ImageViewer가 실행됩니다.

앱이 **"손상되었기 때문에 열 수 없습니다"**라고 나온다면 그것은 이 안내로 해결되는 문제가 아니라 잘못 만들어진 파일입니다. [이슈](https://github.com/Ezenitac-AiService/ImageViewer/issues)로 알려 주세요.

## Linux (실험적)

Linux 빌드는 자동 검사를 통과하지만 실행 검증을 거치지 않은 **실험적** 빌드입니다.

```bash
tar -xzf ImageViewer-linux-x64.tar.gz
./ImageViewer/imageviewer
```

파일 대화상자를 쓰려면 GTK 3 런타임이 필요합니다.

## 시작이 안 될 때

ImageViewer는 시작에 실패하면 원인을 알려 주는 창을 띄우고, 진단 기록을 다음 위치에 남깁니다.

| OS | 진단 기록 위치 |
|---|---|
| Windows | `%LOCALAPPDATA%\ImageViewer\logs` |
| macOS | `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs` |
| Linux | `~/.config/imageviewer/logs` |

이슈를 올릴 때 이 폴더의 최근 `imageviewer-YYYY-MM-DD.log` 내용을 함께 붙여 주세요. 기록에는 전체 폴더 경로가 남지 않고 파일 이름만 남습니다.
