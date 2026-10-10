# WP-23 작업 로그 — 최소 합성 군사 팩

| 항목 | 값 |
|---|---|
| 상태 | 구현 검증 기록 작성, 독립 검토/QA/통합 대기 |
| 담당 | Codex 구현 세션; 실제 Claude 계획을 받아 구현 |
| 브랜치 | `wp/23-m2-military-pack` |
| 기준 main | `0d50a83f862bf575e8051a1c7cf12da7f0ea0428` |
| 대상 REQ | REQ-CNT-01·REQ-GEN-03의 최소 군사 콘텐츠 부분; REQ-MIL-01/02/03·REQ-SAV-01/02·REQ-GEN-04 기존 계약의 회귀 증거 |
| 선행 WP | 인수된 생산·군사·V7 계약 및 WP-22 조회 패널 |

## 계획과 변경 범위

새 `data/packs/testland_m2_military`32파일, 전용 Rust10검사/공용 테스트 helper, 별도 CJS loopback, 별도 Playwright13검사와 config를 추가했다. `assets/ASSETS.toml`에는32레코드를 append했고 기존 바이트를 유지했다. 기존 팩/검사/CI/HANDOFF/제품 Rust/스키마/생성 TS/클라이언트 UI/01·02·ADR은 변경하지 않았다. PNG도 커밋 대상이다.

6국/120프로빈스/12주의 기존 production 팩을 복사했다. 새 pack name ko/en과 군사 initial을 선택하며, N03/N06만 model1 허용·재고0을 더했다. 이것은 전 국가 템플릿 바인딩을 요구하는 기존 로더를 위한 fixture 구성이다. 국가별 템플릿 허용 정책을 새로 정의하지 않는다. 군사 입력은 기존 small의 인력8·장비6·훈련2일/11개 stats, 군대6개 capacity2·priority0, 배경0/0·초기 사단/작업0이다. 생산 모델/9defines·지도/경제/국가/소유/시간 바이트는 유지했다.

## 테스트 우선 기록

| 요구 범위 | 검사 | 구현 전/첫 실행 | 구현 후 |
|---|---|---|---|
| 팩과 공통 바인딩 | strict_six_nation_input_and_all_model_bindings | 새 팩 부재 실행 실패. 기존 production allowlist 복사 직후 N03/N06의 `DisallowedModel` 의미 실패(`red-model-binding.log`) | 두 국가만 model1 확장, strict0warnings; 전용 data5 PASS |
| 로더 음성 대조 | semantic_rejections/strict_unused/missing_model | 오류 종류/중복 general 범위의 첫 기대 오류 보존 | allowlist/background/army/trainingdays/context/unknown binding/division/production/FTL/unused와 정상 대조 PASS |
| 실제 생산 관측 | observed_production_training_cancel_conservation_and_v7_resume | `red-schedule.log`의 초기0 예상값 실패는 **관측 보정 기록**이며 제품 결함 RED가 아님 | 6국 모두 D5, Train tick120, first Training tick144, held6/stock2 |
| 회계 거부 | nonzero_background_without_ownership_is_accounting_mismatch | SaveContext 문자열에서 typed 원인 검색은 실패 | 직접 Simulation typed `AccountingMismatch`와 SaveContext의 기존 generic 실패를 각각 검사; 정상 대조 PASS |
| 네이티브 프로세스 | fresh_process_measures_all_nations_then_v7_resume_equals_continuous_and_repeat | 팩 내부 save 경로는 기존 SavePath guard가 거부 | 소유 temp의 팩 밖 save로 변경, 새 OS process 복원/연속 실행/반복 bytes 일치 PASS |
| 서버 준비 | host_accepts_fresh_and_real_paused_v7/negative controls | 새 checkout client/dist 부재, helper의 JSON 의존성 준비 오류 | client build 먼저, 새 의존성 없이 전용 server2 PASS |
| 실제 브라우저 | 새 m2-military13 | 최초 명령 cwd 오류; 출처 문서 추가 후 오래된 checkpoint는 실제 `PackMismatch`로 거부 | 팩 파일 확정 후 새 checkpoint로13 PASS; 실패 이력 보존 |

초기 Rust 컴파일 오류(private getter/형 추론/없는 Army getter)는 준비 실패이고 의미 있는 요구 RED로 세지 않았다. 원래 테스트를 지우거나 약화하지 않았다. `AccountingMismatch` 배경, Train 실패, 중복/비소유 Cancel을 통과시키기 위한 제품 수정은 없다.

## 실행한 검증

증거 루트는 Git 밖 `/workspace/scratch/openhoi-wp23-military-pack`이다. 브라우저 산출물은 이 worktree의 `target/wp23/` 아래에 있다. 공통 환경은 Rust1.99.0, Node24.19.0/npm11.9.0, Python3.12, system Chromium151.0.7922.173이며 `source /workspace/toolchains/env.sh` 후 `CARGO_TARGET_DIR=/tmp/openhoi-wp15-target`, jobs2/incremental0/test·dev debug0/offline=true를 적용했다. client node_modules는 같은 lockfile의 기존 캐시를 잠시 링크했고 새 의존성을 설치하지 않았다.

| 실제 명령/수용 검사 | 종료 | 결과/증거 |
|---|---|---|
| `cargo fmt --all -- --check` |0| fmt.log |
| `cargo clippy --workspace --all-targets -- -D warnings` |0| clippy-first.log; 기존 ts-rs serde attribute macro 경고는 보존 |
| `cargo test --workspace` |0| workspace-first.log:75 suite,401 PASS,0fail/ignored(문서 검사 포함) |
| `cargo test -p oh_data --test m2_military_pack` |0| data-final.log:5 PASS |
| `cargo test -p oh_save --test m2_military_pack` + 선택 output env |0| save-reviewed.log:2 PASS; checkpoint-reviewed/{paused-training.ohsave,expected.json} |
| `cargo test -p oh_cli --test m2_military_pack` |0| native-second.log:1 PASS; workspace에서도 새팩으로 PASS |
| `cargo test -p oh_server --test m2_military_pack` |0| server-pass.log:2 PASS; workspace에서도 PASS |
| `oh_cli validate --deny-warnings` new military/old production/old m2 |0 각각| strict-testland_m2_military/production/m2.log:0warnings; 최종 군사 pack hash862cf8ea21596178 |
| `npm --prefix client test`(실제 pretest wire fixtures 포함) |0| client-test.log:17file/269 PASS |
| `npm --prefix client run build` |0| client-build-second.log; 기존 큰 번들 경고 유지 |
| 새 CJS 실제 fresh/restored 서버 |0| wire-reviewed.log + wire-reviewed/receipts.json, 각6국 별도소켓; served JS byte equality, rejection/conservation,12 close1000 |
| 새 Playwright 군사 팩 |0| browser-reviewed.log:13 PASS, target/wp23/browser-reviewed; 6국 native Join fresh/restored, 실제 Training, ko/mobile/closedquery0/noMilitaryCommand |
| 기존 Playwright 군사 패널 |0| browser-regression.log:8 PASS, target/wp23/browser-regression; 기존 검사 파일 불변 |
| 기존 O2 CJS fresh/restored |0| o2-regression-final.log + o2-regression/{fresh,restored}-wire.json; 각6국/actual servedJS/close1000 |
| docs/assets release/localisation/architecture/npm license |0 각각| docs.log/assets.log/localisation.log/architecture.log/npm-licenses.log; docs0warning/assets0error/npm103deps0error. 기존 client unused localisation 경고는 유지 |
| `git diff --check` |0| 공백 오류 없음 |

전체 Rust suite에는 기존 economy/schema/legacy save/production/military/target/restore 회귀 검사가 포함된다. Linux 로컬 실행 기록이며 Windows/macOS/browser별 통과나 아직 실행하지 않은 exact-head CI 성공을 대신 주장하지 않는다. 새 CJS/새 Playwright config는 기존 CI에 연결되지 않으므로 리뷰/QA가 직접 실행해야 한다.

## 관측과 보존

시드1의 같은 스크립트 두 실행은 canonical/hash/저장 bytes가 같다. 실제 원장 allocation[2]로 생산하고 초기 stock0에서 첫 stock≥6을 각 국가에서365일 이내 측정했다. 모두5일, Train120, Training144이며 각 국가 available242/committed0/reserved8·held6·stock2, paused hash `668a2d664ea11fe8`이다. Pause 뒤 Cancel은 reserved−8/available+8/committed불변/stock+held6, progress0/startNone/equipmentempty를 보존한다. nextJob6/사단0/빈 queue 및 생산선 원 bits는 불변이다. 비소유/없는 template/Terminal 거부는 canonical/hash/자원 회계에 영향을 주지 않는다. 새 OS process V7 query와 동일 tail 뒤 상태/저장 bytes도 연속 실행과 일치한다.

각 웹소켓은 독립 simulation을 가지므로6국 질의를 하나의 공유 command stream으로 오해하지 않았다. fresh/restored 별도 CJS hosts2, 새 Playwright hosts2, 기존 패널 hosts2, O2 hosts2는 소유 PID ledger와 finally SIGINT/exit0/no residual을 확인한다. 5초 후 SIGKILL은 실패 정리용이며 PASS로 인정하지 않는다.

처음 checkpoint의 팩 hash는 출처 문서 추가 뒤 달라졌고 서버가 PackMismatch로 거부했다. stale checkpoint를 강제 로드하지 않았으며 새 출력 디렉터리로 재생성했다. README의 코드 라이선스 표기를 프로젝트의 GPL-3.0-or-later로 정정한 뒤에도 새 checkpoint-reviewed 및 실제 wire/browser를 다시 실행했다. 모든 실패 로그는 보존한다.

공유 캐시가 커져 /tmp 여유가 일시적으로1GiB 아래로 내려간 것을 발견했다. 기존 source/worktree/증거를 삭제하지 않고 Cargo package cache와 이미 검사된 재생성 가능한 테스트 바이너리만 정리했다(`completed-test-cache-cleanup.json`). 후속 무거운 작업 전 여유를 회복했다. 공간 부족 실패/정책 변경은 없었다. 원래 작업 트리와 다른 lane의 WIP는 유지한다.

## 계획에서 실제 계약에 맞춘 최소 보정

- 새 name_key를 쓰고 기존 production.ftl을 그대로 복사하면 unused `o2-pack-name`의 ko/en 경고로 strict가 실패한다. **새 팩의 해당 한 줄만 제거**했고 장비 키는 그대로다. 바이트 보존 검사가 이 정확한 FTL 차이를 검사한다.
- general 중복은 기존 계약의 `(nation,general)` 중복이며 다른 국가의 같은 문자열은 허용된다. 음성 대조는 같은 nation/general을 중복시킨다.
- component의 model만 바꾸면 기존 `BindingContextMismatch`, component와 binding을 함께 unknown으로 바꾸면 `InvalidReference`다. 두 경우를 별도로 검사하며 제품 error contract를 바꾸지 않았다.
- SaveContext는 typed 오류를 문자열로 축약한다. typed 회계 원인은 Simulation API로 확인하고 저장 API의 실제 거부도 별도로 확인한다.

## ADR / 결정 필요 / 범위 밖

새 ADR이나 게임 정책 결정을 만들지 않았다. 생산/템플릿/군사/저장/프로토콜 인수 계약을 유지한다. 새 국가별 템플릿 정책·UI 군사 명령·훈련 설계·전투·외교·AI·WP19 런타임 소비·전체WP23/M2는 범위 밖이며 미완료다. 원작 대비 차별화 초안은 “프로프라이어터리 자산 없이 독립 합성 최소 입력과 권위 서버 자원/복원 검증”이며 승인 없이01 §11에 넣지 않았다.

AI가 TOML/문서/검사 구현을 보조했다. 새32 asset notes와 SOURCES에 명시했다. ai_generated=false는 기존 OPEN-09/check_assets의 이미지 분류이며 PNG 바이트 복사·이미지 생성 없음에 따른다. 인간 저작 단독 주장이나 허위 배포 승인 기록을 만들지 않았다. 별도 실제 Claude 독립 코드 리뷰와 독립 QA는 **대기**다.

이 구현 세션은 merge하지 않는다. Draft PR/exact-head CI와 independent review/QA/main 인수는 부모 오케스트레이터가 관리한다. 별도 개발 lane의 도구 설치/LOGH 원본 실행 승인/BP 사용자 요구 대기는 이 군사 팩 검증으로 해소된 것으로 기록하지 않는다.
