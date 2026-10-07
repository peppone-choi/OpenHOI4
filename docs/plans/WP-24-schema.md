# WP-24 팩 해석·검증 스키마

## P-06 P05-3 등록 nation·query 원장 정합 기획 (2026-10-07, 구현 전)

base3d56361의 P05-3 actual 최종은 유효 FAIL2이며 원9878/HEAD/raw96cf4bdd/ZIP3221·manifest3220 불변이다. 원 verify3 worktree/리포트/mutant/index를 읽기만 한다. 이번 소유는 pack_validation의 등록 nation 분기와 additive data/native 회귀, current checker/tests 및 own 문서다. 다른 loader/host policy/codec/원 checker/helper/fixture/query script/workflow는 보호한다.

| 입력·관계 | 기존 계약/실제 검사 | 실패 및 지원 경계 |
|---|---|---|
| scenarios/<id>/nations/*.toml | 등록 NationDefinition deny_unknown_fields의 syntax/type/u16/RGB/필수필드 및 기존 tag/name/government/ideology Fx 범위·합 검사 | map 있는 선택된 국가는 기존 read_scenario의 map/capital/ownership/control 참조 검사. map 없는 empty legacy는 선택 nation/map 관계가 없어 국가 world를 지원하지 않으며 유효 extra조차 unselected/context 오류. malformed를 unsupported 경고로 숨기지 않음 |
| malformed/type/unknown/국가tag·이념 오류 또는 미선택·map 참조 없는 입력 | TOML 원 path/Unicode line/column/원인 | validate/--pack headless run/save-out, server startup/restore/force의 full guard 모두 오류, 새save 미생성·기존save/source 불변. 정상 legacy date/defines와 정상 structured M1은 유지 |
| query world.tick·state infrastructure.tick | 실제 paused source DTO tick/header와 각 ledger.tick이 같아야 함 | raw query와 receipt를 같이 변조해도 독립 source 기준으로 거부 |
| query ledger count/source/op/value/accumulated/base/final | source DTO의 독립 integer verify_ledger로 재계산한 ordered rows, 각 state ID/owner/population/resource/building/province 관계 | missing/type/null/duplicate/time/count/source/final·accumulator 오류를 모두 거부; raw==receipt 일치만으로 성공 판정하지 않음 |
| Fx wire 표시 | 기존 I32F32 raw bits의 decimal String 표시 | 정수/Fraction으로 최근접 even quantization해 원 bits와 대조, finite·범위·정규 decimal 형 확인. Float/임의 게임수치 추가 없음 |
| 원 save_query.cjs selectors/asserts | Welcome accepted, unsupported-create Notice, paused 첫 Snapshot, saved-world supported/시간/2국가6province·첫ledger tick/count/a.raw/final/lastaccumulator, CommandResult sequence1 accepted, speed2 paused/tick48 Snapshot | 각 assertion을 checker에서 실제 다시 검사하고 messages의 원 selector가 선택한 응답과 output 필드 관계도 대조. source config/queue/state/ledger/canonical은 current evidence에서 검증하고 wire time/ledger와 연결 |

원/current capture·원8/current17 및 새동시변조 회귀를 유지한다. 새 증거 경로는 target/evidence/WP-24-P06-current-comparer/뿐이다. 로컬 synthetic 비교는 실제3OS로 판정하지 않는다. 원 current/head/dirty/rawindex/executable/PID/nativeexit/stdout/pack/header/save 검사는 그대로 필수이며 새로운 clean commit에서 actual native normal/force·historical rejection과 SHA 봉인을 재실행한다.

부모 추가 소유 인계로 national.rs의 private document/내재 검사를 pub(crate) read_nation/validate_nation_names/validate_nation_ideology 단계로만 추출했다. selected 경로의 syntax/type/tag→duplicateID→name/government→capital→ideology 순서를 유지하고 복수 오류 회귀6행으로 검사한다. map 없는 legacy와 unselected structured extra는 같은 공통 단계를 검사한 뒤 capital 원 위치에 명시 selection/context 오류를 반환한다. reference를 검사한 것처럼 성공 처리하거나 임의 map을 선택하지 않는다.

| 원 save_query.cjs 행/selector | comparer의 필수 대응 |
|---|---|
| 23 Welcome accepted / 25 unsupported-create | type·accepted 및 header pack/engine, messages 첫 Welcome·첫 Notice와 key 대응 |
| 28 paused Snapshot deepEqual | verified paused DTO의 date/hour/tick/paused/speed와 엄격한 type/field 집합, messages 첫 Snapshot |
| 30 saved-world WorldResult supported | type/request/supported 및 첫 matching WorldResult, world.tick을 source와 대조 |
| 31 world tick·국가2/province6 | source ordered nation/state/province IDs 및 mutable owner/controller/population/resources/buildings/support도 대조, 누락/중복/type/null 거부 |
| 33 ledger tick/count4 | 각 state의 독립 integer ledger.tick/ordered rows 개수와 wire 대조 |
| 34 source a.raw / final !=3 | source/label/op/value·accumulated의 원 DTO rawbit와 Decimal/Fraction 재양자화, 원 Node의 추가 assertion도 유지 |
| 35 final==last accumulator | wire 원문 문자열도 서로 같아야 함, source final/infrastructure/last row rawbits와 각각 대조 |
| 37 CommandResult sequence1 accepted | type/sequence/accepted 및 첫 matching command 메시지 |
| 39 speed2/tick48/paused state | 원 SetSpeed2 이후 date/hour/tick/paused를 source와 대조, matching speed2 Snapshot과 output 객체 일치 |

행 번호는 base3d의 변경하지 않은 원 script에서 직접 읽어 대조했다. raw query와 receipt는 duplicate JSON field를 거부하고 type-strict 비교한 뒤 독립 source 기준을 검사한다. `True == 1` 같은 Python equality로 wire type 오류를 허용하지 않는다. 메시지의 비선택 추가 Snapshot은 원 Node selector와 동일하게 허용하며 원장 row의 source/id/op/order/count는 source 기준으로 엄격히 검사한다. fixed1.31.0의 decimal [from_str](https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html#method.from_str) 최근접 even 의미를 2026-10-07 공식 문서에서 확인했다. 새로운 의존성과 게임 수치는 없다.

## P-06 필수 Save CI producer/consumer 보완 기획 (2026-10-07, 구현 전)

인수 base는 c28f90425905774c23b1d90a5d900c596ade2745다. 4aecc22 clean에서 ff-only 인수했다. 실제 WP17 attempt2는 필수 Save CI의 frozen→server 연결 FAIL과 raw index FAIL로 무효이며 원 baseline/index/증거를 복구하지 않는다. REQUEST-0008 A caller와 원 codec/fixture/expected는 유지한다.

| 경로/producer | consumer·독립 기대 | 증거 계약·실패 |
|---|---|---|
| 기존 check_save_determinism.py capture → original 디렉터리, frozen mutable-v1 두 회 | 기존 compare/3OS와 원 CLI resume | 기존 메서드·test_save_evidence·원 SHA57b13/expected/state/canonical/ledger/해시 단언 불변 |
| 새 check_save_current.py capture → 별도 current 디렉터리, save_fixture capture-current 두 회 | fresh helper resume/CLI resume→전체 상태·canonical/ledger/hash 검증 | original/current mode를 혼용하지 않음; helper capture.json의 기존 fixture 라벨은 상태 시험 이름이며 wrapper evidence_mode=current-m1-v1로 입력 구분 |
| current run-1/pack + paused.ohsave | save_native.py positive, force 양쪽 | 기존 exclusive PID/HTTP200/WS Welcome/query/served JS/정상 Ctrl+C0 단언 전부 유지; 전체 source/save 불변, 입력은 current marker/header/pack 신원으로 연결 |
| original run-1/pack + paused.ohsave | 새 save_historical_native.py negative, force 양쪽 | 실제 server_restore native1/무HTTP·WS/종료 signal 없음, ko/en 원 이름 누락 진단; 원 CLI resume는 정상 |
| 각 OS current artifact/current-save-{ubuntu,windows,macos}-latest | 새 comparer | 같은 exact clean HEAD/platform/mode/schema/pack path·bytes·SHA256·FNV/save·paused SHA/header/defines/scenario/seed/tick/config/queue/전체 DTO/canonical/원장/hash·반복bytes와 새 native PID/command/cwd/exit/executable SHA/stdout/stderr 비교. OS 누락/혼합HEAD/dirty/같은PID/증거누락·변조·wrongmode 거부; server/force result와 원 query·HTTP·nativeexit·paused 입력까지 필수 |

current evidence schema는 engine/test 도구 파생값이며 save/wire/시뮬레이션에 넣지 않는다. 헤더는 기존 bounded OHSV/format1/length/Postcard 필드 순서를 Python 정수로 읽어 native report의 pack/scenario/date/tick/seed/state_hash/definitions_hash/effective_defines_hash와 대조한다. paused 입력은 split의 due48 명령 실행 후 paused=true/speed5/tick48/잔여 tick49 queue를 독립 canonical reference로 확인한다. snapshot full DTO와 current source의 실제 numeric effective defines를 비교한다. 현재 입력은 integer-only이므로 tagged scalar/array/i64 Postcard/FNV reference로 effective defines hash도 계산한다. 미래 Fx define 입력은 이 reference를 먼저 확장해야 하며 float 임의 변환으로 숨기지 않는다. capture 후 before/after 파일 path·bytes·SHA와 saved/paused bytes를 재검사하고 source HEAD/status/semantic index hash/raw index hash 경계도 기록한다. 모든 native 실행은 raw stdout/stderr와 자체 process PID/exit/cwd/argv/executable SHA를 남긴다.

세 OS의 canonical/state/pack/source/save/paused/defines는 같아야 하고 PID/executable SHA/절대 cwd는 OS별 증거라 서로 같을 필요가 없다. synthetic comparer negative tests는 predicate 검증이며 실제3OS 실행 증거가 아니다. workflow는 original capture/compare와 기존 검사를 보존하고 current capture/compare·positive/historical negative를 추가해 양쪽 성공을 필수화한다. deps/ASSETS/다른 CI/codec/helper/client 생성물을 수정하지 않는다. TS 동등성은 기존 contract의 read-only 검사만 실행한다.

## P-06 실제 로드 입구 보완 기획 (2026-10-07, 코드 수정 전)

권위 첫 FAIL은 `E:/openhoi/docs/verify/WP-24.M2-r1.attempt1.md`의 앱 마지막 전문이다. 더 긴 target fullreport는 별도 증거이고 동일 문서로 취급하지 않는다. 원 c2c2/원 evidence는 보존한다.

| 입력/필드 | 실제 계약·범위·기본값·참조/권위 | 검사/적용/실패 |
|---|---|---|
| legacy scenario defines | 선택 `<pack>/scenarios/<id>/defines.toml`, null/문법/비숫자/nested array/overflow 거부, 누락이면 root defines 유지 | root→scenario system.item overlay, m0 loaded.pack.defines 및 TimeConfig/SaveContext에 같은 실제 값을 전달, 단순 registered 표시 금지 |
| runnable time.speed_ms_per_tick | i64 숫자5개 배열, 각0..i64::MAX, fixed/다른형/길이 거부 | scenario가 있으면 effective defines에 필수; speed5개의 기존 runtime 계약, 위치는 원 root 또는 override |
| runnable time.initial_speed | i64 scalar1..5, default 없음 | effective defines에 필수; 실제 legacy run 값으로 검증 |
| registered time/network keys | time 위2키; 기존 network5키 scalar 양수, delta_ms>=100 | 오타/임의 registered key 거부; network 전체 필수성은 server Host의 기존 positive 계약 유지; 다른 미래 시스템 generic numeric defines는 기존 계약 유지 |
| active selected pack | Path→resolve/registered/schema/ref/FTL→snapshot identity | Host::load/load_with_save는 validation 통과 후에만 World/authority/HTTP/WS 생성, force도 validation 면제 아님; validation 전후/hash→typed load identity 일치 |
| loader graph | CLI validate/run/load_national/save-out; server load/load_with_save; low-level m0/national readers; SaveContext | 같은 원인 입구 목록과 실제 지원 계약을 기록; codec/frozen v1와 active server의 현지화 요구는 구분 |

서버 current팩은 full strict validation. 원M0는 승인된 server_startup/server_restore entry만 적용하며 immutable mutable-v1은 CLI v1 resume만 적용한다. plain frozen19와 WP17 movement-pack-97f19의 codec/load_context용 원천은 호환정책 대상이 아니다. 원 codec reader/writer bytes/state/hash와 v1/v2/v3 fixture/expected/helper를 보존한다. 새게임수치/선택팩UI/merge/patch/저장format/이동상태는 변경하지 않는다.

Strict/Active/RestoreV1 일반 목적은 모두 strict이며 명시 caller3종만 정책을 적용한다. CLI resume actual 입구는 bounded header→full validation→SaveContext→전체 identity compare→decode 순서다. 저수준 national/m0 및 codec/load_context용 정의 reader는 resolve/schema/ref/effective time/network 계약과 모든 present FTL syntax/parity/static-ref를 검사하지만 사용하지 않는 manifest display-name의 필수 번역은 full active validator 소유다. validate 내부는 private typed read_scenario를 사용해 모든 실제 키의 원 file context를 모은 다음 FTL을 한 번에 검사한다. public low-level reader를 우회하는 active host는 없다.

독립 expected: invalid legacy syntax는 defines 원file/line/column로 CLI exit1; 정상root time+optional speed override는 loaded TimeConfig에서 해당 실제 speed값; override 누락은 root값, 필수 effective key 누락/0초과길이/음수/fixed/unknown time key/nested array/invalid nation/ref/FTL/dependency/version/conflict는 오류. 정상 current server startup HTTP200/WS101와 native Ctrl+C exit0; invalid pack의 startup/restore는 자체 native exit1이며 HTTP/WS없음, save/load source bytes와 이미 만든 정상 authority가 보존되어야 한다. 원 v1 codec은 frozen context로 재개동일, current oldsave는 기존 PackMismatch 유지.

### REQUEST-0008 A 제한 잠정 채택 정책 데이터 schema

저장 경로 `crates/oh_data/policies/legacy-validation.toml`, JSON schema는 `crates/oh_data/schema/legacy-validation.schema.json`이다. engine 소유 include_str 정책이고 입력팩/CLI flag/WS/client가 지정하지 못한다. CEO DIRECTIVES 2026-10-07T10:57:22의 REQUEST-0008 A 원 SHA2477248184afc83d1ae5102a4ed12f11c4957c939d13b0c391724216b4b30ddc 한정 잠정 채택 후 구현했다. 이전 승인대기/후보 제거의 경과는 worklog에 보존한다. gameplay defines 수치가 아니며 identity literal은 정책 데이터에 둔다.

| 필드 | 형/필수/범위·기본값 | 의미/검사·수명/권위 |
|---|---|---|
| policy_format | u16, 필수,1만 지원/null·default 없음 | engine 정책 format; 원 save format을 변경하지 않음 |
| entries[].policy_id | String stable unique, 필수 | 기술 인수 추적 ID, engine 소유 |
| sources[].id/source_commit/source_reference | unique ID/40lowerhex String/봉인 원천 reference, 필수 | b6516ed 원M0 또는 c2c2 원mutable-v1 provenance; runtime source 다운로드 없음 |
| entries[].source_id | String, 필수, sources의 존재 ID | immutable 원파일 신원 목록 참조 |
| pack_id/pack_version | regex ID/SemVer String, 필수 | exact id/version 비교, 범위req 아님 |
| pack_content_hash | 16hex String/u64 parse, 필수 | 기존 sorted-path/raw-bytes postcard/FNV identity, 다른 byte 면제 없음 |
| sources[].files[{path,bytes,sha256}] | exact source3/19 파일의 relative POSIX path/u64 length/64lowerhex SHA256, 필수 | sorted unique path, dot/dot-dot/absolute/backslash/colon 거부; runtime 전수byte SHA/length/경로·목록·개수 일치 필요, extra/missing/link/reparse 거부 |
| scenario_id/scenario_kind | String ID + enum empty/national, 필수 | server originalM0=testland/empty, CLI originalv1=m1/national, M0/NAT 혼동 거부 |
| caller | enum server_startup/server_restore/cli_v1_resume, 필수 | Strict/Active/RestoreV1 일반 목적 면제 없음; cli_m0_run entry 없음, force는 입력조건 아님 |
| save_format | optional u16, server_restore/cli_v1_resume에서는 required1, server_startup에서는 absent | 실제 bounded header의 format/scenario/world 유무/단일 pack id·version·hash와 검증한 context를 결합; 미래 format/없는 header 거부 |
| source_field/name_key | literals manifest.name_key/testland_name, 필수 | display metadata 하나만 제외, 다른 name/government/ideology 키 면제 없음 |
| diagnostic_code/locales | enum missing_message_value + exact set[ko,en], 필수 | 메시지 문자열 매칭 대신 구조화 진단 code/키/locale/원field를 사용; duplicate/missinglangs 거부 |
| disposition | enum warning_unavailable, 필수 | 원 file/line/column/원code·field·key·locale·cause 및 unavailable 보존, applied_policy ID 기록, fake번역 없음 |
| ResolvedPack.source_files/PackFingerprint | sorted Vec<FileFingerprint> + 기존 content_hash | resolve snapshot→content 검증 후 재대조→typed load/SaveContext 후 재대조; 바뀌면 부분 pack/authority 반환 없음, save/wire/게임상태 제외 |

엔트리는 caller·saveformat·scenario·id·version·전체원파일신원·필드·키·진단code·언어가 전부 일치할 때만 적용한다. 구조화 missing_message_value는 localisation 사용키 검사에서만 생성하고 syntax/parity/staticref 등은 다른 code다. 최종 report는 부분허용 후에도 다른 모든 오류를 그대로 보존하며 source fingerprint와 typed context identity를 재검사한다. 원M0 standalone validate는 exit1이며 일반 legacy/새fixture/다른context/force/1byte변경에 면제를 주지 않는다. 원M0 caller3와 mutable-v1 CLI caller 하나로 네 entries만 있다. 테스트는 SHA·length·path·diagnostic code/field/key/locale 조건변조, 실제 byte추가/삭제/FTL/dependency변조, 잘못된 header/purpose, unrelatederror 보존, 검증 중 source-change를 검사한다.

부모 추가 인수: M0 headless CLI/library는 기존 렌더 없는 empty-definition 계약을 유지하여 low-level resolve/schema/effectiveTime/presentFTL syntax/parity/staticref를 검사한다. 새 cli_m0_run 면제 entry는 없다. current national active/run는 full 검증을 유지한다. 부모 원manifest 경로는 `.orchestrator/evidence/REQUEST-0008-identities.json`, 문서 원bytes 봉인경로는 `docs/decisions/evidence/historical-load-policy/`다. FNV만이 아닌 원파일 전수 SHA256/길이/경로집합을 대조하며 input팩/flag/client가 정책을 선택하지 못한다.

2026-10-07, M2-r1. 코드 변경 전 설계. REQ-MOD-01/02/04, REQ-LOC-03. 게임 공식·패치·모드 UI는 추가하지 않는다.

## 필드와 권위

| 필드 | 의미/타입/단위/범위 | 필수/null/default | 참조/수명/변경권한 | 분류/REQ |
|---|---|---|---|---|
| Manifest.id | String `[a-z0-9_]+`, 팩 안정 ID | 필수, null/빈값 거부 | 선택팩 내 unique, host read-only | 정의/MOD-01 |
| name_key | nonempty String, Fluent message ID | 필수, null 거부 | ko/en 메시지 value 필요; manifest 원 위치 | 정의/MOD-01/LOC-03 |
| version | String, SemVer Version | 필수/null 거부 | dependency version의 기준, build metadata 표준 의미 | 정의/MOD-01 |
| engine | String, SemVer VersionReq | 필수/null 거부 | 현재 엔진 crate version과 matches | 정의/MOD-01 |
| depends | Vec<{id,version}>, 기술적인 무단위 관계 | 미지정 [] 기존값; null 거부 | 존재·버전·unique, self/순환 거부 | 정의/MOD-01 |
| conflicts | Vec<pack id> | 미지정 []/null 거부 | 한쪽 선언만 있어도 선택 집합 거부; 미선택 ID 허용; 중복/self 거부 | 정의/MOD-01 |
| load_after | Vec<pack id> | 미지정 []/null 거부 | 대상 선택팩 존재 필요, unique/self/순환 거부 | 정의/MOD-01 |
| ResolvedPack.root/data/content_hash | PathBuf/DataPack/u64 | 성공 시에만 생성 | 파일 상대 UTF-8 경로·bytes sorted postcard FNV-1a; link/reparse 거부, save 기존 hash와 동일 | 파생/MOD-02 |
| ordered packs | Vec<ResolvedPack> | 실패 시 반환 없음 | Kahn DAG, 매 단계 준비된 ID lexical 최소, input/root 경로 순서 무관 | 파생/MOD-01/02 |
| diagnostic | DataError path/1-based Unicode line/column + severity(error/warning) | 실제 원 source, IO/root missing는 1:1 | host 결과만; map warning province 문맥 유지, 오류는 원 필드 offset | 파생/MOD-04 |
| validation maps/scenarios | 기존 map/national 타입과 M0 start_date | 등록된 파일은 모두 검사 | map→terrain/resource/building; state→province; national ownership/control/capital/ideology/modifier | 정의·초기/MOD-04 |
| Fluent entry | message/term ID, value/attributes, AST reference | syntax/duplicate/missing message·term·attribute/cycle 거부 | locale별 파일 집합, ko/en value/attribute parity; variables 번역별 자유, terms locale 자유 | 정의/LOC-03 |
| localisation usage | manifest.name_key, states/VP/nation/government/ideology keys | 양 언어 value 있는 message 요구 | TOML 원 필드 위치, FTL 정적 참조; unused messages는 warning | 파생/LOC-03 |

가변 시뮬레이션 권위 상태는 새로 만들지 않는다. defines는 기존 i64/Fx와 누락 기본값 없음 계약 그대로다. UI/client catalogs는 별도 checker 입력이며 client 변경 없음.

## 관계·실패 경계

`selected Pack[N] → manifest DAG → lexical load order → each pack manifest/defines + registered common/map/scenario → typed references → locale AST → validated host report`.
강한 dependency와 load_after는 같은 순서 제약 그래프다. 동일 간선의 dependency+load_after 중복은 허용하며 각각의 목록 내부 중복은 오류다. diamond는 허용한다. 지도 adjacency cycle은 허용한다. FTL 정적 message/term/attribute 참조 cycle은 거부한다.

팩 선택→파일 목록/bytes snapshot→문법→engine/dependency/conflict/order→등록 콘텐츠→현지화→identity 순서로 처리한다. 오류 시 일부 pack load 성공 객체나 시뮬레이션 상태를 반환하지 않는다. report는 오류/경고를 보존하며 --deny-warnings가 오류 exit1/경고 exit1/정상 exit0을 결정한다. 입력 파일을 쓰지 않는다. 읽기 전후 identity 불일치는 오류로 반환한다. 실제 runtime 초기 World의 경제/보정 적용은 oh_sim 소유이고 WP-24는 이미 등록된 데이터 계약만 검사한다.

## 버전·hash/save/wire/UI

manifest schema와 DataPack shape는 유지한다. 새로운 resolve/validate host API만 추가한다. content_hash는 기존 oh_save::context::pack_hash와 동일한 정렬 상대파일 bytes 정규 직렬화이며 save reader/writer/FORMAT_VERSION=1을 변경하지 않는다. path/root 자체는 hash에 포함하지 않는다. 새 validation DTO는 hash/save/wire/UI에 포함하지 않는다. CLI가 ordered identity를 출력한다. ko/en 누락 검사 CI는 CLI validate 및 별도 catalog checker 실행 경로를 사용한다. M0 root와 기존 M1 내용은 수정하지 않는다; M0 validate는 start_date 전용 legacy adapter로 검사하고 이름 현지화 누락은 숨기지 않는다.

WP-13 trigger/effect/end registry, WP-14 정치/경제, WP-15 장비는 아직 미구현 adapter다. 해당 미래 경로를 현재 validate가 완전히 검사한다고 기록하지 않는다. 새로운 등록되지 않은 콘텐츠는 unsupported 경고로 노출하며 unknown fields는 등록 schema에서 오류다. producer는 validation entry point에 해당 typed loader/키·참조·진단을 연결해야 한다. pack merge/patch/모드 UI는 WP-35다.

## 독립 입력·기대

| 입력 | 독립 기대 |
|---|---|
| a→base, b→base, top→a+b, 입력 top/b/base/a | base,a,b,top; 모든 permutation identity 동일 |
| 독립 z/a/b + a load_after z | b,z,a (ready-set lexical) |
| depends missing/버전 <0.1/engine >=0.2/duplicate id/한쪽 conflict | manifest 해당 값 위치, error, 반환 없음 |
| a→b→a; a load_after b + b depends a | 순환 error, 원 manifest 관계 위치 |
| load_after missing; duplicate dependency/conflict/order/self | 오류, 선택되지 않은 conflict만 허용 |
| shipped testland | 실제 map 1, scenario m1 1, typed 국가/주/프로빈스 로드; 부족한 manifest key는 정확 error |
| copied testland terrain unknown/capital missing/ownership missing/unknown scenario key | 실제 typed loader 오류; Unicode/CRLF 위치 유지 |
| FTL ko only key/en only key/empty value/duplicate across files/invalid syntax | 오류 및 FTL/TOML 원 문맥 |
| FTL select multiline+term+attribute references | 유효; 존재하지 않은 attr/term/cycle 오류 |
| localized unused message 또는 map disconnected warning | 기본 exit0, --deny-warnings exit1, 원 파일 위치 보존 |
| same pack copied root / one byte changed | identity 동일 / identity 변경; 기존 save hash API와 일치 |
| 기존 M0 365일/M1 365일 2회, 기존 저장 tests | 각각 같은 hash, 동결 golden/fixture 유지 |

## 조사·미완료 대조

구현 후 인수 경계(아래는 독립 PASS 선언이 아님): `resolve_packs`, `validate_packs`, `content_hash`, `localisation::validate_catalogs`가 공개 host API다. 기존 `load_pack`은 M0 metadata/defines 저수준 reader로 유지하며 기존 national/M0/save 호출의 호환을 깨지 않는다. 여러 선택 팩의 resolved order를 gameplay merge에 연결하는 부분과 서버/모드관리 UI는 WP-35 후속이다. validate의 registered adapter는 common ID 목록, map geometry/state/VP/palette, M1 national scenario/nations/modifiers 및 M0 start_date이고, 미래 등록 파일은 unsupported warning으로 표시한다.

CLI `load_national`/`run --pack` 및 `run --pack --save-out`은 full host validation을 먼저 통과한 다음 World/SaveContext를 생성한다. 잘못된 dependency/FTL이면 sim 생성·저장 파일 작성 전에 거부한다. 서버 Host::load/load_with_save와 CLI resume도 full host guard를 통과한다. M0 headless reader 및 codec/load_context와 active host의 이름 현지화 경계는 위 P-06 계약을 따른다. multi-pack gameplay merge/선택관리 UI는 해당 모드 WP 후속이다.

| 추가 기술 필드/입력 | 계약/권위/수명 | hash/save/REQ |
|---|---|---|
| client KeyUse key/path/line/column | Python source 정적 t literal 필수, union catalog에 있는 일반 literal은 보수적 사용 표시; generated resource-/building-/terrain 키는 현재 common registry 기준, target uses.json만 작성 | 진단 파생, hash/save 제외, LOC-03 |
| frozen source_commit/path/files[{path,bytes,sha256}] | b6516ed 원19파일 불변; fixture 밖 JSON, source bytes audit; v1 capture만 사용하며 live generic 경로 유지 | 원 v1 pack identity와 saved.ohsave bytes 유지, MOD-02 호환 |
| capture-current | 현재팩의 같은 v1 상태/저장·재개 검사를 독립 출력으로 실행, committed old fixture 비교와 구분 | 새로운 current pack identity, 기존 format1 그대로 |

실제 legacy validate는 이름키 ko/en 누락 오류를 내며 M0 run golden은 유지한다. base 폴더가 현재 없어 실제 validate IO 오류다. 등록 adapter 전체를 미래 MOD-04 전체 완료나 base 콘텐츠 존재로 대신 표시하지 않는다. client computed 키의 모든 가능한 runtime 값을 정적 scanner가 증명하지는 않으며 parity/registered DTO 검사와 실제 runtime/UI 후속 범위를 구분한다.

원작 자료를 사용하지 않았다. schema-source-review의 공식 위키 열람불가를 유지한다. 기술 근거(2026-10-07 확인): [semver 1.0.28 VersionReq](https://docs.rs/semver/latest/semver/struct.VersionReq.html), [Fluent syntax 0.12.0](https://docs.rs/fluent-syntax/latest/fluent_syntax/), [MIT OR Apache-2.0](https://docs.rs/crate/fluent-syntax/latest). 신규 dependency 및 콘텐츠 소유는 부모에 요청했다. 이 문서는 구현/독립 검증 PASS나 전체 MOD-04 완료 증거가 아니다.
