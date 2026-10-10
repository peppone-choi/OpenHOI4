# ADR-1603: 훈련 명령 권한과 응답 이후 조회

## 상태

issue17의 사용자 승인된 훈련 시작·취소 UI 구현 후보다. 실제 Claude 계획을
받아 인수된 main9cbc93f에서 진행한다. 독립 변경 리뷰·CI·병합은 별도 게이트다.

## 결정

기존 MilitaryCommand의 Train/Cancel만 사용한다. 게임 계산·wire·save·팩은
바꾸지 않는다. ProductionPanel의 기존 국가 시작 흐름이 Join 국가를 정하며,
군사 패널 국가 선택은 표시 필터다. 새 자동 Join·reconnect·세션 reset을 만들지
않는다. 현재 호스트의 각 연결은 별도 로컬 simulation이다.

Time/Production/Military는 하나의 BigInt u64 sequence를 공유한다. 실제
send=true 뒤에만 sequence와 발행 원장에 접수한다. 원장은 최대64개의 미응답
발행만 보존하는 client 기술 예산이며 게임 훈련 한도가 아니다. 현재 socket의
정확한 발행 sequence만 CommandResult로 소비하고 중복·미래·미발행 결과는
폐기한다. 새 연결은 발행 원장을 비우되 증가 sequence를 유지한다.

군사 명령은 scope마다 pending1개다. 실제 Join 국가와 현재 검증된 군사 view가
필요하다. foreign 표시 필터·foreign job·종료 job·missing/unsupported/stale
권위는 명령을 허용하지 않는다. close/filter/player/socket 전환은 이전 명령
권위를 비우며 최신 scope의 검증된 조회를 다시 받아야 한다. filter 전환 시
기존 query pending slot은 유지하고 전환 시점 serial보다 새로운 조회만 명령
권위를 복구한다. 이전 명령을
자동 재전송하지 않는다.

서버는 enqueue 뒤 실제 sim.step의 해당 명령 의미 결과를 CommandResult로
보낸다. 수신 시점의 군사 query 발행 serial을 barrier로 기록한다. 그보다
새 serial의 상관 authoritative MilitaryResult를 받은 뒤에만 pending을 풀고
success/rejected를 표시한다. ACK 전에 발행한 조회가 뒤늦게 도착해도 명령을
다시 활성화하지 않는다. 기존 one-pending/dirty 조회와 Qty ledger generation
수명은 유지한다. 인력·장비·진행은 서버 view만 표시한다.

MilitaryPanel의 controls props는 optional이다. standalone의 기존 읽기 전용
표시는 유지하며 App에서만 버튼·상관 feedback을 연결한다. Ready 배치·군
우선순위·편제 편집은 후속이다. 응답 영구 소실에 대한 timeout/자동 retry는
추가하지 않았다. 사용자의 명시적인 close/reopen·기존 reconnect가 새 scope를
만들며, reconnect의 별도 로컬 session을 저장 복구나 공유 persistence로
표시하지 않는다.

## 검증 경계

실제 UI 클릭 RED/GREEN과 unit barrier/권한/발행 검사를 분리한다. native
훈련·취소·반환과 시간 진행, 실제 HTTP upgrade의 CONNECTING, ko/en·키보드·
mobile을 검사한다. 지연/미발행 ACK·unsupported·close 주입은 표시한다.
실제 localhost HTTP proxy가 pre-ledger client의 static bytes를 제공하고 current 서버의
WebSocket frame을 그대로 사용하는 호환 대조는 frame mock이 아니다. 그
대체를 native embedded legacy serving으로 기록하지 않는다.

전체 게임 QA·독립 runtime QA·전체 WP-16/M2 완료는 이 단위로 선언하지 않는다.
