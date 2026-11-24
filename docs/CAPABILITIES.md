# 기능 목록 (Capability Register)

**언어 / Language**: **[한국어](CAPABILITIES.md)** | [English](CAPABILITIES.en.md)

ImageViewer는 **근거가 있는 기능만** 배포합니다. 모든 기능은 다음 세 등급 중 하나입니다.

- **Core**: 활성 최우선 지원 대상(`macOS`; `iOS`는 기능 `007` 예정)에서 실행·빌드 결과를 검증하는 기능 (`Linux` 및 예정된 `Android`는 하위 우선순위 **빌드 전용·미검증**, `Windows`는 헌법 `v3.0.0`에 따라 제외)
- **Extended**: 하나씩 따로 검증하는 기능
- **Unavailable**: 이번 버전에서 제공하지 않는 기능. 앱 화면과 문서에 나타나지 않습니다.

Unavailable 기능을 Extended로 올리려면 네 가지가 모두 필요합니다.

1. 실제 샘플 파일로 하는 자동 테스트
2. 악의적이거나 손상된 입력에 대한 자동 테스트
3. 활성 최우선 지원 대상(`macOS`, `007` 이후 `iOS`)에서의 실행 검증 기록
4. 한·영 사용자 문서

근거 하나라도 무효가 되면 그 기능은 즉시 Unavailable로 내려갑니다. 등급 규칙의 출처는 [프로젝트 헌법](../.specify/memory/constitution.md) (`v3.0.0`)의 Media Decoding Strategy이고, 자세한 정의는 [specs/006](../specs/006-platform-tier-realignment/spec.md) 및 [specs/004](../specs/004-launch-reliability-refactor/spec.md) FR-020–FR-024에 있습니다.

> **현재 검증 상태 (2026-11-24)**: macOS Apple Silicon(로컬·CI)에서 자동 테스트와 실행 검사(`ImageViewer.app`)를 통과했고, Linux(`x86_64`)는 CI에서 컴파일·테스트(`Checks (ubuntu-latest)`)만 통과하며 런타임 검증을 수행하지 않습니다. Windows는 헌법 `v3.0.0`에 따라 제외되었습니다. **깨끗한 Apple Silicon·Intel Mac 및 Gatekeeper 첫 실행 확인 창은 아직 확인하지 않았습니다.** 깨끗한 기기 검증(릴리스 체크리스트)을 통과하기 전에는 릴리스를 게시하지 않습니다.

## Core

| id | 기능 | 화면 노출 | 근거 | 확인한 환경 |
|---|---|---|---|---|
| `launch` | 다운로드 파일 실행, 실패 시 원인 안내와 진단 기록 | 예 | `diagnostics_tests`, `panic_hook_tests`, `settings_tests::load_outcome`, `packaging/verify/launch-check.sh` | macOS arm64 |
| `open-file-dialog` | 파일·압축 파일 열기 대화상자 | 예 | `interaction_tests` (Primary+O) | macOS arm64 |
| `open-folder-dialog` | 폴더 열기 대화상자 | 예 | `interaction_tests` (Primary+Shift+O) | macOS arm64 |
| `open-drag-drop` | 창에 파일 끌어다 놓기 | 예 | `interaction_tests::dropped_file_opens_that_path` | 미확인(수동 체크리스트) |
| `open-command-line` | 실행 인수로 경로 열기 | 예 | `cli_tests`, `launch-check.sh` | macOS arm64 |
| `open-with-os` | macOS Finder "다른 앱으로 열기" | 예 | 번들 콜드·웜 열기 기록 (research.md S2) | macOS arm64 |
| `navigate` | 이전·다음·처음·마지막, 마우스 휠·트랙패드, 뒤로·앞으로 버튼 | 예 | `interaction_tests`, `interaction_model_tests`, `launch-check.sh` 키 이동 | macOS arm64 |
| `zoom-pan` | 확대·축소·이동 | 예 | `interaction_tests`, `viewport_and_chrome_tests` | macOS arm64 |
| `fit-modes` | 원본·너비 맞춤·높이 맞춤·창 맞춤 (`0`/`7`/`8`/`9`) | 예 | `interaction_tests`, `viewport_and_chrome_tests` | macOS arm64 |
| `rotate-flip` | 회전·좌우·상하 반전 | 예 | `viewport_and_chrome_tests::image_quad_*` | macOS arm64 |
| `fullscreen` | 전체 화면 전환 | 예 | `interaction_tests` | 미확인(수동 체크리스트) |
| `format-jpeg` `format-png` `format-gif` `format-webp` `format-bmp` `format-tiff` | 정지 이미지 디코딩 (내용 시그니처로 판별) | 예 | `format_guard_tests::identify`, `decode_color_resize_tests` | macOS arm64 |
| `animation` | 애니메이션 GIF·WebP 재생 | 예 | `animation_and_multipage_tests` | 미확인(수동 체크리스트) |
| `archive-zip` | ZIP·CBZ 읽기 전용 열람 (stored·deflate·zstd) | 예 | `sources_tests` (경로 이탈, 심볼릭 링크, 항목 수, 압축 폭탄, 암호화) | macOS arm64 |
| `settings` | 설정 창과 저장, 손상 시 복구 안내 | 예 | `settings_tests` | macOS arm64 |
| `language` | 한국어·영어, 첫 실행 시 OS 언어 따름 | 예 | `settings_tests::first_launch_language`, `logger_and_i18n_tests` | macOS arm64 |
| `filter-basic` | 확대 표시 방식: 픽셀 그대로(`U`)·부드럽게(`I`) | 예 | `interaction_tests` | macOS arm64 |

## Extended (후보)

릴리스 전 깨끗한 기기 검증에서 활성 최우선 대상(`macOS`; `007` 이후 `iOS` 포함)을 통과해야 유지됩니다. 실패하면 `crates/iv-app/src/app.rs`의 `EXTENDED` 값으로 숨깁니다.

| id | 기능 | 화면 노출 | 근거 | 알려진 제한 |
|---|---|---|---|---|
| `slideshow` | 슬라이드쇼 (`Space`) | 예 | `filter_and_slideshow_tests`, `interaction_tests` | — |
| `info-panel` | 이미지 정보·EXIF 패널 (`Tab`) | 예 | `exif_thumbnail_watch_tests` | — |
| `multipage-tiff` | 여러 페이지 TIFF 넘기기 (`PageUp`/`PageDown`) | 예 | `animation_and_multipage_tests`, `interaction_model_tests` | — |
| `image-list` | 이미지 목록 창 (`Enter`) | 예 | `interaction_tests` | 정렬은 목록 표시에만 적용되고 탐색 순서에는 적용되지 않습니다 |

## Unavailable

| id | 기능 | 비고 |
|---|---|---|
| `updater` | 업데이트 확인·설치 | 배포본은 네트워크에 접속하지 않습니다. 코드는 `iv-platform` `updater` 기능에 보존 |
| `format-extended` | JPEG XL, PSD, DDS, PCX, CUR, ICO, QOI, PNM, HDR, EXR, TGA | `iv-media` `extended-formats` 기능에 보존·테스트 |
| `os-codecs` | 카메라 RAW, HEIC/HEIF, AVIF, JPEG 2000 (OS 코덱) | `iv-platform` `os-codecs` 기능에 보존 (`ImageIO`) |
| `shell-registration` | OS 셸 메뉴·파일 연결 등록 (`--register-shell`) | 스위치는 "이 버전에서 사용할 수 없음"으로 안내 |
| `archive-bzip2` | bzip2로 압축된 ZIP 항목 | 라이선스 검토 전까지 제외 |
| `thumbnail-strip` | 하단 썸네일 줄 | 이전 구현은 파일 이름만 표시했음 |
| `filter-sharpen` | 부드럽게+선명하게 필터 (`S`) | |
| `custom-window-chrome` | 사용자 정의 제목 표시줄 | OS 기본 창 장식 사용 |
| `accessibility` | 화면 낭독기 지원 (AccessKit) | 의존성 예산 때문에 제외. 알려진 접근성 공백 |
