# ADR-2202 정상 편제 Qty 기여 기록

| 항목 | 값 |
|---|---|
| 상태 | 채택 — producer 한정, wire/UI 계약은 보류 |
| 날짜 | 2026-10-10 |
| 관련 WP·REQ | WP-22, REQ-UI-04 |

## 맥락

정상 편제는 Qty 합산7개, 인력 가중 평균3개, Fx 최저 속도로 계산한다.
기존 StatLedger는 Fx와 modifier Add/Mul 의미론이므로 이 계산에 적용하면
단위와 연산을 왜곡한다. Normal은 실제 사단·훈련·저장에 들어가므로 표시용
기록을 그 안에 추가하면 기존 canonical/hash/save 계약을 바꾼다.

## 결정

군사 전용 `aggregate_with_ledger`는 기존 Normal과 별도 NormalQtyLedger를
반환한다. 기존 aggregate는 같은 계산에서 Normal만 반환한다. strength,
soft_fire, hard_fire, defense, breakthrough, frontage, supply_use의 실제
checked 합산 루프가 값과 누적값을 Qty 그대로 기록한다. base는 Qty0이고
정상 결과와 마지막 누적값은 같은 계산 결과다. 오류 순서·문구를 유지하며
실패 시 부분 기록을 반환하지 않는다.

source는 component ID, occurrence는 데이터 로더의 role별 정렬 후 position이다.
combat 다음 support 순서를 유지하고 반복 component를 별도 기록한다.
기록 ID는 `{field}:{role}:{position}`다. 진단 타입은 Serialize/Deserialize를
구현하지 않고 Normal·정의 identity·군사 상태·canonical/hash/save에 넣지 않는다.
가중 평균과 최저 속도를 Add/Mul 원장으로 설명하지 않는다.

## 표시·호환성 경계

선언된 template supply_use는 실제 사단 보급 소비나 감편 효과가 아니다.
producer에는 가변 tick/nation이 없으며 미래 조회의 권위 provenance는 별도
wire 계약에서 정의해야 한다. 이번 결정은 기존 MilitaryView, legacy 응답,
generated TS 또는 client validator를 바꾸지 않는다.

기존 exact template 검증과 이미 열린 구 client 때문에 필수 원장을 기존
응답에 무조건 추가하는 배포 전제는 확인되지 않았다. 유효한4096개 최대
구성 입력에서는 제안 DTO의 원장만69MiB 이상이 될 수 있다. versioned opt-in,
template별 또는 bounded page 조회, 명시 packet budget과 scope 수명은 부모의
보완 계획 뒤 결정한다. 이번 ADR이 그 프로토콜을 채택한 것은 아니다.

## 결과와 영향

독립적인 Qty·순서·발생별 기여·checked 오류·직렬화 보존 검사가 가능하다.
기존 aggregate가 진단을 만들고 버리는 추가 allocation 비용은 있으며 전체
성능 검사는 후속 게이트다. 새 의존성·게임 규칙·팩 변경은 없다. 실제 tooltip,
독립 검증·리뷰·CI 통합과 전체 WP-22/M2는 이 결정으로 완료되지 않는다.
