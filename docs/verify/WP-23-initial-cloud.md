# WP-23 초기 합성 콘텐츠 독립 검증

2026-10-08 새 구현 비참여 검증 세션의 직접 실행 결과다. **PASS — 지정한 초기 콘텐츠 범위**이며 전체 WP-23·REQ-CNT-01 캠페인 플레이·M2 완료 판정이 아니다.

| 항목 | 값 |
|---|---|
| 구현 커밋 | `b7c5b6898855a136456f7b558724f18cafa0656b` |
| 기준 main | `0776e6b5a67951821860490964c402e518310ca5` |
| 입력 | `data/packs/testland_m2`, scenario `m2_initial` |
| 초기 범위 | 가상 6개국·120개 육지 프로빈스·12개 주, 기존 경제 producer 입력 |
| 소스 보존 | 전후 HEAD·535개 추적 파일 목록/SHA·diff/status 동일, clean |

## 직접 실행과 결과

| 검사 | exit | 실제 결과 |
|---|---:|---|
| 구현 diff·source identity·전후 snapshot | 0 | 신규 팩28파일·asset manifest 추가·작업 로그 외 변경 없음. 기준 505개 파일 bytes와 기존 asset prefix 보존 |
| 두 독립 위치 재생성·구조 감사 | 0 | 대상과 두 출력의 28개 상대 파일 SHA 동일. 600×500 전체 픽셀, 120색/각2500픽셀, 연결된 블록, 격자 인접218개 |
| `oh_cli validate --deny-warnings data/packs/testland_m2` | 0 | hash `d656804578786a61`, maps1/scenarios1 |
| `python3 tools/check_assets.py --release` | 0 | 오류0 |
| `python3 tools/check_docs.py` | 0 | 오류0·경고0 |
| `cargo fmt --check` | 0 | PASS |
| `oh_cli run --pack data/packs/testland_m2 --scenario m2_initial --days 365 --seed 1 --hash-out` 두 번 | 0 / 0 | 모두 `fff7c2cc478d8331` |
| 같은 팩 180일 실행·저장 뒤 새 프로세스185일 resume | 0 / 0 | split `16caff1160221a94`, final `fff7c2cc478d8331` |
| 별도 초기 서버·실제 시스템 Chromium 콘텐츠 smoke | 0 | WebGL2/SwiftShader, frames435, 300000 index 픽셀 대조, 실제 지도 선택·4모드·EN/KO·pause/resume |
| 새 서버 정상 최종 save restore·query | 0 | 2000-12-31/hour0/tick8760, 매 국가 PC100(raw6553600), 기대한 세계/경제 입력 |
| 두 서버 종료·포트 재사용 | 0 | 실제 WS1000 clean close, SIGINT exit0, 같은 localhost 포트 bind/listen 성공 |
| `git diff --check <기준 main> HEAD` | 0 | 오류 없음 |

16개 실질 실행 receipt는 모두 exit0이다. 검증자는 제품 소스 `4ae0446a…`에서 이미 빌드한 CLI/서버를 사용하면서 기준 main·대상 commit의 crates/client/Cargo.lock/rust-toolchain identity를 직접 확인했다. 실제 HTTP로 받은 embedded client18파일 SHA도 비교했다. 변경하지 않은 전체 Rust/clippy/client suite를 이번에 반복 실행했다고 기록하지 않는다.

## REQ·출처와 보존

REQ-CNT-01의 초기 DV 입력을 직접 검사했다. 각 주는10프로빈스, 각 국가는2주/20프로빈스이며 모든 프로빈스는 정확히 한 주에 속하고 수도는 자국 소유다. ko/en 각31키의 중복·빈 문자열·참조 누락이 없다. 실제 server query는 국가마다 인구2000·IC20·4부문 각5·steel4·capacity/available250·committed/reserved0을 확인했다.

지도는 원작 자료를 쓰지 않은 자체 RGB 격자이며 중립 시험 이름과 기존 자체 입력의 출처·잠정 값은 팩 SOURCES에 명시했다. 새28파일의 CC-BY-SA-4.0 등록을 확인했다. 제품 규칙·시뮬레이션 코드·client·engine/schema/save/protocol·CI·locks·기존 M0/M1·golden/repro 변경이 없다. 미결정 production/supply/military/combat/diplomacy/AI 규칙을 구현하지 않았다.

- PNG SHA256: `e3eeb968006ecd7c557b55b55a6d9b55e3d05cbc7bbecc252cbac4802a8b527c`
- CSV SHA256: `aa6079ea7816196b2ebee8633ee1407fcb379bb75cd5a27a5baf1f44d8207b3a`
- 추적 소스 전후 snapshot SHA256: `c7978a80b7667f0b70efb38604ca569c8f22e0d19a0d33cf02149445854a5636`

## 남은 범위·발견 사항

기존 `/favicon.ico` HTTP404 console1건은 관측·보존했다. 정확한 URL/문구만 ancillary로 기록했고 다른 console/page/network 오류와 실패·외부 요청은 없었다. 전체 console-clean이라고 주장하지 않는다.

현재 서버의 `testland` 자식 경로 계약 때문에 신규 팩과28파일 SHA·manifest ID가 같은 임시 staging을 사용했다. 임의 direct-pack 서버 선택은 구현하지 않았다. Linux/Chromium151 범위이며 다른 OS·브라우저·대규모 성능·AI-bench·전체 콘텐츠·캠페인 플레이는 남았다. 이 문서 작성 시 같은 main CI와 병합 통합은 후속 확인 대상이다.

[구현 작업 로그](../worklog/WP-23-initial.md)와 실제 수량·해시·query 결과가 일치했다. 원 실행 receipt·스크린샷·helper·save와 내부 기록은 Git에 넣지 않는다.
