# ADR-0002 브라우저 GPU 렌더링 경로

| 항목 | 값 |
|---|---|
| 상태 | 채택 — 기술 구현 계획, 게임 결정 상태 변경 없음 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-08, REQ-PLAT-01, REQ-PLAT-03, REQ-MAP-04, REQ-MAP-05 |

## 맥락

사용자는 웹 3D 게임에 사용할 수 있는 기술과 가용 하드웨어 가속을 활용하고 주요 브라우저를 지원하도록 지시했다. 기존 계획은 Three.js/WebGL2 지도였으며 WebGPU 선택·대체 경로와 셰이더 호환 정책을 명시하지 않았다. M0는 서버·웹 셸 기반이고 실제 지도는 M1/WP-08이다.

## 결정

Three.js WebGPURenderer로 WebGPU를 우선 사용하고 WebGL2로 대체한다. 가용 API 검사뿐 아니라 어댑터·초기화 실패를 처리한다. WebGL2도 시작할 수 없는 환경에는 현지화 안내를 제공한다. 브라우저·OS의 가속·보안 설정은 자동 변경하지 않는다.

인덱스 텍스처·색 조회·국경의 기본 알고리즘은 공통 TSL/노드 머티리얼로 구성하며 WGSL과 GLSL 백엔드의 결과를 검증한다. 원시 GLSL을 WebGPU에 그대로 넘기지 않는다. 인스턴싱·버퍼 갱신·LOD·압축 텍스처와 셰이더 기능은 지도와 측정된 성능 요구에 맞게 적용하고, 백엔드마다 가능한 기능·한계를 구분한다.

OpenGL은 브라우저의 WebGL/OpenGL ES 경로를 통해 활용한다. 서버의 결정론 시뮬레이션은 권위 Rust 코어에 유지하며 GPU 렌더링으로 옮기지 않는다. 기술 선택을 근거로 지형·게임 규칙·카메라 요구를 임의 확장하지 않는다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| WebGPU만 사용 | 새 GPU API에 집중 | 미지원·초기화 실패 환경에서 주요 브라우저 요구를 만족시키기 어렵다 |
| WebGL2만 사용 | 기존 경로와 호환 | 가용 최신 GPU API를 활용하는 사용자 요구에 따라 대체 경로로 유지한다 |
| 브라우저에서 네이티브 OpenGL 직접 호출 | — | 웹 그래픽 API 경로는 WebGL/WebGPU다 |

## 결과와 영향

저장 포맷·시뮬레이션 해시는 변경하지 않는다. WP-08은 두 백엔드의 지도 기능 동등성, 초기화 실패, 장치 제한을 검증한다. CI의 소프트웨어 렌더링 결과와 실제 장치의 GPU 성능 측정은 별도로 기록한다. Three.js 고정 버전 0.186.1의 실제 API와 생성 셰이더는 구현 시 소스·테스트로 다시 확인한다. M0 셸에 GPU 지도가 구현됐다는 증거로 이 ADR을 사용하지 않는다.

2026-10-06 공식 근거: [Three.js WebGPURenderer](https://threejs.org/docs/pages/WebGPURenderer.html)는 기본 WebGPU 선택, WebGL2 대체, `forceWebGL` 옵션을 명시한다. [WGSLNodeBuilder](https://threejs.org/docs/pages/WGSLNodeBuilder.html)와 [GLSLNodeBuilder](https://threejs.org/docs/pages/GLSLNodeBuilder.html)는 백엔드별 셰이더 생성 계약의 근거다. [MDN WebGL](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API)은 하드웨어 가속과 OpenGL ES 기반 API를 설명한다. 실패 처리와 기능 동등성은 이 프로젝트의 구현·검증 요구다.
