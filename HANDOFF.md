# 인수인계

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
