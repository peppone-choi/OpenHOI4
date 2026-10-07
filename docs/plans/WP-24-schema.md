# WP-24 팩 해석·검증 스키마

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

CLI `load_national`/`run --pack` 및 `run --pack --save-out`은 새 host validation을 먼저 통과한 다음 World/SaveContext를 생성한다. 잘못된 dependency/FTL이면 sim 생성·저장 파일 작성 전에 거부한다. 기존 M0 default run과 v1 resume/원 save reader의 legacy scope는 유지한다. 서버 startup의 선택팩 orchestration과 multi-pack merge는 해당 서버/모드 WP가 새 API를 연결하는 후속이다.

| 추가 기술 필드/입력 | 계약/권위/수명 | hash/save/REQ |
|---|---|---|
| client KeyUse key/path/line/column | Python source 정적 t literal 필수, union catalog에 있는 일반 literal은 보수적 사용 표시; generated resource-/building-/terrain 키는 현재 common registry 기준, target uses.json만 작성 | 진단 파생, hash/save 제외, LOC-03 |
| frozen source_commit/path/files[{path,bytes,sha256}] | b6516ed 원19파일 불변; fixture 밖 JSON, source bytes audit; v1 capture만 사용하며 live generic 경로 유지 | 원 v1 pack identity와 saved.ohsave bytes 유지, MOD-02 호환 |
| capture-current | 현재팩의 같은 v1 상태/저장·재개 검사를 독립 출력으로 실행, committed old fixture 비교와 구분 | 새로운 current pack identity, 기존 format1 그대로 |

실제 legacy validate는 이름키 ko/en 누락 오류를 내며 M0 run golden은 유지한다. base 폴더가 현재 없어 실제 validate IO 오류다. 등록 adapter 전체를 미래 MOD-04 전체 완료나 base 콘텐츠 존재로 대신 표시하지 않는다. client computed 키의 모든 가능한 runtime 값을 정적 scanner가 증명하지는 않으며 parity/registered DTO 검사와 실제 runtime/UI 후속 범위를 구분한다.

원작 자료를 사용하지 않았다. schema-source-review의 공식 위키 열람불가를 유지한다. 기술 근거(2026-10-07 확인): [semver 1.0.28 VersionReq](https://docs.rs/semver/latest/semver/struct.VersionReq.html), [Fluent syntax 0.12.0](https://docs.rs/fluent-syntax/latest/fluent_syntax/), [MIT OR Apache-2.0](https://docs.rs/crate/fluent-syntax/latest). 신규 dependency 및 콘텐츠 소유는 부모에 요청했다. 이 문서는 구현/독립 검증 PASS나 전체 MOD-04 완료 증거가 아니다.
