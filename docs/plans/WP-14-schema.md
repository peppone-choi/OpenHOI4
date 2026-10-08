# WP-14 경제·정치 및 고정 측정 입력 스키마

2026-10-08. 구현 기준은 REQUEST-0010 A-CEO-r1 수정판이다. 사용자 확정이 아니다. 구현 전 작성한 설계와 현재 실제 producer 계약을 기록한다. 독립 PASS/통합 판정은 부모 소유다.

## 계층·필드·참조

| 경로/필드 | 의미·타입·범위 | presence/참조/권위 | 보존 |
|---|---|---|---|
| scenario.economy | 선택 상대 정의 파일 ID, null/escape 거부 | 부재는 기존 실행; 명시 빈 정의도 별도 존재. common/economy/ID.toml | pack identity 및 새 정의 hash |
| EconomyDefinition.buildings | ID→name_key/단계비용 Qty/단계별슬롯 i64/cap Qty/IC단가 Qty/infra 설정 | 모든 값 명시, 비용>0, 나머지 비음수; 기존 map 건물과 명시 참조 | 불변 정의 |
| laws | 범주·단계·name_key·PC비용 Qty·AST조건·IC보정 Fx·징병 Fx·소비재base/slope Fx | duplicate category+step 거부; 조건은 기존 typed AST, 미지원 조건 거부 | 불변 정의 |
| nations | nation ID→선택 laws/PC Qty/상한 Qty/일일 Qty/stability Fx/mobilization Fx/네비율 Fx/committed/reserved i64/IC보정 Fx | 신규 정의에서 전 국가 필수, 기본값 없음, 안정/동원/비율 0..1, 정확합1 | 초기→private mutable |
| states | state ID→slot_limit i64 | 모든 실제 주 참조; 비음수 | 초기·불변 |
| quantity ledger | 계산tick·주별IC/자원/인구 기여·적용 보정·합계·배분·잔여 raw bits | Qty I48F16와 Fx I32F32 별도; 변환은 checked wide 정수내림; 실제 applied bits 일치 | DTO/canonical/hash/new save |
| project | nation queue의 stable ID·주/건물·target=current+1·progress Qty·휴면 사유 | 중복 target/slot 거부, owner loss 보존, 회복 target conflict skip | 전체 queue/예약/진행 저장 |
| manpower | capacity/committed/reserved/available/overcommitted i64 | checked 합계·floor(pop*Fx), consumer 내부 API | 원장·저장 |
| commands | 비율/한단계건설/취소/순서/법령 | enqueue preview 및 execution atomic, 조건은 명령 직전 상태 | pending/journal/replay |
| host/wire | 새 상태 presence/DTO/raw bits | old None/v1~v4 경로 보존; 실제 additive v5 및 typed runtime 검사 | 새 native/save/repro tests |

새 상태 생성은 초기 계산을 하되 PC를 지급하지 않는다. 정상 날짜 경계 Economy→Construction→Manpower→Politics→Ledger→종료 평가를 유지한다. Pause/query/restore 지급 없음. phase Err는 clone commit 전 전체 rollback, 의미 command Err는 성공 step에서 queue 소비. 실제 후속 생산/자원부족/군사/항복/훈련/시장 consumer는 WP15/16/20/29이며 해당 REQ 전체 완료가 아니다.

## 산술과 독립 입력

비율은 Fx raw 32 fractional bits, 누적량 Qty raw 16 fractional bits, 인력은 i64다. Qty×Fx는 i128 곱을 32bit 내림한 뒤 i64 checked 변환. 국가 IC는 주/건물 ID순 Qty합→국가 Fx→선택법령 category순 Fx. 소비재 제외 배분은 각 floor, 소비재는 floor+정확합 잔여. 하한은 기존 Fx.checked_mul 내림→checked_add→0..1 제한. ceil raw IC는 ceil((남은 Qty raw<<32)/보정 Fx raw), 유효 진행 floor(rawIC*Fxraw>>32). 초과는 같은 Qty 원장에 폐기 기록한다. 0 cap/보정 skip. PC 지급은 i128 sum.min(cap), 입력 PC<=cap 검사. committed+reserved는 i64 checked 합계.

独립 경계: IC raw5/각1/4→2,1,1,1; A소유/B통제→A기여; 하한변경 rollback; 완료필요3/cap6/원IC10→첫3·둘6·미사용1; cap0/보정0 skip; owner-loss/회복target충돌 보존; capacity100/60/10→available30·감소50→over20; PC0/cost3/경계daily3→명령거부후PC3; 안정도1+epsilon·overflow·NaN/null/정의누락 거부; paused/futurequeue 및 fresh process 전체DTO/canonical/hash 일치. RED 및 실제 결과는 worklog에 남긴다.

## 고정 benchmark 입력

측정 source는 baseline81deb와 실제 current HEAD 각각의 library다. 측정 pack은 git archive baseline81deb의 data/packs/testland **전체** 한 경로이며 current 콘텐츠와 독립이다. sorted relative 전체 file path/bytes/SHA 및 디렉터리 목록을 보존한다. symlink/reparse·특수파일·경로이탈·추가/누락/손상을 거부한다. archive/source lock/driver/metadata의 실제 path dependency·registry checksum·binary SHA를 보존한다. build linkage 뒤, load/warmup/측정 전후 pack identity와 binary identity를 재검사한다. 양쪽 load의 실제 full DTO packhash를 동일성 대조하고 종료 DTO/canonical/hash도 기존 comparer로 대조한다. m1/seed1000/240000/warm2400/two pairs15%/240s/1200s 고정. 새 경제/M2UI 성능을 이 벤치가 증명하지 않는다.

独립 회귀는 prior 전체 pack 추출→current에 신규 콘텐츠만 추가→양 binary에 같은 prior pack; extra/missing/손상·symlink/reparse·escaped 참조·build 이후 input 변경을 실제 파일로 주입하여 실패한다. CI/workflow/policy는 변경하지 않는다.

## WP-23 실제 입력 인수 (초기 구현)

scenario.toml 루트 `economy = "synthetic"`는 `common/economy/synthetic.toml`을 선택한다. `crates/oh_data/tests/fixtures/economy/valid.toml`은 실제 loader·strict validator가 검사한 독립 입력이다. JSON structural 출력은 `crates/oh_data/schema/economy.schema.json`이며 실제 의미 검증은 `oh_data::economy::Definition::validate`와 `trigger::economy_conditions`, 선택 파일/locale 검사는 `pack_validation`이다. 미선택 경제 파일은 오류다.

기존 map state의 `population/resources/buildings/infrastructure`는 계속 필수이고 실제 권위 초기값이다. 신규 `state_slots`는 그 state ID 전체를 정확히 덮고, `nations`는 nation ID 전체를 덮는다. typed ID는 배열 순번이 아니다. `buildings` 정의 ID는 기존 common/buildings.toml registry를 참조하며 비용/slots 벡터의 index0은 level0→1, index1은1→2다. stage cap은 벡터길이, 단계별 슬롯은 누적합이다. `infrastructure_factors`는 현재 Fx 값과 정확히 일치하는 값을 찾아 그날 시작 보정으로 쓰며 보간·누락 fallback은 없다. `infrastructure_levels` 부재는 일반건물, 명시 벡터는 각 완성 단계에서 바꿀 infrastructure base Fx 값이다. 두 벡터 길이는 비용 단계 수와 같아야 한다. 국가 IC는 각 기존 state.buildings 수준×해당 ic_per_level Qty 합이다. 공업 아닌 건물도 ic_per_level을 명시0으로 둔다.

`laws`는 ID→Law, 범주별 step 중복 거부; `economy_category`/`conscription_category`는 명시 선택한 category이며 같아도 된다. 국가 laws는 모든 정의 category를 정확히 하나씩 선택한다. 모든 법령에는 **양수 cost** Qty, condition typed AST, ic_multiplier Fx, conscription_ratio Fx 0..1, consumer_base Fx0..1, instability_slope Fx>=0가 필수다. 이름 키는 ko/en 실제 locale 키이며 일반 ID 검증과 다르다. 초기 법령 condition 만족은 강제하지 않는다(변경 전 조건이며 초기 시나리오 선택은 명시 입력). 미래 조건/효과는0도 capability 오류다.

새 scalar는 TOML exact decimal 문자열이며 float/NaN/overflow를 거부한다. committed/reserved/state_slots/slots는 TOML i64 비음수다. PC cap/daily는 각 국가 명시값이며 상한보다 큰 PC를 거부한다. 기본값을 추가하지 않았다. 새 gameplay defines key는 사용하지 않으며 모든 수치는 경제 정의에 명시한다.

첫 인수 커밋5577ee0은 data schema/strict loader와 초기 kernel이다. 이후 실제 v5·host/wire·repro 및 전용 native 증거 수집을 연결했다. 최종 source와 실행 결과는 WP-14 worklog를 따른다. WP-23의 실제 m2 팩·독립 검증·원격 3OS 통합 판정은 별도다.

## 현재 권위·실적 snapshot·저장 계약

`oh_sim::Economy`는 private mutable이며 Simulation의 read-only getter로 읽는다. 배분/건설/취소/재정렬/법령 명령과 지원 정치 효과는 clone 검증 후 commit한다. 다른 국가 state, 중복 project/target, 슬롯·하한·비용·condition 불충족을 거부한다. 기존 command variant 뒤에 Economy를 추가하여 원 canonical tag를 보존한다. 현재 비율/법령/인력 capacity와 마지막 일일 적용 ledger는 구분한다. quantity ledger는 tick·법령·stability·capacity snapshot, 주별 인구/자원/건물수준×단가, source별 IC multiplier와 적용 Qty, min/ratios/allocation/residual을 보존한다. construction ledger는 당시 owner/state/building/target/start progress/cost/cap/infrastructure/factor와 factor_evaluated, consumed/applied/discarded/completed/dormancy를 보존한다. skip 값은 reason과 evaluated=false로 표시하며 실제 보정을 주장하지 않는다. construction의 당일 infrastructure는 해당 tick의 modifier 적용값을 시작 시 고정하고 완료된 infrastructure base는 다음 날짜 계산부터 쓴다.

내부 manpower API는 Reserve/Commit/Consume/Cancel/Return을 checked transaction으로 적용한다. 플레이어 wire/command로 노출하지 않는다. WP-16이 실제 예약·훈련 객체 권한/수명 및 exactly-once consumer를 인수해야 한다. 공장 생산/효율/자원 부족/WP-15, 군사 안정·동원/항복/WP-20, 시장/WP-29를 만들지 않았다. 지원 조건은 stability/mobilization/정치자본/law, 효과는 정치자본/안정도/동원도/law이며 효과 0도 미지원 capability는 거부한다. 산업 score는 종료 시 실제 Qty IC 입력×Fx weight의 Qty term·tick/input_tick만 제공하고 미지원 점수 축을 0으로 채우지 않는다.

Scenario의 economy 부재는 None이며 legacy canonical/save 경로를 유지한다. 명시 빈 정의는 buildings/laws/nations/state_slots 네 map 모두 빈 값과 두 category 빈 문자열을 요구하는 별도 presence sentinel이다. Some(empty)의 경제 국가·점수는 생성하지 않으며 command/정치 capability도 없고 v5만 허용한다. 부분 빈 정의는 오류다.

`SimulationSaveV5`는 기존 v3 base/movement/strait/optional trigger, typed pending command 전체와 Economy 정의 hash/권위/실적 원장을 저장한다. v1~v4 export는 economic presence를 거부하고 legacy restore는 economic context를 거부한다. v5 bounded preflight와 restore는 mode·정의 identity·국가/주/건물/법령 참조·비율/인력/queue/progress·실적 산술을 검사한다. force는 이 검사를 우회하지 않는다. 의미상 오래된 future command는 restore 시 미리 실행하지 않고 성공 step에서 기존 command 규칙으로 소비한다.

Host는 `--scenario ID`로 실제 national 입력을 선택하고 saved server도 같은 명시 context로 복원한다. Economic wire command의 nation은 Join의 실제 선택국에 묶이며 wire가 임의 nation을 지정하지 않는다. Query::Economy의 typed view는 definitions_hash·actual state_hash·현재 nation권위·실적 ledger/project·실제 pending 전체·optional 산업 score를 반환한다. 각 FixedValue는 exact decimal string/value, signed raw bits string, fractional_bits16/32를 함께 내보낸다. client는 shape/필수 field·ID 정렬·범위·십진↔bits 일치만 검증하며 게임 규칙을 계산하지 않는다. ko/en dormancy 문자열은 현지화 키다. UI 화면은 후속 WP 범위다.

기존 current-save capture 아래 새 `economy-v5` namespace는 실제 native repeat save·fresh/paused resume·CLI save-out endpoint·journal/replay와 full DTO/canonical/hash/queue를 보존한다. 기존 원 v1/M1/server/force/historical 경로·workflow는 유지한다. 새 native server 정상/force restore는 실제 경제 query/명령 거부와 futurequeue를 독립 Python encoding/ledger 기준으로 대조한다. 누락·tamper·native exit·source/binary/PID/플랫폼·3OS mismatch 거부 tests는 synthetic control이며 실제 추가 OS 증거가 아니다.

## 벤치 추가 소유 인수

driver/compare/test_compare의 팩 hash 계측은 초기 좁은 소유 밖에서 관측되어 부모가 보류하고 원 diff를 보존했다. CEO 구체 검토 후 추가 범위로 인수했다(ADR-1401, planning/bench-driver-additive-CEO). 원 driver bytes와 동일하지 않으며 과거 실측에 두 필드를 소급 추가하지 않는다. 새 두 실제 library는 같은 새 driver bytes/SHA를 쓰고 timer 이전/elapsed·완료 검사 이후 actual pack_hash를 출력한다. warmup prior-native hash 바인딩 및 모든 표본 before-after/양측/두 쌍 동일성과 원 DTO/canonical/hash/15%·정책을 유지한다. 기술 검토는 실측/독립 PASS가 아니다.

## M2-r3 P-06 복원 불변식 (수정 전 기록)

독립 P05 F-01은 a933의 costs/slots 3단계 정의에서 현재 industry level4를 v5 CLI normal/force와 서버가 수락하는 반례다. 초기 loader는 같은 level4를 거부했다. REQ-ECO-06의 단계·슬롯 계약과 REQ-SAV-02의 이어 실행 계약을 동일 권위 경계에서 검사한다. 정의/format/default/게임 규칙은 추가하지 않는다.

v5의 **현재 World 건물**은 모든 주에서 known map building 및 해당 경제 building 참조, level>=0, level<=costs.len, 단계별 slots[0..level] checked 합계<=해당 주 state_slots를 만족해야 한다. 소유/통제나 현재 프로젝트 유무에 따라 검사를 생략하지 않는다. World 공통 restore의 알려진 참조/음수/정렬/주 집합 검사는 유지하고 경제 전용 stage/slot 검사를 보완한다. 명시 빈 경제는 기존 presence sentinel이고 current 경제 제약 producer가 없으므로 새 제한을 적용하지 않는다. None/v1~v4는 변경하지 않는다.

독립 경계 기대: level0·정확 stage max3·정확 slotlimit는 수락, level4·음수·미등록 map building·등록됐지만 경제 정의 없는 building(0도 포함)·누적 slot 초과·누적 i64 overflow·missing state reference는 거부한다. 현재 level3과 이전 ledger level1이 함께 있는 정상 construction 완료 상태는 수락해야 한다. 소유권 상실/휴면/회복 targetconflict·보존 예약은 기존 검사 범위이며 현재 occupied slots 검사에 active/dormant project 예약을 임의로 더하지 않는다. 모든 복원 실패는 원 Simulation/RestoreContext·입력 save/pack/미생성 output을 보존한다.
