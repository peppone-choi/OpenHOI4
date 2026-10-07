# ADR-1702 typed Strait context와 추가 v3 저장

| 항목 | 값 |
|---|---|
| 상태 | 기술 채택; REQUEST-0007 A CEO 한정 게임행동 잠정 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-17, REQ-MIL-04, REQUEST-0007 |

## 맥락
원97f의 v2 DTO에는 네 raw 시간계수와 일반/하천 route만 있다. river 필드의 의미를 strait로 재해석하거나 새 field를 v2 Postcard 끝에 붙이면 종류 구분/기존 byte 읽기와 canonical이 바뀐다. 부모93e3dc1과 원SHA2731e99b719f836d42d2100da349392d199cbebba301796cbbeed098104b2e54/독립8산술을 읽어 CEO A 한정 잠정을 인수했다.

## 결정
원 Factors와 UnitInput/v1/v2 DTO는 그대로 둔다. 별도 typed StraitContext/StraitFactors 및 trusted Movement::with_straits가 양끝점 allowed/land·실제 MapDataStrait·direction/kind를 검증한다. 종류는 저장·canonical에서도 명시한다. normal/river 공식은 그대로, strait는 distance÷speed→terrain→infrastructure→supply→strait (river 곱 없음), 매 단계 checked floor 및 최종양수.

Movement.straits=None은 skip 직렬화로 원97f 바이트/hash/정책을 유지한다. 명시 Some context는 모든 unit에 지정하며 empty direction map도 implicit factor가 아니다. 새 authority는 v3 frozen v2 base+ordered kind/raw context reader/writer로 보존한다. v1/v2 지원/fixture/producer·원 canonical을 유지하고 자동 migration은 없다. context 있는 v2 export는 손실 방지 오류, context 없는 legacy export/encode는 기존 v1/v2다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| v2 factors[3] river를 strait로 해석 | field 추가 없음 | 적용슬롯의 의미 혼동과 explicit kind 부재 |
| v2 끝 field 추가 | 코드 짧음 | Postcard 구버전 표현/decoder 파괴 |
| missing factor1 | 진입 쉬움 | 채택조건 위반 |

## 결과와 영향
v3는 기술적으로 추가되는 가역적 저장 버전이며 기존 저장거부/migration이 아니다. 원97f는 커밋/증거를 그대로 보존하고 v2 native fixture를 동일 byte로 회귀검사한다. 신규 vectors는 기존 policy로 bounded preflight하며 unknown version은 거부한다. UI/wire/host 실제unit 생성은 후속WP16/22이다. 해군/봉쇄/통행권/진행중 context 갱신 규칙과 D/OPEN 추가확정은 구현하지 않는다. 새 dependency/외부버전 변경은 없다.
