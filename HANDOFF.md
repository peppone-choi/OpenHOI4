# OpenHOI4 WIP 인수인계

2026-10-08 사용자 재개 지시로 새 환경에서 WP-14 독립 인수·P07 검사를 시작했다. 이번 재개는 미완성 소스의 검증부터 진행하며 릴리스나 M2 완료 판정이 아니다. 반복 감시 자동화는 PAUSED로 유지한다. 실행 계획과 공유 변경 순서는 [M2 재개 계획](docs/plans/M2.md)을 따른다.

## 새 환경 재개 상태

- 시작 기준 main은 `743c2a941af5422b94e850b6064b19cffbebac31`이며 원격과 로컬 HEAD가 일치했다. 초기 추적 파일 변경은 없었다.
- 같은 기준 main의 7개 push workflow가 success인 것을 새로 확인했다. [CI](https://github.com/peppone-choi/OpenHOI4/actions/runs/37783866384), [Save determinism](https://github.com/peppone-choi/OpenHOI4/actions/runs/37783866339) 등이다. 이는 새 WP-14 독립 인수 결과나 과거 P07 전체 PASS를 대신하지 않는다.
- Rust 1.99.0·rustfmt·clippy, Node 24.19.0/npm 11.9.0, Python 3.12.14를 확인했다. 캐시와 빌드 산출물은 작업 환경의 쓰기 가능한 경로를 사용한다.
- 새 검증 담당은 별도 worktree에서 실행하며 추적 소스를 수정하지 않는다. 두 fixture manifest·save hash, 문서·release 에셋·fmt·clippy, Rust 250개 검사, 클라이언트 typecheck/build와 시스템 Chromium을 사용한 M0 브라우저 12개 검사는 통과했다. full client 검사는 최초 palette timeout을 남긴 뒤 조용한 환경에서 같은 timeout으로 246개 모두 통과했다. target2 유효 server control에서는 정상 경제 응답 뒤 WebSocket close 오류가 남았다. P07은 FAIL이며 P06 수정·새 독립 검증이 필요하다. 도구/CDN 차단과 테스트 실패를 구분하고 전체 PASS로 기록하지 않는다.
- 위 초기 FAIL 뒤 Close 응답을 flush하는 서버 수정과 전체 값 비교를 보존한 palette 검사 변경을 별도 구현 담당이 작성했다. 수정 소스 `4ae0446a01487e9d6d1415e7191a78407a8e0b6f`에 새 독립 검증이 지정한 WP-14 producer·P07 repair 범위에서 PASS를 판정했다. Rust 251개, 기본 client 246개, unchanged native target 12개·restore 4개, 저장/재개·legacy·별도 서버 lifecycle, 시스템 Chromium M0 12개·M1 3개가 통과했다. 검증 전후 HEAD·505개 추적 파일 SHA·diff/status는 동일했다. [공개 검증 요약](docs/verify/WP-14-P07-cloud-repair.md)을 따른다. 초기 FAIL은 이 후속 결과와 구분하여 보존한다.
- 수정과 재개 문서는 [PR #2](https://github.com/peppone-choi/OpenHOI4/pull/2)로 정상 병합됐다. main `0776e6b5a67951821860490964c402e518310ca5`의 CI 7개가 모두 success이며 검증한 제품 tree와 같다. 지정한 WP-14 producer·P07 repair 통합 게이트를 충족했다. 후속 production/훈련/항복 consumer와 M2 전체 완료는 아니다. Firefox/WebKit·추가 M1 지도 suites·대규모 성능은 이번 검증 범위가 아니다.
- WP-23 초기 합성 팩 구현 `b7c5b6898855a136456f7b558724f18cafa0656b`는 6개국·120개 육지 프로빈스·12개 주의 별도 `testland_m2` 입력이다. 기존 2개국·6프로빈스 M1·golden은 보존했다. [작업 로그](docs/worklog/WP-23-initial.md)와 [독립 검증](docs/verify/WP-23-initial-cloud.md)을 따른다. 한 번의 새 독립 검증은 지정한 초기 범위 PASS이며 전후535개 추적 파일 bytes·HEAD·diff/status가 동일했다. [PR #3](https://github.com/peppone-choi/OpenHOI4/pull/3)는 main `b112291c11822ffbf6da33b42670ff5cdb226fae`로 정상 병합됐고 같은 main의 CI 7개가 모두 success다. 지정한 초기 팩 통합 게이트를 충족했으며 WP-23 최종 플레이 입력·M2는 미완료다.
- 사용자는 생산 소수 이월·실제 생산 때만 효율 성장·다른 모델 전환 시 미완성량 폐기·부족 자원 비례 배분과, 보급 공유 철도 용량·거리 손실·첫 버전 고정 경로를 승인했다. 초기 수치는 조절 가능한 provisional 데이터로 선택·검증하도록 위임했다. WP-15 구현은 `c7b404a96c685068e6e79c79fecad4be3b42fd95`로 고정했고 당시 main을 기반으로 만든 통합 후보 `95e674407aa820de14249a84bea160eba2728019`에서 한 번의 새 독립 검증을 실행했다. 중복 allowed_models를 조용히 정규화하는 schema/parser 불일치 한 건으로 FAIL이었다. 별도 P06 수정 `832f8a0ae5eeef3d2327bda60bedd7cc2015363e`를 통합한 `40f703cf331125abc4d3b78f431e6f7fc7253476`에서 해당 finding만 표적 확인해 PASS를 받았다. [검증 요약](docs/verify/WP-15-production-cloud.md)을 따른다. 전체 검토를 반복하지 않으며 PR·같은 main CI는 후속 게이트다.
- WP-19 첫 standalone 계산 단위 `74bc16bb4278479d2742786a6e2513c251934317`는 한 번의 새 독립 검증에서 지정 범위 PASS다. [검증 요약](docs/verify/WP-19-standalone-cloud.md)을 따른다. 22개 소스 검사와 9개 독립 검사, 전후545개 추적 파일 동일·기존536개 파일 bytes 보존을 확인했다. 모듈은 아직 등록되지 않았으며 실제 사단/이동 consumer·schema/save/wire/server/browser는 미구현이다. [PR #4](https://github.com/peppone-choi/OpenHOI4/pull/4)는 main `c169a19a5fc3de5f5c3273d681a126a3921ba461`로 정상 병합됐고 후보와 main tree가 동일하다. 같은 main CI 7개 모두 success로 standalone 단위 통합 게이트를 충족했다. 현지화 checkout은 원래 attempt 1에서 성공했고 재실행하지 않았다. 공유 변경은 WP-15 인수 뒤 WP-19 순으로 진행한다.

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
2. WP-23 초기 합성 팩은 독립 PASS와 같은 main CI를 확보했다. WP-19 standalone 단위도 독립 PASS와 같은 main CI를 확보했다. WP-15는 한 번의 전체 QA와 기존 finding 표적 확인으로 독립 PASS를 확보했고 PR·같은 main CI를 진행하며 각 판정 범위를 유지한다. 과거 P07 기록을 이번 실행으로 바꾸지 않는다.
3. 승인된 생산·보급 정책과 위임된 provisional 수치는 각 ADR에 단위·반올림·민감도를 기록하고 WP-15 → WP-19 → WP-23 최종 콘텐츠의 공유 schema/save/protocol 통합 순서를 따른다. 미인수 사단/이동 계약이나 승인과 충돌하는 새 규칙은 별도로 보고한다. WP-16은 WP-15 뒤다. 새 수치·현지화·에셋은 기존 defines·ko/en·출처 매니페스트 규칙을 따른다.

## 공개 범위

제품 변경113경로를 기존 WIP에서 선별했다. 운반용 예외는 `tests/repro/WP-14-M2-r3-restore-stage/INPUT_MANIFEST.json`의 비실행 출처2필드와 같은 폴더의 README, `tests/repro/WP-14-M2-r3-target-range/README.md`의 설명3경로뿐이다. 내부 출처 경로와 실행 기록을 상대 fixture 경로·일반 재현 설명으로 정리했다. target-range manifest, save·팩22파일씩·CRLF·기대값·테스트 코드와 나머지 제품484경로의 Git blob·모드는 그대로다.

일반 지침·도구와 고정 의존성·라이선스·에셋 원천은 유지한다. 내부 세션·비교/조율 기록, 증거 ZIP·실행 바이너리·raw API·비밀정보는 새 tree에서 제외한다. 라이선스가 확인된 독립 지리 원천 ZIP, 지도 런타임 데이터와 자체 회귀 save fixture는 프로젝트 입력이며 내부 증거 압축파일과 구분한다. 배포 권한 없는 원작 게임 자료는 사용하지 않는다.

원본 로컬 작업·16개 미게시 commit·증거·worktree는 보존했다. 이 인계는 원본 이력의 일괄 push나 history rewrite가 아니다. 기존 공개 조상 commit에 포함됐던 증거는 이번 정상 삭제 commit으로 이력에서 없어지지 않으며 이력 정리는 범위 밖이다.
