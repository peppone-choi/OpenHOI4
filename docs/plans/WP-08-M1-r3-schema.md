# WP-08 M1-r3 P-06 계약·관측 동등성

기반 `688b3539e2df4ac5451e154e1d379a9ff21fa47b`, 구현 branch `codex/wp08-m1r3-stability`. 대상은 REQ-PLAT-03의 DPR 회귀 검사 비용과 초기 연결 상태의 원인 관측이다. REQ-MAP-04/05의 표시 의미는 보존한다. M1 기본 모델을 최종 전체 국가·주 스키마로 확대하지 않는다.

## 근거와 범위

- exact6b6 일반 CI37543260232는 preferred/forcedGL 전체30초 timeout으로 실패했다. M1 gate는 유효 FAIL이며 source688의 후속 CI 성공으로 소급 변경하지 않는다.
- f19 진단37542615440의 별도 B73/c900 한 회에서는 Chromium 비용이 감소했지만 WebKit 시간은 증가했다. 모든 DPR case는 WebGL2였다. [보존된 보고서](../verify/evidence/WP-08-Linux-diagnostic-attempt3/REPORT.txt)와 원아티팩트11449133117은 제품 채택·GPU driver 원인 증명이 아니다.
- 비교 대상 strict template SHA `c9002e018ad536cc48308973a5d3400d69a8925c9732e936bdef30144ae429ac`의 기술 변경만 정상 fixture에 적용한다. 임시 profile SDK patch/import/workflow/tools는 적용하지 않는다.
- 클라이언트 제품의 renderer/network/App은 입증된 결함이 있을 때만 수정한다. Edge source750의 Connected/Waiting for server/host 없음은 아직 원인이 미확정이다. 저장/Rust/wire 변경은 소유 밖이며 먼저 부모에게 보고한다.

## 필드·수명·권위

다음 값은 테스트가 같은 실제 canvas handle에서 읽는 일시적 관측이다. 신규 게임 상태·wire·HTTP·save 필드가 아니다. 기존 [지도 계약](WP-08-schema.md)의 metadata/dense u16·팩 identity·WorldView 참조/정렬·실패 원자성을 변경하지 않는다.

| 값 | 의미·type·unit·범위 | 필수/default/null·참조 | 수명·권위·REQ |
|---|---|---|---|
| epoch handle | 실제 HTMLCanvasElement 객체 | 필수, locator로 취득; null은 실패 | DPR 한 단계 전후, 표시 DOM; PLAT-03 |
| hostCount/canvasCount | 현재 host/하위 canvas 수, 정수≥0 | 필수, 각각1; 중복 거부 | DOM read 시점, 단계 한정; PLAT-03 |
| connected/current/hostCurrent | 같은 handle이 현재 단일 host의 단일 canvas인가, bool | 모두 true, fallback 없음 | screenshot 전·후 및 rawGL RAF 내부; PLAT-03 |
| backend/dpr | 원 renderer label·실제 devicePixelRatio, string/number | 기존 backend, 요청 DPR와 정확히 같음 | 브라우저 관측, 선택 방식 변경 없음; PLAT-03 |
| box | x/y/width/height, CSS px number | 초기 고정 box와 전체 구조 동일 | DOM rect, 단계 전후; MAP-04/05 |
| camera | 원 dataset JSON의 x/y/zoom/width/height | 기존 saved camera와 전체 구조 동일 | 표시 임시 상태, hash/save 대상 아님; MAP-04/05 |
| viewport | width/height/scrollX/scrollY, CSS px | screenshot 원 viewport 좌표계 | capture 전 snapshot; PLAT-03 |
| buffer | width/height/expectedWidth/expectedHeight, physical px integer | Math.floor(CSS×실제 DPR), 양축 정확값 | 실제 canvas, poll·후snapshot; PLAT-03 |
| resources | media/listeners/observers/rafs, integer count | 원 instrumentation 값; listeners6/observers1, 최종 모두0 | JS 표시 자원만, GPU 메모리 전체 증명 아님; PLAT-03 |
| PNG/rawGL | 원 full decoded RGB / native RGBA·error·lost | 원 좌표·채널±1/error0/lostfalse 그대로 | 같은 epoch screenshot/다음 RAF; PLAT-03 |
| authority | tick/date/hour 및 선택·원장 | 서버 원문, 브라우저 계산 없음 | 기존 DOM expect와 snapshot 기록; NET-05/PLAT-03 |

refs는 canvas→현재host의 객체 관계만 추가 관측한다. 서버 entity ID/dense 순서/owner≠controller/null 규칙은 그대로다. Snapshot/Delta runtime 검증·WorldResult 병합·팩 현지화·MapView mount gate도 변경하지 않는다. rawGL은 같은 canvas가 capture 전과 RAF 내부에서 현재인지 검사하며, 교체·중복·탈착은 실패한다. 실패를 재접속·Reset·sleep·정상값으로 숨기지 않는다.

## 순서·샘플 동등성 표

| 원 단계/단언 | 묶은 관측 | 보존·추가 실패 경계 |
|---|---|---|
| request DPR→poll 실제DPR/buffer→frames/box/camera/mode | 순서와 원 DOM expect 모두 그대로 | 6 Chromium DPR `[2,1.5,1,2,1.5,1]`, 다른 engine native `[2]` |
| old canvas 연결/GL contextLost | 기존 old handle evaluate 그대로 | WebGPU 연결 보존/GL 이전 context loss 의미 유지 |
| capture 전 box/camera/viewport 개별 SDK reads | 같은 epoch의 한 readEpoch | initial box/saved camera와 동일 단언, host/canvas/current/connected도 확인 |
| full native viewport 또는 element PNG→전체 decode→원 sample | 기존 screenshot API/좌표/RGB 그대로 | screenshot 반환 DPR 부작용을 clip 변경으로 가리지 않음 |
| host에서 canvas 재조회 후 RAF readPixels | capture와 같은 epoch handle, RAF 전·내부 membership 검사 | stale/detached/중복 canvas의 성공값 거부 |
| capture 후 실제DPR/buffer→선택/원장/authority DOM expects | 원 순서 그대로 | 원 expect 수·timeout 유지, 원장/서버 시간 정상값 대입 금지 |
| 후 resources/buffer 개별 reads | 같은 epoch의 후 readEpoch | 전후 box/camera/backend/DPR/cardinality 및 buffer floor 단언, listeners6/observers1 유지 |
| reload404→canvas0/lifetime0/authority→finally evidence→context.close | 기존 그대로 | cleanup·close는 기존 본문30초 안; retry/skip0 |

`client/e2e-m1/dpr-observation.ts`는 위 browser reader와 Node 단언을 공유하며, 독립 negative가 실제 DOM snapshot/handle을 통해 같은 단언을 검사한다. 허용된 차이는 SDK 왕복을 줄여 같은 epoch에서 읽고 더 엄격한 membership을 확인하는 것이다. renderer cadence/shader/GPU/backend/카메라를 바꾸지 않는다. 원 full suite의 8개 testMatch/82(Windows5제품205)는 유지한다. negative 파일은 그 목록에 추가하지 않고 별도 지정한다.

## 실행·producer 계획

모든 새 증거는 전용 worktree `target/m1-r3-p06` 절대 경로. 실행 전에 npm pretest의 Rust wire fixtures/proto generate, M0 localization/malformed 및 M1의 8 OH_*_EVIDENCE 기본 경로를 조사하고 절대 ignored 경로로 지정한다. GIT_OPTIONAL_LOCKS=0, transform/TMP 캐시도 ignored 경로에 기록한다. fullM0와 fullM1는 원 옵션·5제품·각1회, 새 own ports19593/19594; negative는 실제 DOM reader/원단언 false-value red→원복green 기록이다. 원 timeouts와 기대값을 성공할 때까지 바꾸지 않는다.

Edge 관측은 이미 있는 원문/source를 먼저 읽고, 필요 시 `target/m1-r3-p06/edge-observation-plan.txt`의 한 회 plan으로 원 adapterReject body/Edge 옵션/5초 기대를 보존한다. WS frame sent/received·runtime shape·HTTP/console/error·최종 UI만 수동 관측하며 wire를 변조하지 않는다. 유효 Snapshot이 UI로 가지 못하는 결함이 입증되지 않으면 제품 경계를 수정하지 않는다.

npmci/unit/typecheck/build·fmt/clippy/workspace·M0hash2/actualM1hash2·필요 저장 회귀·docs checker를 실행하고 실제 exit를 worklog에 적는다. 새 정확한 P05/P07/정상 동일 main CI/P12는 부모 소유다. 계약/hash/save/프로토콜/골든/defines/현지화/의존성/에셋 변경 없음. 기존 실패·미실행을 별도 보존한다.

## 실행 중 확인한 구체 경계

- 원본 PNG 관측 red channel에만 +2를 주입한 native Chromium preferred1회에서 원 ±1 Node 단언이 exit1이었다. 바로 원본 bytes를 복구했다. 공유 helper의 자체 테스트만으로 원 단언을 대신하지 않는다. 주입은 최종 추적 코드에 남기지 않았다.
- 네 가지 실제 DOM/GL negative는 false current, 중복 host/canvas, detach/없는 버퍼의 -1, 다음 실제 RAF 직전 교체를 검사한다. observer 검증이며 game renderer/성능 증명으로 세지 않는다.
- 임시 Edge wrapper의 최초 cwd 및 import.meta 로딩 오류는 본문0회였고 원문을 보존했다. cwd를 원 client로 명시하고 원 dependency/baseline origin을 복원한 뒤 유효 원 adapterReject body 관측은 단1회였다. Hello→Welcome→Join→Snapshot→Query→WorldResult 및 정상 time/host/GL2/PNG를 기록했다. source750의 실패 원인은 미재현·미확정이며 제품 경계를 수정하지 않는다.
- M0 `network.spec.ts`는 환경 변수 없이 `target/wp05/*-browser.json`을 쓰는 기존 고정 producer다. 해당 ignored 경로는 원 fixture대로 사용하고5개 출력의 사본을 전용 절대 evidence root로 수집했다. workspace/native save tests의 고정 target 산출물도 원 경로를 유지한다. 구성 가능한8개 OH evidence·report·cache/log는 모두 전용 절대 경로이며 추적 evidence는 바뀌지 않았다. 이것을 모든 내부 test producer가 환경 변수로 재지정됐다고 기록하지 않는다.
