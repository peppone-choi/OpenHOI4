# WP-11 저장 v1 스키마·복원 계약

| 항목 | 값 |
|---|---|
| 작성일·단계 | 2026-10-07, M1-r2 구현 전 설계(90ebf63) 후 P-03 구현 연결. 계약/예제와 실제 실행 결과는 [작업 로그](../worklog/WP-11.md)에서 구별한다 |
| 브랜치·기준 소스 | `codex/wp11-save`, `bbf8762922cf74c3b54f6274c1bff83b0977da62` |
| 근거 | 01 §4·§8 REQ-SAV-01/02, AC-M1-04; 02 §3·§5.11·§7·§12.2·§14.3·§15; P-03 |
| 선행 계약 | ADR-0201, ADR-0401/0402, ADR-0901, ADR-1001; [착수 점검](WP-11-resume-review.md), [기획 규칙](schema-planning.md), [조사 범위](../research/schema-source-review.md) |
| 기술 선택 | [ADR-1101](../adr/ADR-1101-save-v1-boundary.md). 기술 계약 채택은 제품 구현·독립 PASS·M1 게이트 판정을 뜻하지 않는다 |

## 1. 목표·범위·현재 상태

90ebf63 설계 당시 `oh_save/src/lib.rs`는 자리 모듈이고 `Simulation/State/TimeConfig/World/WorldInputs/StatLedger`는 검증된 외부 복원 API가 없었다. 후속 P-03에서 private 권위를 유지하는 DTO 검증 경계와 native 재개를 구현했다. 이 문서의 예제는 기대 계약이며 실제 실행·불충족·독립 검증 상태는 작업 로그를 따른다. 기존 M0 직렬화와 골든은 보존한다.

M1 범위는 파일 encode/decode·안전한 교체, 순수 sim export/import, CLI 저장/재개와 서버 시작 시 저장 파일 로드다. M3 WP-37의 자동저장·순환 보관·HTTP 목록/업로드/다운로드·저장 버튼·구버전 마이그레이션 UI, M2 WP-25의 명령 로그/재현 ZIP은 구현하지 않는다. 국가 PC/법령/지도자·경제/인력/전투/외교·진행 중 RNG 스트림·멀티플레이 배정/소유 권한은 현재 실제 모델에 없으며 이 파일에 가상 저장 필드를 넣지 않는다.

구현 수락에 필요한 증거는 REQ별 최초 실패→통과, 실제 mutable M1 N+M 대 N 저장→새 native process→M의 동일 canonical hash, 오류 시 상태·큐·이전 파일 보존, 3 OS 동일 fixture SHA/재개 hash, 정확 구현 HEAD의 독립 검증과 통합 HEAD CI다. 설계 검사만으로 이 증거를 대신하지 않는다.

## 2. 공통 표현·단일 기준

표의 `필수`는 필드 위치가 항상 존재하고 숨은 기본값이 없다는 뜻이다. postcard는 이름 기반 JSON이 아니므로 누락은 잘린 값/필드 오류, 추가 값은 trailing 오류다. `Option<T>`는 필수 위치에 None/Some 태그를 가진다. None과 ID 0은 다르다. 문자열은 UTF-8 원문, locale/대소문자/Unicode 정규화로 변경하지 않는다. 식별자 검증은 각 행의 기존 도메인을 따른다. 파일의 collection 길이와 문자열 길이는 §8의 할당 전 제한을 적용한다. 표의 SAV-01/NAT-01 등 축약은 각각 REQ-SAV-01/REQ-NAT-01을 가리키며 원래 식별자를 바꾸지 않는다.

DTO의 맵은 **정렬된 `(key,value)` 목록**으로 저장하고 입력 단계에서 엄격 증가·중복을 검사한 뒤 BTreeMap으로 만든다. BTreeMap Deserialize가 중복을 조용히 덮어쓰게 하지 않는다. 엔티티 Vec는 ID 엄격 증가, queue는 `(tick,nation,sequence)` 엄격 증가다. 재정렬해서 잘못된 외부 파일을 정상화하지 않는다. Modifier Vec만 §5의 전체 정렬 키를 사용한다. export도 같은 정렬/검증 계약을 지킨다.

| 계층 | 단일 기준·수명·소유 | 저장·hash·복원 정책 |
|---|---|---|
| 정의/레지스트리 | 검증된 로컬 팩 → oh_data → 불변 Defs, 세션 수명 | 팩 바이트/identity만 저장. Defs/지도 비트맵/Arc/절대 경로는 본문 제외, 로컬 팩에서 재구성 |
| 시나리오 초기값 | oh_data `LoadedNational.scenario`, 새 세션 생성 시 한 번 | 저장된 mutable 상태를 초기값으로 덮어쓰지 않는다. 시작일은 날짜/tick 검증 문맥 |
| 가변 권위 입력 | oh_sim private State/TimeConfig/queue/WorldInputs, 명령/스케줄러만 변경 | 모두 본문과 기존 Simulation canonical hash에 포함 |
| 파생 원장/적용값 | oh_sim이 snapshot tick에 함께 계산·공개 | 저장된 값은 검증 증거. 같은 tick에 재계산해 **모든 행 raw bits와 순서** 비교 후 계산 결과를 설치 |
| 저장 메타데이터 | oh_save/host, 한 파일 수명 | engine/pack/player/saved_at 등. Simulation hash에 추가하지 않음 |
| wire/UI | oh_proto 변환 → 기존 runtime guard → 표시용 client | 저장 DTO를 wire로 보내지 않음. §10 연결 검증 |

## 3. 컨테이너와 헤더: 필드 순서가 v1 포맷 계약

파일은 `OHSV(4 bytes) | format_version(u16 LE) | header_len(u32 LE) | postcard(HeaderV1) | one zstd frame(postcard(SimulationSaveV1))`다. 접두부는 정확히 10 bytes다. 내부 정수는 postcard varint/zigzag이며 native memory dump/LE 고정 정수와 혼동하지 않는다. 표의 순서로 헤더를 직렬화한다. 본문 저장 DTO 버전은 컨테이너 format_version에 종속한다.

| 필드 | 의미·타입·단위·범위 | required/null·기본값 | 참조·권위·검증·수명 | REQ/WP |
|---|---|---|---|---|
| prefix.magic | `[u8;4]`, ASCII OHSV | 필수, null 없음 | oh_save 포맷 식별; 다르면 즉시 거부 | SAV-01 / WP-11 |
| prefix.format_version | `u16`, v1=1 | 필수 | v0/미래값은 본문 해석 전 UnsupportedFormat; force 불가 | SAV-01 / WP-11 |
| prefix.header_len | `u32 LE`, 헤더 바이트 수, 1..header cap | 필수 | checked `10+len`, 파일 범위·cap 검사 뒤 slice 생성 | SAV-01 / WP-11 |
| header.engine_version | String, SemVer, 현재 workspace `0.1.0` | 필수·비어 있음 불가 | 빌드의 CARGO_PKG_VERSION. M1은 정확 버전 일치만 지원; 임의 SemVer 호환 추정 없음 | SAV-01 / WP-11 |
| header.format_version | `u16` | 필수 | prefix와 정확 일치, 중복값 mismatch 거부 | SAV-01 / WP-11 |
| header.scenario_id | String, `[a-z0-9_]+` | 필수 | body.state.scenario와 일치, 명시 로컬 pack에서 로드 가능해야 함. 파일 경로로 신뢰하지 않음 | SAV-01 / WP-11 |
| header.packs | `Vec<PackV1>`; id 오름차순·중복 없음 | 필수·빈 목록 거부 | M1 실제 로더는 단일 팩이므로 정확 1개. 목록 표현은 유지하지만 병합/순서 해결을 구현했다고 주장하지 않음 | SAV-01 / WP-11; MOD-01 / WP-24 후속 |
| packs[].id | String, manifest `[a-z0-9_]+` | 필수 | pack resolver는 호출자가 제공하는 id→root. 저장 파일 안의 root/path 없음 | SAV-01 / WP-11 |
| packs[].version | String, SemVer | 필수 | 로컬 검증된 manifest version 대조; mismatch 처리 §7 | SAV-01 / WP-11 |
| packs[].content_hash | `u64`, FNV-1a64 | 필수, 0도 값 | §7 전체 상대 경로/바이트 identity; defs identity와 구별 | SAV-01 / WP-11 |
| header.game_date | `{year:u32,month:u8,day:u8}` 순서 | 필수 | Date::new와 body date 비교; Gregorian year 1..u32::MAX | SAV-01 / WP-11 |
| header.tick | `u64`, 완료된 게임 시간 수 | 필수, 0 유효 | body.state.tick과 같음. 저장은 step 사이에만 | SAV-01/02 / WP-11 |
| header.seed | `u64`, 게임 RNG 시드 전체 범위 | 필수, 0 유효 | body.state.seed와 같음; wall clock/새 seed로 대체하지 않음 | SAV-01/02 / WP-11 |
| header.state_hash | `u64`, 기존 Simulation canonical postcard의 FNV-1a | 필수, 0 유효 | 검증·재구성한 candidate hash와 정확 일치. 파일 SHA와 다른 검사 | SAV-01/02 / WP-11 |
| header.player_nations | `Vec<u16>` ID 엄격 증가 | 필수·빈 목록 허용 | M1 기본 CLI/observer는 빈 목록. 있으면 world nations에 모두 존재; 한 파일의 호스트 메타데이터이며 연결/권한 승인 아님 | SAV-01 / WP-11; MP-02/03 / WP-49 후속 |
| header.saved_at_utc | `i64`, Unix UTC seconds, 0..253402300799 | 필수·null 없음 | host SystemTime을 save 계층에서 변환. 1970-01-01..9999-12-31, 초 정밀도 표시용, sim hash 제외. fixture는 명시 0 | SAV-01 / WP-11 |
| header.definitions_hash | `Option<u64>` | 필수 위치, M0 None/M1 Some | body.world.definitions_hash와 같고 재로드한 World identity와 같아야 함; force 불가 | SAV-01/02 / WP-11 |
| header.effective_defines_hash | `u64`, 병합 후 로컬 defines identity | 필수 | §7 별도 타입태그 정규화. 시뮬레이션에 없는 metadata hash; force 불가 | SAV-01/02 / WP-11 |

본문에 별도 seed/hash/pack 목록 사본을 더 만들지 않는다. 중복 헤더/두 body/두 zstd frame·skippable frame·뒤의 0 byte도 거부한다. 접두/헤더 format, 헤더/State의 scenario/date/tick/seed, 헤더/World의 definitions_hash 각각을 비교하며 한쪽을 우선하여 수정하지 않는다.

## 4. 본문 SimulationSaveV1: 시간·설정·큐

본문 최상위 순서는 `state, config, queue, world`다. `world: Option<WorldSaveV1>`는 저장 DTO에서는 M0에도 명시 None을 기록한다. 기존 Simulation canonical에서는 M0 world None 필드 생략을 그대로 유지한다. 따라서 **저장 DTO bytes를 canonical state bytes라고 해시하지 않는다**.

| 필드 | 의미·타입·정밀도·범위 | required/null·기본값 | 참조·수명·권위·변경 경로 | hash·복원·근거 |
|---|---|---|---|---|
| state.scenario | String, `[a-z0-9_]+` | 필수 | 초기 시나리오 ID, 세션 수명; host 생성→sim 보관 | 포함; 헤더 일치, SAV-01/02 WP-11 |
| state.seed | `u64`, 전체 범위 | 필수 | 세션 고정, 초기 생성/검증 import만 | 포함; RNG 구조 §6 |
| state.tick | `u64`, 다음 step의 시작 tick | 필수 | 성공한 비정지 step만 +1; paused pump는 유지 | 포함; calendar 관계와 queue 하한 검사 |
| state.date.year/month/day | `u32/u8/u8`, Gregorian | 필수 | year>=1, month 1..12, 실제 월별 day; 성공한 시계 단계만 갱신 | 포함; Date::new, 1900-02-29 거부·2000-02-29 수락 |
| state.hour | `u8`, 0..23 시간 | 필수, 0 유효 | 시나리오 시작 hour=0, tick당 1시간 | 포함; 시작일 ordinal에 tick/24 더한 date, tick%24 hour를 checked 정수로 검증 |
| state.paused | bool | 필수, false를 누락 기본값으로 쓰지 않음 | 명령 단계 변경; 저장을 위한 host 잠금과 게임 pause를 구별 | 포함; 그대로 복원, 자동 해제 없음 |
| state.speed | `u8`, 1..5 | 필수 | 명령 단계 변경 | 포함; 0/6 거부, TIME-01 WP-04 선행 |
| config.speed_ms_per_tick | `[u64;5]`, host pacing ms/게임시간 | 필수, 정확 5개 | 각 0..u64::MAX, 0 허용. sim은 sleep하지 않음 | 포함; 현재 defines 값으로 대체하지 않음. host Instant 덧셈 범위는 server 시작 때 별도 오류 |
| config.initial_speed | `u8`, 1..5 | 필수 | 원래 설정, 현재 state.speed와 같을 필요 없음 | 포함; TimeConfig::new 사용, 보존 |
| queue | `Vec<PendingV1>` | 필수, 빈 배열 유효 | 모든 pending 명령의 소유는 sim. export는 BTreeMap 순서를 그대로 | 포함; 검증 후 BTreeMap 변환, 변경 실패 원자적 |
| queue[].tick | `u64`, 실행 예약 절대 tick | 필수 | saved tick 이상, 동일 tick 허용; 미래 tick의 표현 가능성은 u64 도메인 | 포함; 과거 예약 거부. 현재 마지막 달력일에서 step overflow는 기존 실패 원자성 유지 |
| queue[].nation | `u16`, NationId 정렬 namespace | 필수, 0/65535 유효 | **현재 Pause/SetSpeed는 국가를 참조하지 않음**. 서버가 NationId(0)을 사용하고 팩 국가는 1/2임 | 포함; time 명령에 nation 존재 검사로 현재 큐를 무효화하지 않음. 이후 국가 대상 명령은 variant별 badref 검사 추가 |
| queue[].sequence | `u64`, host 도착 순번 | 필수, 0 유효 | 같은 tick/nation 안에서 유일; 서로 다른 tick/nation은 동일 값 가능 | 포함; 복합키 중복 거부. server 재개 arrival는 남은 queue sequence 최댓값 이상에서 시작, checked +1 |
| queue[].command | enum `Pause(bool)=0`, `SetSpeed(u8)=1` | 필수, 알 수 없는 태그 거부 | Pause bool 정확 decode, SetSpeed 1..5 | 포함; 저장/import 경계에서 invalid speed 거부. 기존 enqueue는 invalid speed를 보관할 수 있지만 그런 순간의 checkpoint는 InvalidQueue 오류, 파일/queue 보존 |
| world | `Option<WorldSaveV1>` | 필수 위치; M0 None·M1 Some | 로더 모드와 대응. M1을 None으로 바꿔 M0 hash를 만들 수 없음 | 일부/전체 포함 정책 §5·§6 |

현재 큐는 time 명령 둘뿐이다. 국가 소유권·전투 대상 검증을 임의로 추가하지 않는다. saved queue sequence가 u64::MAX면 파일 복원 자체는 유효하나 server가 새 입력 순번을 배정할 수 없으므로 시작을 명시 거부한다. CLI의 pending 처리에는 이 host 제한을 덧붙이지 않는다. import는 명령을 실행하거나 큐를 비우지 않는다.

## 5. 본문 WorldSaveV1: 정의 identity·권위 입력·원장

World 순서는 `definitions_hash, inputs`; inputs 순서는 `nations, states, provinces`다. 표의 필드는 모두 세션 수명이며 수정 주체는 oh_sim 명령/스케줄러 또는 검증된 import다. 이름/색/수도/지형/인접/지도 인덱스는 불변 Defs가 소유하고 본문에 중복하지 않는다. 모든 수치 Fx는 `raw_bits:i64` 한 필드로 저장, 정확한 I32F32 `bits / 2^32`, 최소 -2^31·최대 2^31-2^-32다. `Fx::from_bits`만 사용하며 문자열/f64/반올림 표시값을 거치지 않는다. 아래 필드는 **전부 기존 M1 canonical hash 포함**이다.

| 필드 | 의미·타입·실효 범위 | required/null·기본값 | 참조·기준·검증 | REQ/WP·상태 |
|---|---|---|---|---|
| world.definitions_hash | `u64`, 불변 의미 identity | 필수 | 헤더 및 로컬 재구성 World와 정확 일치; 포함 경계 §7 | SAV-01/02 WP-11, NAT-01 WP-09 선행 |
| inputs.nations | `Vec<NationV1>` ID 엄격 증가 | 필수, M1 비어 있음 거부 | 로컬 정의 nation ID 집합과 정확 대응, 추가/누락 거부 | SAV-02 WP-11, NAT-01 WP-09 선행 |
| nations[].id | `u16`, 안정 NationId | 필수, 0/65535 유효 | 순번 아님; 정의 ID 존재·중복 없음 | 동일 |
| nations[].tag | String, 대문자 ASCII/숫자·비어 있지 않음 | 필수 | 해당 ID의 정의 tag와 일치, 전체 tag 유일; mutable serializer에 있는 식별자 사본 | 동일 |
| nations[].government | String, 비어 있지 않은 현지화 키 | 필수 | 현재 M1은 초기 정부 효과 없음. 기존 로더는 현지화 키 존재 레지스트리를 검사하지 않으므로 save가 새 정치 registry를 만들지 않음 | NAT-01 WP-09 선행; 구체 정치 규칙 WP-14 후속 |
| nations[].support | 정렬 `(String,i64 raw bits)` 목록 | 필수·빈 목록 거부 | 키 비어 있지 않음·유일, 값 0..2^32·checked 합 정확 2^32. 이름은 이념 현지화 키, tag와 별개 | NAT-01 WP-09 선행; SAV-02 WP-11 |
| inputs.states | `Vec<StateV1>` ID 엄격 증가 | 필수 | map state ID 집합과 정확 대응, 모든 주 검사 뒤 공개 | MAP-03/08 WP-07/09 선행; SAV-02 WP-11 |
| states[].id | `u16`, 안정 StateId | 필수, 0 유효 | 정의 주 존재 | 동일 |
| states[].owner | `u16`, 법적 귀속 NationId | 필수·None 불가 | nations 존재; 시나리오 초기 소유와 같아야 하는 제한은 없음 | MAP-08 WP-09 선행; 변경 WP-20 후속 |
| states[].population | `i64`, 초기/현재 인구 수, 0..i64::MAX | 필수 | 현재 M1 저장 가능한 입력. **가용 인력 아님**; 새 인력 공식 없음 | MAP-03 WP-07/09 선행; SAV-02 WP-11 |
| states[].resources | 정렬 `(registry id:String,i64)` 목록 | 필수, 빈 목록 유효 | 수량 0..i64::MAX, 로드된 resource registry에 존재·키 중복 거부. 누락 key는 목록에 없는 그대로, 0 항목 삽입 안 함 | 동일; 경제 WP-14 후속 |
| states[].buildings | 정렬 `(registry id:String,i64)` 목록 | 필수, 빈 목록 유효 | 단계 0..i64::MAX, building registry 존재·중복 거부. 슬롯/비용 미승인 규칙 없음 | 동일; 건설 WP-14 후속 |
| states[].base | `i64 raw bits`, 인프라 기본 Fx | 필수 | 기존 초기 정의는 nonnegative integer; 저장형은 Fx. 음수 base 거부, 양의 최하위 bit 보존. definition 초기값으로 덮어쓰지 않음 | UI-04 WP-10 선행; SAV-02 WP-11 |
| states[].modifiers | `Vec<ModifierV1>` | 필수, 빈 목록 유효 | 전체 키 `(target_stat,op,source,expires,value raw signed bits)` 비감소. expired 항목도 보존, 동일 전체 expired 항목도 현재 모델에서 표현 가능하므로 삭제하지 않음 | UI-04 WP-10/09 선행; SAV-02 WP-11 |
| modifiers[].source | String, 기여/현지화 식별자 | 필수 | 비어 있음 거부, source별 규칙/수치 도입 없음 | 동일 |
| modifiers[].target_stat | String | 필수 | 현재 World 적용 대상 `infrastructure`만 허용; 새 stat는 별도 schema/WP | 동일 |
| modifiers[].op | enum Add=0/Mul=1 | 필수 | 미지 enum 거부. source가 같아도 Add/Mul 각각 허용 | 동일 |
| modifiers[].value | `i64 raw bits`, Fx 전체 signed 범위 | 필수 | 음수/0 허용, 임의 clamp 없음; checked 계산 overflow는 거부 | 동일 |
| modifiers[].expires | `Option<u64>`, 첫 제외 절대 tick | 필수 위치, None=영구 | `saved tick < expires`일 때만 활성. Some(0)/과거/현재 만료 유효, 절대 날짜가 아님 | 동일 |
| states[].infrastructure | `i64 raw bits`, 실제 적용 Fx | 필수 | 임의 게임 상한/음수 clamp를 넣지 않음. 재계산 ledger.value raw bits와 정확 같아야 함 | 동일 |
| states[].ledger | `LedgerV1` | 필수 | 계산 결과 사본, 아래 모든 필드 재계산 비교. 외부 StatLedger Deserialize 없음 | 동일 |
| ledger.target_stat | String | 필수 | infrastructure 및 계산 대상과 일치 | 동일 |
| ledger.tick | `u64` | 필수 | State.tick과 정확 같음; 오래된 원장 거부. 현재 World.evaluate는 advanced=false인 paused step에도 next.tick에서 실행하고 초기 원장 tick은 0 | 동일 |
| ledger.value | `i64 raw bits`, 최종 Fx | 필수 | 적용값과 계산결과 모두 일치 | 동일 |
| ledger.entries | `Vec<EntryV1>` | 필수·최소 1개 | 기본행 1개→활성 Add source 사전순→활성 Mul source 사전순. 행 수·순서·필드 전부 비교 | 동일 |
| entries[].source | `Option<String>` | 필수 위치 | 첫 Base만 None; 나머지 Some(정확 modifier.source) | 동일 |
| entries[].op | enum Base=0/Add=1/Mul=2 | 필수 | 첫 Base, 이후 계산 단계와 일치 | 동일 |
| entries[].value | `i64 raw bits` | 필수 | 첫 base, 이후 해당 modifier.value 그대로 | 동일 |
| entries[].accumulated | `i64 raw bits` | 필수 | 행별 checked_add/checked_mul 이후 결과, 표시된 소수와 비교하지 않음 | 동일 |
| inputs.provinces | `Vec<ProvinceV1>` ID 엄격 증가 | 필수 | map province ID 집합과 정확 대응, water도 누락 불가 | MAP-03/08 WP-07/09 선행; SAV-02 WP-11 |
| provinces[].id | `u16`, 안정 ProvinceId | 필수, 0 유효 | dense GPU index와 다름, 정의 province 존재 | 동일 |
| provinces[].state | `Option<u16>` | 필수 위치 | land=Some(map의 정확 StateId), sea/lake=None | 동일 |
| provinces[].owner | `Option<u16>` | 필수 위치 | land=Some(그 주의 현재 owner), water=None. Some(0)는 nation 0 소유 | 동일 |
| provinces[].controller | `Option<u16>` | 필수 위치 | land=Some(존재 NationId), owner와 달라도 유효; water=None | 동일; 변경 WP-20 후속 |

기존 Scenario 로더는 모든 초기 modifier의 `(source,operation)` 유일성을 요구하고, 현재 계산기는 **활성** `(target_stat,op,source)` 중복만 거부한다. 저장은 더 넓은 실제 mutable 입력 경계에 맞춰 expired 중복을 보존하되 현재 활성 중복을 거부한다. 전체 항목의 target/source shape 검사는 만료 여부와 무관하게 수행한다. M1 canonical 정렬과 해시를 변경하지 않는다.

복원에서는 주 하나의 검증 성공으로 즉시 공개하지 않는다. 국가→전체 주 입력→전체 프로빈스 참조→전체 주 원장 재계산·대조 순서로 candidate를 만든다. resource/building registry는 현재 Defs에 없으므로 §11의 읽기 문맥 확장이 필요하다. 정의 주들에 등장한 key 합집합을 registry의 대용으로 쓰면 등록됐지만 아직 사용되지 않은 key를 잘못 거부하므로 그 방식은 쓰지 않는다.

## 6. 참조 그래프·canonical hash·RNG

```text
host PackResolver(id -> local root) -> validated LoadedNational -> immutable Defs
 Header.packs[1] -> manifest/version/whole-pack content_hash
 Header.scenario_id -> scenario.start_date + effective Defines
 Defs.nations[id] <1:1> WorldInputs.nations[id]
 Defs.map.states[id] <1:1> WorldInputs.states[id] --owner--> nations[id] (N:1)
 Defs.map.provinces[id] <1:1> WorldInputs.provinces[id]
 land province --state--> exactly 1 defined state --contains--> >=1 land province
 land province --owner--> that state's owner; --controller--> exactly 1 nation
 water province --state/owner/controller--> None
 state.base + all modifiers --evaluate(State.tick)--> ledger + infrastructure
 Simulation.state + config + ordered queue + optional world -> canonical hash
```

국가/주/프로빈스 ID 도메인은 분리한다. 동일 정수 ID가 서로 다른 도메인에 존재해도 충돌이 아니다. 관계 중복/미존재/ID 집합 누락/육지의 복수 주 소속을 거부한다. 저장에는 주→프로빈스 목록을 별도 mutable 필드로 넣지 않으므로 저장이 정의의 소속 그래프를 바꿀 수 없다. 지도 인접 순환은 정상이며 로컬 로더가 검사한 그래프를 사용한다. 현재 모델에는 생성/삭제/alive allocator나 계층 순환 필드가 없어 가상 allocator/cycle 검사 항목을 저장하지 않는다.

canonical hash는 기존 `oh_core::state_hash(&Simulation)` 경계다: State 필드 순서, TimeConfig, BTreeMap pending, M1 World의 definitions_hash 및 모든 WorldInputs/원장. Defs 자체·header.engine/packs/player_nations/saved_at·압축률·파일 경로·host Instant/deadline/channel·wire 문자열·카메라·selection·GPU 캐시는 제외한다. header.state_hash를 payload 자체 해시나 M0 시간 hash로 대신하지 않는다. FNV는 인증 서명이 아니며 파일 SHA256은 독립 증거/무결성 대조용이다.

현재 Simulation에는 persistent RNG/draw cursor가 없다. oh_core의 `RngKey { game_seed:u64, system:SystemId(u32), day:GameDay(u64), entity:EntityId }`가 ChaCha8 시작 스트림을 결정한다. seed_bytes는 seed(8LE)+system(4LE)+day(8LE)+entity tag(4LE)+entity ID(8LE); Global=0, Province=1, State=2, Nation=3, Division=4다(ADR-0201). GameDay는 시작 이후 경과일이며 달력 날짜와 별도 타입이다. 저장하는 실제 seed/date/tick으로 현재 경과일을 보존한다. 현재 M1 scheduler는 RNG를 소비하는 gameplay 시스템이 없으므로 RNG 소비 재개를 검사했다고 쓰지 않는다. 향후 시스템이 스트림 중간/분할 작업을 틱 사이에 유지하면 그 실제 cursor/작업 상태를 새 format·canonical·독립 fixture에 추가해야 한다. 시작 키를 재생성하는 것만으로 중간 draw 위치가 복원되지는 않는다.

## 7. 세 identity와 --force

`content_hash`는 팩 root 아래 모든 일반 파일의 `(상대 UTF-8 경로, bytes)` Vec를 **전체 상대 경로 사전순**으로 정렬하고 기존 postcard/FNV-1a64로 계산한다. 경로 separator는 `/`; file bytes는 원본 그대로이며 CRLF/LF를 정규화하지 않는다. root 절대 경로·mtime·열거 순서 제외. symlink/reparse link·비정상 파일·비UTF-8 경로·읽기 실패를 거부한다. manifest/defines/localisation/SOURCES/지도/시나리오 등 파일 선택 필터 없음. 저장 대상은 pack tree 밖이어야 한다. save 파일이 pack 해시를 바꾸는 자기참조를 막는다.

현재 server의 `pack_hash`는 directory별 정렬 DFS와 `to_string_lossy`를 사용한다. 이는 임의 경로(`a/x`, `a.txt`)에서 전체 상대 경로 정렬과 다르고 손실 경로 충돌도 가능하다. 구현 시 oh_save의 공통 host helper로 server/CLI identity를 통일하고 해당 역전 fixture를 검사해야 한다. 기존 protocol 필드 의미는 동일하고 지도/client fixture 변경은 하지 않는다. CI checkout된 pack bytes가 3 OS에서 같은지도 evidence로 따로 기록한다.

`definitions_hash`는 현재 World::from_loaded 경계 그대로: nation 정의(정부/초기 지지율 포함), province 정의와 kind/terrain/RGB/coastal/island, state id/name_key/province 목록, width/height/index, palette/map id/centers, ordered edges(kind/distance Fx), VP, km_per_pixel. State 초기 population/resources/buildings/infrastructure·scenario ownership/control/modifiers·effective defines는 이 identity에 없음을 명시한다. 이 입력들은 pack content_hash가 커버하고 현재 저장된 WorldInputs가 권위다. state definition membership은 Defs에서 검증한다. `definitions_hash`만으로 전체 팩 같음을 주장하지 않는다.

`effective_defines_hash`는 root defines + 시나리오 overlay 후 BTreeMap system/item 순으로 `(system, Vec<(item, TaggedValue)>)`를 postcard/FNV로 해시한다. TaggedNumber는 Integer=0(i64)/Fixed=1(i64 raw bits), TaggedValue는 Scalar=0(TaggedNumber)/Array=1(Vec<같은 타입>)다. `1`과 `1.0`, scalar와 한 요소 array를 구별한다. arrays는 순서 유지. host save cap은 별도 `oh_save/defines.toml`에서 읽으며 game effective defines hash나 sim hash에 넣지 않는다.

| 비교 | 기본 | 명시 --force | 결과/이유 |
|---|---|---|---|
| magic/format/engine/version 구조·범위·길이·압축·본문 hash·모든 참조/원장 | 거부 | 거부 | 손상/미지원 우회 불가 |
| pack id/목록 집합·scenario 존재 | 거부 | 거부 | M1 single pack resolver 계약; 엉뚱한 팩 대체 불가 |
| pack content_hash/version만 달라짐 | 거부+진단 | 아래 조건 전부 통과 시 경고+수락 | 예: localisation/SOURCES 또는 manifest version 변경. 경고에 stored/current id/version/hash 기록 |
| 로컬 definitions_hash != 저장 hash | 거부 | 거부 | 새 정의로 hash를 조용히 재기준화하지 않음 |
| effective_defines_hash != 저장 hash | 거부 | 거부 | 새 규칙/계수를 강제로 수용하지 않음 |
| saved 날짜/tick과 현재 scenario start_date 관계 불일치 | 거부 | 거부 | 달력/RNG day 의미 보존 |
| state input/관계/원장·player ID 오류 | 거부 | 거부 | force로 임의 world를 활성화하지 않음 |

force에서도 원래 header.state_hash와 candidate hash는 같아야 한다. 정의를 변경하여 생긴 hash 차이를 새 값으로 고치지 않는다. 수락한 파일은 원본 bytes를 수정하지 않으며 재저장할 때 현재 검증된 팩 metadata로 새 파일을 만든다. force로 수락한 provenance는 host 진단에 남기고 UI 경고 화면은 WP-37 후속이다. pack 파일이 load 사이에 변하면 전/후 content hash가 다르므로 거부한다. M1은 호출자 소유의 안정 로컬 파일을 전제로 하며 적대적 동시 filesystem 변경에 대한 보안 경계를 주장하지 않는다.

## 8. decode 한계·실패 순서·포맷 호환

정책 수치는 새 `crates/oh_save/defines.toml`의 `[save]`로 host에 동봉한다. game rules/시계와 분리된 신뢰한 정책이며 저장 파일의 cap 필드를 신뢰하지 않는다. 코드에는 정책 숫자를 중복하지 않는다. 계획값은 header_max_bytes=65536, file_max_bytes=67108864, body_max_bytes=268435456, string_max_bytes=4096, queue_max_entries=1000000, modifiers_per_state=4096, map_entries_max=65536, packs_max=64, zstd_window_log_max=23, compression_level=3이다. 본문/문자열/집합 budget 합산은 checked 정수로 한다. nation/state/province 수는 u16 ID 도메인 한계와 로드된 정의의 정확 개수를 함께 제한한다. ledger 행 수는 modifiers cap+1이다. cap 정책 변경은 포맷 필드가 아니라 host 수용 정책 변경이며 오류 진단에 현재 limit을 낸다.

1. 제한된 read로 file_max+1을 검사한다. metadata 길이만 믿거나 무제한 read_to_end 하지 않는다. 10B prefix truncation·magic·format→header_len/checked offset→bounded header 순서다.
2. allocation-free shape preflight에서 String slice/UTF-8·collection 개수·입력 잔여량과 allocation budget을 먼저 검사하고 한 postcard 값으로 typed decode한다. 검사 전 collection size_hint를 reserve하지 않는다. `u64::MAX` 개수+짧은 입력은 allocation 전에 실패해야 한다. 구현의 추가 host budget/charge는 ADR-1101 구현 보강을 따른다.
3. 모든 required 필드·format/engine/metadata shape·duplicate/missing pack ID를 검사한다. bad metadata면 zstd를 열지 않는다. 허용 경로는 호출자가 정하며 header로 임의 경로를 읽지 않는다.
4. zstd 표준 frame magic 확인, dictionary 없음, skippable/concatenated 금지. `Decoder::with_buffer(&[u8]).single_frame()`와 window cap을 사용하고 body_max+1을 넘으면 즉시 오류. frame content size가 없거나 거짓이어도 실제 출력 cap 적용. 끝까지 읽어 frame truncation/checksum을 검사하고 `finish()`의 미소비 compressed slice가 비어 있음을 확인한다.
5. body 한 postcard 값·잔여 없음·bounded DTO 검사. decode 후 같은 DTO를 재encode하여 원본 postcard header/body bytes와 비교한다. overlong/nonminimal varint 같은 비정규 표현을 거부한다. canonical reencode는 구조 검사를 대신하지 않는다.
6. 팩 identity·metadata 중복·DTO 관계·달력/큐→모든 원장 재계산 비교→candidate canonical hash 순서로 검사한다(§9). 오류마다 `section/field/entity ID/queue key/expected/actual` 문맥을 준다. 바이너리에는 TOML 줄/열이 없으며 byte offset 가능한 decode 오류와 pack 로더의 원래 path/line/column을 구별한다.

M1 첫 포맷은 v1만 읽는다. v0/v2 fixture는 알려진 버전 메시지로 거부하며 변환 경로를 제공했다고 쓰지 않는다. 형식 변경 PR은 §7.2와 이전 유효 샘플의 decode→마이그레이션 또는 명확한 거부→hash fixture를 함께 둔다. 새 optional 필드도 postcard positional layout 변경이면 버전을 올려야 한다. engine 버전 다름은 UnsupportedEngine이고 `--force`로 완화하지 않는다. zstd 구현 버전/압축 설정 변경은 로드 가능성과 fixture SHA 변화 모두 검토하되 압축 bytes를 sim hash에 넣지 않는다.

## 9. 순수 import·파일 교체의 원자성

예정 sim API는 `Simulation::export_save() -> Result<SimulationSaveV1, SaveStateError>`와 `Simulation::from_save(dto, RestoreContext) -> Result<Simulation, SaveStateError>`다. 타입 이름은 구현 때 동일 의미로 조정 가능하지만 책임은 고정한다. DTO는 `oh_sim::save_state`에 두며 Serialize/Deserialize가 가능해도 private live 타입은 Deserialize를 열지 않는다. RestoreContext는 host가 검증한 시작일·불변 defs/world template·registry 집합을 공급한다. sim은 filesystem/clock/pack discovery를 호출하지 않는다.

`oh_save::decode(bytes, resolver, limits, force)`는 header와 검증된 candidate를 반환하고 caller의 live sim을 받지 않는다. caller는 Ok일 때만 `live = candidate`로 한 번 교체한다. 따라서 decode 실패는 이전 state/config/world/queue/조회 결과에 영향이 없다. export는 read-only이고 실패해도 queue를 소비하지 않는다. 원장의 source·op·value·accumulated·target/tick/final bits와 적용값을 각각 대조한다. 계산 overflow/활성 중복은 전체 import 실패다. 모든 입력을 설치한 뒤 원장을 나중에 고치는 방식은 금지다.

파일 저장은 host가 step 경계에서 일관된 export를 얻은 뒤 encode/검증을 끝내고, 대상과 **같은 디렉터리**의 tempfile에 전체 bytes를 write_all→flush→file.sync_all→`NamedTempFile::persist(target)` 순서로 실행한다. 기존 target을 지우거나 먼저 truncate/copy하지 않는다. 부모 디렉터리는 미리 존재해야 하며 임시파일/target이 pack 아래면 거부한다. caller는 대상별 쓰기를 직렬화한다. 최종 교체 전의 encode/write/flush/sync/persist 실패는 이전 target bytes와 live 상태/queue를 보존한다. 실패 임시파일 cleanup 오류는 진단하고 다른 파일을 삭제하지 않는다.

최종 persist가 commit point다. 성공 뒤 directory sync 지원/실패는 `Committed { durability_warning }`으로 보고하고 이전 파일이 보존됐다는 Err로 잘못 반환하지 않는다. M1 보장은 읽는 쪽이 완전한 옛 파일 또는 완전한 새 파일을 보는 교체 원자성이다. 전원 손실/특수·네트워크 filesystem의 완전한 crash durability를 보장했다고 쓰지 않는다. 표준 tempfile persist도 파일/디렉터리 sync를 자동 수행하지 않는다. 최초 저장/덮어쓰기·Windows 잠긴 파일·Unix rename 실패를 3 OS에서 각각 시험한다. 테스트 전용 writer fault injection은 실제 shared write/commit 경로를 호출하고 product retry/timeout을 완화하지 않는다.

## 10. CLI·서버·wire·실제 조회

다음은 **예정 인터페이스**이며 이번 설계 작업에서 실행하지 않았다.

| 경로 | 예정 계약 | 성공/실패·권위 |
|---|---|---|
| `oh_cli run --pack <root> --scenario <id> (--ticks N 또는 --days N) --seed S --save-out <file> [--hash-out]` | 기존 M1 run 후 step 경계에서 저장; 기존 인수 의미/M0 run 골든 보존 | 저장 실패면 nonzero, 이전 file 보존. hash-out stdout은 기존 16 hex만, 경고/저장진단 stderr |
| `oh_cli resume --load <file> --pack <root> (--ticks M 또는 --days M) [--force] [--save-out <file>] [--hash-out]` | header scenario/seed/time/config/world/queue를 복원. seed/scenario/time override 인수 없음 | 입력/file 오류 nonzero. M은 **advanced ticks** 수이며 paused step pump를 진행으로 세지 않음. 현재 tick resume 없이 영구 paused면 PausedCannotAdvance로 종료, 임의 자동 resume 없음 |
| server `--load-save <file> [--force]` | 기존 pack-root로 native 시작 때 완전 검증한 M1 template 사용. 현재 server 지원 m1 또는 기존 M0 testland만, 다른 scenario 명시 거부 | bind/세션 공개 전 실패. Join(local,None)는 저장 상태 clone에서 시작; saved pause/speed 보존. Create는 load 모드에서 unsupported-create, 새 seed로 저장값 덮어쓰기 없음 |
| Session 시작 | saved pending queue를 보존, arrival=max pending sequence 또는 0; checked 새 순번 | 현재 host Instant 범위/arrival overflow는 시작 전 오류. 새 Instant/deadline은 host pacing이며 sim hash 제외 |
| 기존 Snapshot/WorldResult/Query | 같은 oh_proto 변환으로 로드한 actual world와 원장 표시 | 기존 Rust→TS→runtime guard 그대로. 저장 바이트/DTO를 MessagePack 메시지로 받아들이지 않음 |

M1 서버 연결은 **CLI에서 만든 파일을 새 서버 프로세스로 로드하여 기존 브라우저에서 조회**하는 동선이다. live browser에서 save 버튼을 누르거나 업로드/다운로드한 동선은 없다. 서버 자체의 live save 요청·공유 세션/파일 저장권한은 WP-37/WP-49에 맡긴다. M1 CLI 저장 자체가 실제 서버 thread checkpoint라고 보고하지 않는다. server saved template은 현재 연결마다 별도 sim을 만드는 모델에 따르며 여러 플레이어가 같은 mutable 세션을 공유한다고 주장하지 않는다.

저장 v1 때문에 protocol version/variant를 추가하지 않는다. TimeState date/hour/speed/paused/tick 문자열, WorldView nation/tag/정부/지지율·state/population/resources/buildings/infrastructure/ledger·province owner/controller/null은 기존 서버 변환과 runtime shape 검사를 거친다. 로드한 값을 원래 scenario로 재생성하여 보여주는지, ID0을 None으로 표시하는지, 원장 최하위 bit가 표시 반올림에 묻혀도 서버 hash/원장 raw bits가 일치하는지를 테스트한다. 실제 브라우저 조회/재개 PT는 새 서버 exact exe/JS/pack identity와 URL/PID를 기록하고 scripted fixture 주입과 분리한다. client 지도/Playwright fixture는 WP-11이 수정하지 않는다.

## 11. 구현 소유·범위 의존·다음 순서

| 소유 파일·책임 | 예정 변경 |
|---|---|
| `crates/oh_save/{src,tests,examples,tools}/`, Cargo.toml, 새 defines.toml | HeaderV1/limits/codec/pack identity/file commit; fixture helper·독립 format 기준·새 process DT·3OS evidence |
| `crates/oh_sim/src/save_state.rs`, lib.rs/time.rs/world.rs 및 관련 tests | private getter 기반 export, 순수 검증 import·원장 비교. 기존 Serialize/hash 경계 유지 |
| `crates/oh_cli/{src,tests}/`, Cargo.toml | save-out/resume 최소 native 경로; 파서/paused/error/출력 계약 |
| `crates/oh_server/{src,tests}/`, Cargo.toml | 시작 load-save/template·arrival 복원·공통 pack hash 호출만. map handler/지도 fixture 소유는 WP-08 |
| 새 `.github/workflows/save-determinism.yml` | 현재 pinned action/toolchain을 재사용한 save 전용 3OS capture/compare. 일반 client steps/기존 검사 약화 없음 |
| `docs/plans/WP-11-schema.md`, ADR-1101, worklog/WP-11.md | 계약·검증 증거·범위 보고 |
| root Cargo.toml/Cargo.lock·.gitattributes·assets/ASSETS.toml | 공유파일 필요 항목만: 의존 resolution/pin, `.ohsave binary` fixture 취급, 직접 제작 fixture 등록. 사전 충돌 대조, 타세션 수정 보존 |

**추가 소유 배정 반영:** 설계 때 없던 oh_data resource/building registry read-only 반환은 후속 P-03에서 `crates/oh_data/src/map.rs`와 관련 tests를 좁게 배정받아 구현했다. registry 파서를 oh_save에 복제하거나 definition에 나온 key만 허용하지 않았다. 기존 검증 결과 전달이며 지도 HTTP/client 렌더링 변경과 분리한다.

다음 구현 순서: (1) 후속 지시/읽기 문맥 소유 확정, T01/T02 red와 format fixture red 기록 → (2) sim DTO/export/import·참조/원장 실패 원자성 → (3) bounded postcard/zstd/header/identity·파일교체 → (4) CLI native fresh-process 연결 → (5) server startup load·조회/arrival → (6) 독립 정수/바이트 기준·3OS capture 및 실패 증거 검사 → (7) fmt/clippy/workspace test·M0/M1 hash2·docs/license/assets, 자기 브랜치 커밋 → (8) 새 정확 HEAD 독립 검증에 계약/fixture/실패 이력 전달. 구현자가 PASS/완료 게이트를 판정하거나 main을 병합하지 않는다.

남은 기술 확정은 Cargo의 실제 zstd-safe/sys·tempfile 전이 resolution과 각 라이선스/3OS build 확인, 압축 bytes 동일성의 실제 실행, host Instant 최대 pacing 수용 범위의 각 OS 진단이다. 상위 계약은 정해졌으며 결과가 다르면 테스트 기대값/timeout을 낮추지 말고 ADR에 정확한 제약·변경안을 보고한다. 사용자 게임 규칙 결정이 새로 필요한 항목은 현재 없다.

## 12. 독립 입력→기대값 예제 (실행 결과 아님)

fixture는 직접 제작하며 `tests/fixtures/save-v1/`(정확 위치는 oh_save tests 아래 가능)에 JSON 설명/ohsave bytes/SHA256/pack inventory·canonical bytes·FNV 기대값을 함께 둔다. native fixture helper는 시험 입력만 만드는 executable이며 게임 UI 행동/사용자 플레이 증거가 아니다. 독립 검증자는 구현 `state_hash`를 호출해 얻은 값만 기대값으로 복사하지 않는다. 별도 postcard 정수 encoder/FNV·Gregorian ordinal·i128 rawbit 계산으로 expected bytes/hash/ledger를 만든다. 기존 골든은 갱신하지 않는다. 오케스트레이터 독립 초안 `E:/openhoi/docs/plans/M1-r2-gate-evidence.md`는 작성 시 전용 worktree에 없고 main의 문서를 읽기만 했으며, 아래 floor 예제는 그 자체 계산 계약과 대조했다. 그 초안의 branch 통합은 부모 담당이다.

| ID·대상 | 입력·동선 | 독립 기대값과 오류 후 불변성 |
|---|---|---|
| T01 SAV-01 | M1 world Some·모든 header 필드·rawbit/expired modifier·nonempty queue 저장/로드 | 저장 시/로드 직후 canonical bytes 동일, 필드별 값·순서 일치. 헤더 표시시각 변경은 state hash 동일, 파일 SHA는 변경 |
| T02 SAV-02/AC-M1-04 | 실제 M1 fixture start=2000-02-28, seed=7, N=2일/M=2일. 연속96 advanced ticks vs48 tick 저장→프로세스 종료→새 native process 로드→48 advanced ticks | split tick48=2000-03-01 00시, 끝 tick96=2000-03-03 00시. actual nation/state/province/원장·queue 포함 canonical hash 동일. PID/process exit/각 command/head 기록 |
| T03 예약순서 | tick48에 (nation0,seq1)Speed2, (0,9)Pause(true), (42,1)Pause(false), (42,3)Speed5; tick49에 Speed3 | 명령 결과 순서가 위 복합키 순서, tick48 step 후 speed5·paused=false·tick49; 다음 step speed3·tick50. 역 insertion order도 같은 queue bytes/hash. import가 advance/소비하지 않음 |
| T04 pause/5속도/config | 별도 speed1..5 각각 저장, config=[0,7,11,13,17], initial_speed2·현재 speed5; paused tick48에 resume 현재 tick 예약 또는 resume 없는 파일 | loaded state/config 정확, 0 pacing 유효. resume 있는 파일 pump 후 advance; 없는 paused 파일 추가 days1 CLI는 PausedCannotAdvance nonzero·원본 file/queue 보존; zero days는 저장 가능 |
| T05 expiry/rawbits | state3 base=2^32 bits, Add(a.raw)=1bit expires49, Add(b.add)=2^31 bits, Mul(c.mul)=2*2^32 bits. tick48/49/50에 별도 저장/새 process 로드 | tick48 ledger final=12884901890 bits·Base/Add/Add/Mul 4행; tick49/50=12884901888 bits·Base/Add/Mul 3행. 입력 a.raw는 계속 잔존. 만료 당일 date가 아니라 정확 tick으로 제외 |
| T06 signed/경계 | i64::MIN/MAX·-1/0/1 Modifier bits, expiry None/0/MAX; base>=0, 음수 Mul 별도 fixture | 저장형 rawbits 자체 손실 없음. 각 계산 `Add=i128 합`, `Mul=(a*b)>>32` 단계별 i64 범위; overflow는 전체 거부, 후속 상쇄 허용 안 함 |
| T06b 중간 floor/역순 | Q=4294967296, base=2Q+1, Add=Q expires24, Mul=3Q/2. tick23/24/25 저장, modifier 입력 순서를 뒤집은 같은 데이터로 초기 생성 | tick23=19327352833 bits, tick24/25=12884901889 bits. 0.5 rawbit가 단계별 floor로 버려지는 기대값을 i128로 검사. expired Add 입력은 유지, 원장 행에서만 제외. 초기 로더의 정렬 뒤 canonical 같음; 정렬 안 된 외부 saved Vec는 거부 |
| T07 비연속/0ID | nation {0,42}, state {3,65535}, land province {0,10,65535}, water {60}. p0 state3 owner0 controller42, water 전부 None | Some(0) 보존·ID lookup 성공, Vec 순번으로 해석 안 함. state3 owner와 p0 owner 불일치/unknown controller/land None/water Some/주 소속 변경은 InvalidReference |
| T08 상태/키 shape | nation/state/province 중복/미존재/누락·map key 중복/역순·unknown resource/building·support 합 2^32±1·negative count·empty government/source·unknown stat | 해당 field/entity 오류. 기존 sim snapshot/hash/config/world/queue byte 비교 동일, 정상 후속 query/step 가능 |
| T09 날짜/큐 | year0·1900-02-29·month13·hour24·valid date/tick 불일치·speed0/6·config 길이4/6·past tick·동일 복합key·unknown command tag | 범위/관계 오류. 2000-02-29·0ID/seq0·같은 sequence 다른 nation/tick 수락. time queue nation namespace를 국가 owner badref와 혼동 안 함 |
| T10 원장/중복 | stored ledger tick/target/value/행순서/source/op/value/accumulated 중 각각 한 필드 변조, applied bits ±1, active 동일 source/op 중복 | LedgerMismatch 또는 DuplicateModifier, 전체 상태/queue 불변. expired 동일 source/op만은 보존/수락; same source Add+Mul 수락 |
| T11 container | 0..9B 접두 truncation·bad magic·v0/v2·header_len0/MAX/초과·중간 truncation·잘못된 varint/bool/option/enum | 분류된 오류, large size_hint+짧은 입력은 cap/decoder 검사에서 할당 전 실패 |
| T12 압축/잔여 | 압축 잘림/checksum 오류·body cap 초과 팽창·window cap 초과·두 frame/empty second frame·skippable·frame 뒤0·header/body postcard 뒤0 | 각각 Compression/Limit/Trailing 오류; 첫 정상 frame만 조용히 수락하지 않음. cap-1/cap/cap+1 각 경계, 불변성 동일 |
| T13 중복metadata/hash | header date/tick/seed/scenario/format/defs 중 1개만 변경, hash만 ±1, 누락/중복 player/pack·bad saved_at | MetadataMismatch/StateHashMismatch 등; state hash 검사가 shape/ref/ledger 검사 대용이 아님. force에서도 거부 |
| T14 identity/force | localisation bytes/manifest version만 변경 vs 정의 RGB/map membership/defines/start_date 변경, pack ID 대체 | 첫 두 경우 기본 거부, force 경고+원래 canonical hash 그대로. defs/defines/refs/start_date/id 오류는 force도 거부. 원본 파일 수정 없음 |
| T15 파일보존 | 기존 file SHA 고정. encode cap·write/flush/sync fault·persist 실패·Windows 공유잠금 또는 Unix rename 실패 주입, 최초 저장·성공 덮어쓰기 | commit 이전 실패: old SHA·live state/queue 동일, 무한 retry 없음. 성공: new 완전파일 decode. commit 뒤 durability 경고는 committed로 구분, old 보존 오류로 보고 안 함 |
| T16 실제 조회 | T05/T07 파일을 server 새 process --load-save, Join·Query world/state·기존 브라우저 패널/정지/속도 조회 | 시간/world/owner/controller/ledger가 로드 candidate에서 나온다. 정의 initial world로 리셋 안 함. loaded paused면 실제 UI에서 해제. upload/save 버튼을 시험했다고 쓰지 않음 |
| T17 3OS/정확HEAD | Linux/Windows/macOS 같은 committed fixture, saved_at=0 및 고정 압축 settings. 각 OS 생성 save 2회·fresh process resume, 정상 CLI normal save 재개도 별도 | 동일 fixture .ohsave 전체 bytes/SHA256, postcard body/canonical SHA/FNV, resumed FNV·queue·원장 rawbit 각각 일치. normal timestamp 파일 SHA 일치는 요구하지 않음. exact git HEAD/exe SHA/pack inventory/명령/exit/PID 필수 |
| T18 증거 실패 자체 | 3OS 중 1개 artifact 누락·다른 HEAD·M0 command 대체·cached output·SHA/hash/command 불일치 | compare nonzero. 전 OS 결과 없이 3OS PASS 표시 불가, 기존 core/M0/M1 반복 hash workflow는 보존 |
| T19 실제 invalid pending | M0 world None와 M1 Some 각각 기존 enqueue에 미래 SetSpeed(0/6)를 넣고 export/save; 검증용 DTO에는 같은 무효 명령을 넣어 import/decode | export/save는 InvalidQueue로 명시 실패, snapshot/hash/config/world/queue·old file SHA 그대로. import도 거부. 기존 executor에서는 해당 tick에 InvalidSpeed 결과를 내고 명령을 소비하는 현행 동작이 유지됨을 별도 대조. save가 명령을 미리 실행/삭제해서 성공하지 않음 |

T02는 T03/T05 예약·expiry가 실행 중 실제 변경되는 fixture와 결합한다. T19의 선택 이유는 임의 파일의 invalid command를 실행 경계까지 지연시키지 않고 import에 유효한 명령 인수만 설치하기 위해서다. 그 결과 현재 live 큐의 **모든 가능한** 순간이 저장 가능하다는 보장은 하지 않는다. invalid pending이 있는 순간은 명시 오류이고 기존 executor 처리 이후 다시 checkpoint할 수 있다. 이 제한을 REQ-SAV-01 검증 리포트에 남겨야 한다. clock/ledger 오류가 나는 기존 step은 queue/state 전체가 유지된다는 선행 계약도 보존한다.

T06b의 base `2Q+1`은 현재 TOML StateDefinition.infrastructure가 i64 정수인 초기 로더에서 만들 수 없다. 독립 fixture DTO에서 raw base와 각 ledger 행을 별도 정수로 명시하여 검증 import로 valid sim을 만들고 export/save를 검사한다. 이것을 실제 gameplay의 인프라 변경 명령이나 UI 행동으로 기록하지 않는다. T02/T05는 실제 로더의 정수 base와 정확 decimal-string modifier로 만들고 스케줄러의 expiry 변경을 검사한다.

canonical 독립 encoder는 mutable 모든 필드를 열거하며 한 필드씩 변조해 bytes/hash 민감도를 확인한다. 독립 검증 전후 exact HEAD·index bytes·tracked 목록·각 SHA·diff/status가 같아야 한다. fixture 생성/검증 stdout·stderr·exit가 없는 사례는 계획으로 남긴다. 검증자는 추적 파일을 수정하지 않고 ignored target 아래 증거만 만든다.
