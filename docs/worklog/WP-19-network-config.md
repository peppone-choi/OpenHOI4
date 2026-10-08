# WP-19 작업 로그 — 외부 네트워크 설정·초기 World 진단

| 항목 | 값 |
|---|---|
| 상태 | 검증 대기 — 이 단위 독립 QA·PR·같은 main CI 후속 |
| 담당 | 구현 세션 P-03/P-06 |
| 브랜치 | wp/19-network-config-cloud |
| 기준 | 인수된 main 6191c0ab5e1cfa835da4e9de76e2aafe6863d042 |
| 대상 REQ | REQ-GEN-02 strict 입력; REQ-SUP-01/02/04 네트워크 입력·진단 부분 |
| 선행 WP | WP-19 standalone/World adapter, WP-15 생산·비축 producer |

## 계획과 변경

외부 strict TOML loader, real World config conversion, native initial-World inspect CLI를 구현했다. 단일 구현자가 oh_data/oh_sim/oh_cli 최소 모듈 등록·CLI dispatch를 담당했다. immutable Config와 bounded read, positive Fx/Qty·중복·참조·정확한 현재 infra 표를 검증한다. accepted graph/allocation/world_adapter 알고리즘은 바꾸지 않았다. CLI는 실제 초기 loader/modifiers를 사용하며 Simulation을 생성·진행하지 않는다.

설정은 등록된 pack/scenario/WorldDefs나 system presence가 아니다. explicit --config가 필수이고 --load/step/query/save는 지원하지 않는다. config identity는 정규 진단 provenance다. 원본 pack/fixtures/goldens/assets/locks·schema/save/proto/server/client/dependencies 변경은 없다. HANDOFF/M2는 루트가 별도로 수정·관리하며 이 제품 commit에 포함하지 않는다.

## 테스트 우선 기록과 실패 보존

| 요구 경계 / 시도 | 구현 전·중간 실제 결과 | 최종 결과 |
|---|---|---|
| 데이터 explicit refs / normalized identity / strict negatives | 컴파일 가능한 loader stub에서3 FAIL(exit101) | 확장 data11 PASS |
| 실제 native inspect dispatch | 처음 결합 실행에서 CLI1 FAIL(exit101), command가 없어서 run usage 반환; 그 실행의 sim 검사를 수행한 것으로 주장하지 않음 | native CLI8 PASS |
| real parsed Config → World adapter | 별도 실제 Config를 읽은 adapter stub1 FAIL(MissingContext, exit101) | adapter7 PASS |
| data 참조 오류 control | 확장 data8 PASS/1 FAIL(exit101); 오류로 고른10→30이 실제 유효 edge였음. 존재하지 않는999로 교정했고10→30은 후속 정렬 positive control에서도 유지 | data refs·positive controls PASS |
| 첫 결합 구현 | LoadedScenario.pack을 content_hash 보유형으로 잘못 가정하여 E0609(exit101). optional pack-hash 출력 제거; 권위/API 필드 추가하지 않음 | 실제 loader/CLI PASS |
| compile fix 뒤 결합 실행 | CLI1/data9 PASS, adapter1 MissingContext FAIL(exit101). 합성 Defines에 실제 지도 hills 계수가 빠졌음; 테스트가 명시적 hills 계수를 제공하도록 교정 | 확장22 PASS |
| 빈/격리 그래프의 actual infra context | state3·edges없음에서 누락을 허용해 Ok(Network)를 반환한 실제 FAIL(exit101) | 모든 실제 현재 주 level의 정확한 factor 확인 후 PASS |

실패 출력은 private evidence에 보존했고 Git에 raw 실행 기록을 넣지 않았다. 이후 metadata 수량/정렬·native NationId0/captured-capital controls를 추가하여 최종 새 검사는 data11+adapter7+CLI8=26개다. 이전22개 algorithm assertion과 기존 World adapter 검사도 보존했다.

## 실행한 검사

Rust1.99.0·기존 offline cache·단일 기존 target, source environment 후 explicit target/jobs2/incremental0/debug0를 사용했다. workspace/tmp 각각1GiB guard를 유지했다. 새 full target/node_modules/frontend/browser 산출물을 만들지 않았다.

| 명령 / 검사 | exit | 실제 결과 |
|---|---|---|
| cargo test -p oh_data --test supply_network --locked --offline (stub) | 101 | 3 FAIL |
| cargo test -p oh_sim --test supply_config_adapter -p oh_cli --test supply_network --locked --offline (초기) | 101 | CLI1 FAIL; 이후 target 미실행 |
| cargo test -p oh_sim --test supply_config_adapter --locked --offline (실제 parsed Config/stub) | 101 | 1 FAIL |
| 확장 data / 첫 결합 / compile-fix 결합 시도 | 101 각각 | 위 참조 control·E0609·MissingContext를 분리하여 보존 |
| 새 세 suite 결합 (최초 전체 controls) | 0 | 22 PASS |
| adapter isolated context 이름 필터 | 101 | 1 FAIL, 다른6 filtered; 전체검사로 확대하지 않음 |
| cargo test -p oh_data -p oh_sim -p oh_cli --locked --offline (context 수정 뒤) | 0 | 238 PASS(3 doc-tests 포함), 추가3 controls 이전 |
| cargo test -p oh_data --test supply_network -p oh_sim --test supply_config_adapter -p oh_cli --test supply_network --locked --offline -- --nocapture (최종) | 0 | 26 PASS |
| cargo clippy -p oh_data -p oh_sim -p oh_cli --all-targets --locked --offline -- -D warnings | 0 | PASS |
| cargo test -p oh_data -p oh_sim -p oh_cli --locked --offline (최종 신규 controls 포함) | 0 | 241 PASS(3 doc-tests 포함); 전체 workspace 판정은 아님 |
| 별도 native CLI 프로세스2회 | 0/0 | stdout3,396 bytes 동일, 아래 SHA |
| cargo fmt --all --check / python3 tools/check_docs.py / git diff --check | 0/0/0 | fmt PASS / 문서 오류·경고0 / whitespace 오류 없음 |
| 기준 snapshot 대비 frozen113 파일 bytes 검사 | 0 | 모두 동일 |

최종 fmt/docs/diff·113입력 byte 보존도 확인했다. 전체 workspace/server/browser/save integration gate나 새로운 Simulation 결정론 검사로 주장하지 않는다. 기존 영향 crate의 save/native 회귀가 통과했다는 사실과 이 단위가 새 save semantics를 검증했다는 주장은 구분한다.

## 증거와 ADR

- 제품: `oh_data::supply_network`, `oh_sim::supply::config_adapter`, `oh_cli::supply_network`; 각 전용 integration tests와 standalone 합성 TOML fixture.
- [ADR-1903](../adr/ADR-1903-external-network-diagnostics.md), module README: format/units/bounds/authority/absence·explicit empty/identity/미구현 계약.
- 두 native 초기 네트워크 stdout SHA256: `d46d745bb674c40ee7fa39d28a793602c4c2846648cd6077a20ed17115c1bde4`. 의미는 초기 정적 네트워크 진단만이며 full Sim/save hash가 아니다.
- Frozen pack/golden/repro/assets/Cargo/npm lock113파일은 기준572개 SHA snapshot과 동일했다. 이번 manifest SHA256 `c716240860a02e6e20fe334c903fce1e6147bd9020dfd634111f0facfe201991`은 evidence 형식의 hash이지 새 pack identity가 아니다.
- private logs/receipt/normalized initial output은 루트에게 별도로 전달한다. 독립 QA와 같은 main CI PASS를 구현자가 선언하지 않는다.

## 결정 필요 / 범위 밖 발견 / 못 한 부분

actual Division demand·strength/org/attack·수명·current-leg effect consumer가 없다. 전 세계 infrastructure curve/table policy, hub/port instance authority, 동적 rail lifecycle/construction, 차량 C/k 및 supply state/save presence는 후속이다. source ID는 기존 per-nation Network 이름 공간으로 한정한다. controllerNone은 실제 water에서만 존재하며 landNone producer를 제조하거나 World restore 규칙을 바꾸지 않는다. 유효 coastal hub/port도 UnsupportedBuildingInstance다. 명시적 빈 source/rail은 network metadata이지 daily ratio0/1이나 phase producer가 아니다. static rail·exact coeffs를 caller input으로 명시하고 server 권위 query로 가장하지 않는다. 전체 WP-19/REQ-SUP-02/M2 완료 판정은 못 하며 이 범위에 포함하지 않는다.

## 원작 대비 차별화 초안

기존 WP-19 fixed-route/shared-capacity/checked-ledger 계약을 유지하고 독립 합성 입력으로 strict diagnostic provenance를 제공한다. 원작 자료·수치·경계·클라이언트는 참조하지 않았다.
