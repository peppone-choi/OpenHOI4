# WP-14 경제·정치 및 고정 측정 입력 스키마

2026-10-08. 구현 기준은 REQUEST-0010 A-CEO-r1 수정판이다. 사용자 확정이 아니다. 이 문서는 구현 전 설계이며 아래 경로는 새 구현 계약이다.

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
| host/wire | 새 상태 presence/DTO/raw bits | old None/v1~v4 불변; additive v5 계획, runtime 검사 | 새 native/save/repro tests |

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

현재 인수 커밋은 data schema/strict loader와 초기 권위·명령·일phase kernel까지만 증명한다. 새 저장/host wire/repro의 최종 구현·native 검사는 계속 진행 중이며 schema 설명의 v5는 아직 완료 증거가 아니다. WP-23은 이 실제 데이터 계약으로 신규 입력을 작성할 수 있으나 전체 경제 producer/저장 통합 PASS로 취급하지 않는다.
