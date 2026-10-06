# ADR-0802 지도 초기화 실패의 수명 관리

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-08, REQ-PLAT-03 |

## 맥락
main 79140fb의 Linux Firefox CI는 WebGL2 초기화 실패 안내와 함께 `can't access property "getSupportedExtensions", this.gl is null` pageerror를 냈다. backend 가용성 실패와 제품의 예외 처리 실패를 분리해야 한다. 기존 87c1a99 Windows 독립 PASS는 Windows 범위의 증거이며 Linux 성공을 뜻하지 않는다.

고정 설치된 Three.js 0.186.1 소스를 확인했다. `WebGLBackend.init`은 null context를 `WebGLExtensions`에 전달하여 실패한다. `Renderer.dispose()`는 초기화되지 않았어도 마지막에 비동기 `setAnimationLoop(null)`를 호출한다. 이 함수는 이미 실패한 `init()` promise를 다시 기다리고, dispose 내부에서 기다리지 않은 promise가 같은 오류를 unhandled rejection으로 다시 낸다. Windows Firefox에서 getContext 반환값을 null로 만드는 기존 no-backend 시험에 pageerror 빈 배열 단언을 추가하면 같은 메시지로 실패한다.

## 결정
`createMap`의 init 실패 경로에서는 Renderer.dispose를 호출하지 않는다. 부분적으로 할당된 device는 공개 destroy API로, 실제 GL context가 있을 때는 WEBGL_lose_context 확장으로 해제한다. 아직 host에 붙이지 않은 canvas와 그 listener 참조는 수거될 수 있다. 정상 초기화 후 해제는 기존 renderer dispose를 유지한다. 라이브러리 소스와 브라우저 설정은 변경하지 않는다.

no-backend 회귀는 직접 GL 실패와 선호 GPU adapter=null 이후 GL 실패를 모두 검사한다. 두 언어의 안내, canvas 없음, JS 오류 없음, 정상 권위 국가·원장·일시정지 tick 보존을 확인한다. 원시 context가 없는 경우 지도 픽셀 성공을 주장하지 않는다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| dispose 반환 promise만 catch | 수정이 작음 | 내부에서 기다리지 않는 setAnimationLoop rejection을 잡지 못함 |
| Three 소스 패치·업그레이드 | 상위 구현을 바꿀 수 있음 | 고정 의존성과 변경 범위 확대 없이 제품 실패 경로를 고칠 수 있음 |
| 브라우저 GPU preference 강제 | 일부 context를 열 수 있음 | 02 §11.3 및 사용자 설정 변경 금지와 충돌 |

## 결과와 영향
시뮬레이션·프로토콜·defines·저장 포맷·지도 골든은 바뀌지 않는다. 단발 [Linux 진단 run 37493784925](https://github.com/peppone-choi/OpenHOI4/actions/runs/37493784925)은 제품 수정 전 2134b13에서 같은 Firefox155 빌드를 비교했다. headless GL1/2가 없고 GPU API도 없었으며 GL2 creationerror는 `AllowWebgl2:false restricts context creation on this system.`이었다. Xvfb headed는 GL1/2, renderer `llvmpipe, or similar`, maxTextureSize16384, 실제 RGBA[37,91,173,255], GL error0, pageerror0이었다. 이 결과는 해당 runner/build의 가용성 대조이며 Firefox/Linux 전부의 일반 법칙이나 게임 oracle 통과를 뜻하지 않는다.

일반 CI는 M0 전 엔진의 기존 headless 검사와 Chromium/WebKit M1 headless 검사 전체를 유지한다. Firefox M1 전체는 Xvfb headed로 같은 코드·픽셀 oracle·malformed/pageerror 단언을 실행한다. 별도 native headless Firefox 검사는 API 주입 없이 실제 context와 adapter를 읽고, 가용하면 실제 앱 프레임/RGB를, 없으면 ko/en 안내·canvas0·JS오류0·권위 국가/원장/paused tick 보존을 검사한다. 이 새 검사는 M1 전체에도 추가한다. 어떤 브라우저도 skip하지 않고 preferences·GPU flags·보안 설정을 강제하지 않는다. 임시 진단 workflow/script는 최종 제품 tree에서 제거하고 커밋 이력과 raw 증거를 보존한다.

버전·동작 출처: 고정 lockfile 및 `client/node_modules/three/src/renderers/common/Renderer.js`의 init/dispose/setAnimationLoop, `webgl-fallback/WebGLBackend.js`의 init, `WebGLExtensions.js` 생성자. Xvfb는 [Playwright 공식 CI 문서](https://playwright.dev/docs/ci)의 임시 Linux headed 실행 방법으로 확인했다(2026-10-07). 과거 Firefox 버그 보고만으로 현재 CI의 driver/context 원인을 확정하지 않는다.
