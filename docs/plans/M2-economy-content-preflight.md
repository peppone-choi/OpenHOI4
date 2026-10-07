# W2 경제·Testland producer 인수 대조

2026-10-07. W1 진행 중 오케스트레이터가 작성한 준비 문서다. W2는 W1 전체 통합 뒤 시작한다. WP-14와 WP-23은 서로 다른 파일을 소유하며, 실제 스키마 출력이 생기기 전에 상대 필드를 추측해 작성하지 않는다.

2026-10-08 인수 갱신: 아래 미정 표는 작성 당시 이력이다. 실제 게임 의미는 [REQUEST-0010 A-CEO-r1 수정판](../decisions/REQUEST-0010.md)의 CEO 한정 잠정을 채택했으며 원 PC 선지급 후보는 채택하지 않았다. [WP-14 배정·producer 계약](WP-14-M2-r2-contract.md)에 파일 책임·Fx/Qty 단위·phase·호환·WP-23 전달을 정리했다. 아직 W1 전체 통합이나 W2 구현 착수가 아니며 아래 역사 ff78은 최신 통합 소스로 쓰지 않는다. 현재 main의 실제 WP-13/17/24 및 WP-25 독립 검증 인수 뒤 담당 schema가 실제 타입/필드/최대값을 고정한다.

## 실제 기존 모델과 새 시스템

M1은 국가 태그/ID/수도/정부/이념 지지율, 주 초기 인구·자원·건물·소유, 프로빈스 통제와 원장 기초를 표현한다. 국가 산업/안정도/법령/PC/가용 인력의 실제 일일 시스템 완료는 아니다. 원 M1 state와 v1 fixture·hash를 보존한다.

01 §4.2·§4.3은 주 공업 시설 수준 합과 보정치의 국가 IC, 네 부문 비율 배분, 소비재 하한, 순서 있는 건설 대기열/상한/비용/인프라 보정, 인구와 징병 비율의 인력, 일일 PC·0~100% 안정도/동원도·법령 범주/단계/비용/조건을 정했다. 각 합성 콘텐츠 수치가 이 구조의 새 게임 공식을 승인하는 것은 아니다. 소비재 하한과 안정도의 결합, 소유/통제에 따른 산출 범위, 건설 단계 전환 잔여 IC, 인력 초기 pool·일일 증감·훈련 예약의 수명 등 문서에 없는 의미는 구체 입력·결과 후보로 먼저 검토한다. 점령·세계 시장·장비 생산·동원 조직력 등 후속 규칙을 neutral 성공으로 구현하지 않는다.

## 단일 기준과 후속 인수

| 도메인 | WP-14 설계·실제 출력 | WP-23 콘텐츠 입력 | 후속 consumer |
|---|---|---|---|
| 국가 IC·배분 | 기준 주 집합·Fx 단위·원장 Add/Mul·합계/하한·실제 적용값 | 새 m2 초기 비율·공업 시설·defines | WP-15 생산, WP-19 보급·WP-13 종료 점수 |
| 자원 | 주 산출과 국가 집계·부족/비축 여부·mutable stock과 생산량 구분 | 자원 ID·새 주 산출 | WP-15 소요/충족률, WP-29 시장 |
| 건설 | 프로젝트 ID/건물/주·순서/상한/비용·완료/잔여/취소·슬롯 참조 | 건물 정의·새 주 슬롯/수준·보정 defines | WP-19/29 보급시설, WP-22 UI |
| 인력 | 초기 인구·가용 pool·수량 상한·징병 법령 비율·consumer 변경 API | 새 주 인구·국가 초기 pool·법령 | WP-16 훈련/보충, 후속 손실·통제 규칙 |
| 정치 | PC 단위/상한/일일량·안정도/동원도·법령 ID/category/step/조건/비용·원자성 | 새 국가 초기값·법령 definitions/ko/en·defines | WP-13 actual adapters, WP-20 전쟁, WP-22 UI |

표의 의미·수명·누락/null·기본값·범위·갱신 위치는 실제 WP-14-schema/타입의 필드별 표에 기록한다. 같은 값의 중복 cache와 원장 invalidation, constructor·command·phase·restore·wire 검증, 소유권/동점/정렬/비연속 ID, overflow·누락 define·실패 state/queue/clock 경계를 직접 검사한다. client는 결과 표시와 명령만 담당한다. 새 경제·정치 상태와 queue는 새 저장형·canonical에 누락 없이 포함하고 원 v1/v2/v3를 보존한다.

## Testland 경계

REQ-CNT-01은 6개국·프로빈스 100~200, REQ-GEN-03은 날짜 범위·이념·자원·장비·지형·건물 종류의 데이터 정의다. WP-23은 새 `m2` 시나리오·전용 map·초기값·자체 원천 기록을 만들며 원 M0/M1 파일은 덮지 않는다. 합성 기하/국가/현지화 등 기존 계약은 먼저 준비할 수 있다. 경제·정치와 registry 참조는 실제 producer 인수 후 쓴다. 장비·대대의 정의 계약이 아직 없다면 미정 필드를 만들어 파일을 성공 로드한 것으로 쓰지 말고 실제 WP-13/14의 참조 registry와 WP-15/16 후속 타입 경계를 부모에 보고한다.

팩 전체 content_hash 변화와 frozen 원팩 identity는 별도로 기록한다. 같은 pack 내 새 map/common/locale가 원 M1 동작에 영향을 주는지 직접 검증하고, 원 historical fixture는 정확 원천으로 검사한다. 미래 미등록 파일을 warning으로 두고 `--deny-warnings` 성공이라고 선언하지 않는다. WP-23의 실제 validate·6국/100~200·참조/원천·engine-agnostic domains와 전체 평시→전쟁→항복 플레이 증거는 구분한다. 전체 플레이는 후속 WP-15/16/18/19/20/21/22·M2 게이트가 필요하다.

W2가 통합됐다는 기록은 WP-14/23의 실제 구현·새 독립 PASS·같은 main CI에 한정한다. M2 전체 AC/REQ를 이 두 WP로 완료했다고 기록하지 않는다. 차량 v2의 M4 한정 잠정은 WP-15 실제 재고/상한·WP-19 공급 C 단위 인수를 위한 후속 검사 항목이며 W2의 보급 차량 소모 구현 권한이 아니다.

## M2-r2 실제 producer 인수 전 확인

2026-10-07. WP-13 제출 source `ff78a7501bace6fac1741d6cc596be524f887451`은 독립 P-05에서 runtime guard·현재 HEAD comparer 누락이 재현되어 수정 대상이다. 아래는 제출 타입을 읽은 계획이며 그 source의 통합·독립 PASS를 뜻하지 않는다. W2 착수 전 최종 수정 source의 유효 PASS·P-07·같은 main 필수 CI와 실제 타입을 다시 인수한다.

- 기존 World의 StateState는 인구·자원·건물 i64와 인프라 Fx/원장, NationState는 ID·정부·이념 지지율만 갖는다. 건물 키를 임의 산업 시설로 해석하거나 기존 주 자원 산출을 비축으로 다시 쓰지 않는다. WP-14의 새로운 immutable 정의·초기 입력·mutable 소비/예약/원장·derived 집계를 각각 구분한다.
- WP-13의 새 TriggerState/SaveV4는 optional trigger 정의 신원·국가 flags·종료 checkpoint와 전체 pending queue, movement/strait presence를 보존한다. WP-14는 flags/초기평가/종료/과거 queue를 다시 초기화하지 않는다. 일일 경제 phase가 성공한 뒤 종료 조건을 평가하고 phase 오류는 전체 상태·큐·clock을 commit하지 않는 기존 순서를 유지한다.
- stability/mobilization/political_capital/has_law 및 해당 효과는 현재 실제 producer가 없어 capability 오류다. WP-14가 실제 private mutable 타입·bounds·단위·조건 참조와 effect 원자성을 제공할 때만 이 adapter를 연결한다. 산업 종료 점수는 실제 국가 IC Fx·원장·시점 계약을 인수하며 비슷한 기존 건물/인프라를 대용 값으로 쓰지 않는다. 나머지 점수 축·항복/진영은 후속 WP-20 범위다.
- 새로운 경제 상태의 canonical/hash/save/wire는 raw 정수 bits·정렬·nullable/presence·정의 identity·future queue를 모두 포함한다. 원 None/v1/v2/v3 및 WP-13 v4 fixture/expected/hash는 보존한다. 필요하면 별도 additive 버전으로 만들며 과거 저장 자동 migration·기존 입력 덮어쓰기·force의 mode identity 면제는 승인하지 않았다.
- WP-23은 원 m1 scenario/국가/map/정의·locale bytes를 유지하고 새 m2 전용 원천을 쓴다. 같은 pack에 파일이 추가되면 팩 전체 hash가 달라지므로 기존 fixture를 현재 팩으로 다시 만들지 않는다. 현지화 key 집합과 pack-level strict validator가 원 m1과 새 m2 각각을 검사하는 경계를 실제 producer schema와 대조한다.

실제 결과를 바꾸는 미정 행동은 WP-14 schema 작성 중 구체 입력/결과/추천안으로 먼저 P-10 요청한다. 최소 대조 목록은 다음과 같다. 이 표는 추천안의 채택이나 새 게임 규칙을 뜻하지 않는다.

| 미정 의미 | 구현 전 필요한 구체 후보·독립 예제 |
|---|---|
| 국가 산출의 주 집합 | 소유와 통제가 다른 주의 IC·자원·인구 기여, 점령 후속과 구별 |
| 소비재 하한 | 경제 법령/안정도 결합 공식과 범위, 하한 미달 배분 거부 또는 보정, 합계·raw 잔여 |
| 건설 | 프로젝트별 IC 상한·순차 배분, 당일 완료 잔여의 다음 프로젝트 전달, 슬롯/단계 상한·취소/비용 수명 |
| 가용 인력 | 법령 변경 전후 총량과 이미 소비/예약한 인력, 초기 pool·추가/반환의 수명·음수/상한 |
| 정치 | PC 일일 지급 시점/상한, 법령 변경 비용·조건·단계 및 효과 delta의 경계 처리 |
| 후속 미지원 | 안정도의 생산 효율/항복 보정, 동원도의 조직력/피로 등 아직 없는 consumer를 지원 성공으로 숨기지 않기 |
