# 검증 리포트 — WP-15 production authority

| 항목 | 값 |
|---|---|
| 판정 | PASS — 최초 combined 후보의 유일한 FAIL을 P06 표적 확인으로 해결 |
| 검증자 | 구현 비참여 새 독립 검증 담당, 분리된 worktree, 추적 파일 무변경 |
| 최초 검증 커밋 | `95e674407aa820de14249a84bea160eba2728019` |
| P06 통합 후보 | `40f703cf331125abc4d3b78f431e6f7fc7253476` |
| 검증 일시 | 2026-10-08 |
| 범위 | REQ-ECO-03 및 REQ-MIL-01 장비 재고 부분 |

한 번의 새 전체 QA에서 확인한 한 건의 schema/parser 불일치를 별도 구현 담당이 수정했다. 수정 뒤 전체 코드 검토를 반복하지 않고 해당 실패 재현·회귀·유효 controls를 표적 확인한다. 전체 WP16·사단/보급/전투 소비·M2 완료 판정은 포함하지 않는다.

## 직접 실행한 최초 QA

Rust 1.99.0, Node 24/npm 11, 기존 lock과 offline cache를 사용했다. 모든 최종 Cargo 검사는 incremental0/jobs2/debug0이다. 기본 재현은 [작업 로그](../worklog/WP-15-cloud.md)와 공개 `production_fixture` example·`production_query.cjs`를 따른다. 별도 독립 하네스는 실제 checkout의 코드를 참조하며 제품 구현을 복사하지 않았다.

| 명령/검사 | 종료 코드 | 실제 결과 |
|---|---|---|
| `cargo test --workspace --locked` | 0 | 269개 PASS |
| `cargo fmt --all --check` | 0 | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | PASS |
| `npm --prefix client test -- --reporter=dot` | 0 | 249개 PASS |
| `npm --prefix client run build` | 0 | typecheck/Vite PASS, 기존 chunk 경고 |
| 외부 독립 하네스 `cargo test --offline --manifest-path <harness>/Cargo.toml -- --nocapture` | 0 | 독립 7개 PASS |
| `production_fixture capture <out> 8760` 두 번 + 별도 `resume <pack> <save> 8760` | 0/0/0 | 전체 보고서·canonical bytes·반복 save 일치 |
| strict 생산 입력 controls | 기대 0/1 | 유효 입력·13개 거부 PASS, 중복 allowed_models 잘못 수락 |
| `node crates/oh_server/tests/production_query.cjs <url> <capture.json>` | 0 | V6 복구·유효 SetIC·잘못된 모델/IC/ID/raw·중복 sequence·second Join·close1000 |
| 실제 Chromium 151 독립 브라우저 검사 | 0 | 권위 표시·생산 명령·지도·EN/KO |
| normal/force save 서버 | 기대 1/0/1 | PackMismatch 거부·force 동일 권위 복구·없는 production 모드 거부 |
| `check_save_current.py capture`, `check_save_determinism.py capture` | 0/0 | None/V1·economy V5·역사 V1 writer 보존 |
| `economy_target_native.py`, `economy_restore_native.py` | 0/0 | 기존 native12개/restore4개 |
| `save_native.py`, `save_historical_native.py` | 0/0 | 현재 실제 서버/query/served JS/종료, historical normal/force 거부 |
| docs/architecture/assets release/npm licenses/diff | 0 | 문서 오류·경고0, npm103개 의존성 오류0 |

## REQ·실패 경계

- IC/resource 비례 배분: 독립 Hamilton22,477개 작은 사례와 Python 임의 정밀도 정수 oracle256개 큰 입력.
- 실제 `calculate_day`1,728개: stability/unit cost/resource ceilings/reservation/fulfillment/output/carry/productive growth를 독립 정수 공식과 비교.
- unknown field/resource/model·generation/stock/cost/family/name/selection·missing define·tuning/zero cap·중복 TOML key 거부와 유효 controls.
- whole stock·same model 보존·switch carry 폐기/retention·cancel 재고 보존·monotonic ID. 취소된 미래 intent는 semantic rejection으로 소비하며 stock overflow는 clock/queue/world/economy/production/RNG/canonical state 전체 rollback.
- V6 valid round trip·truncated/limited/corrupt decode·forged tick/budget/ID/owner·duplicate queue·definitions identity/mode, 메모리 생성 TS와 production/economy/scenario schema byte 동일.
- 서버 정상 종료 뒤 같은 포트 두 곳에 실제 서버를 다시 실행해 HTTP200·exit0. served JS는 해당 checkout dist bytes와 동일.

## 최초 FAIL과 P06

`allowed_models=["test_model_1","test_model_1","test_model_2"]`가 strict CLI에서 exit0/hash `e5601c90a013b2a0`으로 수락됐다. Schema의 `uniqueItems:true`와 달리 직접 BTreeSet 역직렬화가 중복을 지웠다. 이 최초 FAIL을 PASS로 덮어쓰지 않는다. P06은 입력 중복을 먼저 거부한 뒤 같은 canonical sorted set을 만들며 schema shape·save/hash 구조·유효 Definition identity를 보존한다. 구현 담당의 실패→표적3개 data tests/clippy/fmt 통과와 독립 수정본 확인을 구분한다.

2026-10-08 17:38:26 UTC, 같은 독립 검증 담당이 분리된 수정 worktree에서 아래 표적 확인을 수행했다. 한 번의 전체 QA와 이 finding 확인을 합쳐 지정 WP-15 단위 독립 QA 게이트는 PASS다. 최초 FAIL 기록을 보존하며 새 전체 코드 검토는 하지 않았다.

| 수정본 표적 명령/검사 | 종료 코드 | 결과 |
|---|---|---|
| `cargo build -p oh_cli --locked` | 0 | 실제 수정 checkout 소스로 CLI 재빌드 |
| `cargo test -p oh_data --locked --test production` | 0 | 3개 PASS; JSON duplicates/schema uniqueItems/Definition canonical identity 포함 |
| `cargo fmt --all --check` | 0 | PASS |
| `cargo clippy -p oh_data --all-targets --locked -- -D warnings` | 0 | PASS |
| 이전 정확한 strict CLI FAIL 입력 | 1 | `duplicate production allowed model: test_model_1` |
| 인접/떨어진 중복 배열 strict CLI | 각각1 | 명시적 중복 오류로 거절 |
| 유효/역순 unique 배열 strict CLI | 각각0 | 수락; 원문 파일 순서가 다른 pack hash의 동일성은 요구하지 않음 |

수정은 parser/tests/ADR/worklog4개 파일이다. 수정 후보566개 추적 파일·HEAD·목록·SHA·diff/status 전후 동일, 최종 clean이다. Inventory SHA256 `0ba508707401a0d28e321a0abf21dd543944e2e85720a5cf49bb78023c919ac6`. 최초269 Rust/249 client/브라우저 검사는 원래 후보의 결과이며 이번 표적 확인에서 반복하지 않았다. PR·같은 main CI는 이 문서 작성 시 후속 게이트다.

## 결정론과 소스 보존

- Fresh split: `e58d99fc710c06d7`; +8760 ticks: `33aeeaac00078319`; 별도 resume 전체 보고서 동일.
- V6 반복 save SHA256: `c4b81203b4073fb6686b1b4721c9ea4d46e4b5b58851050047ad70e087e34556`.
- 역사 V1 SHA256: `57b13c62ef1a6f88ade1851d0b3ae398157f95ca5bc246068eac3b2a964322fa`; 역사 split/resume `4eb703aad086c7aa`/`d19d028857bb7485`.
- 최초 QA566개 추적 파일·HEAD·목록·SHA·diff/status 전후 정확히 동일, 최종 clean. Inventory SHA256 `fc205bde043f377ed3d03e4c695e6f9cc1b1f8bc5f64c295864b2c53837bd6ba`.
- 기존 packs/goldens/repro/historical save fixtures/Cargo/npm locks bytes 보존. 새 시뮬레이션 float/HashMap 순회/RNG/시계/unsafe 및 원작 자료 사용 없음. provisional required defines·단위·반올림·민감도는 [ADR-1501](../adr/ADR-1501-production-contract.md)에 기록한다.

## 실제 실행 한계

최초 Cargo는 client dist 전제조건으로 exit101, legacy helper 첫 실행은 하드코딩된 target 경로 부재로 exit1이었다. 전제조건을 갖춘 후 위 최종 명령을 통과했다. npm pretest가 생성한 세션 소유 중복 build cache만 종료 뒤 정리했고 fixtures·증거·기존 바이너리·worktrees는 보존했다.

브라우저 첫 timeout은 미래 switch intent가 있는 입력으로 이전 모델 완료를 기다린 검증 설정 오류였다. 미래 intent 없는 새 입력으로 브라우저 검사 PASS를 확인하고 두 기록을 보존했다. 정확히 `/favicon.ico`404 console 오류만 ancillary이며 그 외 page/console 오류·실패/외부 요청 없음. 전체 console 무오류로 표현하지 않는다.

실행 중 임시 node_modules 링크 때문에 일부 helper에 dirty:true가 있었지만 추적 파일은 바뀌지 않았고 링크 제거 후 전후 비교가 동일했다. 실제 검증 tree에서 다시 빌드한 바이너리의 identity를 따로 확인했다. 과거 clean CI capture나 구현자의 재사용 바이너리 자체로 확대하지 않는다. Windows/macOS·추가 브라우저는 로컬 QA 범위 밖이며 PR·같은 main CI는 후속 게이트다.
