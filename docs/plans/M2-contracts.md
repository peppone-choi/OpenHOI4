# M2 공유 스키마·인수 계약

2026-10-07, M2-r1. 구현 전 기획이며 아직 제품 구현/검증 증거가 아니다. 실제 담당 타입·경계는 각 WP-schema 문서에서 구체화한다. 원작 규칙/수치는 쓰지 않는다.

## 필드 분류와 소유

| 필드/도메인 | 의미·단위·형·범위 | 누락/null/default | 참조·수명·권위 | hash/save/wire/UI | REQ/WP |
|---|---|---|---|---|---|
| Pack manifest id/version/engine/depends/conflicts/load_after | 문자열 ID·SemVer 요구식, lexical 안정 순서 | 기존 M0 기본값만 유지, unknown 거부 | pack→pack 다대다, 강한 의존 DAG·load_after DAG, duplicate/missing/version/conflict 거부, host 불변 | 파일별 identity hash; sim I/O 금지; UI pack 정보는 현재 Welcome | MOD-01/02/04, WP-24 |
| Validation diagnostic path/line/column/severity/entity | 상대/절대 원 파일 문맥·1-based Unicode 위치·warning/error | 성공을 뜻하는 가짜 문맥 금지 | 원 source; validate 파생 출력; 상태 생성 전 실패 | hash/save 제외; CI 누락 키 검사 ko/en·FTL semantics | MOD-04/LOC-03, WP-24 |
| Trigger AST/primitive registry/scope | 키 하나 노드·compare Fx/date/bool·중첩16/효과1000의 기존 명세 | unsupported·잘못된 arg/reference 로드시 거부; chance는 event만 | nation/state/province typed ID, self/capital/owner는 구체 context; 선언형 정의와 mutable flag 분리 | AST/immutable registry pack identity, flags/end state는 canonical/save 포함; 실제 UI 결과는 후속 | AGD-05/TIME-04, WP-13 |
| Scenario end_date/end_conditions/score_weights | Gregorian date·조건 목록·데이터 가중치; score 축은 01 §3.3 | legacy M0/M1 미지정은 기존 behavior 유지; M2 필수 조건은 명시 | scenario 불변 정의, 종료 상태는 server 권위, once 종료/관전 의미를 설계 | 종료 원인/시점/점수 적용 입력 모두 save/hash; 결과 wire/runtime 함께 대조 | TIME-04, WP-13 |
| Movement division_id/nation/position/route/progress | DivisionId u32, NationId/ProvinceId u16; 거리 km·속도 km/h Fx, 이동시간 hour Fx | zero ID는 유효, 배열 index 아님; 속도0/음수 및 overflow 거부 | unit→nation 1, route→province N; ID sorted; 군/장비/훈련은 WP-16; live command가 단일 변경 경로 | mutable route/progress/pending 명령 canonical/save 포함; 새로운 wire는 runtime 확인; 전체 조작 UI WP-22 | MIL-04, WP-17 |
| Movement graph/policy/modifiers | MapData edge.distance_km, impassable·land/sea/lake 분리; terrain/infra/supply/river Fx 계수 | missing coefficient를 숫자1로 은폐하지 않음; 없는 전투/통행권 규칙은 후속 context | 기존 불변 geometry 단일 기준; tie-break ID/path lexical; mutable 위치와 정의 분리 | source identity/define hash; 경로 조회는 파생; 육해 이동 범위 구분 | MIL-04, WP-17; WP-19/20/31 후속 |
| Repro bundle metadata/start/ordered commands/end/expected hash | ZIP bundle.toml·commands.log machine/human·선택 start.ohsave, tick u64·hash u64 | 구조/버전/미지정·중복·미지존재/경로 escape·size/overflow 거부 | snapshot와 pending/live command 중복 재생 금지, pack identity 일치; start state 생성 후 chronological 동일 큐 | 모든 신규 command와 save adapter 포함, legacy 실행 유지; CLI 전용 | SAV-05, WP-25 |
| Bench baseline/current/threshold/samples | host elapsed·지표, 같은 runner 연속 기준+현재, 15% 초과 2회 | baseline absent/잘못된 수치를 성공으로 취급 금지 | wall clock은 host bench에서만; sim 작업량 조건 동일 | sim hash 불변; raw 결과+기준 commit/환경 보존; bench CI 신규 파일 소유 | PERF-03, WP-25 |

## 관계·갱신·실패 경계

`Pack[N] → Manifest/Defines/Map/Scenario → Defs → Simulation → ordered Commands → World/Movement/End → read-only DTO`를 단일 경로로 둔다. `Nation → State(owner)`와 `Province(controller)`는 별개다. `Division → Province(position)`의 장비/편제 정의는 WP-16에서 더해지며 WP-17은 경로/이동 입력과 mutable 위치만 소유한다. 데이터 참조 ID는 위치가 아닌 typed ID로 검사한다. graph 순환은 경로에서 허용하고 dependency DAG 순환은 거부한다.

파싱/구조/참조/초기 생성에서 실패하면 live 상태를 만들지 않는다. 입력 거부는 큐와 상태를 보존한다. 예정 명령 의미 오류는 명령 결과 거부·해당 큐 소비와 성공 step clock 진행을 구분한다. 전체 phase 계산 오류는 clone commit을 하지 않으며 state/queue/clock을 보존한다. atomicity와 범위를 각 WP가 테스트로 증명한다.

WP-13 registry의 미래 primitive는 검증된 AST/argument/type 계약과 명시 실행 adapter로 둔다. WP-14/15/20의 아직 없는 값·효과를 constant/default/no-op 성공으로 흉내내지 않는다. 초기 원시 항목 전체의 문법 테스트와 현재 실제 지원 adapter 및 후속 미구현을 별도 표로 표시한다. 종료일·공통 조건 엔진은 실제 Simulation 실행 경로에 연결한다.

## 독립 경계 예제

| 입력 | 기대 |
|---|---|
| 순서 다른 pack 입력/diamond depends 및 비연속 nation/province IDs | 동일 정렬 identity/결과; 중복/미존재/순환은 file/line 오류 |
| trigger 키2개/잘못된 scope/알 수 없는 effect/깊이17/효과1001 | 상태 변경 전 거부; 깊이16/효과1000은 경계 검사 |
| end_date 직전/같은날/다음날·저장 재개 | 설계한 시간 경계에서만 종료, 종료 상태/원인 hash·restore 동일 |
| 거리10km·속도2km/h·보정1.5 | 이동시간7.5h의 정확 Fx 원장; 실제 tick 진입·잔여 시간 반올림은 구현 전 ADR로 명시 |
| 동점 경로·시작=끝·막힌 edge·고립·없는 ID·속도0/음수·overflow·다른국가 명령 | 명세 tie-break/빈 경로/거부, 실패 state/queue 보존 경계 확인 |
| 중간 이동+예약 명령 저장 후 새 process 재개 | 전체 구조와 canonical hash/위치/경로/진행 동일; legacy M0/M1 fixture/hash 그대로 |
| repro 기대 hash 한 bit/pack byte/명령 순번/번들 중복/ZIP escape 변조 | 원 실행은 재생 일치; 변조는 정확 원인 실패 및 파일/live 상태 보존 |
| bench 독립 표본 current=115/116, baseline=100, 실패 1회/2회 | 15% 초과 경계와 두 회 재현 판정; NaN/0기준/누락은 거부 |

위 산술은 자체 합성 검증 입력이며 콘텐츠 기본값 승인이 아니다. 이동 반올림/보정 적용순서와 종료 점수의 미정 게임 의미는 담당 설계에서 확인하고 필요하면 P-10으로 CEO에 전달한다.

## 저장·버전·콘텐츠

현재 FORMAT_VERSION=1/SimulationSaveV1·m1-v1 fixture와 M0 canonical golden은 동결한다. 기존 v1 state에 신규 필드를 덧붙여 Postcard 표현을 바꾸지 않는다. 새 format reader/writer를 추가할 경우 v1→v1 왕복·실행/hash와 신규 상태의 별도 version roundtrip을 동시에 증명한다. 기존 저장을 다른 포맷으로 변환하거나 거부하는 작업은 별도 구체 승인 전 실행하지 않는다. 모든 mutable 입력/queue/RNG/보정·만료가 새 상태 canonical/저장에 들어가야 한다.

Testland의 M2 콘텐츠는 새 scenario `m2`와 전용 지도/초기값으로 둔다. 기존 M0는 examples/m0/testland, 기존 M1은 testland/scenarios/m1이다. 팩 전체 content_hash 변경 때문에 old M1 save를 새 팩에 그대로 열 수 없는 기존 경계를 새 호환 파괴와 혼동하지 않는다. frozen 기존 fixture는 원 pack copy/identity로 검증하며 신규 pack identity를 별도 기록한다. 새 schema version/파일 필수성과 후속 레지스트리 정의는 producer 출력과 함께 인계한다.
