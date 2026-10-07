# ADR-2401 팩 선택 순서·Fluent·고정 v1 입력

## P-06 P05-3 후 등록 경로·query 원장 인수 (2026-10-07, 구현 전)

actual P05-3의 유효 FAIL2를 보존한다. map 없는 empty legacy가 nation 파일을 읽지 않지만 registered로 분류하던 간극은 기존 NationDefinition TOML/type/내재 제약 검사 후 unselected/map-context 오류로 거부한다. legacy world/국가 게임규칙·임의 map 선택·caller 예외는 추가하지 않는다. structured 선택 nation 참조는 기존 read_scenario가 검사하며 valid extra도 지원한 것처럼 반환하지 않는다. 오류는 원 source 위치/원인을 보존한다.

current comparer는 raw query==receipt 외에 검증한 paused source DTO와 독립 integer ledger reference를 기준으로 tick/count/source/op/value/accumulated/final 및 전체 state ID 관계를 검사한다. 기존 save_query.cjs의 모든 selectors/asserts를 문서에 대응하고 직접 다시 검사하며 malformed/동시변조도 거부한다. wire Fx decimal은 정수/Fraction 최근접 even rawbit 대조로 확인해 float 계산이나 새 수치를 도입하지 않는다. 원 정책/loader/save codec/fixture/expected/golden/원 query script/workflow는 바꾸지 않는다.

부모가 national.rs의 private document/내재 검사 추출만 추가 인계했다. pub(crate) 공통 read_nation/validate_nation_names/validate_nation_ideology를 재사용해 타입/범위 의미가 두 validator에서 갈라지지 않게 하며 selected 오류 순서 tag→duplicateID→name/government→capital→ideology를 유지한다. 기존 world/map/선택/FTL/public DTO/API는 동일하다. 복수 오류 회귀6행과 기존 workspace 검사를 실행한다. [fixed1.31.0 decimal parser](https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html#method.from_str)의 최근접 even 의미는 2026-10-07 공식 원문에서 확인했다.

## P-06 필수 저장 CI 입력 연결 결정 (2026-10-07, 구현 전)

CEO DIRECTIVES 2026-10-07T12:21:27과 WP17 attempt2 actual FAIL에 따라 frozen capture를 서버 positive에 전달하던 CI 연결을 보완한다. REQUEST-0008의 역사 mutable-v1 CLI-only 계약과 원 save codec/fixture/expected/비교/단언은 바꾸지 않는다. 기존 check_save_determinism.py capture/compare는 원 frozen 3OS producer/consumer로 그대로 두고, 새 check_save_current.py가 이미 존재하는 capture-current를 별도 출력에 두 회 생산하고 fresh native/CLI 재개·전체 DTO/원장/canonical/hash/팩·header·save/source 불변성과 같은 clean HEAD의3OS를 비교한다. save_native.py의 기존 HTTP/WS/query/served JS/exclusive PID/Ctrl+C0 검사는 current paused 저장으로 실행하며 역사 서버거부는 별도 native negative 도구로 force 양쪽 확인한다. 원/current artifact names·mode/schema를 구별하고 필수 workflow에서 original/current 비교가 모두 성공해야 한다. synthetic OS label 시험은 실제3OS 성공으로 기록하지 않는다. raw index FAIL은 원인 미확정 무효이며 baseline/index를 복구하지 않는다.

## P-06 정정 기획

REQUEST-0008 A는 CEO DIRECTIVES 2026-10-07T10:57:22의 원 SHA2477248184afc83d1ae5102a4ed12f11c4957c939d13b0c391724216b4b30ddc 한정 잠정 채택을 적용한다. 앞서 임시 FNV-only 후보를 제거한 기록은 유지한다. 최종 구현은 engine-owned 정책 TOML/전체원파일 path·bytes·SHA256와 typed 진단, 명시 caller3종 및 실제 bounded v1 header→pack/scenario/context 관계를 함께 검사한다. 형/맥락/force로 범위를 넓히지 않는다. SHA256은 공식 [sha2 0.11.0 manifest](https://docs.rs/crate/sha2/latest/source/Cargo.toml.orig)의 MIT OR Apache-2.0/MSRV1.85와 [Digest/Sha256 API](https://docs.rs/sha2/latest/sha2/index.html)를 2026-10-07 확인했고 부모의 oh_data Cargo/lock 소유 인계로 default-features=false를 추가한다. backend/settings/workspaceCargo는 바꾸지 않는다. 원3/19 source metadata는 부모 봉인 ZIP/identity와 runtime에서 전수대조하며 정책은 팩 밖에 두어 원pack/save identity를 오염시키지 않는다.

첫 독립 검증 actual final은 registered legacy defines를 검사하지 않는 분기와 server의 직접 national reader 호출을 재현했다. legacy override를 root→scenario 실제 numeric overlay로 연결하고 runnable time/등록network 타입·범위·원 source 진단을 검사한다. Host의 load/load_with_save와 CLI resume에 full validation, typed load 이후 전체 파일 fingerprint 재대조를 연결한다. 기존 M0/frozen codec과 active localised팩의 계약은 WP-24-schema의 P-06 표로 분리한다.

임시 FNV-only 후보는 승인 대기 지시로 제거했고 그 당시 green을 최종 증거로 사용하지 않는다. 승인 후 정책은 server_startup 원M0, server_restore 원M0+실제 bounded v1 header, cli_v1_resume 원M0 또는 immutable mutable-v1+실제 bounded v1 header 세 caller에만 연결한다. 각 entry의 전체 3/19파일 경로·길이·SHA256, FNV64, pack id/version, scenario id/kind, header format/scenario/world유무/단일 pack 목록이 모두 맞아야 한다. 두 typed MissingMessageValue 진단의 manifest.name_key/testland_name/ko·en만 warning_unavailable로 바꾸며 원 code/field/key/locale/cause/path/line/column을 보존한다. 다른 오류는 유지한다. Strict/Active/RestoreV1 일반 목적에는 entry가 없고 --force는 정책을 선택하지 않는다. standalone 원M0 validate는 여전히 exit1이다. plain frozen19 identity `13318001328374612931` 및 WP17 movement-pack-97f19는 예외 대상이 아니다.

M0 headless CLI/library는 renderer 없는 empty-definition 계약을 유지한다. 저수준 m0/national 및 SaveContext/load_context용 정의 reader는 resolve/schema/ref/effective defines와 모든 present FTL syntax/parity/static reference를 검사하고, 사용하지 않는 manifest UI 이름의 필수성은 full host guard가 검사한다. cli_m0_run entry는 없다. 원 v1/v2/v3 codec, header/body/preflight, fixture/expected/helper는 변경하지 않는다. 새로운 validation DTO/정책은 save/wire/pack content identity에 포함하지 않는다.

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-24, REQ-MOD-01/02/04, REQ-LOC-03 |

## 맥락

02 §5는 SemVer 의존/충돌, 순환 거부, 파일·줄 오류와 Fluent를 정하지만 독립 팩 동률과 load_after 미선택 대상, Fluent 정적 참조 검사 구현을 정하지 않았다. 현재 manifest/defines와 map/national typed loaders를 유지하면서 새로운 host validation 경계를 추가한다. 기존 v1 저장 테스트는 live testland를 복사하여 생성한 저장 bytes를 committed fixture와 비교한다. 현지화 name_key 추가는 content hash를 바꾸므로 기존 fixture를 수정하면 호환 증거를 잃는다.

## 결정

`resolve_packs`는 선택 집합의 engine/dependency VersionReq를 실제 Version에 적용한다. 중복 ID/목록/self/missing dependency/missing load_after를 오류로 거부한다. conflict는 한쪽 선언만으로 거부하고 미선택 conflict는 허용한다. dependency+load_after 그래프에서 준비된 최소 lexical ID를 반복 선택한다. 같은 간선의 두 목록 중복은 허용한다. 실패는 부분 로드 객체를 반환하지 않는다. `validate_packs`는 정렬된 실제 팩의 기존 map/common/national 타입·키·참조를 검사한다. 아직 등록되지 않은 콘텐츠 파일은 unsupported warning이고 deny-warnings에서 실패한다. 미래 WP의 registry를 성공 adapter로 가장하지 않는다.

Fluent 정식 AST parser `fluent-syntax =0.12.0`을 사용한다. 문법/중복 message·term·attribute, static message·term·attribute 미존재/순환, ko/en message value·attribute parity와 data name/government/ideology 사용을 검사한다. 기본 함수 NUMBER/DATETIME만 지원한다. 변수는 locale별 표현에 맡긴다. unused message는 warning이다. client checker는 literal t 호출을 필수 참조로 수집하고 code literal을 보수적 사용 증거로 센다. 동적 client 키는 catalog parity, 실제 data DTO 키는 pack registry 기준을 함께 검사한다. 임의 computed key 집합을 전부 정적으로 증명한 것으로 기록하지 않는다.

팩 identity는 기존 oh_save의 정렬된 `(relative POSIX UTF-8 path, raw bytes)` postcard/FNV와 동일하게 계산한다. link/reparse를 거부한다. 읽기 전후 hash가 달라지면 검증 결과를 거부한다. save format/reader/writer와 M0/M1 canonical 상태는 변경하지 않는다.

부모의 명시 파일 소유 인계로 b6516ed4e571a0359e1da2ee69e9b68e3356339f의 원19파일을 `oh_save/tests/fixtures/m1-pack-v1`에 그대로 보존한다. `m1-pack-v1.source.json`은 원 커밋/경로/각 SHA256/크기를 기록하며 pack 밖에 둬 identity를 오염시키지 않는다. 기존 generic copy_pack/mutable_pack은 live pack을 계속 검사한다. 별도 copy_v1_pack/mutable_v1_pack만 committed save_fixture capture에 연결한다. capture-current는 새 팩으로 같은 capture/재개 상태 검사를 제공한다. check_save_determinism.py, committed save bytes, expected JSON, assertion을 바꾸지 않는다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 입력 순서 동률 | 간단 | host 인자 순서가 load order/identity를 바꿈 |
| 정규식 FTL 파서 | 의존성 없음 | multiline/select/term/attribute 오류와 참조를 증명하지 못함 |
| 기존 v1 fixture 교체/--force | 현재팩으로 실행 | 원 호환/identity assertion을 약화하므로 금지 |
| live copy에서 새 파일 제외 | 작음 | 고정 입력이 무엇인지 모호하고 미래 콘텐츠 변경도 다시 깨짐 |

## 결과와 영향

신규 의존성 하나만 추가하며 lock의 기존 crate 버전/간선을 유지한다. 신규 기술량은 게임 defines 수치가 아니고 게임 공식은 없다. mutable sim/save/wire/client를 바꾸지 않는다. 실제 standalone validate와 CI 경로가 있고 M0 name_key 누락 및 아직 없는 base는 원 오류로 기록한다. 멀티팩 교체·패치/관리UI·미래 gameplay registry는 WP-35와 해당 producer 후속이다.

2026-10-07 공식 확인: [Fluent syntax 0.12.0 parser/AST](https://docs.rs/fluent-syntax/latest/fluent_syntax/), [MIT OR Apache-2.0 라이선스](https://docs.rs/crate/fluent-syntax/latest), [SemVer VersionReq matches/prerelease 의미](https://docs.rs/semver/latest/semver/struct.VersionReq.html). 02 §14.3 허용 목록에 부합한다. 원작 자료는 사용하지 않았다.
