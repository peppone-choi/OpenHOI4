# WP-08 Linux 렌더러 CI 실패 조사

2026-10-07. main79140fbfc9966fc5104df735f068615633181d5b의 일반 [CI37491594894](https://github.com/peppone-choi/OpenHOI4/actions/runs/37491594894)는 client job FAIL이다. 다른6job 및 Core/Simulation success를 전체 CI 성공으로 쓰지 않는다. Windows87c1a99의 독립 PASS는 해당 환경·실행 범위로 보존한다. W2 완료 판정과 다음 저장 WP-11 착수는 보류한다.

## 실제 관측

Linux Firefox에서 M1 브라우저 18FAIL/81PASS. map backend/frame 속성이 생성되지 않았으며 지원불가 안내가 나왔다. national의 오류 수집에는 `can't access property "getSupportedExtensions", this.gl is null`이 추가됐다. malformed-wire에서는 지도 지원불가와 wire 거부 안내가 동시에 있어 기존 전역 alert 단언이 두 요소를 찾았다. 실제 context 초기화 실패와 정리 중 추가 예외를 분리해 조사한다. 원인을 버전이나 OS 이름만으로 추정해 확정하지 않는다.

원본 실패로그·GitHub 아티팩트 ZIP·API digest는 [증거](../verify/evidence/WP-08-CI-attempt1/ci-failure.json)에 있다. ZIP SHA256 `304081a8ea7c55a0329509c4e2f202dd527b6ed7d717cfaa42403dfab81bb09b`를 API와 직접 대조했고 전체 원본을 보존했다. review 사본은 ZIP member/SHA를 따로 기록했다. 검증 결과와 CI 실패를 서로 덮어쓰지 않는다.

## 정상 열람한 공식 자료

- [Playwright CI](https://playwright.dev/docs/ci): 기본 headless와 Linux headed의 임시 Xvfb 디스플레이 실행을 구분한다. 같은 테스트·픽셀 단언을 실제 디스플레이 실행 환경에서 검사하는 방법을 검토한다. 사용자 OS·개인 브라우저/GPU·보안 설정을 바꾸거나 기존 검사를 skip하지 않는다.
- [Mozilla1375585](https://bugzilla.mozilla.org/show_bug.cgi?id=1375585): 과거 headless WebGL 구현 제약의 조사 참고다. 이 자료만으로 현재 Firefox155/Linux 실패 원인을 확정하지 않는다.
- [Mozilla2036597](https://bugzilla.mozilla.org/show_bug.cgi?id=2036597): 2026년 macOS의 Firefox150 이후 headless 회귀 기록은 플랫폼 차이를 확인하는 참고다. Linux·Windows 전체 문제로 일반화하지 않으며 거기에 나온 보안 관련 preference 우회는 사용하지 않는다.

## 실행 범위

WP08 구현 채팅이 기존 source에서 초기화 실패→정리/pageerror를 먼저 재현·수정한다. 실제 가용 backend에서 기존 픽셀·국경·선택·대체 경로 단언을 유지하고, 실제 없는 backend에서는 현지화 안내·정상 권위 상태 보존·추가 JS 오류0을 검사한다.

로컬 Linux 서비스/설정은 가동하거나 바꾸지 않는다. 현재 WP08 브랜치에만 한정한 단발 Linux capability probe는 구현자가 구체 파일/커밋을 작성하고 부모가 검토·푸시한다. raw headless/headed context/버전/초기화 오류·실제 픽셀과 원본 아티팩트를 대조한다. 임시 진단 workflow는 최종 main에 남기지 않는다. sandbox·권한·네트워크·GPU preference 강제·테스트 약화로 CI를 통과시키지 않는다. 진단과 수정 후 정확한 커밋을 새 독립 검증하고 main CI를 재확인한다. 새로운 게임 규칙이나 전역 하네스는 추가하지 않는다.

## 단발 Linux 진단 실제 결과

정확한2134b131575ac30db48691e839f627405da66b52의 [진단37493784925](https://github.com/peppone-choi/OpenHOI4/actions/runs/37493784925)에서 동일 Firefox155를 headless/headed로 비교했다. headless는 WebGL1/2 context가 없고 GL2 creation error에 AllowWebgl2:false가 나왔다. 같은 빌드의 임시 Xvfb headed는 WebGL1/2 context·max texture16384, llvmpipe(or similar)·RGBA[37,91,173,255] 일치·glError0/pageerror0이었다. Chromium의 headless SwiftShader도 실제 GL2 pixel이 일치했다. WebKit의 reported Apple GPU 문자열은 Linux에서 실제 Apple 장치 증거로 쓰지 않는다.

[원본 identity/API digest](../verify/evidence/WP-08-linux-probe/identity.json), 전체 로그·6JSON/PNG·원본ZIP을 보존했다. ZIP SHA256 `3816dc0db972596914542727dbe7c78daa388a05cc4c006cee18d5e072487a77`가 API digest와 일치했다. 이 원시 capability 결과를 Rust/앱 지도 또는 전체CI PASS로 주장하지 않는다.

이 결과를 근거로 기존 Firefox 지도 픽셀 단언을 Linux 임시 Xvfb headed 환경에서 그대로 실행하고, Chromium/WebKit headless 검사는 유지한다. Firefox native headless의 실제 가용성/지원불가 안내·권위 상태 보존·추가 pageerror0은 별도 새 회귀로 확인한다. prefs/보안/GPU flags 변경·skip 또는 기존 단언 완화 없이 구현자가 구체 CI/수명 경계와 ADR을 갱신한다. 실제 앱 테스트와 정확한 최종 main CI 결과는 이후 별도로 대조한다.
