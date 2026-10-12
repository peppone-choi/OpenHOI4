# OpenHOI4 실제 개발 화면 — 2026-10-12

Source: `5b6b4f7953cb6ed181e28c6df117dfc89c70670e`의 실제 Rust 서버와 Chromium 151.0.7922.173 웹 클라이언트 화면입니다. 이미지 생성·목업·게임 화면 수정 없이 원본 PNG를 보존했습니다.

**6개국·120개 프로빈스의 작은 합성 테스트 지도입니다. 완성 세계지도가 아닙니다.** 이동 후보는 로컬 WIP이며 전체 게임, WP16/WP17/M2 완료나 독립 검증 완료를 의미하지 않습니다. 보급 이동 효과는 아직 연결되지 않았습니다.

1. [지도와 훈련·배치 패널](OpenHOI4-01-synthetic-test-map-training-deployment.png): 훈련 Ready 상태에서 군과 배치 위치를 선택한 전체 화면입니다.
2. [배치된 사단의 이동 진척](OpenHOI4-02-synthetic-test-division-movement-progress.png): 지도에서 목적지 50을 선택하고 실제 Move를 실행했습니다. 사단 0, 경과 3시간, 잔여 497시간과 경로를 표시합니다.
3. [Stop 결과](OpenHOI4-03-synthetic-test-stop-after-current-edge.png): 실제 Stop 명령 뒤 이미 진입한 구간만 완료하고 프로빈스 2에서 정지했습니다. 잔여 경로와 이동 시간은 0입니다.

일부 피드백·생산 문구에 나타나는 `The server sent an unavailable message`는 미완성 현지화 fallback 표시입니다. 이 표시 결함도 숨기거나 이미지에서 고치지 않고 그대로 보존했습니다. 원본 3장을 시각 확인했으며 토큰·계정 개인정보·로컬 경로·터미널 내용은 관찰되지 않았습니다.

## 원본 PNG SHA256

| 파일 | SHA256 |
|---|---|
| OpenHOI4-01-synthetic-test-map-training-deployment.png | `d0dccb918618e88a81203156c5df59632e04fd48c9622c0809127d6112dd12a3` |
| OpenHOI4-02-synthetic-test-division-movement-progress.png | `d0f5b2f3464d8398418d7672b8359f43888bdd83980f779fe08de82215048417` |
| OpenHOI4-03-synthetic-test-stop-after-current-edge.png | `5f271ac36cf1c0f92f50e4657355e42247e7a4cfb129bfe577423ea377afc527` |
