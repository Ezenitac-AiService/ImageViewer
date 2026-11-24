# 변경 이력 (Changelog)

**언어 / Language**: **[한국어](CHANGELOG.md)** | [English](CHANGELOG.en.md)

이 프로젝트의 주요 변경 사항은 본 문서에 기록됩니다.

## [1.1.0-alpha.2] - 미출시 (실행 안정성 리팩터링 · 플랫폼 우선순위 재정렬)

`v1.1.0-alpha.1`이 macOS에서 사용할 수 없었던 문제를 고치고, 헌법 `v3.0.0`에 따라 플랫폼 지원 계층을 재정렬한 버전입니다. GPLv3 §5(a)에 따른 수정 고지: 2026-11-16–17 및 2026-11-24 `Ezenitac-AiService`가 수정했습니다.

### 수정됨 (Fixed)
- **macOS**: 창이 입력에 반응하지 않던 문제와 다운로드 앱이 "손상됨"으로 보이던 문제. 직접 작성한 창·렌더링 코드를 `eframe`으로 교체하고, 번들 전체를 ad-hoc 서명한 Apple Silicon·Intel 공용(universal) 파일로 배포합니다.
- 시작에 실패하면 아무 말 없이 종료되던 문제. 이제 원인을 알리는 창을 띄우고 진단 기록을 남깁니다.
- 여러 인스턴스가 동시에 설정을 저장할 때 저장이 실패할 수 있던 경쟁 조건.
- 폴더 내용이 바뀌면 처음 연 이미지로 돌아가던 문제.

### 변경됨 (Changed)
- **플랫폼 우선순위 재정렬 (2026-11-24, Constitution v3.0.0)**:
  - **최우선 (Top Priority, 빌드 결과 및 실행 검증)**: `macOS` (`aarch64` · `x86_64`)를 활성 최우선 검증 대상으로 전환했으며, `iOS` (`arm64`) 지원은 마일스톤 `007`에서 순차 구현할 예정입니다.
  - **하위 우선순위 (Lower Priority, 빌드 전용·미검증)**: `Linux` (`x86_64`)는 CI 컴파일·단위 테스트 및 빌드 전용 아카이브 생성만 유지하고 실행·창 검증을 수행하지 않습니다(`Android` 빌드 전용 예정은 `007`).
  - **지원 제외 (Excluded)**: `Windows` 빌드·패키징·실행 검증 및 Windows 전용 의존성(`windows`, `winreg`, `winresource`)을 제거했습니다(전환 이전 Windows C#/.NET `v1.0.6` 기록은 `legacy-v1.0.6` 태그 참고).
- 이번 버전은 검증된 기능만 제공합니다([기능 목록](docs/CAPABILITIES.md)). 업데이트 확인, 확장 포맷, OS 코덱, 셸 등록, 썸네일 줄은 제외했고 앱은 네트워크에 접속하지 않습니다.
- 파일 형식을 확장자가 아니라 내용으로 판별하고, 512MB를 넘는 파일은 읽기 전에 거부합니다.
- 처음 실행할 때 OS 언어(한국어·영어)를 따르고, 설정에서 언어를 바꿀 수 있습니다.
- macOS Finder "다른 앱으로 열기"를 지원합니다.
- 진단 기록 시각은 UTC로 표기합니다.

### 보안 (Security)
- `zip` 9(취약점 4건 수정), `lru` 0.19(unsound 2건 수정), `notify` 8(유지보수 중단 의존성 제거)로 올렸습니다.
- 릴리스는 최우선 대상(`macOS`)에서 실제로 실행해 검사하고, `cargo deny`에서 알려진 취약점이 발견되면 초안이 생성되지 않습니다.

## [1.1.0-alpha.1] - 2026-11-16 (독립 크로스 플랫폼 Rust 포크 — 알파 프리릴리스)

### 변경 사항 (Changed)
- **독립 포크 분리**: [edith2580/ImageViewer](https://github.com/edith2580/ImageViewer)(`v1.0.6`)에서 파생된 독립 크로스 플랫폼 포크 저장소로 `Ezenitac-AiService/ImageViewer`를 분리했습니다.
- **크로스 플랫폼 Rust 전면 전환**: 기존 Windows 전용 C#/.NET 코드베이스를 Windows·macOS·Linux를 지원하는 4-크레이트 Rust 워크스페이스(`iv-core`, `iv-media`, `iv-platform`, `iv-app`)로 재작성했습니다.
- **라이선스 및 고지 정비**: 루트 `LICENSE`(GNU GPLv3)와 일치하도록 Cargo 워크스페이스 메타데이터를 `GPL-3.0-only`로 통일하고, `NOTICE`(`NOTICE.en.md`), `THIRD_PARTY_NOTICES.md`(`THIRD_PARTY_NOTICES.en.md`), `deny.toml`(`cargo-deny` 라이선스 검증 게이트)을 추가했습니다.
- **업데이트 및 식별자 격리**: 자동 업데이트 피드를 `Ezenitac-AiService/ImageViewer` 전용으로 격리하고 OS별 릴리스 에셋 자동 선택을 적용했으며, 기존 `ImageViewer` 설정 폴더(`settings.json`) 호환성을 유지하면서 macOS 번들 ID를 `com.ezenitac.imageviewer`로 갱신했습니다.
- **레거시 아카이브 보존**: 전환 이전의 Windows C#/.NET `v1.0.6` 기준선을 불변 Git 태그 `legacy-v1.0.6`에 보존했습니다.

## [1.0.6] - 업스트림 레거시 기준선

- 원작자 `edith2580`(`edith2580/ImageViewer`)의 Windows C#/.NET 릴리스이며, Git 태그 `legacy-v1.0.6`에 아카이브되어 있습니다.
