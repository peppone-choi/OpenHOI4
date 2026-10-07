# ADR-1701 결정론적 육지 이동 시간과 저장 추가 경계

| 항목 | 값 |
|---|---|
| 상태 | 기술 계산/저장 채택; 진행 edge Stop/reroute CEO 한정 잠정 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-17, REQ-MIL-04 |

## 맥락
01 §4.7은 거리÷속도×보정을 정하지만 반올림/틱 잔여시간과 중간 edge reroute는 정하지 않았다. 기존 v1 Postcard와 legacy canonical은 동결이다. 보급/접근/편제 후속 효과는 이번 작업의 규칙이 아니다.

## 결정
Fx checked_div(distance,speed)→checked_mul(terrain)→infrastructure→supply→river의 정확 순서로 fixed 기본 바닥 반올림을 적용한다. 모든 입력과 결과는 양수, overflow/0 underflow는 오류다. 한 tick은 1h이며 첫 edge의 elapsed에 시간 budget을 적용하고 완료 edge에서만 province를 갱신한다. 잔여 budget은 다음 edge에 같은 tick에 사용, 도착후 남은 budget은 버린다. 경로는 이 계산 시간의 최소합, 동점 전체 ID 경로 lexical. 보정 적용값을 route duration과 함께 저장하고 복원시 정확 bit로 재계산한다.

CEO 2026-10-07 REQUEST-0006 A 한정 잠정 채택으로 진행 edge reroute/Stop은 그 edge 도착까지 유지한다. 정지후 위치 순간이동/반환 시간을 임의 만들지 않는다. 입력 enqueue preview와 apply 재검증, phase clone commit으로 실패 원자성을 유지한다. 이동 None의 serde skip은 기존 bytes를 보존한다. 이동 Some은 canonical에 전체 권위 입력/진행/예약큐 포함. v1 DTO는 변경하지 않고 별도 v2 body와 reader/writer 추가, legacy encode는 v1 자동 선택. 동일 header 필드 모양에 format_version2, prefix/header 동일 version. 신규 상태를 v1에 쓰려 하면 손실 방지 오류다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| edge마다 ceil hour | 정수 tick 단순화 | 작은 edge에서 잔여 시간이 사라지고 거리÷속도 값 손실 |
| 중간 reroute 즉시 출발점 복귀 | 단순 | 진행거리 순간 삭제와 비용 규칙 발명 |
| v1 끝에 optional field | 구현 짧음 | Postcard 필드 순서/기존 fixture 호환 파괴 |
| 계수 누락1 | 초기 host 연결 쉬움 | 미구현 보급/접근 규칙을 neutral 성공으로 은폐 |

## 결과와 영향
외부 context는 필수이며 실제 게임 unit/보급/접근 adapter는 후속 WP다. 이번 새 dependency/version/API 외부조건 변경은 없다. 저장 reader는 v1/v2를 모두 지원하고 allocation preflight는 기존 정책을 재사용한다. 최단 경로 queue는 ordered heap과 ID/path lexical 순으로 결정론적이다. schema의 독립 합성 경계값을 구현/검증에 전달한다.

부모 main 01 §4.7 / 02 §5.11 / 03 P-03 및 REQUEST-0006 원문 SHA256 0b9e2ba3756fa1022a876f11369dc8b4e4909fc49e84ea35bfbf9b7d75734b07, 독립 정수8경계(2026-10-07)를 읽고 채택 범위를 대조했다. 사용자 D/OPEN 추가확정·콘텐츠값·새 보급/접근/전투·저장 migration 승인이 아니다. None만 legacy이며 Some(empty)·정지unit·context·모든 Move/Stop 예약도 v2다. restore context는 기존 local world/pack/defines identity를 재검증하며 v2 external applied factors/access는 저장 authority로 복원·참조/양수/순서/route formula raw bit를 검사한다. context는 trusted host 초기 생성에서만 설정하고 수명은 unit/Simulation과 같다. 동적 갱신 API는 후속이다. 클라이언트/DTO 조회에서 이 값을 변경할 수 있는 경로는 없다.

strait 누락 대조: 01 §4.1의 인접 crossing kind와 WP31 항구 간 해상수송은 같은 개념으로 추정하지 않는다. 현재 solver는 strait를 탐색 후보에서 제외한다(그 경계만 이어지면 NoPath). strait 통과 정책/계수·담당 WP는 결정 필요이며 해상수송에 임의 위임하지 않는다. sea/lake unit 위치 및 직접 육지 목적지는 InvalidReference, 실제 고립/impassable만의 연결은 NoPath, 보정 context 누락은 MissingContext로 구분한다.

P-06 후속 ADR-1702가 REQUEST-0007 A의 typed Strait 슬롯/explicit context/additive v3를 추가한다. 본 ADR의 원97f strait 미지원/미정 기록은 당시 상태이며, 일반/하천·REQ6·legacy bytes 기술 선택은 그대로 보존한다.
