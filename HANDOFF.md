# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-07 05:10 KST, M1-r1 두 묶음 통합 후 최신 CI 실패·읽기 진단을 인계 |
| 현재 마일스톤 | M0 PASS 보존. M1 진행 중. 전체 목표 M6 |
| 기본 브랜치 CI | 최신 main941765e의 [일반CI](https://github.com/peppone-choi/OpenHOI4/actions/runs/37522389014) client FAIL·다른6jobs success. [Core](https://github.com/peppone-choi/OpenHOI4/actions/runs/37522389167)·[Simulation](https://github.com/peppone-choi/OpenHOI4/actions/runs/37522389067) success. 최신 전체CI 녹색 아님. 이전 main79b0aa3의 세CI·actualM1세OS ZIP/hash60448355cecffa9d와5602c6c의 세CI success는 보존. 이번 종료 문서 HEAD CI는 실제 관측 상태를 RUN_RESULT에 기록하며 성공 대기를 종료 조건으로 추가하지 않음 |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

최초 read-only M1 준비 뒤 CEO가 전달한 사용자 본실행 승인으로 M1-r1을 진행했다. RUN 종료 조건은 M1 게이트·두 묶음 통합·결정으로 가능한 WP 없음 중 먼저 오는 것이다. W1(WP07/10/12)과 W2(WP09→08)는 source main79b0aa3에서 새 독립 PASS·통합·같은 HEAD main CI를 충족했다. 이후 문서 HEAD941765e에서 DPR 두 사례30초 timeout이 발생해 최신 CI 녹색이 아니다. CEO의 05:05 C-02/P08 지시에 따라 두 묶음 통합으로 이 RUN을 진행중 상태로 종료하고, 최신 실패·읽기 진단을 다음 RUN에 인계한다. 무기한 새 수정/검증 루프를 이 RUN에 추가하지 않는다. WP11과 M1 게이트는 미착수이며 최신 CI 실패 정리 뒤 다음 RUN에서 진행한다. 유지보수 WP01 AI 정책은 별도 묶음으로 세지 않았다.

M0 gate d6ae2c8의 AC7 PASS·sameHEAD CI15jobs·7artifact ZIP SHA와 제품 골든은 유지했다. M0의 시간 hash를 실제 M1 국가/주·원장입력·큐 hash로 대체하지 않는다. 저장 재개는 아직 미구현이다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP00 | 역사 CLI done1 실패 | M0 증거 | 앱 정상 종료를 CLI0으로 기록하지 않음 |
| WP01~06 | M0 통합 | 기존 경로 | M0 증거 보존 |
| WP01 AI 정책 | 유지보수 독립 PASS·통합 | codex/wp01-reviewed-ai-assets, WP-01-ai-policy | M1 묶음 횟수 제외 |
| WP07/10 | 독립 PASS·통합 | codex/wp07-map-data / codex/wp10-ledger | data/원장 기반에서 WP09/08 실제 연결됨 |
| WP12 | 최초 malformed Snapshot FAIL 뒤 수정8831613 새 PASS·통합 | codex/wp12-localization | 첫 FAIL·새 PASS 보존 |
| WP09 | 독립 PASS·통합 | codex/wp09-national-state, WP-09 | 실제 국가/주/원장/ordered queue/Fx/expiry/hash와3OS 확인 |
| WP08 | 9fbc 새 전체 독립 PASS·통합ebc1c5b·source main79b/5602 CI success, 최신941 DPR CI timeout | codex/wp08-map-rendering, WP-08; 기존 구현 채팅 읽기 진단 중 | M1-r2에서 원본·단계시간 대조→필요한P06→새독립검증/최신main CI |
| WP11 | 미착수 | 없음 | 다음 RUN, 실제 M1 save-resume |
| M1 게이트 | 미판정 | 없음 | WP11 전체통합 뒤 새 독립 게이트 |

## 세션과 실패 이력

사용자 승인 앱 별도 local 채팅·WP별 명시 Git worktree로 실행했다. 모든 명령은 담당 경로, 검증은 exact commit detached 새 경로다. 부모가 HEAD/index/추적 목록/각SHA/diff/status를 전후 비교했다. 04:41 snapshot의 19채팅 completed는 이전 관측이다. 최신941 CI 실패 뒤 기존 WP08 구현 채팅01a11186-8f53-75c2-b9fb-873b27d8af82/local을 읽기 진단으로 재개했다가 완료보고 후 종료했다. 최종앱idle/completed·cursor a62a6b0a-9800-4c41-882e-b7bcb95ecd27:120, source9fbc/clean, 지정 E:/openhoi/.orchestrator/wt/WP-08이다. 현재19채팅은 모두종료됐다. 본인 ignored target/wp08-dpr-ci2/HANDOFF.json·SHA256SUMS·prior-main941·png-original-profile.json/.log에 원본/13PNG 색일치/로컬decode시간을 기록했다. 새server/port/PID/testloop/추적수정/commit은없다. 로컬decode시간으로Linux단계비용·timeout원인을확정하지않는다. 실제최종보고는 docs/verify/evidence/WP-08-main941-CI-FAIL/diagnosis-final.json에보존했다. 추적 제품/fixture 수정·새커밋·새독립검증은 다음 RUN의 명시 재개 전 하지 않도록 전달했다. 진단 채팅을 종료/삭제하거나 중복 생성하지 않는다. CLI exit는 해당 없음으로 유지한다. [registry](docs/plans/evidence/M1-r1/app-threads.json)와 최신 인계 snapshot에는 실제 상태를 기록했다. 모델·인증·승인·sandbox·네트워크 설정 변경·전역 하네스 활성화·공개 배포·태그·비용 실행은 없다. 미병합 브랜치/worktree를 삭제하지 않았다.

## 최신941 CI 실패와 다음 진단

일반37522389014/client112470811853은 M036 PASS·M1 Chromium/WebKit80/82 PASS, native/headed Firefox 후속은 skipped다. DPR preferred는 완결4steps의 buffer/PNG/rawGL/RGB77,111,155·camera/authority가 정상이고 다섯 번째PNG도 생성됐지만 line56의 추가 관측에서 전체30초 deadline이 끝났다. forcedGL은6steps 전부 정상이며 finally canvas0/resources(media/listener/observer/RAF)0/errors[]인데 line59 cleanup poll에서30초가 끝났다. 수치 불일치 assertion은 보고되지 않았으며 계측비용 누적 가능성을 진단 중이다. 원본에 단계별 시간이 없어 CPU/driver 내부 원인·제품결함/순수fixture를 확정하지 않는다.

[원본 CI 증거](docs/verify/evidence/WP-08-main941-CI-FAIL/)의 artifact11440549215는2,781,949byte·169members, API digest와 ZIP SHA2a3cd2e7231b769c3dda8ae4bf79b3133000d25301429a7533ea0b9912b98a0a가 일치한다. 부모 source6파일 wrapper ZIP/각SHA 불일치0. 기존 P05와 source79b/5602 success는 소급 변경하지 않고, 최신941 FAIL을 녹색으로 쓰지 않는다. 후보 개선은 같은 시점의 DOM/buffer/authority/resource 관측을 묶어 protocol 왕복을 줄이는 시험 구현이며 아직 적용하지 않았다. 6번 연속DPR·각PNG/rawGL/buffer/oldcontext/camera/authority·최종cleanup·30초/5초poll/retry0/skip0 강도를 보존한다.

WP08 [첫 f58 FAIL](docs/verify/WP-08.attempt1.md)은 valid-CORS metadata/index 다른 origin redirect 허용, [87 PASS](docs/verify/WP-08.attempt2.md)는 수정된 HTTP 경계의 당시 Windows 범위다. 이어 main791 Linux Firefox18FAIL/81PASS·gl=null 정리예외를 별도 보존했다. branch2134 순수 native capability probe는 Firefox headless지원없음·같은buildXvfbheaded RGBA를 확인했으나 앱PASS가 아니다. 임시 probe workflow/script는 최종 tree에서 제거됐다.

[2c FAIL](docs/verify/WP-08.attempt3.md)은 Windows175/M060·unit130을 통과했지만 CI argv 파싱과 WebKit resize 캡처·GPU pipeline 실패 누락을 재현했다. CI 한 줄 수정62는 Linux106·세CI·actualM1세OS hash를 통과했으나 [62 FAIL](docs/verify/WP-08.attempt4.md)은 독점19438 resize 재현과 동일 제품 GPU 원본 대조로 전체FAIL을 유지했다. Three pipeline 내부 오류가 compileAsync resolve로 숨겨져 빈 지도를 성공으로 표시하는 경로를 별도로 수정했다. 최종 9fbc05a1e29062f93bb362450aaddc2068aa41e9의 새 전체 독립 [전문](docs/verify/WP-08.md)과 raw red→green을 완료 근거로 사용한다. 기존 FAIL/PASS/CI 원본은 소급 변경하지 않는다.

[953 FAIL](docs/verify/WP-08.attempt5.md)은 F02/F03 수정 뒤 Windows195/M060과 실제 GPU pipeline 실패·fallback을 통과했지만 Linux resize 30초 deadline과 CSS 고정/DPR-only 6경로 회귀가 남았다. 1↔2 DPR에서 자동 drawing buffer는 이전 크기에 머물고 Reset 뒤에만 갱신됐다. 기존 250ms 전환 캡처와 카메라 전환 중 값은 보존하고, 같은 canvas epoch·안정된 첫 프레임의 PNG/GL 픽셀 판정과 구분했다. verify5의 raw Git index 바이트 변경 원인은 미확정이며, semantic index·4041파일SHA·HEAD·목록·diff/status가 같았다는 사실만 기록했다. 새 verify6의 실제 불변 결과는 별도 전문과 부모 after snapshot에 있다.

a15 임시 Linux 진단은 원래 케이스·30초/expect5초·픽셀 단언을 유지했다. full77/78 실패와 isolated2 PASS의 원본 trace를 대조해 bbox와 canvas의 반복 교체가 없고 deadline 이후 teardown에서 ElementDetached가 발생함을 확인했다. CPU·driver 내부 원인은 입증하지 않았다. 별도 normal a15의 두 afterEach 오류는 진단 fixture의 undefined path 오류이며 제품 픽셀 실패와 분리했다. 원본459MB artifact는 ignored 경로에 유지했고 선택 trace와 API digest/SHA를 추적 증거로 보존했다. 임시 workflow/config/fixture/import는 최종 tree에서 제거됐다.

최종9fbc는 기존 RAF에서 DPR 변화를 관측하고 compile 완료 시 최신 viewport/DPR를 적용한다. 기존 resize/pipeline/지도 픽셀 기준은 그대로 유지했다. M1 실행 일정만 workers1로 바꾸고 DPR 사례를 추가했다. 새 기본 Windows205·Linux124 전체 결과를 판정에 사용했으며, 이를 물리 모니터 이동·다중 사용자 성능·CPU 내부 원인의 검증으로 확대하지 않는다. 기존 locator screenshot이 DPR을 재설정하는 관측과 실패 원본은 남기고, DPR 독립 캡처는 native CDP full viewport로 수행했다. Firefox/WebKit의 동적 DPR은 미지원 범위이며 정적 DPR2 결과와 구분한다.

runtime 감사: 구현자가19437 서버 bind10048 실패 후 기존 verify4 서버에 UI 입력한 source주장은 철회됐다. 최초nativecase는 충돌 전, 최종resize근거는 새19438/PID9344·시간·exe/servedJS/packidentity로 한정했다. 새 최종검증 current19453/19454·old19455~58 및 본인 추가 포트는 독점 own PID·servedJS·input epoch를 확인했다. 파일불변성과 runtime 독립성은 따로 검증한다. 부모 준비파일 오류 출력이 첫verify2에 전달된 이력은 즉시 전문 정정·초기 읽기만·tracked무변경으로 기록했다.

부모가 생성한 중복 증거 사본 삭제는 자동 승인 검토가 blocked by policy로 거절했다. 삭제하지 않고 가역 대안으로 `.orchestrator/evidence/WP-08-attempt3-generated-duplicate-0e294ea/`에 폴더 전체를 옮겼다. 추적 파일·참조0과 전후4,791파일/272,351,325byte/각SHA exact를 확인해 parent-duplicate-relocation.json을 보존했다. 필요한562파일은 attempt3-final 원본 ZIP/SHA에 따로 보존했다. 실행 중 옛 preview 종료도 자동 검토가 차단해 재시도하지 않고 새 포트로 확인했다.

## 실제 화면과 한계

별도 [UI 탐색](docs/play/M1-r1.md)은 f58에서 실제 지도4모드·선택/호버·팬줌·나라/주패널·원장·ko/en·시간5단계/정지·키보드·좁은화면을 직접 조작한 관측 범위 PASS다. 자동 scripted GUI/fault injection과 별도다. 저장 UI는 없어 실행하지 않았다. 속도5의 급격한 날짜 진행·범례·국가 선택 의미·좁은 화면 스크롤은 사용성 의견으로 남겼다.

현재 대표 [미리보기](http://127.0.0.1:19461/)는 source9fbc/PID21824·실행파일 SHA0f35105daf5c2e9492637cd7de9fc1d0ebc0866a64d22ec7aa2993f310231ea2·제공JS1170783byte/SHAd3707b855fe8bbc8d39ce319bbd17685b3e228396791634e91be62b00d86a5de다. parent/CEO의 exe·PID·args·JS/dist 일치와 [원본40파일/직접IAB 화면](docs/verify/evidence/WP-08-preview-9fbc/README.md)을 보존했다. 정지tick6/2000-01-01 6시에서 10/30·ko/en·네모드·원장1/0·팬줌Reset·탭viewport1100×800→1280×720 동선을 직접 확인했다. 원시 상태 주입이나 물리 창/DPR 시험으로 기록하지 않는다. 기존19418=f58/PID19656,19425=87/PID32876,19431=2c/PID8836,19451=953/PID26488은 과거 preview로 유지한다. 각 exe/servedJS/hash/date/state와 캡처를 구분했다. 앱 open 요청 queued만으로 실제 탭 표시를 주장하지 않았다. 합성 Testland이며 실세계/전세계 품질·성능·실제 Safari/Apple hardware·WP11 저장·전체M1 게이트 증거로 확대하지 않는다.

## 마지막 로컬 검증

| 명령·검사 | 결과 |
|---|---|
| 새 WP08 독립 전체 | Windows M1 205/M0 60·unit130/Rust80 PASS, native lifecycle10·pipeline6·compile-DPR2·malformed30·redirect120·독립PNG72 PASS, Linux 실제M1 124/M0 36 PASS. 원본2040 ZIP67,839,746byte/SHA d21b1b80cc9e52821892066abef1669d9898718a7f99c10926fe179bea148bbe 각sourceSHA 일치0 / actual 양backend pixels·국경·CORS외부GET0·resize·pipeline reject·cleanup·원자성·TS·lifecycle·licenses/assets/docs |
| 부모 P07 통합 | main ebc1c5b의 16검사 모두 exit0·후보와 제품221 Git blob 동일. [원문](docs/verify/evidence/WP-08-final-integration/summary.json)과 [sameHEAD mainCI/actual3OS](docs/verify/evidence/WP-08-final-main-ci/same-head-ci.json) |
| M0 365/seed1 hash2 | b039d35666b77fc2 두 번 동일 |
| actualM1 365/seed1 hash2 | b595dc2a1e5b4f8c 두 번 동일 |
| actualM1 1000틱·세 OS 원본 ZIP/API digest | 60448355cecffa9d 모두 동일 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D10/D12 | REQUEST0001 | 2026-10-06 | 사용자 확정 반영 GPL3later/직접dataCCBYSA4/OpenHOI4 | 없음 |
| OPEN01/03/04/05/11 상세 | REQUEST0002 | 2026-10-06 | M1~M3·기존 범위 상세 후보 준비 | 미완성 M4/M5 공식 |
| OPEN02 상세 | REQUEST0003 | 2026-10-06 | 간소화 평화 후보 준비 | 미완성 M4 점수/참가/요구처리 |
| OPEN07/08 상세 | REQUEST0004 | 2026-10-06 | 기상·점령·첩보 후보 준비 | 미완성 M4 공식 |
| OPEN12 상세 | REQUEST0005 | 2026-10-06 | 업적 후보 준비 | 미완성 M6 기록/조건/mod/multi |

M1 결정 대기 없음. OPEN01~12 개별 답과 포함 범위는 확정 반영했고 일괄 기본안 승인은 철회됐다. REQUEST0002~0005는 준비방식A만 CEO 잠정 채택, 현재 미완성 공식은 미승인이다. 승인 범위의 가역적 구체안은 입력/거부/상태변경/출력/경계·저장호환·되돌림 비용을 완성해 CEO 검토 뒤01에 잠정 기록하고 구현한다. 확정 상태·범위·복구 어려운 삭제·설정·비용·공개·법적 판단은 사용자 전용이다. 승인 확장은 최종M6에서 후순위 표기만으로 누락하지 않는다. 함선 설계·공개호스팅·계정은 시점미정 후속 로드맵이다.

## 알려진 문제

| 문제 | 범위 |
|---|---|
| 최신941 DPR CI 두 사례 timeout | 원본6완결/4완결 관측과30초 deadline 구분, 기존worker 읽기 진단 인계. 최신main 녹색 아님 |
| 실제 저장/재개·validate/AIbench/repro 일부 명령 | 후속 WP이며 실행했다고 쓰지 않음 |
| Windows WebKit 폰트 두께 | glyph/loaded/axis 회귀 통과, 실제Safari 미검증 |
| 일부 M0 ASSETS/deny 설명·workspace 라이선스 metadata | D10 미정 시절 문구 후속정리, 현재 정책은01/02/README 확정 |
| 이전 자동worktree/제외target와 중복 생성 사본 | 보존·미추적 잔여를 삭제 우회하지 않음 |

지리 자료는 취득 단계 고정 로컬파일·URL/date/version/license/SHA로 굽고 런타임 외부지도API/tiles/CDN은 금지한다. DEM 능선/고개/강 자료 offline생성·overlay검수, NE/OHM 라이선스 예외·ETOPO CC0 후보 조사는 후속WP32/45며 아직 원천 취득·세계산출 완료가 아니다. 원작 공개화면은 사용자가 허용한 분할원칙/밀도 참고만, 경계복사/트레이싱/파일추출/수치표는 금지한다. 이 지시로 새이동/전투효과를 만들지 않는다.

## 다음 세션이 처음 할 일 (3개 이내)

1. 최종 registry/main HEAD·최신CI·941 실패 원본과 기존 WP08 읽기 진단 채팅의 실제 상태를 확인한다. 같은 채팅을 이어받아 단계 시간·fixture 비용/제품 경계를 대조하고 필요한 P06·새독립검증·최신main CI를 먼저 정리한다. 종료 문서 CI 미완료/실패는 CEO §4 결과와 대조하며 성공으로 가정하지 않는다.
2. 이후 WP11은 docs/plans/WP-11-resume-review.md와 schema-planning/schema-source-review를 읽고 State/TimeConfig/ordered queue/WorldInputs/base/expired modifiers/Defs identity·pack/schema/version·유효 import·원자적 교체·잘못된 format 거부·native process 재개를 field별 기획 후 구현/새독립검증/같은HEAD3OSCI로 진행한다. 현재 지속 RNG cursor가 없으며 seed/date/tick+DR03 stream key뿐이므로 존재하지 않는 cursor를 저장했다고 주장하지 않는다. 위키 본문 접근제한·클린룸을 유지한다.
3. WP11 통합 후 M1 필수14REQ/AC7를 새 독립 게이트로 판정한다. M1 PASS 전까지 M1 완료라고 기록하지 않는다.
