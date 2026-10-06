# ADR-0803 지도 표시 surface와 파이프라인 실패 복구

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-08, REQ-MAP-05, REQ-PLAT-03 |

## 맥락
독립 verify3/4의 Windows WebKit26.6은 viewport1280×720→1100×800→1280×720→Reset 후 screenshot에서 canvas가 배경 RGB[32,41,56]으로 표시됐다. 같은 RAF의 실제 default framebuffer는 RGBA[40,100,180,255], GL error0/context lost=false였다. 본인 fresh Playwright 관리 서버19440에서도 실패를 재현했다. 원시 버퍼 성공과 화면 픽셀 성공은 별개다. 옛 WebKit/Playwright 이슈를 현재 환경의 확정 원인이나 검사 면제 근거로 쓰지 않는다.

별도로 실제 Chrome/Edge GPUDevice의 createRenderPipelineAsync를 거부시키면 Three0.186.1은 error record를 세우면서 compileAsync promise를 resolve한다. 기존 코드는 canvas/backend/frames 성공을 보고하지만 지도 픽셀이 배경이었고 device도 살아 있었다. available GL2 fallback 또는 양쪽 실패 안내라는 기존 계약이 이 경계에서도 필요하다.

## 결정
실제 크기 또는 DPR이 바뀐 WebGL surface는 새 renderer/canvas로 재생성한다. camera offset x/y/zoom, 지도 모드, 선택 ID, 최신 서버 WorldView를 처음 그릴 때부터 복원한다. 첫 프레임을 그리기 전에는 새 camera를 공개하지 않는다. 같은 크기 Reset은 canvas width/height를 다시 쓰지 않고 카메라만 갱신한다. WebGPU는 기존 resize 경로를 유지한다.

ResizeObserver 안에서 DOM을 재생성하지 않는다. 기존 observer를 끊고 취소 가능한 다음 RAF에 재생성한다. dispose는 해당 RAF·렌더 RAF·observer·pointer/wheel listener를 해제한다. metadata epoch의 AbortSignal을 init/compile 경계에서 확인하고, 이전 renderer는 자기 canvas만 제거하여 더 새 renderer의 surface를 지우지 않는다. 모든 WebGL backend에 같은 경로를 적용하며 사용자 브라우저나 GPU 설정을 바꾸지 않는다. resize마다 WebGL 재컴파일 비용이 생기는 점을 허용한다. 성능 목표를 달성했다고 주장하지 않는다.

Three r186 backend의 createRenderPipeline 호출을 투명하게 전달하며 실제 생성된 pipeline key를 기록한다. 완료 후 GPU backend record.error와 GL programGPU LINK_STATUS를 확인한다. GL 성공 link 상태는 key별로 캐시하고 GPU error flag는 렌더 시에도 확인한다. Three 소스/API 호출 결과를 변경하거나 콘솔 오류를 숨기지 않는다. 이 adapter의 내부 record 이름은 고정 r186에 의존하므로 Three 변경 시 실제 native API 실패 회귀로 다시 검증해야 한다.

compile 실패는 성공 프레임 전에 처리하고, dispose를 기다려 GPUDevice.destroy 완료 후 GL2로 넘어간다. 의도적인 해제 전에 disposed를 세워 late device-lost 이벤트가 별도 fallback을 만들지 않게 한다. 양쪽이 실패하면 기존 ko/en 안내와 canvas0, 정상 권위 패널을 유지한다. 실패 surface의 backend/frames 진단도 제거한다.

## 검토한 대안
| 대안 | 관측 / 버린 이유 |
|---|---|
| CSS size만 변경하지 않기, 같은 canvas detach/attach | 동일 screenshot 회색 실패 |
| preserveDrawingBuffer 컨텍스트, translateZ(0) | 동일 screenshot 회색 실패. 채택하지 않음 |
| synchronous ResizeObserver 재생성 | 픽셀은 통과했지만 undelivered notifications pageerror가 생김. 다음 RAF로 수정 |
| 원시 버퍼 픽셀만 인정 / 회색 기준 / WebKit skip | 실제 화면 검사 계약을 만족하지 못함 |
| compileAsync resolve만 인정 | 실제 pipeline API 실패가 resolve 안에 숨겨져 잘못된 성공이 됨 |

## 결과와 영향
카메라/표시 수명만 바뀌며 시뮬레이션·네트워크 스키마·저장·defines·원본 지도/픽셀 기준은 바뀌지 않는다. resize 회귀는 실제 버퍼와 screenshot, 4모드×6셀 서버 wire RGB oracle, 선택·언어·시간·카메라·원장을 확인한다. pipeline 회귀는 실제 device 생성/호출/파괴 수, 실제 GL2 픽셀 또는 안내, GL shaderSource 실패, pageerror0와 권위 상태를 검사한다. native adapter 없는 엔진은 정상 GL/없음 대조로 따로 기록하며 native GPU compile failure PASS로 세지 않는다.

동작 출처는 고정 `three/src/renderers/webgpu/utils/WebGPUPipelineUtils.js`의 error/finally resolve, `WebGPUBackend.dispose`의 device.destroy, `webgl-fallback/WebGLBackend.js`의 programGPU/link 경계, `common/CanvasTarget.js`의 buffer resize, 본인 및 독립 실제 PNG/readPixels이다. [Playwright 공식 이슈17904](https://github.com/microsoft/playwright/issues/17904), [이슈586](https://github.com/microsoft/playwright/issues/586)은 역사적 비교 자료로만 확인했다(2026-10-07). scripted Playwright 검사는 직접 IAB 탐색 플레이와 구분한다.

## DPR-only 관측과 갱신 보강

독립 verify5의953에서 실제 CDP DPR1→2, CSS viewport/canvas box 유지 시 Chrome/Edge/Chromium의 preferred/forcedGL 6경로 모두 자동 canvas ratio 갱신이 기본5초 안에 일어나지 않았다. Reset 뒤에는 ratio2와 정상 PNG가 나왔다. 자체 회귀도 제품953에서 같은6 FAIL을 재현했다. 크기 비교식에 DPR이 포함되어 있어도 ResizeObserver 호출이 없으면 그 비교에 도달하지 못한다.

기존 렌더 RAF에서 devicePixelRatio와 마지막 적용 ratio를 비교하고 달라졌을 때만 동일 resize 함수를 호출한다. 새 타이머·listener·브라우저 설정은 추가하지 않는다. 이미 재생성 RAF가 예약됐거나 disposed이면 resize를 다시 예약하지 않고, 렌더 루프도 재생성 대기 중에는 종료한다. compile/debug 비동기 경계 뒤 첫 프레임 전에 resize를 다시 확인하여 그 사이의 DPR/크기도 반영한다. WebGPU는 현재 canvas/device를 갱신하고 WebGL은 위의 camera·mode·선택·최신 world 복원 경로를 쓴다. 기존 RAF 취소·observer·6개 pointer/wheel listener 정리는 유지한다.

resolution MediaQueryList.change만 연결한 후보는 실제 CDP DPR2/resolution2=true 관측에도 canvas1배로 남아 폐기했다. 기존 RAF 비교는 별도 이벤트 배달에 의존하지 않는다. [CSSOM View의 DPR/MediaQueryList](https://drafts.csswg.org/cssom-view/), [Media Queries resolution](https://drafts.csswg.org/mediaqueries-4/#resolution), [Resize Observer box 정의](https://drafts.csswg.org/resize-observer/#resize-observer-interface)는 2026-10-07 primary draft로 확인했다. 기본 content-box 관측과 device-pixel-content-box 관측은 별개의 계약이며, 해당 표준은 이 브라우저의 CDP 이벤트 배달을 검증한 증거로 대신 쓰지 않는다.

신규 DPR 회귀는 Chromium 계열의 실제 CDP2→1.5→1 반복/고정 CSS box, automatic buffer=Math.floor(CSS×DPR) 양축 정확값, nondefault camera/mode/selected30/ledger0, 서버 tick/date/hour 및 최신 display-only terrain palette를 검사한다. Chromium element screenshot에서 설정된 context DPR1로 돌아가는 관측과 surface 교체가 캡처 중 겹친 원본은 보존한다. live DPR PNG는 clip 없는 native full-viewport Page.captureScreenshot으로 전후 DPR/buffer ratio까지 검사하고 실제 화면 RGB±1과 동일 RAF의 GL RGBA/error0/lostfalse를 확인한다. GL 이전 canvas 분리/context lost, metadata·index 각1회, 404 epoch 뒤 observer/listener/RAF0과 authority 보존도 확인한다. Firefox/WebKit은 native DPR2 생성과 element screenshot만 검사하며 live DPR 변경 성공으로 세지 않는다. 기존 resize와4모드×6셀의 locator screenshot·안정 대기·RGB±1 단언은 그대로 유지한다. Linux953 timeout의 원인은 별도 phase 진단으로 판정하며 이 DPR 수정으로 해소됐다고 추정하지 않는다.

## Linux 전체·단독 실행 예산 비교

953 CI37504540992의 Chromium은 정상 resize/reset PNG 이후 States screenshot에서30초 전체 한도가 끝났다. 임시a15는 제품·테스트 단언·파일명/정렬/30초를 유지하고100ms 읽기 전용 bbox/canvas epoch/frame/camera/권위/viewport 기록과 trace를 추가했다. run37509083162의 full2workers는77 PASS/Chromium1 timeout, 별도 fresh server의 isolated1worker는 Chromium/WebKit2 PASS였다. 원본 artifact11435271106은458616866bytes/117members/SHA256 c0a6802b1d29404d762fe9b20734eca8a8466027d1bb69755e1c3a681d44b3ff로 API digest와 일치했다.

full Chromium220 sample은 CSS854×330→674×410와 canvas ID1→2만 보였다. bbox 위치37,327은 고정이고 repeated canvas/scrollbar feedback은 관측되지 않았다. first surface 평균 frameMs52.8, resize surface97.1이었다. UI 단계를 쌓아 약24초 뒤 첫 resize에 도달했고 resizedScreenshot의 trace48782→50906 도중 AfterHooks50788에서 전체30초가 만료됐다. restored viewport51484와 screenshot53361은 teardown fixture parent 아래 이어졌으며 실제 protocol의 restored setViewportSize 호출은 없었다. 따라서 뒤의 Element not attached는 test timeout 뒤 종료와 이어진 async body의 결과로 구분한다. 제품이 host를 지웠다는 증거가 아니다.

isolated Chromium271 sample은 ID1→2→3/정상크기, first surface frameMs30.4/resize48.9,8개 PNG/24RGB/카메라·권위 단언을 모두 마쳤다. body·collector 구간28269ms, teardown은 별도약3.4초여서 reporter31405ms PASS를30초 한도 변경으로 해석하지 않는다. WebKit full/isolated도15.2초/7.1초에 통과했다. 비교는 병렬 실행 중 frame/action latency 증가와 누적 예산 소진을 입증하지만 CPU 사용률·driver 내부 원인까지 측정하지 않았다. pause/tick의 변화는 본래 언어·run/pause 순서와 일치했고 다른 테스트의 공유 권위 조작이 실패 원인이라는 증거는 없다. trace/collector 오버헤드와 실행순서 차이도 한계로 남긴다.

M1 Playwright config의 worker를1로 명시해 GPU 표시 fixture의 겹치는 실행을 제거한다. 원래 테스트와 새 DPR 테스트41개/engine을 모두 그대로 실행하고30초·expect5초·retry0·skip0·원래 locator screenshot 안정 대기·픽셀±1을 유지한다. serial describe나 단계 분할을 사용하지 않으므로 앞선 case 실패가 뒤의 case를 건너뛰지 않는다. M0 config, CI 작업/프로젝트/브라우저 coverage, 제품 렌더 빈도와 shader는 바꾸지 않는다. [Playwright parallelism](https://playwright.dev/docs/test-parallel)과 [workers 설정](https://playwright.dev/docs/api/class-testconfig#test-config-workers)은2026-10-07 primary로 확인했다. 이 결정은 동일 한도 단독 실행의 실측 성공에 근거한다. 최종 제품·새 전체Linux CI 성공과 독립P05는 별도로 재확인해야 하며 순수probe 성공으로 대체하지 않는다.
