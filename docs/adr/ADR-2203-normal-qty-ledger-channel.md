# ADR-2203 정상 편제 기여 내역 조회 채널

| 항목 | 값 |
|---|---|
| 상태 | 채택 — 독립 QA·리뷰 및 통합 대기 |
| 날짜 | 2026-10-10 |
| 관련 WP·REQ | WP-22, REQ-UI-04, REQ-NET-03 |

## 맥락

ADR-2202 producer는 실제 checked Qty 합산에서 기여 내역을 만든다.
원장을 모든 MilitaryView에 넣으면 최대4096개 편제에서 메시지가 커지고
기존 client의 정확한 응답 검증을 깨뜨린다. 정상 편제 정의는 tick마다
바뀌지 않으며 실제 사단 감편·보급 소비와 구분해야 한다.

## 결정

Hello/Welcome, PROTOCOL_VERSION, 기존 MilitaryResult·MilitaryView·편제
DTO는 그대로 유지한다. 기존 Query의 kind `military-normal-ledger.v1`은
MilitaryNormalLedgerCapabilityResult를 반환하는 capability probe다.
지원 확인 후 `military-normal-ledger.v1:<template>`로 편제 하나만 요청한다.
request는 `military-ledger:<canonical u64>`이며 template은 기존 식별자
문법과64자 제한을 따른다. capability가 없는 구 서버의 같은 request
unsupported-query/QueryResult/null state만 기능 미지원으로 처리한다.
unknown-template·invalid-template·집계 실패·초과 크기는 데이터 조회 실패로
처리하고 다른 편제 조회와 기존 군사 표시를 유지한다.
request 문법 오류는 그 긴 문자열을 echo하지 않고 예산 안의 기존
invalid-message Notice로 거부한다. 유효한 request만 상관 응답을 받는다.

MilitaryNormalLedgerView는 정의 해시·상태 해시·tick·편제ID·7개 필드를 담는다.
세 해시·tick과 집계는 같은 sim thread 요청에서 읽는다. 원장은 상태에
저장하지 않는다. 필드 순서는 strength, soft_fire, hard_fire, defense,
breakthrough, frontage, supply_use다. base는 Qty0, value·기여·누적은
FixedValue I48F16의 decimal/raw bits다. 역할별 정렬 후 occurrence 위치와
`field:role:position`을 유지하며 최대12combat+4support다. 중복 component도
별도 항목이다. 가중 평균3개와 Fx 최소 속도는 이번 원장 범위 밖이다.

client는 정확한 키·순서·기여 참조·고정소수 transport 인코딩을 검증한다.
게임 합산을 다시 하지 않는다. 표시 전 정의 해시, base MilitaryView의
편제 존재, 7개 최종 bits를 대조한다. 상태 해시와 tick은 provenance이며
Delta마다 원장을 다시 요청하지 않는다. 정의 불일치는 원장을 비우고
base 정의 해시가 바뀔 때까지 재요청을 억제한다.
base 조회 실패로 권위가 없어진 경우에는 데이터 요청을 보류하고 같은
request의 원장 응답을 버린다. 빈 hash를 정의 불일치 잠금으로 만들지
않으며, null→정상 base 복구도 generation 변경으로 처리해 새 조회를 발행한다.

## 전송 예산

새 게임 수치나 frozen pack의 defines를 추가하지 않는다. 호스트 옵션
`--military-ledger-budget-bytes`는 이 새 채널의 전체 MessagePack 응답
(type/request/provenance 포함)에만 적용한다. 기본값은 이미 로드된
network.max_message_bytes이며, 최소 오류 패킷 크기보다 작으면 그 실제
인코딩 크기까지 올린다. 명시 옵션은 그 최소 크기 미만·비정규 숫자를
거부한다. 수신 max_message_size와 legacy 응답은 변경하지 않는다.
실제 기존 팩 기본값65536을 사용하며 게임 balance/hash에는 영향이 없다.
초과 또는 인코딩 실패는 같은 request의 ledger-too-large/null ledger로
응답한다. 실패 패킷까지 예산에 맞고 소켓과 base 군사 표시를 유지한다.

## 수명과 표시

별도 MilitaryLedgerResponses가 socket/epoch/선택generation/request를
검사한다. pending은 probe 또는 data 하나이며 최신 선택 하나를 합친다.
A→B→A에서는 첫 A 응답을 버리고 슬롯을 해제한 뒤 새 A를 요청한다.
serial은 close/new socket에도 재사용하지 않는다. CONNECTING의 send 실패는
pending을 만들지 않고 Snapshot/Delta admission에서 재시도한다.
패널 닫기·표시 국가 scope 변경·새 연결은 capability·원장을 비운다.
disconnect는 마지막 검증 원장에 stale을 표시하고 pending을 무효화한다.
구 소켓·중복·미래 request는 표시하지 않는다. 별도 timeout은 MVP에
추가하지 않았으므로 응답이 영원히 사라지면 같은 소켓에서는 새 선택도
기다리며 패널 닫기/재열기 또는 재접속으로 새 epoch를 시작한다.

별도 Qty 컴포넌트는7개 서버 값과 기여·누적값을 그대로 표시한다.
native details/summary는 키보드와 mobile에서 열 수 있고 ko/en 역할·연산
문자열을 쓴다. component ID는 code로 표시한다. 선언된 supply_use는 현재
부족분 또는 실제 사단 소비라는 의미로 확장하지 않는다.

## 검증 경계

단위 검사는 actual producer 투영·상태 불변·기존 응답 키 보존과 최대
occurrence/64자ID/큰 Qty를 검사한다. 실제 서버와 그 서버가 제공하는 JS
bytes를 확인하는 Chromium 검사와, 지연/구서버 응답 주입 수명 검사를
구분한다. 새로운 client+실제 baseline 서버도 별도로 검증한다.
이 변경은 명령 UI·편제 편집·weighted 평균 원장·전체 WP-22/M2 인수가 아니다.
