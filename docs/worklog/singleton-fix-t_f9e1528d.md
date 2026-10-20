# D1-R2 geometry harness 정책 수정

- taskId: t_f9e1528d / project: openhoi / stage: fix
- 기준: PR28 a5f12c82c129414f9e4bf3aff41cd02c0edd7121. 독립 진단 6af066ab6e8d6c9f9c8c895b2845b9e514d44b50.
- branch: fix/t_f9e1528d. 원래 PR28 branch, 진단 branch, 제품 입력/로더/기존 CI job 변경 없음.

## 승인 범위

D1-R2는 비제품 geometry harness에만 최대 두 번의 동시 snapshot 결정을 허용한다. 1단계는 원본 면적 >=2인 직접 4방향 이웃, 2단계는 1단계에 후보가 없었던 원본 singleton만 대상으로 직전 snapshot 면적 >=2인 직접 이웃을 사용한다. 가로 wrap/세로 nonwrap, same-kind 우선/공유변 최다/작은 ID 순위는 유지한다. 각 단계 모든 결정을 마친 뒤 쓰며 단계 안 chain/순차 갱신/원거리 fallback/3단계 반복을 금지한다. 2단계 뒤 후보가 없으면 명시 FAIL이고 입력은 변하지 않는다.

단계별 mapping/count/kind 변경, 모든 original ID의 최종 surviving ID, 잔존 singleton, 원본 2~8px 면적 통계를 report.json에 기록한다. 원본 non-singleton 픽셀 불변과 제거 ID 참조/빈 프로빈스 부재를 검사한다. 독립 oracle은 역순으로 결정을 내리고 snapshot 면적을 재계산하며 원본 모든 픽셀, PNG, 종류, 간선, 최종 ID 및 반복 산출물을 대조한다. Rust 실제 load_map/순환/seam/극관 assertion은 완화하지 않는다.

## 검증

로컬 소형 unittest 8개 PASS(고정 seed 100개 독립 oracle 포함), diff --check PASS. 전체 지도 계산과 Rust 빌드/시험은 GitHub-hosted 기존 world-preview-geometry job에서만 실행한다. 실제 hosted 결과는 외부 evidence/t_f9e1528d/FIX.md 및 results/t_f9e1528d-RESULT.json에 기록한다. 실행 결과 없이 실제 load_map 성공을 주장하지 않는다.

D2~D5/Q1 미정. 종류 변경은 기술시험 한정이며 제품 지도 채택이 아니다. productComplete=false. 제품QA/M1/AC10/actual product pack/gameplay/merge NOT_RUN. Windows/GCP/운영/배포/초기화/새 비용 작업 없음. 다음 담당 openhoi-review.
