# OpenHOI4 WIP 인수인계

2026-10-08 기준. 이 반영은 미완성 소스의 인계이며 릴리스나 M2 완료 판정이 아니다. 개발 자동 진행은 STOPPED, 감시는 PAUSED다. 별도 사용자 재개 지시 없이 M2·P07·WP23 개발 루프를 시작하지 않는다.

## 현재 상태

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

기존 P07·기능 테스트·게임 실행은 재시작하지 않는다. 과거 WP-14 검증의 restore 범위 및 MIN 연산 문제는 수정 소스에 반영됐지만, 이전 FAIL·검증 무효 기록을 PASS로 바꾸지 않았다. 첫 P07과 초기 이어가기의 출력 inventory/경로 처리 오류, 독립 검증의 npm 출력 경계 실패도 별도로 보존돼 있다. 독립 검증 일부 초기 기록은 나중에 재구성됐고 native aggregate 후처리 누락이 있어 완전한 실행 감사로 주장하지 않는다. Safari 실제품 등 추가 브라우저와 실제 대규모 게임 성능도 확인 대상이다.

## 다음 작업

1. 이 WIP와 01/02의 잠정·미결정 규칙, 실제 schema·저장 호환 경계를 확인한다.
2. 개발 재개 승인이 있다면 P07의 완료·부분·미실행 범위를 구분해 계획하고 같은 main CI와 통합 판정을 확보한다. 이 인계만으로 WP-14 통합 완료나 M2 게이트 통과를 선언하지 않는다.
3. 그 뒤 WP-23 완결 Testland 및 후속 소비자를 계획한다. 새 수치·현지화·에셋은 기존 defines·ko/en·출처 매니페스트 규칙을 따른다.

## 공개 범위

제품 변경113경로를 기존 WIP에서 선별했다. 운반용 예외는 `tests/repro/WP-14-M2-r3-restore-stage/INPUT_MANIFEST.json`의 비실행 출처2필드와 같은 폴더의 README, `tests/repro/WP-14-M2-r3-target-range/README.md`의 설명3경로뿐이다. 내부 출처 경로와 실행 기록을 상대 fixture 경로·일반 재현 설명으로 정리했다. target-range manifest, save·팩22파일씩·CRLF·기대값·테스트 코드와 나머지 제품484경로의 Git blob·모드는 그대로다.

일반 지침·도구와 고정 의존성·라이선스·에셋 원천은 유지한다. 내부 세션·비교/조율 기록, 증거 ZIP·실행 바이너리·raw API·비밀정보는 새 tree에서 제외한다. 라이선스가 확인된 독립 지리 원천 ZIP, 지도 런타임 데이터와 자체 회귀 save fixture는 프로젝트 입력이며 내부 증거 압축파일과 구분한다. 배포 권한 없는 원작 게임 자료는 사용하지 않는다.

원본 로컬 작업·16개 미게시 commit·증거·worktree는 보존했다. 이 인계는 원본 이력의 일괄 push나 history rewrite가 아니다. 기존 공개 조상 commit에 포함됐던 증거는 이번 정상 삭제 commit으로 이력에서 없어지지 않으며 이력 정리는 범위 밖이다.
