# WP-13 trigger/종료 스키마 — 구현 전 설계

2026-10-07, base f44e16e079cad68e21825a1d9c7bcf1745170412. REQ-AGD-05/REQ-TIME-04. REQUEST-0009 A는 CEO 한정 잠정이며 사용자 D/OPEN 확정이 아니다. 01 §3.3/4.5, 02 §3/5.6/5.7/12.2/15, P-03, schema-planning/M2/preflight 3종/contracts/evidence, WP24-schema/ADR2401, WP17-schema/ADR1701/1702 및 종료 정책 원 ZIP 3member를 대조했다. 조사 기록의 위키 본문은 미사용이다. 아래는 설계이며 구현/독립 PASS/CI 증거가 아니다.

## 실제 producer 인수

- `national::read_scenario`는 resolve_packs→scenario overlay→typed Scenario→map/palette→nation/ownership/control/modifier refs를 검사한다. registered `validate_for_purpose`가 이 reader를 호출한다. DiagnosticCode::Data와 source DataError(path/line/column)를 유지한다. engine-owned historical 정책/force 의미를 변경하지 않는다.
- `World::from_loaded`의 실제 nation/tag/support, state.owner, province.controller 및 map VP는 존재한다. 국가 IC/정치/전쟁/장비/생존/진영 값은 존재하지 않는다. World 기존 definitions_hash tuple은 보존하고 trigger 정의 신원은 별도 state에 보관한다.
- `Simulation::step`: Commands→1h clock→Movement→일일 슬롯→World::evaluate→Snapshot. clone commit으로 phase/clock 실패를 전체 rollback하고 예정 의미오류는 결과를 남기며 소비한다. `Movement`/typed `StraitContext` 및 v1/v2/v3는 그대로 인수한다.

## 필드/권위/분류

모든 신규 선택 필드는 누락=None, 명시 null=거부, 숫자/string 기본값 없음. TOML 자체에 null이 없지만 JSON/schema 입력도 같은 정책을 지킨다. 아래 신규 시나리오 필드 중 하나라도 존재하면 Some trigger 모드다(명시 빈 정의 포함). 모두 없으면 None legacy canonical/reader/writer를 보존한다.

| 필드 | 의미·형·단위·범위 | 필수/null/default | 참조·수명·변경권한·분류 | hash/save/wire/UI·REQ |
|---|---|---|---|---|
| scenario.end_date | Gregorian 문자열; year1..u32MAX, 월/일 실제 달력 | optional; start보다 작으면 오류 | scenario 불변 정의; inclusive 마지막날 | 별도 definition hash/v4; TIME-04 |
| scenario.end_conditions | single-key Condition AST; all/any nonempty, not 1개, primitive typed arg | optional; {},[],null 오류 | 명시 root에 평가; 초기1회/성공 시간전진 뒤; 불변 | definition hash, runtime 읽기만; TIME-04/AGD-05 |
| scenario.end_root | 국가 tag String | 국가 조건에 필수; 날짜-only에는 불필요; 지정시 실제 선택국 | tag→NationId, 위치/index 아님; 불변 | definition hash/v4 identity; AGD-05 |
| scenario.flag_keys | Vec<String>; valid_id=[a-z0-9_]+; 중복 오류 | optional; 명시 [] 허용 | 허용 nation 키 집합; 정의 | definition hash; AGD-05 |
| scenario.initial_flags | BTreeMap<tag,Vec<key>>; 초기 키 중복/미존재 거부 | optional; {} 허용 | 선택 nation 1→키 N; 초기→가변 | definition hash 및 모든 국가의 empty 포함 state/v4 |
| scenario.effect_programs | BTreeMap<valid_id,EffectProgram{root:Option<tag>,effects:Vec<Effect>}> | optional; {}와 명시 empty effects 허용; root null 거부 | trusted host/후속 agenda/event가 ID로 enqueue; client 명령 미노출 | definition hash; pending program ID 전체 canonical/v4; AGD-05 |
| Condition primitive args | date 문자열/bool/tag/u16/Fx decimal string/strict compare{gte,lte,eq}/typed compound | 항목마다 정확 필수 필드, unknown/type/range 거부 | nation/province/state/building/equipment/law/ideology 도메인 구분; refs 검증 | AST 정의; unsupported host 사용 거부 |
| Effect scope.target | self/nation:TAG/state:u16/capital_state/faction_leader/owner_of_state:u16/enemy_of_war | 필수 String; 잘못된 표기·문맥 거부 | nation/state typed context; scope 복귀는 값 전달; 반복 target 없음 | 정의; 후속 faction/war context capability 거부 |
| Effect primitive args | strict typed scalar/compound; end_scenario=valid_id source | 숫자는 exact Fx 문자열/i64; ref/type/range 검사 | source ID는 데이터 원인; 같은 ID 중복 실행은 한 typed 원인으로 집합화 | definition hash; 실제 변경은 command transaction |
| TriggerState.definitions_hash | u64 FNV canonical typed 정의 신원 | Some mode 필수 | immutable 정의/initial values/AST/weights 전체; 생성 이후 불변 | canonical/v4/복원 대조 |
| TriggerState.flags | nation ID순 Vec<(u16,sorted unique Vec<key>)> DTO; live sorted Vec/transaction host BTreeMap/BTreeSet | 선택 nation 집합과 정확 일치, 빈 집합 보존 | 초기 생성, set/clear만 변경, 실행 수명, 국가별 분리 | canonical/v4/query; AGD-05 |
| TriggerState.ended | Option<EndCheckpoint{tick:u64,date:DateV1,hour:u8,causes:Vec<EndCause>}> | 미종료 None; 종료 원인 nonempty | 전체 scenario 한번 종료; clone transaction 성공 뒤 commit; 가변→고정 | canonical/v4; query/save 허용; TIME-04 |
| EndCause | Date / Condition / Explicit(String valid_id) | enum 순서 Date→Condition→Explicit lexical, unique | 동시 모든 원인; program 여러 sourceID 보존 | canonical/v4; UI 후속 WP22 |
| score_weights | strict 4필드 victory_points/industrial_capacity/survival/faction_victory, exact Fx 문자열 | 전체 지정 또는 누락; null/overflow 거부; nonzero unsupported 오류 | signed Fx 범위는 기술 저장범위; 새 normalization 없음 | immutable identity; score inputs/derived result 후속 표 참조 |
| Simulation.trigger | Option<TriggerState> | None만 legacy, Some(empty) 유지 | 초기 권위 생성/Commands/종료 평가만 변경 | serde skip None으로 원 M0/M1 canonical 유지 |
| Command::Effects | program ID String, key=(tick,NationId,sequence) | 존재 program/root nation 권한 검증 | trusted API; enqueue preview 거부는 전 상태 보존, scheduled 의미오류 소비 | canonical/새 PendingV4; public client 입력 없음 |

## registry 문법과 actual host capability

Registry 각 초기 항목에 argument schema·설명·독립 예시를 둔다. 문법 parse/ref validation과 Simulation host capability는 별도 단계다. 미지원 항목은 해당 사용하는 팩의 registered load에서 오류이며 실행 no-op 성공 금지.

| 조건 | 문법 | Simulation 실제 읽기/지원 | 후속 |
|---|---|---|---|
| date_gte | Gregorian string | State.date | 현재 |
| nation_is | tag | World.nation(root).tag | 현재 |
| controls_province | u16 | World.province(id).controller | 현재 |
| owns_state | u16 | World.state(id).owner | 현재 |
| has_flag | valid_id | nation flags | 현재 |
| ideology_support | {ideology, value:compare Fx} | NationState.support[key] | 현재 |
| at_war/at_war_with | bool / tag | 미지원 | WP20 |
| stability/mobilization/political_capital/has_law | compare Fx / law ID | 미지원 | WP14 |
| chance | Fx0..1 | event context 문법만; Simulation end/if 거부 | M3 event RNG adapter |

| 효과 | 문법 | Simulation 실제 변경/지원 | 후속 |
|---|---|---|---|
| set_flag/clear_flag | valid_id | nation 집합 idempotent | 현재 |
| end_scenario | valid_id explicit source | 전체 effects 성공 후 종료 후보 | 현재 |
| add_stability/add_mobilization/add_political_capital | exact signed Fx string | 미지원 | WP14 |
| set_law/add_building/transfer_state/declare_war/add_manpower/add_equipment | strict lawID/{state,building,levels}/{state,nation}/tag/i64/{equipment,amount} | 미지원 | WP14/15/20 |

## 관계·깊이·예산·오류 순서

Scenario→선택 tag/ID; ownership/controller→실제 World; flag keys←initial/set/clear/has; program ID←queue; end source ID←explicit cause. nation/state/province는 정수 위치가 아니다. duplicate ID/keys·missing refs는 거부. AST는 유한 트리이며 재귀 program 호출/자동 국가 반복이 없다. `self`는 caller root, `capital_state`는 실제 capital의 state, `owner_of_state`는 actual owner로 resolve한다. nation flags에 state target이면 capability/context 오류; faction/war target은 현재 load 거부.

AST root 깊이1; all/any/not 자식+1, scope effect child+1, if.condition와 양 branch child+1. 최대16은 문법 기술 상한(02§5.6); 17 거부. condition/effect 루트 각1. 효과 전체 call 하나에 primitive 실제 실행만 카운트; scope/if 자체는 카운트하지 않는다. 분기 미실행 항목은0. 1000 성공, 1001번째 실행 전 오류로 전체 transaction rollback. runtime AST는 validated immutable 정의만 사용한다. 문법 최대깊이를 parse 이전 raw shape 검사로 제한하여 과도한 재귀를 거부한다.

오류 우선: raw syntax→typed shape/depth→날짜/root/initial keys→모든 branch ref/type/range→host primitive/scope capability→live 생성. 기존 national 오류 순서는 앞에서 보존하고 신규 검사는 기존 refs 뒤 추가. 신규 오류는 실제 scenario source field와 AST path를 message에 보존한다. enqueue는 Ended→past→duplicate→program/reference/root authorization→preview; 실패 전상태 불변. 예정 effect 실패는 해당 transaction state/flags/end 후보를 rollback하고 command 결과 거부·큐 소비, 기존 step 시간전진 계약 유지. clock/movement/ledger/end 평가 실패는 전체 step state/queue/clock 보존. effect candidate는 tick이후 날짜/조건과 안정 병합하여 checkpoint를 한번 commit한다. Pause 중 효과 명시 종료는 clock불변 checkpoint, Pause/Speed만으로 조건 재평가 없음.

## 저장·버전·wire·host

새 `SimulationSaveV4`는 **frozen SimulationSaveV3 base**(모든 nested legacy queue empty) + movement_present/strait_present bool + full PendingV4 + TriggerState DTO다. 이 구조는 frozen 기존 타입을 저장 substrate로 재사용하며 실제 movement None/Some(empty), explicit strait Some(empty)을 두 bool로 구분한다. 원 v1/v2/v3 field/variant/order는 바꾸지 않는다. trigger None은 기존 writer 선택; Some은 값이 empty이거나 새 queue만 있어도 OHSV4. export old API는 Some에서 명시 V4 required로 손실 방지한다. v4 restore는 bool/empty consistency 검사 뒤 원 v1/v2/v3 restore를 실행, local World의 검증된 trigger 정의와 hash/flags/cause/root/ref/order/date/tick을 대조하고 전체 queue를 복원한다. 재평가/마이그레이션 없음.

bounded v4 preflight는 기존 v3 reader shape를 그대로 끝까지 읽은 후 bools/새queue tag·string 및 triggerhash/nationflags/end 원인 enum을 같은 Limits cap/allocation charge/budget으로 검사한다. 새 AST는 save에 넣지 않고 검증된 local defs로 재로드한다. unknown version/trailing/noncanonical/definition identity mismatch/잘못된 원인·order·queue는 새 live값 생성 전에 거부. 원 force/history policy는 그대로다.

Simulation.with_world/with_movement에서 World defs의 trigger 정의를 자동 인수하여 초기1회 평가한다. SaveContext.simulation·CLI national·Host national 입구는 같은 생성자를 사용한다. CLI의 requested ticks는 host 상한으로 보고 정상 종료 checkpoint에서 native0으로 반환하며 sim.step 자체는 Ended 오류다. server session은 ended이면 펌프를 멈추고 조회를 계속 받으며 새 명령은 scenario-ended 응답, thread를 오류로 끊지 않는다. 새 Query kind=trigger/TriggerResult만 더하여 TriggerView(definitions_hash:hex16, flags:{nation:u16,keys:string[]}[], ended:EndView|null)을 반환한다. EndView는 tick:u64 decimal string/date:server String/hour:u8/typed causes(Date/Condition/Explicit{source})다. ko/en scenario-ended 응답과 생성 TS/runtime shape를 함께 검사한다. 기존 TimeState/WorldView 메시지 bytes와 protocol m0-v1은 유지하며 전체 결과 UI는 WP22 후속이다. client는 게임 규칙을 계산하지 않는다.

## 독립 정상·경계·오류 예제

- 1900/2100-02-28→03-01 inclusive48h; 2000 동일72h; 2099-12-31 하루24h→2100; 2004-02-29 하루24h. 직전23:00 미종료/다음00:00 Date. start=end 하루, end<start 거부, u32MAX 마지막날 step overflow 전체 보존.
- initial date_gte=start 참은 tick0 Condition; nation_is NTH와 owns STH false, all(false,true)=false/any=true/not inverse. 읽기 전후 canonical/RNG/queue 동일.
- root 누락/null/없는 tag, flags 중복/없는 key/타입/null, primitive 키2개·unknown·compare키2개·Fx 범위·ID domain·모든 branch refs를 로드 거부한다.
- 깊이16/17, 실제 effect1000/1001; if unselected branch는 예산 사용0; scope 복귀 뒤 NTH clear/STH set 유지. 같은 key set/clear 두번은 idempotent.
- set→end a→end z→실제 host 실패: 전 state/end/queue/RNG 동일; accepted transaction은 Date/Condition/Explicit(a)/Explicit(z) 안정순서.
- 종료 future Move/Stop/Effects/Pause 보존, 새 enqueue/step 전부 거부·동일 hash, stopped unit/empty route/explicit empty strait 저장 보존.
- v4 신규 ended/active/empty 정의·새queue만 있는 형태를 capture2회, 별도 fresh PID process resume fullDTO/canonical/hash/원save reencode로 검사. 원None/v1/v2/v3 회귀는 기대값 교체 없이 실행.

## 후속 누락 추적

| 요구 | 실제 producer 의미 | 현재 지원 | consumer/검증 |
|---|---|---|---|
| 보유 승점 | map VP integer + province.controller는 있지만 보유의 소유/통제 의미 미정 | score accessor 미구현/nonzero capability 오류 | WP20 의미 인수, WP22 결과 UI, M2 gate |
| 산업 역량 | 국가 IC absent; state.buildings/infrastructure를 IC로 바꾸지 않음 | nonzero capability 오류 | WP14 actual IC Fx/원장/수명, WP22 |
| 생존/진영 승리 | World에 없다 | nonzero capability 오류 | WP20 actual typed snapshot→WP22 |
| 항복/관전 | scenario end와 다른 요구 | 입력 실행하지 않음 | WP20/22 실제 player/관전 선택 |
| 가중합 | signed Fx weight×실제 Fx input checked arithmetic, 축순 VP/IC/survival/faction; raw ledger·checked sum | host-independent 산술만; zero 미사용 축은 input None, 가짜0 없음 | 실제 producers 뒤 full four-axis 소비 증거 필요 |
| registry reference | 자체 schema/설명/examples 산출물 | 기술 문서 | REQ-MOD-07 후속 전체 reference |

## 전용 actual 3OS 도구 소유

신규 `crates/oh_sim/tools/check_trigger_determinism.py`, 해당 test_trigger_evidence.py, `oh_save/examples/trigger_fixture.rs`, `.github/workflows/trigger-determinism.yml`만 추가한다. 원 checker/workflow/helper/fixture는 보호한다. source HEAD/clean/native PID/executableSHA/command/cwd/exit/stdout/stderr와 capture repeat bytes/fresh resume fullDTO/canonical/hash/definition·pack identity/원input을 artifacts로 보존하고 missing OS/dirty/다른 HEAD/조작/누락을 compare가 거부한다. synthetic 회귀는 actual3OS 증거가 아니다. 실제3OS/mainCI 관측은 부모의 통합 이후다.

## 실제 구현 경로와 복원 모드 대조

- 신규 with_world는 world를 설치한 후 TriggerState::initial→checkpoint(evaluate_condition=true)를 정확1회 수행한다. with_movement는 movement를 먼저 검증하고 위 builder를 한번 호출한 후 movement를 설치한다. 지원 조건은 World/date/flags만 읽으며 movement를 읽지 않는다.
- 기존 public from_save/v2/v3는 local World::Defs trigger Some과 legacy None save의 mode mismatch를 거부한다. v4는 constructor를 호출하지 않고 internal from_save_base/from_movement_save/from_strait_save를 통해 frozen body를 복원한 다음 별도 trigger를 검증·설치한다. 이는 원팩/원fixture의 거부나 migration이 아니라 save/local schema mode identity 검사다. codec check_header에서도 force 여부와 관계없이 mode 일치를 검사한다.
- TriggerState::validate는 정의 hash/nation 집합/key/order/checkpoint clock/원인 정의·source ID 존재 및 날짜 경계만 검사한다. condition 평가나 initial flags 재생성 호출은 없다. saved active condition이 true라도 복원은 재판정하지 않는다. native paused_condition 예제는 Pause true→set_flag x로 condition=true/tick0/미종료를 만든 뒤 freshPID에서 그대로 재개한다. 원 정의 변화는 definition hash/pack identity를 통해 오류로 거부한다.
- 하위 root와 ideology_support는 실제 해당 nation의 support key를 로드에서 검사한다. owner_of_state는 현재 actual owner로 scope를 resolve한다. 현재 소유 변경 효과는 capability 미지원이며 후속 producer가 추가될 때 mutable owner/ideology 참조 계약을 다시 인수해야 한다.
- 새 argument 범위는 syntax용 기술 계약이며 미지원 정치/전쟁/장비/법령 효과의 게임 의미·상한·clamp를 승인하지 않는다. 후속 producer 인수 전 실제사용은 capability 오류다. JSON Schema는 structural shape/null/type를 기술하고 refs/semantic/depth/capabilities는 loader가 별도 검사한다. registry argument schema는 각 primitive의 실제 property와 재귀 $defs를 분리해서 생성한다.
- own trigger_native.py/trigger_query.cjs/trigger_negative.py는 실제 서버 native0·종료 후 query 지속/명령 거부, tick0 startup, paused active restore, 실제 normal/force mode mismatch·unsupported primitive native1 및 원 input 보존을 검사한다. 원 서버/helper/checker 검사를 삭제·완화하지 않는다.
- 기존 RNG는 core의 seed/key 파생이다. 이번 Condition host는 mutable RNG stream을 만들지 않고 chance를 M3 event context 밖에서 거부한다. future M3 chance adapter는 sim RNG와 별도 평가 context/저장 계약이 필요하다. RNG 값0이나 가짜 neutral state를 성공으로 반환하지 않는다.
