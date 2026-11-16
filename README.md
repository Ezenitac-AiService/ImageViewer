# ImageViewer

**언어 / Language**: **[한국어](README.md)** | [English](README.en.md)

[![CI](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/ci.yml)
[![Release Build](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml/badge.svg)](https://github.com/Ezenitac-AiService/ImageViewer/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Ezenitac-AiService/ImageViewer?include_prereleases&sort=semver&label=release&logo=github)](https://github.com/Ezenitac-AiService/ImageViewer/releases)
[![License: GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust: 1.85+ (Edition 2024)](https://img.shields.io/badge/rust-1.85%2B%20(2024)-orange.svg?logo=rust)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](#지원-플랫폼-및-릴리스)
[![Upstream Origin](https://img.shields.io/badge/upstream%20origin-edith2580%2FImageViewer%20(v1.0.6)-informational?logo=github)](https://github.com/edith2580/ImageViewer)

**Windows**, **macOS**, **Linux**를 지원하는 빠르고 안전한 GPU 가속·제로 트러스트(Zero-Trust) 기반 크로스 플랫폼 이미지 및 만화 압축파일(`.zip`/`.cbz`) 뷰어입니다 (`Rust`, `winit`, `wgpu`, `egui`).

> [!NOTE]
> **독립 크로스 플랫폼 포크 안내 및 오픈소스 예절 (Fork Etiquette)**
>
> 본 저장소(**[Ezenitac-AiService/ImageViewer](https://github.com/Ezenitac-AiService/ImageViewer)**)는 **[edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)**(`v1.0.6`, Windows C#/.NET)에서 파생된 **독립 크로스 플랫폼 Rust 포크 프로젝트**로, 원작자(`edith2580`)의 저작권과 `GPL-3.0-only` 라이선스를 존중하며 계승합니다.
>
> - **포크 목적**: Windows·macOS·Linux 네이티브 실행, GPU 가속 렌더링, 제로 트러스트 보안(경로/압축/업데이트 검증)을 위해 순수 Rust 4-크레이트 워크스페이스(`iv-core`, `iv-media`, `iv-platform`, `iv-app`)로 전면 재설계되었습니다.
> - **레거시 아카이브**: 전환 이전의 Windows C#/.NET `v1.0.6` 코드는 [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6) 태그에 보존되어 있습니다.
> - **이슈 등록 예절**: **크로스 플랫폼 Rust 버전에 관한 버그 제보나 기능 요청을 업스트림(`edith2580/ImageViewer`) 원작자에게 등록하지 말아 주세요.** 모든 이슈와 문의는 본 저장소의 [Ezenitac-AiService/ImageViewer/issues](https://github.com/Ezenitac-AiService/ImageViewer/issues)에 등록해 주시기 바랍니다.
> - **원본 Windows 전용 C#/.NET 버전이 필요하신가요?** [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)를 방문해 주세요.

---

## 업스트림(`v1.0.6`)과의 차이점

| 구분 | 업스트림 (`edith2580/ImageViewer` `v1.0.6`) | 독립 포크 (`Ezenitac-AiService/ImageViewer`) |
|---|---|---|
| **지원 OS** | Windows (`x64`) | **Windows**, **macOS** (`arm64`/`x64`), **Linux** (X11/Wayland) |
| **기술 스택 및 렌더링** | C# / .NET WPF + WIC / GDI+ | **Rust (Edition 2024)** + `winit` / `wgpu` (Metal, DX12, Vulkan) / `egui` |
| **색상 관리 및 리샘플링** | Windows WIC | 크로스 플랫폼 ICC 프로파일(`moxcms`) + SIMD 가속 리샘플링(`fast_image_resize`) |
| **자동 업데이트 및 보안** | Windows 인스톨러 업데이터 | 독립 저장소 전용 HTTPS 피드(`Ezenitac-AiService/ImageViewer`), OS별 에셋 자동 매칭, `SHA256SUMS.txt` 무결성 검증 |
| **설정 파일 호환성** | `%LOCALAPPDATA%\ImageViewer\settings.json` | 기존 `ImageViewer/settings.json` 경로 유지 및 `v1.0.6` camelCase 설정 자동 마이그레이션 |

---

## 주요 기능

- **폭넓은 포맷 지원**: JPEG(`zune-jpeg`), PNG, APNG, GIF, WebP, BMP, TIFF, ICO/CUR, JPEG XL(`jxl-oxide`), PSD, DDS, TGA, QOI, OpenEXR(`exr`), Radiance HDR, AVIF/HEIC, RAW 미리보기 및 `.zip` / `.cbz` 압축 아카이브
- **GPU 가속 렌더링 및 색상 관리**: `wgpu` 하드웨어 가속 렌더링, SIMD 리샘플링(`fast_image_resize`), ICC 컬러 프로파일 변환(`moxcms`)
- **2페이지 펼침(만화) 및 열람 모드**: 단일 페이지, 2페이지 펼침 보기, 좌→우(LTR) / 우→좌(RTL) 읽기 방향, 창 맞춤 / 원본 크기(1:1) / 자유 줌, 슬라이드쇼, 테두리 없는 전체 화면
- **제로 트러스트(Zero-Trust) 보안**: 경로 이탈(Path Traversal) 및 심볼릭 링크/정션 차단, 압축 해제 및 HTTP 업데이트 응답 크기 상한 적용, SHA-256 해시 검증 기반 HTTPS 전용 업데이트 확인
- **다국어 UI 지원**: 한국어(`ko`) 및 영어(`en`)

---

## 지원 플랫폼 및 릴리스

사전 빌드된 실행 파일과 체크섬(`SHA256SUMS.txt`)은 [Releases 페이지](https://github.com/Ezenitac-AiService/ImageViewer/releases)에서 다운로드할 수 있습니다.

| 운영체제 | 아키텍처 | 릴리스 아티팩트 | 패키징 스크립트 |
|---|---|---|---|
| **Windows 10 / 11** | `x86_64` | `ImageViewer-windows-x64.exe` | `packaging/windows/package-windows.ps1` (`ImageViewer.nsi`) |
| **macOS** | `aarch64` / `x86_64` | `ImageViewer-macos.dmg` | `packaging/macos/package-dmg.sh` |
| **Linux (X11 / Wayland)** | `x86_64` | `ImageViewer-linux-x64.tar.gz` | `packaging/linux/package-appimage.sh` |

### 릴리스 파일 무결성 검증

```bash
sha256sum -c SHA256SUMS.txt --ignore-missing
```

---

## 키보드 단축키

| 동작 | 단축키 |
|---|---|
| 파일 / 폴더 열기 | `Ctrl+O` / `Cmd+O` |
| 다음 / 이전 이미지 | `Right` / `Left`, `PageDown` / `PageUp`, `Space` |
| 첫 / 마지막 이미지 | `Home` / `End` |
| 2페이지 펼침 보기 전환 | `D` |
| 읽기 방향 전환 (LTR / RTL) | `R` |
| 전체 화면 전환 | `F11` / `Enter` |
| 확대 / 축소 / 초기화 (1:1 / 창 맞춤) | `+` / `-` / `0` / `F` |
| 슬라이드쇼 시작 / 정지 | `S` |
| 설정 / 정보(About) 창 | `F2` / `F1` |

---

## 소스 코드 빌드 및 실행

### 사전 요구 사항
- Rust 툴체인 `1.85+` (Edition 2024)
- Linux 환경 필수 패키지: `libwayland-dev libxkbcommon-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`

### 명령어

```bash
# 릴리스 모드로 뷰어 실행
cargo run -p iv-app --release

# 워크스페이스 포맷, 타입 체크, 엄격한 Clippy 린트 및 전체 테스트 실행
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 기여 및 보안 안내

- **기여 가이드**: 독립 포크 개발 원칙(`upstream` 병합 금지 정책) 및 품질 게이트는 [CONTRIBUTING.md](CONTRIBUTING.md) ([English](CONTRIBUTING.en.md))를 참고해 주세요.
- **보안 정책**: 보안 취약점 비공개 제보 방법은 [SECURITY.md](SECURITY.md) ([English](SECURITY.en.md))를 참고해 주세요.

## 라이선스 및 저작권 고지

- **라이선스**: GNU General Public License v3.0 (`GPL-3.0-only`) — [LICENSE](LICENSE) 참조
- **저작권 및 변경 고지 (GPLv3 §5)**: [NOTICE](NOTICE) ([English](NOTICE.en.md)) 및 [CHANGELOG.md](CHANGELOG.md) ([English](CHANGELOG.en.md)) 참조
- **서드파티 프레임워크 및 라이브러리 고지**: CI에서 [`deny.toml`](deny.toml)(`cargo-deny`)을 통해 자동 검증됨 — [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) ([English](THIRD_PARTY_NOTICES.en.md)) 참조
