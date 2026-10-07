# 검증 리포트 — WORLD-PREVIEW-M2-r2-P05-2

| 항목 | 값 |
|---|---|
| 판정 | **FAIL — 유효한 독립 검증** |
| 검증자 | Codex 새 검증 세션. 구현 비참여, 추적 파일·index 무변경 |
| 브랜치·커밋 | detached HEAD `2d920ea86c6d8045adb7d3115007230e52c3ccf2` |
| 검증 일시 | 2026-10-07 17:31~17:55 KST |
| 전용 worktree | `E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2` |
| 범위 | 세계 미리보기·지리 입력·미연결 HUD·실제 3D·Fluent 수정·기존 회귀 |

**기존 F01·F02는 이번 HEAD에서 통과했다. 새 F03·F04 경계 계약 실패로 최종 판정은 FAIL이다.** 현재 13,413개 세계 데이터의 재생성·표시·3D 검사는 통과했다. 후속 지역 품질 개선, WP-32/45 전체, M2/M5 완료를 판정하지 않는다.

## 직접 실행한 명령

모든 명령은 전용 worktree에서 `GIT_OPTIONAL_LOCKS=0`으로 실행했다. argv·cwd·PID·stdout/stderr·native exit는 [증거 디렉터리](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2)에 보존했다.

| 명령·검사 | 종료 코드 | 실제 결과 |
|---|---:|---|
| `npm --prefix client ci` | 0 | 설치 성공 |
| `npm --prefix client test` | 0 | **135개 통과** |
| typecheck / build / npm licenses | 각 0 | 통과 |
| map Python tests | 0 | **11개 통과**, 실제 고정 DEM 검사 포함·skip 없음 |
| docs / assets `--release` / architecture | 각 0 | 통과 |
| 기존 `check_localisation.py` | 0 | 기존 Fluent 경로로 통과 |
| ignored catalog에서 실제 필수 키 제거 후 같은 checker | 1 | 예상한 누락 거부 |
| `cargo fmt --check` | 0 | 통과 |
| `cargo clippy --workspace --locked -- -D warnings` | 0 | 통과 |
| `cargo test --workspace --locked` | 0 | **179개 통과** |
| `cargo build --workspace --locked` | 0 | 자기 dist/target 사용 |
| `cargo deny check licenses bans advisories sources` | 0 | 통과 |
| Testland `validate --deny-warnings` | 0 | 통과 |
| M0·M1 365일 실행 각각 두 번 | 각 0 | 각 해시 일치 |
| 오프라인 bake 두 번, 명시적 ignored 출력 | 각 0 | 원본·두 산출의 4파일 SHA 일치 |
| GLB recipe, 별도 ignored 출력 | 0 | 세 모델 원본 bytes 일치 |
| 입력 거부 9종 | 각 1 | 예상한 거부, 원천·기존 출력 보존 |
| 독립 원천·raster·고도·graph·GLB 검사 | 0 | 정상 데이터 검사 통과, 경계 실패 별도 기록 |
| 원 source capture / production capture | 각 0 | 각각 실제 화면 15개 |
| 최종 독립 browser 검사 / silhouette / client limits | 각 0 | 실제 표시·자원 해제·경계 관측 |
| 기존 Chromium 전체 E2E | 0 | **단일 실행 41 PASS**, skip/flaky 0 |
| 기존 Firefox 전체 E2E | 0 | **단일 실행 41 PASS**, skip/flaky 0 |
| 기존 WebKit 전체 E2E | 0 | **단일 실행 41 PASS**, skip/flaky 0 |
| `edge0-repro.py` | **1** | F03 실제 오류 |
| `id-limit.py` 및 실제 client validator | 0 | **65,536개 허용을 직접 관측 — F04** |

추적 TS generator·tracked bake writer는 실행하지 않았다. `req_net_04_generated_types_are_current`의 읽기 전용 equality 검사가 통과했다.

준비 오류와 검증 helper 오류는 원문을 보존했다. cargo executable 경로, ignored E2E config 문법·webServer cwd, negative 화면의 오류 문구 selector, Vite HMR을 게임 WS로 오인한 단언을 바로잡았다. 원 테스트·checker·timeout·retry·workers·기대값은 변경하지 않았다. 부분 실행을 합산하여 전체 PASS로 기록하지 않았다.

## REQ별 확인

| REQ·계약 | 실제 검사 | 결과 |
|---|---|---|
| REQ-MAP-01/02/07 표시·원천 일부 | 전체 픽셀·ID·연결성·인접·좌표·원천 mask | 현재 데이터 PASS, ID 한도 계약 F04 |
| REQ-MAP-04/05 표시·입력 일부 | 실제 canvas·raster 선택 oracle·프리셋·zoom/pan·clear·경계·모드·resize/DPR | PASS |
| REQ-MAP-10 | 공식 원천 재취득 SHA·입력 거부·오프라인 bake 반복 | PASS |
| REQ-MAP-11 일부 | 실제 river/DEM 입력·대표 고도·edge cost·corner cut·누락 경계 | 정상 예제 PASS, edge 없는 fallback F03 |
| HUD 미연결 계약 | 국가·날짜·자원 placeholder·8개 disabled rail·게임 `/ws` 부재 | PASS |
| REQ-LOC-01/03·F01 | 실제 KO/EN·Fluent 등록·translatorFor·진짜 누락 거부 | PASS |
| 실제 3D 계약 | binary·원 제작 bytes·재생성·scene·깊이·on/off·cleanup·negative | PASS |
| 기존 renderer·F02 | 변경되지 않은 전체 E2E, 세 엔진 각각 단일 실행 | PASS |
| REQ-NET-04 | 추적 TS 생성 문자열 equality | PASS |

### 지도·원천

- WGS84 Plate Carrée, `4096×2048`, **8,388,608픽셀** 확인.
- 실제 **13,413개 = land 7,103 / sea 3,641 / lake 2,669**.
- ID `1..13413`과 zero-based LE ordinal `0..13412` 구분 확인.
- 전부 4-connected, sorted unique adjacency **26,813쌍** 확인. 단일 픽셀 province는 **3,989개**다.
- 전체 source-kind mask, bounds·pixels·대표 좌표, seam 및 독립 SHP parity 좌표 검사를 실행했다.
- 공식 [land 5.1.1](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-land/), [lakes 5.0.0](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-lakes/), [rivers 5.0.0](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-rivers-lake-centerlines/), [이용 조건](https://www.naturalearthdata.com/about/terms-of-use/)을 확인했다. 세 ZIP과 NOAA 고정 TIFF를 재취득하여 SHA가 모두 일치했다.
- NOAA TIFF의 `1008×480`, extent `[74,25,95,35]`, 75 arcsec/F32/PixelIsArea/EPSG:4326·finite 범위를 확인했다. 설치된 독립 TIFF decoder와 제품 reader의 **전체 픽셀이 일치**했다. metre·EGM2008·CC0는 [공식 제품 metadata](https://www.ncei.noaa.gov/access/metadata/landing-page/bin/iso?id=gov.noaa.ngdc.mgg.dem%3Aetopo_2022)와 대조했다.
- 대표 고도 **147개를 직접 재계산**했다. footprint 밖 값은 null이다.
- 작은 graph의 모든 edge를 별도 계산식과 대조했다. corner cut 거부와 양끝 DEM valid 조건을 확인했다. edge가 전혀 없는 경우는 F03이다.

세부 값은 [독립 검사](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2/independent-audit.json)와 [재생성 비교](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2/repro-comparison.json)에 있다.

### 실제 화면·3D

Chrome `154.0.8037.98`의 자기 profile·localhost·page만 사용했다. 실제 argv를 읽어 sandbox·보안·GPU 기본값 보존을 확인했다. Chromium E2E에는 별도 headless `hide-scrollbars` 조건을 명시했다.

- WebGPU **NVIDIA Turing**, texture limit **8192**, WGSL·실제 frame 확인.
- 명시적 WebGL2도 실제 adapter·shader·frame 확인.
- world/Europe/Korea/Himalaya/Americas·geography/province/terrain·선택·해제·zoom/pan·경계·KO/EN·resize·DPR2 검사.
- 선택 ID를 원 index 픽셀 계산과 직접 대조.
- production 15개 캡처의 console/page/HTTP 오류·failedRequests·외부 요청 **0**.
- 개발 페이지의 `ERR_ABORTED`는 원 timing/events와 StrictMode cleanup을 대조해 별도 보존했다. 자기 Vite HMR과 게임 `/ws`를 구분했다.
- 실제 GLB 세 개·**22 mesh**, **160/144/152 triangles**. producer `19cf3cf…` 원문·public copy·recipe 재생성 bytes 일치.
- 실제 scene **6개 샘플·912 triangles**, finite geometry·indices·CCW normals·3축 bbox·ground pivot·opaque material·light·camera·depth 확인.
- 샘플 on/off 차이 **13,335픽셀**. 확대 army 차이 **208,490픽셀**. 모델을 지도 뒤로 옮긴 검사 화면은 off와 **차이 0**으로 깊이 가림 확인.
- cleanup은 샘플·조명을 제거하고 shared geometry 22개/material 10개를 각각 해제했다. abort 거부 확인.
- GLB URI/images/textures/header/hash/path/empty geometry를 거부하면서 지도를 유지했다.
- 자기 Chrome/Vite PID 소멸 확인. Chrome native exit 0과 Windows Vite 명시 종료의 native exit 1을 구분했다.

[세계 화면](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2/browser-independent-final/world.png), [한반도 화면](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2/browser-independent-final/korea.png).

**관찰 한계:** 850×650 KO/EN rail 하단은 687px, army 버튼 하단은 655px까지 이어져 잘린다. 1280×800·1600×1000에는 overflow가 없다. preview schema에 지원 최소 크기가 명시되지 않아 850을 임의로 미지원이라고 선언하지 않았다. 한국 해안 pixelation·지역 밀도 부족, subpixel 섬, Caspian 기본 lake 원천 누락은 남아 있다.

## 결정론

| 구분 | 해시 1 | 해시 2 |
|---|---|---|
| M0 | `b039d35666b77fc2` | `b039d35666b77fc2` |
| M1 | `b595dc2a1e5b4f8c` | `b595dc2a1e5b4f8c` |

저장·재개 회귀는 workspace 실행에서 통과했다. preview에 새 게임 저장 상태는 없다.

## 금지 사항 점검

| 항목 | 결과 |
|---|---|
| 시뮬레이션 f32/f64·HashMap·thread_rng·SystemTime | 검색 결과 없음, 해당 제품 범위 변경 없음 |
| 게임 수치·클라이언트 규칙 계산 | 제작용 defines만 사용, 권위 규칙 없음 |
| 미결정 규칙 구현 | 국가·역사·이동·전투·HUD 명령 추가 없음 |
| 골든·저장·Testland·원 E2E/checker | base bytes 보존 |
| 새 의존성·에셋 | lock 보존, ASSETS 기존 prefix 보존 |
| 현지화 | ko/en 기존 prefix 보존, preview append와 실제 Fluent 정합 |
| 클린룸 | 생성 입력은 독립 지리 원천·자체 recipe, 원작 경계 입력 없음 |
| 사용자 desktop/CUA·focus/input·설정 변경 | 실행 없음 |

## 작업 로그 증거와 실제 결과 대조

현재 13,413개와 a28의 14,510개를 섞지 않았다. 원 a28 FAIL도 새 결과로 덮어쓰지 않았다.

- 체크포인트 **95파일·97 ZIP member·입력 24개**: 전수 불일치 0.
- 원 a28 FAIL **533파일·534 ZIP member**: 전수 불일치 0.
- 관련 exact HEAD source **70파일**과 원 테스트 출력 보존.
- 부모·시작·중간·종료·봉인 후 스냅샷의 **10,001개 목록/각 SHA/HEAD/semantic·raw index/diff/cached/status 모두 동일**.
- raw index SHA: `0f41b665fefbbe2354ba46ea443b7549a502ce2009d8dd1d385b10f397786127`.

봉인: **1,121파일·ZIP 1,122 member**, 전수 검증 성공.

ZIP SHA256: `6f24339f4c197653a92fd91ff1c1b92c85a0f8a0af9dc324c74e26c3b144df87`

[봉인 identity](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2/ZIP-IDENTITY.json), [봉인 후 불변성 비교](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify2/target/evidence/WORLD-PREVIEW-M2-r2-P05-2/POST-SEAL-COMPARISON.json).

## FAIL 항목

| # | 재현 명령 | 기대 | 실제 |
|---|---|---|---|
| **F03** | ignored `edge0-repro.py` | 유효한 diagonal-only same-kind component에서 seed 없는 component는 fallback 후 별도 연결 ID | `partition → geodesic_labels`의 edge 목록이 비어 `np.concatenate` **ValueError**, fallback 전에 종료 |
| **F04** | ignored `id-limit.py`, `limits-browser.mjs` | 문서의 **1-based u16 ID 최대 65535**와 실제 한도 일치 | generator와 실제 client validator가 **65536개 허용**. ordinal 65535가 one-based ID 65536으로 대응 |

두 실패는 synthetic 경계 계약 문제다. 현재 13,413개 고정 세계 bake의 실패라고 기록하지 않는다.

Global DEM·도시 밀도·정밀 능선/고개·강 network 정확도, 1936 국가/주 콘텐츠·게임 기능·최종 지역 품질·동일 HEAD main CI는 이번 검수에서 완료 판정하지 않았다. 이 리포트의 FAIL과 봉인 성공을 앱 종료 코드로 대체하지 않는다.
