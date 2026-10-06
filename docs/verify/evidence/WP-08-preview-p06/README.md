# WP-08 P06 runtime 직접 확인

2026-10-07, 구현 세션의 직접 GUI 확인이며 새 독립 PASS/전체 gate 판정은 아니다.

- 최종 구현 HEAD: 87c1a99a30de25732376d628958c68d37ce64cda. 이후 tracked 파일/커밋 변경 없음.
- 대표 URL: http://127.0.0.1:19425/ . PID32876, 정확 path/args/workdir/exeSHA/HTTP servedJS bytesSHA/packhash는 runtime-identity.json/process.json.
- exe SHA256: 7ac204825929972da31b3e88afadfa2e014b7898b046d1bfbf5b1f9713e714a0, target/debug exe와 같음.
- 실제 HTTP JS assets/index-DvCqic0J.js: 1169414 bytes, SHA256 7c50c1bf1c77e859d3132e2c43ac34db3e056b57a65762260e7593ef3de401b5, client/dist 바이트 일치.
- Pack b8d30ba577f303c3, metadata schema1, scenario m1/default seed1.
- 기존 f58 preview19418/PID19656은 그대로 유지했다. PID/path 검증을 포함한 기존 process 종료·교체 명령이 automatic approval review에 blocked by policy로 거절되었고 상세 사유는 없었다. 종료를 재시도하거나 설정을 바꾸지 않고, 승인된 새포트 preview만 시작했다. 기존 실행파일/원본 캡처 보존.

직접 UI 행동: IAB 새 tab2에서 pause→2000-01-01 12h/tick12, 영문 province30 지도 클릭→Southern country/state→60px right/30px down drag pan→Reset view→ko→province10 지도 클릭→지형 mode→인프라 원장 열기. 패킷/원시state 주입 없음. pan camera center (4,3)→(2.814229249011858,2.4071146245059287), reset (4,3). 실제 WebGPU/WGSL/nvidia turing/max8192. 최종 ko/province10/평야·북부주·원장1/tick12, 시간 정지(버튼 재개), alerts0, 정상 renderer. 대표 초기 상태로 이 탭을 유지했다.

캡처: en-province30.png, pan.png, ko-province10-terrain-ledger.png. direct-ui.json은 실제 DOM/카메라/renderer·행동 기록이며 자동E2E fault fixture와 구분한다. 이 ignored 증거는 부모가 별도 보존하고 독립 검증은 정확87 HEAD에서 수행한다.
