# WP-22 작업 로그 — 읽기 전용 군사 조회 패널

| 항목 | 값 |
|---|---|
| 상태 | 독립 검증·리뷰 대기 |
| 담당 | 구현 세션 |
| 브랜치 | wp/22-military-query-panel |
| 대상 REQ | REQ-UI-01/03의 기본 군사 정보 표시 부분, REQ-NET-03의 군사 상세 수명, REQ-LOC-01 |
| 선행 WP | WP-08, WP-12, WP-14, 인수된 WP-16 producer |

## 계획

기준 main `ffc166d5fc8c0fa4216119170bc076dea63c44cc`에서 부모가 전달한 실제
Claude Opus 5.5/medium 계획을 적용했다. 읽기 전용 화면과 열린 패널의 요청
수명만 구현하고 게임 계산·명령 UI·편제 편집·Rust/server/proto·기존 guard는
수정하지 않는다. 독립 기능 QA·Claude 리뷰·exact-head CI·main 통합을 구현자의
검사와 구분한다. [ADR-2201](../adr/ADR-2201-military-query-display.md)을 따른다.

## 테스트 우선 기록

| REQ | 테스트 이름 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| REQ-UI-01/03, REQ-LOC-01 | militaryPanel 및 실제 키보드/모바일 표시 | 새 module 부재로 두 suite 실패. 기존 실제 main 번들에 군사 열기 버튼이 없어 브라우저 실패 | 새 단위·실제 브라우저 검사 결과는 아래에 기록 |
| REQ-NET-03 부분 | militaryResponses 및 실제 지연·재접속·CONNECTING | 새 module 부재로 suite 실패 | 현재 socket/open epoch, pending1/dirty/followUp, fallback 경계 검사 |

## 실행한 검증

구현자의 표적 신규 unit 8개와 기존 client를 포함한 전체 269개/17파일이
exit0으로 통과했다. 실제 현재 Rust wire fixture를 `cargo run -p oh_proto
--example wire_fixtures --locked --offline`로 생성한 뒤 검사했으며 복사한
이전 baseline 4개를 새 fixture 생성으로 기록하지 않는다. typecheck/build는
통과했다. 새 browser helper의 disconnect 함수 반환형 추론 오류는 명시한
void 타입으로 수정한 뒤 typecheck exit0이었다.

실제 `cargo run -p oh_save --example military_fixture --locked --offline --
target/wp22/military-ready`로 자체 합성 m1/Ready V7 입력을 생성했다. 새 client
빌드 뒤 oh_server package cache를 정리하고 현재 worktree에서 서버를 빌드했다.
첫 실제 군사 browser 실행은 7개 통과·native 명령 검사1개 실패였다.
관전자 session의 raw Train은 `unsupported-session`으로 거절돼 기존 국가
조작/Join 절차를 사용했다. 후속 실행의 Deploy는 존재하지 않는 province1로
거절됐으며 기존 인수 native 검사·실제 fixture의 유효 province10을 사용했다.
서버 허용 규칙이나 성공 기대를 낮추지 않았고 관전자 거절 대조도 유지했다.
해당 native 표적은 exit0으로 Ready/Train Pending/Cancel/Deploy/Division 표시를
확인했다. 테스트 harness가 실제 browser session에 명령을 주입한 것이며
제품 UI가 MilitaryCommand를 전송한 것으로 기록하지 않는다.

첫 실행의 7개는 실제 키보드·ko/en·390×844·닫힌 조회0·local filter,
연속 native Delta 중 지연 응답·닫힘/열림 epoch, 실제 HTTP upgrade gate,
disconnect/reconnect, 군사 없는 실제 M2의 generic unsupported,
조작 국가 identity 초기화, 기존 guard의 malformed/stale를 검사했다.
지연/미래/malformed 응답 주입의 수명 검사와 대체되지 않은 native 명령/조회
검사를 구분한다. 기존 기본 browser 출력 경로는 후속 suite가 덮어쓸 수 있어
군사 suite의 전용 `target/wp22/browser-military` 출력으로 분리했다.

최종 군사 Chromium suite는8개/18.8초·exit0이었다. 대체되지 않은 native
Ready/Train/Cancel/Deploy와 현재 served JS bytes를 확인했다. 관전자 명령은
unsupported-session 거절 대조로 남았다. ko/en·keyboard focus 반환·mobile,
닫힌 조회0·UI MilitaryCommand0, 연속 Delta/epoch/CONNECTING/reconnect/
identity/malformed와 실제 군사 없는 M2 fallback이 통과했다. 전용 worker의
서버2개는 모두 exit0·잔존 PID0이며 finally 종료를 확인했다.

기존 경제 Chromium8개/24.2초는 원래 그대로 실행해 exit0이었다. 기존
production reconnect/명령 접수 차단 수명 검사도 포함한다. 대체 응답의
production 검사를 실제 native 생산 명령 인수로 확대하지 않는다. 기존 M0
Chromium12개/12.5초도 exit0이었다. 총 browser28개이며 실제 로컬 Chromium
제품 하나의 결과다. Firefox/WebKit·다른 OS를 실행했다고 기록하지 않는다.
docs/assets release/architecture/whitespace의 초기 정적 검사는 오류0이었다.
추가 문서를 포함한 최종 docs/assets release/whitespace 검사도 오류0이었다.
fmt와 workspace all-target clippy `-D warnings`는 exit0이며 기존 M1의
1,000tick/seed1 두 번의 hash는 `60448355cecffa9d`로 일치했다. 기존 Rust/data/
schema/save/proto/생성 TS/network/validator/경제 응답/CI 경로는 기준 main과
bytes가 같다. 전체 Rust runtime 검사를 새로 실행한 것으로 확대하지 않는다.
후보 CI와 독립 판정은 별도다.

## 재현

```sh
npm --prefix client ci
npm --prefix client test
npm --prefix client run build
cargo run -p oh_save --example military_fixture -- target/wp22/military-ready
cargo build -p oh_server
cd client
npx playwright install chromium
npx playwright test --config playwright.military.config.ts
```

별도 Cargo target/브라우저를 쓰면 OH_SERVER_EXECUTABLE과
PLAYWRIGHT_CHROMIUM_EXECUTABLE을 명시한다. 다른 fixture 출력은
OH_MILITARY_FIXTURE_ROOT로 지정한다. fixture의 expected.json이 가리키는
생성 팩과 ready.ohsave를 보존한다. worker가 task-owned PID ledger를 남기고
finally에서 SIGINT/종료 대기를 수행하며 모든 child의 exit0·잔존 PID0을
검사한다. 기존 사용자 프로세스를 종료하지 않는다.

## 증거

공개 소스의 unit/e2e/전용 config와 위 재현 순서를 따른다. 실패 원문·브라우저
스크린샷·native response·PID ledger·실행 바이너리는 private 출력에 보존하고
Git에 넣지 않는다. 실제 served JS와 이 worktree의 dist bytes를 native 검사에서
대조한다. 공유 cache가 현재 client identity를 대신하는 것으로 가정하지 않는다.

## 원작 대비 차별화 초안

원작 코드·자료·수치 비교 없이 자체 합성 입력과 권위 조회만 표시한다.
정상 능력치와 실제 소유 자원을 구분하며 감편 전투/보급 효과를 파생하지 않는다.

## 결정 필요·범위 밖 발견·못 한 부분

MilitaryView에 LedgerView가 없어 REQ-UI-04의 실제 원장 tooltip은
server/proto 소유자의 후속 계약이 필요하다. 이를 새 원장으로 합성하지 않는다.
명령/편집 UI·전체 메뉴 흐름·전투/외교/보급과 전체 WP-16/WP-22/AC-M2-01/M2
완료 판정은 이번 범위 밖이다. 독립 기능 QA·Claude 리뷰·후보 CI·main 통합은
미실행이다. 프로젝트 SKILL.md는 없어 AGENTS.md 일반 지침을 적용했으며
설치/검색된 외부 skill을 사용했다고 기록하지 않는다.
