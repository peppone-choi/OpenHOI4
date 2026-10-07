# 결정 요청 REQUEST-0009 — WP-13 종료 경계와 플래그 수명

| 항목 | 값 |
|---|---|
| 요청일 | 2026-10-07 |
| 관련 식별자 | REQ-TIME-04, REQ-AGD-05, WP-13 |
| 막는 WP | WP-13의 실제 종료 상태 전이·플래그 실행 의미만 |
| 막지 않는 WP | WP-24 검증 봉인·통합/CI, WP-13 단일키 AST·registry 인자/참조/host capability 설계 |

## 질문 (한 문장)

기존 데이터 종료와 선언형 효과 범위에서 아래 A의 시간·평가 root·플래그·종료 후 상태 보존 계약을 가역적인 잠정 행동으로 채택할 수 있는가?

01 §3.3은 데이터 종료 조건과 종료일·가중합 점수, 플레이어 항복 뒤 관전 선택을 정했다. 02 §5.6/5.7은 조건 AST·효과·시나리오 필드를 정했으나 날짜의 마지막 시각, 조건 평가 root, 플래그 수명 및 종료 뒤 queue/step의 상세는 미정이다. 현재 Simulation은 명령 처리→한 시간 전진→일일 시스템→원장→snapshot이며 clone commit으로 실패 원자성을 보존한다. 원 M0/M1은 종료 필드가 없다. 이 요청은 후속 항복·진영·관전·경제 공식을 새로 정하지 않는다.

## 선택지

| 선택지 | 결과 | 비용·위험 |
|---|---|---|
| A (추천) | 아래 계약을 WP-13에 한정 잠정 채택; 원 M0/M1과 v1/v2/v3는 그대로, 새 종료/flag 상태는 additive 형식 | 새 상태/저장·host adapter·독립 경계 검사 필요; 후속 실제 scoring/항복/관전 consumer는 별도 인수 |
| B | AST/registry 기술 부분만 진행하고 실제 종료/flags는 상세 결정까지 대기 | TIME-04 실제 실행과 WP-25 종료 재현 인수가 대기; 파서만 있는 상태를 완료로 기록할 수 없음 |

## 추천안과 이유

A는 기존 시나리오 범위에서 명시된 입력만 실행하며 후속 미구현 상태를 성공으로 가장하지 않는다. 부모가 승인 없이 이 의미를 제품 코드에 넣지 않는다. 구현자가 실제 producer 타입에 적용할 schema/ADR을 먼저 작성하고 다른 의미가 필요하면 다시 구체 대조한다.

### 날짜·조건·실행 시점

- `end_date`는 마지막 플레이 가능 Gregorian 날짜다. 정상 한 시간 전진으로 다음날 00:00에 도달하는 step에서 그 날짜 경계 원인을 기록한다. 마지막날 23:00까지 실행한다. `end_date < start_date`는 로드 오류; 같으면 그 하루만 실행하며 생성 시 즉시 종료하지 않는다. 날짜 계산 overflow는 오류이고 상태·queue·clock 전체를 보존한다.
- `end_conditions`는 optional 단일 조건 AST다. 최상위 결합은 기존 `all/any/not`를 데이터가 명시하며 목록의 암묵적 any/all을 만들지 않는다. 조건을 지정했는데 비어 있거나 null이면 오류. nation 값이 필요한 조건은 별도 명시 root 국가 태그가 필요하고 선택된 국가에 실제 존재해야 한다. 날짜만의 조건에는 국가 기본값을 숨기지 않는다. 어떤 국가라도 만족하는 자동 반복은 없다.
- 조건은 정상 초기 권위 상태 생성 뒤 한 번, 이후 시간 전진을 성공시킨 step의 시스템/원장 계산 뒤 평가한다. Pause 중 단순 preference 명령으로 조건 평가를 반복하지 않는다. 조건 평가는 state/RNG/queue에 부작용이 없다. 초기 조건이 참이면 tick0의 종료 상태를 생성한다. 재개는 저장된 종료 상태를 보존하고 이미 종료된 것을 다시 판정하지 않는다.
- 여러 원인이 같은 commit에서 충족되면 날짜, 조건, explicit `end_scenario`의 typed 원인을 안정 순서로 모두 보존한다. 마지막 writer가 이전 원인을 덮지 않는다. `end_scenario`는 현재 효과 실행 전체가 성공한 뒤 종료 후보로 commit하며 뒤에 실패한 효과가 있으면 종료도 플래그도 전부 rollback한다. 원인 code/string 출력은 현지화키/데이터 ID이며 새로운 숫자 콘텐츠 상수는 만들지 않는다.

### 종료 이후

- 종료 상태는 시나리오 전체의 권위 결과 checkpoint다. 종료 tick/date/hour·typed 원인과 실제 지원되는 결과 입력을 canonical/hash/save에 포함한다. 별도 format은 종료·flags 정의만 있고 값이 빈 경우에도 해당 정의 신원을 잃지 않는다. 원 형의 읽기/쓰기/bytes/hash와 fixture를 유지하며 migration·구형 거부는 하지 않는다.
- 종료 후 `step`은 명시 오류이며 state/queue/clock을 전부 보존한다. 새 gameplay 명령 enqueue도 거부하고 queue를 보존한다. 이미 예약된 미래 명령은 삭제하지 않고 checkpoint에 그대로 남긴다. UI의 결과 확인·조회·저장은 시간 진행을 요구하지 않는다. Pause/SetSpeed는 종료 후 새 enqueue에서 함께 거부하여 완료 checkpoint를 바꾸지 않는다.
- 플레이어 항복·관전 선택은 WP-20/22 이후 실제 consumer의 별도 행동이다. 이 A를 이유로 항복/관전 입력을 가짜 true/false로 실행하거나 현재 전체 종료와 동일시하지 않는다. 기존 01 §3.3의 관전 선택 요구는 유지하고 별도 후속 인수로 기록한다.

### 플래그

- 지금 지원되는 플래그는 명시 nation scope에 속하는 문자열 ID 집합이다. 정의에서 허용 키와 초기 국가별 집합을 검증하며 ID는 기존 `valid_id` 규약, 중복 초기 키·미존재 국가/키·null/잘못된 타입은 로드 오류다. `self`는 명시 root, `nation:<TAG>` effect scope는 그 실제 국가다. scope 복귀 뒤 원 국가를 유지한다.
- 플래그는 시나리오 생성부터 해당 실행의 수명 동안 유지되고 set/clear 외 자동 만료·시간 reset은 없다. 같은 국가의 이미 있는 키 set 또는 없는 키 clear는 idempotent 성공이다. 타국의 같은 키는 별도 상태다. state/faction/world flags는 이번 실제 지원이 아니므로 사용하는 팩은 host capability 로드 오류다.
- 효과 목록은 전체 clone에서 성공 후 commit한다. 깊이·실제 primitive 실행수 제한을 포함해 중간 오류면 모든 state·flag·종료 후보·RNG·queue를 보존한다. 조건 `has_flag`는 집합 조회만 한다. 새 경제/전쟁 primitive는 후속 producer 인수 전까지 constant/neutral/no-op으로 성공시키지 않는다.

### 점수·후속 producer 경계

01의 네 축(보유 승점·산업 역량·생존·진영 승리) 가중합 요구를 유지한다. 실제 VP·IC와 후속 WP-20 생존/진영 값의 accessor/단위·수명을 producer와 대조해야 한다. 지원되지 않은 축의 nonzero 가중치는 실제 Simulation host 로드에서 거부한다. 합성 host 산술은 실제 게임 결과로 기록하지 않는다. 명시 0 가중치의 미사용 축도 설명/스키마를 보존하며 최종 M2 gate의 전체 scoring 완료를 이 부분 지원으로 대신하지 않는다. 가중치 범위·연산형/overflow 및 결과 분해 원장은 담당 schema/기술 ADR에서 기존 Fx와 독립 산술로 구체화하고 게임상 새로운 normalization을 만들지 않는다.

## 독립 입력과 기대

| 입력 | 독립 기대 |
|---|---|
| start=2000-02-28/end=2000-02-29, tick47 | 2000-02-29 23:00, 미종료 |
| 같은 실행 tick48 | 2000-03-01 00:00, 날짜 원인으로 한 번 종료; 이후 step 실패·완전 동일 |
| start=end=2001-01-01, tick23/24 | 01-01 23:00 미종료 / 01-02 00:00 종료 |
| end<start 또는 root 국가 누락/미존재 | live 생성 전 로드 오류 |
| 초기 date_gte가 참 | tick0 종료; RNG/queue 변경 없음 |
| all의 참·거짓 / any의 참·거짓 | false / true; 암묵적 배열 결합 없음 |
| 날짜와 조건과 end 효과가 같은 commit에서 참 | typed 세 원인 안정순서, 저장/새process재개 동일 |
| NTH flag x set 두 번·clear 두 번, STH x set | NTH 비어 있음 / STH x 존재; 읽기 RNG불변 |
| NTH set→STH scope set→scope 복귀 후 NTH clear | NTH 없음 / STH 존재 |
| set_flag→end_scenario→실제 실패 효과 | 모든 전 상태/queue/clock/종료/RNG 동일 |
| 종료 checkpoint에 미래 Move 보유 | queue 원본 유지, 새 Move/Pause/Speed 거부, decode/reencode/freshprocess 동일 |
| unsupported scoring 축 nonzero | capability 로드 오류; 0/false 가짜 입력 생성 금지 |

합성 날짜·태그·flag는 테스트 입력이며 Testland 콘텐츠 값 승인이 아니다. 독립 verifier는 자체 산술과 실제 source/입구를 검사하며 이 문서 예제를 복사한 단언만으로 PASS하지 않는다.

## 기본안으로 먼저 진행할 경우 되돌림 비용

채택 전 제품 행동 구현 없음. A 채택 후 새 WP-13 정의/상태/버전·tests만 더한다. 원 M0/M1·v1/v2/v3와 fixture/골든을 유지하므로 되돌릴 때 신규 형식과 신규 데이터만 별도 취급한다. 이미 생성된 신규 저장의 호환 파괴/삭제는 자동 승인되지 않는다. CI·샌드박스·네트워크/모델 설정과 사용자 D/OPEN 상태는 바꾸지 않는다.

## 사용자 답 (사용자 또는 사용자 답을 옮긴 메인 에이전트가 기록)

사용자 추가 확정 답 없음. CEO의 구체 권한 검토와 한정 잠정 채택 대기. 채택 결과는 이 절과 01/02/03·WP-13 schema/ADR에 대응한 뒤 구현한다.
