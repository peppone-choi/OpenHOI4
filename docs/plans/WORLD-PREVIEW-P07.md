# WORLD-PREVIEW 기본 브랜치 통합 검수

2026-10-07 23:55 KST. 같은 M2-r2 RUN의 진행 기록이다. 독립 검증 source `5794505abb2b0cfeaace8005183cb6606fc68301`은 [P05-4 한정 PASS](../verify/WORLD-PREVIEW.M2-r2.attempt4.md)와 원10,017파일/full/raw 불변성·MF976/ZIP977 전수 대조를 인수했다. 이전 F01~F05와 모든 원본·실패는 유지한다.

main `c5d7f8e9628aec8a52227863204e9fc7fc5ab409`에 단순 합류한 검수 커밋은 `ea355b3b446a0a53276f9e1ce96da360bbb41470`다. 실제 merge native1의 ASSETS·ko·en 충돌3개를 보존했고, 공통 base 원문과 양쪽 append를 그대로 합쳤다. 에셋 경로100개와 번역 키161개는 중복이 없고, 제품 Git blob387개가 예상한 양쪽 조합과 모두 같다. 부모는 의미 코드 수정을 하지 않았다.

[원 P07 증거](evidence/M2-r2-WORLD-P07/README.md)는 MF1556/ZIP1557, ZIP16,892,531B/SHA9388119ef663f98355bf788ecaf1aade78a553c712690f42e4c5aa67c18bd7ac이다. 모든 member bytes/SHA/CRC를 직접 대조했다. 원 regression driver와 summary의 `WP13 integration` scope 문구는 재사용한 기존 기본 회귀 driver의 표지다. 원파일을 수정하지 않았으며 실제 이번 범위는 아래와 P07-summary.json에 기록한다.

| 실제 검사 | 결과 |
|---|---|
| 기본 회귀 명령40개 | 39 native0, 구 M0 strict 거부 검사1개 native1/expected1. 전체 driver native0 |
| client 단일 전체 | 243 PASS. 세계 source의140과 기존 main WP13 추가 검사103을 합친 결과 |
| Rust workspace | 195 PASS, fmt/clippy/architecture/라이선스/저장·재개·native 서버 검사 통과 |
| 지도 Python | 33실행,32PASS·기존 DEM 경로 skip1. 새 비용 검사6개 포함; 고정 DEM의 P05 별도 전수 검사와 구분 |
| M0/M1 | 각2회 b039d35666b77fc2 / b595dc2a1e5b4f8c, 기존 이동·해협 fullDTO/canonical/hash·저장 bytes 보존 |
| read-only generated TS | 실제 빌드 oh_proto.rlib의 typescript() stdout4119B가 Git/raw protocol.ts와 exact. 추적 writer 미실행 |
| 원 M1 화면 검사 | 단일 전체41, skip/flaky/unexpected/errors0, native0 |
| 실제 세계 화면 | 최소4인자 ownChrome·full17PNG/지역·도시·해상21PNG/native0, actualWebGPU/WebGL2·3D6/912triangles |
| 실제 선택·연결 | 도시4/해상3 원 pixel ID·hover/pick 일치. HMR1/gameWS0/unknownWS0 |

상단 HUD가 누락됐다는 부모 최초 시각 판독은 잘못이었다. 원 PNG·F07 관찰 기록은 보존하고 별도 정정 증거를 추가했다. main/source 세계 PNG의 whole bytes/SHA/RGB가 모두 같으며, 새 headless DOM 관측에서도 상단96px·OpenHOI4·지표가 visible이었다. 제품 F07은 재현되지 않았고 제품 수정이나 재검증 앱을 만들지 않았다. 기존 native0를 실패로 바꾸지 않는다. ownVite의 명시적 PID 종료 native1은 정상 검사 Chrome native0와 별도다.

현재는 로컬 P07이 통과한 상태다. 이 문서·증거의 새 최종 main 커밋을 푸시하고 같은 HEAD의6workflow/25필수job 및 Save/Trigger 실제3OS artifact를 확인해야 기본 브랜치 통합 판정을 마친다. 이전 ab0의 CI green을 새 HEAD로 대신하지 않는다. 총24,889/육해호수·source tiny·Ireland17/현대 MODIS2002–2003·전세계 DEM/정밀 수로·능선/미연결 권위게임 한계는 유지한다. 전체WP32/45·M2/M5·사용자 외관 승인이나 원W1/W2 묶음 완료로 세지 않는다.

## 통합 커밋 CI·원본 산출물 인수 — 2026-10-08 00:47 KST

통합 커밋 `81deb803944cf6297f13b3f14ca454774cea141b`의 실제6workflow/25필수job이 모두 SUCCESS다. [같은 커밋의 원본 증거](evidence/M2-r2-81deb-CI-final/README.md)는 실제9 Save/Trigger 산출물·API digest·ZIP bytes/SHA/CRC와 새 native comparer original/current/trigger 모두0을 보존한다. P07 checkpoint ea355와 제품387 Gitblob이 동일하다. 새 증거·인계만 추가하는 후속 문서 커밋의 CI는 별도 관측한다.

초기4workflow/8job은 runner0/steps0의 GitHub runner 획득 실패였다. 부모는 각 실패 workflow에 `--failed`를 한 번씩 요청했고 원native0 receipt를 보존했다. 일반/Core/Simulation의 실제 최신 attempt3와 중간attempt2 queued·jobs0, Save attempt2, Trigger/Localisation attempt1은 구분한다. 추가attempt3의 요청 주체는 계정actor만으로 확인되지 않았다. Save Windows/macOS는 원attempt1 성공 capture/15:10·15:08 업로드를 이어받았고 Ubuntu는attempt2 새capture다. 전3OS를 새attempt2 실행으로 기록하지 않는다. 최초 FAIL·log-not-found·추가attempt metadata와 원 dispatch는 originals.zip의 prior-observations 및 attempt-attribution.json에 보존했다.

세계/HUD/실제3D는 새 독립 P05-4의 한정 PASS·main P07·같은 통합 커밋 CI·실제3OS를 충족했다. 전체WP32/45나M2/M5, 사용자 외관 승인으로 확대하지 않는다. 고유WP15·원W1/W2 전체0/0·M2게이트없음은 유지하며 기존 W1-c WP25로 이어간다. 이 기록은 RUN 종료 P08이 아니다.
