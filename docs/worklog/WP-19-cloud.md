# WP-19 작업 로그 — 고정 경로 보급 계산 독립 단위

| 항목 | 값 |
|---|---|
| 상태 | 구현 후 독립 검증 대기; runtime 통합 미착수 |
| 담당 | 구현 세션 |
| 브랜치 | wp/19-supply-cloud |
| 대상 REQ | REQ-SUP-01, REQ-SUP-02, REQ-SUP-04의 standalone graph/formula 부분 |
| 선행 WP | accepted main b112291c11822ffbf6da33b42670ff5cdb226fae; shared slot은 WP15 뒤 |

## 계획

사용자 승인된 고정 경로·유한 공유 철도 용량·거리 감쇠를 새 supply-only 파일에서 구현한다. 수정 가능한 초기 수치는 provisional로 기록한다. shared lib/World/Simulation/schema/save/proto/server/client/HANDOFF는 편집하지 않는다. 다른 세션을 만들지 않는다.

## 테스트 우선 기록

| REQ | 테스트 이름 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| REQ-SUP-01 | bottleneck_before_distance_and_cut | Invariant stub, FAIL | PASS |
| REQ-SUP-02 | finite_budget_and_actual_ratio / distance_loss_is_conserved / shared_feeder_capacity | 세 검사 FAIL | PASS |
| REQ-SUP-04 | zero_isolated_and_excluded_strait | FAIL | PASS |
| REQ-SUP-02 | invalid_inputs_do_not_mutate | FAIL | PASS |
| REQ-SUP-02 | shortage_multipliers_are_positive_time_factors | 별도 stub FAIL | PASS |
| REQ-SUP-02 | starvation_strict_threshold_grace_and_reset | 별도 stub FAIL | PASS |

초기6개는 실제 컴파일된 외부 Cargo harness에서0 PASS/6 FAIL, exit101이었다. Scalar shortage/starvation는 각각 필터 검사1 FAIL, exit101을 구현 전에 별도로 남겼다. 기존 검사·골든 수정은 없다.

## 실행한 검증

외부 temporary manifest는 실제 checkout의 `supply/mod.rs`를 lib path로, 실제 `oh_core`를 path dependency로 가리킨다. 복제된 구현·no-op runtime으로 검사하지 않는다. Rust1.99.0, 기존 offline cache, jobs2, incremental0, debug0, 별도 temporary target을 썼다. 모듈 README에 재현 manifest/명령이 있다.

| 명령 | 종료 코드 | 출력 발췌 |
|---|---|---|
| cargo test --offline --manifest-path <temporary>/Cargo.toml (초기) | 101 | 0 passed;6 failed |
| 같은 명령 (graph) | 0 | 6 passed |
| 같은 명령 (확장 graph) | 0 | 15 passed |
| 같은 명령 shortage / starvation (각 stub) | 각101 | 각1 failed |
| 같은 명령 (scalar 포함) | 0 | 20 passed;0 failed |
| 같은 명령 (물리 rail ref·large Qty 추가 최종) | 0 | 22 passed;0 failed |
| cargo clippy --offline --manifest-path <temporary>/Cargo.toml --all-targets -- -D warnings | 0 | Finished dev profile |
| rustfmt --edition 2024 --check crates/oh_sim/src/supply/mod.rs | 0 | 차이 없음 |
| python3 tools/check_docs.py | 0 | 오류0·경고0 |
| git diff --cached --check | 0 | 차이 문제 없음 |
| 별도 native probe 두 새 process / cmp | 0 | raw ledger CSV 두 출력 동일 |

추가 검사는 source additive/Qty fractional demand, 실제 ratio와 split 잔여량, unrelated component progressive filling, 고정 weights의 약⅔ stranded 사례, route tie/벡터 입력 반전, zero capacity/out-of-range/lost capital, refs/duplicates/음수/nonpositive cost, path/demand overflow, tiny attenuation의 wide infeasible trial,1,323 small quantized conservation cases,100,620 monotonic trial points, shortage counter overflow/양수time과 데이터 수치 민감도를 포함한다. 두 새 process의 실제 source/province/share raw ledger CSV SHA256은 `f5f4e1c1c50270add1045ccc9e2544852de454ac6ec6802abbaca90af58467a6`으로 같았다. 이는 standalone 계산 출력이며 전체 Simulation hash가 아니다. 전체 Rust workspace·기존 CLI world hash·save/server/browser는 이번 unregistered standalone 범위에서 실행하지 않았다.

## 증거

- 코드와 raw ledger: `crates/oh_sim/src/supply/`.
- 요구사항 tests와 tunable sensitivity fixture: 같은 폴더 `tests.rs`, `defines.toml`.
- 재현용 외부 path harness: 같은 폴더 README.
- 소스 단위 독립 검증은 아직 배정/판정 전이다. CI/main 통합 증거는 이 단위에 없다.

## ADR

[ADR-1901](../adr/ADR-1901-supply-contract.md).

## 원작 대비 차별화 초안 (01 §11에 넣을 행)

유한 additive source와 shared feeder rail budget을 raw 단위로 보존하고 fixed-route/fixed-weight 한계와 반올림 손실을 원장에 명시한다. 원작 파일·문장·지도·수치 참고/복사는 하지 않았다.

## 결정 필요 / 미인수 계약

WP16 actual division demand·strength·organization/attack ownership/unit contract, WP17 현재/다음 이동 leg 적용, strict data loader/building·coastal port placement/infra table, WP15 accepted inventory/save/schema slot을 인수해야 한다. WP44 C/k/vehicle reservation scale은 미정이며 Qty를 Fx로 좁히지 않는다. 사용자 승인된 초기 조절값을 별도 새 게임 규칙 승인으로 확대하지 않는다.

## 범위 밖 발견

현재 보급 phase trace label은 producer가 아니다. 기존 movement division에는 demand·strength/공격/조직 소비자가 없다. World aggregate buildings만으로 실제 hub/port 위치를 만들 수 없다.

## 못 한 부분과 이유

Module registration·runtime daily adapter·penalty 실제 소비·전체 step rollback·canonical/hash·additive save·wire/query/overlay는 shared serial slot 및 실제 consumer 계약 부재로 구현하지 않았다. Full WP19/REQ 인수·M2 완료는 주장하지 않는다.

## 수정 기록 (P-06)

독립 검증 지적 전이다. 구현 중 단위 검증 실패 기록은 위 테스트 우선 기록에 보존했다.

## 통합 기록 (P-07)

병합/push하지 않았다. 별도 새 독립 검증과 accepted shared integration이 필요하다.
