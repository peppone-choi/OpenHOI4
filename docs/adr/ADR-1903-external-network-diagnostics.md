# ADR-1903 External network configuration and initial World inspection

| 항목 | 값 |
|---|---|
| 상태 | 채택 — 기술 경계; 독립 검증·통합은 후속 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-19; REQ-GEN-02, REQ-SUP-01/02/04 입력·진단 부분 |

## 맥락

인수된 순수 네트워크 어댑터는 실제 World/Map에서 비용·통제·수도를 읽지만 호출자가 직접 계수와 정적 배치를 만들어야 했다. 실제 Division 수요·strength/org/attack·수명·효과 소비자는 아직 없다. 사용자가 승인한 공유 유한 철도 용량·거리 손실·첫 버전 고정 경로·조절 가능한 provisional 수치를 유지하면서, 명시적 입력의 재현과 오류 진단만 개선한다.

## 결정

외부 TOML 파일을 명시적으로 읽는 `oh_data::supply_network`와 `oh_cli supply-network inspect`를 추가한다. **팩에 등록하지 않고** Scenario/LoadedNational/WorldDefs에 필드를 추가하지 않는다. `--config`는 필수다. 선택되지 않은 일반 run/resume/save/server 경로는 기존 구현을 사용한다. 저장된 World를 검사하는 `--load`, step·daily 공급·division 생성·새 서버 조회는 지원하지 않는다.

형식 version1은 모든 필드를 필수로 받는다. 최상위 `version`, `tuning`, `nations`; tuning은 decimal-string `reference_speed_kmh`, `decay_per_reference_hour`, infrastructure level/factor 배열, rail level/capacity 배열이다. 각 nation은 u16 ID와 명시적 sources/rails 배열을 가진다. source는 u64 ID·소문자 capital/hub/port·u16 province·capacity string; rail은 u16 a/b와 양의 u32 level이다. 모든 구조의 알 수 없는 필드와 누락·타입 오류를 거부한다. TOML null은 유효 입력이 아니다. JSON 로더·새 JSON schema 파일은 이 단위에 제공하지 않는다.

숫자는 float를 거치지 않고 기존 fixed parser로 직접 checked Fx(I32F32)/Qty(I48F16)에 양자화한다. 문자열은 숫자와 선택적 소수점만 허용한다. 부호·공백·지수·NaN·infinity와 overflow를 거부한다. 속도·시간 계수·capacity는 양수여야 하며 양자화 결과0도 거부한다. level과 decay는 비음수다. Qty를 Fx로 좁히지 않는다. 현 수준 Fx→양수 시간 계수는 정확히 찾아야 한다. 기본1·새 curve·보간이 없다.

배열을 먼저 받아 각 원소의 중복을 검사한 뒤 정렬한다. 같은 Fx로 양자화되는 서로 다른 level 문자열도 중복이다. nation ID, infrastructure level, rail capacity level, **각 nation 내부**의 source ID/placement와 rail edge를 중복 거부한다. SourceId는 기존 per-nation Network의 로컬 이름 공간이므로 서로 다른 nation의 같은 ID는 허용한다. rail은 canonical a<b만 허용하며 역순을 조용히 바꾸지 않는다. explicit sources/rails 빈 배열은 유효한 정적 메타데이터다. nation과 infrastructure 선언은 비어 있을 수 없다. rail_capacity 빈 배열은 rail이 없을 때만 유효하다. 이 의미는 시스템 부재나 공급 비율1이 아니다.

형식 자원 상한은 문서1,048,576 bytes, decimal64 bytes, nation1,024개, 개별 계수/각 nation source/rail4,096개, 전체 source+rail16,384개다. 게임 coefficient가 아니라 bounded parser의 기술 상한이다. 파일은 UTF-8 regular file만 읽고 max+1 bytes까지 제한한다. 경로 자체로 팩 등록이나 권위를 얻지 않는다.

참조 검사는 실제 nation/capital, 육지·주·해안, positive Normal/SmallRiver/LargeRiver edge와 명시 rail level을 확인한다. oh_data는 oh_sim에 의존하지 않는다. `config_adapter::prepare`가 실제 World 정의에 다시 확인하고 **모든 실제 현재 주 수준**의 계수를 요구한다. 격리/빈 네트워크에서도 누락을 허용하지 않는다. accepted `world_adapter::prepare`와 `Network::feeders`를 호출하며 비용·경로·공유 capacity 알고리즘은 변경하지 않는다. 실제 building-instance producer가 없으므로 valid hub/port도 `UnsupportedBuildingInstance`다. 정적 rail은 건설·손상·점령 lifecycle의 권위 상태가 아니다.

정규 config identity는 `(external-supply-network-v1, fixed-feeder policy ID, normalized Config)`의 canonical encoding/FNV64다. 튜닝 map·nation map·source ID·rail endpoint 순으로 정렬하며 동등한 fixed 값·순서는 같은 identity다. 원본 파일 bytes hash와 다를 수 있다. 인증용 hash가 아니며 **진단 provenance만** 제공한다. World state_hash/팩 content identity/save presence로 사용하지 않는다.

CLI는 승인된 `load_national` 경로로 팩 검증·initial modifiers·initial World를 읽는다. Simulation을 만들거나 진행하지 않는다. 출력 scope는 `initial_world_network`; 실제 node owner/controller/land/state/terrain/coast, capital, 현재 infra, 방향별 reference travel-hours raw Fx bits, 정적 rail/source capacity raw Qty bits와 source feeder를 낸다. Controller None은 JSON null이며 NationId0은 숫자0이다. 연결 없는 source feeder null도 유효 네트워크 진단이다. demand/delivered/ratio/starvation/effects/divisions/state_hash/save/query authority 필드를 내지 않는다.

## 검토한 대안

| 대안 | 장점 | 이번 단위에서 제외한 이유 |
|---|---|---|
| scenario 선택 + common 문서 등록 | 팩 입력을 일관되게 선택 가능 | optional presence/정규 identity/미선택 파일/restore 계약이 별도 필요 |
| 기본 인프라1 또는 새 curve | 입력이 간단함 | 승인되지 않은 전 세계 정책과 누락 은폐 |
| server live query 또는 empty daily result | 조회 경로가 생김 | 실제 소비자·state·wire 계약이 없고 부재를 공급 결과로 가장함 |

## 결과와 영향

네이티브 진단을 재현할 수 있지만 보급 gameplay나 전체 WP-19/M2 판정이 아니다. schema/save/protocol/server/client/dependencies/locks/원본 packs/goldens는 변경하지 않는다. 실제 Division demand/effect, current-leg timing, 세계 인프라 표 정책, hub/port instances·동적 철도·차량 C/k·보급 저장 presence는 후속 계약이다. 합성 fixture의 4km/h·decay1/128·source20·rail10/20은 승인된 provisional 예시이며 infra factor2/1.5/0.5 등은 테스트가 명시적으로 고른 입력이지 전 세계 curve가 아니다.
