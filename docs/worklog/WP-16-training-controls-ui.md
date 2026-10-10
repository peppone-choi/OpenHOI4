# WP-16 훈련 시작·취소 UI 후보

issue17, branch wp/16-training-controls-ui, 기준 main9cbc93f다. 저장된 M2 계획의
실제 훈련 producer 뒤에 플레이어의 시작·취소 동작을 연결한다. 실제 Claude
계획은 기준의26개 blob을 읽어 확인했고 runtime tests는 실행하지 않았다.

## 변경

공용 commandLedger와 군사 pending/barrier channel, optional MilitaryPanel
controls, App의 현재 Join 국가·연결·조회 수명과 ko/en 버튼/feedback을 연결한다.
기존 MilitaryCommand Train/Cancel만 발행한다. ACK 후 새 query serial의
상관 authoritative view까지 명령을 잠그며 local resource/progress 계산과
낙관적 job 생성은 없다. [ADR-1603](../adr/ADR-1603-training-command-controls.md)과
[재현 절차](../../tests/repro/WP-16-training-controls/README.md)를 따른다.

## 작성자 검사

기준의 실제 embedded client에서 Start training 버튼이 없어 UI assertion
RED(exit1)를 재현했다. guard 중단이 아닌 요구조건 실패다. 구현 후 같은
native UI 클릭의 Pending 시작·취소가 GREEN(exit0)이다. 마지막 filter 수명
보완 뒤 전체 client282개(20 files)·production client build가 exit0다.

직전 후보의 전용 UI9개는 동일 assertion의 순차3+4+2 그룹으로 exit0였다.
native 시작·Pending/Training/Ready 취소·인력8/장비6 반환·관전자/foreign,
real CONNECTING·수동 reconnect·ko/en 키보드/mobile·실제 구형 client를
검사했다. 지연/unknown ACK·close·unsupported는 표시된 frame 주입이다.
실제 구형 source bbebbe1의 dist를 localhost HTTP proxy로 제공하고 current
native WebSocket frame을 그대로 사용했다. browser 보안 설정 예외는 없다.
기존 native wire 검사는 fresh6+restored6 세션·host2개 exit0로 통과했다.
마지막 filter 수명 보완을 포함한 embedded bundle 재빌드는 단일 codegen으로
exit0다. 그 후보 d3e35bb의 전용 UI9개·native12세션 재검사도 모두 exit0이며
served JS가 후보 dist와 일치한다. 소유 host8개 exit0·PID 부재를 확인했다.

첫 full UI run은 장비 반환 기대값이 초기 stock2를 빠뜨린 점, 한국어 전환 뒤
영어 Language selector를 찾은 점, route.fulfill static 대체가 Chromium의
localhost 접근 검사에 걸린 점으로 실패했다. 반환 assertion을 before+6로
수정하고 실제 언어 label·실제 HTTP proxy를 사용했다. assertion을 제거하지
않았다. 수정 후 full run은7개 통과 뒤 메모리 guard가 중단했으며, 같은9개를
순차 그룹으로 통과했다. 중단된 run은 성공으로 계산하지 않는다.

서버 재빌드는 공유 메모리90% guard에서 중단된 이력을 보존한다. 별도 활성
compiler 없이 tmpfs shared memory 약8.3GB와 파일 cache가 남아 있었다.
자기 생성물 fsync·읽기 전용 page-cache advice 뒤 직전 bundle 재빌드는
exit0/guard-stop false, peak15377612800bytes였다. 마지막 bundle 재빌드의
두 guard 중단도 통과로 계산하지 않는다. `cargo rustc -p oh_server --bin
oh_server --locked -- -C codegen-units=1`은 exit0·guard-stop false이며 peak
15292678144bytes다. 처음 target 미지정 rustc 시도는 exit101이고 이후 올바른
binary target을 명시했다.
파일·worktree·shared cache를 삭제하거나 메모리 제한을 바꾸지 않았다.
heavy job1개, Cargo jobs1·browser worker1과 자식 CPU2개 제한을 유지한다.

독립 변경 리뷰·exact-head CI/main 병합은 대기다. 작성자의 실행은 독립 QA
판정이 아니며 개인 세션/내부 실행 기록은 저장소에 추가하지 않는다.

## CI 현지화 상관 대조 보완

후보 d3e35bb의 push client CI는 기존 현지화 case가 UI 미발행 sequence99의
ACK를 alert로 기대해 Chromium/Firefox/WebKit3개에서 실패했다. 새 공용
원장의 미발행 ACK 폐기는 유지한다. 작성자는 같은 embedded binary와 실제
Chromium에서 원 case의 동일 assertion RED(exit1, guard-stop false)를 재현했다.

현지화 case만 실제 UI speed3 클릭의 발행 sequence를 유지한 채 invalid speed9
payload를 native 서버로 보내도록 보완했다. server가 실제로 응답한 실패의
영문·한국어 assertion을 유지한다. 별도 미발행 sequence99의 native Pause
ACK가 관측돼도 alert를 지우지 않아야 하며, 그 server watermark 뒤 실제 UI
Pause 명령의 invalid-sequence 한국어 assertion도 유지한다. 주입 payload는
표시했고 product source·정책·게임 계산은 바꾸지 않았다. 해당 Chromium case
GREEN(exit0, guard-stop false)과 TypeScript typecheck(exit0)를 확인했다.
새 SHA의 독립 delta review와
fresh exact-head CI는 필요하며 기존 실패 run을 재실행으로 숨기지 않는다.

## 남은 범위

실제 게임 QA·독립 runtime 판정·전체 WP-16/M2는 미완료다. 이 candidate는
Ready 배치·군 우선순위·편제 편집·감편 전투/보급 효과·공유 session을 추가하지
않는다. 다음 후보는 기존 권위 계약의 Ready 배치 UI이며 별도 실제 계획이 필요하다.
