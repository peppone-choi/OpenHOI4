# ADR-0202 코어 3 OS 해시 증거 수집 CI

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-02, REQ-GEN-04, REQ-PLAT-02, DR-10 |

## 맥락

WP-02는 실제 3 OS 코어 해시 일치 증거가 필요하다. WP-04의 CLI·1,000틱 루프는 아직 없으며 기존 ci.yml은 단위 테스트 결과만 검사한다. 오케스트레이터가 이번 WP에 별도 코어 probe와 `.github/workflows/core-determinism.yml` 소유를 명시적으로 추가했다. 원격 실행·수집·기본 브랜치 완료 판정은 오케스트레이터가 한다.

## 결정

- `cargo run --locked --quiet -p oh_core --example determinism`은 실제 코어의 Fx/Qty 경계값, 타입 ID, 모든 키 필드에 고비트가 있는 튜플 RNG, BTreeMap 상태를 postcard로 직렬화하고 FNV-1a 해시 한 줄을 출력한다. 픽스처는 Rust 테스트와 공유한다. 독립 Python 참조 바이트·해시는 Rust 단위 테스트에서 비교하며, 기대값을 Rust 출력으로 갱신하지 않는다.
- 새 CI는 push, pull_request, workflow_dispatch마다 ubuntu-latest, windows-latest, macos-latest 세 runner를 모두 실행한다. fmt, 코어 all-targets clippy, 코어 단위·속성·컴파일 실패 테스트와 Python 증거 게이트 테스트가 선행한다. probe를 각 OS에서 두 번 실행해 같은 결과일 때만 `core-hash.txt`를 생성한다.
- 세 아티팩트 이름은 `core-hash-ubuntu-latest`, `core-hash-windows-latest`, `core-hash-macos-latest`다. zip 업로드(`archive: true`)로 이름을 유지한다. 파일이 없으면 업로드 실패다. 비교 job은 세 probe job이 성공한 다음 실행하며, 아티팩트를 별도 하위 디렉터리로 다운로드한다(`merge-multiple: false`). 정확한 세 디렉터리·파일·16자리 소문자 hex 형식·해시 일치를 모두 요구한다. 빠진 OS·추가 아티팩트·오염된 출력·서로 다른 해시는 실패한다. `continue-on-error`나 테스트 생략 조건을 두지 않는다.
- 모든 Actions를 공식 릴리스의 전체 commit SHA로 고정한다. 현재 GitHub-hosted runner에서 사용하는 checkout 7.0.1, setup-python 7.0.0, upload-artifact 7.0.1, download-artifact 8.0.1은 공식 GitHub API의 release·tag commit·라이선스 API 및 해당 태그 README로 확인했다. 모두 MIT다. 기존 ci.yml은 변경하지 않는다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 각 OS 단위 테스트 통과만 기록 | CI가 짧다 | 실제 OS별 해시 아티팩트 비교 증거가 없다 |
| WP-04 CLI가 생길 때까지 기다림 | 전체 루프의 해시를 검사한다 | WP-02 코어 자체의 증거를 지금 만들 수 있다 |
| 아티팩트를 한 디렉터리에 병합 | 경로가 단순하다 | 같은 파일명이 덮어써져 OS 누락을 숨길 수 있다 |
| macOS 야간 실행 또는 Windows 로컬로 대체 | 비용이 적다 | 이번에 명시된 세 OS 코어 증거 수집 요구에 맞지 않는다 |

## 결과와 영향

세 OS 실행과 네 번째 비교 job이 추가된다. 해시는 합성 코어 상태의 `0dd81b8754bcc3f9`이며 게임 규칙이나 1,000틱 시뮬레이션 증거를 대신하지 않는다. Python 테스트가 만든 OS 이름의 임시 파일은 게이트 동작을 확인하는 픽스처일 뿐 실제 Linux/macOS 실행으로 기록하지 않는다. 구현 세션에서는 Windows 두 번의 동일 해시, 참조 바이트 일치와 워크플로 정적 검증만 확인한다. 실제 세 OS CI·독립 검증은 별도 증거가 필요하다.

2026-10-06 확인한 공식 출처: [checkout 7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1), [setup-python 7.0.0](https://github.com/actions/setup-python/releases/tag/v7.0.0), [upload-artifact 7.0.1 README](https://github.com/actions/upload-artifact/blob/v7.0.1/README.md), [download-artifact 8.0.1 README](https://github.com/actions/download-artifact/blob/v8.0.1/README.md). 각 tag SHA·MIT 라이선스·아티팩트 옵션 발췌는 [API 증거](../worklog/evidence/WP-02/action-versions.txt)에 있다. [actionlint 1.7.12](https://github.com/rhysd/actionlint/releases/tag/v1.7.12)의 Windows 배포물을 공식 checksum과 비교한 뒤 워크플로를 검사했다. 별도 shellcheck·pyflakes는 실행하지 않았다.
