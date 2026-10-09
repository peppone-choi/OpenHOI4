# WP-23 작업 로그 — 독립 합성 생산 입력

작성: 최병호 · 2026-10-09

| 항목 | 값 |
|---|---|
| 상태 | 독립 검증 대기 |
| 담당 | 구현 |
| 브랜치 | `wp/o2-m2-production-pack` |
| 대상 | WP-23 초기 콘텐츠와 기존 WP-15 생산/V6/서버 계약의 콘텐츠 인수 |
| 기준 | main `ef6d1fa1fcd9b7cd6350649b25efd11995d2e7ad` |

## 변경

자작 `testland_m2`의6국/120프로빈스/12주/경제 입력을 독립 팩 `testland_m2_production`으로 복사했다. 기존 팩은 수정하지 않았다. `m2_production`이 두 합성 장비 모델과6국 허용 목록·0 재고를 선택한다. 기존 WP-15 fixture의9개 production defines, 두 모델의 비용/steel recipe를 사용하며 밸런스는 잠정이다. ko/en·CC-BY-SA-4.0 자산 manifest·출처/단위/민감도는 팩에 기록했다.

경제/생산 Rust 제품 코드, schema, 생성 wire, 저장 포맷, 훈련/군사 정책은 바꾸지 않았다. 전체 M2/WP-23 완료 판정이 아니다.

## 테스트 우선 기록

팩 작성 전에 전용 data 검사3건을 추가해 실행했다. 새 팩이 없어3건 모두 실패했다. 팩 작성 후 첫 엄격 검사에서는 복사된 구 pack-name 키가 사용되지 않아2건 실패했고, 새 팩의 현지화만 바로잡아3건 통과했다. 이 기록을 save/server 검사의 구현 전 실패나 독립 검증으로 확대하지 않는다.

## 실행한 검사

| 명령/검사 | 실제 결과 |
|---|---|
| `cargo test --offline --locked -p oh_data --test o2_production_pack` |3 PASS:6국/120프로빈스/12주·production·ko/en strict; unknown field/duplicate allowlist/unknown resource/missing tuning; missing key/unused producer 음성 대조 |
| `cargo test --offline --locked -p oh_save --test o2_production_pack` |3 PASS: 실제6국 Create와365일 두 번;180+185일 실제 V6 resume; 불허 모델/재고 overflow atomicity와 유효 대조 |
| `cargo test --offline --locked -p oh_server --test o2_production_pack` |2 PASS: 새 팩 시작·6국180일 V6 복원; unused producer/누락 en key 거부와 유효 대조 |
| `cargo test --offline --locked -p oh_save --test production` | 기존 V6 계약5 PASS |
| `cargo test --offline --locked -p oh_save --test save --test economy_restore` | 기존 M1 저장2·경제 restore2 PASS; legacy 경로와 음성/유효 대조 |
| `cargo test --offline --locked -p oh_server --lib` | 기존 서버9 PASS; nativeTargetConflict/MINInvalidValue 및 실제 정상 restore 대조 포함 |
| `cargo fmt --all --check` | exit0 |
| `cargo clippy --offline --locked -p oh_data -p oh_save -p oh_server --all-targets -- -D warnings` | exit0; 기존 ts-rs serde attribute 안내2건 |
| `npm --prefix client run build`, `cargo build --offline --locked -p oh_server` | exit0; 현재 source 번들 사용 |
| CLI `validate --deny-warnings` | 새 팩 strict exit0 |
| `python3 tools/check_docs.py`, `check_assets.py --release`, `check_architecture.py` | 모두 exit0; 문서 오류/경고0·자산 오류0 |
| `python3 tools/check_localisation.py` | exit0; 기존 client dynamic key의 unused 경고44건. client 현지화 파일은 기준 main과 동일하며 새 팩 ko/en은 strict 경고0 |
| `o2_production_query.cjs`, fresh/restored 실제 loopback 서버 | 각6국 PASS; HTTP JS 바이트 일치, 모델 단위/stock/carry/efficiency/steel ledger, 허용·거부/중복 sequence·권한 검사;12소켓 close1000, 두 서버 exit0 |

365일 hash는 두 실행과 resume 모두 `8d775068118aa67a`였다. 총 steel 차감 raw Qty bits는 국가1/2/4/5 `89247962`, 국가3/6 `89247668`이며 모두 양수였다. 매일 차감이 각국 실제 flow 한도를 넘지 않는지 검사했다. 원본 정의를 수정하지 않고 overflow 음성 입력은 검사 소유 임시 복사본에서만 만들었다.

## 재현과 한계

전용 검사는 `crates/{oh_data,oh_save,oh_server}/tests/o2_production_pack.rs`, 별도 프로세스 검사는 `crates/oh_server/tests/o2_production_query.cjs`다. 공개 재현 순서는 팩 README를 따른다. 생성 save·raw 실행 출력은 팩이나 Git에 넣지 않는다. 고정 metadata의 실제 V6 encode/decode와 마지막 save bytes를 대조했고 기존 기대값을 재생성하지 않았다.

각 서버 소켓은 별도 simulation이므로 공유 멀티플레이 인수가 아니다. 실제 브라우저의 경제 UI·OOB·훈련·보충·보급 consumer·최종 콘텐츠/역사 밸런스는 이 단위의 범위 밖이다. 독립 리뷰와 같은 main CI 통합 판정은 아직 없다.
