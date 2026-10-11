# WP-16 작업 로그 — Ready 배치 UI

| 항목 | 값 |
|---|---|
| 상태 | 검증 대기 |
| 담당 | 구현 세션 |
| 브랜치 | wp/16-ready-deployment-ui |
| 대상 REQ | REQ-MIL-03, REQ-MIL-05 |
| 선행 WP | 인수된 PR18 main30e63654c06a8cf44ee3193a4900dd8dabd8045a |

## 계획

실제 독립 Claude 계획을 전달받은 뒤 기존 Deploy 명령만 연결했다.
전용 full-world 상관 검증, 소유한 Ready job·army·현재 통제 육지 선택,
기본 false의 understrength checkbox, 공용 successful-send 원장과 pending을
사용한다. 실제 서버/경제 원장만 표시하며 로컬 자원·훈련 계산은 없다.
소유·육지·정규 wire 검증은 송신 전, 군 정원·장비·최종 controller 검증은
기존 서버에 있다. [ADR-1604](../adr/ADR-1604-ready-deployment-world-authority.md)를 따른다.

## 테스트 우선 기록

| REQ | 테스트 이름 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| REQ-MIL-03 | canonical owned Ready references and controlled land | 기존 can이 Deploy를 Cancel처럼 취급해 world 없이 true, exit1 | 두 장벽·소유·정규 ID·water/foreign location 포함 통과 |
| REQ-MIL-03 | world authority validation/correlation | helper 없음, import 실패 exit1 | invalid membership·부분 응답·socket/epoch·dirty·누락/unsupported 통과 |
| REQ-MIL-03 | native Ready UI click | 기존 native 서버/실제 TimeControls Ready 후 Deploy 버튼 없음, exit1 | 실제 클릭·served JS 일치·경제/군사 이전·중복 전송1 통과 |
| REQ-MIL-05 | native full army rejection | 기존 CLI 정책을 변경하지 않음; 새 historical RED 주장하지 않음 | 실제 UI 정원 거절 및 기존 native CLI valid control 통과 |

## 실행한 검증

환경 guard는 자식 CPU affinity2/quota4, Cargo jobs1, browser worker1,
CPU/메모리90% 중단 기준을 적용했다. 아래 성공 실행은 guard 중단이 없고
최대 관측 메모리6.75GB, CPU59% 미만이다. 실행 산출물·원본 로그·주입 frame은
Git 밖 전용 scratch에 보존하고 공개 저장소에는 재현 절차만 넣었다.

| 명령/수용 묶음 | 종료 코드 | 결과 |
|---|---|---|
| `cargo run -p oh_proto --example wire_fixtures --locked` | 0 | 기존 Rust wire fixture 생성 |
| 새 출력의 `cargo test -p oh_save --test m2_military_pack --locked` | 0 | 2 passed, ignored0; 실제 V7 Training checkpoint |
| client cwd `node node_modules/vitest/vitest.mjs run` | 0 | 21 files, 288 passed |
| 최종 추가 foreign-province assertion의 militaryCommands 표적 재검사 | 0 | 6 passed; 전체 반복 없음 |
| `npm --prefix client run build` | 0 | typecheck·production build |
| `cargo rustc -p oh_server --bin oh_server --locked -- -C codegen-units=1` | 0 | 새 client bytes 포함 실제 native 서버 |
| 새 Ready config, native deploy/zero-equipment/full-army | 0 | 3 passed |
| 새 Ready config, world/ACK timing와 ko/en/mobile/disconnect | 0 | 3 passed; 주입 timing2와 실제 UI1 구분 |
| 기존 training-controls config | 0 | 10 passed; Train/Cancel/time/shared sequence/CONNECTING/legacy 포함 |
| `cargo test -p oh_cli --test military --locked` | 0 | 실제 native 프로세스13 passed, ignored0; NotReady/foreign army/controller/water/terminal/cap와 valid controls |
| `cargo fmt --check`, `python3 tools/check_docs.py`, `git diff --check` | 0 | 문서 식별자·추적 소스 검사 |

## 증거와 실패 경계

- 최초 wire fixture 준비는 guard 설정 전 수동 중단(exit130) 후 guarded 실행을
  완료했다. 첫 browser 시도는 긴 Unix socket 임시 경로로 시작 실패했다.
  짧은 전용 TMPDIR로 바로잡은 뒤 요구사항 UI RED를 따로 확보했다.
- 첫 focused GREEN 명령은 client fixture의 상대 경로에 맞지 않는 cwd였다.
  cwd를 고쳐 8개 표적 시험이 통과했으며 실패 출력을 보존했다.
- 첫 native2 검사에서 총 인력을 MilitaryView.background와 비교한 잘못된
  assertion, fresh 팩의 즉시 가용 인력 가정으로 실패했다. EconomyResult의
  실제 committed/reserved를 검사하고, 기존 generator 복원 상태에서 실제
  Pause line/Train/TimeControls로 held0 사례를 구성했다. 자원 assertions를
  제거하지 않았으며 잘못된 projection 선택을 수정했다.
- 한 재검사 준비 명령의 잘못된 cwd를 발견하여 해당 소유 실행을 수동
  중단(exit143)했다. 중단된 실행의 host2는 정상 exit0 증거가 없으나 잔존
  PID가 없다. 성공 수용3묶음의 host6은 모두 exit0·잔존PID0이며 기존 실패와
  중단을 성공으로 계산하지 않는다.
- semantic 배치 오류의 현재 reason은 `invalid-message`이다. 장비0 false
  실패는 held manpower·사단을 보존하고 true는 실제 배치한다. 군 정원
  실패는 background·division·job 보존을 대조한다.
- 물·외국 군·잘못된 controller·Ready 직전은 기존 native CLI에서 별도로
  검사했다. UI에서는 후보/권위로 차단하는 사례를 단위 시험으로 구분한다.
  native UI 수용을 직접 WebSocket send로 대체하지 않았다.

## ADR 및 호환 조정

Deploy만 military/world 이중 ACK 장벽을 요구한다. Train/Cancel은 위치가
필요하지 않으며 PR18의 군사 장벽을 보존한다. 모든 군사 ACK가 world를
철회·재조회한다. 최종 계획의 장벽 확장을 기존 기능 전체에 적용하여
world 미지원으로 기존 Train/Cancel이 갇히게 하지 않는 최소 호환 조정이다.
militaryResponses의 기존 serial·pending·dirty로 충분해 파일 자체는 고치지 않았다.

## 원작 대비 차별화 초안

기존 01 §11의 서버 권위 훈련·자원 계약을 브라우저 조작에 노출한다.
새 게임 규칙이나 원작 수치·파일을 사용하지 않았다.

## 결정 필요·범위 밖·남은 게이트

새 게임 설계 결정은 없다. server/sim/proto/save/schema/pack/의존성은 변경하지
않았다. SetPriority·이동·전투·보급·공유 세션 지속성은 후속 범위다.
실제 독립 source review와 독립 검증 PASS, 최종 head CI, 정상 main 통합 및
같은 main CI는 아직 별도 게이트다. 작성자 수용은 전체 WP-16/M2·전체 QA
완료 판정이 아니며 Rust 전체/clippy 반복은 이번 변경에서 실행하지 않았다.
재현은 [Ready 배치 절차](../../tests/repro/WP-16-ready-deployment/README.md)를 따른다.

## PR19 실제 독립 리뷰 P2 수정 후보 (2026-10-11)

원 head9f073f1의 실제 독립 source review는 필수 P2 하나로 HOLD였다.
군사 패널을 연 채 Join 변경/수동 재연결하면 App Welcome callback의 전용
world 조회가 network의 Join 송신보다 먼저 나갔다. native not-joined 이후
새 전용 응답이 없어 군사 view는 Ready여도 배치 선택창이 비활성이었다.
리뷰의 Train/Cancel 군사 장벽 호환 조정은 안전 판정이었다. 리뷰 담당의
runtime 환경에는 Cargo/Rust와 실제 wire fixtures가 없어 NOT_RUN이며
source review를 독립 runtime PASS로 확대하지 않는다.

실제 native server의 패널 열린 Join 변경과 수동 reconnect2개 UI RED를
확보했다. 각 wire는 Hello→deploy-world:2→Join과 not-joined를 기록했고,
TimeControls로 Ready에 도달한 뒤 군 선택창 disabled로 실패했다. filter
조작이나 raw command 송신으로 회복시키지 않았다.

제품 수정은 App의 Welcome 전용 조회를 제거하고 연결별 첫 Snapshot에서만
전용 조회를 보내는3행 delta다. network/wire/군사 ACK 장벽은 고치지 않았다.
추가 회귀는 native Join/reconnect 후 filter 조작 없이 선택·Deploy를 활성화하고
실제 Deployed까지 확인하며 첫 전용 query가 Join 이후인지 대조한다.

수정 뒤 client288·production build는 통과했다. 추가 시험의 findLastIndex가
기존 ES2022 타입 설정에 맞지 않아 build 실패를 남겼고 reduce로 바꿨다.
한 편집 명령의 cwd 오류와 그 뒤 실패 build도 보존했다. 첫 stale-bundle
no-op server build는 새 UI 검증으로 세지 않았고, client 성공 뒤 native server를
다시 빌드했다. 새 Ready8개와 범위 지정 Train/Cancel4개는 모두 통과했고 각 소유 host4개는
exit0·잔존PID0이다. 새 head CI·실제 delta 재리뷰·독립 실행 검증은 별도다. 원 head의 CI13개/51개 success는 원 source HOLD와
구분하며 수정 후보의 CI/재리뷰/독립 runtime 검증을 대신하지 않는다.

독립 검증자는 exact 새 head의 별도 clean worktree에서
`cargo run -p oh_proto --example wire_fixtures --locked`를 먼저 실행한다.
이는 target/wp09/national-wire-fixtures.json과 target/wp16/military-wire-fixture.json을
생성한다. 새 빈 OH_M2_MILITARY_OUTPUT_DIR에서 기존 oh_save m2_military_pack
2개 generator 시험을 실행하고 PackMismatch를 해제하지 않는다. 그 뒤
client 표적13개·build·exact-source native embedding·Ready8개·실제 legacy를
포함한 training-controls10개를 직접 실행하고 tracked identity 전후 동일성과
host exit/PID를 보고한다. 전체 Rust 재실행은 같은-head CI 증거가 있으면
하지 않는다. 작성자 실행을 독립 PASS로 기록하지 않는다.
