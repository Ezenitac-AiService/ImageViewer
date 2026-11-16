# 서드파티 소프트웨어 고지 (Third-Party Software Notices)

**언어 / Language**: **[한국어](THIRD_PARTY_NOTICES.md)** | [English](THIRD_PARTY_NOTICES.en.md)

`ImageViewer`(`Ezenitac-AiService/ImageViewer`, 라이선스: `GPL-3.0-only`)는 아래의 오픈소스 Rust 프레임워크 및 라이브러리를 사용하여 빌드되었습니다. 모든 서드파티 의존성의 GNU GPLv3 호환성은 `deny.toml`(`cargo deny check licenses`)을 통해 자동 검증됩니다.

| 크레이트 | 용도 | 라이선스 | 업스트림 프로젝트 |
|---|---|---|---|
| `winit` | 크로스 플랫폼 창 관리 및 이벤트 루프 | `Apache-2.0` | https://github.com/rust-windowing/winit |
| `wgpu` | 크로스 플랫폼 GPU 렌더링 (Metal / DX12 / Vulkan) | `MIT OR Apache-2.0` | https://github.com/gfx-rs/wgpu |
| `egui` | 즉시 모드(Immediate-mode) UI 오버레이 및 정보/설정 대화상자 | `MIT OR Apache-2.0` | https://github.com/emilk/egui |
| `epaint_default_fonts` | 임베디드 기본 UI 폰트 (*Ubuntu Light*, *Hack*, *Noto Emoji*) | `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` | https://github.com/emilk/egui |
| `zune-jpeg` | 고속 JPEG 디코딩 | `MIT OR Apache-2.0 OR Zlib` | https://github.com/etemesi254/zune-image |
| `image` | 다중 포맷 이미지 디코딩 및 인코딩 | `MIT OR Apache-2.0` | https://github.com/image-rs/image |
| `fast_image_resize` | SIMD 가속 이미지 리샘플링 | `MIT OR Apache-2.0` | https://github.com/Cykooz/fast_image_resize |
| `moxcms` | ICC 컬러 프로파일 관리 및 색공간 변환 | `BSD-3-Clause OR Apache-2.0` | https://github.com/awxkee/moxcms |
| `jxl-oxide` | 순수 Rust JPEG XL 디코딩 | `MIT OR Apache-2.0` | https://github.com/tirr-c/jxl-oxide |
| `exr` | OpenEXR 이미지 디코딩 | `BSD-3-Clause` | https://github.com/johannesvollmer/exr |
| `image-webp` | WebP 이미지 디코딩 및 인코딩 | `MIT OR Apache-2.0` | https://github.com/image-rs/image-webp |
| `zip` | 만화/압축 아카이브 컨테이너 읽기 (`.zip`, `.cbz`) | `MIT` | https://github.com/zip-rs/zip2 |
| `reqwest` | 격리된 릴리스 업데이트 확인용 HTTPS 클라이언트 | `MIT OR Apache-2.0` | https://github.com/seanmonstar/reqwest |
| `rustls` | 순수 Rust TLS 구현체 | `Apache-2.0 OR ISC OR MIT` | https://github.com/rustls/rustls |
| `sha2` | SHA-256 체크섬 무결성 검증 | `MIT OR Apache-2.0` | https://github.com/RustCrypto/hashes |
| `rfd` | 네이티브 OS 파일/폴더 열기 및 저장 대화상자 | `MIT` | https://github.com/PolyMeilex/rfd |

전체 의존성 트리와 상세 라이선스 전문은 워크스페이스 루트에서 `cargo metadata` 또는 `cargo deny list`를 실행하여 확인할 수 있습니다.
