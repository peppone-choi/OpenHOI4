# 인수인계

## 진행 중 M2-r2 새 독립 검증 착수 — 2026-10-07 22:45 KST

WORLD P06-4는 actual idle/completed88/turn `01a1166e-d443-7c01-9ea1-49d6b574b5f9`, 실제 final823자로 종료했다. 최종 source5794505와 [부모 원본 인수](docs/worklog/evidence/WORLD-PREVIEW-M2-r2-P06-4/README.md)는 critical326/source snapshot·artifact554·ZIP556 member bytes/SHA/CRC 전수 불일치0이다. Git323 rawexact와 원래3개 UTF8 CRLF/LF내용동등은 따로 기록했고 원 bytes를 보존했다. 원ZIP50,409,960B/SHA348727bdbe6637f375ef51d62e0dc96745c178eed6d6b77b901f7e18a8d6f036는 생산자 경로에 유지하고 48MB orderedparts2개를 실제 재조립해 전체SHA/556CRC를 확인했다. 원446 네 산출물은 bytes까지 동일하며 원F05·초기client timeout·공간 부족 실패와 회복도 보존했다.

새 P05-4 앱 `01a1169b-dac7-7fb0-b9b1-ea51a36716ba`/turn `01a1169b-dde2-7091-bae0-edc31d1393d2`은 actualactive다. 새 detached `.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify4`, exact5794505, 최초 tracked10,017/전체 SHA·HEAD·semantic/rawindex·diff/status와 rawSHA `d30e2af98022fa3b6f70c5a0add1e3ba484c3676aa8a5fd47f4fc03c498bc6db`를 앱 생성 전에 기록했다. 실제 prompt18,687B/SHA56703152af29fef54dd9d3faa6d079ac053ef442fc24baf84866d7800c903226와 전송 payload bytes가 같다. 기존 앱/기준선을 재사용하지 않고 원180s/3GiB 및 전체 회귀·실화면을 새로 검사한다. 아직 독립 판정이 없다. 고유통합15/W1W2 전체0/0/M2 게이트 없음/지도 외관 우선은 그대로다.

main028d의 과거 같은 HEAD 6CI/25job SUCCESS는 유지하되 이 진행 기록과 새 custody는 현재 커밋 전이다. 새 문서 커밋도 별도 같은 HEAD CI를 확인한다. 이번 공간 복구와 검증 착수는 RUN 종료나 P-08이 아니다.

## 진행 중 M2-r2 관측 — 2026-10-07 22:40 KST

동일 RUN이며 종료 P-08이 아니다. main `028d7349d57280e74a5290d47da8443bc50e6a78`의 실제 동일 HEAD 6개 워크플로/25개 작업이 모두 SUCCESS다. 원격0/0·clean을 확인했고 registry에 기록했다. 고유 통합15, 원W1/W2 전체0/0, M2 게이트 없음은 그대로다. 아래 과거 main/활성 상태는 해당 시점의 관측이다.

WORLD 단일 구현 앱의 성능 수정 소스는 clean `5794505abb2b0cfeaace8005183cb6606fc68301`이다. 원446 대비 생성기 계산·할당 개선, 새 비용 테스트, ADR-3202, 작업 로그의 4파일만 바뀌었다. 자체 fresh72.438/71.594초와 원446 네 산출물 exactbytes 보존, Python33/Rust 회귀·별도client140·원M1 Chrome 단일41 및 화면 캡처를 기록했다. 병행 client139PASS+기존5초 palette timeout1/native1은 그대로 보존하고 별도140PASS와 구분한다. 원 독립 P05-3 F05는 유효 FAIL로 유지한다. seal-2 원 receipt는 ZIP50,409,960B/SHA348727bdbe6637f375ef51d62e0dc96745c178eed6d6b77b901f7e18a8d6f036, critical326/artifact554/member556이다. 아직 actual completed final 인수 및 부모 전수 custody가 끝나지 않아 새 P05-4를 시작하지 않았다.

E: Free0으로 부모 준비 JSON3개와 구현 선택적 비교 helper 쓰기가 실패했다. 원 0바이트 파일과 native 오류를 보존했다. 부모 재생성 incremental 캐시 삭제는 도구 정책에 의해 실행 전 차단됐으며 우회·삭제하지 않았다. 사용자 직접 공간 확보 후 Free약441.7GB를 확인해 봉인 준비를 재개했다. 원 ZIP/증거/브랜치/작업 트리는 보존되어 있다. 실패·회복 기록은 `.orchestrator/evidence/M2-r2-resume/space-exhaustion-preparation-failures.md`와 `space-recovery-complete.json`이다.

새 P05-4 초안과 setup/custody helper는 ignored 경로에 준비되어 있고 syntax만 확인했다. 실제 final·원 source/artifact/member bytes/SHA/CRC 인수 후 새 detached verify4 worktree와 최초 whole/raw 기준선을 만들고 새 앱을 배정한다. 원180초/3GiB/64MiB와 지도/ID·도시/해상 품질을 유지하며 두 timed bake 사이에는 검증자 자신의 heavy 작업을 병행하지 않는다. 사용자/다른 프로세스는 조작하지 않는다. 성능 관측·같은 카메라 지도 RGB 보존은 독립 PASS나 사용자 외관 승인을 대신하지 않는다.

현재 소스의 읽기 전용 merge-tree 사전점검은 ASSETS·ko·en의 append 충돌3개뿐이다. merge-tree native1과 HEAD/status/cached before/after 동등을 기록했으며 raw index bytes까지 대조한 것은 아니다. 실제 병합/P07은 새 독립 PASS 뒤에만 한다. 원 base prefix와 양쪽 append를 읽기 전용으로 대조했고 번역 키161개 중복없음/TOML parse를 확인했다. 이후 실제 union에서도 원문/중복ID·키를 검사하고 통합 상태 회귀와 같은 최종 main CI를 확인한다.

## 진행 중 M2-r2 최신 관측 — 2026-10-07 21:56 KST

새WORLD P05-3는 actual idle/completed15/turn `01a1164e-c360-79e0-935e-15ddcffab85c`의 **유효 FAIL F05**다. exact446540f/원10015파일·HEAD·semantic/rawindex·diff/status 전체동일,actualfinal9662chars·[공식전문](docs/verify/WORLD-PREVIEW.M2-r2.attempt3.md)과MF861/ZIP862 전수0를보존했다. 첫freshbake174.156s/native0/4파일shipped동일,second180.078s/native1 및extra180.188s/native1로원180s/3GiB/64MiBguard를넘었다. 원ZIP155621367bytes/SHA7790c6b6043de851602e3d737f8e0d8d265935572cb981faa36d8cdd7276ac7e와정확ordered48MB parts4를보존하고실제재조립전체SHA를확인했다. 데이터전체33554432sourcepixel/24889ID4connected/47,554adjacency/majorurban1846·3D·기존3browser각단일41·client140/Rust179 PASS는전체성능PASS를대신하지않는다. 원skip1은고정DEM별도독립전수검사와구분한다.

같은WORLD 구현앱 `01a11536-8218-7c93-9082-70931a350875`에성능전용 P06-4를배정했고새turn `01a1166e-d443-7c01-9ea1-49d6b574b5f9` actualactive다. source446/기존WT·branch,原source7/defines/해상도·180s3GiB·기하/ID/도시/sea/원tests/golden·UI/프로토콜/게임/GLB/의존성/CI를유지한다. 원loop의계산·할당비용개선으로가능하면4산출bytes exact보존을우선하며원완성파일복사로fresh생성을가장하지않는다. profiling조건/원parallel부하·명확한실패를기록한새source+새독립P05 freshpair가필요하다. P05-3앱/기준선/index를재사용·교체하거나원FAIL을재시도PASS로없애지않는다. 실제prompt6960bytes/SHAdcdd3811e8ec3de824434567352df3abc6adde73df75098fef02fa7b9e9b59eb와전송payload는같다.

부모main은a0389cc의과거6CI/25job 성공HEAD이며새custody/HANDOFF/docs 갱신은커밋전이다. 통합15/M0M1 PASS/원W1W2 전체0/0/M2게이트없음·외관우선/새기능후속/noCUA·focus·전역input/설정변경·공개배포금지는그대로다. 아래21:22절의P05진행중관측은역사이며현재권위는이절·registry·실제앱API다. 다음은동일소유성능보완final/원증거인수와새독립검수다.

## 진행 중 M2-r2 최신 관측 — 2026-10-07 21:22 KST

이 절은 RUN 종료 P-08이 아니다. 기본 브랜치 HEAD는 `a0389cc34ae45b89a7abb8c424240cfc5252e3d5`, 원격0/0·같은6workflow/25필수job 실제SUCCESS다. WP13 담당범위 통합을 포함한 고유WP15·M0/M1 PASS·원W1/W2 전체0/0·M2게이트 없음은 그대로다. 아래17:32 및이전M1 기록은 역사 관측이다. 이번 새producer custody와현재인계 문서는아직커밋/푸시전이며현재HEAD의과거CI를새문서commit CI로대신하지않는다.

WORLD 동일 구현앱 `01a11536-8218-7c93-9082-70931a350875`의 P06-3 turn `01a115cf-54d3-7ef2-b628-6224ecf88d6d`는 actual idle/completed56·정확final663chars다. 최종source `446540f9755f6d381f789192f3576a8cc17aadd2`, 전용 WORLD-PREVIEW-M2-r2 worktree/기존branch를유지한다. 전세계24,889=land13,882/sea6,954/lake4,053·onepixel6,610·원source-kind onepixel6,125 보존, Ireland지리본섬17중13개≥9px·4tiny/GB68/Honshu+Kyushu173(원source연결한계)/Hokkaido16/Shikoku10이다. 현대 MODIS2002–2003 major시가지footprint 분리와sea ID선택은미리보기한정이고나라/주/경제/전투/1936·2026현재도시/전체DEM·정확능선·고개는미완료다. 후보5/6 실제176.000/147.016s와PeakCommit약2.639GB는원180s/3GiB이내이나첫회여유4초를전체성능보장으로쓰지않는다.

[원producer 인수](docs/worklog/evidence/WORLD-PREVIEW-M2-r2-P06-3/README.md)는artifact671/ZIP673/source-snapshot324 rawbytes·SHA·CRC 전수0, Git321rawexact/3UTF8 CRLF-LF내용동일을별도기록했다. 최초부모Gitraw동등성가정준비FAIL은원helper/로그로보존하며source/index를복구하지않았다. 원ZIP107651219bytes/SHAfe53808374917360550ddf6bff6d2496b9519a99d2f9274ee069103a7fa4dab0는producer경로에보존하고48MB orderedparts3개를실제재조립해같은전체SHA/673CRC를직접확인했다. 원candidate1/2·d1de첫clean·3/4 oldcounter·새counterKeyError·fragmentaudit초안native1×2·a28/2d920 FAIL 모두보존한다. 수정통계11840는큰same-surface target없는noncore조각(427only-small/11413no-external-neighbour)이고실제섬계수가아니다. target의ordinaryprovince 전체가동일urban값이라는요구도없다.

새 독립 P05-3 앱 `01a1164e-c04c-7f01-b036-42cc456be9bd`/turn `01a1164e-c360-79e0-935e-15ddcffab85c`은 actualactive다. exact446540f detached `.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify3`, 최초10,015 tracked/wholeSHA·HEAD·semantic/rawindex·diff/status를앱생성전에봉인했고rawSHA `b6dfe05f2c0d0bf3b7d72a2a3a30978088f59567338b6e6b0641613dba9ce07d`다. 새검증자도자기최초스냅샷과부모원본전체동일을확인했다. 실제prompt16339bytes/SHA579b085aa38704d39ce1356d63ab6f4ed1c5fcf92a7b301dcca4ffb3c9bf2399와전송payload가exact다. 기존 P05 기준선/앱을재사용하지않고추적writer/기준선교체/index복구금지·검증결과아직없음이다.

사용자 ProvGen v1.0 지시도인수했다. CEO가원5,319,403bytes ZIP/README/Settings를읽어LandMap·gray육지크기·해안거리sea·지형/도시점·minmax/산악폭제어를확인했다. source/LICENSE/경계성장·시드·merge·특정알고리즘·재현성은확인되지않았다. WORLD 자기대조문서만추가했고EXE/DLL실행/디컴파일/BMP열람·인수/예시값·palette·코드copy/새grayimporter/canonicalformat/게임/DOPEN변경없음이다. 도시point nearest전체tag는실제footprint분리를대신하지않는다. 원user요청은DIRECTIVES와 `.orchestrator/ceo/research/provgen/ALGORITHM_REVIEW.md`에있다.

다음은새P05 actualfinal+whole/raw불변+원ZIP봉인 인수다. FAIL이면원판정/기준선을유지한P06/새검증으로이어가며PASS면간단한병합·원ASSETS/ko/en append양쪽원문보존·P07/같은최종main필수CI로통합한다. code판단충돌은구현소유가처리한다. 시각미리보기를전체WP32/45·M5 또는원W1/W2 묶음으로세지않고새게임기능은사용자외관우선순서의후속이다. noCUA/사용자focus·전역input·BP프로세스접근/설정변경/공개배포금지와원ToyTestland/golden/save호환·D/OPEN을유지한다. 실제최신상태는 `.orchestrator/app-threads.json`과원앱API가권위다.

## 진행 중 M2-r2 최신 사용자 우선순위 — 2026-10-07 17:32 KST

현재 오케스트레이터 `01a114ca-469a-73c0-a9c4-9df475c37769`는 실행 중이며 이 절은 RUN 종료 P-08이 아니다. WP13 통합 관측 main `a15978321b5e73ccbc8b43eebd63ba3e68d0eb5e`는 origin과0/0·같은6CI/25job 모두SUCCESS다. 부모가 실제9 Save/Trigger3OS artifact/API digest/member/fullstate/canonical/hash/freshresume와 public comparer3종 native0를 직접대조했다. 고유 통합WP15·M0/M1 PASS 유지·M2 게이트 없음·원W1/W2 전체 통합0이다. 아래 이전 M1 종료 기록은 역사다.

WP13 source080e는 새독립 P05-3 `01a11576-f042-7013-8cab-9e6ebb73e8ce` completed10의 유효 담당범위PASS(9951/raw70ea29f/MF2174/ZIP2175 전수0)·sameDBB P07 고유40검사·samea1596CI/실제3OS로 통합됐다. 원de0 P05-2 PASS와 원a652 Trigger pathFAIL·부모P07 freshreceipt누락/CLI timestampprobe원FAIL을 각각보존한다. 원39검사PASS+fresh출력의마지막16gate native0와 M0M1각2/v2v3의미대조를 인수했으며 원assert/source를고쳐통과시키지않았다. 전체4점수축·항복관전·결과UI는후속producer/consumer에남는다.

세계/HUD e209·실3D a28·지리자료/Fluent 정합2d920 원체크포인트는 사용자에게 전달하고 보존했다. a28 원F01/F02와9999/rawaf9986/MF533/ZIP534 전수0를 유지하며, 새2d920 P05-2는 completed9 유효FAIL(F01/F02通과·새F03edge0/F04u16상한). 전문8566chars·10001/raw0f41b665/MF1121/ZIP1122 전수0와 원207498377bytes ZIP의 무손실5부분·부분/전체SHA를보존했다. 같은지도담당 새qualityturn01a1157b는8k해안/4k전역+지역세분·현대도시좌표/실제KoreaDEM·F03/F04회귀를구현중이다. 후보2/3/4의해안kind오류/첫수정실패를보존하고동일최종source 후보5/6의전체kind/연결/bbox/대표DEM/adjacency/4SHA일치를확인했다. 새clean76f7104의20366(land10233/sea6080/lake4053) 실제Korea화면을확인했으며최종후속renderer/봉인·새독립검증 전이다. 표시PASS와최종지역품질을구별한다. marker19cf 원producer도보존했다. 실제활동/종료는registry가권위다.

사용자가 전세계·독립 프로빈스 렌더링과 메인 HUD 외관을 먼저 요청했다. 생성형 임시 마커를 병행하고 기능 연결은 후속이다. computer-use/CUA·사용자 화면/브라우저 포커스·키마우스 조작은 금지이며 Breaking Point UI 세션의 비간섭 검수 방법을 인수한다. [실제 배정·소유·검수 계획](docs/plans/world-preview-priority.md)을 따른다.

후속 사용자 3D 요청으로 지도 위 샘플은 실제 procedural low-poly GLB 메시로 전환했다. 기존 marker 앱이 같은 branch에서 units3d leaf 모델/recipe를 소유하며 원 imagegen PNG는 별도 보존한다. world 앱은 승인된 실제 3D leaf만 인수해 scene/depth/light로 연결한다. 원 PNG를 3D라 쓰거나 미검토 raster branch 전체를 가져오지 않는다. HUD 권위 게임 기능은 추가하지 않는다.

세계/HUD 구현 앱 `01a11536-8218-7c93-9082-70931a350875`, `.orchestrator/wt/WORLD-PREVIEW-M2-r2`, `codex/world-province-preview-m2r2`는2d920원turn completed23 뒤 quality후속 actualactive다. 생성형 마커 앱 `01a11537-ccb8-7f80-938d-6b86803b1bf2`/PREVIEW-MARKERS-M2-r2/codex/preview-marker-assets-m2r2는 completed다. 둘의 최초base는 ea63081이며 exactprompt/payload/cursor는 registry/증거에 있다. WP13 구현앱 `01a114e4-115e-7823-ba09-966b6efb7594`의 P06-2는completed32이며새검증을기다린다. 원ff78 유효FAIL2와rawdf3bb/MF2186/ZIP2187도보존했다.

같은 main43fd의 5CI/21job/Save 실제6개3OS artifact 성공과 macOS attempt1 원FAIL은 보존했다. 이후a9b6도같은5CI21성공이며 새a652는총6CI25job 중Trigger compare만FAIL인원자료를보존한다. 세계미리보기URL `http://127.0.0.1:4317/?world-preview=1`은자기VitePID33284의계속수정중인화면이다. 고정PNG는각checkpoint증거이며전세계/지역/실3D를CEO·사용자에게이미전달했다. 더좋은지역proof를다음에제공하며새기능연결은후속이다. 원Testland/golden/save호환·D/OPEN·기존실패와noCUA/focus금지유지·공개배포없음.

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-07 09:33 KST, M1-r3 첫 독립 P-12 판정 경계 |
| 현재 마일스톤 | M0 PASS 유지. M1 PASS: AC7/7·필수REQ14/14·사용자보류0. 전체목표 M6 |
| 기본 브랜치 CI | 게이트main89964f4의 [일반37550578186](https://github.com/peppone-choi/OpenHOI4/actions/runs/37550578186)·[Core37550578196](https://github.com/peppone-choi/OpenHOI4/actions/runs/37550578196)·[Simulation37550578264](https://github.com/peppone-choi/OpenHOI4/actions/runs/37550578264)·[Save37550578271](https://github.com/peppone-choi/OpenHOI4/actions/runs/37550578271) 모두SUCCESS. 이 P08 commit의 실제최종HEAD·같은HEAD 네CI는 푸시 후 RUN_RESULT/CEO 기록에서 별도대조 |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

2026-10-07 M2-r2 P-09 확인: 아래 M1-r3 종료 인계는 과거 기록이다. 실제 현재 마일스톤은 M2이며 M2-r1 부모와 WP-24 P-05-4 원·후속 turn은 interrupted로 중단됐다. 시작 main `1cf57fee1860dd3611088dd751c30251035775c4`는 clean, 실제 origin `82db68f67827cb0b4e6317fae29d1514a94b64f0`보다 ahead7/behind0이고 현재 HEAD CI는 없다. 원82의 CI/Save FAIL을 보존한다. 기존 검증 source `164f04e6378b959e54da67cbca19b1ff03b6fe9f`의 추적9880/각SHA/HEAD/semantic·raw index/diff/status가 최초와 같고 전용 native PID0임을 확인한 뒤 같은 독립 앱 `01a114b2-8a3d-7c91-9ba0-37959a8fbb11`에서 미완료 리포트·ZIP 봉인을 재개했다. actual final/PASS·P-07·같은 main5CI 전에는 통합 완료가 아니다. 실제 최신 상태는 [M2 증거](docs/plans/M2-evidence.md)와 `.orchestrator/app-threads.json`에 기록한다. 이번 RUN 종료에 아래 전체 인계를 실제 상태로 갱신한다.

이번 RUN은 M1-r3-W4(WP-08 수정) 한 묶음을 통합했다. 총 통합WP12는 그대로이며 새WP로 중복 계산하지 않는다. WP11을 다시 구현하지 않았다. 새 독립 [M1 게이트](docs/gates/M1.md)는 exact89964f4에서 AC-M1-07/REQ-PLAT-03의 이전 FAIL을 현재 PASS로 판정했다. AC6→7의 실제 진전이다. [이전 FAIL](docs/gates/evidence/M1-r2/original-gate-report.md)·[원 bytes](docs/gates/evidence/M1-r2/original-gate-report-identity.json)·[eb87 CI FAIL](docs/verify/evidence/WP-08-main-eb87-CI-FAIL/README.md)을 보존한다.

수정source0ab691d958a96e79f5f421004801a72173870d58→병합02cca1f→증거c316650→P12대상89964f4의 docs/HANDOFF 제외 제품248 Git blob이 동일하다. DPR 같은canvas/epoch 관측을 묶고 negative를 추가했다. renderer/App/network·Rust·wire/save·defines/현지화·의존성/에셋·workflow/config 변경없음. 원30초/expect5초/retry0/skip0·브라우저/GPU·DOM expects·PNG±1/rawGL·권위·camera/buffer·old-context/cleanup을 유지한다.

첫 새 P12 판정으로 C02 종료 경계에 도달했다. P08 커밋·푸시·정확최종HEAD 네CI 관측·RUN_RESULT 후 실제 턴을 종료한다. 제품변경·M2·추가회귀/반복루프를 시작하지 않는다. 다음 RUN은 CEO가 배정한다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP00 | 역사CLI done/exit1 | .orchestrator 역사기록 | 앱 종료를CLI0으로 쓰지 않음 |
| WP01~06 | 기존M0 통합·PASS | 기존경로유지 | 증거보존 |
| WP07/09/10/12 | 기존M1 통합·PASS | 기존앱/worktree유지 | 증거보존 |
| WP08 | 수정통합·새P05 PASS·P07·정상mainCI | codex/wp08-m1r3-stability / .orchestrator/wt/WP-08-M1-r3, source0ab691d | 원인미확정 기록유지 |
| WP08 P05 | PASS, 부모9,686파일/raw불변성 유효 | detached0ab691d / WP-08-M1-r3-verify | 전문·원본보존 |
| WP08 역사임시진단 | f19한회성공, 임시workflow/tools미통합 | WP-08-diagnostic-M1-r2 | 추가임시push/run/retry 자동권한없음 |
| WP11 | 기존통합·독립SAV PASS, 현재AC04 PASS | codex/wp11-save / WP-11 | 저장UI 후속 |
| M1 P12 | PASS, 부모9,722파일/raw불변성 유효 | detached89964f4 / M1-M1-r3-P12 | 원문·증거보존 |

## 실제 세션·증거

기존 WP08 앱01a11186-8f53-75c2-b9fb-873b27d8af82는 idle/completed275, 새P05 앱01a11398-8d27-7262-b6c3-c9f54d75681e는 idle/completed19, 새P12 앱01a113b4-4bc0-7740-ba9e-952792592294는 idle/completed11이다. 부모가 종료후 wait_threads로 직접대조했다. 기존24개는 착수에 턴종료 대조했고 현재진행 자식0이다. [registry](docs/plans/evidence/M1-r3/app-threads.json)에 정확 ID/cursor/HEAD/경로와 이전기록을 보존한다.

Git설치 Bash의 실제 tools/orch.sh status는 WP-00.impl done/exit1 하나, 명령exit0이다. PowerShell 기본bash는 WSL /bin/bash 부재로 실패해 설치된 Git Bash를 명시했다. 설정변경없음. 앱exit는 해당없음이다. 미병합 브랜치/worktree·원target·실패원본을 삭제하지 않았다.

[새P05 전문](docs/verify/WP-08.M1-r3.md)·[원본/부모검사](docs/verify/evidence/WP-08-M1-r3/README.md): manifest655/ZIP656 전수일치, 최초/최종전체JSON/raw동일. 최초create missing-file 지시는 producer0 준비오류이며 원fullbaseline 완성뒤 정정역할/범위로 수행했다. index복구/기준선교체없음. PTY nativeprobe wrapper1/nativeexit미수집 한계를 유지한다.

[새P12 원본/부모검사](docs/gates/evidence/M1-r3-P12/README.md): manifest1,567/ZIP1,568·15,439,305bytes 전수일치, 실제최종메시지 전문동일, 최초/최종JSON/indexbytes동일(actualPython0). rawSHA6077990c30eac37851cab7b8c4bad07a8cb4c12df30069a137755fdc23b76d92. 정책31PASS/1환경FAIL·ignoredouterworkspace환경FAIL은 유지하고 저장소밖 전용processTMP 원test1회PASS·기대cargo-deny6을 구분한다. 원trackedtest/단언/config불변이다. CEO도 직접 전수검사·불변성·전문일치를 확인했다.

검증자는 본인 exact899 서버에서 실제M1/seed1 syntheticTestland 지도4모드·선택/호버·팬/줌/reset·국가/주·원장·ko/en·속도1~5·재개/정지를 직접탐색했다. 18캡처와 이유/기대/실제를 원ZIP에 보존했다. ownPID40524 nativeexit0·정상종료로그·port19521listener소멸을 확인했다. 현재preview로 안내하지 않는다. 기존preview/PID는 건드리지 않았다. 자동E2E/직접GUI/AI벤치를 구분한다.

## 마지막 로컬 검증

| 명령·검사 | 실제 결과 |
|---|---|
| 부모P07 | npmci/unit130/build/typecheck·fmt/clippy/Rust108·M0/actualM1hash각2·licenses/assets/policy/docs·native저장/HTTP/wire/정상종료 모두exit0. [원본](docs/gates/evidence/M1-r3-P07/README.md) |
| 새P05 원Windows전체 | M060/60·M1205/205 PASS, 원설정/skip0/retry0. 원픽셀+2·guard/authority/camera/buffer negative FAIL 실효성 |
| c316 정상CI | 네종류SUCCESS, Linux36/82/nativeFF1/headedFF41. [원본](docs/gates/evidence/M1-r3-CI/README.md) |
| 새P12 | 직접Rust108/unit130·M0/actualM1hash각2·새process48+48vs96전체구조/hash·proto3599bytes+1byte negative·현재3OS Core/M0/M1/savecompare·직접GUI/nativeexit0 통과 |
| exact899 정상CI | 일반/Core/Simulation/Save 전필수job/compare SUCCESS, Linux36/82/nativeFF1/headedFF41. API digest=원ZIP SHA·Pillow PNG독립재계산 |
| 부모P12 불변성/보관 | 최초9,722파일·HEAD·raw/semantic/list/각SHA/diff/cached/status 동일, 전문 및1,567 source/1,568 ZIPmember 전수일치 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D10/D12 | REQUEST0001 | 2026-10-06 | GPL3later/자체data·assets CCBYSA4/OpenHOI4 사용자확정유지 | 없음 |
| OPEN01/03/04/05/11 상세 | REQUEST0002 | 2026-10-06 | 차량v2 CEO잠정문서·실제WP15계약대조/다른후보준비 | 미완성M4/M5공식·미승인콘텐츠값/저장호환파괴 |
| OPEN02 상세 | REQUEST0003 | 2026-10-06 | 간소화평화후보준비 | 미완성M4점수/참가/요구처리 |
| OPEN07/08 상세 | REQUEST0004 | 2026-10-06 | 기상·점령·첩보후보준비 | 미완성M4공식 |
| OPEN12 상세 | REQUEST0005 | 2026-10-06 | 업적후보준비 | 미완성M6기록/조건/mod/multi |

M1 결정차단0. 차량v2 exactbe159를 CEO가 docs04§2로 한정 잠정채택했고 01/02/03·REQUEST 대응을 맞췄다. [원검토](docs/decisions/evidence/transport-v2/README.md)를 보존한다. 사용자OPEN/D확정변경이 아니다. 실제WP15 장비/재고정수형/총계상한/C단위/ID/phase/명령/서버오류/호환을 먼저대조한다. 합성k/r 콘텐츠·생산규칙변경·구저장거부/migration미승인, M1제품적용없음. OPEN01~12 포함방향은 유지하며 함선설계·공개호스팅/계정은 시점미정후속로드맵이다. 비용/운영/공개배포/태그/법적검증승인으로 확대하지 않는다.

## 알려진 문제

| 문제 | 재현 방법·제한 |
|---|---|
| 과거DPR간헐timeout | 원6b6/eb87·이전FAIL ZIP보존. 현재strict관측수정/정상CI는 모든간헐원인해결의 증명아님 |
| Edge초기화면 | WP11verify3 source750 adapterReject Connected/Waiting·지도/시간없음. 유효body관측1회/fullPASS에도 과거원인미재현·미확정 |
| 환경/준비FAIL | P05 missing-file/PTYwrapper1, P12 정책TMP부모workspace·selector/parser·ignoredsealappend 원문유지. 제품FAIL/의도negative/환경FAIL구분 |
| 저장경계 | pending speed0/6 export거부·상태보존. 모든live순간checkpoint/모든FS전원손실durability보장아님 |
| UI한계 | 좁은영어표 제목줄바꿈. GUI속도5define0ms로 성능수치주장없음 |
| 미실행범위 | M3저장버튼/autosave·실세계/전세계·AppleSafari실기기·공개호스팅/계정/배포/태그/비용·AI벤치 |

지리원천은 고정local파일·URL/date/version/license/SHA와 offline생성·독립DEM/하천·overlay검수로 계획한다. runtime외부지도API/타일/CDN금지. 원작공개화면의 분할원칙/밀도 비교만 한정예외이며 경계복사/트레이싱/파일추출/수치표금지. 실제세계원천취득/산출완료·새이동/전투규칙 구현으로 쓰지 않는다. 모델/인증/승인/샌드박스/네트워크·전역하네스변경없음.

## 다음 세션이 처음 할 일 (3개 이내)

1. CEO 실제RUN_RESULT·P08최종HEAD/원격·같은HEAD 네CI와 gateexact899 제품248blob동일성·registry/app상태를 대조한다. 최신실패/대기는 과거green으로 대체하지 않는다.
2. M1 PASS·최종CI 확인뒤 CEO가 별도RUN으로 M2 P09/P02를 배정한다. 현세션은 M2구현미착수. WP별field/type/unit/range/reference/authority/hash/save/wire/UI·독립기대값을 먼저설계한다.
3. 실제WP15 계약이 생기면 차량v2와 C단위/재고/반환상한/phase/명령/호환을 대조한다. 후속미완성공식/콘텐츠값은 별도검토하고 사용자범위·클린룸·지리정책을 유지한다.
