# ImageViewer

**언어 / Language**: **[한국어](README.md)** | [English](README.en.md)

[![CI](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml)
[![Release Build](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Ezenitac-AiService/ImageViewer?include_prereleases&sort=semver&label=release&logo=github)](https://github.com/Ezenitac-AiService/ImageViewer/releases)
[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust: 1.95+ (Edition 2024)](https://img.shields.io/badge/rust-1.95%2B%20(2024)-orange.svg?logo=rust)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20(experimental)-lightgrey.svg)](#지원-플랫폼-및-릴리스)
[![Upstream Origin](https://img.shields.io/badge/upstream%20origin-edith2580%2FImageViewer%20(v1.0.6)-informational?logo=github)](https://github.com/edith2580/ImageViewer)

**Windows**와 **macOS**용 이미지·만화 압축 파일(`.zip`/`.cbz`) 뷰어입니다. Linux 빌드는 실험적으로 제공합니다. Rust로 작성했고 `eframe`/`egui`와 `wgpu` 위에서 동작합니다.

> [!IMPORTANT]
> **처음 실행 안내**: 아직 게시자 인증서로 서명하지 않아, 다운로드 후 처음 한 번은 OS 확인이 필요합니다. [처음 실행하기](docs/FIRST_LAUNCH.md)를 따라 주세요.

> [!NOTE]
> **독립 크로스 플랫폼 포크 안내 및 오픈소스 예절 (Fork Etiquette)**
>
> 본 저장소(**[Ezenitac-AiService/ImageViewer](https://github.com/Ezenitac-AiService/ImageViewer)**)는 **[edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)**(`v1.0.6`, Windows C#/.NET)에서 파생된 **독립 크로스 플랫폼 Rust 포크 프로젝트**로, 원작자(`edith2580`)의 저작권과 `GPL-3.0-only` 라이선스를 존중하며 계승합니다.
>
> - **레거시 아카이브**: 전환 이전의 Windows C#/.NET `v1.0.6` 코드는 [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6) 태그에 보존되어 있습니다.
> - **이슈 등록 예절**: **크로스 플랫폼 Rust 버전에 관한 버그 제보나 기능 요청을 업스트림(`edith2580/ImageViewer`) 원작자에게 등록하지 말아 주세요.** 모든 이슈와 문의는 [Ezenitac-AiService/ImageViewer/issues](https://github.com/Ezenitac-AiService/ImageViewer/issues)에 등록해 주세요.
> - **원본 Windows 전용 C#/.NET 버전이 필요하신가요?** [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)를 방문해 주세요.

---

## 업스트림(`v1.0.6`)과의 차이점

| 구분 | 업스트림 (`edith2580/ImageViewer` `v1.0.6`) | 독립 포크 (`Ezenitac-AiService/ImageViewer`) |
|---|---|---|
| **지원 OS** | Windows (`x64`) | **Windows 10/11** (`x64`), **macOS** (Apple Silicon·Intel 공용), Linux `x64`(실험적) |
| **기술 스택** | C# / .NET WPF + WIC | Rust (Edition 2024) + `eframe`/`egui` + `wgpu` (OpenGL 대체 경로 포함) |
| **포맷** | WIC가 지원하는 포맷 전반 | 이번 버전은 검증된 핵심 포맷만 제공 ([기능 목록](docs/CAPABILITIES.md)) |
| **업데이트** | 자동 업데이트 | 이번 버전에는 없음. 앱은 네트워크에 접속하지 않습니다 |
| **설정 파일** | `%LOCALAPPDATA%\ImageViewer\settings.json` | 같은 경로·형식 유지, `v1.0.6` 설정 자동 이전 |

---

## 이번 버전에서 제공하는 기능

검증 근거와 함께 [기능 목록](docs/CAPABILITIES.md)에 정리되어 있습니다. 요약하면 다음과 같습니다.

- **이미지**: JPEG, PNG, GIF, WebP, BMP, TIFF. 파일 이름이 아니라 내용으로 형식을 판별합니다.
- **애니메이션**: GIF·WebP 재생
- **압축 파일**: `.zip` / `.cbz` 읽기 전용 열람
- **열기**: 파일·폴더 대화상자, 끌어다 놓기, 실행 인수, macOS Finder "다른 앱으로 열기"
- **보기**: 확대·축소·이동, 맞춤 모드, 회전·반전, 전체 화면
- **슬라이드쇼, 이미지 정보(EXIF), 여러 페이지 TIFF, 이미지 목록**: Extended 후보이며, 릴리스 전 검증을 통과해야 유지됩니다
- **안전 장치**: 이미지 크기 상한(기본 150MP), 읽기 전 파일 크기 상한(512MB), 압축 파일 경로 이탈·심볼릭 링크·압축 폭탄·암호화 항목 차단
- **언어**: 한국어·영어. 처음 실행할 때 OS 언어를 따릅니다

업데이트 확인, 확장 포맷(JPEG XL, PSD, HEIC, RAW 등), 탐색기 메뉴 등록, 썸네일 줄은 **이번 버전에 포함되지 않습니다.**

v1.0.6 대비 성능 동등성은 아직 측정하지 않았습니다.

---

## 지원 플랫폼 및 릴리스

실행 파일과 체크섬(`SHA256SUMS.txt`)은 [Releases 페이지](https://github.com/Ezenitac-AiService/ImageViewer/releases)에서 받을 수 있습니다. 모든 릴리스는 각 OS에서 실제로 실행해 검사한 뒤 게시하며, 릴리스 노트에 검증된 플랫폼이 표로 표시됩니다.

| 운영체제 | 아키텍처 | 파일 | 상태 |
|---|---|---|---|
| **Windows 10 / 11** | `x64` | `ImageViewer-windows-x64.exe` (설치 없이 실행) | 지원 |
| **macOS 11 이상** | Apple Silicon · Intel | `ImageViewer-macos-universal.dmg` | 지원 |
| **Linux** (X11 / Wayland) | `x64` | `ImageViewer-linux-x64.tar.gz` | 실험적·미검증 |

```bash
sha256sum -c SHA256SUMS.txt --ignore-missing
```

---

## 키보드·마우스

`Primary`는 macOS에서 `Cmd`, Windows·Linux에서 `Ctrl`입니다.

| 동작 | 입력 |
|---|---|
| 파일 열기 / 폴더 열기 | `Primary+O` / `Primary+Shift+O`, 창에 끌어다 놓기 |
| 다음 / 이전 이미지 | `→` `↓` / `←` `↑`, 마우스 휠, 마우스 앞으로·뒤로 버튼 |
| 처음 / 마지막 | `Home` / `End` |
| 다음 / 이전 페이지 (TIFF) | `PageDown` / `PageUp` |
| 확대 / 축소 | `+` `=` / `-`, `Primary`+휠, 트랙패드 핀치 |
| 원본 / 너비 / 높이 / 창 맞춤 | `0` / `7` / `8` / `9` |
| 보기 초기화 | `Primary+0` |
| 회전 / 반전 | `R` `L` / `H` `V` |
| 이동 (확대 상태) | 끌기, 가운데 버튼 끌기, `Shift`+휠 |
| 전체 화면 | `Alt+Enter`, 캔버스 두 번 클릭, macOS `Ctrl+Cmd+F` |
| 슬라이드쇼 / 이미지 정보 / 이미지 목록 | `Space` / `Tab` / `Enter` |
| 설정 | `Primary+,` |
| 닫기 | `Esc` (열린 대화상자를 먼저 닫음) |

---

## 소스 코드 빌드 및 실행

### 사전 요구 사항
- Rust 툴체인 `1.95+` (Edition 2024)
- Linux: `libwayland-dev libxkbcommon-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`

### 명령어

```bash
cargo run -p iv-app --release
```

품질 검사 명령과 완료 기준은 [CONTRIBUTING.md](CONTRIBUTING.md)를 참고하세요.

---

## 기여 및 보안 안내

- **기여 가이드**: 독립 포크 개발 원칙(`upstream` 병합 금지 정책) 및 품질 게이트는 [CONTRIBUTING.md](CONTRIBUTING.md) ([English](CONTRIBUTING.en.md))를 참고해 주세요.
- **보안 정책**: 보안 취약점 비공개 제보 방법은 [SECURITY.md](SECURITY.md) ([English](SECURITY.en.md))를 참고해 주세요.

## 라이선스 및 저작권 고지

- **라이선스**: GNU General Public License v3.0 (`GPL-3.0-only`) — [LICENSE](LICENSE) 참조
- **저작권 및 변경 고지 (GPLv3 §5)**: [NOTICE](NOTICE) ([English](NOTICE.en.md)) 및 [CHANGELOG.md](CHANGELOG.md) ([English](CHANGELOG.en.md)) 참조
- **서드파티 프레임워크 및 라이브러리 고지**: CI에서 [`deny.toml`](deny.toml)(`cargo-deny`)을 통해 자동 검증됨 — [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) ([English](THIRD_PARTY_NOTICES.en.md)) 참조
