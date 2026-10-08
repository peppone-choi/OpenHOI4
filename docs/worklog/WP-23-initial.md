# WP-23 작업 로그 — 초기 합성 지리·국가·경제 콘텐츠

| 항목 | 값 |
|---|---|
| 상태 | 검증 대기 — 초기 콘텐츠 구현 범위, WP-23/M2 완료 아님 |
| 담당 | 콘텐츠 구현 세션 |
| 브랜치 | `wp/23-testland-cloud` |
| 기준 main | `0776e6b5a67951821860490964c402e518310ca5` |
| 대상 REQ | REQ-CNT-01 초기 DV 입력; 전체 PT/캠페인은 후속 |
| 선행 WP | 인수된 WP-14 producer·WP-07/09 지도·국가 계약 |

## 계획과 변경

독립 팩 `data/packs/testland_m2`에 중립 번호 이름의 가상 6개국·120개 육지 프로빈스·12개 주를 작성했다. 원 RGB 격자 지도, 주/소유/수도/VP, ko/en 문구, 기존 경제 schema에 맞춘 초기값을 담은 28파일이다. 장비·철도·OOB·연구·의제·전투·외교·AI·승리 규칙은 없다. 숫자와 법령 입력은 기존 자체 시험 값의 **밸런스 잠정**이며 영구 세계관이나 최종 밸런스가 아니다. 정의/초기값과 원천은 팩 SOURCES에 명시했다.

`assets/ASSETS.toml`에 신규 팩 28파일의 CC-BY-SA-4.0 출처 항목만 추가했다. 기존 manifest bytes 접두부, M1·M0 examples·golden·repro와 제품 코드·lockfile은 기준 main과 같음을 검사했다. schema/save/protocol/client/engine/CI는 변경하지 않았다.

## 테스트 우선 기록

| REQ / 계약 | 검사 | 구현 전 | 구현 후 |
|---|---|---|---|
| REQ-CNT-01 초기 입력 | pack manifest 존재 assertion | exit1, `REQ-CNT-01 initial pack absent` | 구조 검사 exit0: 6개국·120프로빈스·12주, 소유 수도·정확 분할 |
| 실제 producer 입력 | `oh_cli validate --deny-warnings` | 최초 cargo 빌드는 StorageFull로 검사 전 실패; 요구 실패로 계산 안 함 | exit0, maps1/scenarios1, 오류·경고 없음 |
| 콘텐츠 재현 | 두 독립 출력 위치의 상대 목록/SHA256 | 신규 입력 없음 | 28파일 전부 동일; PNG·CSV 아래 hash |

## 실행한 검증

아래 `oh_cli`/`oh_server`는 인수된 제품 소스 `4ae0446a01487e9d6d1415e7191a78407a8e0b6f`에서 이미 빌드한 실행 파일을 사용했다. 해당 소스와 기준 main의 crates/client/Cargo.lock/rust-toolchain diff가 없고, 이번 자체 client 빌드와 인수된 embedded client의 상대 목록/SHA256도 일치했다. 따라서 디스크 공간을 소모하는 동일 Rust 재빌드는 반복하지 않았다. 이전 251개 Rust 검사나 clippy를 이번에 실행한 것으로 주장하지 않는다.

| 명령 / 검사 | 종료 코드 | 실제 결과 |
|---|---:|---|
| `cargo run -p oh_cli --locked -- validate --deny-warnings data/packs/testland_m2` 최초 빌드 | 101 | 새 worktree의 중복 빌드가 StorageFull로 실패. 자기 미완성 target/debug만 정리; 소스·다른 worktree 보존 |
| `oh_cli validate --deny-warnings data/packs/testland_m2` | 0 | pack=testland_m2, hash `d656804578786a61`, maps1/scenarios1 |
| 독립 구조/두 생성 결과 비교 | 0 | 120 RGB/ID, 모든 픽셀·50×50 연결 블록, 12×10 정확 주 분할, 6개 소유 수도, 28파일 SHA 동일 |
| `python3 tools/check_assets.py --release` | 0 | assets 오류0 |
| `python3 tools/check_docs.py` | 0 | 문서 오류0·경고0 |
| `cargo fmt --check` | 0 | 제품 소스 변경 없음 |
| `oh_cli run --pack data/packs/testland_m2 --scenario m2_initial --days 365 --seed 1 --hash-out` 두 번 | 0 / 0 | 각각 `fff7c2cc478d8331` |
| 같은 팩 180일 실행, `--save-out <임시 split save>` | 0 | split hash `16caff1160221a94` |
| 새 프로세스 `oh_cli resume --load <split save> --pack data/packs/testland_m2 --days 185 --hash-out` | 0 | 최종 `fff7c2cc478d8331`, 연속 365일과 동일 |
| `npm --prefix client ci --offline` 뒤 `npm --prefix client run build` | 127 (명령 묶음) | 기존 캐시에 why-is-node-running3.2.1 없음(ENOTCACHED); 뒤 build는 tsc 없음127. 제품 실패와 구분 |
| `npm --prefix client ci` 정상 패키지 host | 0 | 고정 lockfile대로 61패키지; 소스/lockfile 무변경 |
| `npm --prefix client run build` | 0 | typecheck/Vite 성공; 기존 큰 bundle 경고 유지, accepted client dist와 SHA 동일 |
| byte 동일 임시 `testland` staging + 명시 `--scenario m2_initial` 서버 | 실행 | 28파일 동일, 신규 팩 identity 유지, loader/경로 product 변경 없음 |
| 실제 시스템 Chromium·WebSocket·query 콘텐츠 smoke | 0 (콘텐츠 범위) | 아래 브라우저 절을 따른다. 기존 favicon404는 별도 기록 |
| 새 정상 server restore·query·SIGINT lifecycle | 0 | 최종 save 복원, PC100/2000-12-31 00시/tick8760, query0·server exit0·포트 해제 |

## 브라우저·별도 서버 범위

시스템 Chromium으로 실제 서버가 제공한 앱을 실행했다. 테스트 helper의 첫 한국어 기대값은 기존 `building-industry` 번역 `공업 시설`을 `산업`으로 잘못 적어 실패했다. helper만 고쳤으며 팩·기존 카탈로그·기존 테스트는 바꾸지 않았다.

그 뒤 전체 기능 assertion은 통과했지만 console 오류0 assertion은 기존 `/favicon.ico` HTTP404 때문에 실패했다. 실제 URL을 별도로 확인했으며, 앱 지도/카탈로그 실패와 구분했다. 이 favicon 누락은 이번 콘텐츠 소유 범위 밖인 기존 공통 UI 자원 문제다. helper는 이 정확한 URL/404만 별도 ancillary 항목으로 기록하고 나머지 console/page/network 오류를 계속 거부한다. 없는 favicon을 가짜 응답으로 채우거나 제품 흐름을 바꾸지 않았다.

최종 콘텐츠 smoke는 exit0이었다. Chromium151.0.7922.173, 실제 WebGL2/ANGLE SwiftShader backend에서 frames436을 확인했다. 600×500의 300000개 dense index 픽셀을 모두 대조했으며 world는 6개국/12주/120프로빈스, 매 국가 경제 query는 IC20·4부문 각5·steel4·capacity/available250·committed/reserved0이었다. 국가/주 버튼과 6개 strip의 실제 지도 선택, 4개 지도 모드, EN/KO 문구, pause/resume, 정상 WebSocket1000 종료를 검사했다. 정확히 favicon404 console1개만 별도 분류됐고 다른 console/page 오류·실패 요청·외부 host 요청은 없었다. 따라서 전체 console-clean이라고 기록하지 않는다.

새 프로세스 정상 server restore는 최종 save를 실제 팩으로 복원했다. 첫 snapshot은 2000-12-31 00시/tick8760이며 모든 국가 PC100(raw6553600) 상한과 같은 경제/세계 query 값을 확인했다. 직접 소유한 child에 SIGINT를 보낸 lifecycle의 server exit는0이고 localhost 포트는 해제됐다. 최종 save hash는 다시 `fff7c2cc478d8331`이었다.

원 브라우저/생성 helper·이미지·save·raw logs는 임시 빌드 산출물이며 Git에는 넣지 않는다. 새로운 독립 검증 담당은 구현 커밋의 팩에서 같은 요구 기반 검사를 직접 반복해야 한다.

## 출처·재현

- [팩 README](../../data/packs/testland_m2/README.md): 지도 생성 알고리즘, CLI 실행과 현 서버의 임시 부모/testland 경로 계약.
- [팩 SOURCES](../../data/packs/testland_m2/SOURCES.md): 원 자체 데이터·기준 commit·정확 초기값·밸런스 잠정·CC-BY-SA-4.0.
- PNG SHA256 `e3eeb968006ecd7c557b55b55a6d9b55e3d05cbc7bbecc252cbac4802a8b527c`.
- CSV SHA256 `aa6079ea7816196b2ebee8633ee1407fcb379bb75cd5a27a5baf1f44d8207b3a`.
- 새 원천/의존성 없음. Python3.12.14/Pillow12.3.0, Rust1.99.0, Node24.19.0/npm11.9.0을 사용했다.

## ADR·차별화 초안

새 시스템 기술 계약은 없다. 기존 loader/producer 안의 콘텐츠 배치·중립 이름·RGB 격자 recipe만 정했다. 자체 합성 지도와 명시 원천으로 구현/검증용 초기 입력을 제공하며, 원작 경계·파일·숫자를 사용하지 않는다. 01 §11 편집은 이 구현 소유 범위 밖이므로 통합 담당에게 이 초안을 전달한다.

## 결정 필요·범위 밖 발견·못 한 부분

- 영구 국가 이름/세계관이나 최종 밸런스는 정하지 않았다. 중립 자체 시험 데이터에는 추가 canon 결정이 필요 없었다.
- 생산·보급 게임 정책 및 실제 equipment/rail/division schema는 WP-15/19/16 인수 뒤 작성한다. 이미 존재하는 phase marker를 플레이 기능으로 기록하지 않는다.
- 현재 서버는 임의 direct pack path 선택 대신 부모의 testland 자식 경로를 사용한다. 새 팩의 자체 validate/run은 직접 지원되며 브라우저 확인은 byte 동일 임시 복사로 했다. 엔진 일반화나 favicon 보완은 별도 범위다.
- P-11 예시의 `ai-bench --runs 5`는 아직 CLI 명령이 아니다(현재 USAGE에도 후속 WP로 명시). 실행/AI 결과를 꾸미지 않았다.
- 전체 WP-23, REQ-CNT-01 플레이 수용, M2 캠페인/전투/AI/대규모 성능·다른 브라우저 검증은 미완료다. 이 구현 commit 뒤 새 독립 검증과 같은 main CI/통합 판정은 오케스트레이터 담당이다.
