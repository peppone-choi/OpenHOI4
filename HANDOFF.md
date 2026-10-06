# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-07 08:08 KST, M1-r2 최초 독립 게이트 판정 경계 |
| 현재 마일스톤 | M0 PASS 보존. M1 진행 중: AC PASS6/FAIL1/사용자보류0. 전체목표 M6 |
| 기본 브랜치 CI | 게이트 대상 main6b6 일반 [37543260232](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260232) FAIL. [Core37543260267](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260267)·[Save37543260295](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260295)·[Simulation37543260312](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260312) SUCCESS. 이 P08 문서 commit의 실제 HEAD·CI 관측은 RUN_RESULT/CEO 기록에 별도 명시한다 |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

M1-r1의 W1/W2를 다시 세지 않았다. 이번 M1-r2는 W3 WP-11 저장을 구현하고 독립 SAV PASS·부모 전체 불변성·P07·같은 main c2e8 네 정상 CI SUCCESS로 통합했다. 구현750c·최종문서3ffdc·병합6383caf·증거c2e8·게이트초안6b6은 비문서247 blob 동일이다. 이전의 저장 미착수 상태는 해소했다. [통합 원본](docs/gates/evidence/M1-r2-P07/README.md).

새 독립 [M1 게이트](docs/gates/M1.md)는 exact6b6에 AC-M1-01~06 PASS, AC-M1-07 FAIL를 반환했다. 필수REQ13PASS/1FAIL(REQ-PLAT-03), 사용자보류0이다. Windows M060/M1205 PASS와 저장3OS PASS를 최신 Linux DPR 두 timeout 대신 쓰지 않는다. 이후 같은6b6 Simulation도SUCCESS를 확인했지만 일반CI FAIL는 남는다. C02의 첫 게이트 판정 경계에 도달해 P08·RUN_RESULT·실제 턴 종료로 끊으며 M2나 새수정루프를 시작하지 않는다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP00 | 역사 CLI done1 실패 | .orchestrator 역사기록 | 앱종료를CLI0으로 쓰지 않음 |
| WP01~06 | 기존 M0 통합·PASS | 기존보존경로 | M0 증거 유지 |
| WP07/09/10/12 | 이전 W1/W2 통합·PASS | 기존 앱/worktree | 새게이트 기본동작PASS 근거 유지 |
| WP08 | 제품9fbc 독립attempt6 PASS·기존통합; 간헐DPR/Edge초기상태미해결 | codex/wp08-map-rendering / .orchestrator/wt/WP-08 | 기존구현앱에서 구체P06·새exact독립P05·P07/최신mainCI |
| WP08 임시진단 | f19 한회Linux SUCCESS, 읽기수집 종료, 제품미채택 | codex/wp08-dpr-diagnostic-m1r2 / WP-08-diagnostic-M1-r2 | strict동등성·비용자료 검토; 추가push/run/retry/고립후속 권한없음 |
| WP11 | 통합됨, SAV01/02·AC04 독립PASS | codex/wp11-save / WP-11 | 스키마·native/3OS·파일오류원자성 증거보존 |
| M1 P12 | FAIL, 부모 전체불변성 유효 | detached6b6 / M1-r2-gate | AC07/PLAT03 실패를 다음RUN에 인계 |

## 실제 세션·증거

이번 RUN에 새로 만든 앱은 WP11 구현01a112dd-dd1f-7e71-be87-d898a264386d, 검증1 01a11320-f551-7501-8e23-5466e21e4cc9, 검증2 01a11334-c0d6-7eb2-a9c0-a4a96fbdb8d5, 검증3 01a11347-1a78-77e2-a43f-b02f4ae4877f, M1게이트01a1136b-98e4-7f23-8db6-4ad91758e31e다. 기존WP08앱01a11186-8f53-75c2-b9fb-873b27d8af82를 재사용했다. 최근 실제상태는 모두idle/completed이며 WP08 cursor a62a6b0a-9800-4c41-882e-b7bcb95ecd27:247, 게이트 cursor9f7882c4-ae6e-4731-af1f-d17f46927393:22다. [registry](docs/plans/evidence/M1-r2/app-threads.json)에 정확경로/HEAD/원문/실패를 보존한다. orch.sh status는 실제 역사 WP00.impl done1 하나이며 앱exit는 해당없음이다.

[WP11 첫FAIL](docs/verify/WP-11.attempt1.md)은 동일내용 protocol 재생성 뒤 rawindex가 변해 무효였다. [두번째FAIL](docs/verify/WP-11.attempt2.md)은 기본 E2E출력으로 추적40재작성/30내용변경·raw변경이 발생했다. 복구하거나 새before로 덮지 않았다. [세번째SAV PASS](docs/verify/WP-11.md)는 모든 producer의 실제cwd/env/default를 조사해 원env8개로ignored절대경로를 지정했다. 부모9175파일/HEAD/raw·semantic/list/각SHA/diff/status 동일, Python0다. 그 전체M1 Edge204PASS/1FAIL는SAV범위밖 실패로 유지한다.

새P12는 같은원설정의 Windows M060/M1205를 suite별단1회 실행했다. 부모9659파일/HEAD/목록/각SHA·raw원문/semantic/diff/cached/status가 최초baseline과 동일(actualPython0)임을 직접확인했다. rawSHA38cec4cb98b09067a6e47a85246f6894dea953209ef27c739818e757743ed8c7. [원문/manifest1621건·부모검사](docs/gates/evidence/M1-r2/README.md)는 모든원경로와SHA별raw ZIP매핑을 보존한다. 원target·실패worktree·브랜치를 삭제하지 않았다.

## 마지막 로컬 검증

| 명령·검사 | 결과 |
|---|---|
| 부모P07 | npmci/test130/build, fmt/clippy/workspace, M0hash2/actualM1hash2/type/license/assets/docs 및 실제native저장·HTTP/wire/PID·exe/JS/팩·정상종료 모두exit0 |
| 저장통합 c2e8 네CI | 일반37542282435/Save37542282595/Core37542282478/Simulation37542282466 모두SUCCESS |
| 새독립P12 | Rust108·client130·M060/M1205·native48+48vs96·세OS저장/코어·프로토콜actual생성/변조negative·실제M1hash2 PASS, 불변성exit0. 최신Linux회귀FAIL로게이트FAIL |
| 게이트6b6 일반CI | M036PASS/M180PASS·DPRpreferred/forcedGL 각30초FAIL, 후속Firefox두단계미실행 |
| f19 한회Linux진단 | control/candidate각36→82 exit0. source10전체dictionary 동일·ownPIDgone/LISTEN없음, worker460봉인파일 부모SHA불일치0 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D10/D12 | REQUEST0001 | 2026-10-06 | GPL3later/자체data·assets CCBYSA4/OpenHOI4 확정 유지 | 없음 |
| OPEN01/03/04/05/11 상세 | REQUEST0002 | 2026-10-06 | 기존승인범위 구현·후속후보 준비 | 미완성M4/M5 공식 |
| OPEN02 상세 | REQUEST0003 | 2026-10-06 | 간소화평화 후보준비 | 미완성M4 점수/참가/요구처리 |
| OPEN07/08 상세 | REQUEST0004 | 2026-10-06 | 기상·점령·첩보 후보준비 | 미완성M4 공식 |
| OPEN12 상세 | REQUEST0005 | 2026-10-06 | 업적 후보준비 | 미완성M6 기록/조건/mod/multi |

M1을 막는 사용자결정은 없다. OPEN01~12의 포함방향은 사용자답을 유지하고 미완성세부규칙을 임의확정하지 않는다. 함선설계·공개호스팅/계정은 시점미정 후속로드맵이다. 기술검증실패를 사용자결정 보류로 바꾸지 않는다.

## 알려진 문제

| 문제 | 재현 방법·제한 |
|---|---|
| DPR간헐timeout | 원 map-dpr.spec.ts,6b6 일반37543260232 preferred/forcedGL 두30초FAIL. f166/941/bbf/b73 실패도보존. RGB일치/cleanup0 관측만으로 timeout해결 주장없음 |
| Edge초기상태 | verify3 map.spec.ts adapterReject line101, Connected/Waiting for server·지도/시간없음·controlsdisabled. Snapshot수신/검증/적용 원인은미확정. 이번P12 Windows성공은해결증거아님 |
| strict진단제한 | b73 고정순차pair 한회·모든계측backend WebGL2. Chromium17.623→11.044초/15.451→10.756초,foregroundSDK279→204/205·expect53유지. WebKit두case시간증가. 내부중복SDK합계는walltime아님 |
| 진단원문 한계 | Linuxrawindex/exe/distpayload·OScwd 미업로드/미관측, 일부live DOM/oldcontext원반환값미serialize. length/SHA·원단언/PASS·기록snapshot의관측범위로한정 |
| 저장checkpoint제한 | invalid pending speed0/6 export거부로live큐/파일/hash보존. 모든live순간checkpoint보장아님. 실제전원손실/모든FS durability실험아님 |
| 미실행범위 | M3저장버튼/autosave/브라우저업다운로드·실세계/전세계·Apple Safari실기기·공개호스팅/계정/배포/태그/비용 |

[진단 세번째 원본/전문](docs/verify/evidence/WP-08-Linux-diagnostic-attempt3/README.md), [게이트HEAD 실패원본](docs/verify/evidence/WP-08-main6b6-CI-FAIL/README.md), [f166 원본](docs/verify/evidence/WP-08-mainf166-CI-FAIL/README.md), 이전두준비FAIL를 구분한다. 임시workflow/tools는 main에 통합하지 않았다. 반복/push/고립후속 권한은 소진됐고 자동재실행하지 않는다.

기존대표preview19461/source9fbc와구preview/PID는보존한다. 지리원천은 고정local파일·URL/date/version/license/SHA 및 offline생성, 독립DEM/하천·overlay검수를계획한다. 런타임외부지도API/타일/CDN 금지다. 원작공개화면의분할원칙/밀도참고만한정예외이며 경계복사·트레이싱·파일추출·수치표는금지한다. 실제원천취득/세계산출완료나새이동/전투효과를주장하지않는다. 승인/샌드박스/네트워크/모델설정·전역하네스 변경없음.

## 다음 세션이 처음 할 일 (3개 이내)

1. 최종P08 HEAD·원격·같은HEAD 네CI·게이트FAIL·registry/실제앱상태·C02 RUN_RESULT를대조한다. 최신FAIL/진행중을 과거green으로대신쓰지않는다.
2. 기존WP08앱에서 strict제품후보 동등성과 초기Snapshot 관측의 구체P06계획을 각각검토한다. 필요한수정은 새exact커밋·새독립P05/full불변성·P07/최신mainCI로진행한다. 추가임시Linux반복은자동승인되지않았다.
3. M1모든AC PASS 후에만 M2로진행한다. 다음RUN 경계·field별스키마기획·미정공식·클린룸/지리정책을유지한다.
