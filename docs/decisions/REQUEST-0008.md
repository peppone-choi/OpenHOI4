# 결정 요청 REQUEST-0008 — WP-24 역사 데이터의 명시적 로드 호환

| 항목 | 값 |
|---|---|
| 요청일 | 2026-10-07 |
| 관련 식별자 | REQ-MOD-04, REQ-LOC-03, REQ-SAV-02, WP-24 |
| 막는 WP | WP-24 strict active 검증과 기존 M0 서버/원 v1 CLI resume의 호환 경계만 |
| 막지 않는 WP (계속 진행 중) | WP-24 current strict/server 거부·legacy defines 수정, WP-17 exact 독립 검증 준비 |

## 질문 (한 문장)

정확한 원 데이터 identity와 호출 목적이 일치할 때만 역사 `testland_name`의 ko/en 누락 두 진단을 명시 호환 경고로 남기는 A를 기술 호환 정책으로 채택할 수 있는가?

첫 독립 P-05의 FAIL은 새 validator가 서버 startup에서 호출되지 않는 문제와 registered legacy defines 누락이다. 이를 공통 active 검증으로 연결하면 원 M0 pack 및 immutable mutable-v1 fixture에는 처음부터 없던 manifest name-key 때문에 기존 서버·CLI resume 계약이 거부된다. 원 파일/expected/테스트를 바꾸지 않는 구체 호환 비교다. 새로운 게임 규칙이나 사용자 결정 상태 확정 요청이 아니다.

## 선택지

| 선택지 | 결과 | 비용·위험 |
|---|---|---|
| A | 아래 exact identity/purpose/name/diagnostic 조건을 모두 만족하는 누락 두 건만 compatibility warning. 그 외 모든 오류는 strict 거부 | 정책 데이터와 typed 진단 분류·경계 tests 필요. 경고가 full locale 검증 성공으로 오인되지 않아야 함 |
| B | 공통 active 검증을 유지하되 이름을 실제 제공하는 별도 engine catalog adapter를 설계한다. standalone validate와 active engine catalog 범위·클라이언트 제공·출처/identity를 구분한다 | 기존 팩 bytes 보존 가능하나 새 catalog/asset/참조 우선순위·runtime 전달 계약을 먼저 완성해야 함. blanket missing-key fallback은 허용하지 않음 |

## 추천안과 이유

현재 구현자가 제시한 A의 한정안을 적용한다. 기존 supported M0와 원 v1 resume를 보존하면서 현재 active 팩의 schema/참조/FTL/의존·버전 검증을 실제 진입 경로에 연결한다. 독립 Python postcard/FNV pack_hash로 부모도 아래 값들을 재계산했다. FNV64 하나만으로 원 bytes 동일성을 주장하지 않으며, 아래 원 데이터의 전체 상대 파일 목록·파일별 길이·SHA256가 runtime에서 모두 일치해야 한다. [22파일 원 bytes와 전체 manifest](evidence/historical-load-policy/README.md)를 봉인했다.

| 호출 목적 | 허용 원 source/identity | 필요한 추가 조건 |
|---|---|---|
| server_startup의 empty M0 | b6516ed 원 `examples/m0/testland`, 3파일 전수 SHA/길이/목록 및 content_hash `8747082838669556868` | manifest id=m0_testland/version0.1.0, scenario=testland/empty_m0, name_key=testland_name, typed ko/en missing_message_value 두 진단만 |
| server_restore의 원 M0 | 동일 원 M0 전수 identity | 실제 bounded format1 header/context와 pack/scenario 관계까지 일치, 위 이름 두 진단만 |
| cli_v1_resume | 원 M0 위 identity 또는 기존 immutable mutable-v1 fixture 19파일 전수 identity와 content_hash `12861016185528173627` | 실제 bounded format1 header/context, m0_testland/testland/empty_m0 또는 testland/m1/national_v1의 정확 관계·version0.1.0, name_key=testland_name, typed ko/en missing_message_value 두 진단만 |
| 기타 active / standalone validate / 다른 형 | 없음 | strict 검증 유지 |

plain frozen v1 pack identity `13318001328374612931`를 active HTTP나 CLI 예외에 추가하지 않는다. 서버는 정확 startup 또는 bounded 실제 restore 목적만 사용하고 원 frozen v1 codec은 low-level schema/graph/ref/identity 검사와 별도 roundtrip으로 지원한다. `--force`나 외부 입력이 policy/caller를 선택하거나 오류를 면제하지 않는다. 원 M0 standalone `validate`의 실제 missing-name exit1은 유지한다. 기존 headless M0 CLI run은 렌더 없는 empty-definition 계약이며 low-level resolve/schema/effective time와 모든 존재하는 FTL syntax/parity/static-ref를 검사한다. 이를 새 manifest-name 예외 entry로 만들지 않으며 current national active/run 검증과 구분한다.

숫자 magic literal과 임의 오류 문자열 필터로 구현하지 않는다. 입력팩이 바꿀 수 없는 engine-owned 정책 데이터에 source commit/path/id/scenario/kind/purpose/version/save-format/content identity와 **전체 file-list/bytes/SHA256**, field=`manifest.name_key`/key/lang/typed 진단 종류를 저장한다. typed missing_message_value 두 진단만 warning_unavailable로 변경하며 원 file/line/column/code/cause를 남긴다. warning 전문과 호환 적용 이유를 원 명령 evidence에 남긴다. 다른 FTL syntax/중복/참조·사용키/missing key, dependency/engine/version/conflict/schema/ref/defines 오류는 source가 알려진 경우에도 면제하지 않는다. 실제 codec header/body/fullstate/hash/pack/metadata/preflight 검사도 그대로 수행한다.

독립 expected: 정확 원 M0 active 정상 시작과 CLI original v1 fresh resume는 warning 2건과 원 상태/hash 유지; standalone 원 M0 validate는 exit1. 원팩 byte 하나 변경, 다른 name_key/pack/scenario/목적/형, 기타 FTL 오류 또는 의존/참조 오류는 거부·HTTP/WS 없음·save destination 미생성/원 상태 보존이다. 현재 complete 팩은 warning 없는 strict 성공이며 현재팩 fresh process 전체 state/hash 및 원 historical fixture와의 PackMismatch 검사는 그대로 유지한다.

## 기본안으로 먼저 진행할 경우 되돌림 비용

A의 한정 잠정 채택 뒤에만 예외를 구현한다. 변경은 정책 데이터/진단 목적 분류와 신규 regression이다. 원 fixture/팩/golden/단언 및 v1 codec을 재작성·거부·migration하지 않는다. B는 별도 catalog adapter 설계가 완성된 뒤 검토하며 자동 blanket fallback을 먼저 넣지 않는다.

## 사용자 답 (사용자 또는 사용자 답을 옮긴 메인 에이전트가 기록)

사용자 추가 확정 답 없음. CEO는 원후보 SHA256 `2477248184afc83d1ae5102a4ed12f11c4957c939d13b0c391724216b4b30ddc` 전문을 검토하고 원 M0/19파일+mutable transform의 전체 bytes/SHA/FNV와 16조건 경계를 독립 대조하여 A를 한정 기술 호환 잠정 채택했다. **FNV만의 예외는 승인하지 않았으며** runtime 전체 원파일 목록/길이/SHA256·id/version/scenario/kind/caller/header/context와 정확 진단 조건이 필수다. 원 후보/독립 검토는 같은 evidence 경로의 바이너리 봉인에 보존한다. 새 source의 exact 독립 P-05에서 전체 변조 행렬/native HTTP·WS/원v1/current팩을 검증해야 한다. 경고를 일반 LOC 전체 성공으로 기록하지 않는다. D/OPEN 확정 상태를 바꾸지 않는다.
