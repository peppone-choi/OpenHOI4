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

## M2-r3 P-06: 현재 건물 복원 경계 보완

a933 독립 P05 F-01에서 초기 loader가 거부하는 industry level4(max3)를 v5 복원이 수락했다. 과거 ledger contribution 단계만 검사하고 현재 World의 occupied 건물/슬롯을 검사하지 않은 누락이다. 같은 v5 validate에서 모든 현재 주의 건물에 기존 slots checked 누적/참조 및 costs 단계 상한을 적용한다. 정의가 nonempty일 때만 해당 state_slots 상한과 대조한다. 슬롯 초과는 기존 SlotCap, 단계 초과는 기존 TargetConflict, 합계 overflow는 기존 Overflow로 거부한다.

현재 권위와 historical daily ledger를 같게 만드는 방식은 택하지 않았다. 당일 완성 뒤 현재 level3/이전 ledger level1이 함께 존재할 수 있다. 예약/휴면 프로젝트를 현재 occupied 슬롯에 임의 합산하지 않으며 원 ownership-loss/recovery conflict 전 과정의 full v5 DTO/canonical/hash roundtrip을 추가 검사한다. None 및 명시 빈 경제에는 없는 stage/slot 규칙을 새로 부여하지 않는다. 기존 World 공통 restore의 음수/known map ref/정렬/주 집합 검사는 유지한다. 초기 입력 검사는 변경하지 않는다.

공통 경계는 Simulation::from_save_v5이며 codec decode·encode의 후보 재검증·write_atomic readback·CLI resume·Host normal/force·repro start/replay가 같은 경로를 사용한다. movement/strait/trigger presence도 그 뒤 동일 경제 validate를 거친다. None/v1~v4 경로·format bytes·새 정의 default·의존성·벤치/CI 정책은 변경하지 않는다. 원 반례 bytes/pack과 새 dedicated tests/repro를 보존하고 새 source의 독립 P05·필수 CI는 부모가 인수한다.

## M2-r3 P-06-2: target checked 변환 및 원 Git bytes

P05-2의 MIN target은 project restore의 target-1에서 native101 panic이었다. 프로젝트·historical construction ledger·Construct 예약 합계·daily construction·같은날 예약의 다섯 위치가 같은 checked target_stage를 사용한다. checked_sub(1)→usize 변환→정의 costs/slots 범위로 검사하고 잘못된 값은 기존 InvalidValue를 반환한다. 1/최대 단계 및 유효 휴면 target를현재수준+1로 바꾸지 않는다. panic catch/force 예외/overflow 설정 변경/새 저장 형식은 없다.

첫 F01 fixture는 구현 WT 원CRLF와 Git LF blob이 달라 fresh checkout의 normal이 PackMismatch로 먼저 거부됐다. 이 결과는 stage semantic 성공이 아니다. CEO 원review를 P06-2 inputs/CEO-review.md에 인수했다. .gitattributes의 원103B prefix를 byte-exact 유지하고 아래 literal 두 경로만 -text로 추가한다: tests/repro/WP-14-M2-r3-restore-stage/packs/testland/common/economy/independent.toml 및 tests/repro/WP-14-M2-r3-restore-stage/packs/testland/scenarios/m1/scenario.toml. binary pack ZIP도 고려했으나 기존 Rust/Host fixture 경로를 보존하는 허용된 최소 조치를 택했다. 기존 global patterns/config/다른파일/CI/production/frozen/golden은 변경하지 않는다. 원458B/header/expectedpackhash를 유지하고 Git blob·별도 ignored Git archive 전체22파일 길이/SHA를 원manifest와 대조한다.

MIN 팩5617B는 F01 팩5681B와 다르므로 별도 원 manifest/context를 보존한다. save·Host·native tests는 header/실제 packidentity와 유효control을 확인하고 정확 semantic 원인을 요구한다. 원 P05-2의 FAIL 및 raw-index INVALID를 구분하고 원before/index/mtime를 복원하거나 소급 유효 판정하지 않는다. readonly TS 비교는 기존 public typescript()를 자기 ignored actual-source/lock/linkage probe에서 stdout으로 실행하며 tracked-writing generator를 사용하지 않는다. 새 source의 독립/remote 인수는 부모 책임이다.
