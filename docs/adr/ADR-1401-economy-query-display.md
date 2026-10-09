# ADR-1401 — 경제 조회 표시와 응답 수명

작성: 최병호 · 2026-10-09

상태: 구현 후보, 독립 검증 전. 기존 경제 권위·wire·validator·save 계약을 사용하는 클라이언트 표시 단위다. 법령/건설/배분 명령이나 게임 계산을 추가하는 결정이 아니다.

## 표시

서버 `EconomyView`의 국가 정치/인력·선택 법령/배분 비율, 산업 기여·배율·주별 자원 흐름, 건설 프로젝트/원장, 대기 명령과 산업 점수 입력을 읽기 전용으로 보여 준다. 선택 국가에 해당하는 행만 표시하며 없는 국가는 다른 국가로 대체하지 않는다. 국가 선택은 조회 범위 전환이며 `Join`이나 권한 변경을 보내지 않는다.

모든 wire 정수/고정 소수의 `value`/`bits`는 서버 문자열을 보존한다. UI는 IC·배분·원장·점수·인력을 재계산하지 않는다. FixedValue의 정확한 raw 값과 소수 비트 수는 보이는 값의 tooltip에 유지한다. 배분 배열의 위치는 소비재0/건설1/군수2/수출3이다. 숫자로 표현된 기존 u16 국가/주 식별자는 그대로 사용한다. 이름 키는 현지화하고, 카탈로그가 아직 없는 국가/법령 이름은 식별자로 표시한다.

## 응답 수명

`EconomyResponses`는 현재 socket 객체, 선택 국가, 실제 발행한 대기 요청 하나와 dirty 여부를 보관한다. `economy:<n>`의 증가 정수는 bigint이며 연결/국가가 바뀌어도 다시 사용하지 않는다. Snapshot/Delta에서 기존 world/production 조회와 함께 economy 갱신을 요청한다. 같은 socket/국가에서 응답을 기다리면 기존 ID를 유지하고 추가 갱신을 dirty 한 비트로 모은다. 현재 응답을 접수한 뒤 dirty이면 후속 조회를 정확히 하나 발행한다. 접수할 응답은 현재 socket·국가 수명에서 실제 발행한 대기 request ID와 정확히 같아야 한다. 이전 수명의 응답·발행하지 않은 미래 ID·이전 연결 응답·소비한 응답의 중복은 버린다. 메모리는 Delta 수에 따라 늘어나지 않는다. 연속 Delta가 느린 응답의 ID를 계속 덮어쓰지 않는다.

전환 이벤트에서 기존 경제 view와 발행 요청을 즉시 비운다. 새 연결 또는 국가 조회의 유효한 최신 응답만 화면을 채운다. 연결 중 새 조회가 대기할 때는 현재 수명의 마지막 검증 view를 원장 틱과 함께 유지한다. 실제 연결 종료/기존 validator의 malformed 거부에서는 발행 요청을 무효화하고 마지막 검증 view를 stale로 표시한다. 검증 view가 없으면 disconnected다. 재연결 버튼은 기존 host의 새 연결을 만들며 경제 view를 먼저 지운다. 기존 서버의 연결별 독립 simulation 계약을 사용하고 게임/세션 저장 복원을 보장하지 않는다.

최신 unsupported 응답은 view를 비우고 unsupported 상태를 보여 준다. 빈 nations/ledger/project/pending/score 배열과 null industrial_scores는 자료 없음/제공 안 됨으로 표시하며 값이나 행을 만들어 채우지 않는다. 그 뒤 새로 발행한 최신 유효 응답은 다시 표시할 수 있다.

경제가 없는 기존 M0/M1 host는 `EconomyResult` 대신 기존 `QueryResult`로 unsupported를 반환한다. 현재 발행 economy 요청과 같은 ID이고 supported=false/state=null인 이 응답도 빈 unsupported 표시로 접수한다. 일반 time 조회의 supported 상태는 경제 view로 사용하지 않는다. Rust/protocol/validator를 수정하지 않는 호환 처리다.

재연결과 국가 제어 전환은 기존 production view와 그 검증 연결도 즉시 비운다. 기존 서버는 연결마다 새 simulation을 만들기 때문에 이전 simulation의 생산 행을 새 연결에 사용하지 않는다. 기존 생산 명령은 활성 socket이 현재 production 검증 socket과 같고 연결·국가 제어가 유효할 때만 보낸다. 현재 socket의 알려진 production 조회 응답만 검증 상태를 채운다. 이전 socket callback은 기존 activeConnection 검사로 거절한다. 일반 form의 Enter 제출도 명령 전송 함수에서 같은 경계를 따른다.

## 검사 경계

기존 실제 Rust wire fixture로 SSR·정확한 문자열·ko/en 표시를 검사한다. 응답 관리 단위는 역순·발행하지 않은 미래·중복·이전 socket/국가·unsupported를 검사한다. 실제 브라우저는 기존 M2 팩을 새 `<host>/testland`로 복사한 별도 Rust 서버에서 HTTP 번들 bytes·6국 원장·한/영과 전체 정상 WS 흐름을 검사한다.

지연/역순/malformed 검사의 WS 프록시는 실제 서버 응답을 잡아 두고 순서를 제어한다. 서버는 pause 중에도 Delta를 반복 발행하므로 이 프록시에서만 명시적 시간 명령당 한 Delta를 전달한다. 제품의 최신 요청 조건이나 validator를 완화하지 않는다. 정상 흐름 검사는 이 프록시 없이 실제 반복 Delta를 사용한다. 이 검사를 O2 콘텐츠 인수, 독립 QA, 훈련 정책 또는 공유 멀티플레이로 확대하지 않는다.

독립 리뷰 후 추가한 굶주림 회귀 검사는 실제 반복 Delta를 전부 전달하면서 economy 응답만 잡아 둔다. 여러 Delta 후에도 요청 하나만 대기하고 유효 응답을 표시한 뒤 후속 조회 하나만 발행하는 것을 확인한다. 재연결 생산 회귀는 기존 실제 Rust production wire fixture를 UI 제어 대조로 공급한다. 이 M2 economy 입력 자체에는 production 권위가 없으므로 이 회귀를 실제 생산 통합 인수로 기록하지 않는다. 기존 프로토콜·팩·서버 규칙은 유지한다.

## CONNECTING 전송 접수 경계

조회 ID는 현재 socket의 실제 send가 성공한 뒤에만 대기 요청으로 등록한다. CONNECTING/closing/closed 전송은 false이며 ID나 대기 응답을 접수하지 않는다. 재연결 중 국가 선택은 최신 조회 범위를 바꾸고, 실제 OPEN→Hello/Welcome→Join→Snapshot 뒤 그 범위의 조회를 한 번 발행한다. 기존 같은 수명의 dirty 후속 조회도 같은 전송 접수 경계를 사용한다. 별도 OPEN queue나 재전송 목록을 만들지 않아 중복 발행하지 않는다. 기존 호출자는 send의 boolean을 무시할 수 있고 protocol/validator/게임 계산은 바꾸지 않는다.

추가 회귀는 실제 HTTP upgrade만 OPEN 전에 보류하며 모든 Rust 프레임을 그대로 전달한다. 실제 native WebSocket의 CONNECTING0·OPEN1을 검사하고, 보류 중 두 국가 선택→마지막 국가4의 실제 경제 표시→OPEN 후 국가2 전환·고유 발행 ID를 확인한다. 생산 응답 대체 회귀는 여전히 UI 수명 대조만이며 실제 native production 명령 인수는 합성 생산 팩에서 따로 수행한다.
