# ADR-1901 supply graph and scalar contract

| 항목 | 값 |
|---|---|
| 상태 | 독립 소스 단위의 기술 선택 채택; runtime 통합·독립 검증 대기 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-19, REQ-SUP-01, REQ-SUP-02, REQ-SUP-04 |

## 맥락

사용자는 2026-10-08 공유 철도 용량, 거리 손실, 첫 버전 고정 경로와 수정 가능한 데이터 수치의 초기 선택·검증을 승인했다. 기존 01 §4.9 보급원/수도 연결·통제·거리 규칙을 따른다. 철도 건설·손상·차량·사단 strength/소모 producer는 이 승인의 범위로 만들지 않는다. 현재 독립 단위는 실제 graph/formula 코드이며 shared schema/save/protocol 순서 WP15→WP19를 기다린다.

## 결정

- 순수 입력은 한 국가의 수도·통제/육지 노드, canonical undirected map edge, 별도 철도 cost/capacity, 명시 보급원, division-ID별 Qty 수요다. 입력 nation/실제 사단/건물/항구 해안/terrain/인프라 관계는 이후 trusted adapter의 책임이고 여기서는 없는 권위를 만들어 내지 않는다. ID0은 유효하다. 중복 소스 ID·프로빈스 배치·division ID, 불명확한 refs와 음수량/비양수 edge cost는 거부한다.
- 수도는 통제되면 길이0 self feeder다. 거점/항구 feeder는 통제된 철도에서 최대 bottleneck→최소 travel cost→전체 province-ID 경로 사전식으로 고정한다. Widest 결과의 threshold로 별도 shortest 계산하여 뒤 edge가 bottleneck을 낮출 때 잘못된 prefix를 보존하지 않는다. Rail capacity0은 길로 사용하지 않는다. 비용은 양수여서 zero-cost cycle 동점이 없다.
- Last mile은 Normal/RiverSmall/RiverLarge의 통제된 육지 cost graph다. 해협·해상·통과불가·외국 통제를 제외한다. 입력 cost 단위는 이미 계산된 reference travel-hour; 실제 terrain/infra/reference-speed 어댑터는 아직 없다. 보급 비율을 자기 거리 계산의 입력으로 쓰지 않는다.
- 감쇠 `a=max(0,1-floor(decay*distance))`는 Fx DR09 단계 순서다. Local Qty source budget은 additive이고 수도 예산을 복제하지 않는다. Source dispatch를 capital→source feeder의 공유 무방향 rail capacity에 한 번만 부과한다. Last mile에 같은 철도를 이중 부과하지 않는다.
- 하루 고정 source weights는 `C_raw*a_raw` 비율의 Hamilton floor+largest remainder(동점 SourceId)로 Fx 합1을 만든다. Trial delivery는 `floor(floor(D_raw*r_raw/Q)*w_raw/Q)`, dispatch는 `ceil(y_raw*Q/a_raw)`다. Trial마다 Hamilton delivery를 재배분하지 않아 monotonic feasibility를 보존한다. Split 잔여 raw units는 undelivered rounding ledger에 남긴다.
- Progressive filling은 unfrozen province ratio를 같은 raw increment로 늘리고 bounded binary search 후 포화 constraint 사용자/ratio1을 freeze한다. 매 pass 적어도 하나 freeze하거나 invariant error이며 pass≤positive reachable demand provinces, search≤33 evaluations다. 고정 weights가 대체 보급원 용량을 남길 수 있다. 최종 표시 비율은 실제 Y/D의 floor이지 target ratio가 아니다.
- Qty raw 중간량은 checked i128이다. Trial dispatch가 Qty 범위를 넘어도 wide capacity 비교로 infeasible로 판단한다. 최종 feasible ledger만 checked i64로 변환한다. Qty→Fx narrowing, float, RNG, I/O, 시계가 없다. 계산은 입력을 수정하지 않고 모든 오류에서 새 ledger를 반환하지 않는다. 전체 Simulation step atomicity는 후속 shared integration의 책임이다.
- Scalar shortage plan: `floor+(1-floor)*ratio`, speed floor>0, movement time=`1/speed`. Positive demand이고 ratio<threshold일 때 사단 소유 previous counter+1, equality/recovery/zero demand는0으로 reset한다. Counter>grace일 때 `zero_rate*(1-ratio)` 손실률이다. 실제 strength/조직/공격·현재 이동 leg 소비자는 없다. Zero demand는 모든 배율1·time1·손실0이다.

## 초기 provisional tunables와 민감도

모듈의 `defines.toml`은 실제 pack default가 아닌 검증 입력이다. Qty 단위는 supply unit/day이고 아직 실제 사단 수요 단위의 장착을 주장하지 않는다. 수도20·거점10·항구8, rail level1/2/3 용량10/20/40은 작은 합성 국가에서 공유 병목을 쉽게 드러내는 초기값이다. 철도 placement/건물 level/인프라 표는 실제 입력과 대조하여 후속 정의해야 한다. 차량 k/F/C scale은 미인수다.

Reference4km/h와 decay1/128 per reference-hour는 이상 지형512km에서 감쇠0이 되는 잠정 기준이다. 실제 graph는 Fx travel-hour를 받는다. 거리64에서 공급20은 전달10, decay를 절반으로 하면 전달15, 거리128에서는 전달0이다. 조직 floor1/4, 공격/속도 floor1/2는 공급0에서 time2를 보장한다. 비율0.5에서 조직0.625·공격/속도0.75·time≈1.3333이다. 기존 01 §4.9의0.3 기준을 `floor(0.3*2^32)`로 보존한다. Grace5일이면 equality는 reset, 5일 손실0, 6일부터 잠정 zero-rate1/16/day의 scalar 손실률이 시작한다. 이 값들은 balance 완료나 실제 manpower/equipment 손실의 승인·적용이 아니다.

## 검토한 대안

| 대안 | 장점 | 선택하지 않은 이유 |
|---|---|---|
| Adaptive routing | stranded capacity 회수 | 승인된 첫 버전 fixed routes와 다름 |
| Per-source bottleneck만 사용 | 계산 간단 | 승인된 shared rail conservation 위반 |
| Trial delivery largest remainder | 잔여 전달 감소 | 개별 source share 비단조로 binary search 전제 파괴 가능 |
| Single widest+shortest label | 한 탐색 | 후속 bottleneck 축소 시 잘못된 prefix 유지 |

## 결과와 영향

새 deps·pack mutation·module registration·현재 Simulation/hash/save/wire/클라이언트 변화는 없다. 모든 v1–v5 fixture/golden과 accepted WP15 저장 계약은 후속 통합 시 그대로 검증해야 한다. 이 단위의 별도 하네스 통과는 world save/server/browser·whole-WP19/M2 인수를 대신하지 않는다. 현재 leg 시간 보존/다음 leg 적용, division demand·strength·org/attack lifetime, 엄격 pack schema·공유 additive save·query 계약은 아직 실제 producer와 연결되지 않았다.
