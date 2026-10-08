# 검증 리포트 — WP-16 로컬 정상 편제 단위

| 항목 | 값 |
|---|---|
| 판정 | **PASS — 지정한 로컬 정상 편제 정의·계산·실제 모델 참조·CLI 범위만** |
| 검증자 | 새 독립 Codex 검증 세션, 구현 비참여, 별도 worktree, 추적 소스 무변경 |
| 제품 후보 | `1f858ba2e174b08f8d27da8b26aaa37a2352b43e` / `wp/16-templates-local` |
| 기준 main | `5ca938e5c49d37736fbb778df53a820d6321804c` |
| 검증 시각 | 2026-10-08 22:23:45 UTC |
| 전체 제품 검토 | 14경로 한 번, finding0, P06 없음 |

사용자 승인 범위는 family별 명시 모델, 필드별 합계/인력 가중 평균, 구성 최저 속도다(01 §4.7). Strict 외부 TOML을 읽고 구성 작성자가 명시한 resolved 정상 능력치를 계산한다. 실제 선택 시나리오의 production registry·모델/family·국가 허용 참조를 검증한다. 생산 모델에 없는 전투 능력치로 장비 개수당 기여를 유도하지 않는다. 출력은 `normal_template_stats`, `component_declared_resolved_normal_inputs`, editor legality `not_evaluated`를 명시한다. [ADR-1601](../adr/ADR-1601-resolved-normal-templates.md)과 [사용 안내](../../crates/oh_sim/src/military_templates/README.md)를 따른다.

## 직접 실행한 검사

| 명령·대상 | 종료 코드 | 결과 |
|---|---|---|
| `cargo test --offline --locked --workspace` | 0 | Rust345 PASS,63 suites, failed0/ignored0; 새 source tests13개 포함 |
| `cargo fmt --check` | 0 | PASS |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | 0 | PASS; 기존 ts-rs attribute notices는 보존 |
| `python3 tools/check_docs.py` | 0 | 오류0·경고0 |
| `python3 tools/check_assets.py --release` / `python3 tools/check_architecture.py` | 0 / 0 | PASS |
| `cargo run --offline --locked -p oh_data --example schema` / `cargo run --offline --locked -p oh_proto --example generate` | 0 / 0 | 11경로 재생성 bytes 동일, 정적 legacy schema 포함12경로 보존 |
| `cargo build --offline --locked -p oh_cli -p oh_server` | 0 | 실제 native CLI/server 실행 파일 빌드 |
| 독립 정수 oracle + 실제 `military-template inspect` | 0 | 사전 작성335사례(정상330, overflow/undefined5); 반복·재정렬/동등 decimal·strict 거부/유효 control·추가 경계 포함1241프로세스 모두 PASS |
| 독립 순수 계산 불변성 harness | 0 (최종) | 같은335사례에서 Definitions canonical/identity, 실제 LoadedNational production/economy/scenario/nations 및 Simulation canonical/hash 보존 |
| `python3 crates/oh_server/tests/economy_target_native.py --out <owned-out> --bin-dir <built-bins>` | 0 | 기존 native target12 PASS |
| `python3 crates/oh_server/tests/economy_restore_native.py --out <owned-out>` | 0 / 0 | 기존 restore4 PASS; 임시 의존성 링크 제거 후 추가 clean restore4 표적 PASS |
| CLI strict M1/M2 팩 검증·M1 반복/저장/새 process 재개 | 0 | 아래 결정론·호환 확인 |

모든 Cargo 실행은 Rust1.99.0의 locked/offline·기존 공유 외부 target을 사용했다. 새 의존성·도구/npm/browser 다운로드는 없었다. 기존 client 입력81경로가 byte-identical임을 직접 확인한 뒤 인수된 ignored dist18파일을 hardlink로 재사용했다. 이번 로컬 QA에서 client/browser 검사를 새로 실행했다고 기록하지 않는다.

## REQ별 확인

| REQ 부분 | 실제 검사 | 결과 |
|---|---|---|
| REQ-MIL-02 정의·정상 자동 계산 | 필수 named stats/decimal/domain/unknown-key·원본 duplicate·role/reference·12/4/반복·checked sums·wide i128 가중 최종 floor·Fx minimum | PASS |
| REQ-MIL-02 실제 생산 참조 | 실제 temporary production-positive M1 loader, 모델 미존재/다른 family/국가 불허/선택 binding 문맥 불일치·production None와 유효 controls | PASS |
| REQ-MIL-03 생성 전 정상 요구량 설명 | whole 인력/장비 요구 합계·0 장비·인력0 지원·empty/zero-weight undefined arithmetic | PASS; 사단 생성·훈련·배치 검사가 아님 |

추가 실제 CLI controls는 두 family/model, nation0, fixed quantization/ties-even, 정확1MiB 정상 입력, 사용하지 않는 binding의 실제 모델 검사다. 단순 평균 대신 인력 가중 평균을 검사하고 최종 raw remainder를 대조했다. 구성 occurrence를 제거하지 않으며 지원 인력0을 가중치1로 바꾸지 않는다. 빈 구성/전체 인력0은 파싱 후 계산 불가를 보고하며 새로운 편집기 legality로 기록하지 않는다.

## 결정론·기존 상태·호환

- Strict M1 pack hash: `3bde1bed90d3734e`; M2: `d656804578786a61`.
- M1 seed1 두 fresh process365일과180일 저장 후 새 process185일 재개: 모두 `b595dc2a1e5b4f8c`;180일 split: `b7eca53b4860eadd`.
- 실제 저장 decode의 전체 state canonical SHA256: `a4819acd3db21703ff9d1325854c02729ddd5cbb9a87c137eaf32a07cba41a10`; saved_at0 정규화 bytes SHA256: `fd5e3ca6ba103bc72d96770bc14a7541c25696a2edaf4e482c1909a7abf7f8e3`. 원 envelope roundtrip과 format1/ProductionNone를 보존했다.
- 순수335사례의 실제 Simulation hash `3d80989f13c78f34`와 canonical bytes는 성공·거부 호출 전후 동일했다. 정상 CLI 출력 semantic SHA256 `ec4518157c36983f7543f267db8e537550574e3369199674539d2048c69908ce`는 **진단 출력**이며 Simulation hash가 아니다.
- QA 전후 HEAD·592개 추적 파일 목록/각 SHA·diff/status가 동일·clean이다. 기준582파일 중4개 module/dispatch 등록 외578파일 bytes는 그대로이며 새10파일을 추가했다. 기존 팩·golden·repro·schema/save/wire/TS·라이선스/에셋/lock은 바꾸지 않았다.

## 보존한 초기 시도와 표적 확인

독립 불변성 harness 첫 rustc compile은 캐시 rlib의 서로 다른 serde metadata 선택으로 exit1이었다. Cargo JSON의 정확한 artifact 경로를 사용해 준비를 고쳤고 최종 실행은0이다. Baseline aggregate 첫 assertion은 사람이 읽는 validate stdout 전체와 bare hash를 비교해 exit1이었다. 실제 여섯 명령은 모두0이었고 보존한 같은 stdout 파서만 고쳤으며 런타임을 다시 실행하지 않았다.

첫 restore4는 임시 untracked `client/node_modules` 링크 때문에 `dirty:true`를 기록했다. 추적 파일은 바뀌지 않았으며 링크를 제거하고 clean 상태에서 restore4만 추가 표적 확인해 `dirty:false`를 확인했다. 기존 시도와 후속 확인을 모두 남겼다. 제품 finding/P06 또는 두 번째 전체 검토는 아니다. 구현 세션의 세 requirement-first runtime red와 세 초기 green compile/fixture 실패도 [작업 로그](../worklog/WP-16-normal-templates.md)에 별도로 기록한다.

## 금지 사항·남은 게이트

부동소수점·HashMap 순회·시계/RNG/I/O를 순수 집계에 추가하지 않았다. 새 WorldDefs/Simulation/phase/command·schema/save/wire/client·원작 자료/에셋은 없다. 모델은 실제 참조 검증에만 쓰며 전투 능력치 자동 도출을 주장하지 않는다.

별도 게시 승인 확인이 거절되어 부모 세션이 사용자에게 확인 중이다. **로컬 구현·검사만 허용**하므로 push0/PR0/merge0/CI0이다. main은 인수된 기준5ca를 유지하고 이번 결과는 같은 main CI 통합 게이트가 아니다. 실제 사단/OOB·군/장군·훈련 예약/반환/보충·감편 수요/손실·daily Supply·save/query/wire/UI와 전체 WP-16/M2는 후속이다. 기존01 §4.9의 조직력 회복 요구를 새 미결정으로 바꾸지 않는다.
