# WP-16 작업 로그 — 실제 군사 native host 초기 checkpoint

| 항목 | 값 |
|---|---|
| 상태 | 진행, 컴파일·동작 검사 전 checkpoint |
| 담당 | 구현 세션, native host·합성 fixture 담당 |
| 브랜치 | `wp/16-training-native` |
| 대상 REQ | REQ-MIL-03, REQ-MIL-05, REQ-MIL-09, REQ-ECO-07의 실제 native 입력·조회·저장 경계 |
| 선행 WP | 인수된 WP-15 production 및 WP-16 정상 편제, 동시 구현 중인 실제 military core·V7/wire |

## 계획

- 실제 pack military authority와 실제 `SaveContext::simulation`을 사용한다. 별도 훈련 계산기를 만들지 않는다.
- 군사 명령·조회는 서버와 같은 `oh_proto::MilitaryCommand`/`MilitaryView`를 재사용한다.
- stdin JSON-lines는 실제 queue/step/query/save를 구동하며 fresh process resume를 검사한다.
- synthetic fixtures는 기존 M1을 임시 복사한다. 실제 shipped M1/M2와 기존 golden은 수정하지 않는다.

## 테스트 우선 기록

| REQ | 검사 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| 군사 native host 진입점 | 기존 실제 `oh_cli military run ...` 실행 | exit 1, 기존 run usage 반환 | 미실행 |
| REQ-MIL-03/05/09, REQ-ECO-07의 native 경계 | `crates/oh_cli/tests/military.rs`의 새 실제 프로세스 검사 10개 | source 준비; 정책 전체 historical RED로 주장하지 않음 | 미실행 |

진입점 RED는 실제 기존 실행 파일의 명령 미지원 증거다. 기존 M1에는 군사 authority가 없으므로 그 입력을 군사 정책의 RED로 해석하지 않는다. 컴파일 실패를 요구사항 실패로 기록하지 않는다.

## 실행한 검증

| 명령 | 종료 코드 | 결과 |
|---|---|---|
| 기존 `oh_cli military run --pack data/packs/testland --scenario m1 --seed 1`, stdin query | 1 | 새 command 미지원, 기존 usage |
| Rust 1.99.0 `rustfmt --edition 2024` 담당 Rust 경로 | 0 | source 형식 정리 |

Cargo/npm/클라이언트/서버/whole suite는 아직 실행하지 않았다. 공유 target 컴파일 슬롯은 core 담당 소유이며 native source는 API 제공 뒤 통합해야 컴파일할 수 있다.

## 증거

- 새 native process 검사는 pending backfill·다음 날 진행·Ready·감편 배치·실제 예약 반환·취소 terminal·paused 명령·command-before-clock·군별 priority와 cross-army flat Hamilton·capacity shrink·joint stock/held cap 원자성·실제 V7 fresh resume와 byte 동일성을 구동하도록 준비했다.
- 구체 결과는 실행 전이다. `tests/fixtures/military/README.md`가 합성 수치·재현 입력·후속 범위를 설명한다.
- 기존 명령 실행 원본 출력은 비공개 환경 기록으로 보존한다. 소스에 내부 run record를 넣지 않는다.

## ADR

- 승인된 네 훈련/보충 정책을 실제 core가 계산한다. Native는 기존 서버 DTO를 재사용하는 기술 경계이며 새로운 게임 정책은 없다.
- 입력 line 1 MiB와 request당 백만 step은 host 자원 제한이다. 게임 시간·훈련 기간·배치 제한이 아니다.
- native checkpoint는 byte 비교를 위해 진단 timestamp 0을 사용한다. 실제 Simulation 상태와 queue는 그대로 저장한다.

## 원작 대비 차별화 초안

- 원작 코드·데이터·수치 비교 없이 독립 합성 입력을 실제 권위 Simulation과 같은 서버 DTO로 구동한다. 오케스트레이터가 승인된 문서 범위와 통합한다.

## 결정 필요

- 새 게임 정책 질문 없음. 네 사용자 승인 범위만 사용한다.

## 범위 밖 발견

- shared military core·schema/save/V7/wire/server·generated TypeScript는 별도 구현 담당 소유다.
- actual movement/combat/supply/general modifier/editor legality/AI는 이 native host에서 만들지 않는다.

## 못 한 부분과 이유

- 현재 checkpoint에서 core/wire 의존 소스가 아직 같은 worktree에 통합되지 않아 compile·semantic GREEN·whole checks는 대기한다.
- `oh_cli`가 기존 workspace `oh_proto`를 추가하므로 shared Cargo.lock의 해당 package dependency entry는 core 담당자에게 일임했다. 새로운 crate/version은 없다.
- 독립 검증과 같은 main CI는 오케스트레이터의 고정 통합 커밋 이후에 필요하며 이 checkpoint를 완료로 기록하지 않는다.
