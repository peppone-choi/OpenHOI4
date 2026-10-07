# WORLD-PREVIEW 최소 표시 계약

2026-10-07. 사용자 지도/HUD 선행 지시의 한정 구현이다. WP-32/45, REQ-MAP-01/02/04/07/10의 시각·원천 경로를 시험한다. REQ-MAP-03의 주/국가 콘텐츠 및 REQ-MAP-11 전체 충족을 선언하지 않는다.

- WGS84 경위도 EPSG:4326, Plate Carrée, extent [-180, -90, 180, 90]. x 동쪽, y 남쪽. 4096×2048 cell center. 표시 격자이며 프로빈스는 격자 분할이 아니다.
- 원천 ZIP 내부 polygon SHP는 오프라인 읽기. land v5.1.1, lakes v5.0.0 Natural Earth public domain. 10m은 1:10 million 축척. ZIP·버전·URL·취득일·SHA256은 source-manifest.json. 누락/불일치 즉시 실패. 취득 명령과 생성 명령 분리.
- 무작위 고정 PCG64 seed와 면적 가중 지리 구면 nearest-seed 셀을 육지/바다/호수별 독립 생성. raster 4-connected 조각은 각각 별도 ID. dateline은 맞닿는 인접으로 기록하되 양쪽 raster 조각은 별도 ID. 극점은 cell center ±89.956°까지. 원천보다 작은 섬/호수는 소실 가능. 현재 도시/하천/고도 기반 density·능선은 미반영, 추후 실제 취득에 따라 갱신.
- province ID = 1-based dense ascending u16; index.bin은 dense zero-based u16 LE, R=low byte/G=high byte. ID/RGB mapping은 metadata/provinces JSON. RGB는 ID별 deterministic 표시색. ID는 preview 범위에 한정하며 게임 stable ID가 아니다.
- kind land/sea/lake; pixels, bounds, representative cell lon/lat, geography_color, province_color. elevation은 실제 원천 적용 전 null. adjacency는 raster 수평·수직 및 seam에서 생성한 sorted unique ID pair. 해협/이동/전투 규칙 없음.
- metadata.json schema=world-preview-v1, projection/extent/resolution/options/tools/sources/counts/limitations/hash. provinces.json 및 index.bin/adjacency.json SHA256. 생성 출력 순서는 ID·pair 오름차순, JSON key sort, UTF-8 LF. 동일 입력/options/versions에서 2회 산출 비교.
- Three createMap/model adapter는 WorldView의 display 구조만 재사용한다. nations/states=[]; owner/controller/state/colors=null; tick='0'은 어댑터 자리이며 시뮬레이션 결과가 아니다. 화면에는 게임 날짜/자원 미연결 표시. gameplay/protocol/data pack/save 변경 없음.
- style와 카메라 프리셋 등 표시 설정은 별도 defines.toml에 두고 metadata에 bake. 초기 mode geography, province border on. terrain mode는 실제 DEM 없을 때 disabled. zoom/pan/hover/click/presets만 연결. HUD props/slot을 분리하고 게임 버튼은 disabled.
- runtime은 /preview/world/ baked same-origin files만 요청, redirect:error, SHA/ref/length 검증. 외부 API/tile/CDN/font 없음. 원 App은 ?world-preview=1에서만 별도 React 화면으로 교체.
- generated marker는 부모가 exact bytes/provenance를 인계할 때만 등록·연결. 별도 마커 생성/원천 덮어쓰기 없음. 샘플 marker는 실제 군대가 아니다.

## 완료 조건

로컬 브라우저 전체 세계/지역 캡처, actual ID 개수, 확대·이동·선택·경계 전환, backend/adapter/frame/console/network 증거, 반복 SHA 일치, 기존 client/docs/assets 검사, clean checkpoint commit. 독립 검증 PASS/main CI/WP 전체 완료 판정은 이 구현 세션의 범위 밖.

## 최신 사용자 3D 샘플 표시 계약

부모가 실제 3D unit 요청과 leaf 소유를 인계했다. `client/src/map/renderer.ts`의 선택적 `presentation.sceneExtension(scene, camera)` hook만 추가 소유한다. 기본 경로에는 hook이 없어 기존 scene/plane/input/backend/shader/assert를 보존한다. hook은 compileAsync 전에 실제 GLB geometry/PBR material과 조명을 같은 scene에 추가하고 cleanup 함수를 반환한다. 모든 실패·정상dispose에 cleanup을 호출한다. main plane z=0, 모델은 +Yup/+Zforward 원 좌표에서 preview transform으로 mapXY에 배치하고 z>0·깊이검사·조명을 사용한다. 지리 camera는 기존 OrthographicCamera다. 별도 gameplay 카메라/command/schema 없음.

`client/src/preview/units3d.ts`가 부모 exact metadata/sha를 확인한 local GLB만 fetch/parse한다. external GLB URI/images/textures를 거부한다. 샘플 위치/방향/아트스케일·조명은 preview defines 및 sample metadata로 표시용 값만 정의한다. 실제unit/nation ID 없음. 모델은 sample decoration이며 데이터 연결 미완료를 HUD에 표시. clone별 geometry/material는 shared dispose 중복을 피하고 scene에 추가한 node/lights 제거와 resources 해제를 수행한다. 모델 파일명/manifest/pathescape/hash/reference/GLB2 header 검증 실패는 명확히 표시하고 없는 모델을 sprite로 대체하지 않는다. PNG billboard를3D로 부르지 않는다.
