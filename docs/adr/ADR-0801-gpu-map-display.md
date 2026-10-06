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
시뮬레이션·저장 포맷·골든은 바꾸지 않는다. 표시 defines는 pack identity에는 포함되며 private simulation hash에 영향을 주지 않는다. 실제 adapter/device/frame 측정·브라우저 행렬·픽셀 허용오차 최종 기록은 작업 로그에 추가한다. 독립 PASS/main CI는 별도다.

공식 출처(2026-10-06 확인): https://threejs.org/docs/pages/WebGPURenderer.html , https://threejs.org/docs/pages/DataTexture.html , https://threejs.org/docs/pages/Renderer.html , https://registry.npmjs.org/three/0.186.1 , https://registry.npmjs.org/@types%2fthree/0.186.0 . 로컬 설치 소스 `three/src/renderers/webgpu/WebGPURenderer.js`와 두 texture utils도 확인했다. axum 0.8.9의 json/query feature를 사용하여 추가된 serde_urlencoded 0.7.1(MIT OR Apache-2.0), serde_path_to_error 0.1.20(MIT OR Apache-2.0), form_urlencoded 1.2.2(MIT OR Apache-2.0), ryu 1.0.23(Apache-2.0 OR BSL-1.0)은 허용 목록 내이며 Cargo.lock에 고정한다. https://docs.rs/serde_urlencoded/0.7.1/serde_urlencoded/ , https://docs.rs/crate/serde_path_to_error/0.1.20 . npm license 검사 103 dependencies / 0 errors.
