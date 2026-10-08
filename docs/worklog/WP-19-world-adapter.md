# WP-19 작업 로그 — 등록과 실제 World/Map graph adapter

| 항목 | 값 |
|---|---|
| 상태 | 구현 후보; 새 독립 검증·통합 대기 |
| 담당 | 구현 세션 P03/P06 |
| 브랜치 | wp/19-world-adapter-cloud |
| 기준 | accepted main 6d1d1d9278c7771be9b9ba2943e745a9094b9104 |
| 대상 REQ | REQ-SUP-01/02/04 graph-input 부분; 기존 REQ-MIL-04 cost boundary |

## 계획과 변경

기존 supply 모듈을 등록하고 real World/Map pure graph-input adapter를 구현한다. Option controller/ID0, 방향 비용과 공유 capacity, actual terrain/river/current infrastructure, exact required coefficients/static refs를 검증한다. shared lib.rs는 registration1줄뿐이며 World/Simulation state·data/schema/save/proto/server/client·pack/fixture/golden/lock/asset 변경은 없다. 루트가 후속 통합/CI를 담당한다.

## 테스트 우선 기록

실제 공개 API를 부르는 `actual_world_capital_controller_and_refs`, `actual_map_directional_terrain_river_and_infrastructure`를 먼저 작성하고 adapter가 MissingContext stub인 상태에서 컴파일 후2 FAIL/1 negative control PASS(exit101)를 기록했다. 이후 실제 구현으로4개 검사가 PASS였다. 등록된 이전22개 검사를 컴파일할 때 Input Deref의 test-helper borrow에 E0502가 발생했고 duplicate clone을 별도 local로 둔 뒤 기대 assertion을 그대로 보존했다.

## 실행한 검사

Rust1.99.0, existing offline cache, explicit existing target, jobs2/incremental0/debug0. 매 Cargo 전에 workspace/tmp 각각1GiB 여유를 확인했으며 새 full target/node_modules를 만들지 않았다.

| 명령 | exit | 실제 결과 |
|---|---|---|
| cargo test -p oh_sim --locked --offline --test supply_world_adapter (stub) | 101 | 2 FAIL/1 PASS |
| 같은 명령 (첫 구현) | 0 | 4 PASS |
| cargo test -p oh_sim --locked --offline --lib supply (초기 등록) | 101 | test helper E0502, runtime 요구 실패와 구분 |
| --lib supply --test supply_world_adapter (필터) | 0 | supply24 PASS / 이름 필터된 integration1 PASS; 전체 integration으로 주장하지 않음 |
| cargo test -p oh_sim --locked --offline --test supply_world_adapter (전체 controls) | 0 | 11 PASS |
| cargo test -p oh_sim --locked --offline (최종 metadata 포함) | 0 | affected oh_sim 전체136 PASS(3 doc-tests 포함); integration12/등록 supply24 포함 |

| cargo clippy -p oh_sim --all-targets --locked --offline -- -D warnings | 0 | PASS |
| cargo fmt --all --check | 0 | PASS |
| python3 tools/check_docs.py / git diff --check | 0/0 | 오류·경고0 / whitespace 오류 없음 |
| 기존 실제 integration 바이너리의 metadata test 새 process2회 | 0/0 | 각1 PASS,8 actual network variants/104 raw rows 동일 |
| frozen packs/goldens/repro/assets/Cargo/npm locks 기준 HEAD byte 비교 | 0 | 113파일 동일 |

새 integration 검사12개는 actual capital/current controllers/valid NationId0/water None, 비대칭 목적지 terrain/river/state factor, 소유와 다른 통제국 cut, expires modifier로 변한 실제 infrastructure와 stale exact mapping 오류, missing/invalid speed/factor/decay/terrain/ref/duplicate/level, coastal port/hub unsupported authority, 비용 overflow/rounded zero/World canonical 불변, excluded edge 및 fresh-process network 입력을 다룬다. Pure graph 추가2개는 land flag를 유지한 None 차단과 정상 ID0/우회 route, forward/reverse path 비용과 역방향 비용<=0 거부를 다룬다. 기존22개 보존·단조성/손실/공유 용량 assertion은 약화하지 않았다.

## 증거와 ADR

- 실제 source/API/tests: `crates/oh_sim/src/supply/`, `crates/oh_sim/tests/supply_world_adapter.rs`; 정규 Cargo 재현은 모듈 README.
- [ADR-1902](../adr/ADR-1902-world-network-adapter.md): 단위·방향·권위·실패·미구현 boundary.
- 두 새 process의 NETWORK/NODE/EDGE/RAIL/FEEDER raw104행 SHA256은 `a1ce2a47eb9b39c352b2b14afbc0bc587c8c216264944805d6d2d5121ea07034`로 같았다. Full Simulation/save/hash 새로운 gate로 확대하지 않는다.
- 기존113개 frozen 입력 manifest SHA256 `d47d0cbc70c77b9f586f9b7f1f681678f0e24eaca6f714b20528ec097e902954`, 모두 기준 HEAD bytes 동일.
- 최초 실패·필터 실행·수정과 최종 명령은 구분한다. 내부 command logs/raw evidence는 Git에 넣지 않는다.

## 결정 필요 / 범위 밖 / 못 한 부분

실제 World는 land controllerNone을 생산하지 않으며 restore도 이를 거부한다. Pure graph None 검사를 actual landNone World 검증으로 확대하지 않는다. Global infrastructure curve/table 적용범위, actual WP16 division demand/strength/org/attack/lifetime, current/subsequent leg 효과, hub/port instances/rail placement, daily state·전체 phase rollback·V7/canonical/wire/query/served UI/차량은 이번 범위가 아니다. 어댑터는 명시 Network만 반환하며 Division demand를 만들거나 calculate(empty demands)로 실제 phase를 가장하지 않는다. 전체 WP19/M2 완료·runtime query/penalty 적용을 주장하지 않는다.

## 독립 검증 / 통합

이 문서 작성 시 새 독립 검증·P07/PR/같은 main CI는 대기 중이다. 구현 세션은 commit만 하며 push/merge·다른 세션 생성은 하지 않는다.
