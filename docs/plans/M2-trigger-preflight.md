# WP-13 시작 전 종료·registry 인수 대조

2026-10-07. W1-a 검증 중 오케스트레이터가 작성한 준비 문서다. 이 문서는 새 게임 행동을 채택하거나 구현 완료를 판정하지 않는다. WP-13 담당자가 실제 통합 HEAD와 producer 타입을 읽은 후 자기 schema/ADR에 답을 기록한다.

## 현재 확인한 경계

01 §3.3은 데이터 종료 조건, 플레이어 항복 시 종료와 관전 선택, 종료일의 네 축 가중합 점수를 정했다. 02 §5.6은 단일 키 조건 AST, all/any/not, scope/if 효과, 깊이 16·원시 효과 1000 상한, 미지원 키·인자·ID의 로드 거부를 정했다. 02 §5.7은 start_date/end_date/end_conditions/score_weights를 선언한다. 종료일의 시각, 복수 조건의 최상위 결합, 동시 원인 표시와 이후 명령·관전의 상세 행동은 아직 적혀 있지 않다.

기존 Simulation은 완료한 시간 수를 tick으로 보관한다. 명령을 먼저 적용하고 실행 중일 때 한 시간을 전진한 뒤 이동·일일 시스템·원장·스냅샷 순으로 처리한다. Pause 중에도 명령을 처리하되 시간이 전진하지 않는다. clone에서 검사한 후 commit하는 실패 원자성을 유지해야 한다. 원 M0/M1 시나리오는 종료 필드가 없으며 기존 canonical/save/hash를 유지한다.

WP-17의 실제 인수 계약은 typed movement/strait context와 v1·v2·v3의 보존이다. WP-24의 실제 인수 계약은 pack validation registration, 등록된 실제 loader, typed 진단과 caller별 제한 정책이다. 해당 브랜치의 설명만으로 인수하지 않고 독립 PASS와 P-07·같은 main CI 뒤 통합 타입을 읽는다.

## 담당 설계가 답할 선택

| 선택 | 기존 명세에서 확인할 근거 | 답을 기록할 곳 | 독립 검사 |
|---|---|---|---|
| end_date 시각과 최초 로드 때 이미 충족한 종료 | 달력일은 정의됐으나 시각은 없음 | 구체 입력·결과의 REQUEST와 채택 상태, WP-13 schema | 직전 hour/정확 경계/다음 hour, 윤일, start=end, start>end, 로드·재개 |
| 복수 end_conditions와 self/국가별 평가 | 내부 all/any/not는 정의, 최상위 목록과 평가 root는 미정 | REQUEST와 root context·참조 표 | 빈 목록, 두 조건 중 하나 참, 둘 다 참, 비연속 국가 ID, 타국 scope |
| 동시 종료일·조건·end_scenario 및 종료 뒤 동작 | 종료와 관전 선택만 정의 | REQUEST와 상태 전이·명령 결과 표 | 원인 정렬, 한 번만 종료, Pause/SetSpeed/예약 이동, 큐·clock·상태 실패 보존 |
| score 축의 실제 입력과 수명 | 승점·IC·생존·진영 승리 가중합 | producer별 accessor/단위/참조 및 지원 여부 표 | 값·가중치 0/최대/음수 정책/overflow, 분해와 총합 raw 일치, 종료 snapshot·저장 재개 |
| flags 수명·scope와 effect 원자성 | set_flag/clear_flag/has_flag 이름만 초기 registry에 있음 | 필드 표·REQUEST가 필요한 게임 의미 | 같은 키 타국, 중복 set/clear, 없는 키, scope 복귀, 중간 효과 오류의 전체 상태 보존 |

결정이 필요한 행동은 구체 추천안으로 CEO에 전달한다. 선언만 있는 후속 값을 0/false/neutral/no-op으로 성공시켜 선행 구현을 흉내내지 않는다. scoring의 미래 생존·진영 입력은 WP-20 등 실제 producer 인수가 필요하다. 기존 국가 IC는 01 §4.3의 정의이며 군수 배분량·공장 개수·생산량과 같은 값으로 취급하지 않는다.

## registry와 실제 adapter

각 초기 원시 항목은 인자 스키마·설명·예시, 읽는/바꾸는 필드, 허용 scope, ID domain, 지원하는 host context, 등록 위치를 대응시킨다. standalone 문법의 지원과 실제 Simulation 실행 지원을 분리한다. 사용하려는 실제 host가 미지원이면 해당 팩 로드에서 거부하며 예약 실행에서 처음 실패하게 두지 않는다. 테스트 전용 독립 host는 실제로 상태를 읽고 바꾸는 adapter여야 한다.

조건 평가는 host 상태·RNG·queue·flags를 바꾸지 않는다. chance는 이벤트 발동 context 외에서 로드 거부한다. effect 1000 경계는 AST 노드 수가 아니라 실제 원시 효과 실행 수이며 반복 scope/분기와 전체 실행 예산을 설계에서 대조한다. 깊이의 root·scope/if/not 계산 방법도 독립 16/17 예제에 명시한다.

WP-14/15/20은 자기 producer가 생긴 뒤 registry의 경제·장비·전쟁 항목을 실제 데이터 로드와 권위 실행에 연결한다. WP-23은 이 실제 supported set만 사용한다. registry reference 생성은 REQ-MOD-07의 후속 증거와 구분한다. 원 fixture·expected·명령 단언을 보존하며 새 종료/flags 상태의 hash/save/wire 형식을 additive로 설계한다.
