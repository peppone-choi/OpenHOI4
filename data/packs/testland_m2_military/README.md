# 최소 합성 군사 팩

`testland_m2_military` / scenario `m2_military`은 기존 `testland_m2_production`의 6국·120프로빈스·12주에 훈련·취소와 V7 복원을 검사하는 최소 군사 입력을 더한 독립 팩이다. 실제 전쟁·전투·AI·시대별 콘텐츠나 M2 전체 완료를 주장하지 않는다.

- 생산 모델/가족/9개 잠정 defines는 기존 팩 그대로다. N03/N06만 model1 허용과 재고0을 추가했다. 모든 초기 장비 재고는0이다.
- 기존 로더는 각 템플릿 바인딩이 모든 국가에서 유효해야 한다. 이 공통 model1은 그 계약을 만족하는 합성 입력이며 새 국가별 템플릿 정책이 아니다.
- `m2_small`: 전투 구성1개, 지원0개, 인력8, model1 장비6, 훈련2일. 수치는 기존 합성 `training.toml`의 small에서 가져왔다.
- 군대6개(id0..5, nation1..6, capacity2, priority0), 합성 지휘관, 배경 committed/reserved 모두0. 초기 사단·작업은 없다.
- 지도 PNG/CSV와 지도·국가·경제·시간 입력은 바이트 복사다. 새 팩의 생산 FTL에서 더 이상 참조되지 않는 `o2-pack-name`만 제거했다. 모델 표시 키는 유지하며 새 pack name은 ko/en `pack.ftl`에 있다.

생산 원장의 실제 `allocation[2]`로 6개 생산선을 만들고, 장비6 이상이 되는 첫날을 각 국가에서 측정한다. 현재 관측은 모두5일이다. tick120에 Train을 넣고 tick144에서 처음 Training/보유장비6을 확인한 다음 Pause한다. 이 시점에는 각 국가 재고2·reserved8이며, Cancel은 인력8과 보유장비6을 정확히 반환한다. 이 관측값은 게임 밸런스 규칙이 아니다.

```sh
cargo run -p oh_cli -- validate --deny-warnings data/packs/testland_m2_military
cargo test -p oh_data --test m2_military_pack
OH_M2_MILITARY_OUTPUT_DIR="$PWD/target/wp23/checkpoint" cargo test -p oh_save --test m2_military_pack
cargo test -p oh_cli --test m2_military_pack
npm --prefix client ci
npm --prefix client run build
cargo test -p oh_server --test m2_military_pack
cargo build -p oh_server
OH_M2_MILITARY_OUTPUT_DIR="$PWD/target/wp23/checkpoint" node crates/oh_server/tests/m2_military_query.cjs
(cd client && OH_M2_MILITARY_OUTPUT_DIR="$PWD/../target/wp23/checkpoint" npx playwright test --config playwright.m2-military.config.ts)
```

체크포인트 출력 경로에는 기존 `paused-training.ohsave`가 없어야 한다. 공유 `CARGO_TARGET_DIR`을 쓰면 CJS/Playwright의 `OH_SERVER_EXECUTABLE`을 실제 새 빌드 경로로 지정한다. CJS와 새 Playwright 설정은 기존 CI에서 자동 실행되지 않으므로 로컬 수용 기록을 따로 요구한다. 각 호스트는 localhost에만 바인딩하며 정상 SIGINT/exit0 및 잔여 PID 없음까지 검사한다.

출처·수치 단위·AI 보조 기록은 [SOURCES.md](SOURCES.md), 실제 실행 결과와 제한은 [WP-23 작업 로그](../../../docs/worklog/WP-23-military-pack.md)에 있다. 코드는 GPL-3.0-or-later, 이 팩의 합성 데이터/자작 이미지 계보는 CC-BY-SA-4.0이다.
