# 최신 M1 게이트 HEAD 6b6 CI 실패 원본

정확 `6b6abd79cbda5e4cf30b3d059c2c2f981d63ac9c`의 일반 [37543260232](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260232)/client112541071783는 FAIL다. M0 36 PASS, M1 80 PASS/2 FAIL(preferred·forced GL DPR 각30초), 후속 Firefox 두 단계는 미실행이다. Core/Save success와 이후 Simulation 상태는 별도 API 기록이다.

원 artifact11449149678은 2,832,955bytes/170members, API digest와 ZIP SHA256 `511880e4e1fd7c66ff742567e10c219a48230309f8257c6da573bccf7ad07372`가 같다. API·client.log·identity·원ZIP를 wrapper에 보존하고 SHA256SUMS.json으로 대조한다. 이전 c2e8 정상CI 성공이나 f19 임시진단 성공을 최신 main 성공·원인 해결로 쓰지 않는다. 추가retry를 실행하지 않았다.
