# ADR-0804 DPR 관측을 같은 canvas epoch에서 묶기

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-08, REQ-PLAT-03, REQ-MAP-04/05 |

## 맥락

exact6b6 정상 CI37543260232의 원82회 중 Chromium preferred/forcedGL이 각각 rawGL SDK 관측과 최종 lifetime 정리에서30초를 소진했다. 유효 M1 FAIL은 유지한다. 별도 f19/B73 진단37542615440의 c900 strict candidate 한 회는 Chromium의 SDK 왕복 감소를 보여주었지만 WebKit 시간은 증가했다. 모두 GL2였으며 GPU driver 원인을 증명하지 않았다. 신규 GPU 설정이나 timeout 조정 근거로 쓰지 않는다.

## 결정

[관측 동등성·필드 표](../plans/WP-08-M1-r3-schema.md)를 먼저 기록한 뒤 c900의 관측 묶기만 원 fixture에 적용한다. browser reader는 `dpr-observation.ts`의 self-contained 함수이며 Node 단언과 구분한다. 한 단계의 실제 canvas handle에서 capture 전 box/camera/viewport와 capture 후 buffer/resources를 읽는다. rawGL은 같은 handle에서 원 좌표를 다음 RAF에 읽되 entry와 RAF 내부의 단일 host/canvas·current·connected 조건을 확인한다. 버퍼 poll은 교체 중 canvas가 없을 때 -1 sentinel을 반환해 원 zero-difference readiness를 만족하지 못하게 한다. 중복 host/canvas는 즉시 거부한다.

원 DOM expects, old GL contextLost, Chromium6회 DPR, native2 생성, full viewport/element PNG 및 전체 decode, RGB±1, rawGLerror0/lostfalse, camera·선택·원장·권위 상태, lifetime·finally close를 유지한다. 원 testMatch8개와 timeout30초/expect5초/retry0/skip0, browser/GPU 옵션은 동일하다. 별도 negative 파일은 기존 full suite에 포함하지 않는다.

실제 원 DPR fixture의 PNG 관측 red channel만 +2로 주입한 실행은 기존 Node RGB±1 단언에서 exit1이었다. 원본 bytes를 바로 복구하고 기록했다. 별도 실제 DOM/GL negative는 중복·탈착·RAF 직전 교체를 거부하며 원 표시 성능/renderer 검증의 대용으로 세지 않는다.

## 검토한 대안

| 대안 | 검토 결과 |
|---|---|
| 기존 개별 SDK DOM reads 유지 | 같은 epoch 보장이 약하며 승인된 관측 비용 감소를 반영하지 못함 |
| timeout 확대/재시도/PNG·권위 단언 감소 | 기존 실패 경계를 훼손하므로 채택하지 않음 |
| profile SDK patch 또는 GPU/browser 설정 변경 | 임시 진단 영역이며 정상 fixture에 적용하지 않음 |
| Edge Connected 상태를 정상 Snapshot으로 대체 | 권위 상태를 위조함. 한 회의 비개입 wire 관측으로 원인이 재현되지 않으면 미확정 유지 |

## 결과와 영향

제품 client/map/network/App, Rust/server/protocol/save, hash/golden/defines/현지화/의존성/에셋 계약은 변경하지 않는다. 변경은 테스트 관측 책임에 한정한다. 효과는 같은 예산에서 원 단언을 유지하는지로 확인하고 플랫폼 전체 성능이나 간헐 실패의 해결로 일반화하지 않는다. fresh P-05/P-07/동일 main 정상 CI/P-12 판정은 별도다.

고정 설치 source `client/node_modules/playwright/lib` 및 실제 native 실행을 확인했다. [공식 evaluating 문서](https://playwright.dev/docs/evaluating)는 Node/page 환경 분리와 evaluate 인자를, [JSHandle 문서](https://playwright.dev/docs/api/class-jshandle)는 같은 handle의 evaluate/dispose를 설명한다. [공식 timeout 문서](https://playwright.dev/docs/test-timeouts)의 기본30초/expect5초를2026-10-07 검색 확인했다. 새 버전이나 내부 GPU driver 동작을 추정하지 않는다.
