# M1 게이트 리포트 — 독립 P-12

**판정: FAIL.** AC-M1-01~06은 검사 범위에서 PASS다. **AC-M1-07은 정확 main HEAD의 일반 CI 실패로 FAIL**이다. Windows 전체 브라우저 시험 성공과 저장 결정론 PASS로 Linux DPR 실패를 상쇄하지 않는다. 현재 M1을 막는 사용자 결정은 없다.

| 항목 | 값 |
|---|---|
| 검증자 | 새 독립 Codex 검증 세션, 구현 비참여 |
| 대상 | detached `6b6abd79cbda5e4cf30b3d059c2c2f981d63ac9c` |
| 지정 cwd | `E:/openhoi/.orchestrator/wt/M1-r2-gate` |
| 최종 감사 | 2026-10-07 08:02:57 KST |
| 파일 불변성 | 부모 최초 baseline과 HEAD·raw/semantic index·추적 목록·9,659개 파일 SHA256·diff/cached diff/status 동일 |
| 추적 파일 수정·커밋·새 세션 | 없음 |
| 게이트 전문 저장 | 하지 않음. 이 메시지를 부모가 저장 |

증거는 [final-audit.json](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/final-audit.json), [명령·종료 코드](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/commands.jsonl), [증거 manifest](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/evidence-manifest.json)에 있다. manifest는 경로·길이·SHA256 1,621건을 기록하며 node_modules·빌드 캐시 전체를 포함하지 않는다.

## AC별 판정

| AC | 판정 | 직접 확인한 근거와 한계 |
|---|---|---|
| AC-M1-01 | **PASS** | `oh_data` 지도 테스트 실행. 8×6 비트맵, 안정 ID `[10,20,30,40,50,60]`, 육지·바다·호수·지형, dense index, 정렬된 4방향 인접 골든과 river/strait/impassable 덮어쓰기 확인. 육지의 정확히 한 주 소속, 인구·자원·건물·인프라와 잘못된 참조 거부 확인. |
| AC-M1-02 | **PASS — 명시된 기본 동작 범위** | 본인 빌드의 단일 Rust 서버 실행 파일에서 HTTP/WS로 접속. Windows 5제품 전체 시험에서 정치·통제·지형·주 모드, 실제 WebGPU/WebGL2 픽셀, 세 국경, 선택·호버·팬·줌·Reset 통과. 3종 이상의 모드 스크린샷과 국가/주 화면을 확인했다. DPR 플랫폼 회귀와 과거 Edge 초기 상태 실패의 해결 판정은 포함하지 않는다. |
| AC-M1-03 | **PASS** | 실제 M1 국가/주 Query·패널, owner≠controller, 원장 기본값·최종값·기여·출처·tick 확인. 서버 rawbits 계산·재계산·overflow 원자성 테스트와 현재 `national.spec.ts` 5제품 실행 통과. M0의 HTML 원장 fixture만으로 판정하지 않았다. |
| AC-M1-04 | **PASS** | 실제 M1 국가/주/프로빈스·큐·modifier/expiry가 있는 48틱 저장을 종료 후 새 native 프로세스로 로드하여 48틱 재개. 연속 96틱과 **전체 구조 및 hash 동일**. 실제 CLI 2일 재개, 같은 HEAD의 세 OS 저장 artifact·compare·각 OS saved-server Query까지 확인했다. |
| AC-M1-05 | **PASS** | 원래 M0/M1 전체 시험에서 ko/en 전환과 실제 자체 호스팅 Noto Sans KR 로드 확인. 5제품 font loaded/check, 적용 family, HTTP bytes·SHA·OFL 고지와 동일 origin 요청 확인. 한국어 화면을 직접 열람했다. |
| AC-M1-06 | **PASS** | 정지·재개와 속도1~5를 실제 서버 명령으로 검사. `req_time_02_tick_day_month_snapshot_order` 등 scheduler 테스트 통과. 월 경계의 고정 phase 순서, 일반 tick·일 처리, 국가/sequence 큐 순서와 저장 후 config·paused·speed 보존 확인. 미구현 gameplay slot의 효과까지 구현됐다는 판정은 아니다. |
| AC-M1-07 | **FAIL** | 기존 담당 WP의 독립 PASS 및 통합 blob은 확인했으나, **같은 HEAD 일반 CI가 M1 DPR 두 건으로 실패**했다. REQ-PLAT-03은 게이트에서 FAIL이며 후속 Linux Firefox 두 단계도 미실행이다. 같은 HEAD Simulation CI의 macOS·compare는 마지막 API 관측에서 진행 중이었다. |

현재 스크린샷·수치 증거는 [M1 E2E](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/e2e/m1), [M0 현지화 E2E](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/e2e/m0), [DPR 관측](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/dpr-observations.json)에 있다.

## 필수 14 REQ 대응

아래 PASS는 해당 기능 검사 결과다. §15의 전체 통합 완료 조건은 AC-M1-07 FAIL로 충족되지 않는다.

| REQ | 담당 독립 증거·이번 직접 검사 | 판정 |
|---|---|---|
| REQ-TIME-01 | WP-04/05, 실제 서버 정지·속도1~5, scheduler·저장 config | PASS |
| REQ-TIME-02 | WP-04, tick/day/month 순서·정렬 큐·실제 M1 hash2·저장 재개 | PASS |
| REQ-MAP-01 | WP-07, RGB 정의·비트맵·안정 u16 ID·수역·지형 테스트 | PASS |
| REQ-MAP-02 | WP-07, 인접 골든·명시 덮어쓰기·순서 보존 | PASS |
| REQ-MAP-03 | WP-07, 주 필드·육지 단일 소속·registry와 참조 거부 | PASS |
| REQ-MAP-04 | WP-08 exact9fbc P-05, 현재 양 backend 모드·서버 RGB·부분 갱신 | PASS |
| REQ-MAP-05 | WP-08, 세 국경 RGB/굵기·ID0/65535·dense256·선택/호버·카메라 | PASS |
| REQ-NAT-01 | WP-09, 국가 정의·수도/정부/지지율·주 Query·패널 | PASS |
| REQ-SAV-01 | WP-11 exact750c 독립 P-05, 현재 save/contracts/relations/file 검사 | PASS |
| REQ-SAV-02 | WP-11, 현재 새 process 전체 구조·hash 재개 및 exactHEAD 세 OS 비교 | PASS |
| REQ-PLAT-03 | 현재 Windows 실제 backend 검사는 통과. **exact main Linux DPR preferred/forcedGL timeout과 후속 Firefox 미실행** | **FAIL** |
| REQ-NET-02 | 단일 exe 임베드 정적 HTTP·WS·지도·현지화, 현재 실제 saved-server | PASS |
| REQ-LOC-01 | WP-12·현재 Fluent runtime·ko/en·국가/주/원장 전환 | PASS |
| REQ-LOC-02 | WP-12·Noto Sans KR 실제 로드/bytes·OFL·ASSETS | PASS |

WP-08의 유효 독립 PASS는 `9fbc05a1e29062f93bb362450aaddc2068aa41e9`다. 현재 `client` 전체 blob은 그 커밋과 동일했다. WP-11 유효 독립 SAV PASS는 `750c2732b00c164ddc252fa26c4984ababd64f35`다. **750c→c2e8→배정6b6의 지정 247개 비문서 blob을 Git tree로 직접 대조하여 모두 동일**함을 확인했다. [247개 대조 원문](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/product247-audit.json).

이전 WP-11 두 검증의 무효/FAIL과 WP-08 과거 FAIL은 유지한다. exact6b6의 WP-11 작업 로그 상단에 남은 초기 상태 문구는 문서 갱신 누락이며, 그 문구나 부모의 완료 주장으로 제품 판정을 내리지 않았다.

## 직접 실행한 명령

모든 shell 명령은 지정 cwd에서 실행했다. Git 읽기는 `GIT_OPTIONAL_LOCKS=0`과 `git -C`를 사용했고 Cargo PATH를 명시했다. Node E2E 자식 cwd는 지정 worktree의 `client`였다.

| 명령·검사 | 실제 종료 코드 | 결과 |
|---|---:|---|
| `npm --prefix client ci` | 0 | 설치 성공 |
| `npm --prefix client test` | 0 | 원래 pretest 포함, 7 files / 130 tests PASS |
| `npm --prefix client run build` | 0 | 현재 production client 빌드 |
| `cargo fmt --check` | 0 | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | PASS |
| `cargo test --workspace --locked` | 0 | Rust 일반·문서 테스트 합계 108 PASS, failed/ignored 0 |
| `cargo build --workspace --locked` | 0 | 현재 exe 빌드 |
| 실제 M0 365일·seed1 두 회 | 각각 0 | `b039d35666b77fc2` |
| 실제 M1 1,000틱·seed1 두 회 | 각각 0 | `60448355cecffa9d` |
| ignored Rust harness의 실제 `oh_proto::typescript()` | 0 | 생성 전체 3,599bytes가 Git blob·worktree와 동일 |
| protocol 변조1byte copy 대조 | 검사 성공 | 변조 copy 불일치 확인, tracked 파일 재생성 없음 |
| `check_save_determinism.py capture --out …/target/m1-gate/native` | 0 | native capture/resume 두 회 |
| `save_native.py --capture …/native --out …/server` | 0 | own PID·새 포트·실제 HTTP/WS·JS identity·종료0 |
| exactHEAD 세 OS Save artifact local compare | 0 | bytes·구조·hash 동일 |
| exactHEAD 세 OS Core artifact local compare | 0 | `0dd81b8754bcc3f9` 동일 |
| 저장 증거 거부 unittest | 0 | 8 PASS; 가짜 OS label 시험은 OS 실행 증거로 사용하지 않음 |
| 원래 M0 full E2E | 0 | Chrome/Edge/Chromium/Firefox/WebKit **60 PASS** |
| 원래 M1 full E2E | 0 | 같은 5제품 **205 PASS** |
| 별도 표준 zlib PNG 디코드 | 0 | Windows/Linux 실제 PNG 8건 표시 RGB 일치 |
| docs/assets/architecture 검사 | 각각 0 | docs 0오류·0경고, assets 0오류 |
| npm license 검사 | 0 | 103 dependencies, 0오류 |
| 최종 전체 SHA·raw index 직접 대조 | 0 | 부모 최초 baseline과 동일 |

E2E는 원래 config·순서·단언·GPU 설정·30초 timeout·5초 expect·retry0·skip0을 유지하여 **suite별 단 한 번** 실행했다. 여덟 `OH_*_EVIDENCE`를 각 suite의 새 ignored 절대 경로로 전달했다. npm pretest·Vitest·workspace 테스트의 고정 `target/wp05`, `wp09`, `wp12`, `wp04-test-fixtures`, `wp11-native-red`, `client/dist`, `client/test-results`도 ignored·tracked0을 확인했다. 추적 경로인 기본 WP-12 evidence 출력은 사용하지 않았다. [Effective-output-paths](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/Effective-output-paths.md).

보조 출력의 CP949 오류와 최종 요약 helper의 누락 key 오류가 있었다. 원명령 exit와 원문을 보존했고, UTF-8 읽기 및 별도 직접 Git/raw/전체 SHA 비교로 감사를 마쳤다. 제품 시험 재실행이나 baseline 교체는 하지 않았다.

## 정확 HEAD 정상 CI·artifact 원문

GitHub API와 실제 artifact ZIP을 직접 조회·다운로드했다.

| Workflow | 같은6b6 결과 | OS·compare 관측 |
|---|---|---|
| [일반 CI 37543260232](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260232) | **FAIL** | 서버3OS·licenses·assets·docs SUCCESS, client FAIL |
| [Core 37543260267](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260267) | SUCCESS | 세 OS 및 compare SUCCESS, 실제 ZIP/hash 직접 대조 |
| [Save 37543260295](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260295) | SUCCESS | 세 OS 및 compare SUCCESS, 실제 저장·새 process·saved-server 확인 |
| [Simulation 37543260312](https://github.com/peppone-choi/OpenHOI4/actions/runs/37543260312) | **마지막 관측 in_progress** | Linux/Windows SUCCESS 및 원본 artifact 확인. macOS 완료·세 OS compare는 미확인 |

일반 CI는 M0 **36 PASS**, M1 Chromium/WebKit **80 PASS / 2 FAIL**이다. preferred·forced GL 모두 DPR 시험의 전체30초 deadline에 실패했다. 뒤의 Firefox native capability와 Xvfb full M1 단계는 skipped다.

일반 artifact `11449149678`은 2,832,955bytes·170members이고, API digest와 실제 ZIP SHA256이 다음 값으로 일치한다.

`511880e4e1fd7c66ff742567e10c219a48230309f8257c6da573bccf7ad07372`

Save artifact의 API digest=실제 ZIP SHA256도 각각 확인했다.

| OS | Artifact ID | ZIP SHA256 |
|---|---:|---|
| Windows | 11449772028 | `c7bb0b32a5b738337466a976ffe802de2e8f40d07b65b238c011605307bcb288` |
| macOS | 11449408007 | `54ea95d6aed6254f6917d75253cb63a577bb84a2048cb84bd09e4896be421d4b` |
| Linux | 11449063147 | `cf54f550af1807cb383dad68e8a83369484e46aae0cf0ce4a3308d12462a4f20` |

Core 세 OS 및 Simulation 관측 가능한 Linux/Windows ZIP의 digest·SHA·실제 명령도 [API 원문 디렉터리](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/api)에 보존했다. Simulation의 현재 세 OS 완료를 주장하지 않는다.

과거 c2e8의 네 CI 성공은 통합 당시 역사다. 배정6b6의 실패를 대신하지 않는다.

## 저장 계약·결정론

현재 직접 실행한 native fixture는 split tick48, continuous tick96이다. 각 capture와 restart PID가 달랐고, 로드 직후 DTO가 split 전체 구조와, 재개 DTO가 continuous 전체 구조와 일치했다.

| 항목 | 확인 값 |
|---|---|
| 저장 fixture SHA256 | `57b13c62ef1a6f88ade1851d0b3ae398157f95ca5bc246068eac3b2a964322fa` |
| 저장 직후 hash | `4eb703aad086c7aa` |
| 새 process 재개·연속 hash | `d19d028857bb7485` |
| canonical SHA256 | `e7376ea46cfaee09b976b97c12bc6f082e94bd6000f97ef013840b0675ee151c` |
| 팩 identity | `b27b821484f4c43b` |

현재 fixture와 독립 원문을 스키마/ADR/소스에 대조했다.

- OHSV·v1·길이·엔진/시나리오·팩 ID/version/hash·날짜/tick/seed/state hash·definitions/effective defines identity를 검사한다. 정의는 로컬 팩에서 재구성한다.
- 국가·주·프로빈스, 물의 null, 소유/통제, config, ordered queue, base/modifiers/expiry, 원장 행 전체 i64 rawbits와 적용값을 보존한다.
- current fixture의 rawbit1 modifier는 tick48에 활성이고 tick49부터 제외된다. 만료된 modifier 입력은 남고 원장과 적용값이 바뀐다.
- current 큐는 `(tick,nation,sequence)` 순서다. time 명령의 nation은 정렬 namespace이며 정의 국가 참조로 임의 해석하지 않는다.
- 잘못된 import는 live/큐 교체 전에 거부된다. 손상·trailing·미래 포맷·잘못된 관계·원장 위조·중복/overflow와 파일 write/flush/sync/persist 오류 검사를 실행했다.
- commit 전 파일 실패는 이전 파일/상태/큐 보존, commit 후 directory sync 오류는 committed warning으로 구분한다. 오류 주입 시험은 실제 전원 손실 실험이 아니다.
- `--force`는 정의·defines·상태 유효성/hash가 일치하는 팩 content/version 차이의 제한적 경고 수용이다. 손상·잘못된 참조·포맷·엔진 불일치를 우회하지 않는다.
- **invalid pending checkpoint 제한:** live enqueue에 잘못된 speed 명령이 남은 순간은 `InvalidQueue`로 저장/export가 거부된다. 명령을 조용히 지우지 않으며 상태/큐를 보존한다.
- seed는 저장한다. **존재하지 않는 persistent RNG cursor를 저장했다고 주장하지 않는다.**

기존 exact750c 새 독립 P-05의 비기본 config·안정 ID0/65535·28+33틱 별도 fixture·정수 postcard/FNV 원문도 읽었다. 그 1,277개 컨테이너·70개 DTO/헤더 검사를 이번 세션에서 새로 실행했다고 쓰지 않는다. 이번에는 동일 제품 blob의 기존 검사 재실행, 새 native 저장·구조 비교, exact6b6 세 OS 원본을 직접 확인했다.

본인 saved-server PID21080은 종료0이었다. exe SHA는 `3098acd9f28f5d3bc8f6bcaf5df00f30061b774ae8544c3c2df9d3e9a91ec187`, served JS SHA는 `d3707b855fe8bbc8d39ce319bbd17685b3e228396791634e91be62b00d86a5de`이며 본인 dist와 동일했다. 각 OS의 실제 saved-server도 Query exit0·server exit0·팩/JS identity를 확인했다.

## 알려진 실패·진단의 한계

| 기록 | 확인·판정 |
|---|---|
| **exact6b6 일반 CI** | preferred는 기록된 DPR5단계까지 색·buffer·rawGL 일치하지만 마지막 단계/cleanup 관측 미완결. forcedGL은 6단계·resource0 기록이 있어도 전체 timeout 결과는 FAIL이다. 누수나 제품 RGB 오류로 단정하지 않는다. |
| f166 일반37540383809 | API·로그·artifact 직접 확인. M036 PASS/M181of82, preferred30초 FAIL·Firefox 후속 skipped. ZIP SHA `3721b97cc096a290c800c6326e2a7eb764edc9eefa03aa9d3ee8f1f42527a1cb`. |
| 941/bbf/b73 | 보존된 원본·리포트를 읽었다. 반복 DPR timeout 및 미실행 Firefox를 과거 PASS로 덮지 않는다. |
| WP-11 verify3 Edge | 원래 full205에서 204 PASS/1 FAIL. 실제 error-context는 Connected·Waiting for server·지도/시간 없음·controls disabled다. 초기 Snapshot 수신/검증/적용 중 원인은 미확정이다. GPU/save 결함으로 단정하지 않는다. 이번 Windows 성공은 해결 증거가 아니다. |
| 임시 f19 진단37542615440 | 원본 API·ZIP/digest 직접 확인. control M036→M182, candidate M036→M182 모두 통과. ordered82 enumeration과 전체10 state dictionary 동일 확인. |
| 임시 후보 비용 | Chromium preferred 17.62→11.04초, RPC638→433; forced15.45→10.76초, RPC596→423. 한 번의 별도 진단이며 control도 통과했다. 제품 원인 해결·후보 채택·새 P-05 PASS·정상 main CI 대체가 아니다. |
| 앞선 임시 진단 두 FAIL | HTML producer 누락과 config/server cwd 준비 실패를 보존한다. DPR 후보 비교 성공으로 소급 변경하지 않는다. |

## 독립성·미실행·후속 결정

최종 raw index 원문은 부모 `before.index`와 byte 동일하다. SHA256은 다음과 같다.

`38cec4cb98b09067a6e47a85246f6894dea953209ef27c739818e757743ed8c7`

[전체 final-after.json](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/final-after.json)과 [raw final-after.index](/E:/openhoi/.orchestrator/wt/M1-r2-gate/target/m1-gate/final-after.index)을 보존했다. 추적 파일 9,659개를 각각 다시 읽어 SHA를 대조했다. 본인 E2E 포트19481/19482는 최종 LISTEN이 없었다. 기존 preview·다른 PID·포트를 사용하거나 종료하지 않았다. 부모의 별도 postcheck를 이미 수행된 것으로 주장하지 않는다.

시뮬레이션 소스 검사에서 f32/f64·HashMap 순회·thread_rng·시스템 시계/I/O 사용을 발견하지 못했다. 전체 DR의 형식적 증명은 아니다. 제품·테스트·골든·timeout·설정을 변경하지 않았다.

직접 UI 증거는 기존 **M1-r1 합성 Testland 탐색**과 **WP-11의 저장 파일을 로드한 새 서버 재개 탐색**으로 구분했다. 이번에는 자동 E2E와 해당 원문/스크린샷을 확인했으며 새 수동 CUA 플레이를 수행했다고 주장하지 않는다.

미실행·미확인 범위는 현재 Simulation macOS/compare 완료, 실패 뒤의 exact6b6 Linux Firefox 단계, Apple Safari hardware, 전세계/실세계 지도, M3 자동저장·저장 버튼·브라우저 파일 업다운로드, 공개 호스팅·계정·비용·배포다. 없는 validate/AI/repro 명령을 실행했다고 쓰지 않는다.

D-10/D-12의 실제 사용자 확정 방향은 문서에 유지돼 있다. **M1 사용자 전용 보류는 없다.** 이번 차단은 기술 검증 실패다. 다음 필요한 조치는 DPR/초기 상태 실패의 소유자 P-06, 새 exact 독립 P-05, 통합 후 같은 HEAD의 정상 CI 및 빠진 범위 확인이다.

후속 OPEN 방향과 미정 세부 규칙도 문서에 보존돼 있다. REQUEST-0002~0005의 장비·보급·공군·해전·특수 무기, 간소화 평화회의, 기상·저항·첩보, 업적 세부 규칙은 해당 M4~M6 구현 전에 필요하다. 함선 설계는 이후 로드맵이며, M6 자가 호스팅·세션 코드·계정 없음과 이후 공개 호스팅/계정 지원을 구분한다. 이 항목들은 현재 M1 CI 실패를 사용자 결정 보류로 바꿀 근거가 아니다.