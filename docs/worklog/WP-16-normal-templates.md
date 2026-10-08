# WP-16 작업 로그 — 외부 정상 편제 정의·집계

| 항목 | 값 |
|---|---|
| 상태 | 로컬 구현·검증 대기, 미게시 |
| 담당 | 독립 구현 세션 |
| 브랜치 | wp/16-templates-local |
| 대상 REQ | REQ-MIL-02 정상 구성요소/편제 정의·자동 집계 부분; REQ-MIL-03 생성 전 요구량 설명만 |
| 선행 WP | 인수된 WP-15 생산 모델, 기준 main 5ca938e5c49d37736fbb778df53a820d6321804c |

## 계획

사용자가 승인한 명시 family/model 바인딩, 필드별 합산·인력 가중평균, 최저 구성요소 속도를 로컬에서 구현한다. 범위·수치 단위·참조·권위·실패/수명·schema/save/wire 경계를 [ADR-1601](../adr/ADR-1601-resolved-normal-templates.md)에 먼저 명시했다. 정상 능력치는 구성요소가 작성한 resolved 입력이며 production Model에 없는 장비 전투식을 유도하지 않는다. strict 외부 TOML, 순수 집계, 실제 pack/national production 검증을 쓰는 CLI만 추가한다.

## 테스트 우선 기록

| REQ 부분 | 테스트 | 구현 전 실제 결과 | 구현 후 실제 결과 |
|---|---|---|---|
| REQ-MIL-02 strict 정상 정의 | strict_definitions_have_valid_control | runtime FAIL, exit101: valid input이 NotImplemented; missing 입력 거부 control PASS | PASS |
| REQ-MIL-02 자동 합/가중/최저속도·요구량 | normal_sum_weighted_min_and_counts | 실제 파싱 성공 뒤 aggregate NotImplemented runtime FAIL, exit101 | PASS |
| REQ-MIL-02 실제 CLI 경계 | native_command_exposes_specific_grammar | 실행 가능한 native binary가 새 grammar 미지원 runtime FAIL, exit101 | PASS |

이후 추가한 범위/precision/참조/native valid·negative controls는 처음의 red 세 검사와 구분한다. 모든 후속 검사를 각각 구현 전 red로 실행했다고 주장하지 않는다.

첫 green 시도는 기존 CLI load_national의 반환이 production을 보유하지 않은 LoadedScenario여서 compile exit101이었다. 새 명령은 기존 pack 검증과 source identity 확인 함수를 그대로 쓰고 실제 national::load_scenario를 직접 읽도록 고쳤다. 두 번째 green 시도는 native test oracle의 i32 literal 합 overflow compile101이며 i64 oracle로 고쳤다. 세 번째 green은 missing-production fixture에 unregistered production 파일이 남아 pack 검증이 먼저 거부하는 실제 FAIL101이었다. 해당 테스트가 소유한 임시 파일만 제거해 유효 production-None pack control을 만들었다. 원본 팩과 기존 검사는 변경하지 않았다. 네 번째 direct 실행은13/13 PASS다. 이 시도들은 독립 QA finding이나 새 전체 리뷰가 아니다.

## 실행한 검증

모든 Cargo 명령은 기존 고정 toolchain·locked/offline·공유 외부 target 설정으로 실행했다. 새 dependency/lock, npm/browser/tool 다운로드는 없다.

| 명령·대상 | 종료 코드 | 실제 결과 |
|---|---|---|
| cargo test --offline --locked -p oh_data -p oh_sim -p oh_cli --test military_templates | 0 (최종) | data5 + sim4 + native CLI4 =13 PASS |
| cargo test --offline --locked --workspace | 0 |345 PASS,63 test-result suites,0 failed/ignored |
| cargo fmt --check | 0 | formatting PASS |
| cargo clippy --offline --locked --workspace --all-targets -- -D warnings | 0 | PASS; 기존 ts-rs deny_unknown_fields attribute notices 출력은 보존 |
| python3 tools/check_docs.py | 0 | 오류0·경고0 |
| python3 tools/check_assets.py --release | 0 | PASS |
| python3 tools/check_architecture.py | 0 | PASS |
| cargo run --offline --locked -p oh_data --example schema | 0 | 기존10 schema 생성 bytes 동일 |
| cargo run --offline --locked -p oh_proto --example generate | 0 | 기존 TS bytes 동일; 정적 legacy schema까지12경로 보존 |
| oh_cli validate --deny-warnings data/packs/testland | 0 |3bde1bed90d3734e |
| oh_cli validate --deny-warnings data/packs/testland_m2 | 0 |d656804578786a61 |
| oh_cli run --pack data/packs/testland --scenario m1 --days 365 --seed 1 --hash-out | 0,0 | 두 fresh process 모두 b595dc2a1e5b4f8c |
| 동일 M1 180일 save-out 후 fresh process resume --days 185 | 0,0 | split b7eca53b4860eadd →365일 b595dc2a1e5b4f8c |

새 직접 테스트는11 필수 named stats 누락, decimal/type/unknown-field/domain/size, raw duplicate definition/equipment/binding, role/ref/authored binding 문맥,12/4와 반복 보존, Qty sum·인력 가중 remainder·Fx min, 인력0 지원·empty/zero-weight UndefinedArithmetic, wide i128 및 i64/Qty overflow를 검사한다. Native CLI는 실제 임시 production pack의 국가/모델/종류/허용성 및 production-None controls, forbidden stepping/load flags, 두 fresh process 동일 output·동등 decimal identity를 검사한다.

workspace 서버 빌드는 기존 인수된 ignored client/dist를 hardlink로 재사용했다. 먼저81개 tracked client 입력이 byte-identical임을 확인했다. 새 client 빌드나 browser 실행이라고 기록하지 않는다. 변경한 추적 기존 제품 경로는 data/sim/CLI module 등록4경로뿐이며 독립 root 문서3경로는 구현 commit에서 제외한다.

## 증거

- E1: 새 strict definitions/parser와 synthetic normal.toml, 실제 production reference 검증.
- E2: pure aggregate와 native command의 red→green 기록,345개 Rust 검사·strict팩·기존 save/resume control.
- E3/E4/E5: 이번 범위에 UI/프로토콜/실제 게임 화면 변경 없음. 새 독립 QA·기본 브랜치 CI 증거는 후속이며 이 구현 로그가 대신하지 않는다.

## ADR

[ADR-1601](../adr/ADR-1601-resolved-normal-templates.md), [사용 안내](../../crates/oh_sim/src/military_templates/README.md).

## 원작 대비 차별화 초안 (01 §11)

- 원작 데이터·코드·수치 비교 없이, 명시 모델 문맥을 가진 정상 구성요소 입력을 whole counts/Qty/Fx로 집계하고 원본 중복 및 exact production references를 진단한다. 격자 편제 대신12/4 목록을 보존한다. 편집기/실제 사단 legality·runtime는 후속이다.

## 결정 필요

| 식별자 | 내용 | 막는 범위 |
|---|---|---|
| WP-16 후속 | 모델의 장비 전투 능력치를 구성요소 정상 능력치로 해석하는 단위/혼합 정책 | per-item 또는 장비·대대 전투 도출 |
| WP-16 후속 | 훈련 예약·환불·보충·군/장군 및 편집기 legality | 실제 사단 생성·장비/인력 소비·배치 |
| WP-19 후속 | 실제 사단 수요·회복/화력/이동 효과·손실·수명 | daily Supply consumer |

## 범위 밖 발견

production Model에는 전투 능력치가 없다. 현재 M1/M2는 production-positive 콘텐츠가 아니므로 새 테스트가 소유한 M1 복사본에 기존 synthetic production을 실제 scenario로 선택한다. 원본 팩을 채우거나 fake 레지스트리를 넣지 않았다.

## 못 한 부분과 이유

위 미승인 후속·save/query/wire/UI·전체 WP-16/M2 인수는 구현하지 않았다. 이번 지시는 로컬 구현·검증만이므로 push/PR/main merge를 하지 않는다. 별도 독립 QA 한 번을 기다리며 이 로그는 인수 PASS 또는 통합 완료 선언이 아니다.

## 수정 기록 (P-06)

- 독립 QA finding 수신 전, 해당 없음.

## 통합 기록 (P-07)

- 로컬 후보 commit으로 동결; 게시/병합 보류. 독립 QA와 별도 통합 승인·동일 main CI 게이트는 후속이다.
