# ADR-1604 Ready 배치 위치 권위

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-11 |
| 관련 WP·REQ | WP-16, WP-22, REQ-MIL-03, REQ-MIL-05 |

## 맥락

Ready 배치는 기존 `MilitaryCommand.Deploy`를 사용하는 클라이언트 기능이다.
기존 지도 표시의 `world` 응답과 retained WorldView는 요청·Join 수명이
배치 권위와 다르다. MilitaryView에는 state hash가 있지만 WorldView에는
tick만 있으므로 두 응답의 원자적 동일 hash를 주장할 수 없다.

`oh_data::map`은 state에 Land만 허용하며 육지의 단일 state 소속을 검증한다.
WorldView의 ProvinceView.state와 StateView.provinces는 그 연결의 양방향
투영이다. owner·terrain 문자열로 육지를 추정하지 않는다.

## 결정

- `WorldAuthority`는 전용 full-world `deploy-world:N` 요청 하나와 dirty를
  관리한다. 현재 socket/epoch의 발행한 요청과 일치하는 supported 응답만
  권위를 준다. ID·소속·참조 검증 실패, missing/unsupported는 권위를 닫는다.
- panel open, Join의 Welcome, filter 변경, 군사 ACK에서 요청한다. Delta마다
  추가하지 않는다. 새 요청·ACK·scope 종료는 권위를 폐기하며 dirty는 응답
  뒤 한 번 재조회한다. 기존 군사 조회 slot·dirty와 sequence 원장은 유지한다.
- 선택 후보는 검증된 state 소속 육지이면서 controller가 Join 국가인 곳이다.
  자기 Ready job·자기 army·canonical u64/u16·명시적 boolean을 확인한다.
  군 정원·장비 충분성·자원 이전은 서버가 판정한다. 체크박스 기본값은 false다.
- Deploy pending은 ACK 때 기록한 military/world 발행 serial을 각각 넘은
  authoritative 응답 둘이 있어야 끝난다. 도착 순서와 무관하다. ACK만으로
  성공을 표시하거나 자동 retry하지 않는다. 두 view의 tick 동일성을 요구하지 않는다.
- Train/Cancel은 PR18의 military ACK 장벽을 보존한다. 위치가 필요 없는 기존
  동작을 world 지원 여부에 묶지 않는 호환 조정이다. 모든 군사 ACK는 배치
  위치 권위를 철회·새로 요청하고, Deploy만 두 장벽을 요구한다.
- job별 선택 초안은 world 권위 generation·scope·명령 상태 변경으로 초기화한다.
  연결이 끊긴 filter 변경에서는 마지막 군사 표시와 stale 상태를 보존하며
  송신 권위는 없다. standalone read-only panel도 유지한다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| retained 지도 또는 고정 `world` 응답으로 승인 | 추가 조회 없음 | 다른 scope·부분 응답·미발행 응답과 혼동 |
| 새 wire land 플래그 | 명시적 종류 | 기존 loader의 state 계약으로 충분하며 producer 범위 변경 불필요 |
| World/TimeState tick 동등 또는 매 Delta 재조회 | 표면상 최신성 | 같은 hash/원자성을 증명하지 못하며 지속 재조회·권위 철회 유발 |

## 결과와 영향

서버·sim·schema·save·wire·pack·의존성 변경은 없다. world 조회 뒤의 controller
변경과 정원 변화는 기존 서버의 최종 검증에 맡긴다. semantic 거절은 현재
서버의 `invalid-message`로 표시하며 정밀한 새 거절 이유를 발명하지 않는다.
SetPriority·이동·전투·보급·공유 multiplayer 지속성은 후속 범위다.
