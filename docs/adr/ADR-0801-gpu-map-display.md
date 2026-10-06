# ADR-0801 GPU 지도 표시와 검증 경계

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-08, REQ-MAP-04, REQ-MAP-05, REQ-PLAT-03 |

## 맥락
02 §11의 공통 TSL·RG8·256² 조회 텍스처 계약을 실제 Three API와 연결한다. WP-09 서버 WorldView가 색과 참조의 유일 소스다. 지도 표시는 외부 지도 서비스에 접속하지 않는다.

## 결정
- `three` 0.186.1 MIT, `@types/three` 0.186.0 MIT를 정확히 고정한다. npm 공식 registry 조회와 설치 패키지 LICENSE를 직접 확인했다. 기본 WebGPURenderer.init → 실제 backend/limits 검사 → compile/render, 실패하면 새 WebGL2 renderer를 만든다. forceWebGL URL 옵션은 진단용이며 실제 WebGL2를 초기화한다.
- RG8 nearest/no mip/flipY=false. top-left row-major 배열과 plane UV의 Y 반전을 명시한다. CPU pick은 카메라 좌표를 원본 배열로 역변환한다. 선택 없음은 -1 dense sentinel로 모든 u16 실제 ID와 충돌하지 않는다.
- RGBA8 256×256 색/소유국/주 조회, ref validity B 채널로 null과 ID0를 구분한다. 공통 TSL textureLoad가 WGSL/GLSL을 생성한다. 국경은 화면 선 굵기와 카메라 축척에 따라 이웃 좌표에서 비교한다. 국가 > 주 > province 우선순위와 강조 RGB·굵기는 pack defines의 표시 설정이다. 규칙 효과 없음.
- 변경된 색 texel만 CPU에서 갱신하고 1×1 DataTexture를 공개 `copyTextureToTexture` API로 목적 조회 texture에 복사한다. Three r186의 공통 backend는 DataTexture.updateRanges를 처리하지 않으므로 그 API를 부분 전송 증거로 삼지 않는다. 측정은 실제 mode 전환에서 변경 byte 수·CPU 갱신 시간과 렌더 픽셀로 기록한다. M5 대형 데이터 최적화는 범위 밖.
- metadata는 map ID/width/height/dense IDs/pack hash/raw byte FNV-1a hash/길이/표시 설정을 Rust 타입에서 제공하고 TS를 생성한다. index는 pack query가 현재 해시와 다르면 409, 없는 지도는 404. 서버가 시작 때 검증한 원본 배열만 제공한다. 같은 서버 HTTP만 사용한다.
- raw RGB 바이트가 그대로 화면의 RGB 값이 되도록 lookup NoColorSpace, renderer LinearSRGBColorSpace/NoToneMapping, unlit material을 쓴다. 게임 UI에는 backend 진단을 넣지 않고 data-attribute/로그에 둔다.
- 1280×720에서 상단 시간, 지도 모드, 지도·선택과 옆 스크롤 패널을 배치한다. 작은 화면은 한 열로 전환한다. NationalPanels props와 서버 원장 문자열 계약을 보존한다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 별도 WGSL/GLSL 셰이더 | backend별 최적화 | 공통 알고리즘 검증 비용 증가 |
| full lookup upload | 간단 | 델타/모드에서 불필요한 256 KiB 전송 |
| 외부 타일/API | 세계지도 즉시 표시 | 고정 로컬 팩 경계와 사용자 지시 위반 |

## 결과와 영향
2026-10-07 P-06: 독립 검증에서 metadata와 index.bin의 기본 fetch가 CORS를 허용한 외부 origin의 302를 따라가 실제 GPU 지도를 표시하는 결함을 재현했다. 같은 origin으로 initial URL을 쓴 것만으로 외부 요청 경계가 보장되지 않았다. 두 경로를 동일 helper로 요청하고 `location.origin`에 고정한 절대 URL, `mode: 'same-origin'`, `redirect: 'error'`를 사용한다. HTTP redirect 자체를 browser fetch 단계에서 거부하므로 redirect 대상에 GET을 전송하지 않는다. 같은 origin redirect도 명시 거부하고 서버의 직접 endpoint와 pack query/hash/cache 동작을 유지한다. 기존 map-data-error 현지화와 authority 조회/선택 상태는 보존한다. 서버·wire/schema_version·시뮬레이션 변경은 없다.

공식 Fetch 표준의 redirect error는 network error를 반환하며, same-origin mode는 다른 origin URL을 거부한다. 2026-10-07 확인: https://fetch.spec.whatwg.org/#http-fetch , https://fetch.spec.whatwg.org/#http-redirect-fetch , https://developer.mozilla.org/en-US/docs/Web/API/Request/redirect , https://developer.mozilla.org/en-US/docs/Web/API/Request/mode . Response.url 사후 검사만으로 이미 전송된 요청을 막았다고 쓰지 않는다. 테스트는 실제 CORS 응답을 native browser의 follow 양성 대조로 소비해 유효성을 먼저 확인하고, app의 외부 GET 수/302/ko·en/canvas/기존 상태를 대조한다. 기존 독립 FAIL과 f58 원본 증거는 보존한다.

P-06 검증 fixture는 loopback HTTP source proxy에서 실제302를 보내며 나머지 HTTP/WS는 실제 Rust 실행파일에 전달한다. 별도 origin의 loopback 서버는 CORS200·원본metadata/bytes·exposed x-pack-hash를 제공한다. APIRequest의 no-follow302 probe와 browser native follow의 실제 CORS200/body 일치를 양성 대조로 확인한 다음, app의 실패 phase GET1/foreign GET0과 ko/en 안내를 검사한다. 시험 전용 서버/패킷 주입은 제품 endpoint나 GUI 플레이로 기록하지 않는다. 첫 로드에서는 renderer가 생성되지 않고, packidentity 재로드에서는 기존 renderer가 해제되면서 권위UI/선택·시간·원장은 보존된다. 기존 카메라 화면까지 남는다고 주장하지 않는다. Browser response/requestfailed/pageerror 이벤트차이는 JSON으로 남기고 그것만 PASS 기준으로 쓰지 않는다. 상세 의미는 WP-08-schema 문서의 오류 경계 표와 같다.

시뮬레이션·저장 포맷·골든은 바꾸지 않는다. 표시 defines는 pack identity에는 포함되며 private simulation hash에 영향을 주지 않는다. HTTP schema_version=1, precision을 보존하는 decimal byte_length, 필수 runtime shape·ID 참조 검사를 최종 계약에 적용한다. 초기 미버전 preview는 명시 거부하며 첫445b935 캡처는 역사 증거로 보존한다. 사용자 schema 추가 지시를 적용한 필드 표/참조·원자성/후속 미구현은 docs/plans/WP-08-schema.md다.

새 screenshot baseline은 client/e2e-m1/baselines/display-fixture.png로 고정했다. 독립 12×6 source fixture에서 x2 province-only / x4 state-only / x6 nation 경계의 직접 RGB·굵기 기대값과 실제 PNG pixels를 먼저 대조하고 눈으로 검토했다. 기존 골든을 바꾸지 않았다. 1280×720, DPR1, locale en-US, unselected/unhovered, camera reset, synthetic seed1/m1, no AA/mip, raw RGB로 캡처한다. 채널오차≤1, 전체 canvas mismatch≤0.5%이며 별도 직접 RGB·국경·굵기 단언은 그대로 유지한다. CI 기존 M1 testMatch에 map.spec.ts를 추가하고 target/wp08 증거를 기존 artifact에 추가했다. 검사 완화/OS·GPU 설정 변경 없음.

실제 본 세션의 Windows 브라우저 결과: Chrome154/Edge154는 WebGPU/WGSL, adapter.vendor=nvidia/architecture=turing(max8192), Chromium153는 WebGL2/GLSL SwiftShader(max8192), Firefox155 WebGL2(max16384), WebKit26.6 WebGL2(max16384). Firefox의 renderer 문구는 개인정보 보호로 `or similar`가 포함되므로 물리 GPU 정확한 모델로 확정하지 않는다. WebKit의 renderer 문자열도 실제 기기 사양과 동일하다고 단정하지 않는다. tiny 8×6 렌더 frame interval·shader source length·backend report는 개별 JSON에 기록했으며 실세계 지도 성능 목표/CI 실행/다른 OS hardware를 검증했다고 쓰지 않는다. 실제 WebGPU에서도 네 모드 RGB·독립 경계·selection/hover·부분색4byte 갱신이 실제 screenshot oracle을 통과했다. 어댑터/기기 실패·insecure는 테스트 주입 후 실제 WebGL2 픽셀로 검사했고 no-backend/texture limit은 명확한 ko/en 오류다.

독립 PASS/main CI는 별도다.

공식 출처(2026-10-06 확인): https://threejs.org/docs/pages/WebGPURenderer.html , https://threejs.org/docs/pages/DataTexture.html , https://threejs.org/docs/pages/Renderer.html , https://registry.npmjs.org/three/0.186.1 , https://registry.npmjs.org/@types%2fthree/0.186.0 . 로컬 설치 소스 `three/src/renderers/webgpu/WebGPURenderer.js`와 두 texture utils도 확인했다. axum 0.8.9의 json/query feature를 사용하여 추가된 serde_urlencoded 0.7.1(MIT OR Apache-2.0), serde_path_to_error 0.1.20(MIT OR Apache-2.0), form_urlencoded 1.2.2(MIT OR Apache-2.0), ryu 1.0.23(Apache-2.0 OR BSL-1.0)은 허용 목록 내이며 Cargo.lock에 고정한다. https://docs.rs/serde_urlencoded/0.7.1/serde_urlencoded/ , https://docs.rs/crate/serde_path_to_error/0.1.20 . npm license 검사 103 dependencies / 0 errors.
