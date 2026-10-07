# ADR-1301 typed trigger registry와 additive 종료 저장

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-13, REQ-AGD-05, REQ-TIME-04 |

## 맥락

REQUEST-0009 A의 CEO 한정 잠정 게임행동을 실제 WP24 registered loader와 WP17 movement/strait/save producer에 연결한다. 기존 Scenario/World에는 종료/flags가 없고 기존 v1/v2/v3 DTO는 frozen이다. 실제 producer/필드/독립 예제 대조는 [WP-13-schema](../plans/WP-13-schema.md)에 있다.

## 결정

AST는 single-key typed condition/effect enums, strict compound args, 명시 roots/scopes로 검증한다. registry 문법과 Simulation capabilities를 분리하여 미지원 실제사용을 로드에서 거부한다. nation flags/typed 종료 원인만 actual mutable adapter로 실행하고 이미 존재하는 World 조회를 연결한다. program ID는 trusted command adapter의 안정 참조이며 client 명령으로 노출하지 않는다. 전체 effects는 clone transaction, 한 call의 실제 primitive 실행수1000/깊이16 경계를 지킨다. 날짜/조건/명시원인은 enum/ID lexical로 병합한다.

불변 정의는 World::Defs에 보관하며 기존 World definitions_hash를 변경하지 않는다. 신규 optional Simulation.trigger에 정의 canonical 신원/flags/checkpoint를 둔다. None은 기존 canonical skip, Some(empty)도 신규 identity를 포함한다. v4는 frozen v3 substrate와 두 presence bool/full 신규queue/trigger DTO이며 기존 formats 타입/reader/writer/hash/policy/force/fixtures는 유지한다. v4 bounded preflight는 기존 cap/charge/budget을 재사용한다. legacy 결과를 신규 성공으로 대체하지 않는다.

오류 우선·scope 복귀·host正常종료·초기1회/step뒤 평가·signed exact Fx 가중합 기술연산과 미지원 score axis/항복관전 후속 추적을 schema에 명시했다. 게임상 새 normalization/생존/진영 규칙은 만들지 않는다. 신규 dependency 없음; 기존 pinned crates API와 local producer를 사용한다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| v1/v2/v3에 field/variant 추가 | 짧은 구현 | 원 binary bytes/order 파괴 |
| unsupported 값을0/false/no-op | 광범위 성공처럼 보임 | 실제 producer/capability 계약 위반 |
| AST를 save에 반복 저장 | 독립 입력 | pack 검증과 정의 단일 기준 중복/재귀 allocation 위험 |

## 결과와 영향

정의/초기/가변/파생 상태를 구분하며 새로운 값을 포함한 replay/checkpoint는 v4를 사용한다. 별도 trigger actual3OS workflow/checker는 원CI와 별도로 필수 compare한다. actual3OS 실행/독립 P05/mainCI는 부모 소유이며 로컬 synthetic으로 대신하지 않는다. UI/네 축 실제 score/후속 host adapters는 표로 추적하고 이번 부분지원으로 M2 전체 완료를 선언하지 않는다.

## P-06-1 보완

P-05-1에서 신규 wire guard의 중복·순서·ID·u64 상한·unknown member 누락과 public comparer의 stale source 승인 문제가 확인됐다. 신규 trigger wire에만 exact 필드·타입·canonical 정렬·유일성 검사를 추가한다. 기존 연결의 invalid-server-message 처리 전에 공개 콜백이 호출되지 않아야 한다. 서버 권위 판정이나 게임 규칙을 추가하지 않는다.

public comparer의 기준은 read-only로 관측한 현재 clean checkout HEAD다. artifact source-before/after/result/manifest가 이 HEAD에 묶이고 성공 출력 전 source/index가 그대로여야 한다. 내부 순수 비교 helper를 둬서 predicate 테스트를 실제 소스 권한과 분리하며 public CLI는 override 없이 현재 소스를 검사한다. 임시 Git·실제 subprocess 회귀의 합성 OS label을 actual3OS 성공으로 기록하지 않는다. 기존 checker의 canonical/native/save/source 검사와 보호된 workflow·v1/v2/v3/protocol/save 정책은 유지한다.

## P-06-2 경로 충돌 수정

최초 통합 actual CI a652에서 download-artifact의 `trigger-evidence/`가 untracked로 잡혀 public clean-source 검사가 native1을 반환했다. 정상 증거를 source 밖의 기존 ignored `target/trigger-evidence`에 저장하고 compare의 root도 같이 옮긴다. dirty 검사 완화나 새 ignore 추가는 필요 없다. workflow의 이 두 경로와 실제 public CLI/Git 경로 회귀만 바꾸며 원 3OS producer·검사·CI FAIL 기록을 유지한다. 새 독립 P-05/P-07/main CI 판정은 부모가 한다.
