# ADR-2201 군사 조회 표시와 열린 패널 수명

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-09 |
| 관련 WP·REQ | WP-22, REQ-UI-01, REQ-UI-03, REQ-NET-03, REQ-LOC-01 |

## 맥락

WP-16의 실제 군사 producer와 저장·조회는 인수됐으나 웹 앱은 군사 정보를
표시하지 않았다. 기존 MilitaryView는 관전자를 포함한 전체 국가 범위이며
templates/armies/divisions/jobs/background/pending을 반환한다. 군사 입력이
없는 팩은 기존 generic QueryResult로 unsupported를 반환한다. 현재 조회에는
LedgerView가 없어 REQ-UI-04의 기여 원장을 합성할 수 없다.

## 결정

별도의 읽기 전용 MilitaryPanel을 키보드로 열고 닫는다. 서버의 정상 편제
수치·실제 소유 인력/장비·진행/상태·참조를 그대로 표시하며 정수 문자열과
raw fixed bits를 보존한다. 원래 배열 순서를 바꾸거나 감편 전력·보급 수요·비율을
계산하지 않는다. 표시 국가 필터는 전체 조회에 대한 로컬 표시이며 Join이나
조작 국가를 변경하지 않는다. 정상 편제 목록과 전체 권위 hash/next ID는 필터
밖의 전역 정보다. 군사 명령/편집 UI는 제공하지 않는다.

닫힌 패널은 군사 조회를 발행하지 않는다. 열린 패널은 최초 열림과
Snapshot/Delta에 조회하며 성공한 send 뒤에만 `military:<serial>` 요청을
등록한다. 하나의 pending을 유지하고 추가 갱신은 dirty 하나로 합친다.
일치하는 현재 socket/요청/open epoch의 응답을 소비한 뒤 후속 조회를 한 번
발행한다. 미래·역순·중복·닫힌 epoch·이전 socket 응답과 unrelated generic
응답은 버린다. CONNECTING 전송 실패는 pending을 만들지 않는다.

패널 닫힘은 상세와 필터를 초기화하고 열기 버튼으로 focus를 돌린다.
실제 조작 국가/연결 identity 변경과 재접속은 이전 상세·필터를 즉시 비운다.
연결 끊김/기존 transport guard의 malformed 응답은 마지막 검증 상세가 있으면
stale로 표시하되 요청 수명은 무효화한다. 새 연결의 첫 검증 응답 전에는
이전 상세를 표시하지 않는다. 기존 economyResponses/network/validator/proto와
production 명령 권한 판정은 수정하지 않는다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 항상 전체 군사 조회 | 단순 연결 | 닫힌 화면도 조회하고 오래된 응답 범위를 구분하기 어렵다 |
| 모든 패널 응답 관리 일반화 | 코드 재사용 | 이미 인수된 경제 수명에 불필요한 공유 변경을 만든다 |
| 서버에서 국가별 투영·원장 추가 | 원장과 국가별 payload | server/proto 소유자의 별도 계약·단위이며 이번 범위 밖이다 |

## 결과와 영향

새 의존성·게임 규칙·schema/save/wire/생성 TS 변경은 없다. 기존 Fluent ko/en과
자체 호스팅 CJK 폰트·CSS 테마를 사용한다. 가변 폭 기록은 카드와 줄바꿈으로
390px 화면에 표시한다. 실제 원장 tooltip과 명령 UI·편집기·전체 WP-22/M2
판정은 후속이며 이 기술 결정을 게임 설계의 사용자 결정 상태로 확대하지 않는다.
