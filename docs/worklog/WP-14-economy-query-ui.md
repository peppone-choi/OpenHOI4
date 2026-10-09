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

## 실제 독립 리뷰 후 delta 수정

기존 후보 `07289034689b6053764c4ded04f42e6288f134c1`의 push/PR CI가 통과했지만 실제 독립 리뷰는 두 Medium 결함으로 막혔다. 같은 socket/국가에서 연속 Delta가 경제 대기 요청 ID를 덮어써 느린 응답을 영구 거절할 수 있었고, 재연결은 이전 simulation의 production view를 보존해 새 연결에서 오래된 행에 명령을 보낼 수 있었다. 이전 CI 성공을 이 결함의 해결이나 인수로 기록하지 않는다.

조회는 대기 ID를 유지하고 추가 갱신을 dirty 비트로 모아, 유효 응답을 표시한 뒤 후속 조회 하나를 발행하도록 수정했다. 국가/연결 수명 변경은 여전히 즉시 무효화한다. Production은 재연결/국가 제어 전환에서 view와 검증 socket을 비우며, 현재 socket의 알려진 응답 이후에만 현재 연결·제어국을 대조해 명령을 보낸다. 이 수정은 App/응답 관리와 검사·문서뿐이며 Rust/protocol/validator/save/팩/생성물은 그대로다.

새 단위 검사를 먼저 실행해 기존 코드의 두 번째 발행 ID가 null이 아닌 economy:2로 나와 실패했다(exit1). 기존 번들을 실제 Rust 서버에 embed해 새 브라우저 검사를 실행했다. 실제 Delta4개 이상을 그대로 전달하는 동안 요청이1개 대신7개여서 실패했고, 생산 회귀는 재연결 후 패널이0개 대신1개여서 실패했다. 초기 production 회귀 준비에서는 unsupported가 generic QueryResult임을 놓쳐 패널이 뜨지 않았으며, 기존 실제 Rust projection을 이 응답 경로에도 공급해 준비 오류와 제품 실패를 분리했다.

수정 후 전체 client는14파일256 PASS(exit0)이며 실제 Rust pretest fixture 생성도 다시 실행해 exit0이며 production/economy fixture bytes가 이전과 일치했다. 처음 잘못된 cwd의 Vitest 실행은 relative fixture/assets ENOENT16건이었다. 올바른 client cwd에서 기존 검사를 바꾸지 않고256개가 통과했다. Typecheck/Vite build와 새 번들 native embed는 exit0이다. 기존 지연 검사3개는 같은 수명에서 Delta가 새 ID를 만들던 준비 방식 대신, 실제 국가 수명을 전환해 이전/현재 요청을 구분하도록 수정했으며 미래/이전/중복/unsupported/문자열·stale 검증을 유지했다.

실제 Chromium151의 전체7개 브라우저 검사에서 두 회귀와 기존5개가 모두 통과했다. 첫 전체 실행의7 PASS 요약에도 결합 shell 결과가1로 남아 따로 보존했고, 후속 전체 실행은 실제 Playwright 종료 코드를 명시적으로 기록해7 PASS/exit0을 확인했다. 연속 Delta 회귀는 반복 publication을 필터링하지 않는다. 재연결 생산 회귀는 기존 실제 Rust projection을 공급하는 UI 수명 검사이며 M2 pack production 게임 인수가 아니다. 새 실제 독립 delta 리뷰·최종 SHA CI/main 인수는 대기다.

## CONNECTING 국가 선택 후속 리뷰 수정

실제 독립 리뷰는 37c13b8의 두 기존 blocker 해결을 확인했지만 CONNECTING 중 국가 선택이 실제 전송되지 않은 요청을 pending으로 기록하는 Low 결함을 발견했다. 전송 성공 뒤에만 pending을 접수하고 Snapshot에서 최신 국가 범위를 발행하도록 수정했다. network send가 실제 전송 여부 boolean을 반환하며 응답 관리자 callback이 그 결과를 사용한다. 기존 protocol/validator/Rust/팩/소유 규칙은 그대로다.

새 단위 검사는 기존 코드에서 economy:1이 null이 아니라서 실패했다(exit1). 실제 이전 번들과 Rust host에 HTTP upgrade gate를 적용한 브라우저 검사는 CONNECTING 중 국가를2→4로 선택한 뒤 OPEN에서도 loading으로 남아 실패했다. WS 프레임·서버 응답을 대체하지 않았으며 initial native 연결과 해당 국가 버튼을 실제로 사용했다. 수정 후 client257개·typecheck/Vite/native build가 통과했고 실제 Chromium 전체8개가 PASS, 명시 기록한 Playwright exit0이다. 같은 socket의 coalescing·과거/미발행 응답 거절, 경제 없는 M1, 기존 reconnect 생산 UI 회귀를 보존했다.

최종 delta는 실제 독립 리뷰 대기다. Native 생산 명령 접수는 대체 production 응답 회귀로 증명하지 않으며 O2 입력을 포함한 통합 서버·브라우저 인수에서 실제 accepted/거절과 자원 상태를 검사한다. main 통합·전체 M2 완료는 아직 주장하지 않는다.

## O1·준비 경계 main 통합 재검사

PR #12와 PR #9가 병합된 main `9721b19ea5ff495b06ff3680eea85a3890e97387`을 정상 병합했다. 유일한 HANDOFF 충돌의 양쪽 내용을 보존했고 network 자동 병합에서 기존 O1 military validator와 O3 send 성공 판정을 함께 확인했다. O3 App·응답 수명·CONNECTING 브라우저/upgrade gate 검사는 `6b75814c0ec57374e4f65df4eb3b63690aa8bc3b`와 동일하다. 제품 Rust·팩·schema·wire/저장·의존성·CI 설정은 main과 같으며 O2 팩은 별도다.

새 main baseline에서 실제 Rust fixture를 생성한 client261개(15파일), typecheck/Vite·현재 번들 native embed·docs/assets/architecture가 통과했다. 공유 package/build-script 캐시는 다른 checkout의 번들을 재사용하지 않도록 먼저 정리했다. 실제 Chromium151 경제 suite8개와 M0전체12개, 별도 O2 생산 팩 bytes를 사용한 실제 native 생산 UI 검사1개가 모두 exit0이었다. 생산 조합 검사는 프레임/응답 대체 없이 국가2 제어→Create접수·실제 생산 응답의 국가/모델/IC값·경제 국가4조회가 제어국을 바꾸지 않음·정상 소켓 종료와 새 세션 Create접수를 확인했다. 각 소켓은 여전히 별도 simulation이다. 최초 조합 준비의 연결 중 Reconnect 버튼 대기 timeout은 보존하며 실제 종료 경로 재검사와 구분한다.

CONNECTING 수정의 실제 독립 Claude delta 리뷰와 새 exact-head CI는 대기이며 O2/O3 main 병합·최종 V7/V8/quota/rollback 및 전체 결합 M2 인수는 남아 있다. 이전 대체 production lifecycle 검사가 실제 생산 명령 접수를 증명한다고 바꾸지 않는다.
