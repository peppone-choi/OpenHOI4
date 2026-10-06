# mainbbf 원본 CI 실패

2026-10-07 M1-r2 재개 중 직접 확인. main `bbf8762922cf74c3b54f6274c1bff83b0977da62`, [일반 CI 37525085041](https://github.com/peppone-choi/OpenHOI4/actions/runs/37525085041)·client112479929446 failure다. 다른6job·Core37525084900·Simulation37525084902는 success이며 전체 main 녹색은 아니다.

M0 36 PASS·M1 Chromium/WebKit81/82 PASS. preferred DPR 사례는 전체30초 timeout이고 finally `context.close()`에서 이미 닫힌 page/context/browser 오류가 기록됐다. forcedGL은 통과했으며 후속 Firefox native/headed 단계는 skipped다. 실패 단계/원인은 원본 및 별도 계측으로 확인해야 한다. 이 단일 forcedGL 성공을 해결이나 제품/fixture 원인 확정으로 쓰지 않는다.

artifact11442486461은 2,778,456byte·168members·API digest와 ZIP SHA256 `b22b503ad29133c2fd1bc2fe4eb2866f34b218143d466f113fc748e33d8b2b43`가 일치한다. 원본 ZIP/API/log/identity는 `original-evidence-bytes.zip`에 보존하고 source byte SHA는 `preservation.json`에 있다. 일반 텍스트 리뷰 사본의 줄바꿈 정규화와 원본 바이트를 구분한다. 기존 main941 및 과거 독립 PASS는 소급 수정하지 않는다.
