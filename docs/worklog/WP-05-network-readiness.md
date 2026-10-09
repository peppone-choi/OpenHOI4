# WP-05 작업 로그 — Create 검사의 준비와 관찰 창

작성: 최병호 · 2026-10-09

| 항목 | 값 |
|---|---|
| 상태 | 독립 검증 대기 |
| 브랜치 | `test/network-readiness` |
| 기준 | main `ef6d1fa1fcd9b7cd6350649b25efd11995d2e7ad` |
| 범위 | 기존 REQ-NET-01 Create 브라우저 검사의 준비 경계·종료 정리 |

## 관측과 분리

생산 데이터 팩 후보의 push CI client job은 기존 Create 검사에서 Delta count0으로 실패했다. 앞의 not-joined/unsupported/invalid-message/Snapshot/지원 time query 검사는 통과했고 다른35개 M0 브라우저 검사는 통과했다. 같은 후보의 뒤 PR client job은 M0 36개·M1 82개·native Firefox1개·Xvfb Firefox41개를 통과했다. 뒤 성공으로 앞 실패를 지우거나 무조건적인 flake로 판정하지 않는다.

이 검사의 소스와 `oh_server` gateway/session, M0 time/network defines는 기준 main과 데이터 팩 후보에서 byte-identical이다. 해당 서버는 기존 M0 pack만 로드한다. 원 검사는 WebSocket 생성 직후750ms 타이머를 시작하며, onopen·Hello·Create를 처리한 뒤에야 simulation이 시작된다. M0 speed1의 tick 간격은500ms이고 session이 첫 step 후 Delta를 발행한다. 준비 지연이 관찰 창을 소비할 수 있다.

## 실패 → 통과 증거

실제 Chromium151.0.7922.173, 기존 M0 팩, 바이트가 같은 Rust 권위 코드와 기준 main Vite HTTP 자산으로 loopback에서 분리했다. 실행 파일의 UI bundle은 별도 표시 후보의 것이어서, 진단 프록시가 기준 main의 실제 빌드 자산을 공급했다. Rust 재빌드나 제품 코드/데이터/프로토콜/설정 변경은 하지 않았다. 지연은 이 private 프록시의 WebSocket upgrade 준비 단계에만 주입했다.

| 조건 | 준비·첫 Snapshot·첫 Delta (생성 후 ms) | 결과 |
|---|---|---|
| 원 타이머·지연0 | open5.8 / Snapshot97.0 / Delta508.3 | Delta1, 나머지 원 assertion 통과 |
| 원 타이머·준비 지연350ms | open355.6 / Snapshot400.1 / 750ms 종료까지 Delta 없음 | 원 assertion 중 Delta>0만 실패하는 증상 재현 |
| Snapshot 감지 뒤 최소750ms·준비 지연350ms | open355.9 / Snapshot401.0 / Delta867.4 | Delta1, 모든 원 assertion 통과 |

원 CI에는 socket open/Snapshot의 개별 수신 시각이 없다. 위 실험은 준비 지연으로 같은 증상이 생기는 것을 증명하며, 원 CI 실행의 실제 지연 원인이나 정확한 actor/browser 시각을 확정하지 않는다. 제품의 Delta 상실/정지 문제는 이 분리 실험에서 관측되지 않았다. CI의 Chromium153 실제품과 로컬 Chromium151을 같은 환경이라고 기록하지 않는다.

## 변경과 검사

Create 검사에서 frame을 수신 시점부터 보존하고, bounded Playwright poll로 Snapshot 준비를 확인한 뒤750ms wait를 실행하므로 감지 후 최소750ms를 관찰한다. Poll 감지와 browser/host scheduling을 포함한 전체 경과 시간은 원 검사와 동일750ms라고 주장하지 않는다. readiness가 실패하면 테스트도 실패한다. 모든 기존 not-joined/unsupported/invalid/Snapshot/time query assertion과 Delta count>0, count<=8, 연속 sequence1..N을 그대로 유지했다. 관찰이 끝나면 소켓을 닫고 종료 콜백까지 bounded poll로 확인한다.

변경된 실제 검사 본문은 준비 지연350ms에서3회·지연0에서3회 통과했다. 같은 baseline HTTP/실제 Rust 서버를 쓰는 network.spec.ts 전체6개 Chromium 검사도 통과했다. TypeScript typecheck·문서 검사·diff whitespace는 exit0이다. source/assertion 대조는8개 기존 expect와 감지 후 최소750ms 관찰(설정 wait750ms) 보존을 확인했다. 이 수정은 데이터 팩/경제 UI와 별도이며 Rust·생성물·제품 network 코드·CI 설정을 바꾸지 않는다. private proxy·raw job 로그·실행 결과는 Git에 넣지 않는다.

공개 재현은 기존 `npm --prefix client run build`, `cargo build -p oh_server --locked` 뒤 client에서 `npx playwright test e2e/network.spec.ts`다. 설치된 실제 browser를 사용한다. 독립 QA/main CI 인수 전이며 전체 네트워크·성능·M2 완료 판정이 아니다.

실제 독립 코드 리뷰는 b6b5fe8 후보 source의 required0을 확인했다. 동일51개 CI 성공 뒤 PR #12는 main 0fe68a968982aeaa07809fd1769b9a00b62f9f5b로 정상 병합됐다. 위 최소 관찰 시간 표현은 후속 리뷰의 문서 교정이며 테스트 본문·assertion을 바꾸지 않는다. 해당 main의 CI 인수는 별도로 확인한다.
