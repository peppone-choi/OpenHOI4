# WP-16 Ready 배치 UI 재현

기존 clean-room `testland_m2_military`와 기존 save generator를 사용한다.
소유한 Ready 작업을 실제 웹 화면에서 군·위치 선택 후 Deploy로 배치한다.
WebSocket 직접 명령 주입으로 UI 클릭을 대신하지 않는다.

```sh
export CARGO_BUILD_JOBS=1
export OH_M2_MILITARY_OUTPUT_DIR="$(mktemp -d)"
cargo run -p oh_proto --example wire_fixtures --locked
cargo test -p oh_save --test m2_military_pack --locked
npm --prefix client ci
npm --prefix client test
npm --prefix client run build
cargo rustc -p oh_server --bin oh_server --locked -- -C codegen-units=1
export OH_SERVER_EXECUTABLE="$(pwd)/target/debug/oh_server"
export PLAYWRIGHT_CHROMIUM_EXECUTABLE=/usr/bin/chromium
cd client
npx playwright test --config playwright.ready-deployment.config.ts
```

generator는 기존 `paused-training.ohsave`가 있는 출력 경로를 거절한다. 팩
mismatch를 강제 해제하지 않는다. 브라우저 worker1, Cargo jobs1과 실행
환경의 CPU/메모리 90% stop guard를 함께 적용한다. Chromium의 Unix socket
길이 제한 때문에 임시 경로는 짧은 전용 디렉터리를 사용한다.

## 수용 범위

- 실제 TimeControls로 Ready에 도달, served JS가 현재 build bytes와 동일,
  Deploy 클릭 두 번에서 전송 한 번, authoritative Deployed·군 편입·지정
  province·held 장비 이전·reserved→committed 경제 원장 이전을 확인한다.
- 장비0 입력은 새 generator의 복원 상태에서 실제 Pause line, Train3회,
  TimeControls를 사용한다. 기존 stock2의 분배 뒤 마지막 새 job의 held0을
  관측하고 false 거절/true 배치를 대조한다. fresh 팩의 즉시 훈련을 가정하지 않는다.
- 복원 입력에서 추가 훈련2회 후 군 정원2를 채우고 세 번째 Ready 배치의
  거절과 자원·사단·job 보존을 확인한다. 이 수치는 합성 fixture의 수용 값이며
  제품 UI에는 하드코딩하지 않는다.
- 타이밍/누락 응답 frame 주입 사례는 파일명과 증거에 주입을 표시한다.
  stale/미발행/old epoch world, 초안 초기화, 군사/world ACK 두 장벽을 검사한다.
- ko/en, 키보드 Enter, mobile390px, 실제 localhost proxy 종료 뒤 stale filter와
  송신 금지를 확인한다. 기존 training-controls suite의 실제 legacy bundle
  대조와 Train/Cancel 회귀 검사는 별도로 실행한다.
- 기존 `cargo test -p oh_cli --test military --locked`의 실제 native 프로세스
  검사로 NotReady·foreign army·wrong-controller·water·terminal·full army와
  valid control을 대조한다. 수용 UI와 CLI 실행 경계를 구분한다.

`MilitaryView.background`는 총 경제 인력 원장이 아니므로 자원 이전은
EconomyResult로 비교하고 background 자체는 보존 여부를 비교한다.
독립 리뷰/검증·정확한 head CI·main 통합은 이 재현 문서의 실행만으로
충족되지 않는다. 전체 WP-16/M2와 전체 게임 QA 완료가 아니다.
