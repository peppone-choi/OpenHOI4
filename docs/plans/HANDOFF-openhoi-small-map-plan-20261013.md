# 기획 인계: 제품 지도 위 사단 위치 표시(읽기 전용) — 1개 작업

| 필드 | 값 |
|---|---|
| taskId | openhoi-small-map-plan-20261013 |
| project / stage | openhoi / plan |
| repository | https://github.com/peppone-choi/OpenHOI4 |
| branch | buzz/openhoi-small-map-plan-20261013/openhoi-plan (로컬, push 안 함) |
| cwd | /srv/openclaw-dev/company/agents/openhoi-plan/worktrees/openhoi-small-map-plan-20261013 |
| 기준 commit | 7c9ded87e5cf7eae66a896a8e9e32a20f544c44c (= 원격 main, PR #19 병합) |
| 제안 구현 task | openhoi-division-map-markers (이름은 총괄이 확정) |
| 다음 담당 | OpenHOI4 구현 담당(Codex GPT-6.1 Sol) → 독립 리뷰(Claude Opus 5.5) |

## 1. 과거 후보 A1–A3 회수 여부 (먼저 확인한 것)

| 확인 | 결과 |
|---|---|
| 원격 `git ls-remote` | `wp/32-new-province-map-first` 브랜치 없음 |
| 원격에서 47a1a14 / 5b6b4f7 fetch | `not our ref` (원격에 객체 없음) |
| 공용 기준 저장소 `/srv/openclaw-dev/company/repos/openhoi.git` | 두 commit 없음 |
| HISTORY.json 경로 `/workspace/worktrees/openhoi-new-province-map-first`, `/workspace/scratch/...` | 이 호스트에 없음 (다른 실행 환경 경로) |
| WINDOWS-INVENTORY.json | `E:\openhoi`, `E:\openhoi-task-scratch`, `E:\openhoi-task-tmp` 존재 기록만 있음. 브랜치/commit 정보 없음 → 후보 존재 미확인 |

결론: 이 호스트와 원격에서는 후보를 **회수할 수 없다**. 그래서 리뷰 인계를 하지 않는다.
후보는 덮거나 재작성하지 않는다. 실제 세계 지도 생성(Python 생성기·지도 파일·팩)은
이 작업 범위에서 **제외**해서 나중에 후보를 회수해도 충돌하지 않게 한다.

**Blocker B-1 (작업 1과 별개, 작업 1을 막지 않음)**: 후보 회수에는 Windows의
`E:\openhoi`(또는 E:\openhoi-task-*)에서 `git cat-file -t 47a1a14924fddd4aef3a504c6324e6eac3c35085`와
`git branch --list wp/32*`를 실행해 확인해야 한다. 공유 GUI mutex가 필요 없는 읽기 전용
git 확인이다. 실행 주체는 총괄/Windows 경로 담당. 해제 조건: commit이 있으면 → push 승인을
받아 원격 브랜치로 올리고 독립 리뷰 인계. 없으면 → 손실로 기록하고 WP-32 실제 지도는
새 기획 작업으로 다시 시작(사람 결정 필요: 재생성 범위·원천 자료).

## 2. 선정 작업: 지도 위 사단 위치 마커 (읽기 전용)

### 목적
현재 main의 제품 지도(`MapView`)에는 사단이 보이지 않는다. 사단 데이터
(`MilitaryDivisionView.province`)는 이미 서버에서 오고 군사 패널에서만 쓰인다.
이 작업은 이미 받은 군사 view를 재사용해 **사단이 있는 프로빈스에 마커를 그리고,
마커를 클릭하면 그 프로빈스를 선택**하게 한다. 그러면 Deploy(PR #19) 결과인
"사단 생성"이 처음으로 지도 위에 보인다. 나중의 이동 화살표·전투·점령 표시가 올라갈 바탕이 된다.

### 근거
- 사용자 우선사항: 사단·생성을 지도에 표시 (CONTEXT.txt).
- 기존 구현 재사용: `MilitaryView.divisions[].province/nation/army/id` (client/src/proto/protocol.ts:66,72),
  `App.tsx`의 `military` 상태, `MapView`의 `onSelect`, 렌더러가 이미 게시하는 카메라
  (`host.dataset.camera`, client/src/map/renderer.ts:91).
- 지도 비의존: 마커 위치는 클라이언트가 이미 받은 인덱스(`decodeIndex`)의 픽셀 평균에서
  화면 표시용으로만 계산한다. testland와 나중의 실제 세계 지도 모두에서 그대로 작동한다.
- 서버·wire·저장·시뮬레이션 변경이 없다 → 가장 작고 충돌이 적다.

### 범위에 넣지 않는 것
이동 명령·화살표, 전투·점령 표시, 새 서버 query/wire 필드, 군사 패널을 닫았을 때의 자동 조회,
줌 단계 LOD 집계(REQ-MAP-06, M3), 3D/InstancedMesh 아이콘(WP-38), 지도 생성/팩/에셋 변경,
`renderer.ts` 셰이더 변경, WebGPU 검증.

### 파일 소유 범위
| 파일 | 변경 |
|---|---|
| `client/src/map/divisionMarkers.ts` (신규) | 순수 함수: `MilitaryView`+`WorldView`+index → 프로빈스별 `{province, nation, count, x, y}` (ID 순 정렬, 미지 프로빈스는 `map-data-error`) |
| `client/src/map/divisionMarkers.test.ts` (신규) | 단위 테스트 |
| `client/src/map/DivisionMarkers.tsx` (신규) | 지도 host 위 DOM overlay, 카메라 값으로 위치 계산, 클릭 → `onSelect(province)` |
| `client/src/map/MapView.tsx` | 선택 prop `divisions?: MilitaryView|null` 1개 추가 + overlay 마운트. 기존 로드/fallback 로직 변경 금지 |
| `client/src/map/renderer.ts` | 필요하면 카메라 변경 콜백 1개만 추가(기존 `publishCamera` 지점). 그 밖 변경 금지 |
| `client/src/App.tsx` | `military.view`를 `MapView`에 전달하는 1줄 |
| `client/src/shell.css` | 마커 스타일 몇 줄 |
| `client/public/locales/{ko,en}.{ftl,json}` (기존 형식 따름) | 마커 접근성 라벨 키(예: `map-division-marker`) |
| `client/e2e-m1/division-markers.spec.ts` (신규) + 기존 military 계열 config에 등록 | 브라우저 확인 |
| `docs/worklog/WP-16-division-map-markers.md` (신규), `HANDOFF.md` 상단 항목 | 기록 |

wire/proto, `crates/**`, `data/**`, `assets/**`, `tools/**`는 수정하지 않는다.

### 인수조건
1. 군사 view가 `ready`이고 사단이 있으면, 사단이 있는 각 프로빈스에 마커가 정확히 1개 보인다.
   마커에는 그 프로빈스의 사단 수가 보이고 소유 국가를 구분할 수 있다(국가 색 재사용,
   새 팔레트 상수 금지). 같은 프로빈스에 여러 국가가 있으면 국가별 마커를 나눈다.
2. 마커 위치는 해당 프로빈스 영역 안에 있고, 카메라 이동·줌·리셋·리사이즈 뒤에도 프로빈스를 따라간다.
3. 마커 클릭은 그 프로빈스 선택과 같다(`selected-province`가 바뀜). 드래그·호버·기존 프로빈스 클릭 동작은 그대로다.
4. Deploy ACK 뒤 군사 view가 새로 오면 새 사단 마커가 나타난다(기존 ready-deployment 흐름 재사용).
5. 군사 view가 `null`/`loading`/`stale`/`unsupported`/`disconnected`면 마커가 없다
   (stale 표시는 하지 않음 — 낡은 위치를 권위처럼 보이지 않게). 패널 닫힘 = 마커 없음.
6. view의 `province`가 world에 없으면 마커를 그리지 않고 기존 `map-data-error` 규칙을 따른다(조용히 무시하지 않음).
7. 마커는 키보드 포커스 가능한 버튼이고 ko/en 라벨이 있다.
8. 서버/wire/저장 해시·기존 e2e·스크린샷 기준선이 바뀌지 않는다(마커가 없는 기존 화면은 동일).

### 확인 계획
- 로컬(가벼움, Hostinger): `npm --prefix client run typecheck`, `npx vitest run src/map/divisionMarkers.test.ts`.
  RED 먼저: 마커 부재를 테스트로 확인한 뒤 구현.
- 단위 테스트: 빈 사단 → 0개; 한 프로빈스 여러 사단 → count 합; 두 국가 같은 프로빈스 → 2개;
  미지 프로빈스 → 오류; 입력 순서가 바뀌어도 결과 동일.
- GitHub CI(무거운 검사): 기존 `ci.yml`의 client test·build·Playwright(chromium/firefox/webkit)와
  새 `division-markers.spec.ts`(testland_m2_military, Deploy 뒤 마커 출현·클릭 선택·카메라 이동 추종).
- Windows GUI·HOI4·GCP 사용 없음.

### 실행 순서
1. 구현 담당: main 7c9ded8 기준 새 worktree/브랜치, RED 테스트 → 구현 → 로컬 가벼운 확인 → push·PR.
2. GitHub CI 통과 확인.
3. 독립 리뷰 1회(Claude). 지적 수정 후 그 범위만 재검증.
4. 병합 담당이 기존 PR 정책으로 병합. 배포·초기화 없음.

## 3. Blocker 요약
- B-1: 과거 후보 A1–A3 회수 미확인(위 §1). 작업 1을 막지 않는다.
- 작업 1 자체: 없음.
- 알려진 위험: 실제 세계 지도가 아직 main에 없어서 이 작업의 화면 확인은 testland 위에서만 된다.
  제품 완료 주장이 아니고 사단 표시 기반 작업이다.

## 4. 이 인계의 검증 근거
읽은 것: CONTEXT.txt, HISTORY.json, WINDOWS-INVENTORY.json, HANDOFF.md, docs/02-production-spec.md
(WP/REQ 표), ADR-0701, client/src/map/*, client/src/proto/protocol.ts, client/src/App.tsx,
.github/workflows/ci.yml. 실행한 것: `git ls-remote`, 원격/공용 저장소 commit 조회.
구현·빌드·테스트·게임 실행·GitHub 쓰기는 하지 않았다.
