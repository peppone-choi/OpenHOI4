# 검증 리포트 — WP-19 외부 설정·초기 World CLI

| 항목 | 값 |
|---|---|
| 판정 | PASS — strict 외부 설정·실제 초기 World 진단 범위 |
| 검증자 | 구현 비참여 새 독립 담당, 분리된 같은 commit worktree, 추적 파일 무변경 |
| 제품 커밋 | `59174530fcc75c332460263b2b87cc971bbe846e` |
| 기준 main | `6191c0ab5e1cfa835da4e9de76e2aafe6863d042` |
| 검증 일시 | 2026-10-08 |

## 재현과 범위

Rust 1.99.0과 고정 의존성을 사용한다. 외부 TOML과 실제 초기 World의 순수 네트워크 진단만 검증하며 Simulation을 step하지 않는다. 형식·단위·경계는 [ADR1903](../adr/ADR-1903-external-network-diagnostics.md)와 [README](../../crates/oh_sim/src/supply/README.md)를 따른다.

```sh
cargo test -p oh_data --test supply_network --locked
cargo test -p oh_sim --test supply_config_adapter --locked
cargo test -p oh_cli --test supply_network --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
python3 tools/check_docs.py
python3 tools/check_assets.py --release
python3 tools/check_architecture.py
```

명령은 `oh_cli supply-network inspect --pack <national-pack> --scenario <id> --nation <u16> --config <external.toml>`이며 네 옵션이 모두 필수다. 기본 M1에는 필요한 이동 계수가 없으므로 MissingContext를 반환한다. 기본 팩을 고치거나 default를 추가하지 않는다. 전용 CLI 검사는 독립 복사본에 synthetic 이동 계수를 명시한 실제 초기 World와 새 프로세스를 사용한다. `capital.toml`은 설정 형식 예제이며 모든 기존 팩에 그대로 적용된다는 의미가 아니다.

## 직접 확인한 결과

새 단위의 전체 검토는 한 번 수행했고 제품 finding은 없었다. P06 수정은 필요하지 않았다. 다음은 구현자의 실행 기록을 대신 사용하지 않고 독립 담당이 직접 확인한 결과다.

| 명령·검사 | exit·결과 |
|---|---|
| workspace Rust | 0, 332 PASS·60 suite, 새 검사26개 포함 |
| workspace all-targets clippy·fmt | 각0 |
| 문서·release assets·architecture·diff | 각0 |
| 실제 CLI semantic cases | 136개 최종 PASS, fresh process137회 |
| 별도 실제 source-path 하네스 | 0, 초기 World 정수 공식108개·오류 불변성6개 |
| 실제 CLI 방향별 비용 | 독립 정수 공식16방향, 공유 capacity 확인 |
| 기존 schema10개·protocol TS1개 재생성 | bytes 동일 |
| strict M1·M2 초기 팩 | 각0 |
| native target·restore | 최종 aggregate12/12·4/4 PASS |
| 기존 M1 365일 반복·2일 저장/새 process363일 재개 | 동일 hash·canonical bytes |
| QA 전후 HEAD/list/SHA/diff/status | 581개 동일·clean |

실제 CLI는 unknown/missing/numeric/count/file-size/UTF8/중복·정규화 alias/refs/unsupported/필수 argv 오류와 정상 대조를 함께 검사했다. 최초136개 중135개가 예상 결과와 일치했고 하네스의 empty-array 직렬화를 고친 한 항목만 재확인했다. 실제 국가0·수도 capture·초기 modifier1.125의 정확한 계수·방향별 비용을 확인했다. 별도 하네스는 실제 initial World108변형마다 기존 이동 연산 순서의 정수 공식을 비교하고 성공 시 canonical 불변성을 확인했다. 오류6개에서도 원본을 보존했다.

기존 frontend 입력81개 bytes를 확인하고 서버 build 선행조건인 ignored dist18파일을 hardlink로 재사용했다. native 정상 query의 두 실패에는 같은 package-lock의 기존 의존성을 잠시 연결했다. 모든 transient 링크를 제거했다. 새 npm 설치나 client/browser 실행으로 기록하지 않는다. 검증 제품 후보에서 기존567파일·지정 입력113파일과 기존 algorithm/World adapter 검사 bytes를 보존했다. QA가 추적 파일을 바꾸거나 커밋하지 않았다.

## 결정론

- 독립 실제 CLI 출력 SHA256 두 번 모두 `fd9413051d2c22e3e2b3db81f3c45ca4365e246446127448e51bf9189e52c086`이다. 입력 순서와 동등한 수치 표현을 바꿔도 진단 출력과 정규화 config identity `ba4b29425f509786`이 같다. 이들은 새 supply Simulation/save hash가 아니다.
- 기존 M1 seed7·365일 반복과2+363일 저장 재개의 최종 hash는 `3d83f30e5091dfa6`, canonical SHA256은 `c4556d9638f6a87abf5af1d20510f3a98323fe4d09569f5383a14967d9cf3dd2`로 일치했다. 네트워크 상태가 없는 기존 save roundtrip도 유지했다.
- 제품 QA commit은 위591이며 추적581파일 manifest SHA256은 전후 `afbd82f60c2ed031aafb99ec36aec63bf1ee1c131ea3df37a2b84e80e482deb0`로 동일하다. 제품 tree는 `7d14d4678a17deadb9d31a15f211b34fdc604a4f`다.

## 권위와 남은 범위

설정은 caller가 선택한 static sidecar다. pack/scenario 등록·content hash·Simulation presence를 바꾸지 않는다. 정규화 config identity는 진단 출처이며 state hash나 supply save 결정론 증명이 아니다. ID/placement/무방향 rail/정규화 level 중복은 정규화 전에 거부한다. 실제 nation·수도·지도 refs와 통제를 검증하며 국가0과 None을 구분한다.

실제 초기 modifier를 적용한 모든 state level의 양수 계수가 필수이며 고립/빈 네트워크에도 default1·curve를 만들지 않는다. 방향별 비용과 무방향 공유 용량을 구분한다. empty source/rail은 명시 진단 입력이고 fake demand나 Day를 만들지 않는다. Hub/Port 참조가 유효해도 building instance 권위가 없어 unsupported다. rail은 건설·손상 상태가 아니다.

실제 WP-16 사단 demand/strength/org/fire, daily 효과·phase·원자성·canonical/save/query/wire/served UI와 전체 WP-19/M2는 후속이다. 기존 packs/goldens/repro/assets/locks·schema10개와TS1개·None/V1–V6 저장과 기존 Simulation 상태는 이번 단위의 변경 대상이 아니다.

## 실패 기록과 통합

구현자의 requirement-first 실패와 준비 오류는 [작업 로그](../worklog/WP-19-network-config.md)에 보존했으며 독립 판정을 대신하지 않는다. 독립 실행에도 다음 준비/비교 오류가 있었다.

- empty-array 하네스 직렬화가 필드를 생략했다. 원래 결과를 보존하고 정확한 root `nations=[]` 입력에서 MissingContext를 확인했다.
- native target 최초 실행은12개 중10개 PASS였다. 정상 query2개는 Node `@msgpack/msgpack` 경로 누락으로 실패했으며 서버HTTP200/exit0을 받았다. 기존 동일 의존성을 연결한 뒤 **두 정상 대조만** 재실행해 PASS를 받았다. 전체 검토나 전체 helper를 반복하지 않았다. restore4개는 최초 exit0이었다.
- `/tmp`와 workspace 사이 hardlink 시도는 EXDEV로 helper 실행 전에 실패했다. 검증자가 소유한 임시 symlink로 준비하고 실행 뒤 제거했다.
- 이번 범위에 필요하지 않은 고립 legacy M0 입력의 active validate는 기존 ko/en `testland_name` 누락으로 exit1이었다. 해당 파일·validator는 변경되지 않았으며 기본 M0 Rust CLI 검사는 통과했다. strict M1·M2는 별도로 통과했다.
- 초기 frozen 경로 선택은106개였다. 기존 공개 inventory의 전체 asset 경로를 포함해113개를 자체 기준 SHA와 비교했다. 검사 수를 소급해 최초106개가113개였다고 기록하지 않는다.

제품 finding은 없으며 준비 실패를 제품 수정으로 숨기지 않는다. 모든 시도가 exit0이었다고 주장하지 않는다. 공개 문서는 재현 명령과 사실 요약만 포함하고 내부 하네스·명령 원장·raw 출력은 Git에 넣지 않는다. 한 번의 새 독립 QA는 지정 범위 PASS이며 PR·같은 main CI는 문서 작성 시 후속 게이트다. 전체 WP-19/M2 인수는 아니다.
