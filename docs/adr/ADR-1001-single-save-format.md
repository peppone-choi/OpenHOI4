# ADR-1001: 저장 형식 단일화 (0.1 전까지 호환 없음)

근거: 사용자 결정 2026-11-06 ("기본안대로 진행해", BACKLOG 검토 기본안 1번).
아직 플레이어가 없으므로 0.1 출시 전까지 이전 저장 파일 호환을 약속하지 않는다.

## 결정

- 파일 형식 버전은 `oh_save::FORMAT_VERSION` 하나뿐이다. 현재 값은 9다.
  1~8은 예전 레이아웃 번호이며 다시 쓰지 않는다.
- 본문은 `oh_sim::save::SimulationSave` enum 하나다. 변형(Core, Movement,
  Strait, Trigger, Economy, Production, Military)은 시뮬레이션이 실제로 돌리는
  시스템에서 고른다. 각 변형은 기존 층 DTO(V1~V7)를 감싼다.
- 다른 버전 파일은 `UnsupportedFormat: file vN, supported v9`로 거부한다.
  migration은 만들지 않는다(REQ-SAV-04의 "명확한 거부" 쪽).
- 본문 모양을 바꾸는 PR은 `FORMAT_VERSION`을 올리고, 커밋된 `.ohsave`
  fixture를 새 형식으로 다시 만든다. 예전 바이트를 보존하는 호환 fixture는 두지 않는다.
- 모드 검사(Military/Production/Economy/TriggerModeMismatch)는 헤더 번호가 아니라
  본문 변형과 로컬 정의를 비교한다. 오류 이름과 우선순위는 그대로다.

## 바뀐 것

- 제거: v7 잡 기록 상한(4096)과 v7/v8 구분, 8개 버전 상수, 버전별 decode 분기.
- 제거: 원 source97f v2 원본 fixture(`movement-v2-97f.*`), 그 팩 사본
  (`movement-pack-97f/`), 이를 읽던 호환 테스트 2개.
- 변환: `m1-v1.ohsave`(v1)와 WP-14 회귀 입력 3개(v5)를 같은 상태 그대로
  v9로 다시 감쌌다. 새로 만든 M1 저장 바이트가 변환본과 같음을 확인했다.
  게임 상태 해시(split `4eb703aad086c7aa`)는 바뀌지 않았다.
- 유지: 층 DTO와 복원 검증 코드, 3-OS 저장 바이트 비교, `legacy-validation`
  팩 정책(save_format은 9로 갱신). 팩 호환은 이 결정의 범위가 아니다.

## 후속

층 DTO를 평평한 구조 하나로 합치는 일은 필요해질 때 한다. 이제 호환 약속이 없으므로
층에 필드를 직접 더해도 된다.
