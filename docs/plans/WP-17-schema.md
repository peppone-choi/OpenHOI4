# WP-17 이동 스키마 — REQ-MIL-04

2026-10-07. 아래 첫 표와 strait 미지원은 최초97f의 구현 전 계획/당시 범위다. 현재 REQUEST0007 대응은 아래 P-06 절에서 확장하며 원 기록을 보존한다. 최소 이동 authority이며 편제/훈련/장비/군은 WP-16이다. 기존 01 §4.7의 거리÷속도×보정만 사용한다. 공개 위키 본문은 조사 기록상 접근 실패이며 사용하지 않았다.

| 필드 | 의미·타입·단위·범위 | 필수/null/default | 참조·수명·권위·분류 | hash/save/wire/UI·REQ |
|---|---|---|---|---|
| Simulation.movement | Option<Movement>; 미설정은 legacy | 기존 생성자는 None; 신규 생성자는 명시 초기값 | Simulation 초기 생성 후 Commands/Movement만 변경; 가변 | None은 기존 canonical 바이트 유지, Some은 전체 hash/v2 save; MIL-04 |
| Unit.id/nation/province | DivisionId u32 / NationId u16 / ProvinceId u16; 0 유효, 비연속 가능 | 모두 필수, null 없음 | unit→nation 1, 현재 완료 도착 province 1; nation은 unit 소유이며 province 소유/통제와 별개; 초기→가변 | 전체 hash/v2 save; wire/편제/UI WP-16/22 후속 |
| Unit.speed | Fx I32F32 km/h, >0 | 필수, 기본값 없음 | WP-16 trusted 초기 adapter; 이번 생성 뒤 불변 | hash/v2 save, overflow 거부; MIL-04 |
| Unit.allowed | ID 순 BTreeSet<ProvinceId>; 자기위치 포함 | 명시 필수, neutral 통행권 없음 | 외부 WP-20 접근 context, 모든 참조 land 존재; 초기/권위 입력 | hash/v2 save, 자동 통제 추론 없음 |
| Unit.corrections | BTreeMap<(from,to), Factors> 방향별 edge | 명시 필수, 경로에 필요한 값 누락 오류 | map의 실제 인접 land normal/river edge만; from/to와 unit 다대다; trusted adapter; 초기 입력 | hash/v2 save; 원 지도는 Defs single source |
| Factors.terrain/river | 각각 Fx >0, 정의 계수 | Defines movement.terrain_<terrain>, movement.river_<normal/small/large>에서 명시; missing/array/negative 거부 | 도착 지형/edge kind는 MapData 정의; from_defines adapter | 실제 적용계수 전체 hash/save |
| Factors.infrastructure/supply | Fx >0 시간 배율 | trusted context 필수, default 없음 | WP-14/19의 원 수치→배율 공식은 후속; 현재 인프라/보급을 자동 해석하지 않음 | 실제 입력 hash/save; 후속 갱신 adapter 미구현 |
| Route.legs | Vec<Leg> 남은 경로; Leg from/to ProvinceId, hours Fx | 빈 route=정지, null 없음 | 첫 from=현재 province, 연속 land edge; 양수 hours; 파생 계산 결과를 권위로 유지 | 전체 hash/save, 복원시 formula/context와 bit 일치 검사 |
| Unit.elapsed | Fx hour; 0≤elapsed<첫 leg.hours; 빈 route는0 | 초기0; default 게임 계수 아님 | Movement phase만 증가; 중간 edge의 위치는 출발 province 유지 | hash/save; 잔여 시간 처리 ADR-1701 |
| Command::Move/Stop | DivisionId 및 목적지 ProvinceId, 발행 nation은 ordered key | 모두 필수, 현재 world/movement 미설정은 오류 | 큐 key=(tick,nation,sequence); 단일 변경 경로 | 전체 queue hash/v2 save; CommandV1 동결 |
| SimulationSaveV2 | legacy SimulationSaveV1 base(큐는 빈 Vec)+ordered PendingV2+UnitV2 Vec | v2는 movement 필수 | 별도 추가 reader/writer; v1 reader/writer 그대로, migration 없음 | OHSV version2; v1 header 모양 유지하되 version2 |

관계: Defs.MapData → Unit.position/allowed/correction endpoints → Route. unit ID는 sorted unique, nation/province 참조는 존재해야 한다. 경로 탐색은 순환 지도 허용/양수 cost/중복 정점 없는 최단 경로. 동점은 전체 province ID 경로의 lexical 순. 물/호수/strait/impassable은 이번 육지 solver에서 통과하지 않는다. 해상 수송은 WP-31, strait 통과 규칙/계수와 담당은 미정이며 성공 no-op으로 다루지 않는다.

갱신: enqueue에서 현재 권한/입력/경로를 clone preview로 검증하고 실패시 큐/상태 보존. 예정 입력은 적용시 재검증하여 의미오류면 해당 명령만 거부·소비. Commands→시간1h→Movement(ID순)→기존 world ledger→atomic commit. phase/clock overflow는 전체 상태/큐/시계 보존. paused는 Commands만 적용하고 이동 elapsed/위치/clock 불변. 속도5는 host pacing일 뿐 이동시간 가속 아님.

CEO 2026-10-07 REQUEST-0006 A 한정 잠정 채택: 중간 reroute는 현재 진행 edge를 유지하고 그 끝점부터 새 목적지까지 재탐색한다. Stop은 진행 중이면 현재 edge 완료 후 정지, 아직 elapsed0이면 즉시 정지. 이는 REQUEST-0006 A의 게임행동 잠정이며 기술 계산/저장 ADR과 구분한다. 전투 후퇴는 후속이다.

독립 입력/기대값:
- 합성 0→10→30 및 0→20→30, 모든 edge10km/speed2/factors(1.5,1,1,1): 각7.5h; 동일 cost이면 [0,10,30], ID index 접근 금지. 7틱 후 출발/elapsed7, 8틱 후10/elapsed0.5, 15틱후30/빈 route.
- 자기위치: 빈 route/시간0. 없는 ID/타국 명령/속도0 또는 음수/중복 unit/허용집합 누락/계수 누락 또는0/음수: 상태 생성 혹은 enqueue 거부, 상태·큐 동일.
- 물/호수 목적지는 InvalidReference, impassable/strait만 연결: NoPath. 새로운 접근/보급이 없는 경로를1로 숨기지 않음.
- 10km/2kmh×(1.5,1,1,1)=7.5h; distance Fx MAX/speed1bit 또는 곱 overflow/양수 결과 underflow0: 명시 오류, atomic.
- 1틱 진행 후 reroute to start: 현재 edge 완료 후 되돌아오는 별도 land 경로, elapsed 보존. Stop은 현재 edge 도착 후 정지; Pause는 progress 보존.
- 입력 순서 다른 units/correction map: canonical 동일. 목적지/발행nation/sequence/계수/elapsed1bit 변경: canonical 달라야 함.
- 이동 중(첫 leg elapsed1h)+향후 Move/Stop 예약 저장: 원본 DTO/모든 hash bit/재개 결과 동등. 기존 m1-v1 fixture read→write 구조/원본문자·hash 보존, format1 생성 유지.

버전: v2 bounded preflight는 기존 Limits의 queue/map entries/allocation budget을 재사용해 모든 신규 벡터/맵을 역직렬화 전 검사한다. 신규 버전은 prefix/header version 일치, 미래 버전 거부. v2의 route/context semantic 검사와 hash 대조도 필수다. 기존 v1 DTO/export/from_save는 그대로 유지하며 이동 상태의 v1 export는 명시 v2 필요 오류로 손실 방지한다. 기존 저장의 거부/마이그레이션은 하지 않는다.

후속/미완료 경계: WP16의 실제 unit 생산/팩 초기 adapter, WP19/20 context 갱신과 보급·통행권 공식, WP31 수송, WP22 실제 조작 UI. 이번 기존 server에는 unit 생성/국가 배정이 없으므로 외부 클라이언트 입력/DTO를 미리 성공시키지 않는다. 공개 wire 변경 없이 host용 Simulation builder/큐 API를 인수한다.

부모 main 01 §4.7 / 02 §5.11 / 03 P-03 및 REQUEST-0006 원문 SHA256 0b9e2ba3756fa1022a876f11369dc8b4e4909fc49e84ea35bfbf9b7d75734b07, 독립 정수8경계(2026-10-07)를 읽고 채택 범위를 대조했다. 사용자 D/OPEN 추가확정·콘텐츠값·새 보급/접근/전투·저장 migration 승인이 아니다. None만 legacy이며 Some(empty)·정지unit·context·모든 Move/Stop 예약도 v2다. restore context는 기존 local world/pack/defines identity를 재검증하며 v2 external applied factors/access는 저장 authority로 복원·참조/양수/순서/route formula raw bit를 검사한다. context는 trusted host 초기 생성에서만 설정하고 수명은 unit/Simulation과 같다. 동적 갱신 API는 후속이다. 클라이언트/DTO 조회에서 이 값을 변경할 수 있는 경로는 없다.

strait 누락 대조: 01 §4.1의 인접 crossing kind와 WP31 항구 간 해상수송은 같은 개념으로 추정하지 않는다. 현재 solver는 strait를 탐색 후보에서 제외한다(그 경계만 이어지면 NoPath). strait 통과 정책/계수·담당 WP는 결정 필요이며 해상수송에 임의 위임하지 않는다. sea/lake unit 위치 및 직접 육지 목적지는 InvalidReference, 실제 고립/impassable만의 연결은 NoPath, 보정 context 누락은 MissingContext로 구분한다.

## P-06 REQUEST-0007 A 인수 설계 (코드 전, 2026-10-07)

부모 main93e3dc1의 01 §4.7/02 §5.11/03 P-03/REQUEST-0007와 원 SHA256 `2731e99b719f836d42d2100da349392d199cbebba301796cbbeed098104b2e54`, 자체 독립8산술을 읽었다. CEO 한정 잠정 게임행동이며 D/OPEN 추가확정은 아니다. 원97f 커밋/테스트/증거와 원문은 보존한다.

| 새 필드 | 의미/형/단위/범위 | 필수/null/default | 참조/수명/권위/분류 | hash/save/version/REQ |
|---|---|---|---|---|
| Movement.straits | Option<BTreeMap<DivisionId, directed context map>> | 기존 constructor None=97f legacy 계약, 새 trusted with_straits는 Some(빈 map 가능); unit마다 명시 context 필수 | unit별 1; 생성 이후 immutable; only trusted host 초기입력 | None skip→97f canonical 그대로; Some 전체 hash와 추가 v3 save; MIL-04 |
| StraitInput.unit/corrections | DivisionId u32; BTreeMap<(ProvinceId,ProvinceId),StraitContext> | 모든 필드 명시, 중복unit 거부, direction reverse 자동 채움 없음 | 존재 unit 1, 양 endpoint land/allowed 및 실제 MapDataStrait; 정의+초기 adapter | unit 참조/정렬을 v3에 보존 |
| StraitContext.kind | CrossingKind(Normal/RiverSmall/RiverLarge/Strait) | 필수, 실제 Strait만 수락; 다른 kind 오류 | MapData.edge.kind와 일치, normal/river 슬롯과 구별 | enum tag raw canonical; v3 kind u8 검증, MIL-04 |
| StraitFactors.terrain/infrastructure/supply/strait | 각각 Fx >0 시간배율 | 전부 필수, null/1fallback 없음 | terrain/strait는 Defines 어댑터에서 movement.terrain_<target>, movement.strait; infra/supply는 trusted 외부입력, directed applied 값 수명=unit | checked distance÷speed→terrain→infra→supply→strait; river는 곱하지 않음; raw i64 전부 hash/save |
| SimulationSaveV3 | frozen SimulationSaveV2 base + Vec<UnitStraitsV3> | v3 explicit Some context를 반드시 포함, base/v2에는 새field/variant 추가없음 | 정렬unique unitID, correction endpoints/kind/raw factors; references local restore context와 검증 | OHSV3 reader/writer 추가; v1/v2 모두 읽기·쓰기·hash·실행 유지, migration 없음 |

흐름: trusted with_straits 생성→unit/context/land/allowed/kind/양수 구조 검사→enqueue preview→Commands apply 재검사→1h Movement→atomic commit. explicit mode에서 allowed 연결의 실제 Strait를 탐색 후보에 넣고 해당 direction context가 없으면 MissingContext(계산 실패를 NoPath로 숨기지 않음), 양끝점 allowed 밖은 edge 후보 제외/목적지 InvalidReference. legacy None은 원97f의 restricted land/river policy이며 새로운 정책 context를 자동 부여하지 않는다. 미래 host adapter는 명시 Some mode와 context를 공급한다. 함대/봉쇄/항구/통행권·진행중 allowed 변경 정책은 없다.

기존 v2 bytes/head/hash 재개는 97f의 immutable source/producer 출력과 비교한다. v3 preflight는 frozen v2 base shape 후 unit별 context 벡터/enum kind·4 raw factors를 같은 queue/map cap·512byte entry allocation charge/예산으로 검사한 다음 semantic validation한다. unknown 미래 version·prefix/header mismatch·중복/미존재/비육지/wrong kind/null 표현/zero·negative·overflow·underflow는 새 live 값을 만들지 않는다.

독립 예제: 10km/speed2, terrain/infra/supply=1, strait2는 10h=42949672960raw; tick9 출발/elapsed38654705664, tick10 도착/elapsed0. 정상 대체8h이면 normal path, 동점10h이면 전체 ProvinceId lexical. river7계수가 strait2와 중복되어70h가 되면 FAIL. direction reverse 누락은 MissingContext. 같은context insertion순서와 unit순서는 같은 canonical. applied strait1bit/kind(Defs identity) 변경은 hash sensitivity; wrong context kind는 거부. tick9+pendingStop/Move+Pause 저장/새process 재개 및 REQUEST-0006 reroute/Stop 잔여경계 유지. missing/0/음수/overflow/underflow/wrong reference/kind/nonland/타국 입력은 state/queue/clock 불변.

P06 추가 실제 DTO 계약: SimulationSaveV3.base:SimulationSaveV2는 frozen fieldorder이며 base.base.queue는 빈Vec이고 base.queue가 모든 time/Move/Stop pending이다. straits:Vec<UnitStraitsV3{unit:u32,corrections:Vec<StraitCorrectionV3{from:u16,to:u16,kind:u8,factors:[i64;4]}>>는 모든 unit과 같은 sortedunique ID집합이다. 단위는 km/h·hour·무차원 시간계수, 각 factors slot은 [terrain,infrastructure,supply,strait] 순 raw I32F32 i64. kind=3(Strait)만 유효하고 0/1/2/255는 거부한다. 각 collection은 기존 map_entries_max 및 allocation_budget/entry_charge512의 preflight와 sortedunique semantic check 대상이다. Some(empty units/context)는 v3, Some unit의 empty corrections도 v3로 저장되어 missing direction을1로 만들지 않는다. host constructor/input/export/restore 참조검사와 live state는 독립이며 wire/파생조회 DTO에는 새 변경권한이 없다.
