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
