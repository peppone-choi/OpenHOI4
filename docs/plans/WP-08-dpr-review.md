# WP-08 DPR 진단 후보 검토

2026-10-07, M1-r2. 이 문서는 구체 후보와 실행 조건을 기록한다. Linux 원인 해결·제품 완료·독립 PASS 판정이 아니다.

## 관측과 원인 한계

제품·시험 대상 221 Git blob은 source9fbc와 mainbbf에서 동일하다. main79b/5602의 정상 전체 CI는 성공했으나 이후941·bbf·b73에서는 Chromium DPR preferred 또는 forcedGL이 30초 시험 본문 제한에 도달했다. 원본은 [941 실패](../verify/evidence/WP-08-main941-CI-FAIL/README.md), [bbf 실패](../verify/evidence/WP-08-mainbbf-CI-FAIL/README.md)에 있고 [b73 원본](../verify/evidence/WP-08-mainb73-CI-FAIL/README.md)도 보존했다. b73 일반37526905431은 M0 36/M1 81of82, preferred DPR poll timeout이며 Core37526905492와 Simulation37526905427은 success다. 전체 main 녹색은 아니다.

완결된 단계의 PNG·buffer·rawGL·camera·authority와 일부 최종 cleanup은 정상이다. 완결되지 않은 단계나 닫힌 page의 null 관측을 통과로 취급하지 않는다. bbf의 finally context.close 오류는 앞선 본문 중단을 가릴 수 있다. 서로 다른 중단 위치와 같은 제품의 성공 이력은 누적 비용 가능성과 일치하지만 Linux 내부 비용·간헐 surface 결함 여부를 확정하지 않는다.

## 검토한 정확 후보

ignored strict 후보의 SHA256은 `c9002e018ad536cc48308973a5d3400d69a8925c9732e936bdef30144ae429ac`다. buffer는 정확히 한 host/canvas를 검사한 직접 조회로, box/camera/viewport와 마지막 buffer/lifetime은 같은 canvas handle의 관측으로 묶는다. 전후 connected/current host/cardinality/backend/DPR/camera/box를 검사하며 rawGL은 RAF 전과 안에서도 같은 handle을 확인한다. 잘못된 epoch를 재선택하여 통과시키지 않는다.

기존 6 DPR 값·PNG 전체 디코딩/픽셀 허용치·rawGL/error/lost·old context·camera/선택/원장/권위·metadata/index 횟수·수명/cleanup·본문 finally/context.close·30초/5초 poll·retry0/skip0·브라우저/GPU 옵션은 유지한다. Chromium Frame.expect 53회, Firefox/WebKit 18회도 유지한다. 부모 검토에서 찾은 첫 canvas만 고르는 strictness 차이는 중복 host/canvas 검사로 보강했다.

Windows 원본 10사례 두 실행은 39.105초/38.709초, strict 후보 10사례는 34.015초로 각각 exit0이었다. Chromium foreground 요청은 preferred276/forced282에서 각각207로 줄었다. 각 브라우저에서 모두 빨라진 것은 아니다. foreground는 WS relay를 제외한 Playwright client-driver 요청이며 내부 CDP 전체 왕복 수가 아니다. 시간 합은 겹칠 수 있고 SDK 계측·순차 단일 관측의 한계가 있다. Windows 결과를 Linux 해결 근거로 대신하지 않는다.

실제 DOM 중복 host/canvas·교체/분리와 connected/current/backend/DPR/box/camera 오염을 거부하는 부정 시험이 통과했다. 이것은 DOM guard의 실효성 근거이며 GPU 결함 주입이나 독립 검증이 아니다. 부모는 봉인된 598파일의 크기/SHA를 직접 대조해 불일치0을 확인했고, 필요한 원본 묶음 ZIP9,872,114byte/SHA256 `4380f66c40edb2d4b1c66d9186dbf3b8fc094041335f8ca9b2f27573890ae7d4`를 ignored 증거에 보존했다. 구현자 전문·raw 시각·PNG·프로토콜 요청·tested source/diff/identity를 구분한다.

## 잠정 채택과 실행 전 조건

CEO는 이 정확 후보를 검증 대상으로, 가역 Linux 진단 1회를 기존 승인 범위로 잠정 채택했다. D·OPEN 확정 상태나 새 게임 규칙을 바꾸는 결정이 아니다. 최종 제품 수정·독립 PASS·Linux 해결을 승인한 것으로 기록하지 않는다.

임시 `codex/wp08-dpr-diagnostic-m1r2` 브랜치의 해당 workflow/tools 경로 push 한 번만 진단을 실행한다. 별도 source checkout은 exactb73에 고정한다. 각 arm은 원래 M0 36사례 다음 원래 순서의 full M1 82사례를 한 번 실행한다. 부모는 local commit의 정확 파일·SHA·준비 검사 원문을 직접 확인한 후 push한다. [GitHub 공식 manual-run 조건](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)에 따르면 새 workflow_dispatch 파일은 기본 브랜치에 있어야 하므로, 임시 파일을 main에 넣지 않는 좁은 push trigger를 사용한다.

실행 전 보강은 job부터 GIT_OPTIONAL_LOCKS=0, 실제 raw index SHA와 semantic index·목록·각 파일SHA·HEAD/diff/status 전후 대조, ordered project/file basename/title와 실제 파일 집합 비교, /proc/PID/exe 바이트와 빌드 파일의 SHA 대조, servedJS와 source dist 바이트 대조다. Git 읽기의 index refresh를 막는 관측 방식이며 승인/샌드박스/네트워크 설정 변경이 아니다.

원래 정상 CI는 유지하고 별도 push trigger 결과를 진단과 구분한다. 임시 파일을 main/제품 브랜치에 통합하지 않는다. 자동 반복·고립 follow-up·설정/GPU/prefs/단언/timeout 완화·비용/공개 배포/태그는 승인하지 않았다. 양 arm 실패 원문과 전후 상태/API digest도 보존한다. 작은 P-06이 필요하면 Linux 결과와 단언 동등성 검토, 새 정확 커밋, 새 독립 P-05와 부모 불변성, 최신 main 정상 전체 CI를 거쳐야 한다.

## 첫 진단 실행의 준비 실패

부모의 정확 파일 검토 뒤 임시 커밋 `8a386647ef7194a3edaf3a2ccc06c1de083eebfc`를 한 번 push했다. [실제 진단37531273066](https://github.com/peppone-choi/OpenHOI4/actions/runs/37531273066)은 첫 M0 선행 시험33통과·3실패로 종료했다. 세 실패는 모두 `../target/wp12/ledger-ko.html` ENOENT다. 정상 CI의 `npm test`가 `client/src/i18n.test.tsx`에서 ko/en 원장 HTML을 생성하는데, 임시 workflow에 그 생성 단계가 빠졌다. 부모의 실행 전 workflow 검토에서도 이 준비 의존을 놓쳤다. 제품이나 DPR 실패로 분류하지 않는다.

이어 같은19471 포트의 bare socket.bind가 Errno98로 중단되어 M1 원본·후보와 두 번째 warmup은 미실행이다. 이 오류만으로 LISTEN 잔존 또는 TCP TIME_WAIT를 확정하지 않는다. prep 전후·run 시작·첫 warmup 뒤 raw/semantic index·HEAD·목록·각SHA/diff/status는 동일하지만 전체 `product-state-after.json`은 생성되지 않았다. 불변성 증거를 그 확인 범위에 한정한다.

artifact11445156019는1,602,175byte·73members·SHA256 `acd52b982e8265a635e4bfefdf8e6690a88d891f0a2fc9ce39d30cd9c13da2f1`이며 API digest와 일치한다. 첫 warmup의 /proc 실행 바이트와 source build, servedJS와 source dist 바이트도 일치한다. 이 첫 실행으로 Linux DPR 비용·후보 효과를 판단하지 않는다. 추가 push/재실행은 하지 않았으며 정상 생성 동선·파일SHA, 포트 경계·실제 서버 종료 관측, 중단 시 최종 기록을 보강한 별도 구체 제안만 준비한다. 새 실행은 그 제안의 CEO 검토 뒤 배정한다.

첫 진단의 [부모 수집 원본](../verify/evidence/WP-08-Linux-diagnostic-attempt1/README.md)을 원문 로그/API/원본 ZIP 바이트와 함께 보존했다. 같은 임시8a의 별도 정상 CI37531272956/Core37531272909/Simulation37531273012는 성공했고 원래 DPR fixture를 썼다. 이는 strict 후보 효과나 main 최신 녹색 증거가 아니다. 정상CI와 진단 실패를 따로 기록한다.

후속 ignored 제안은 npm ci→원래 npm test→ko/en 원장 HTML bytes/SHA 기록→build, 구간별19471~19474 포트 및 이전 own PID/LISTEN 종료 관측, 준비·구간·전체 finally 원문과 source-after 보존을 포함한다. LISTEN 관측 실패·runtime identity/cleanup/불변성 실패면 남은 구간은 미실행으로 둔다. 유효한 full M1 단언 실패는 비교 데이터로 보존하면서 이미 계획한 다음 arm을 한 번 진행하고 전체 판정은 실패로 유지한다. 부모는 실제 파일·diff를 검토했고, 모의 실패 주입과 정상 HTML 생산 확인을 요청했다. 새 실행은 아직 미승인·미실행이다.

## 수정 진단의 두 번째 준비 실패

CEO가정확6파일수정안을추가1회대상으로채택했고부모는6d1fdccb3aed29709ceab7acc416a71c731c517d의15개committed/local/shadow바이트와manifest·정확6diff를대조한뒤단1push했다. [두 번째 원본](../verify/evidence/WP-08-Linux-diagnostic-attempt2/README.md)은진단37537984450/11446829246의FAIL을보존한다. 실제Linuxproducer130과controlM036은통과했지만복사config의기본server cwd가바뀌어M1 startup0case로실패했다. candidateM0/M1은미실행이고DPR비교측정은없다. 부모와CEO도config 위치가server 상대경로를바꾸는준비조건을검토에서놓쳤음을기록한다.

이번에는whole source-after·partial/error·남은미실행기록이생성됐다. 원source9139/rawindex/HEAD/semantic/목록/각SHA/diff/status의8지문을직접대조한다. 첫warmup은actualPID/exe/servedJS byte동일과종료후PID없음/LISTEN없음이관측됐고,M1은기동이실패해해당identity를증명하지못했다. TIME_WAIT를첫barebind원인으로확대하지않는다.

[공식 cwd 계약](https://playwright.dev/docs/test-webserver)과source의pinnedSDK를확인하고원client cwd를두generatedconfig의 webServer.cwd로명시하는2파일ignored최소안을검토중이다. 원command·옵션·브라우저·oracle/6DPR/82집합·timeout/retry/skip은유지한다. 같은actualconfig loader/native준비/팩·exe·servedJS와정리·outputguard를먼저검사해야하며추가Linuxpush/run은아직미승인이다. 다른성공CI를이실패나후보효과로대신하지않는다.


## cwd 보정 세 번째 한 회 Linux 진단 (M1-r2)

CEO가 승인한 최소2파일만 정확f19에 커밋했고 부모가15 committed blob/manifest/diff를 직접 검토한 뒤 단1push했다. 임시 [37542615440](https://github.com/peppone-choi/OpenHOI4/actions/runs/37542615440)는 sourceb73/strictc900의 control 원M036→fullM182 / candidate 원M036→fullM182 고정4구간 모두exit0다. [원본·읽기 전문·부모감사](../verify/evidence/WP-08-Linux-diagnostic-attempt3/README.md)에 API digest와같은artifactZIP·407members와 source10전체dictionary·worker460봉인파일 SHA 대조를 보존했다. 이전두준비FAIL는 유지한다.

단1회 순차pair에서 Chromium preferred17.623→11.044초/forced15.451→10.756초, foregroundSDK279→204/205·Frame.expect53→53이다. WebKit은1.925→2.282초/1.718→1.813초로 느려졌고 모든브라우저 개선은 아니다. 양arm PNG28전체표본/rawGL/후보epoch/권위camera/정리0을대조했으나 이번backend는모두WebGL2다. SDK합계는 overlap 가능, rawindex·exe/distpayload와OS cwd 미업로드/미관측·일부live DOM/oldcontext반환값미serialize라는관측한계는전문에있다.

진단branch정상CI3개도별도SUCCESS며 strict제품채택/새P05/main최신CI로대신쓰지않는다. 게이트source main6b6 일반37543260232는 원DPR preferred·forcedGL 두30초FAIL/M180of82/후속Firefox미실행으로다시실패했다. [최신실패원본](../verify/evidence/WP-08-main6b6-CI-FAIL/README.md). 기존Edge 초기Snapshot대기204/1도원인미확정이다.

다음 RUN은 strict 동등성·원53단언/6DPR/PNG/rawGL/epoch/authority/lifetime/finally/contextclose·30초/5초/retry0/skip0 보존의 구체P06 제품안과 초기Snapshot동선의 읽기 관측계획을 각각검토한다. 필요제품수정은기존구현앱→새exact커밋→새독립P05/전체불변성→P07/동일mainCI로진행한다. 이번승인된임시push/run은소진됐고추가retry/고립후속은없다. C02 새게이트판정뒤이번RUN을끊으며동일RUN에서새수정루프를열지않는다.
