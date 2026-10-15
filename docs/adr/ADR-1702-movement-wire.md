# ADR-1702: 서버 권위 이동 명령과 조회

## 상태와 범위

S1 구현 후보. PR21 exact head `fb1539adc231e34eccf42895d46d456448bff1dc` 위의 별도 stacked 브랜치다.
PR21 병합 영수증 차단은 유지한다. PR21보다 먼저 병합하지 않고 PR21 병합 뒤 main과 정합/rebase 후 재검증한다.
S2 UI, M-1 지도 로딩, 실제 세계·국가 레이어, 저장 형식과 이동 규칙은 변경하지 않는다.
기존 1936 시작 요구를 유지하며 1939를 전제하지 않는다.

## 결정 근거

ADR-2203은 `PROTOCOL_VERSION`/기존 응답을 보존한 Query 기능 확장의 선례다.
ADR-1603과 기존 MilitaryCommand는 Join 국가, 공용 단조 sequence, 실제 sim 결과 ACK 후 Snapshot의 선례다.
이에 `m0-v1`과 기존 메시지 DTO·바이트는 보존하고 새 tagged MovementCommand/MovementResult만 추가한다.
구 서버는 미지 명령을 invalid-message로, movement query를 unsupported-query로 거부한다.
새 클라이언트는 명령 전에 `movement` 조회의 supported를 확인할 수 있다. client 구현은 S2 소유다.

명령은 canonical u64 division/sequence와 u16 destination만 받는다. 국가·속도·계수·allowed를 받지 않는다.
Join 국가만 sim::Command::Move/Stop의 nation이 된다. 군사 명령과 같은 시뮬레이션 큐·step 경로를 사용한다.
조회는 같은 sim thread에서 state_hash/tick/Unit getter를 읽고 FixedValue I32F32로 투영한다.
현재 간선이 없으면 current_leg_hours=0이다. movement 컨텍스트가 없어도 빈 units로 성공한다.
remaining_hours 계산 실패는 simulation-error로 반환하며 잘못된 권위 값을 만들어내지 않는다.
경로는 현재 province 다음부터 목적지까지이며 Stop은 기존 현재 간선 보존 규칙을 그대로 따른다.

## 검증

기존 wire fixture·save hash·결정론·client e2e는 기존 CI를 그대로 실행한다.
새 protocol 검사 및 separate-process native 검사는 GitHub-hosted CI server 3 OS에서 실행한다.
부분 서버 검사를 실제 지도·제품 QA 완료로 표현하지 않는다. Windows GUI/원작 HOI4/GCP/배포/초기화는 NOT_RUN이다.
