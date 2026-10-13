# WP-16 지도 사단 마커 — 구현 후보

- taskId: openhoi-division-map-markers-20261013
- project / stage: openhoi / implement
- repository: https://github.com/peppone-choi/OpenHOI4
- branch: buzz/openhoi-division-map-markers-20261013/openhoi-implement
- base: 7c9ded87e5cf7eae66a896a8e9e32a20f544c44c (착수 시 원격 main 재확인)
- cwd: /srv/openclaw-dev/company/agents/openhoi-implement/worktrees/openhoi-division-map-markers-20261013
- 계획: [기획 인계](../plans/HANDOFF-openhoi-small-map-plan-20261013.md)
- 상태: 구현 후보; 독립 리뷰/병합/제품 QA 미실행

## 구현 / 인수조건 대응

1. 이미 받은 ready MilitaryView만 App에서 전달. province+nation별 count와 서버 nation.color 재사용. 여러 국가는 한 프로빈스 내부 대표점에 묶인 개별 버튼으로 구분한다.
2. 인덱스 두 번의 선형 순회로 평균에 가장 가까운 **실제 해당 프로빈스 픽셀 중심**을 선택한다. 비볼록/분리 영역도 내부점 보장; 동률은 row-major. 앵커는 지도 로드 때 한 번만 계산하고, 군사/세계 view 갱신은 집계만 한다. publishCamera 콜백과 백분율 overlay로 pan/zoom/reset/resize를 추종한다.
3. 버튼은 기존 onSelect만 호출한다. 렌더러 소유 host의 형제 overlay는 빈 공간 pointer-events:none; 기존 host 입력/셰이더/로드 검증/fallback은 유지. 마커 위 wheel은 기존 host 핸들러로 전달한다.
4. Deploy ACK 뒤 기존 군사 결과 갱신을 그대로 소비; 새 query/이동 명령 없음.
5. null/loading/stale/unsupported/disconnected/패널 닫힘은 App에서 null을 전달해서 마커 없음.
6. 없는 province/nation/픽셀 대표점은 부분 결과 없이 map-data-error; 기존 지도 alert 사용.
7. 기본 button 키보드 동작 및 ko/en Fluent count/province/nation 라벨, focus-visible.
8. server/wire/save/crates/data/assets/tools 및 기존 스크린샷 baseline 무변경. 계획 후보 A1–A3는 건드리지 않음.

## 가벼운 실제 검증

- 2026-10-13: 함수 작성 전 신규 unit suite RED (`Cannot find module ./divisionMarkers`).
- 최초 구현 검증에서 테스트의 ES2023 toReversed와 부동소수점 exact 비교 실패. 기존 baseline 수정 없이 신규 테스트만 ES2022 reverse/close 비교로 수정.
- `npm --prefix client ci --ignore-scripts`: pinned dependencies 설치 성공, package/lock 변경 없음.
- `npm --prefix client run typecheck`: PASS.
- `cd client && npx vitest run src/map/divisionMarkers.test.ts`: 5 PASS. null/빈 view, 합계, 두 국가, 순서 불변, 비볼록/분리 내부점, 미지 reference/픽셀 오류, camera projection 확인.
- `git diff --check`: PASS.

## GitHub-hosted 검증 범위

새 division-markers.spec.ts는 기존 M1 config에 등록되어 CI의 chromium/webkit 및 Xvfb Firefox 묶음에 포함한다. 군사 전용 config에도 등록한다. 각 시험은 testland_m2_military 팩 복사본과 실제 서버를 만들고 finally에서 PID 종료/부재/exit0 증거를 남긴다. 기존 checkpoint가 필요 없는 fresh Train→Ready→Deploy 시나리오다.

- native Deploy 출현, 실제 인덱스와 화면 중심 pick 일치, 카메라 pan/marker-wheel/reset/resize, 클릭/ko/en 키보드 선택, 패널 닫힘 뒤 군사 자동조회 없음.
- injected correlated unknown/unsupported 군사 결과는 native와 명시적으로 구분. 실제 socket disconnect 뒤 stale 마커 숨김.
- CI run URL/최종 source SHA와 결과는 PR 및 총괄 인계에 기록. 로컬 build/browser/heavy suite는 실행하지 않는다.

## 미실행 / 경계

독립 리뷰, 병합, Windows GUI/HOI4, GCP, 배포/초기화 모두 NOT_RUN. testland 검증은 실제 세계 지도 제품 완료를 뜻하지 않는다. 과거 47a1a14/5b6b4f7 후보 회수는 별개 총괄 작업이다. 다음 담당 openhoi-review는 총괄이 배정한다.
