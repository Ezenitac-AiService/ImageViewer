# 변경 이력 (Changelog)

**언어 / Language**: **[한국어](CHANGELOG.md)** | [English](CHANGELOG.en.md)

이 프로젝트의 주요 변경 사항은 본 문서에 기록됩니다.

## [1.1.0-alpha.1] - 2026-11-16 (독립 크로스 플랫폼 Rust 포크 — 알파 프리릴리스)

### 변경 사항 (Changed)
- **독립 포크 분리**: [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)(`v1.0.6`)에서 파생된 독립 크로스 플랫폼 포크 저장소로 `Ezenitac-AiService/ImageViewer`를 분리했습니다.
- **크로스 플랫폼 Rust 전면 전환**: 기존 Windows 전용 C#/.NET 코드베이스를 Windows·macOS·Linux를 지원하는 4-크레이트 Rust 워크스페이스(`iv-core`, `iv-media`, `iv-platform`, `iv-app`)로 재작성했습니다.
- **라이선스 및 고지 정비**: 루트 `LICENSE`(GNU GPLv3)와 일치하도록 Cargo 워크스페이스 메타데이터를 `GPL-3.0-only`로 통일하고, `NOTICE`(`NOTICE.en.md`), `THIRD_PARTY_NOTICES.md`(`THIRD_PARTY_NOTICES.en.md`), `deny.toml`(`cargo-deny` 라이선스 검증 게이트)을 추가했습니다.
- **업데이트 및 식별자 격리**: 자동 업데이트 피드를 `Ezenitac-AiService/ImageViewer` 전용으로 격리하고 OS별 릴리스 에셋 자동 선택을 적용했으며, 기존 `ImageViewer` 설정 폴더(`settings.json`) 호환성을 유지하면서 macOS 번들 ID를 `com.ezenitac.imageviewer`로 갱신했습니다.
- **레거시 아카이브 보존**: 전환 이전의 Windows C#/.NET `v1.0.6` 기준선을 불변 Git 태그 `legacy-v1.0.6`에 보존했습니다.

## [1.0.6] - 업스트림 레거시 기준선

- 원작자 `edith2580`(`edith2580/ImageViewer`)의 Windows C#/.NET 릴리스이며, Git 태그 `legacy-v1.0.6`에 아카이브되어 있습니다.
