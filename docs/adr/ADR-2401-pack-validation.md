# ADR-2401 팩 선택 순서·Fluent·고정 v1 입력

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
