# ADR-0402 실제 시뮬레이션 3OS 해시 CI 증거

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-04, REQ-GEN-04, REQ-GEN-06, REQ-PLAT-02, AC-M0-02 |

## 맥락
코어 결정론 probe는 게임 시뮬레이션을 실행하지 않는다. AC-M0-02에는 실제 빈 시나리오 1,000틱의 반복 실행과 지원 세 OS 증거가 필요하다. 사용자 추가 배정으로 별도 workflow와 수집·비교 도구를 WP-04가 소유한다. 기존 core-determinism.yml과 ci.yml을 변경하지 않는다.

## 결정
`sim-determinism.yml`은 매 push/PR/수동 실행에서 ubuntu-latest, windows-latest, macos-latest를 모두 실행한다. 각 OS가 `cargo run --locked --quiet -p oh_cli -- run --scenario testland --ticks 1000 --seed 1 --hash-out`을 두 번 실행한다. 수집기는 종료 코드 성공·정확한 소문자 hex 16자리·반복 일치를 요구하며 run-1.txt, run-2.txt, sim-hash.txt를 아티팩트에 보존한다. 이전 성공 표식은 재수집 전에 지워 실패 후 오래된 성공 표식이 남지 않는다.

별도 compare job은 always 조건으로 실행한다. OS 이름별 아티팩트 세 개가 정확히 있어야 하고, 각 두 출력과 해시 파일 및 OS 사이 해시가 모두 일치해야 한다. 누락·여분·잘못된 hex·실행 간 불일치는 실패한다. Python 실패 경로 테스트도 각 OS에서 실행한다. 모의 아티팩트 테스트는 게이트 검사만이며 실제 OS 실행 증거로 간주하지 않는다. 루트 오케스트레이터가 브랜치/main CI와 실제 아티팩트를 수집하기 전에는 세 OS 증거가 있다고 선언하지 않는다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 기존 코어 해시 증거 사용 | 기존 CI 활용 | 실제 1,000틱 시뮬레이션을 검사하지 않음 |
| 아티팩트 중 있는 것만 비교 | 비교 코드 단순 | OS 누락에도 성공할 수 있음 |
| macOS를 야간에만 실행 | 실행 시간 감소 | 공개 저장소 세 OS 매 push/PR 지시와 충돌 |

## 결과와 영향
Rust는 기존 고정 toolchain 1.99.0을 사용한다. 외부 Actions는 공식 저장소 tag의 SHA로 고정했다. 2026-10-06에 공식 릴리스 페이지 및 `git ls-remote`의 정확한 refs/tags 출력으로 확인했고 `docs/worklog/evidence/WP-04/action-refs.txt`에 보존한다.

| Action | 버전 | SHA | 공식 출처 |
|---|---|---|---|
| checkout | v7.0.1 | 3d3c42e5aac5ba805825da76410c181273ba90b1 | https://github.com/actions/checkout/releases/tag/v7.0.1 |
| setup-python | v7.0.0 | 5fda3b95a4ea91299a34e894583c3862153e4b97 | https://github.com/actions/setup-python/releases/tag/v7.0.0 |
| upload-artifact | v7.0.1 | 043fb46d1a93c77aae656e7c1c64a875d1fc6a0a | https://github.com/actions/upload-artifact/releases/tag/v7.0.1 |
| download-artifact | v8.0.1 | 3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c | https://github.com/actions/download-artifact/tree/v8.0.1 |

배포·쓰기 권한은 추가하지 않으며 contents: read만 사용한다. 로컬 Windows 해시·단위 테스트·모의 비교 증거와 GitHub 실제 세 OS 증거를 구분한다.
