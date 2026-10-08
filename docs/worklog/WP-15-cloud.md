# WP-15 작업 로그 — production authority

| 항목 | 값 |
|---|---|
| 상태 | 독립 검증 대기 (구현자 자체 검사 결과, 완료 판정 아님) |
| 담당 | 구현 세션 |
| 브랜치 | wp/15-production-cloud |
| 기준 main | b112291c11822ffbf6da33b42670ff5cdb226fae |
| 대상 REQ | REQ-ECO-03, REQ-MIL-01 장비 재고 부분 |
| 선행 WP | 승인된 WP-14 economy/restore/P-07 repair |

## 계획과 구현 범위

사용자가 승인한 소수 장비 carry, 실제 생산일 효율 성장, 다른 모델 변경 시 부분 폐기, 자원 비례 배분을 구현했다. 수치 선택 위임에 따라 독립 synthetic provisional defines를 선택했다. 자세한 단위·수식·수명·경계는 [ADR-1501](../adr/ADR-1501-production-contract.md)에 기록했다. 게임 결정 문서의 상태는 변경하지 않았다.

실제 opt-in production 입력/strict loader/schema, 군수 Qty IC를 쓰는 생산 라인, whole i64 available stock, Fx 효율·Qty carry, 정확한 일일 자원 ledger, 공통 명령/atomic daily phase, additive V6 save/restore/repro, Rust query/command→생성 TS→별도 서버→브라우저 흐름을 추가했다. 화면에는 저장된 비용·군수 예산·라인 IC·효율·carry·재고·자원 충족률과 required/reserved/debited가 표시된다. 직접 입력한 IC를 고정소수점 전송값으로 인코딩하는 것 외에 클라이언트가 게임 수식을 계산하지 않는다.

## 테스트 우선 기록

| REQ | 테스트 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| REQ-ECO-03 | production 최초 carry/pre-growth/무효 설정/결정론 tests | `cargo test -p oh_sim --locked --test production`: exit101, production 모듈/API/Command가 없어17 compile errors | 동일 요구 검사가 통과, 최종 simulation production11 tests PASS |
| REQ-MIL-01 재고 부분 | whole stock/same-model 보존/different-model 폐기 | 같은 최초 테스트 컴파일 FAIL에 포함 | 생성·보존·폐기·retention·cancel 검사 PASS |
| REQ-ECO-03/저장 경계 | 실제 생성 입력 V6 codec/new-process resume | 첫 V6 왕복에서 기존 header 검사에 format6가 빠져 EconomyModeMismatch FAIL | V6 mode 검사 추가 후5 save tests PASS |
| 실제 UI integration | 생산 국가 선택 후 Create | 첫 브라우저 실행 FAIL: 이미 Join한 socket에서 두 번째 Join이 거절돼 생산 권한 없음 | 초기 Join 국가를 지정한 새 연결로 수정 후 실제 Chromium PASS |

추가 strict-input·공유 자원·multi-resource·range·sensitivity·forged-save·client transport 검사는 구현 중 확대했다. 이 추가 검사 전체가 구현 전에 실패했다고 주장하지 않는다. 최초 실패 출력과 원시 receipts는 Git 밖에 보존했다.

## 실행한 검증

Linux 환경 Rust1.99.0, Node24.19.0/npm11.9.0을 사용했다. CARGO_INCREMENTAL=0, build jobs2, debug info0의 단일 별도 target을 사용했다. npm 잠금 파일이 같은 기존 설치를 링크로 재사용했고 그 링크는 커밋하지 않는다.

| 명령/검사 | 종료 코드 | 실제 결과 |
|---|---|---|
| `cargo fmt --all --check` | 0 | 형식 통과 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 통과; 기존 ts-rs serde attribute 진단은 그대로 |
| `cargo test --workspace --locked` | 0 | 269 tests PASS, 실패·무시0 (doc tests 포함) |
| production 대상 검사 | 0 | data2/simulation11/save5; 실제 loader·단위·수식·rollback·codec·restore |
| `npm --prefix client test -- --reporter=dot` | 0 | 249 tests PASS; 실제 Rust production projection 검사3개 포함 |
| `npm --prefix client run build` | 0 | tsc + Vite production build; 기존 큰 chunk 경고 유지 |
| `cargo run -p oh_proto --example generate --locked` | 0 | TS 생성 |
| `cargo run -p oh_data --example schema --locked` | 0 | schema 생성 |
| `oh_cli validate --deny-warnings <generated-pack>` | 0 | synthetic production/economy pack strict 통과, pack hash b9123214f3381074 |
| `oh_cli run --pack <generated-pack> --scenario m1 --days 365 --seed 1 --hash-out` 2회 | 0,0 | opt-in production 초기 상태 hash c84de3c8e61be4f5 동일 (이 CLI 실행은 라인 명령 없이 시작) |
| `production_fixture capture OUT 8760` 2회 + 별도 `resume PACK SAVE 8760` | 0,0,0 | 실제 생산 라인/future switch 포함, 전체 authority reports 및 repeated save bytes 동일 |
| `check_save_current.py capture --out target/wp15/legacy-current` | 0 | 현재 None/V1 입력 및 기존 economy V5 native/new-process capture 통과 |
| `check_save_determinism.py capture --out target/wp15/legacy-v1` | 0 | 역사 V1 새 writer bytes가 보존 fixture와 정확히 동일 |
| 별도 `oh_server --load-save` + `node crates/oh_server/tests/production_query.cjs URL CAPTURE_JSON` | 0 (query helper) | tick24/carry/efficiency/queue/hash 복구, valid control·무효 모델/IC/ID/raw·duplicate sequence·second Join rejection·clean1000 close |
| 변경된 content hash의 normal server restore | 1 (의도된 거절) | 원본 저장에 대한 host mismatch 거절 |
| 변경 host `--force` + 같은 실제 query helper | 0 (query helper) | 경고와 함께 같은 생산 authority 유지; mode mismatch는 force에서도 거절 |
| installed Chromium151.0.7922.173 실제 server/browser | 0 (helper) | nation control, Create/SetIC/Pause/Resume, 성장/whole stock, 일일 steel ledger, Switch/cancel 재고 보존, ID 미재사용, EN/KO, map canvas |
| `python3 tools/check_architecture.py` | 0 | 의존성 방향 통과 |
| `python3 tools/check_assets.py --release` | 0 | 오류0 |
| `npm --prefix client run licenses` | 0 | 103 dependencies, 오류0 |
| `python3 tools/check_docs.py` | 0 | 오류·경고0 |
| `git diff --check` | 0 | whitespace 오류0 |
| frozen-input/lockfile diff against base | 0 | data/packs, tests/golden, tests/repro, save fixtures, Cargo.lock, client/package-lock.json 수정0 |

초기 workspace check는 built client dist가 없어 build prerequisite 오류가 났고 client build 후 해결했다. 처음 architecture 명령은 toolchain PATH 누락으로 실행되지 않았고 환경 로드 후 실제 exit0을 확인했다. 위 원인들은 assertion PASS로 바꾸어 기록하지 않았다. 골든 기대값 갱신·의존성 추가·기존 에셋/입력 수정은 하지 않았다.

## 재현 입력과 증거

새 합성 원천은 `oh_data/tests/fixtures/production/{valid.toml,defines.toml,en.ftl,ko.ftl}`이며 assets manifest에 등록했다. `oh_data/tests/support/production.rs`가 frozen M1을 새로운 ignored `target/wp15-fixtures/<process>-<serial>/testland`로 복사하고 기존 합성 economy와 새 production 입력을 그 복사본에만 추가한다. 테스트 geometry는2국가/6프로빈스 M1 복사본이며 원본 M1 또는 최종 M2 장비 팩이라고 주장하지 않는다.

재현 가능한 source example:

```sh
cargo run -p oh_save --example production_fixture --locked -- capture target/wp15/capture-a 8760
cargo run -p oh_save --example production_fixture --locked -- capture target/wp15/capture-b 8760
# capture JSON의 pack_path와 생성된 saved.ohsave를 사용한다.
cargo run -p oh_save --example production_fixture --locked -- resume PACK target/wp15/capture-a/saved.ohsave 8760
# 서버는 pack-root/testland를 선택하므로 PACK의 부모를 사용한다.
oh_server --port 19416 --pack-root PACK_PARENT --scenario m1 --load-save target/wp15/capture-a/paused.ohsave
node crates/oh_server/tests/production_query.cjs http://127.0.0.1:19416/ target/wp15/capture-a/capture.json
```

실제 라인 생산 capture의 split hash e58d99fc710c06d7, +8760 ticks hash33aeeaac00078319, 동일 V6 저장 SHA256 c4b81203b4073fb6686b1b4721c9ea4d46e4b5b58851050047ad70e087e34556. 별도 프로세스에서도 split/continued 전체 report와 canonical bytes가 일치했다. 이전 +240 ticks 실행도 d5adf29dd8b71bf9로 일치했다. 역사 V1 SHA25657b13c62ef1a6f88ade1851d0b3ae398157f95ca5bc246068eac3b2a964322fa와 split4eb703aad086c7aa/resume d19d028857bb7485를 그대로 재생성했다.

이것은 Linux 구현 중의 검사이며 clean frozen commit 독립 P-05나 Windows/macOS 동등 실행 증거가 아니다. 원시 logs·capture·screenshots·browser helper는 ignored target 또는 외부 scratch에 두며 public summary만 기록한다.

## ADR / 차별화 초안

[ADR-1501](../adr/ADR-1501-production-contract.md): required data defines, pre-growth output, exact proportional residuals, conservative raw resource ceilings, no hidden redistribution, checked atomic stock/carry ledger, frozen legacy/new V6 split. Binary-friendly synthetic tuning과192/96/48 productive-day sensitivity, model change partial disposal, transparent authority resource ledger를 OpenHOI4의 독립 설계 설명 초안으로 제안한다. 오케스트레이터가 승인된 전역 문서에 반영한다.

## 결정 필요 / 범위 밖 발견 / 남은 작업

현재 WP-15 정책의 추가 승인 blocker는 없다. WP-16의 편제/충원 소비, research·combat·vehicle 효과, WP-19 runtime demand/rail 연결, 최종 M2 장비/철도/OOB 팩은 구현하지 않았다. 미지원 AddEquipment 효과는 amount0에도 미지원이며 fake capability를 추가하지 않았다.

기존 로컬 서버는 연결마다 독립 simulation이다. 국가 선택은 새 local 연결을 시작/복구하는 동작이고 기존 게임의 공유 multiplayer 또는 reconnect persistence를 보장하지 않는다. 두 번째 Join 거절 정책은 보존했다. 브라우저의 기존 `/favicon.ico`404 한 건을 정확한 URL/message로 별도 ancillary로 기록했다. 다른 console/page 오류, failed/external requests는 없었다; 전체 console 무오류라고 주장하지 않는다.

공개된 검증 결과는 구현자 자체 검사다. 별도 새 검증 세션의 P-05, 오케스트레이터의 다른 독립 모듈과 통합, 동일 main CI는 아직 필요하다. 이 로그는 WP-15 또는 M2 전체 완료를 선언하지 않는다.
