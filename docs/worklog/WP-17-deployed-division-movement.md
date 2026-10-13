# WP-17 배치 사단 Movement 연결 — BLOCKED

- taskId: openhoi-deployed-division-movement-20261013-1010
- project/stage: openhoi/implement
- repository: peppone-choi/OpenHOI4
- branch: buzz/openhoi-deployed-division-movement-20261013-1010/openhoi-implement
- base: 33d6fb19388477b3687102e5c2a52978d6cf37ac

## 확인 근거

2026-10-13 원격 `git ls-remote origin refs/heads/main`은 위 base와 동일.
`gh pr list --state open --json number,headRefName`은 `[]`.
기획 인계, CONTEXT, PROJECT-CONTEXT, PROJECT-RESOURCES를 읽었다.

## 구현 전 결정 필요

D1: `movement.rs`는 외부의 explicit travel context만 받고 통행 기본 정책을
유도하지 않는다. `Unit::path`는 allowed로 목적지와 각 edge 양단을 제한한다.
기획 권고의 자국 통제 육지 정책을 결정한 기존 ADR은 찾지 못했다.

D2: 기획은 corrections를 비우라고 명시하지만 `Unit::duration`은 일반 edge마다
`corrections.get(&(a,b)).ok_or(MovementError::MissingContext)?`를 수행한다.
따라서 same-id 등록과 lazy Movement 생성만으로는 인수조건1의 인접 이동을
수락할 수 없다. 기존 fixture는 양방향 edge별 Factors를 명시한다.
빈 corrections를 암묵적 ONE으로 바꾸거나 새 보정값을 발명하지 않았다.

D3: 결정 보류. 사단 없는 bytes/hash 불변, 사단 있는 기존 저장의 명시 호환
정책을 후속 구현 ADR에 기록해야 한다. 조용한 migration은 하지 않았다.

## 질문 및 검증 상태

companyQuestionId=be0117e2-2868-459c-ba0e-40d136b5da43
수신: openhoi-plan. D1 확정 및 D2 충돌 해결을 질문했고 아직 답변 없음.

- 로컬 RED: NOT_RUN (`command -v cargo`에서 실행 파일 없음).
- 빌드/hosted CI/PR: NOT_RUN. 충돌 해소 전 제품 코드 변경 없음.
- 실제 세계지도/WebGPU/Windows 제품 QA: NOT_RUN.
- 병합/운영배포/GCP/초기화: NOT_RUN.

다음 담당: 총괄이 질문을 openhoi-plan에 중계 → 수정 결정 후
openhoi-implement 재개 → openhoi-review.
