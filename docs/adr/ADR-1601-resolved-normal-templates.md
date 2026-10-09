# ADR-1601 — 외부 정상 편제 입력과 순수 집계

- 상태: 채택된 로컬 기술 설계, 통합 게이트 대기
- 대상: WP-16 / REQ-MIL-02, REQ-MIL-03의 정의·정상 능력치 부분
- 선행: WP-15 실제 생산 모델 레지스트리

## 배경과 범위

사용자가 장비 종류별 모델 명시, 능력치별 합산·인력 가중평균, 가장 느린 구성요소의 속도를 승인했다. 이 단위는 `military-template inspect`로 명시된 외부 TOML의 정상 구성요소를 집계한다. 기존 production Model에는 전투 능력치가 없으므로 입력은 **구성요소 작성자가 이미 해석한 정상 능력치**다. 장비 한 개의 화력 × 요구 개수를 계산하거나 장비와 대대 능력치를 중복해서 더하지 않는다. 모델 교체의 전투 효과를 도출하지 않는다.

실제 선택 시나리오의 LoadedNational.production, 국가, allowed_models, 모델 family를 검증한다. 구성요소가 작성한 정확한 family/model 문맥과 편제 바인딩이 일치해야 한다. 바인딩만 바꾸고 이전 정상 능력치를 조용히 재사용하는 것은 거부한다. 각 구성요소의 입력을 변경한 것은 작성자가 새 해석 문맥을 명시한 것이며 엔진이 장비 전투식을 유도한 것이 아니다.

사단 인스턴스·초기 배치·훈련·예약·환불·보충·전투·실제 수요/효과/손실·편제 UI는 구현하지 않는다. 편집기/배치 가능한 편제라는 판정도 하지 않는다. 지원 중복이나 최소 전투 대대 수 정책은 이 단위에서 정하지 않는다.

## 데이터 계약

문서는 `version = 1`, `components`, `templates` 배열을 모두 명시한다. 모든 노드는 알 수 없는 키를 거부하며 null/default를 제공하지 않는다. 전부 빈 배열인 정의 문서는 표현 가능하지만 없는 template 선택은 참조 오류다. 파일은 정규 파일·UTF-8이고 읽기/파싱 전에 최대 1 MiB를 검사한다. 구성요소·편제·종류별 입력 배열은 최대 4096개다. ID는 최대64 ASCII `[a-z0-9_]+`, 수치 문자열은 최대64 unsigned decimal 문자다. 이 한도는 자원/포맷 한도이며 밸런스 수치가 아니다.

구성요소는 `id`, `role = "combat" | "support"`, `manpower`, `equipment`, `stats`가 필수다. equipment 원소의 `family`, `model`, `items`도 필수다. 정의 ID·equipment family·binding family 중복은 원본 배열에서 검출하고 거부한다. maps/sets로 먼저 정규화하지 않는다. 편제는 `id`, `combat`, `support`, `bindings`를 명시한다. composition의 ID 반복은 보존하며 combat 최대12/support 최대4만01 §4.7에 근거해 검사한다. 참조와 role이 맞아야 한다. 바인딩은 모든 해당 실제 모델을 국가 허용 목록에 대조하며 사용하지 않는 추가 바인딩도 같은 검증을 받는다. 선택하지 않은 다른 편제의 국가 허용성은 현재 국가를 기준으로 추정하지 않는다.

| 필드 | 단위·범위 | 집계 |
|---|---|---|
| manpower, equipment.items | nonnegative whole i64 사람/개 | checked i64 합 |
| strength, soft_fire, hard_fire, defense, breakthrough | nonnegative I48F16 정상 선언 점수 | checked Qty 합 |
| frontage | nonnegative I48F16 전면 폭 단위 | checked Qty 합 |
| supply_use | nonnegative I48F16 보급 단위/일의 설명 데이터 | checked Qty 합, 실제 Demand 아님 |
| organization, armor, piercing | nonnegative I48F16 정상 선언 점수 | 인력 가중평균 |
| speed_kmh | strictly positive I32F32 km/h | 모든 구성요소의 최소 속도 |

I48F16 raw 상한은 i64::MAX이고 I32F32 raw 상한도 i64::MAX다. decimal은 f64를 통과하지 않고 기존 fixed parser로 한 번 양자화한다. fixed parser의 정밀도 반올림을 사용하며 각 입력을10진수 그대로 더한 결과라고 주장하지 않는다. 속도가 양자화 후0이면 거부한다. 모든 필수 능력치의 누락·숫자 타입·음수·지수·비정상/overflow 입력을 거부한다. 다른 Qty 값0과 인력/장비0은 표현 가능하다.

인력 가중평균은 모든 occurrence의 `Qty raw * manpower`를 checked i128에 모아 마지막에 한 번 나눈다. nonnegative 값의 나눗셈은 floor다. raw numerator remainder를 분모인 총 인력과 함께 진단 출력한다. 순차 평균이나 구성요소별 중간 나눗셈은 하지 않는다. 예를 들어 organization30@1000과60@100은 약32.727 점이며 단순45가 아니다. 인력0 지원은 가중치0이며 속도·합산 능력치는 여전히 해당 선언을 반영한다. 손실 소수 이월 상태를 만들지 않는다.

빈 composition은 최소 속도가 없고 총 인력0은 가중평균이 정의되지 않는다. 파서는 이를 보존하고 집계가 `UndefinedArithmetic`을 반환한다. 이는 새로운 편집기 최소 대대/양수 인력 규칙이 아니다. overflow·잘못된 참조·바인딩 오류도 입력/기존 상태를 바꾸지 않는다.

## 권위·수명·identity·호환

외부 sidecar는 caller의 명시 경로에서만 읽는다. pack/scenario 등록·기본 편제 선택·WorldDefs·Simulation state·phase·명령·save·wire·server query·client에 넣지 않는다. parsed Definitions는 외부에서 수정하거나 deserialize할 수 없다. 각 검사 호출은 실제 기존 pack 검증, 실제 national loader, source identity 재검사를 사용한다.

정규화된 component/template maps, sorted composition(반복 보존), explicit policy version을 기존 canonical state_hash 알고리즘에 전달해64bit **진단용 identity**를 만든다. 같은 값의 decimal 표현 및 배열 순서 변화는 같은 identity를 만든다. 이 값은 Simulation state_hash·pack content hash·save presence·암호학적 인증이 아니다. 기존 schema/TS·저장 버전과 팩·golden에는 변경이 없다. 별도 JSON Schema를 생성했다고 주장하지 않는다.

CLI 출력 scope는 `normal_template_stats`, 능력치 출처는 `component_declared_resolved_normal_inputs`, game_editor_legality는 `not_evaluated`다. 실제 생산 바인딩 metadata와 whole equipment requirements, raw Qty/Fx 및 인력 가중 나머지를 출력한다. 모델은 기존 family/name_key/generation만 제공하며 여기서 전투 능력치를 얻었다고 표기하지 않는다.

## 독립 경계와 후속

정상 합·불균등 가중평균·최저 속도·12/4·반복·0 인력·빈/0 총 인력·정밀도와 wide intermediate/overflow, 실제 production Some/None·모델/종류·국가 허용 목록, 두 fresh process CLI output을 대조한다. 고정 M1/M2·저장/생성 schema/프로토콜 bytes를 보존한다. 독립 QA와 main CI까지 통합 게이트를 별도로 판정한다. 이 로컬 구현은 전체 WP-16 또는 M2 완료가 아니다.
