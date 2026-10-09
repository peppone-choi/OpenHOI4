# OpenHOI4 WIP 인수인계

## 경제 조회 UI 후보 (2026-10-09, 작성: 최병호)

실제 독립 리뷰가 기존 `0728903` 후보에서 연속 Delta의 조회 굶주림과 재연결의 이전 production 유지 두 결함을 찾았다. 같은 socket/국가의 대기 조회 ID를 보존하고 dirty 갱신을 후속 조회 하나로 모으며, 재연결 때 production view/검증 연결을 비워 현재 연결의 검증 응답 전에 명령을 차단하도록 수정했다. 두 실제 브라우저 실패를 재현한 뒤 client256개·실제 Chromium7개가 통과했고 최종 Playwright exit0을 기록했다. 기존 CI 성공과 이번 수정의 새 리뷰/CI를 구분한다. [작업 로그](docs/worklog/WP-14-economy-query-ui.md)의 delta 절을 따르며 새 실제 독립 리뷰와 최종 SHA CI/main 통합은 대기다.

main `ef6d1fa1fcd9b7cd6350649b25efd11995d2e7ad` 기반 `wp/o3-economy-panel`은 읽기 전용 EconomyPanel과 현재 socket/국가 수명·최신 발행 요청의 응답 관리 단위다. 국가/연결 전환은 화면을 비우고 malformed/disconnect는 마지막 검증 view를 stale로 남긴다. 미발행 미래/역순/이전 연결/중복 응답은 수락하지 않는다. 서버의 문자열 숫자와 산업·건설 원장, 배분 위치0/1/2/3을 보존한다. Rust/protocol/validator/save·기존 팩·법령/게임 계산은 수정하지 않았다.

구현자의 client255검사, 실제 소스 client/server 빌드와 실제 Chromium5검사가 통과했다. 기존 M2의 별도 임시 testland host·HTTP JS bytes·6국·ko/en, 큰 정수/빈 배열, 역순/미발행 응답·unsupported·stale·재연결을 검사했다. 경제 없는 원본 M1의 generic QueryResult도 최신 발행 ID일 때만 unsupported로 표시하며 Rust/wire를 바꾸지 않았다. 지연 프록시는 pause 중에도 발행되는 반복 Delta의 전달 창을 제어하며 정상 검사는 실제 전체 WS 흐름을 그대로 사용한다. 초기 지연 검사 실패와 최종 통과의 경계는 [작업 로그](docs/worklog/WP-14-economy-query-ui.md)에 남겼다.

[ADR-1401](docs/adr/ADR-1401-economy-query-display.md)의 표시·응답 수명을 따른다. 독립 리뷰/main CI 인수 전이며 전체 WP-14/M2나 공유 멀티플레이 완료가 아니다. O2 생산 팩·O1 군사훈련 정책과 별도 브랜치다.

2026-10-08 사용자 재개 지시로 새 환경에서 WP-14 독립 인수·P07 검사를 시작했다. 이번 재개는 미완성 소스의 검증부터 진행하며 릴리스나 M2 완료 판정이 아니다. 반복 감시 자동화는 PAUSED로 유지한다. 실행 계획과 공유 변경 순서는 [M2 재개 계획](docs/plans/M2.md)을 따른다.

## 새 환경 재개 상태

- 시작 기준 main은 `743c2a941af5422b94e850b6064b19cffbebac31`이며 원격과 로컬 HEAD가 일치했다. 초기 추적 파일 변경은 없었다.
- 같은 기준 main의 7개 push workflow가 success인 것을 새로 확인했다. [CI](https://github.com/peppone-choi/OpenHOI4/actions/runs/37783866384), [Save determinism](https://github.com/peppone-choi/OpenHOI4/actions/runs/37783866339) 등이다. 이는 새 WP-14 독립 인수 결과나 과거 P07 전체 PASS를 대신하지 않는다.
- Rust 1.99.0·rustfmt·clippy, Node 24.19.0/npm 11.9.0, Python 3.12.14를 확인했다. 캐시와 빌드 산출물은 작업 환경의 쓰기 가능한 경로를 사용한다.
- 새 검증 담당은 별도 worktree에서 실행하며 추적 소스를 수정하지 않는다. 두 fixture manifest·save hash, 문서·release 에셋·fmt·clippy, Rust 250개 검사, 클라이언트 typecheck/build와 시스템 Chromium을 사용한 M0 브라우저 12개 검사는 통과했다. full client 검사는 최초 palette timeout을 남긴 뒤 조용한 환경에서 같은 timeout으로 246개 모두 통과했다. target2 유효 server control에서는 정상 경제 응답 뒤 WebSocket close 오류가 남았다. P07은 FAIL이며 P06 수정·새 독립 검증이 필요하다. 도구/CDN 차단과 테스트 실패를 구분하고 전체 PASS로 기록하지 않는다.
- 위 초기 FAIL 뒤 Close 응답을 flush하는 서버 수정과 전체 값 비교를 보존한 palette 검사 변경을 별도 구현 담당이 작성했다. 수정 소스 `4ae0446a01487e9d6d1415e7191a78407a8e0b6f`에 새 독립 검증이 지정한 WP-14 producer·P07 repair 범위에서 PASS를 판정했다. Rust 251개, 기본 client 246개, unchanged native target 12개·restore 4개, 저장/재개·legacy·별도 서버 lifecycle, 시스템 Chromium M0 12개·M1 3개가 통과했다. 검증 전후 HEAD·505개 추적 파일 SHA·diff/status는 동일했다. [공개 검증 요약](docs/verify/WP-14-P07-cloud-repair.md)을 따른다. 초기 FAIL은 이 후속 결과와 구분하여 보존한다.
- 수정과 재개 문서는 [PR #2](https://github.com/peppone-choi/OpenHOI4/pull/2)로 정상 병합됐다. main `0776e6b5a67951821860490964c402e518310ca5`의 CI 7개가 모두 success이며 검증한 제품 tree와 같다. 지정한 WP-14 producer·P07 repair 통합 게이트를 충족했다. 후속 production/훈련/항복 consumer와 M2 전체 완료는 아니다. Firefox/WebKit·추가 M1 지도 suites·대규모 성능은 이번 검증 범위가 아니다.
- WP-23 초기 합성 팩 구현 `b7c5b6898855a136456f7b558724f18cafa0656b`는 6개국·120개 육지 프로빈스·12개 주의 별도 `testland_m2` 입력이다. 기존 2개국·6프로빈스 M1·golden은 보존했다. [작업 로그](docs/worklog/WP-23-initial.md)와 [독립 검증](docs/verify/WP-23-initial-cloud.md)을 따른다. 한 번의 새 독립 검증은 지정한 초기 범위 PASS이며 전후535개 추적 파일 bytes·HEAD·diff/status가 동일했다. [PR #3](https://github.com/peppone-choi/OpenHOI4/pull/3)는 main `b112291c11822ffbf6da33b42670ff5cdb226fae`로 정상 병합됐고 같은 main의 CI 7개가 모두 success다. 지정한 초기 팩 통합 게이트를 충족했으며 WP-23 최종 플레이 입력·M2는 미완료다.
- 사용자는 생산 소수 이월·실제 생산 때만 효율 성장·다른 모델 전환 시 미완성량 폐기·부족 자원 비례 배분과, 보급 공유 철도 용량·거리 손실·첫 버전 고정 경로를 승인했다. 초기 수치는 조절 가능한 provisional 데이터로 선택·검증하도록 위임했다. WP-15 구현은 `c7b404a96c685068e6e79c79fecad4be3b42fd95`로 고정했고 당시 main을 기반으로 만든 통합 후보 `95e674407aa820de14249a84bea160eba2728019`에서 한 번의 새 독립 검증을 실행했다. 중복 allowed_models를 조용히 정규화하는 schema/parser 불일치 한 건으로 FAIL이었다. 별도 P06 수정 `832f8a0ae5eeef3d2327bda60bedd7cc2015363e`를 통합한 `40f703cf331125abc4d3b78f431e6f7fc7253476`에서 해당 finding만 표적 확인해 PASS를 받았다. [검증 요약](docs/verify/WP-15-production-cloud.md)을 따른다. 전체 검토를 반복하지 않았다. [PR #5](https://github.com/peppone-choi/OpenHOI4/pull/5)는 main `6d1d1d9278c7771be9b9ba2943e745a9094b9104`로 정상 병합됐고 후보와 main tree가 동일하다. 같은 main CI 7개 모두 success로 WP-15 생산·비축 producer 단위의 통합 게이트를 충족했다. WP-16과 M2 전체 판정은 후속이다.
- WP-19 첫 standalone 계산 단위 `74bc16bb4278479d2742786a6e2513c251934317`는 한 번의 새 독립 검증에서 지정 범위 PASS다. [검증 요약](docs/verify/WP-19-standalone-cloud.md)을 따른다. 22개 소스 검사와 9개 독립 검사, 전후545개 추적 파일 동일·기존536개 파일 bytes 보존을 확인했다. 그 standalone 판정 당시 모듈은 등록되지 않았으며 실제 사단/이동 consumer·schema/save/wire/server/browser는 미구현이었다. 아래의 후속 등록·World/Map 단위와 판정 시점을 구분한다. [PR #4](https://github.com/peppone-choi/OpenHOI4/pull/4)는 main `c169a19a5fc3de5f5c3273d681a126a3921ba461`로 정상 병합됐고 후보와 main tree가 동일하다. 같은 main CI 7개 모두 success로 standalone 단위 통합 게이트를 충족했다. 현지화 checkout은 원래 attempt 1에서 성공했고 재실행하지 않았다. 공유 변경은 WP-15 인수 뒤 WP-19 순으로 진행한다.

- WP-19 후속 등록·World/Map 단위는 인수된 main `6d1d1d9278c7771be9b9ba2943e745a9094b9104`를 기준으로 구현 `ae5bfba50e7dab59c93a28fee620ec17cf68059c`를 통합 후보 `7cbb5572c0a11dfb279016ddafb95777bb53456b`에 반영했다. `oh_sim::supply` 등록과 순수 네트워크 어댑터만 추가하며 한 번의 새 독립 검증에서 지정 범위 PASS를 받았다([검증 요약](docs/verify/WP-19-world-adapter-cloud.md)). 전후 571개 추적 파일·HEAD·diff/status가 같았고 workspace Rust 306개·clippy·독립 방향 경로/실제 World 공식 검사가 통과했다. [PR #6](https://github.com/peppone-choi/OpenHOI4/pull/6)는 main `6191c0ab5e1cfa835da4e9de76e2aafe6863d042`로 정상 병합됐고 후보와 main tree가 같다. 같은 main CI 7개 모두 success로 지정한 World/Map 네트워크 입력 단위의 통합 게이트를 충족했다. 실제 통제국 `None`과 국가 ID0, 목적지 지형·현재 주 인프라에 따른 방향별 비용과 무방향 공유 철도 용량을 구분한다. 인프라 계수는 호출자가 제공한 양수 표에서 현재 level을 정확히 찾으며 누락을 default1이나 새 curve로 채우지 않는다. 유효 World의 육상 None producer는 없고 그래프 입력에서만 그 경계를 검사한다. hub/port instance·철도 건설 상태·실제 사단 수요·daily state·save/query/UI는 후속이다. [ADR](docs/adr/ADR-1902-world-network-adapter.md)을 따른다.

- WP-19 외부 설정·초기 World CLI 후속 단위는 main `6191c0ab5e1cfa835da4e9de76e2aafe6863d042` 기반 제품 `59174530fcc75c332460263b2b87cc971bbe846e`의14경로다. 한 번의 새 독립 검증에서 지정 범위 PASS를 받았다([검증 요약](docs/verify/WP-19-network-config-cloud.md)). workspace Rust332개·새 경계26개·CLI136 semantic cases·World 정수 공식108개·기존 schema/TS/팩/저장 보존과 native target12/restore4 aggregate PASS를 확인했다. 준비 오류와 표적 재확인 이력은 요약에 남기며 제품 finding/P06은 없었다. QA 전후581파일·HEAD/list/SHA/diff/status는 동일·clean이었다. Caller가 명시한 strict TOML sidecar를 실제 초기 World에 대조하며 config identity는 진단 출처다. 기본 M1의 없는 이동 계수나 현재 infra 계수를 default로 채우지 않는다. pack/scenario presence·실제 사단/daily/save/query/wire/UI와 전체 WP-19/M2는 후속이다. [PR #7](https://github.com/peppone-choi/OpenHOI4/pull/7)는 main `5ca938e5c49d37736fbb778df53a820d6321804c`로 정상 병합됐다. 후보와 main tree가 같고 같은 main CI 7개 모두 success로 지정한 외부 설정·초기 World CLI 단위를 인수했다.

## WP-16 로컬 초기 편제 단위

사용자는 family별 명시 모델, 필드별 합계/인력 가중 평균, 구성 최저 속도를 승인했다(01 §4.7). 최신 인수 main `5ca938e5c49d37736fbb778df53a820d6321804c`에서 별도 구현·독립 검증 worktree로 착수했다. Strict 외부 구성/편제 정의·실제 production 모델/family/국가 허용 참조·정상 편제 순수 계산·native CLI로 한정한다. 현재 생산 모델에 전투 능력치가 없어 구성에 명시한 정상 능력치를 집계하며 모델 전투 기여를 자동 파생했다고 기록하지 않는다.

제품 후보 `1f858ba2e174b08f8d27da8b26aaa37a2352b43e`의14경로는 한 번의 새 독립 검증에서 지정한 로컬 범위 PASS를 받았다([검증 요약](docs/verify/WP-16-normal-templates-local.md)). Finding/P06은 없었다. Rust345개·새 검사13개, 독립 계산335사례·실제 CLI1241프로세스, native target12/restore4와 추가 clean restore4 표적 확인, 기존 생성물/팩/저장·M1 결정론을 확인했다. 검증 전후592파일·HEAD/list/SHA/diff/status는 동일·clean이며 기존582파일 중4개 module/dispatch 등록 외578파일 bytes를 보존했다. 초기 준비 오류와 untracked 의존성 링크의 dirty 관측은 요약에 남겼다.

별도 게시 승인 확인이 거절되어 부모 세션이 사용자 답을 기다리는 동안 로컬 구현·검사만 허용된다. Push·PR 생성·main 병합을 수행하지 않는다. 로컬 독립 PASS는 main CI 통합 게이트나 WP-16/M2 전체 완료가 아니다. 훈련 예약/반환·보충·실제 사단/OOB·감편 수요/손실·daily·save/query/wire/UI는 보류한다.

아래는 이전 선별 인계의 상태와 재현 한계를 보존한 내용이다. 이전의 STOPPED 문구는 이번 사용자 재개 지시로 대체됐으며, 과거 미실행 검사를 이번에 실행한 것으로 바꾸지 않는다.

## 이전 선별 인계 상태

M0·M1 완료 기록과 고유 WP 16개 통합 기록을 기준으로 한다. M2는 미완료다. 서버 권위 시뮬레이션과 브라우저 클라이언트를 사용하며 기본 서버는 M1 최소 Testland를 로드한다. WP-14 경제·정치 소스와 검증용 독립 입력은 WIP로 포함했다. WP-23 완결 Testland 본작업은 착수하지 않았다.

WP-14의 네 번째 독립 검증은 담당 범위에서 PASS였다. 이후 P07 통합 검사는 사용자 중단: 최초 41개와 새 42·43번은 완전한 실행 기록, 44번은 process exit=0이나 후처리 미완료, 45~53번과 최종 의미 검사는 미실행이다. 이 부분 기록을 전체 P07 PASS로 확대하지 않는다. 같은 main의 CI와 WP-14 통합 판정은 당시 미완료였다. 이번 선별 인계의 정적 검사·PR CI와 원 P07은 별개다.

## 빌드와 실행

Rust 1.99.0(`rust-toolchain.toml`), Node.js 24.4 이상·npm 11.4.2를 사용한 작업이다. Node 최소 조건은 24, npm 최소 조건은 11이며 정확한 의존성은 Cargo.lock과 client/package-lock.json에 고정한다. Python 3.11 이상과 OS 빌드 도구도 필요하다. 저장소 루트에서:

```sh
npm --prefix client ci
npm --prefix client run build
cargo run -p oh_server --locked -- --open
```

서버 종료는 실행 터미널에서 Ctrl+C. 데이터 팩을 다른 위치에 두면 `--pack-root`를 지정한다. 화면 없이 M1을 실행하는 명령:

```sh
cargo run -p oh_cli --locked -- run --pack data/packs/testland --scenario m1 --ticks 1000 --seed 1 --hash-out
```

## 검사 명령과 한계

다음은 인수 후 재현할 명령이며, 이 문서에 있다는 이유로 이번 인계에서 실행됐다는 뜻은 아니다.

```sh
python tools/check_docs.py
python tools/check_assets.py --release
python tools/check_architecture.py
cargo fmt --check
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
npm --prefix client test
npx --prefix client playwright test
```

이 인계의 문서 검사(오류·경고0), 에셋 `--release` 검사, 아키텍처 정적 검사는 exit0이었다. 아키텍처 첫 시도는 PATH에 cargo가 없어 실행되지 않았고, 설치된 도구 경로를 지정한 오프라인 재검사에서 통과했다. 전체 `git diff --cached --check`는 원본 CRLF를 고정한 두 회귀 TOML의64줄을 trailing whitespace로 보고해 실제 exit2다. 입력 bytes/pack identity를 보존하므로 이를 정규화하지 않았다. 인계 문서·메타데이터만의 whitespace 검사는 exit0이다.

이전 선별 인계에서는 기존 P07·기능 테스트·게임 실행을 재시작하지 않았다. 이번 새 인수는 사용자 재개 지시에 따른 별도 검증이다. 과거 WP-14 검증의 restore 범위 및 MIN 연산 문제는 수정 소스에 반영됐지만, 이전 FAIL·검증 무효 기록을 PASS로 바꾸지 않았다. 첫 P07과 초기 이어가기의 출력 inventory/경로 처리 오류, 독립 검증의 npm 출력 경계 실패도 별도로 보존돼 있다. 독립 검증 일부 초기 기록은 나중에 재구성됐고 native aggregate 후처리 누락이 있어 완전한 실행 감사로 주장하지 않는다. Safari 실제품 등 추가 브라우저와 실제 대규모 게임 성능도 확인 대상이다.

## 다음 작업

1. 이 WIP와 01/02의 잠정·미결정 규칙, 실제 schema·저장 호환 경계를 확인한다.
2. WP-23 초기 합성 팩은 독립 PASS와 같은 main CI를 확보했다. WP-19 standalone 단위도 독립 PASS와 같은 main CI를 확보했다. WP-15는 한 번의 전체 QA와 기존 finding 표적 확인으로 독립 PASS를 확보했고 main `6d1d1d9278c7771be9b9ba2943e745a9094b9104`의 CI 7개 모두 success다. WP-19 등록·World/Map 단위는 별도의 새 검증과 통합 게이트로 판정한다. 과거 P07 기록을 이번 실행으로 바꾸지 않는다.
3. 승인된 생산·보급 정책과 위임된 provisional 수치는 각 ADR에 단위·반올림·민감도를 기록하고 WP-15 → WP-19 → WP-23 최종 콘텐츠의 공유 schema/save/protocol 통합 순서를 따른다. 미인수 사단/이동 계약이나 승인과 충돌하는 새 규칙은 별도로 보고한다. WP-16은 WP-15 뒤다. 새 수치·현지화·에셋은 기존 defines·ko/en·출처 매니페스트 규칙을 따른다.

## 공개 범위

제품 변경113경로를 기존 WIP에서 선별했다. 운반용 예외는 `tests/repro/WP-14-M2-r3-restore-stage/INPUT_MANIFEST.json`의 비실행 출처2필드와 같은 폴더의 README, `tests/repro/WP-14-M2-r3-target-range/README.md`의 설명3경로뿐이다. 내부 출처 경로와 실행 기록을 상대 fixture 경로·일반 재현 설명으로 정리했다. target-range manifest, save·팩22파일씩·CRLF·기대값·테스트 코드와 나머지 제품484경로의 Git blob·모드는 그대로다.

일반 지침·도구와 고정 의존성·라이선스·에셋 원천은 유지한다. 내부 세션·비교/조율 기록, 증거 ZIP·실행 바이너리·raw API·비밀정보는 새 tree에서 제외한다. 라이선스가 확인된 독립 지리 원천 ZIP, 지도 런타임 데이터와 자체 회귀 save fixture는 프로젝트 입력이며 내부 증거 압축파일과 구분한다. 배포 권한 없는 원작 게임 자료는 사용하지 않는다.

원본 로컬 작업·16개 미게시 commit·증거·worktree는 보존했다. 이 인계는 원본 이력의 일괄 push나 history rewrite가 아니다. 기존 공개 조상 commit에 포함됐던 증거는 이번 정상 삭제 commit으로 이력에서 없어지지 않으며 이력 정리는 범위 밖이다.
