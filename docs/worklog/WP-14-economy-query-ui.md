# WP-14 작업 로그 — 경제 조회 UI

작성: 최병호 · 2026-10-09

| 항목 | 값 |
|---|---|
| 상태 | 독립 검증 대기 |
| 담당 | 구현 |
| 브랜치 | `wp/o3-economy-panel` |
| 기준 | main `ef6d1fa1fcd9b7cd6350649b25efd11995d2e7ad` |
| 범위 | 기존 EconomyView의 읽기 전용 UI·응답 수명·실제 HTTP/WS/브라우저 인수 |

## 구현

`EconomyPanel`은 국가별 산업·정치·인력·법령·배분과 원장·프로젝트·대기 명령·산업 점수를 서버 값 그대로 표시한다. 배열 배분은 소비재0/건설1/군수2/수출3이며 wire 정수와 고정 소수는 Number/parseFloat로 변환하지 않는다. Exact raw bits는 tooltip에 보존하고 표는 가로 스크롤을 허용한다. ko/en을 함께 추가했다.

`EconomyResponses`는 현재 socket·선택 국가 수명에서 가장 최근 실제 발행한 `economy:<n>`만 받는다. 발행되지 않은 미래/이전/중복 응답은 무시한다. 국가/연결 전환에서 view를 먼저 비우고, disconnect/기존 validator의 malformed 거부에서는 마지막 검증 view를 stale로 남긴다. 재연결은 새 view를 기다린다. unsupported와 빈 배열/null score를 명시적으로 표시한다. 조회 국가 선택으로 게임 권한을 바꾸지 않는다.

Rust 제품 코드·프로토콜/생성물·validator·save·기존 팩은 수정하지 않았다. 법령/배분/건설 명령이나 게임 계산을 추가하지 않았다. [ADR-1401](../adr/ADR-1401-economy-query-display.md)을 따른다.

## 테스트 우선·초기 실패

전용 테스트를 먼저 작성하고 실행했으며 아직 없는 EconomyPanel import에서 exit1이었다. 이 시점에는 테스트 본문이 실행되지 않았으므로5개 semantic assertion의 실패로 세지 않는다. 구현 후 전용5개와 기존 App SSR1개를 함께 통과했다.

최종 호환성 확인에서 기존 경제 없는 M1은 generic QueryResult를 사용함을 확인했다. 전용 검사를 먼저 추가해5 PASS/1 FAIL을 관측하고, 최신 economy 요청에 대한 unsupported generic 응답만 받도록 처리했다. supported time 응답·다른 요청은 여전히 버린다. 원본 M1 실제 서버 검사를 추가했으며 제품 Rust나 wire를 바꾸지 않았다.

첫 브라우저 실행은 정상 실제 M2 검사만 통과했고 지연 응답3개가 실패했다. Pause 중에도 실제 host가 Delta를 발행해 계속 새 economy 요청을 만드는 흐름이었다. 테스트 프록시에서만 명시적 시간 명령당 한 Delta를 전달해 지연 응답의 발행 창을 제어했다. 정상 검사에는 프록시가 없다. 그 뒤 한 전체 실행의 미래 응답 창 검사에서 loading 타이밍 실패가 남았고, 진단용 단독 실행은 통과했다. 최종 전체4개와 해당 검사 연속3회가 통과했다. 기대값/제품의 최신 요청 조건/validator를 완화하지 않았다. 초기 실패를 통과로 덮어쓰지 않는다.

## 실행한 검사

| 검사 | 실제 결과 |
|---|---|
| `npm --prefix client test` |14파일255 PASS; pretest의 실제 Rust wire fixture 생성은 Rust 빌드와 직렬 실행 |
| `npm --prefix client run build` | typecheck와 실제 Vite 번들 exit0; 기존 large chunk 안내 |
| `cargo build --offline --locked -p oh_server` | exit0; 위 소스 번들을 embed. 기존 ts-rs serde 안내2건 |
| `playwright.economy.config.ts` 실제 Chromium |5 PASS: M2 6국·HTTP bundle bytes·WS·ko/en; 역순/미발행 미래/국가 전환/unsupported; malformed·stale·reconnect; 큰 정수/빈 배열과 원본 M1 generic unsupported |
| 미래/역순/국가/unsupported 브라우저 검사 연속3회 |3 PASS, 한 worker |
| `cargo fmt --all --check` | exit0; Rust 소스 변경 없음 |
| `cargo clippy --workspace --offline --locked -- -D warnings` | exit0; Rust 소스 변경 없음 |
| docs/assets/architecture 검사 | exit0; 문서 오류/경고0·자산 오류0 |
| client localisation 검사 | exit0; 기존 dynamic key unused 경고44건. 새 경제 키는 ko/en에 모두 있으며 새 unused 경고 없음 |

브라우저 정상 검사는 기존 `testland_m2`를 새 임시 `<host>/testland`에 복사해 `m2_initial`로 실행한다. 원본 M1은 별도의 임시 host와 다음 loopback 포트에서 `m1`로 실행한다. O2의 미인수 생산 팩을 기다리거나 연결하지 않는다. 연결마다 독립 simulation인 기존 서버 계약을 사용한다. 공개 재현:

```sh
npm --prefix client ci
npm --prefix client test
npm --prefix client run build
cargo build --locked -p oh_server
cd client
npx playwright test --config playwright.economy.config.ts
```

서버 실행 파일을 별도 target에 빌드했다면 `OH_SERVER_EXECUTABLE`을 명시한다. 공식 배포 Chromium 실행 파일을 사용할 때는 `PLAYWRIGHT_CHROMIUM_EXECUTABLE`을 명시할 수 있다. 스크린샷과 일반 재현 요약은 기본 ignored `target/evidence/o3/economy` 또는 `OH_E2E_EVIDENCE`에 기록하며 Git에 넣지 않는다.

## 한계

구현자 검사이며 독립 QA·main CI 통합 판정은 아직 없다. 브라우저는 실제 Chromium만 실행했고 Firefox/WebKit 실제품을 검사했다고 기록하지 않는다. 지연/음성 프록시 인수와 정상 전체 WS 흐름을 구분한다. 재연결은 서버 저장 복원이나 공유 멀티플레이를 보장하지 않는다. 전체 경제/콘텐츠/M2 완료, 군사훈련 정책과 후속 게임 명령은 별도다.
