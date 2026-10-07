# 검증 리포트 — WORLD-PREVIEW-M2-r2-P05-1

| 항목 | 값 |
|---|---|
| 판정 | **FAIL** |
| 검증자 | Codex 독립 검증 세션. 구현 비참여, 추적 파일·index 무변경 |
| 브랜치·커밋 | detached HEAD `a28f49c03f572a637aa699ca292810c9c9117896` |
| 검증 일시 | 2026-10-07 16:46~17:10 KST |
| 전용 worktree | `E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1` |
| 범위 | a28의 첫 세계 지도·미연결 HUD·실제 3D 샘플 표시 |

FAIL 사유는 preview 현지화와 기존 Fluent 검사 사이의 불일치, 그리고 단일 전체 E2E 실행의 녹색 증거 미충족이다. 지도 데이터와 실제 3D 표시 검사는 통과했다. 기존 WP의 이전 PASS를 소급 취소하거나, 후속 지도 개선 결과를 판정하지 않는다.

## 직접 실행한 명령

모든 명령은 전용 worktree에서 `GIT_OPTIONAL_LOCKS=0`으로 실행했다. argv·cwd·선택 환경값·PID·stdout/stderr·native exit는 [증거 디렉터리](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1)에 보존했다.

| 명령 | 종료 코드 | 결과 요약 |
|---|---:|---|
| `npm --prefix client ci` | 0 | 설치 성공 |
| `npm --prefix client test` | 0 | **134개 통과** |
| `npm --prefix client run typecheck` | 0 | 통과 |
| `npm --prefix client run build` | 0 | 통과 |
| `npm --prefix client run licenses` | 0 | 통과 |
| `cargo fmt --check` | 0 | 통과 |
| `cargo clippy --workspace --locked -- -D warnings` | 101 → 0 | 최초 dist 미생성 오류 보존. client 빌드 후 통과 |
| `cargo test --workspace --locked` | 101 → 0 | 같은 빌드 순서 오류 후 최종 **179개 통과** |
| `cargo build --workspace --locked` | 0 | 자기 dist/target 사용 |
| M0·M1 365일 실행, 각각 두 번 | 각 0 | 각 해시 일치 |
| `oh_cli validate --deny-warnings data/packs/testland` | 0 | 통과 |
| `cargo deny check licenses bans advisories sources` | 0 | 네 검사 통과 |
| `python tools/check_docs.py` | 0 | 오류 0, 경고 0 |
| `python tools/check_assets.py --release` | 0 | 오류 0 |
| `python tools/check_architecture.py` | 0 | 통과 |
| `python tools/check_localisation.py` | **1** | preview 키의 ko/en 메시지 누락 진단 **68개** |
| map Python tests | 1 → 0 | 최초 런타임의 scipy 부재 보존. 설치된 Python 3.11.0에서 9개 실행, 8개 통과·1개 skip |
| 오프라인 bake ×2, 명시적 ignored 출력 | 각 0 | 원본과 4파일 SHA 전부 일치 |
| GLB recipe 재생성, 별도 출력 | 0 | 세 모델 원본 bytes 일치 |
| 원천 mutation 6종 | 각 1, 의도한 거부 | missing/hash/path/ZIP/SHP/ID 오류, 산출 디렉터리 미생성 |
| 독립 데이터·GLB 검사 | 0 | 전체 픽셀·연결성·인접·형상 통과 |
| 최종 headless page 검사 | 1 | 요구 최소 크기 검사 통과. 최소 크기 밖 720px 영문 overflow 추가 관찰 |
| 실제 scene·cleanup·abort 추가 검사 | 0 | 통과 |
| 기존 M1 E2E 첫 전체 실행 | 1 | 40 통과, redirect 증거 수집 1 실패 |
| 보안·가속 기본 인자를 제외한 전체 E2E | 1 | 39 통과, scrollbar에 따른 폭 차이 2 실패 |
| headless scrollbar 조건을 맞춘 해당 두 원 테스트 | 0 | 기대값 변경 없이 2 통과 |

추적 TS generator와 tracked bake writer는 실행하지 않았다. `req_net_04_generated_types_are_current`가 생성 문자열과 추적 TS를 읽기 전용으로 직접 비교하여 통과했다.

## REQ별 확인

| REQ·계약 | 검사하는 테스트·증거 | 실제 검사 여부 | 결과 |
|---|---|---|---|
| REQ-MAP-01/02/07 표시·원천 일부 | 독립 SHP parity, 전체 index·dense ID·4-connected·seam adjacency | 직접 실행 | PASS |
| REQ-MAP-04/05 표시·입력 일부 | 실제 Three canvas, 원 index 선택 ID, preset·zoom·pan·clear·모드·경계·resize/DPR | 직접 실행 | 첫 표시 범위 PASS; 전체 E2E 증거는 아래 FAIL 항목 참조 |
| REQ-MAP-10 오프라인·provenance | 공식 ZIP 재취득 SHA, embedded version, 파일 집합, 입력 거부, 반복 bake | 직접 실행 | PASS |
| REQ-MAP-11 | metadata 제한과 disabled terrain 확인 | 직접 확인 | DEM·능선·하천·도시 밀도 미반영 |
| HUD 표시 계약 | OpenHOI4, 날짜·자원 placeholder, 국가 미선택, rail 8개 disabled | 직접 실행 | PASS |
| KO/EN·REQ-LOC-01/03 | 실제 전환 및 기존 Fluent 검사 | 직접 실행 | 화면 전환 PASS, 검사 통합 **FAIL** |
| 실제 3D 계약 | binary decode·재생성·원 제작 bytes·scene·확대 crop·on/off·dispose/abort | 직접 실행 | PASS |
| 기존 backend/pipeline·권위·입력 보존 | 변경되지 않은 M1 E2E | 직접 실행 | 성공·실패를 구분 보존; 단일 전체 녹색 없음 |
| REQ-NET-04 | generated TS equality test | 직접 실행 | PASS |

### 지도·원천 결과

- WGS84 Plate Carrée, extent `[-180,-90,180,90]`, x 동쪽/y 남쪽, **4096×2048** 확인.
- **8,388,608픽셀**, province ID `1..14510`, index의 zero-based `0..14509` 및 u16 LE 구분 확인.
- 실제 개수: **land 7,881 / sea 3,960 / lake 2,669**.
- 14,510개 전부 4-connected. sorted unique adjacency **29,294쌍**, seam **86쌍** 확인.
- 독립 SHP reader와 ring parity로 한반도·히말라야·미주·남극·섬·호수 등 15개 좌표를 대조했다.
- Natural Earth 공식 페이지의 [land 5.1.1](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-land/), [lakes 5.0.0](https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-lakes/), [public domain 조건](https://www.naturalearthdata.com/about/terms-of-use/)을 확인했다. 공식 ZIP 재취득 bytes도 고정 원천 SHA와 일치했다.
- `index.bin/provinces.json/adjacency.json/metadata.json`은 원본·bake 1·bake 2 모두 동일했다.

세부 값은 [독립 검사](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/independent-audit.json), [재생성 비교](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/repro-comparison.json)에 있다.

### 실제 화면·3D 결과

최종 인정 화면은 `browser-minimal/`과 `supplement/`이다. Chrome **154.0.8037.98**의 실제 headless 전용 profile을 사용했다. 최종 argv는 headless·profile·debug transport 인자만 포함하며 sandbox를 유지했다.

- WebGPU: **NVIDIA Turing**, WGSL·실제 frame 확인.
- forced WebGL2: **ANGLE / GTX 1660 Ti / D3D11**, 실제 frame 확인.
- 전세계·유럽·한반도·히말라야·미주, 선택·확대·이동·표시 전환·DPR2 캡처 보존.
- 원 index 픽셀 `(2200,423)`의 **ID 4388**과 hover/click 선택 일치.
- 정상 page console error·pageerror·HTTP 404 없음. page resource는 same-origin이며 게임 `/ws` 요청 없음.
- GLB 외부 buffer/image/texture, empty mesh, 잘린 header, 잘못된 magic/hash/path/중복/redirect를 거부했다. 외부 page 요청 없이 지도 표시를 유지했다.
- 세 모델 **22개 mesh**, army/air/navy **160/144/152 triangles**. 원 제작 `19cf3cf…`의 관련 10파일과 현재 bytes, public copy, metadata SHA, Node recipe 재생성 bytes 일치.
- map scene의 **6개 샘플·912 triangles**, opaque PBR·유한 벡터·유효 indices·CCW normals·3축 bbox·ground pivot·회전 transform 확인.
- cleanup은 scene에서 샘플·조명을 제거했다. 공유 자원은 유지한 뒤 geometry 22개/material 10개를 각각 dispose했고 abort를 거부했다.
- 확대 crop에서 탱크·항공기·함선을 구분했다. 지역 샘플은 여러 프로빈스를 덮는 큰 표시 크기이며, 전세계에서는 작게 보인다. 최종 아트 크기 승인으로 해석하지 않는다.

[전세계](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/browser-minimal/world.png), [유럽](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/browser-minimal/europe.png), [3D scene 결과](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/supplement/result.json).

## 결정론

| 구분 | 해시 1 | 해시 2 |
|---|---|---|
| M0 | `b039d35666b77fc2` | `b039d35666b77fc2` |
| M1 | `b595dc2a1e5b4f8c` | `b595dc2a1e5b4f8c` |

저장·새 process 재개 및 movement/strait 저장 경계 테스트도 workspace 실행에서 통과했다. 세계 미리보기에는 새 게임 저장 상태를 추가하지 않았다.

## 금지 사항 점검

| 항목 | 결과 |
|---|---|
| 시뮬레이션의 f32/f64 | oh_sim/oh_ai source 검색에서 발견 없음 |
| HashMap 순회·thread_rng·SystemTime | 해당 source 검색에서 발견 없음 |
| 코드 내 수치 리터럴 | 지도 표시 값은 defines, 모델 아트 값은 recipe. 새 게임 수치 없음 |
| 미결정 규칙 구현 | 이번 preview에 국가·역사·이동·전투 규칙 없음 |
| 골든 기대값 변경 근거 | 테스트·기대값·fixture 변경 없음 |
| 클린룸 | 독립 지리·자가 제작 procedural GLB 사용. 원작 자료 추출·복사·트레이싱 없음 |
| desktop·CUA·전역 입력 | 사용 없음. 읽기 전용으로 얻은 foreground/cursor 전후 동일, restore 없음 |
| 브라우저 실행 제약 | 초기 Playwright 기본 인자 포함 시도는 보존하고 최종 증거에서 제외. 최종은 가속·보안·권한 override 없이 sandbox 유지 |
| 추적 파일·index | 최초 부모 baseline과 최종·봉인 후 모두 동일 |

## 작업 로그 증거와 실제 결과 대조

- 첫 지도/HUD/실3D 주장은 실제 canvas·mesh·page 캡처로 확인했다. PNG/billboard를 3D로 사용하지 않았다.
- producer a28 원 캡처 ZIP **16 members**와 원 SHA를 대조했다. 3D 제작 ZIP은 **67 payload + checksum manifest = 68 archive members**로 전수 확인했다.
- e209의 resource404와 a28을 섞지 않았다. 최종 a28 정상 page의 response/location 로그에서는 404가 없었다.
- 실제 npm 테스트 수는 **134**다. 작업 로그의 이전 132개 기록과 구분한다.
- DEM·하천·도시 분할은 현재 지도에 미반영이다. 균일 nearest-seed가 최종 형상이라는 승인이 아니다.
- preview는 별도 ko/en dictionary와 시스템 폰트를 사용한다. Fluent 검사 통합과 이 화면의 자체 호스팅 CJK 폰트 사용 PASS를 주장하지 않는다.
- 720px 영문 레일 overflow는 요구 최소 창 크기 **1280×720 밖**의 추가 관찰이다.
- 최초 scipy 부재, dist 생성 전 Rust 실패, 검증 config ESM 오류, ZIP 개수 해석 오류, summary decoding 오류 및 부적절한 색 개수 heuristic은 검증 환경·절차 오류로 원 기록을 보존했다. 제품 테스트를 약하게 만들지 않았다.

봉인한 [evidence.zip](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/evidence.zip)은 **57,992,677 bytes / 534 members**다.

- ZIP SHA256: `db5f74efcd5e9d808e384cd97a4dedae741e8b5025c4df4963bf97e22a84ecba`
- manifest SHA256: `6b97827f4aeb3951113ca2ea853eb78ed6a3a6b156c7103fecbde76fbd6c0c5b`
- raw index SHA256: `af99860cc56909da1019bd6aa5ad01250efd821672d7ea471604d8589a32c0c1`

봉인 후 **533개 payload bytes**와 ZIP SHA를 재확인했다. HEAD·9,999개 추적 목록/각 SHA·semantic/raw index·diff/cached diff/status는 부모 original-before와 동일하다. [봉인 identity](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/ZIP-IDENTITY.json), [봉인 후 비교](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/POST-SEAL-COMPARISON.json).

## FAIL 항목

| # | 재현 명령 | 기대 | 실제 |
|---|---|---|---|
| F01 | `python tools/check_localisation.py` | ko/en Fluent 등록과 누락 검사 exit 0 | exit **1**. `PreviewHud.tsx`·`WorldPreview.tsx`의 `preview`, `country`, `unconnected`, `sample` 등 **68개 누락 진단**. 별도 dictionary가 기존 Fluent 계약과 맞지 않음 |
| F02 | ignored `e2e.py final/minimal/scrollbars` | 같은 실행 조건의 전체 기존 E2E 녹색 증거 | 첫 전체 **40 PASS/1 FAIL**: `map-redirect.spec.ts:155`에서 navigation 중 증거 조회 실패. 최종 minimal 전체 **39 PASS/2 FAIL**: scrollbar 폭 839와 baseline 854 차이. 해당 두 원 테스트의 조건을 맞춘 실행은 PASS했지만 단일 전체 녹색으로 합산하지 않음 |

F02의 실패 원인이 a28의 제품 hook이라고 확정하지 않았다. 원 실패와 환경 차이, 후속 성공은 [E2E 결과](E:/openhoi/.orchestrator/wt/WORLD-PREVIEW-M2-r2-verify1/target/evidence/WORLD-PREVIEW-M2-r2-P05-1/e2e-failures-summary.json)에 구분되어 있다.

DEM 취득 subset 테스트 1개는 자료 부재로 skip했다. 다른 브라우저 제품·엔진, 사용자 desktop 플레이, 같은 HEAD main CI, 1936 역사 콘텐츠, WP-32/45 전체, M2 통합 및 전체 M5는 이번 검수의 미검증 범위다. 후속 HEAD의 지형·하천·프로빈스 개선에는 이 판정을 적용하지 않는다.
