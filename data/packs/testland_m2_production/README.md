# Testland M2 독립 합성 생산 팩

작성: 최병호 · 2026-10-09

`testland_m2`의 자작6개국/120프로빈스/12주 지도와 경제 입력을 자체 완결 복사한 생산 인수 입력이다. 기존 M1/M2는 수정하지 않으며 팩 상속·dependencies가 없다. `m2_production` 시나리오가 `common/production/initial.toml`을 명시적으로 선택한다.

두 합성 장비 모델은 같은 family의 generation1/2이며 IC-days/item과 steel units/item은 각각1/2다. 국가1/4는 두 모델, 국가2/5는 모델1, 국가3/6은 모델2를 허용하며 모든 초기 장비 재고는0이다. 전투 효과·훈련·OOB·보급·연구·AI·승리 규칙은 포함하지 않는다.

9개 production defines는 승인된 WP-15 자작 합성 fixture 값을 그대로 사용한다. 지도·경제·공장·인구·법률·배분 수치 역시 기존 M2 입력을 보존하며 모든 밸런스는 잠정이다. steel은 재고를 새로 만든 것이 아니라 기존 경제의 일일 flow4/국가를 생산 ledger에서 예약·차감하는 자원이다. 상세 출처와 단위는 SOURCES.md를 따른다.

```sh
cargo run -p oh_cli --locked -- validate --deny-warnings data/packs/testland_m2_production
cargo test -p oh_data --test o2_production_pack
cargo test -p oh_save --test o2_production_pack
cargo test -p oh_server --test o2_production_pack
```

기본 CLI run은 생산라인을 자동 생성하지 않는다. 실제365일 생산·반복hash·180일 저장+185일 재개는 전용 save 검사가 실제 각국 Create 명령을 접수해 검사한다. 서버는 새 임시 `<host>/testland`에 이 팩을 byte-identical 복사하고 `--scenario m2_production --pack-root <host>`로 시작한다. 이것은 기존 서버 선택 계약을 사용하며 임의 팩 라우팅을 추가하지 않는다.

별도 프로세스 HTTP/WebSocket 검사는 다음 순서로 재현한다. `<evidence>`는 이 검사만의 새 디렉터리다. 먼저 실제 Vite 번들과 서버를 빌드하고, save 검사가 만들어 낸180일 paused V6 입력으로 서버를 띄운다.

```sh
npm --prefix client ci
npm --prefix client run build
cargo build --locked -p oh_server
OH_O2_OUTPUT_DIR=<evidence> cargo test -p oh_save --test o2_production_pack
cargo run --locked -p oh_server -- --port 8080 --pack-root <host> --scenario m2_production --load-save <evidence>/paused-180.ohsave
```

별도 터미널에서 저장 복원을 검사한다. 이어 서버를 Ctrl+C로 정상 종료하고 `--load-save` 없이 다시 시작해 새 시작을 검사한다.

```sh
node crates/oh_server/tests/o2_production_query.cjs http://127.0.0.1:8080/ <evidence>/expected.json restored
node crates/oh_server/tests/o2_production_query.cjs http://127.0.0.1:8080/ <evidence>/expected.json fresh
```

각 소켓은 별도 simulation이다. 검사는6개국 권한과 생산 projection·명령을 확인하며 공유 멀티플레이 월드나 실제 브라우저 UI 검사를 뜻하지 않는다.

데이터·복사된 자작 지도: CC-BY-SA-4.0, OpenHOI contributors. 원작 자료·역사 수치표·외부 게임 데이터는 사용하지 않았다. 전체 WP-23/M2 완료가 아니며 독립 리뷰·인수 전이다.
