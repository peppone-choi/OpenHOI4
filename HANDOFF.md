# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-07, M1-r1 WP-08 새953 수정 제출 뒤 전체 독립 검증·LinuxCI 중 |
| 현재 마일스톤 | M0 PASS 보존. M1 진행 중 |
| 기본 브랜치 CI | main79140fbfc9966fc5104df735f068615633181d5b의 [일반CI](https://github.com/peppone-choi/OpenHOI4/actions/runs/37491594894) client FAIL·다른6jobs success, [Core](https://github.com/peppone-choi/OpenHOI4/actions/runs/37491595077)와 [Simulation](https://github.com/peppone-choi/OpenHOI4/actions/runs/37491594793) success. 전체CI 녹색 아님. 이전 b1da8f4 세CI/실제M1 세OS SHA/hash60448355cecffa9d 일치 증거는 보존 |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

M1 준비만 하라는 최초 요청 뒤 CEO가 사용자 승인 본 실행을 전달해 같은 실행에서 진행했다. RUN 종료 조건은 M1 게이트 판정·묶음 두 개 통합·결정으로 가능한 WP 없음 중 먼저 오는 것이다. W1 통합 1개를 판정했다. W1은 WP07/10/12, W2는 공용 파일 충돌을 피한 WP09→08, W3 저장 WP11이다. 묶음 두 개가 먼저 통합되면 WP11과 M1 게이트는 다음 RUN이다.

WP07 지도와 WP10 원장 기반은 독립 PASS 후 main에 통합했다. WP12는 malformed Snapshot이 화면을 비게 하는 첫 독립 FAIL을 보존하고 P06을 배정했다. 수정8831613에 새 독립 PASS를 받아 main14fa031에 통합했다. 국가/주 원장 query와 GPU 지도는 아직 후속 연결이며 합성 원장fixture를 전체AC로 판정하지 않는다. 실제 국가/주 상태 hash·save-resume는 M0 시간 hash와 구분한다.

M0 게이트 `d6ae2c8`의 AC7 PASS와 당시 동일 HEAD CI15jobs·7artifactZIP SHA는 재확인했다. [M0 게이트](docs/gates/M0.md)와 보존registry는 유지한다. M0 제품 골든·독립 예제팩은 바꾸지 않는다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP00 | CLI done1 실패 보존 | 과거 M0 기록 | 앱 정상 종료를 CLI0으로 바꾸지 않음 |
| WP01~06 | M0 통합됨 | 과거 worktree 정리·일부 제외 파일 잔여 | M0 증거 보존 |
| WP01 AI정책 후속 | 독립 PASS·main CI 성공 | codex/wp01-reviewed-ai-assets, WP-01-ai-policy | 별도 M1 묶음 횟수에 미산입 |
| WP07 | 통합됨 | codex/wp07-map-data, WP-07 | 데이터검증 범위 PASS, 실제 표시 WP08/09 |
| WP10 | 통합됨 | codex/wp10-ledger, WP-10 | 계산 기반 PASS, 실제 적용·조회 WP09 |
| WP12 | 수정 독립 PASS·통합됨 | codex/wp12-localization, WP-12 | 실제 원장 WP09 후속 |
| WP09 | 통합됨 | codex/wp09-national-state, WP-09; 구현206fb45/mainb1da8f4 | 실제 국가/주·원장·M1 hash/CLI/3OS PASS, 지도선택 WP08 후속 |
| WP08 | 이전2c/62 독립FAIL 뒤 새953 수정 clean 제출·자체195/M060 PASS; 새전체P05·branchCI 중 | codex/wp08-map-rendering, WP-08; verify5 detached953/4041tracked/전용19443·44 | 새 전체 독립 판정→통합main CI; W2 미계수 |
| WP11 | 다음 RUN 예정 | 아직 없음 | 실제 M1 mutable state·예약명령 save-resume |
| M1 게이트 | 미판정 | 없음 | WP11까지 전체통합·독립 새검증 |

## 세션 상태와 실행 경로

사용자가 승인한 앱 별도 local 채팅·WP별명시 Git worktree 경로를 썼다. 모델·인증·샌드박스·승인·네트워크 설정은 변경하지 않았다. 모든 구현 명령은 WP worktree, 검증은 정확한 구현커밋 detached별도 worktree다. 부모도 검증 전후 HEAD/index/추적목록/각SHA/diff/status를 비교했다. 현재 채팅ID/실제상태/경로/커밋은 `.orchestrator/app-threads.json` 및 종료 시 보존복사본을 참조한다. 원본 첫 WP12/WP08 FAIL 검증 worktree는 보존했다. 현재 실행 중인 채팅은 기존 WP08 구현 01a11186-8f53-75c2-b9fb-873b27d8af82의 LinuxCI P06이다. WP08 새검증01a111db-1a89-78a1-b2b8-c8c20c812bfc는87c1a99 Windows PASS와2458파일불변성을 제출하고 종료했으며 main9dcf199에 통합했다. 새 제품 수정에는 새 독립 검증이 필요하다. [보존 registry](docs/plans/evidence/M1-r1/app-threads.json)는 이후 종료 시 갱신할 사본이다. 미병합자료를 삭제하지 않았다. 새 검증 첫 메시지에서 부모가 준비파일 오류 출력을 잘못 전달해 즉시 전문으로 정정했다. 정정 전 main 문서/Git 읽기만 했고 파일 수정·세션 생성·프로세스 조작은 없음을 검증자가 보고했다. 실제 검증 시작은 지정 worktree 2458 tracked clean에서 별도 기록했다.

WP08 f58c0c6의 [첫 독립 FAIL 전문](docs/verify/WP-08.attempt1.md)과 273개 원본 증거를 보존했다. unit130/M1 E2E135/M0 E2E60·실제 WebGPU/WebGL2·픽셀·해시는 통과했지만, valid-CORS metadata와 index redirect가 다른 origin 응답을 받는 실제 결함으로 최종 FAIL이다. CORS가 잘못 구성된 최초 helper 실패와 수정된 실제 재현을 구분했다. 부모1905 tracked SHA/index/list/diff/status 불변 확인. 구현 채팅이 수정하고 새 독립 채팅이 정확한 수정 커밋을 검증한다.

별도 [UI 탐색 리포트](docs/play/M1-r1.md)는 f58c0c6에서 지도/선택/4모드/팬줌/시간5단계/원장/ko-en-ko/키보드/좁은화면을 직접 조작한 관측 범위 PASS다. 전체 WP08 또는 M1 게이트 PASS를 대신하지 않는다. 12캡처·관측JSON·제공JS·전후불변 증거를 보존했다. 속도5의 급격한 날짜 진행, 색 범례·국가 선택 의미, 좁은 화면의 스크롤은 사용성 의견이며 진행불가·크래시는 관측되지 않았다. 저장UI는 없어 미실행이다.

현재 대표 preview는 http://127.0.0.1:19425/ 의 수정87c1a99 단일 실행파일이다. 부모도 PID32876/path/명령·exeSHA7ac204825929972da31b3e88afadfa2e014b7898b046d1bfbf5b1f9713e714a0·servedJS1169414bytes/SHA7c50c1bf1c77e859d3132e2c43ac34db3e056b57a65762260e7593ef3de401b5와 dist 대응을 확인했다. packhash b8d30ba577f303c3/schema1/m1seed1. 구현자의 새 직접IAB 관측은 정지/speed1/tick12/2000-01-01 12시/ko/지형/선택10/원장1이다. 원시상태 주입 없이30남부선택→pan→reset→ko→10/원장 동선을 확인했다. [새 runtime/3캡처와 부모identity](docs/verify/evidence/WP-08-preview-p06/README.md)를 보존했다. 이 직접 확인을 새 독립 PASS로 대신하지 않는다. 앱 open 요청 queued를 실제 탭 표시 확인으로 기록하지 않았다.

기존19418 f58/PID19656 서버는 유지된다. 기존 preview 종료를 포함한 교체 명령은 자동 승인 검토에서 blocked by policy로 거절됐고 구체 사유는 없었다. 종료 재시도·우회·설정 변경 없이 별도 새19425 preview를 시작했다. 첫 f58 플레이의 정지/tick1687141/2192-06-19 및 최초445 unavailable terrain 문구 캡처는 과거 증거로 보존한다. 대표URL과 과거URL을 혼동하지 않는다.

## 마지막 로컬 검증

| 명령·검사 | 결과 |
|---|---|
| W1 통합 fmt/clippy/workspace tests·npmci/test123/build/type/license | 14fa031에서 모두 exit0 |
| M0 365일seed1 hash2 | b039d35666b77fc2 두 번 같음; M1국가hash 증거로 사용하지 않음 |
| WP07 독립 지도오류 | 94개 독립 입력, map11·Rust59(Doctest 포함) PASS |
| WP10 독립 원장계산 | 경계27436·순열120·serde/mismatch114 PASS |
| WP12 새 독립 실제 서버 UI | 5브라우저75검사·wire489+누락41+transport10 PASS |
| assets --release·정책32tests·checkdocs | 오류0·문서경고0 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D10·D12 | REQUEST0001 | 2026-10-06 | 실제답변 확정반영 | 없음; 코드GPL3later/직접데이터CCBYSA4/OpenHOI4 |
| OPEN01/03/04/05/11 상세 | REQUEST0002 | 2026-10-06 | M1~M3·미승인 문서후보 | M4/5구체규칙 |
| OPEN02 평화회의 상세 | REQUEST0003 | 2026-10-06 | M1~M3·간소화후보 | M4점수/참가/요구처리 |
| OPEN07/08 상세 | REQUEST0004 | 2026-10-06 | M1~M3·별도후보 | M4weather/occupation/spy세부 |
| OPEN12 업적 상세 | REQUEST0005 | 2026-10-06 | M1~M5·소수업적후보 | M6조건·기록·mod/multi |

OPEN01~12는 일괄 기본안승인 철회 후 모두 개별 사용자답변으로 범위를 반영했다. 포함 결정과 구체 규칙의 잠정 채택은 다르다. 준비방식 A만 CEO잠정채택이며 현재의 미완성 defines/공식 후보는 미승인이다. REQUEST가 있다는 이유로 모든 세부 공식을 사용자 전용으로 분류하지 않는다. 기존 승인 범위의 가역적 추천안은 구체 입력·거부·상태변경·출력·경계 기대값·저장호환/되돌림 비용을 완성해 CEO 검토를 받는다. 채택 뒤 01에 잠정 규칙을 기록하고 구현한다. 확정 상태·범위·복구 어려운 삭제·설정·비용·공개·법적 판단은 사용자 전용이다. 함선설계와 공개호스팅·계정은 후속로드맵에 보존한다. 공개배포·태그·실제운영비용은 승인하지 않았고 실행하지 않았다. 이름 확정을 법적상표검증으로 주장하지 않는다.

후속 검토 순서: M1을 우선 끝낸 뒤 M4 차량/기상/점령/첩보와 간소화 평화 회의 후보의 계산·경계를 완성하고 CEO의 권한/가역성 검토로 넘긴다. 다음은 M5 장비/공군/함대/특수무기와 스크립트 기술 선택, 마지막은 M6 업적이다. REQUEST0005의 서버 설치 공유 프로필은 다인 플레이에서 타인 기록을 합치는 한계가 있어 기존 세션·국가 배정 기준의 국가 기록과 개인별 표시/보존 대안을 비교한다. 공개 계정을 전제하지 않는다. 최종 M6에서는 기존 후순위 표기와 별개로 승인 확장 목록의 WP·독립 PASS·CI 증거를 대조한다.

## 알려진 문제

| 문제 | 재현 방법·범위 |
|---|---|
| WP12 최초 빈화면 오류 | originala91d324 malformed Snapshot, 첫FAIL 보존·8831613수정·새PASS |
| WP08 HTTP redirect | f58 metadata/index 다른origin render 최초FAIL 보존; 수정87c1a99 새독립 검증중 |
| Windows WebKit 폰트 얇음 | loaded/axis/glyph는통과, 실제Safari 미검증, 화면/제한기록 |
| 일부 M0 provenance/라이선스설명 구식 | ASSETS/deny의 D10미정 당시문구·workspace metadata 후속정리; 확정정책은01/02/README |
| validate/AIbench/repro/완결게임/저장 일부 후속 | 미구현명령을 실행했다고 기록하지 않음 |
| 초기 앱 자동worktree·WP03verify 잔여 제외파일 | 이전 HANDOFF 기록의 미확인thread/제외target 보존, 강제삭제하지 않음 |

사용자 지리 지시 반영: 자료는 취득 단계에 고정파일·URL/date/version/license/SHA로 보존하고 게임 중 외부지리API·타일·CDN을 쓰지 않는다. DEM능선/산맥/고개/강 자료를 offline생성·검수에 포함한다. 원작공개화면은 분할원칙/밀도만참고하는 사용자한정예외이며 경계복사/트레이싱/파일추출/수치표는금지한다. NE/OHM라이선스예외·ETOPOCC0후보 공식자료를 확인했으나 원천취득/채택/전세계산출은 후속 WP32/45다. 새이동·전투규칙을 만들지 않는다.

## 다음 세션이 처음 할 일 (3개 이내)

1. 현재 registry와 WP08 P06 실제 HEAD/상태/증거를 확인한다. 정확한 수정커밋의 새독립 검증에서 metadata/index 외부redirect 요청0·정상 동봉팩/마지막화면 회귀를 판정한다. 기존FAIL과 별도UI플레이 결과를 보존한다.
2. 두 묶음이 통합되면 이 RUN을 끝내고 HANDOFF 커밋·푸시 후 정확한 최종 HEAD CI를 확인한다. 저장 WP11은 다음 RUN에서 실제 M1 state와 새독립 DT/3OS 증거로 진행한다.
3. 전체필수14REQ/AC7를새독립게이트로판정하고 기록. M1이PASS될때까지M1완료라고쓰지않음.

WP09 후속 검증: M1 실제server15/M0server60·독립브라우저110·TOML변조29·Fx최하위bit/expiry/queue/config/다중주실패원자성 PASS. 부모1515 tracked SHA/index/list/diffstatus 무변경. M1 365일seed1 두회 b595dc2a1e5b4f8c, 1000tick60448355cecffa9d. [전문](docs/verify/WP-09.md), [세OS 원본ZIP metadata](docs/verify/evidence/WP-09/main-ci/same-head-ci.json).

WP11 착수 전 [저장 경계 점검](docs/plans/WP-11-resume-review.md)을 읽는다. 현재 Simulation은 seed/date/tick을 보존하며 지속 RNG cursor가 없다. 존재하지 않는 cursor를 저장했다고 주장하지 않는다. State/TimeConfig/ordered queue/WorldInputs/base/만료된 modifier/Defs identity를 field별 계약에 포함하고 새 유효 import·원자적 교체·독립 재개 fixture를 먼저 설계한다. [스키마 기획](docs/plans/schema-planning.md)과 위키 본문 [접근 제한 기록](docs/research/schema-source-review.md)을 따른다.

현재 LinuxCI 실패 증거와 범위: [조사](docs/research/WP-08-linux-renderer-ci.md), [원본CI SHA](docs/verify/evidence/WP-08-CI-attempt1/ci-failure.json). Firefox18FAIL/81PASS·gl=null 정리 추가예외를 보존했다. 임시단발 브랜치한정 capability probe는 최종2c/62 tree에서 제거됐다. 테스트/prefs/security/OS 설정을 우회하지 않았다. 신규 게임규칙·전역하네스가 아니다.

후속 exact2c CI37495546810은 argv 파싱 오류로 Firefox native/headed가 실행되지 않아 FAIL. --project=firefox 한 줄만 바꾼62a89ae의 CI37497228851/Core37497228704/Sim37497228737은 모두 성공했다. 실제 Linux M0 36/M1 70+1+35=106, 원본 ZIP af85ffa4cd2c8e18c94ba008713f8bb4af28b3c46278246708be68332bcd5d9e/API digest 일치, actualM1 세OS ZIP·hash60448355cecffa9d 일치. branch 성공이며 main791 실패를 대체하지 않는다.

[독립3차 FAIL 전문](docs/verify/WP-08.attempt3.md): 2c에서 Windows175/M060/unit130 등은 통과했지만 WebKit resize 후 화면·PNG가 배경색이 되고, 실제 WebGPU pipeline rejection을 Three가 resolve해 빈 화면을 성공으로 보고한다. 부모3128파일불변 확인. 동일제품62의 CI 수정 대조는 verify4, 새제품수정은 기존 구현자가 수행 중. W1만 묶음 완료/W2 미계수/WP11 미시작이다.

최신9531121ee5677e43436be69364fdef4bfce2fe86은 WebGL surface 복원·pipeline error/GL link 확인·resource cleanup/AbortSignal 경계를 수정하고 resize/pipeline 회귀를 추가했다. 기존175 tests·pixel기준·5제품·Rust/data/assets/lock/proto/CI blobs는 그대로다. 자체 최종195/M060·unit130·Rust/type/docs/build 등 PASS를 독립PASS로 사용하지 않는다. 새 verify5 채팅01a11248-9ae6-7410-b958-d6b57baceec3은 detached953의4041tracked clean·빈전용19443/44를 확인하고 전체 P05를 진행한다. exact953 CI37504540992/Core37504541035/Simulation37504540929는 실행 중이다. 새대표19451 preview만 별도 준비하며 제품 추적 변경은 금지했다.

이후953 일반CI37504540992는 client112409705674 FAIL로 종료했다. Linux M036 PASS/M1 Chromium·WebKit78중77PASS와 Chromium resize4mode 캡처 loop의 total30000ms timeout이다. fontsloaded/scrollintoView/elementstable 대기 기록이며 픽셀 불일치·제품 원인은 아직 미확정이다. Firefox native/headed는 이전 단계 실패 뒤 실행되지 않았다. Core/Sim만 성공, 실제M1 세OS ZIP digest/hash60448355cecffa9d는 [Simulation 한정 metadata](docs/verify/evidence/WP-08-CI-attempt3/simulation-only-identity.json)에 보존했다. 전체CI녹색으로 쓰지 않는다. 원본149member ZIP SHA3c4ad1a62c167eeb9fc6235d788275032e3460017fc746a487645d14d3e40fc5는 API digest와 일치했다. 구현자 P06 진단/953고정 독립검증 계속, timeout·단언·browser 완화 없음.

최신9fbc05a1e29062f93bb362450aaddc2068aa41e9가 DPR 기존RAF 갱신과M1 worker1 실행 격리를 제출했다. a15 trace에서 고정bbox·viewport변경당1canvas교체·full 약24초후resize/30초timeout과teardown 뒤ElementDetached·같은단언isolated body28.269초PASS를 분리했다. CPU/driver 내부·권위간섭·동시플레이 성능 원인은 확정하지 않았다. 원본459MB는ignored보존하고 선택member SHA/geometry/report/4trace는 [진단 증거](docs/verify/evidence/WP-08-linux-phase/)에 보존했다. 임시workflow/config/fixture/import 최종tree삭제, 기존resize/원래assert/timeout/일반CI/Rust/data/assets/lock 동일을 부모 확인했다.

exact9fbc [일반CI37513851532](https://github.com/peppone-choi/OpenHOI4/actions/runs/37513851532)·[Core37513851556](https://github.com/peppone-choi/OpenHOI4/actions/runs/37513851556)·[Simulation37513851414](https://github.com/peppone-choi/OpenHOI4/actions/runs/37513851414)는 모두success. 실제Linux M036/M1 C-W82+nativeFF1+Xvfb41=124, artifact11437106228/230member/3419737byte ZIP8e81ca441b9db3a4eeb5d27cbf1e2fffdd7d44308290363d335e8a7554244ddf/API digest일치 및3OSM1hash6044를 부모 회수했다. branch 성공이며 main791 실패·독립최종판정을 대신하지 않는다.

새verify6 채팅01a1128f-0f4f-7661-b480-97741eb87b50 detached9fbc/5008trackedclean/전용19453·54에서 default205/M060·dynamicDPR/fractional/nativecapture·realpipeline failure·원본red62/red953·최신§5.11 실제read-only 사본·원자성·resource/GUI/실제Linux자료를 전체검증 중이다. 최신planning 파일은 본인ignored prior-attempt/latest-planning에 실제복사됐다. CLI M1표기의과거--pack-root는서버와혼동된부모프롬프트오류로정정했으며실제CLI는--pack data/packs/testland이다. W2 미계수/WP11 미시작 유지.

현재 대표 preview는 [19431](http://127.0.0.1:19431/)의 제품2c/PID8836/packb8d30ba577f303c3다. 부모 exeSHA8c80989d67c9f352024c37bd86f05cbdad4bde7622769b0862119cab47d027e7·servedJS SHA8e4fef1d102dbb4035a82396e106dc003d16a7905fa3f878411fa88c17831bac을 재확인했다. [실제 IAB 관측·원시 캡처](docs/verify/evidence/WP-08-preview-2c/)는 pending P06 제품과 구분하며 독립 PASS나 최종 제품 증거가 아니다. 19418/19425는 역사 preview로 유지된다.

포트 독립성 감사: 구현자가19437 서버 시작의 bind10048 실패를 확인하기 전에 기존 verify4 서버에 UI 입력했다. 구현자의 own-source 주장은 철회되고 원문 충돌은 보존됐다. verify4 최초 singlecase는 충돌 전, 후속 resize epoch는 영향 가능 범위를 별도 기록한다. verify3의19428와 구현자의 새19440 재현은 분리돼 있다. 최종 새 독립 검증은 독점 새포트와 실제 PID/servedJS/input epoch를 확인해야 하며 추적 불변성만으로 runtime 독립을 주장하지 않는다.

부모가 생성한 중복 증거 사본의 삭제는 자동 승인 검토가 blocked by policy로 거절했다. 삭제하지 않고 안전한 가역 대안으로 `.orchestrator/evidence/WP-08-attempt3-generated-duplicate-0e294ea/`에 폴더 전체를 옮겼다. 추적파일·추적참조0을 먼저 확인했고 이동 전후4,791파일·272,351,325byte·각SHA가 정확히 동일하다. [이동 기록](docs/verify/evidence/WP-08-attempt3-final/parent-duplicate-relocation.json)을 보존했다. 필요한562파일은 attempt3-final에 별도로 보존됐으며 원본 failed 검증 worktree/미병합 자료는 삭제하지 않았다.
