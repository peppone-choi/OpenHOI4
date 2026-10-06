# WP-08 지도 표시 계약 설계·누락 대조

2026-10-06. 사용자 추가 schema-planning-directive 적용. 기준 01 REQ-MAP-04/05, REQ-PLAT-03, 02 §4.3/11, WP-07 지도/ADR-0901. 현재 M1 최소 표시 계약이며 국가·경제·지도 전체 스키마의 최종판이 아니다. 후속 schema 변경 전에 아래 불변성과 fixture를 대조한다. 위키 조사는 오케스트레이터 담당이며 이 세션은 접근하지 않은 내용을 근거로 삼지 않는다.

## 소유·수명 경계

| 분류 | 단일 기준 | 수명·변경 권한 | 저장/해시 | WP/REQ |
|---|---|---|---|---|
| 불변 defs | oh_data MapData/registry, visuals, nation definitions | 팩 로드시 검증; 실행 중 client 변경 불가 | pack identity 포함, save에는 defs 미포함 | WP-07/09; REQ-MAP-01/03 |
| 시나리오 초기값 | ownership/control overrides·state inputs | 팩 제작자, load_scenario에서 상태 생성 | WP-09 초기 world hash | WP-09; REQ-NAT-01/MAP-08 |
| 가변 권위 상태 | simulation WorldInputs/time/RNG/queue/modifiers/expiry | Rust sim command/tick 경계만 변경 | WP-09 canonical hash. 실제 save/restart WP-11 미구현 | WP-09/10/11; DR-01~10 |
| 파생 조회 | WorldView RGB, refs, ledger strings/tick | sim thread atomic Query에서 생성; client 읽기 전용 | 재생성; 별도 gameplay state 아님 | WP-09/08; REQ-NET-05 |
| HTTP map 계약 | 검증된 host World.defs.map | host load 동안 고정; 동일 실행의 pack hash만 받음 | raw index hash·pack identity, HTTP immutable key | WP-08; REQ-MAP-04/05 |
| 표시 설정 | m1 defines map_display | 팩 제작자; host 검증, UI/renderer 사용 | pack identity 포함, sim state hash 미포함 | WP-08; REQ-PLAT-03 |
| UI 임시 상태 | camera/mode/selected/hover | browser 입력; 규칙·수치 변경 불가 | 저장/authority hash 대상 아님 | WP-08; REQ-MAP-04/05 |

## 필드별 HTTP/wire 의미

전부 필수이며 기본값 없음. HTTP JSON은 MapMetadata Rust→generated TS가 유일 정의다. MessagePack WorldView는 기존 WP-09 DTO를 보존한다. 배열/byte 단위는 프로토콜 표현이며 게임 수치가 아니다.

| 필드 | 의미·타입/단위·범위 | null/default/참조·정렬 | 소유·변경/수명 | REQ/WP |
|---|---|---|---|---|
| map_id | registry ID string `[a-z0-9_]+` | 불변 validated map ID; URL path와 world 동일 | defs→host | MAP-04/WP-08 |
| schema_version | u16, 초기 contract=1 | 필수, 구형 누락/미래 version 명시 거부; 버전은 protocol 기술 상수 | host→generated TS/runtime | NET-04/MAP-04 |
| width/height | u32, texel 수, >0 | WorldView와 같음; 선택 backend limit 이하 | defs→host | MAP-04/PLAT-03 |
| province_ids | Vec<u16>, 실제 안정 ID 0..65535 | 중복 불가, ID 오름차순, dense 위치 0..65535; CSV 행 순서 독립 | defs→host | MAP-05/WP-07/08 |
| index.bin | RG8 bytes, little endian dense u16 | row-major top-left; 정확히 width×height×2; 모든 index < ids.length | defs→host | MAP-04/05 |
| pack_hash | 16자리 lowerhex, 현재 PackInfo identity | Welcome 값과 일치, query pack 필수; 불일치409 | host→client | NET-04/MAP-04 |
| index_hash | raw bytes FNV-1a 64 lowerhex | 수신 bytes와 대조; canonical sim state와 다른 대상 | host→client | MAP-04 |
| byte_length | decimal string, byte 수 | 정밀도 보존. 실제 HTTP body 길이와 대조 | host→client | MAP-04 |
| style.background/nation_border/state_border/province_border/selected/hovered | RGB24→[u8;3], 0..255/channel | 필수, NoColorSpace raw RGB; gameplay 효과 없음 | defines→host→renderer | MAP-05/PLAT-03 |
| style.nation_width_milli/state_width_milli/province_width_milli | u32 >0, CSS pixel ×1000 | 국가 > 주 > province 겹침 우선순위, 카메라 scale로 texel 거리 변환 | defines→renderer | MAP-05 |
| style.highlight_milli | u32 1..1000, 강조 혼합 비율/1000 | hover 후 selection 우선 | defines→renderer | MAP-05 |
| style.fit_milli | u32 1..1000, 초기 화면 점유율/1000 | 지도 종횡비 유지 | defines→camera | MAP-05 |
| style.zoom_min_milli/zoom_max_milli | u32 >0, 배율×1000; min≤max | game rule 아님; wheel 제한 | defines→camera | MAP-05 |
| style.wheel_milli | u32 >0, wheel 확대 감도 | exp 입력에 적용, 임의 게임효과 없음 | defines→camera | MAP-05 |
| style.drag_threshold | u32 >0 CSS pixel | 클릭과 pan 구분 | defines→camera | MAP-05 |
| WorldView.tick | decimal u64 string | Query 시점의 authority tick; 원장 tick과 같음 | sim→proto→client | NAT-01/WP-09 |
| mode_keys | string[] localization keys | owner/control/terrain/state; 기존 contract 보존 | proto→client | MAP-04 |
| ProvinceView.id/state/owner/controller | id u16, refs Option<u16> | 물 province state/owner/controller=null; ID0 유효; 소유와 통제 별도 | sim→proto | MAP-05/08 |
| owner_color/controller_color/state_color | nullable RGB u8³ | null은 neutral_color 사용; client 효과 계산 금지 | proto→renderer | MAP-04 |
| terrain_key/terrain_color | registry ID string/RGB u8³ | plains/hills/ocean/inland_water 현지화 키 등록; 지형효과 없음 | defs→proto→renderer/UI | MAP-04 |
| neutral_color | RGB u8³ | sea/lake null owner/state용 서버 팔레트 | visuals→proto | MAP-04 |
| nations/states/provinces | ID-정렬 arrays | 각각 unique stable ID; 상세 Query는 nation/state 필터됨. 기존 전체 참조 fallback 유지 | sim atomic 조회 | NAT-01/MAP-05 |
| LedgerView strings/entries/tick | 서버 formatted 숫자·row순서·tick | 원장 그대로 전달, browser Fx 계산/인구→인력 추정 없음 | sim→proto→NationalPanels | NAT-01/WP-09/10/12 |

## 참조 그래프·다중성·오류 원자성

2026-10-07 P-06 전송 계약 보강: metadata와 index.bin은 현재 페이지 `location.origin`을 기준으로 만든 절대 HTTP URL에서만 요청한다. Fetch `mode: same-origin`과 `redirect: error`를 함께 사용해 CORS 허용 여부와 무관하게 다른 origin과 모든 HTTP redirect(같은 origin도 포함)를 요청 단계에서 거부한다. 최종 Response.url을 확인하는 것만으로는 이미 발생한 외부 요청을 취소할 수 없으므로 그것을 경계로 삼지 않는다. 기존 정상 endpoint/query pack hash/cache 기본 동작은 유지한다. 오류는 map-data-error ko/en, 새 GPU/canvas 생성 전 실패하며 부모 WorldView/선택/원장/시간을 변경하지 않는다. metadata/index 각각 첫 load와 기존 정상 선택 이후 identity reload의 실제 302·CORS 200 양성 대조/외부 GET 0을 검사한다. 과거 외부 요청0 기록은 정상 경로에 한정된 증거이며 독립 검증의 유효 CORS redirect FAIL을 취소하지 않는다.

오류 경계별 기대값: 초기 로드 실패는 map 안내/canvas0과 정상 권위 패널이다. pack identity 변경으로 재로드하다가 전송 실패하면 이전 renderer가 effect cleanup으로 해제되어 canvas0이며 날짜/시각/tick/worldTick·선택 province·국가·주·원장 값을 보존한다. 이전 카메라의 실행 중 지도를 보존한다고 주장하지 않고 새 geometry를 옛 정의와 섞지 않는다. 기존 malformed/world lookup 부분갱신 거부는 HTTP 재로드와 다른 경계이고 기존 지도/선택/카메라·원장·시간을 보존하는 별도 회귀다. 재로드의 Welcome identity 주입은 fault fixture이며 실제 GUI 플레이 증거가 아니다. 브라우저 오류 이벤트는 엔진별 차이가 있어 actual source HTTP302/Location, native follow+CORS200 원본 body, browser 요청 목록과 foreign server GET 수로 경계를 판정한다. WebKit은 정확한 차단 source URL의 access-control 진단을 pageerror로 보고하기도 하므로 그 값만 예상 네트워크 진단으로 기록하고 그 외 JS 오류는 거부한다.

map 1→N dense index↔실제 Province ID 일대일. 육지 province N→1 state, state 1→N provinces. owner/controller는 각각 N→1 nation이며 물은 null이다. 주인은 control_override로 대체하지 않는다. 국가 capital→province는 검증된 육지 refs. 순환 게임 관계를 새로 만들지 않는다. 나라와 state의 ID 공간은 서로 별도다.

oh_data가 중복/미존재 ID·PNG/CSV 불일치·state 육지 대응·palette 누락을 팩 로드시 거부한다. WP-08은 HTTP metadata shape/범위/identity/전체 dense IDs, raw length/hash/indices와 lookup WorldView 완결 refs를 초기 GPU 생성 전에 대조한다. 실패 시 빈/누락 데이터를 external 다운로드로 숨기지 않고 현지화 오류를 표시한다. malformed world wire는 WP-09 runtime guard가 마지막 유효 상태를 보존하며 socket을 닫는다. 상세 Query는 전체 WorldView refs를 잃지 않게 existing arrays에 ID로 병합한다. 조회 실패는 authority state를 변경하지 않는다. renderer 초기화/compile 실패는 새 WebGL2로 대체하고 둘 다 실패 시 안내한다. camera 이동은 map 밖 선택 null을 보존한다.

색·ref lookup 업데이트는 한 UI update에서 변경 texel을 GPU에 적용하고 다음 render가 사용한다. selection dense sentinel -1은 실제 ID0/65535와 충돌하지 않는다. refs는 RGB validity channel로 null과 ID0를 구별한다. state/nation 세부 필드·population와 manpower·control lifecycle을 새 규칙 없이 확대하지 않는다.

## 호환·미구현 대조

HTTP map contract는 신규 M1 endpoint이며 기존 M0 wire version m0-v1/variant/골든 유지. 신규 DTO 변경 때 Rust generate/runtime schema 검사/fixture를 함께 갱신한다. pack content 변경으로 pack hash가 바뀌므로 index query cache identity는 명시 거부한다. 최종 지도 HTTP schema_version=1을 도입하고 초기 미버전 preview/미래 version과 구조 불일치를 runtime guard에서 명시 거부한다. 이 신규 표시 계약의 구형/미래형 fixture는 HTTP 오류 검사에 포함한다. 저장 포맷 호환은 WP-11 소유, 구현 또는 저장재개 PASS라고 쓰지 않는다.

M1 기초 world fields는 연결되어 있으나 전투/인력/생산/철도/점령/저장 등 후속 필드를 완성된 전체 국가·주 모델로 주장하지 않는다. M5 tiled maps/UV wrap/units/fronts·고도 원천 bake는 WP-08 범위 밖.

## 독립 경계 fixture·검사 예정표

| fixture/검사 | 명시 기대값 | 연결/누락 점검 |
|---|---|---|
| 실제 testland singleexe | 8×6, IDs10..60, province10/30 ownership·state·terrain ko/en, 실제 Query nation/state/ledger | 기존 골든/필터 refs 보존 |
| 독립 12×6 display fixture | IDs[0,10,200,32768,65534,65535], x2 province-only, x4 state-only, x6 nation 경계 | 국가≠주≠province 실제 pixel oracle, backend 동등성 |
| sea/lake/ID0/high sparse | null≠ID0, 65535 선택 가능, clear=-1 | RG8/validity·Yflip·CPU/GPU 일치 |
| pan/zoom/outside | pointer anchored zoom, outside→null, reset→정확 초기 좌표 | camera 및 viewport |
| missing/index length/hash/meta | 404·409·잘못된 DTO/bytes→현지화 오류, 외부 요청0 | 원자 실패/다운로드 없음 |
| metadata/index CORS redirect | 다른 origin의 실제 유효 CORS200 body/header를 positive control로 확인한 뒤, app의302에서 외부 GET0·ko/en 오류·canvas0·기존 정상 WorldView/선택/시간/원장 보존 | P-06 독립FAIL 회귀; 같은 origin redirect도 따라가지 않음 |
| adapter/init/insecure/limits/no backend | 실제 fallback shader pixels, 모두 실패 안내 ko/en | mock flags만 PASS 금지 |
| frozen camera screenshots | 동일 픽셀 기준과 채널±1, boundary interior oracle | 환경/viewport/DPR/backend/date/seed 기록 |
| 기존 M0/M1 회귀·hash | 기존 E2E60+15/TS regeneration/365day hashes2 | 후속 WP 검증과 혼동 금지 |

독립 검증 세션은 이 표의 명시 기대값으로 직접 재실행하며 구현자가 PASS gate를 선언하지 않는다.
