# ADR-1701: 배치 사단의 기존 이동 권위 연결

근거: 기획 담당 실제 답변 @c4d298f, companyQuestionId
be0117e2-2868-459c-ba0e-40d136b5da43, 총괄의 2026-10-13 재개 인계.
원 기획 문서 D2 수정은 NOT_RUN이므로 이 실제 답변을 정정 근거로 삼는다.

## D-1 통행 범위

Deploy 시점 국가가 통제하는 육지 스냅샷을 allowed로 등록한다.
controller가 없으면 owner를 사용한다. World 로더도 통제 override 부재 시
소유자를 사용한다. 통제 변경, 전쟁, 통행권 갱신은 이 작업 범위 밖이다.

## D-2 항등 보정

allowed 양단을 갖는 Normal/RiverSmall/RiverLarge 인접 간선 양방향에
terrain/infrastructure/supply/river = Fx::ONE을 명시한다.
거리와 사단 normal.speed_kmh를 그대로 사용한다. 빈 corrections 원 지시는
기획 오류로 정정되었다. from_defines에 필요한 정의가 없으므로 호출하거나
새 defines를 추가하지 않는다. 후속 시스템이 준비되면 from_defines로 교체한다.
새 lazy 컨텍스트는 straits=None이며 기존 해협 정책은 변경하지 않는다.

## D-3 명시적 저장 호환

기존 movement DTO를 그대로 사용한다. 사단 없는 저장은 컨텍스트를 생성하지
않으므로 bytes/hash 및 v1~v8 레이아웃은 바뀌지 않는다.
사단 있는 기존 저장에서 same-id 이동 유닛이 없으면 oh_save codec의
encode/decode가 DeployedMovementMissing으로 거부한다. 자동 재구성/조용한
migration은 없다. 위치/국가/속도 불일치도 거부한다. 저수준 Simulation DTO
복원 API는 기존대로이며 host codec 검증이 호환 경계다.

Deploy 자원소비와 등록은 동일 명령 트랜잭션으로 commit하며 실패는 모두
rollback한다. Movement phase 완료 후 roster province를 동기화한다.
Stop은 기존 의미(이동 중 첫 간선 유지, 뒤 경로 폐기)를 그대로 따른다.
