# M1 지도 렌더러 API 확인

확인일 2026-10-06. WP-08 구현 전 공식 문서와 고정 r186 소스를 확인한 기술 참고다. 실제 GPU 지도 구현·호환성·성능 PASS 기록은 아니다.

Three.js [WebGPURenderer 문서](https://threejs.org/docs/pages/WebGPURenderer.html)와 [r186 구현](https://github.com/mrdoob/three.js/blob/r186/src/renderers/webgpu/WebGPURenderer.js)은 WebGPU 기본 경로와 WebGL2 대체 경로를 제공한다. `forceWebGL`은 WebGL2를 명시하며 기본 생성에서는 fallback 함수를 넘긴다. 이 지원 설명만으로 모든 초기화 실패나 장치 제한이 처리됐다고 판정할 수 없다. WP-08에서 실패 지점과 실제 최종 backend·렌더 픽셀을 확인한다.

[DataTexture 공식 문서](https://threejs.org/docs/pages/DataTexture.html)의 기본값은 nearest 필터, mipmap 생성 없음, flipY false, unpackAlignment 1이다. CPU 지도는 상단 시작 행 순서이므로 shader UV와 CPU 선택 좌표의 Y 방향을 독립 픽셀 기준으로 대조해야 한다. `RG8` 인덱스의 low/high 바이트와 256×256 색상 표를 다루며, 기본값에 기대기보다 필요한 계약을 명시한다. 색 공간 변환을 거친 스크린샷과 원시 서버 색을 비교할 때 캡처 조건·허용 오차도 기록한다.

[OrthographicCamera 공식 문서](https://threejs.org/docs/pages/OrthographicCamera.html)는 카메라 속성 변경 후 projection matrix 갱신을 요구한다. 팬·줌·화면 크기 변경 뒤의 CPU 선택 좌표와 실제 표시가 같아야 한다. 지도 밖 선택과 dense index→실제 province ID 대응을 별도로 검사한다.

위 자료는 의존성 버전·API의 공식 근거이며 실제 설치 버전과 라이선스는 WP-08 lockfile·ADR·명령 증거로 확인한다. OS·브라우저 설정을 바꾸어 GPU 테스트를 통과시키지 않는다. 테스트 소프트웨어 렌더링과 실제 장치 성능은 구분한다. 게임 중 지리 자료는 동봉 팩·자체 HTTP만 쓰며 외부 API·타일·CDN을 요청하지 않는다.
