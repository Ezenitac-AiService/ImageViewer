# ImageViewer 기여 가이드 (`Ezenitac-AiService/ImageViewer`)

**언어 / Language**: **[한국어](CONTRIBUTING.md)** | [English](CONTRIBUTING.en.md)

**ImageViewer** 프로젝트에 관심을 갖고 기여해 주셔서 감사합니다!

## 독립 포크 운영 원칙

`Ezenitac-AiService/ImageViewer`는 `edith2580/ImageViewer`(`v1.0.6`)에서 파생된 **독립 크로스 플랫폼 Rust 포크 프로젝트**입니다.

- **업스트림(`upstream`) 병합 금지**: `edith2580/ImageViewer`(`upstream`)의 커밋이나 PR을 병합(`git merge upstream`)하지 마세요. 레거시 C#/.NET `v1.0.6` 기준선은 `legacy-v1.0.6` 태그에 보존되어 있으며, `main` 브랜치는 순수 Rust 코드베이스로만 운영됩니다.
- **라이선스 호환성**: 모든 기여 코드는 `GPL-3.0-only` 라이선스를 준수해야 합니다. 새로운 Cargo 의존성을 추가할 경우 반드시 `deny.toml`(`cargo deny check licenses`)을 통과해야 하며, 주요 라이브러리는 `THIRD_PARTY_NOTICES.md` 및 `THIRD_PARTY_NOTICES.en.md`에 함께 문서화해야 합니다.
- **이슈 및 PR 등록처**: 모든 이슈와 PR은 반드시 `Ezenitac-AiService/ImageViewer`에만 등록해 주세요. 크로스 플랫폼 포크 관련 문의를 업스트림(`edith2580/ImageViewer`)으로 안내하거나 전달하지 않습니다.

## 개발 및 품질 게이트

Pull Request를 열기 전에 로컬에서 아래 검증 명령이 모두 통과하는지 확인해 주세요.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
cargo test --workspace
cargo deny check advisories bans licenses sources
cargo deny --all-features check advisories bans licenses sources
packaging/verify/component-count.sh
```

배포 설정(기본 기능)과 전체 기능 설정을 모두 테스트합니다. 전체 기능에는 아직 배포하지 않는 기능(업데이트, OS 코덱, 확장 포맷, 셸 등록)이 포함됩니다. 자세한 내용은 [기능 목록](docs/CAPABILITIES.md)을 참고하세요.

## 완료 기준 (Definition of Done)

작업은 다음을 모두 만족해야 완료입니다.

1. 실패하는 테스트를 먼저 작성하고, 구현 후 통과합니다.
2. 동작을 **실행한 앱에서 직접 확인**했습니다. 헌법 `v3.0.0`에 따라 **실행 검증은 최우선 지원 대상(`macOS`, `007` 이후 `iOS`)에만** 적용되며, `Linux`(`007` 이후 `Android` 포함)는 **빌드 전용·미검증** 계층으로 컴파일·단위 테스트만 확인합니다(`Windows` 제외). 확인한 환경과 미확인 환경을 PR에 명시합니다. 라이브러리 테스트 통과만으로는 완료가 아닙니다.
3. 화면이나 문서에 새로 노출되는 기능은 [기능 목록](docs/CAPABILITIES.md)에 근거와 함께 등록되어 있습니다.

## 릴리스 절차

1. `v*` 태그를 올리면 릴리스 워크플로가 최우선 **macOS**(Apple Silicon·Intel) 번들과 하위 우선순위 빌드 전용 **Linux**(`x86_64`) 아카이브를 빌드하고, **macOS에서만 실행 검증**을 수행합니다(`Linux` 런타임 검사 없음, `Windows` 제외). `macOS` 검증·`Linux` 빌드·`cargo deny` 중 하나라도 실패하면 초안이 만들어지지 않습니다.
2. 통과하면 **초안(draft) 릴리스**만 만들어집니다.
3. 관리자가 실제 깨끗한 **macOS (Apple Silicon · Intel)** 기기에서 [수동 체크리스트](specs/006-platform-tier-realignment/quickstart.md)를 실행하고, 결과를 `launch-verification-manual.md`로 초안에 첨부합니다.
4. 그 다음에만 게시합니다.

### 코딩 표준
- 테스트 주도 개발(TDD) 원칙에 따라 프로덕션 코드 수정 전 실패하는 테스트를 먼저 작성합니다.
- 제로 트러스트(Zero-Trust) 보안 불변식(경로 이탈 차단, 심볼릭 링크 거부, HTTPS 전용 업데이트 및 SHA-256 검증, 테스트 우회 로직 금지)을 엄격히 유지합니다.
- 프로젝트 헌법(`.specify/memory/constitution.md`)에 따라 커밋 메시지와 사용자 대상 요약은 한국어(원칙 I), 코드·주석·명세 문서는 영어(원칙 II)를 기본으로 작성하되 GitHub 주요 문서는 국문 기본(`*.md`) 및 영문 병행(`*.en.md`) 페이지를 유지합니다.
