# ADR-1902 — Real World/Map network boundary

| 항목 | 값 |
|---|---|
| 상태 | 구현 후보; 새 독립 검증·통합 대기 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-19, REQ-SUP-01/02/04의 graph-input 부분, REQ-MIL-04 기존 비용 계약 |

## 맥락

Accepted main `6d1d1d9278c7771be9b9ba2943e745a9094b9104`의 WP15 producer 계약과 이미 인수된 WP19 standalone 계산을 연결하는 작은 소스 단위다. 사용자가 승인한 공유 철도 용량·거리 감쇠·첫 버전 고정 경로·provisional 수치는 유지한다. 독립 함수 등록과 실제 World/Map 입력 경계만 추가한다. 공급 daily phase/사단 demand·strength·org/attack/현재 leg 소비·save/wire/UI는 아직 없다.

## 결정

- `oh_sim::supply`를 정규 모듈로 등록한다. 기존22개 검사 assertion/기대 범위는 보존하고 변경된 입력형의 생성부만 조정한다. 공유 `Simulation` fields/phase/hash/body/commands는 바꾸지 않는다.
- `Network`는 nation/capital/nodes/land/rails/sources/decay를 가지며 demand를 갖지 않는다. `Input`은 Network와 명시적인 DivisionId/Qty demand 벡터를 분리한다. 어댑터는 Network만 만들고 empty/zero demand나 dummy division을 만들지 않는다. `feeders()`는 static feeder 연결 메타데이터이며 fulfilled Day가 아니다.
- `Node.controller:Option<NationId>`로 None과 유효 ID0을 보존한다. Eligibility는 land와 `controller==Some(nation)`이다. 현재 유효 World는 state가 있는 land에 Some controller를 보장하며 restore는 None land를 거부한다. 실제 water None 전달과 pure graph land None 차단을 각각 검사한다. 유효 land None을 만들기 위해 World/restore 규칙을 바꾸지 않는다.
- 각 canonical edge의 `cost`는 a→b, `reverse_cost`는 b→a이다. Capacity는 무방향으로 하나만 공유한다. 실제 이동의 목적지 terrain/owning-state infrastructure는 비대칭일 수 있으므로 평균/min/max로 비용을 대칭화하지 않는다. Widest capacity→shortest directed cost→full path lex tie 정책은 그대로다.
- `world_adapter::prepare(&World,&Defines,NationId,&TransportTuning,&StaticMetadata)`는 real Map/World의 국가·수도·province refs·current control/land/coastal·actual edge kind/distance·현재 state infrastructure를 읽는다. Normal/RiverSmall/RiverLarge 육지 edge만 reference graph에 들어가며 sea/lake/Strait/Impassable은 공급 rail이 되지 않는다.
- `movement_hours`와 `Factors::from_defines`의 기존 순서 distance/reference speed→목적지 terrain→명시 infrastructure time factor→reference supply factor1→river를 양방향에 적용한다. 이 factor1은 fulfilled ratio를 뜻하지 않는다. Reference speed는 명시 Fx>0, decay는 Fx>=0이고 해당 실제 level의 positive factor를 exact table로 찾아야 한다. 누락·0·음수·overflow·rounded-zero 비용은 오류다. Reference4km/h 등 테스트 수치는 기존 승인된 provisional이며 runtime default가 아니다.
- Static rails는 actual eligible map edge의 canonical refs와 명시 positive level이며 같은 caller tuning의 positive Qty capacity table을 정확히 참조한다. unknown level은 MissingContext이고 중복·잘못된 refs는 거부한다. 이것은 World 건설/손상 level을 가장한 state가 아니다.
- Sources는 명시 metadata이며 capital 위치는 실제 nation capital과 같아야 한다. 기본 land/state/coastal 조건을 검사하되 hub/port의 실제 building-instance producer가 없으므로 UnsupportedBuildingInstance를 반환한다. State aggregate industry/buildings 또는 coastal flag만으로 보급원을 만들지 않는다.
- 어댑터는 &World를 받고 state·clock·queue·movement·economy·production을 바꾸지 않는다. 결과는 caller-owned 값이다. Canonical/hash/save/wire fields·현재 pack defaults·historical readers/writers·legacy fixtures는 그대로다. 분리된 demand 입력 이후의 순수 solver와 scalar effects는 여전히 실제 consumer 연결을 증명하지 않는다.

## 검토한 대안

| 대안 | 장점 | 선택하지 않은 이유 |
|---|---|---|
| 무통제 None→ID0 | 필드 단순 | 실제 국가0과 충돌하고 통제를 만들어냄 |
| 단일 대칭 비용 | 기존형 유지 | 목적지 지형/인프라 비용을 왜곡 |
| 인프라 level 자체를 factor/default1/interpolation | 즉시 연결 | 승인된 factor producer 정책이 아님 |
| 비어 있는 demand로 Day query | UI 연결 쉬움 | 실제 사단 producer를 가장함 |
| State building 집계에서 hub 위치 추정 | 입력 절약 | 건물 instance/위치 권위가 없음 |

## 결과와 남은 실제 계약

이 단위는 일반 Cargo가 supply 코드를 검사하고 실제 World/Map 경계값을 공급하는 API다. 전체 WP19·REQ/M2 인수나 플레이 기능은 아니다. 새 daily state·save V7·protocol/query·served UI·차량·건설을 추가하지 않는다. 두 새 process의 network raw 메타데이터 비교는 full Simulation/save 결정론 검사로 확대하지 않는다.

전세계 infrastructure factor 정책은 미정이다. 추천은 owning-state current Fx infrastructure에 대한 명시 positive table부터 시작하고 actual supported levels를 데이터에 지정하는 것이다. 변경된 fractional level이 표에 없으면 MissingContext를 유지하며 전 범위 curve/interpolation은 별도 정책 결정 후 정의한다. 현재 leg committed time 보존/다음 leg 신규 factor 적용, actual WP16 demand/strength/org/attack/lifetime, port/hub instances와 rail placement, WP44 C/k/vehicle reservation/stock-bound 및 supply-only additive save presence 계약이 필요하다. 초기 수치 위임을 이런 추가 게임 정책의 승인으로 확대하지 않는다.
