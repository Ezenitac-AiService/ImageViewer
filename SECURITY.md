# 보안 정책 (Security Policy)

**언어 / Language**: **[한국어](SECURITY.md)** | [English](SECURITY.en.md)

## 적용 범위

본 보안 정책은 **Ezenitac-AiService/ImageViewer**(크로스 플랫폼 Rust 에디션)에 적용됩니다.

## 보안 취약점 제보 방법

경로 이탈(Path Traversal), 압축 파일 슬립(Zip-Slip), 안전하지 않은 심볼릭 링크 처리, 이미지 디코딩 중 정수 오버플로나 과도한 메모리 사용 등 보안 취약점을 발견하신 경우 공개 이슈(Public Issue) 대신 비공개 채널로 제보해 주세요.

- `https://github.com/Ezenitac-AiService/ImageViewer` 저장소의 **GitHub Private Vulnerability Reporting** (`Security` -> `Advisories` -> `Report a vulnerability`) 기능을 사용해 주세요.
- 제보 시 아래 정보를 포함해 주시면 신속한 확인에 도움이 됩니다:
  - 영향을 받는 운영체제(OS), 아키텍처 및 `ImageViewer` 버전
  - 최소 재현 절차 또는 샘플 파일 (민감한 개인정보 제외)
  - 기대 동작과 실제 발생 결과

제보 접수 후 72시간 이내에 확인 회신을 드리는 것을 목표로 하며, 패치 완료 후 공개 일정을 협의합니다.

> 참고: 본 크로스 플랫폼 Rust 포크의 취약점을 업스트림 `edith2580/ImageViewer` 저장소에 제보하지 말아 주세요.

## 안전 속성과 이를 증명하는 테스트

이 문서에 적은 안전 속성은 모두 아래 자동 테스트로 증명됩니다. 테스트가 없는 속성은 문서에 적지 않습니다(specs/004 FR-033). 위반 사례를 발견하면 위의 비공개 제보 절차를 이용해 주세요.

| 속성 | 테스트 |
|---|---|
| 압축 파일 항목이 압축 파일 밖의 경로를 가리키면 건너뜀 (Zip Slip) | `path_security_tests::archive_entry_key_normalization_rejects_traversal_and_symlinks`, `sources_tests::zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks` |
| 압축 파일 안의 심볼릭 링크 항목과 폴더 안의 심볼릭 링크는 열지 않음 | `sources_tests::zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks`, `sources_tests::folder_source_natural_sorts_and_skips_symlinks` |
| 항목 5만 개·중앙 디렉터리 64MB·컨테이너 4GB 초과 압축 파일 거부 | `sources_tests::zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks` |
| 압축 해제 크기 상한을 넘는 항목(압축 폭탄) 거부 | `sources_tests::zip_entry_larger_than_the_cap_is_refused_when_read` |
| 암호화된 압축 항목 거부 | `sources_tests::encrypted_zip_entries_are_refused` |
| SVG·HTML·PDF·PostScript 등 스크립트·문서 내용은 확장자와 관계없이 거부 | `decode_color_resize_tests::script_and_document_payloads_named_png_are_refused` |
| 내용이 지원 형식으로 판별되지 않으면 디코딩하지 않음 (확장자 무시) | `format_guard_tests::identify::*`, `decode_color_resize_tests::random_bytes_named_jpg_are_unsupported_without_decoding`, `decode_color_resize_tests::extended_format_signatures_are_unsupported_in_the_default_build` |
| 이미지 크기 상한(기본 150MP)을 헤더 단계에서 확인해 메모리 할당 전에 거부 | `decode_color_resize_tests::huge_declared_png_dimensions_fail_before_allocation`, `decode_color_resize_tests::enforces_pixel_cap_and_performs_simd_downscale_and_hdr_tonemap` |
| 512MB를 넘는 파일은 읽기 전에 거부 | `sources_tests::oversized_file_is_refused_before_it_is_read` |
| 64KB를 넘거나 손상된 설정 파일은 기본값으로 복구하고 알림 | `settings_tests::load_outcome::oversized_file_is_recovered_from_damage`, `settings_tests::load_outcome::malformed_json_is_recovered_from_damage` |
| 심볼릭 링크로 된 설정 파일은 읽지 않음 | `settings_tests::load_outcome::symlinked_settings_file_is_unreadable` |
| 여러 인스턴스가 동시에 설정을 저장해도 파일이 손상되지 않음 | `settings_tests::load_outcome::concurrent_saves_from_two_instances_never_corrupt_the_file` |
| 진단 기록에는 전체 폴더 경로·줄바꿈 제어 문자가 남지 않음 | `logger_and_i18n_tests::async_logger_redacts_paths_rotates_and_flushes_on_drop`, `diagnostics_tests::values_never_carry_control_characters_or_host_paths`, `path_security_tests::path_redactor_strips_paths_and_sanitizes_control_chars` |
| 배포본에는 네트워크 클라이언트가 없음 (앱이 네트워크에 접속하지 않음) | `packaging/verify/component-count.sh` (CI) |
| 알려진 취약점이 있는 의존성은 릴리스를 막음 | `cargo deny check advisories` (CI·릴리스) |
