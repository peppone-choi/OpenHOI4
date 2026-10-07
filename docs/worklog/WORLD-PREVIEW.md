# WORLD-PREVIEW 작업 로그 — 실제 세계 지도와 메인 HUD

| 항목 | 값 |
|---|---|
| 상태 | 진행, 첫 시각 체크포인트 |
| 담당 | 구현 세션 |
| 브랜치 | codex/world-province-preview-m2r2 |
| 대상 REQ | REQ-MAP-01/02/04/07/10 일부. REQ-MAP-03/11 및 WP-32/45 전체는 미완료 |
| 선행 WP | 검증된 WP-07/08 renderer; base ea63081e0b79369227c99384922db80c6ef09fb1 |

## 계획

최초 schema/ADR 작성 후 fixed land/lake mask·구면 독립 분할을 bake하고 기존 renderer의 preview-only display adapter에 연결한다. 실제 전세계와 지역 미리보기를 먼저 제출하고 후속 3D 샘플·DEM 취득·세부 검사를 진행한다. 원 nation/state/sim/network/Testland 변경 없음. 전용 worktree에서만 명령 실행. 부모 외 새 세션 생성 없음.

## 테스트 우선 기록

| REQ | 테스트 이름 | 구현 전 결과 | 구현 후 결과 |
|---|---|---|---|
| REQ-MAP-10 | source missing/hash mismatch/path escape | module missing exit1 원로그 python-red.log | 3 tests PASS |
| REQ-MAP-01/02/07 | connectivity/island/seam adjacency, polygon hole/coordinates, u16LE | 동일 원실패 | 3 tests PASS |
| REQ-MAP-04/10 | baked SHA/schema/ref/IDs/display has no authority | model missing exit1 client-red.log | 전체 client132 tests PASS |

## 실행한 검증

모든 원로그는 이 WT `target/evidence/WORLD-PREVIEW/`에 보존.

| 명령 | 종료 코드 | 출력 발췌 / 로그 경로 |
|---|---|---|
| python -m unittest discover -s tools/maps -p test_world_preview.py | 0 | python-green.log, 6 tests |
| npm --prefix client ci | 0 | npm-ci.log, 103 packages, 0 vulnerabilities |
| npm --prefix client run build | 0 | build.log; 기존 large chunk warning 유지 |
| npm --prefix client test | 1→0 | npm-test.log: cargo PATH 미발견. 프로세스 PATH에 기존 C:/Users/user/.cargo/bin 추가 후 npm-test-path.log:132 PASS. 전역 설정 미변경 |
| python tools/check_assets.py | 0 | assets-first.log, 0 errors |
| python tools/check_docs.py | 0 | docs-first.log, 0 errors/warnings |
| node tools/maps/capture_preview.mjs | 1→0 | 처음 Vite root 미지정으로 404 원실패 capture-first.log. own PID9940 identity 확인 후 종료하고 positional client root로 재기동. capture-full-width.log exit0 |

## 증거

- E1: land/lakes source-manifest 및 원 ZIP SHA, defines/metadata. 최초21097 bake에서 geometry 연결성을 보존하며 seed 수 조정 후14510 실제 ID. 조정은 게임 콘텐츠 변경이 아니다.
- E2: `target/evidence/WORLD-PREVIEW/world.png`, `europe.png`: 격리 headless Chrome 1600×1000 실제 rendered page. WebGPU/NVIDIA Turing, WGSL10539, frames14/10. 오류 없음. 실제 interactive foreground UI 검증/성능 벤치라고 주장하지 않는다.
- E3: browser-process.json 및 browser-capture.json: own headless PID/argv/cwd/profile, console/requests/frame/adapter/camera. 사용자 desktop/tab/focus/globalinput 조작 없음. 최신 사용자 지시 이전 CUA inventory read-only 1회 후 중단. 모든 screenshot은 ownpage만.
- E4: server-process.json, vite.stdout/stderr.log: own node Vite local127.0.0.1:4317, WT cwd, port/argv/PID. 실행: `node client/node_modules/vite/bin/vite.js client --host 127.0.0.1 --port 4317 --strictPort`; URL `http://127.0.0.1:4317/?world-preview=1`.
- 초기 generic shell CSS가 main960px/section 스타일을 preview에 적용하는 실제 캡처에서 발견, preview-local override 후1600 full viewport 재촬영. 최종 첫 캡처는 override 이후만.

## ADR

- [ADR-3201](../adr/ADR-3201-world-preview.md), [계약](../plans/WORLD-PREVIEW-schema.md).

## 원작 대비 차별화 초안 (01 §11에 넣을 행)

- 원작 경계·UI·국기 없이 독립 generalized geography/구면 분할, 지리 provenance와 실제 누락 표시. HUD는 후속 권위 표시 슬롯만 제공하며 게임 기능 미연결을 표시한다.

## 결정 필요

없음. 국가/역사/주/게임 수치 승인은 이번 시각 요청에 포함되지 않아 구현하지 않았다.

## 범위 밖 발견

- 원 large renderer bundle 경고. 기존 renderer palettes의 반복 linear find 비용이 약180ms/update 관측. 기존 renderer 강도 변경 없이 우선 재사용.

## 못 한 부분과 이유

- 첫 체크포인트: DEM·능선·하천·도시 density 미반영; REQ-MAP-11 전체 미완료. source resolution 이하 섬/호수는 미보존 가능.
- 3D unit leaf asset 부모 인계 대기; 화면 unit slot은 아직미연결. 최초2DPNG는 등록/copy하지 않는다. 이후 실제3D sample overlay 요청을 별도 후속 체크포인트에서 연결한다.
- zoom/pan/select/border/모든region/resize/WebGL2·Rustfmt/clippy/workspace/hash2 및 반복bake 종합검사는 후속 진행. 기존 독립PASS/mainCI/WP 전체/M5 완료를 선언하지 않는다.

## 수정 기록 (P-06)

Vite root/cargo PATH/preview CSS 문제 원실패 보존 후 수정.

## 통합 기록 (P-07)

구현 브랜치 commit까지만. main merge/push/tag/public hosting 없음.
