# ADR-0502 빌드된 클라이언트 임베드와 로컬 실행

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-05, REQ-NET-02, REQ-NET-06, REQ-PLAT-01 |

## 맥락
M0는 지도 전 단계의 웹 셸을 실제 서버 실행 파일에서 제공해야 한다. 02는 정적 파일 제공을 정했으나 임베드 방식과 fresh checkout 빌드 순서를 정하지 않았다. Node/Vite 개발 서버를 사용자에게 따로 실행시키지 않아야 한다. 기본 브라우저와 종료 신호의 OS 차이도 처리해야 한다.

## 결정
- `oh_server/build.rs`가 `client/dist`의 실제 Vite 빌드를 재귀 수집해 include_bytes로 실행 파일에 포함한다. index.html·assets 디렉터리·JS 번들이 없으면 빌드를 실패시키고 준비 명령을 출력한다. 빈 페이지나 별도 개발 서버로 숨기지 않는다. `/`·JS·CSS·현지화 JSON을 Content-Type과 함께 제공하고 없는 경로는 404다.
- 새 checkout의 순서는 `npm --prefix client ci` → `npm --prefix client run build` → `cargo build --workspace --locked`다. fmt/clippy/test도 서버 build.rs를 실행할 수 있으므로 npm 빌드를 먼저 한다. `ci.yml`의 서버 세 OS와 client E2E 모두 이 순서를 가진다. WP-04 sim-determinism.yml과 core-determinism.yml은 수정하지 않는다.
- 현지화 ko/en JSON은 브라우저 초기 연결 표시용으로 번들에 포함하고 같은 파일을 HTTP로도 제공한다. 키 기반 최소 M0 UI만 제공하며 Fluent·자체 CJK 폰트·전체 테마는 WP-12의 소유다. 원본 JSON 두 파일을 ASSETS.toml에 자체 제작으로 등록한다. 프로젝트 라이선스 D-10 상태를 바꾸지 않는다.
- 기본 주소는 `http://127.0.0.1:8080/`이고 `--port`로 변경한다. 원격 바인드 옵션은 없다. 기본 데이터팩 루트는 저장소 상대 `data/packs/examples/m0`이며 배포 시 `--pack-root <명시적 경로>`를 사용할 수 있다. manifest ID m0_testland/scenario ID testland만 M0에서 로드한다. 실행 파일은 client/dist나 Node 없이도 제공하지만 데이터팩 입력 세 파일은 이 허용 경로에 필요하다. M1 그래픽·지도 제공 전체를 구현했다고 주장하지 않는다.
- --help, 기본 접속 주소·팩·사용법·Ctrl+C 안내, 잘못된 옵션/중복/범위 밖 포트/누락 인수, 바인드 충돌과 데이터팩 오류를 명확히 출력한다. 시작 실패는 비정상 종료한다.
- --open은 Windows rundll32 URL handler, macOS open, Linux xdg-open에 URL을 별도 인수로 전달한다. 브라우저 launcher가 종료를 기다려도 HTTP나 Ctrl+C가 막히지 않게 별도 호스트 스레드에서 실행한다. 실행 오류는 수동 접속 주소와 함께 보고한다. OS 설정·기본 브라우저 설정은 바꾸지 않는다.
- tokio Ctrl+C와 axum graceful shutdown에 서버 중지 watch를 연결한다. 활성 WebSocket에 Close를 보내고 시뮬레이션 채널의 sole sender를 버린 후 스레드를 join한다. full queue에서도 Stop enqueue 실패로 종료가 막히지 않는다.
- `lifecycle.py`는 기본 옵션 실행과 별도 폴더+명시 데이터팩 실행, HTTP·실제 MessagePack WebSocket·정상 종료·포트 반환을 확인한다. Windows는 숨긴 독립 child console에만 실제 CTRL_C_EVENT를 보내고 Linux/macOS는 SIGINT를 보낸다. 호출자의 콘솔에 신호를 보내지 않는다. --open 선택 테스트는 실제 launcher의 성공 종료를 기다린다.
- Playwright는 Vite preview 대신 oh_server 실행 파일을 시작한다. 기본 CI는 Chromium/Firefox/WebKit 3개 엔진이다. 로컬에 제품이 설치되어 있으면 OH_BROWSER_PRODUCTS=1로 Chrome/Edge도 검사한다. Safari 제품(macOS) 및 그 외 Chromium 파생 제품은 엔진 성공과 구분해 미검증으로 기록한다. M0 DOM 셸 검사이므로 GPU·지도 성능 증거로 쓰지 않는다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| Vite dev/preview를 별도 실행 | 개발이 편함 | 단일 실행 파일 사용성·실제 정적 파일 제공 증거가 아니다 |
| dist 없으면 빈 셸로 대체 | Rust만 빌드 가능 | 누락을 정상 응답으로 숨긴다 |
| 런타임 dist 경로에서 파일 읽기 | 서버 재빌드 없이 교체 | 실행 파일 외 클라이언트 배포 파일에 의존한다 |
| 데이터팩까지 런타임 임베드 | 완전 단일 파일 | 이미 지원된 명시적 팩 경로를 유지하고 공용 로더를 재사용하는 쪽이 최소 변경이다 |

## 결과와 영향
Rust 서버를 포함한 workspace 빌드 전에 npm production build가 필요하다. 실행할 때는 Node/Vite가 필요 없다. 소스와 tests, CI가 그 순서를 함께 명시한다. Windows에서 기본 브라우저 launcher와 실제 CTRL_C_EVENT를 검증하고 세 OS 실행은 CI에 연결한다. 이 구현 세션은 Linux/macOS 실행 또는 Safari 제품 검증을 로컬 실행했다고 기록하지 않는다.

참고: [axum graceful shutdown](https://docs.rs/axum/0.8.9/axum/serve/struct.WithGracefulShutdown.html), [Tokio signal](https://docs.rs/tokio/1.53.2/tokio/signal/index.html), [Windows console control event](https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent), [Playwright browser engines/products](https://playwright.dev/docs/browsers), [실제 서버 연결과 연결 종료의 Playwright API](https://playwright.dev/docs/api/class-websocketroute). 버전·제품 실제 실행 조건은 target/wp05/*-browser.json과 작업 로그에 기록한다.
