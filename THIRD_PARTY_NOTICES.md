# 서드파티 소프트웨어 고지 (Third-Party Software Notices)

**언어 / Language**: **[한국어](THIRD_PARTY_NOTICES.md)** | [English](THIRD_PARTY_NOTICES.en.md)

`ImageViewer`(`Ezenitac-AiService/ImageViewer`, 라이선스: `GPL-3.0-only`)의 **배포본**은 아래 오픈소스 Rust 라이브러리를 직접 사용합니다. 각 라이브러리는 [기능 목록](docs/CAPABILITIES.md)의 기능 id로 사용 근거를 밝힙니다(specs/004 FR-034). 이를 포함한 모든 의존성의 라이선스 호환성, 알려진 취약점, 출처는 CI와 릴리스에서 `cargo deny check advisories bans licenses sources`([`deny.toml`](deny.toml))로 자동 검증합니다.

배포본에 포함되는 패키지 수(간접 의존 포함)는 `packaging/verify/component-count.sh`로 측정하며 2026-11-24 기준 활성 3개 타깃(`aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`) 통합 고유 패키지는 277개입니다(이전 4타깃 298개 / 변경 전 446개).

## 배포본에 포함되는 직접 의존성

| 크레이트 | 용도 | 근거(기능 id) | 라이선스 | 업스트림 |
|---|---|---|---|---|
| `eframe` | 앱 셸(창, 입력, 렌더러 연결) | `launch`, 모든 UI | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `egui` | UI(툴바, 대화상자, 상태 표시) | 모든 UI | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `epaint_default_fonts` | 내장 기본 UI 글꼴 (*Ubuntu Light*, *Hack*, *Noto Emoji*) | 모든 UI | `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` | https://github.com/emilk/egui |
| `winit` | 창 생성과 이벤트 루프 | `launch`, `open-with-os` | `Apache-2.0` | https://github.com/rust-windowing/winit |
| `wgpu` | GPU 렌더링 (Metal / Vulkan, CPU 렌더러 포함) | `launch` | `MIT OR Apache-2.0` | https://github.com/gfx-rs/wgpu |
| `glow` | OpenGL 대체 렌더링 경로 | `launch` | `MIT OR Apache-2.0 OR Zlib` | https://github.com/grovesNL/glow |
| `pollster` | 파일 대화상자·어댑터 조회의 비동기 대기 | `open-file-dialog`, `launch` | `Apache-2.0 OR MIT` | https://github.com/zesterer/pollster |
| `rfd` | OS 기본 파일·폴더 대화상자, 치명 오류 대화상자 | `open-file-dialog`, `open-folder-dialog`, `launch` | `MIT` | https://github.com/PolyMeilex/rfd |
| `objc2`, `objc2-foundation`, `objc2-app-kit` | macOS "다른 앱으로 열기" 연결 | `open-with-os` | `MIT; Zlib OR Apache-2.0 OR MIT (app-kit)` | https://github.com/madsmtm/objc2 |
| `sys-locale` | 첫 실행 시 OS 언어 확인 | `language` | `MIT OR Apache-2.0` | https://github.com/1Password/sys-locale |
| `directories` | OS별 설정·로그 폴더 위치 | `settings`, `launch` | `MIT OR Apache-2.0` | https://github.com/soc/directories-rs |
| `serde`, `serde_json` | 설정 파일 읽기·쓰기 | `settings` | `MIT OR Apache-2.0` | https://github.com/serde-rs/serde |
| `fs4` | 설정 저장 시 파일 잠금 | `settings` | `MIT OR Apache-2.0` | https://github.com/al8n/fs4-rs |
| `rand` | 실행 ID와 임시 파일 이름 | `launch`, `settings` | `MIT OR Apache-2.0` | https://github.com/rust-random/rand |
| `thiserror` | 오류 형식 정의 | 전체 | `MIT OR Apache-2.0` | https://github.com/dtolnay/thiserror |
| `image` | PNG·GIF·WebP·BMP·TIFF 디코딩 | `format-*` | `MIT OR Apache-2.0` | https://github.com/image-rs/image |
| `zune-jpeg` | JPEG 디코딩 | `format-jpeg` | `MIT OR Apache-2.0 OR Zlib` | https://github.com/etemesi254/zune-image |
| `image-webp` | 애니메이션 WebP 디코딩 | `animation` | `MIT OR Apache-2.0` | https://github.com/image-rs/image-webp |
| `tiff` | 여러 페이지 TIFF 디코딩 | `format-tiff`, `multipage-tiff` | `MIT` | https://github.com/image-rs/image-tiff |
| `moxcms` | ICC 색 프로파일을 sRGB로 변환 | `format-*` | `BSD-3-Clause OR Apache-2.0` | https://github.com/awxkee/moxcms |
| `fast_image_resize` | 화면 크기에 맞춘 축소 디코딩 | `format-*` | `MIT OR Apache-2.0` | https://github.com/cykooz/fast_image_resize |
| `kamadak-exif` | EXIF 방향·정보 읽기 | `format-jpeg`, `info-panel` | `BSD-2-Clause` | https://github.com/kamadak/exif-rs |
| `lru` | 디코딩 결과 캐시 (최대 8개) | `navigate` | `MIT` | https://github.com/jeromefroe/lru-rs |
| `rayon` | 백그라운드 디코딩·미리 읽기 | `navigate` | `MIT OR Apache-2.0` | https://github.com/rayon-rs/rayon |
| `zip` | ZIP·CBZ 읽기 (stored·deflate·zstd) | `archive-zip` | `MIT` | https://github.com/zip-rs/zip2 |
| `notify`, `notify-debouncer-mini` | 폴더 변경 감시 | `navigate` | `CC0-1.0; MIT OR Apache-2.0` | https://github.com/notify-rs/notify |

## 배포본에 포함되지 않는 라이브러리

아래 라이브러리는 기본으로 꺼진 기능 뒤에 있어 배포본에 포함되지 않지만, CI의 전체 기능 빌드에서 계속 빌드·테스트합니다.

| 기능 | 라이브러리 |
|---|---|
| `iv-platform` `updater` | `reqwest`, `rustls`, `sha2` |
| `iv-platform` `os-codecs` | `objc2-image-io`, `objc2-core-graphics`, `objc2-core-foundation` |
| `iv-platform` `shell-registration` | Linux XDG 도우미만 유지 (추가 외부 크레이트 없음) |
| `iv-media` `extended-formats` | `jxl-oxide`, `image_dds`, `rawpsd`, `image-extras`, `exr` |

전체 의존성 트리는 `cargo tree -p iv-app -e normal`로 확인할 수 있습니다.
