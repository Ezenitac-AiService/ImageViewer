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
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### 코딩 표준
- 테스트 주도 개발(TDD) 원칙에 따라 프로덕션 코드 수정 전 실패하는 테스트를 먼저 작성합니다.
- 제로 트러스트(Zero-Trust) 보안 불변식(경로 이탈 차단, 심볼릭 링크 거부, HTTPS 전용 업데이트 및 SHA-256 검증, 테스트 우회 로직 금지)을 엄격히 유지합니다.
- 프로젝트 헌법(`.specify/memory/constitution.md`)에 따라 커밋 메시지와 사용자 대상 요약은 한국어(원칙 I), 코드·주석·명세 문서는 영어(원칙 II)를 기본으로 작성하되 GitHub 주요 문서는 국문 기본(`*.md`) 및 영문 병행(`*.en.md`) 페이지를 유지합니다.
