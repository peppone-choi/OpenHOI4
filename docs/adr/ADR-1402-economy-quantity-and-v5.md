# ADR-1402 경제 수량 권위·실적 원장과 additive v5

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-14, REQ-ECO-01/02/04/06/07, REQ-NAT-02/03/04 |

## 맥락

REQUEST-0010 A-CEO-r1 수정판의 producer를 기존 결정론·private 권위·None/v1~v4 계약에 연결해야 한다. 이는 사용자 게임 규칙 확정이 아니다. 소비자 공식·군사 효과·시장·항복은 후속 WP 소유다.

## 결정

불변 경제 입력을 scenario.economy가 가리키는 strict TOML로 두고 기존 map/nation/building/law/locale/AST 참조와 초기 파생값을 loader에서 검증한다. 누락·null·overflow·부정확 비율·음수·법령 cost0을 거부한다. 새 양은 Qty I48F16, 비율은 Fx I32F32, 인력은 i64다. Qty×Fx는 wide i128 내림과 checked 변환으로 처리한다. 소비재 min은 Fx checked_mul 후 checked_add와 clamp 순서이며 큰 slope의 중간 overflow를 clamp로 숨기지 않는다.

Simulation의 optional private Economy와 clone transaction으로 명령/효과/일일 phase를 처리한다. 실제 날짜 경계에서 Economy→Construction→Manpower→Politics를 적용하고 초기화·Pause·query·restore는 PC를 지급하지 않는다. 성공 step의 의미상 command 오류는 예정 명령을 소비하지만 phase 오류는 world·clock·권위·queue 전체를 rollback한다. 소유주 기준 IC/인구/자원 흐름, 정확 네 비율/residual, 한 단계 건설·예약/휴면·raw IC 올림·초과폐기·당일 미사용 전달, 인력 capacity/committed/reserved를 저장한다. 내부 manpower API는 consumer용이며 player 명령이 아니다.

일일 quantity/construction ledger에 계산 tick과 당시 입력·보정·적용량·반올림/폐기·skip 사유를 함께 저장한다. 현재 법령/비율과 과거 적용 snapshot을 별도로 읽고 restore 시 원장 산술을 재검증한다. 산업 score는 실제 IC Qty×weight Fx와 입력/term/tick만 노출하며 미지원 축은 없게 둔다.

새 저장은 additive format5이며 기존 base/movement/strait/trigger와 전체 typed pending queue·economic identity/state/ledger를 보존한다. 기존 command/DTO enum 뒤에 새 variant를 추가하고 경제 None이면 이전 canonical 경로를 유지한다. v1~v4는 경제 presence를 담을 수 없고 v5만 경제 context를 복원한다. force도 mode/context/정의·권위·원장 검사를 우회하지 않는다. 명시 빈 정의는 별도 presence를 보존하되 경제 국가/점수/정치 capability를 생성하지 않는다.

Host 선택국에 묶인 명령과 실제 readonly Query::Economy를 제공한다. wire는 exact decimal/raw bits/fractional width, full pending queue와 actual hash를 전달한다. client는 인코딩·shape guard만 담당한다. 기존 current-save artifact 아래 economy-v5 native capture/normal·force server proof를 추가하며 원 v1/M1/3OS gates/workflow는 유지한다. 독립 Python canonical/wire 기준과 실제 native 프로세스 원출력을 함께 보존한다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 수량을 Fx로 저장 | 기존 ratio 타입 재사용 | Qty 범위를 조용히 좁힘 |
| v4에 필드 삽입 | version 수 감소 | 원 bytes/canonical 호환 파괴 |
| 부재/빈 입력을 기본0 국가로 변환 | 조회 단순 | 실제 producer/presence를 왜곡 |
| 과거 ledger를 현재 법령으로 검증 | 작은 DTO | 적용 당시 산술·provenance 손실 |

## 결과와 영향

새 의존성·외부 버전·원 에셋·게임 수치를 추가하지 않았다. 수치는 명시 경제 정의이며 화면 문자열은 ko/en 현지화 키다. 원 None/v1~v4·원 테스트/골든·CI는 보존한다. 실제 적용·경계·codec·host/wire/repro·거부 증거는 WP-14 작업 로그를 따른다. 산업 producer를 포함해도 전체 REQ와 신규 경제 활성 부하 성능·후속 소비자·독립 P05/3OS/main CI가 완료된 것은 아니다.
