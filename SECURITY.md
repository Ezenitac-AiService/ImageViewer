# 보안 정책 (Security Policy)

**언어 / Language**: **[한국어](SECURITY.md)** | [English](SECURITY.en.md)

## 적용 범위

본 보안 정책은 **Ezenitac-AiService/ImageViewer**(크로스 플랫폼 Rust 에디션)에 적용됩니다.

## 보안 취약점 제보 방법

경로 이탈(Path Traversal), 압축 파일 슬립(Zip-Slip), 안전하지 않은 심볼릭 링크 처리, 이미지 디코딩 중 정수 오버플로, 업데이트 피드 위변조 등 보안 취약점을 발견하신 경우 공개 이슈(Public Issue) 대신 비공개 채널로 제보해 주세요.

- `https://github.com/Ezenitac-AiService/ImageViewer` 저장소의 **GitHub Private Vulnerability Reporting** (`Security` -> `Advisories` -> `Report a vulnerability`) 기능을 사용해 주세요.
- 제보 시 아래 정보를 포함해 주시면 신속한 확인에 도움이 됩니다:
  - 영향을 받는 운영체제(OS), 아키텍처 및 `ImageViewer` 버전
  - 최소 재현 절차 또는 샘플 파일 (민감한 개인정보 제외)
  - 기대 동작과 실제 발생 결과

제보 접수 후 72시간 이내에 확인 회신을 드리는 것을 목표로 하며, 패치 완료 후 공개 일정을 협의합니다.

> 참고: 본 크로스 플랫폼 Rust 포크의 취약점을 업스트림 `edith2580/ImageViewer` 저장소에 제보하지 말아 주세요.
