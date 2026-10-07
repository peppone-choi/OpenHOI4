# WORLD-PREVIEW P06-3 지리 coverage·시가지·해상 계약

출발 clean `ab64f2d4ef0ce3f001e90be91ae15946c51f880d`. 같은 구현 소유의 후속이다. 최신 부모 DIRECTIVES와 유효한 원2d920 P05-2 F03/F04 FAIL을 읽었다. 이전 봉인·실패·native·후보와 원 GLB/recipe를 보존한다. 새 증거는 `target/evidence/WORLD-PREVIEW-quality-P06-3`만 쓴다. 공유 부모 문서·게임/프로토콜·원 검증/기대값·CI·의존성/locks를 바꾸지 않는다.

## 사용자 목표와 판정 범위

전체 육지+바다+호수 **20,000~30,000**은 사용자의 제작 목표이며 최소20,000을 지향한다. 30,000을 새 절대 상한으로 확정하거나 육지만20~30k로 해석하지 않는다. u16 one-based1..65535는 별도 기술 상한이다. 총계만으로 품질을 판정하지 않는다. 본토/큰섬/도시/연안·대양/호수와1pixel 통계를 각각 기록하고 과도한 작은 조각의 선택 가독성을 검사한다. 도시 전투/인구/승점/경제/항만/해상전투/전략해역 그룹·국가/주/역사1936 기능은 없다.

CEO aggregate 조사는 현대 본체 전수 경계 자료가 아니며 생성 입력으로 사용하지 않는다. 공개 원작/모드 count의 범위·버전·community/port/역사값 한계를 유지한다. 원작/모드 지도·provinceCSV·ID색·경계·수치표 다운로드/trace/인수 없음. TFR 플레이어 경험은 정성적 사례이며 실제 전수 경계/화면을 읽었다고 주장하지 않는다.

## 지리 면적과 coverage

먼저 원8K의 source-kind 연결 component와 실제 province ID를 직접 계수한다. Ireland(-8,53)의 섬은 북아일랜드까지 포함한 연결 지리이며 국가 영토가 아니다. Great Britain/Honshu/Hokkaido/Kyushu/Shikoku와 주요 대륙·큰섬을 같은 방법으로 검사한다. 대륙별 display-window 통계는 기하 창의 잘린 면적임을 따로 표시한다. 각 component의 실제 pixel수·land IDs·분포·largestshare·bbox·구면 근사 km²와 citypoint coverage를 기록한다.

면적은 평균 지구반지름을 defines에서 읽어 `R² Δλ (sin φnorth − sin φsouth)`로 cell 면적을 계산하여 합한다. 구면 cell-center 원천 raster에 의한 근사이며 WGS84 타원체/조사측량 면적이나 게임 값이 아니다. Plate Carrée pixel수만 면적으로 부르지 않는다. 표본 주변 physical 거리도 같은 반지름을 사용한다.

공통 생성 후보: 각 source-kind 연결 육지의 면적/실제 해안 거리·도시 위치·validDEM slope에 따른 시드 할당을 보장한다. 큰 섬이 전역 random draw에서 비는 구조를 제거하고 본토/섬 전체에 동일 알고리즘을 적용한다. Ireland만의 특례·uniform rectangular provinces·경계 noise 없음. 작은 원천 섬/호수/수로를 삭제하거나 단절 지리를 merge하여 숫자를 맞추지 않는다. 기존4K graph/8K coast 변환의 kind 불변·무anchor 유효case와 마지막4-connected 정규화를 유지한다.

## 독립 시가지

후보는 [Natural Earth Urban Areas](https://www.naturalearthdata.com/downloads/10m-cultural-vectors/10m-urban-area/)의 polygon SHP: 공식 설명은2002–2003 MODIS1km에서 유래한 dense habitation이며 일반화1:10M이다. 행정 경계/도시 인구 LandScan 레이어를 사용하지 않는다. 공개 [public-domain 조건](https://www.naturalearthdata.com/about/terms-of-use/)과 실제 고정 ZIP embeddedversion/URL/date/SHA/CRC/형식·좌표를 확인한다. 취득은 별도30초/20MiB 한도·새파일/원실패 보존, 생성은 offline이며 runtime API/tile/CDN/font가 없다. 새 dependency/APIkey/서비스/설정 없음.

시가지 polygon mask를 actual land에만 교차하고 각 연결 시가지를 주변 nonurban land와 별도 raw ID로 구획한다. 실제 강/호수/바다는 원kind로 남는다. 도심 point seed만으로 urban footprint를 대신하지 않는다. 거대 시가지 내부 분할은 실제 면적/하천/도시 위치에 따른 표시 후보로만 검토하며 모든 도시의 개수 규칙을 확정하지 않는다. source footprint 밖은 **nonurban-in-source**이고 교외/농촌을 권위 분류하지 않는다. 2002–2003 시대/일반화·소실/누락을 UI와 metadata에 명시한다.

Dublin/London/Seoul/Tokyo 고정 geographic sample에 가장 가까운 실제 도시 point 좌표·distance, source footprint cell/면적·ID·province urban fraction·주변 nonurban/교외 후보·인접/hover/pick/선택 가독성을 같은 scale로 대조한다. point가 source footprint 밖이면 누락을 표시하고 buffer/임의 행정구역을 만들어 도시라고 부르지 않는다. 새 필드는 preview JSON에만 additive이며 authority/proto/save는 바꾸지 않는다.

## 해상 분할

sea 고유ID·경계·인접·hover/pick/면적을 표시한다. lake와 sea 및 미구현 상위 전략해역을 구별한다. 실제 source 해안 주변을 physical 거리/spacing으로 상대 세분하고, 넓은 대양은 면적 가중 독립 시드로 상대 넓게 할당한다. 같은-kind graph가 수로/섬/호수 경계를 지키며 dateline seam도 검사한다. English Channel/Irish Sea/Japan 근해/Atlantic 실제 province 통계·경계·선택 화면을 같은 지리 축척으로 제출한다. 미세 sea/lake로 목표 개수를 채우지 않는다.

## 계약·검수

8192×4096 RG8u16LE/dense1based/256lookup/8K texture capability/orthographic camera/CPU selection·shader assert·fallback·원 GLB bytes 유지. bake180초/3GiB/index64MiB 그대로 감독하고 초과면 실패를 보존한다. 늘릴 필요가 생기면 실제 측정과 ADR부터 작성한다. 새 기술 수치·색·카메라·density·area limits는 defines/metadata이며 게임 계수가 아니다.

유효case별 RED→GREEN, source SHA/CRC/missing/coords/urban-vs-nonurban/large-component coverage/sea samples/knownCaspian미포함/F03edge0/F04u16overflow·noanchor, allpixelkind/urbanclass·everyID4connected/bbox/대표DEM/adjacency/seam/같은 source두bake4SHA. 같은최종renderer의ab64 oldbaked vsnewrealPNG는 과거 전체source재검수와 구별한다. firstIreland/UK/Dublin/sea proof를 clean 여부와 함께 즉시 부모 인계한다. 기본회귀/Rust hashes·원M1단일전체·license/Fluent/docs/assets를 끝까지 실행하고 cleanHEAD·native·실제adapter/frame·HTTP/pageerror·actualabort/unknownfailure·ViteHMR/gameWS별로 봉인한다. own 최소 headless Chrome/profile/PID/localhost만 사용한다. 독립P05/mainCI/전체WP·M5 판정은 부모 몫이다.
