# ADR-1101 저장 v1 검증 DTO·복원·파일 교체 경계

| 항목 | 값 |
|---|---|
| 상태 | 채택 (구현 전 기술 계약, 제품 구현·검증 대기) |
| 날짜 | 2026-10-07 |
| 관련 WP·REQ | WP-11, REQ-SAV-01, REQ-SAV-02, AC-M1-04, DR-01~DR-10 |

## 맥락

02 §7.1은 OHSV/postcard/zstd와 헤더 필드를 정하지만 byte order·전체 mutable 본문·한계·오류·force·원자적 교체를 정하지 않았다. 현재 Simulation hash에는 config/queue/WorldInputs/원장/definitions_hash가 있고 Defs는 제외된다. private 권위 객체를 임의 Deserialize로 열면 잘못된 상태와 파생 원장을 live로 공개할 수 있다. M0와 M1의 기존 canonical hash를 유지해야 한다.

## 결정

필드별 타입/단위/required/null/참조/권위/기대값은 [WP-11-schema](../plans/WP-11-schema.md)의 §2~§12를 계약으로 삼는다.

- 포맷 v1: 4B OHSV+u16 LE version+u32 LE header length+정규 postcard 헤더+단일 표준 zstd frame(정규 postcard DTO). prefix 밖 정수는 postcard encoding이다. header/body/frame trailing과 비정규 표현은 명시 거부한다.
- live 타입 Serialize 경계를 유지하고 oh_sim 소유의 저장 DTO만 Deserialize한다. export/import는 순수하고 모든 값·참조·달력/큐·원장 rawbit를 검사해 candidate를 만들며 host가 Ok일 때 한 번 교체한다. 저장 DTO bytes가 아니라 기존 Simulation canonical bytes를 해시한다.
- 모든 State/TimeConfig/queue/nation/state/province·base/modifiers/expiry·원장/적용값을 저장한다. 원장은 saved tick에 재계산하여 전 필드를 대조한 뒤 설치한다. 존재하지 않는 RNG cursor나 미래 gameplay 필드를 넣지 않는다.
- 팩 identity는 전체 UTF-8 상대 경로 정렬 Vec<(path,bytes)>의 postcard/FNV, definitions_hash는 현재 World 의미 경계, effective_defines_hash는 타입 태그를 포함한 병합 defines 경계로 분리한다. defs를 저장하지 않고 팩에서 재구성한다. force는 정의/defines/날짜/참조/원장/hash가 같을 때 pack content/version 차이만 경고 수용한다. 손상·포맷·엔진 불일치는 우회 불가다.
- 신뢰한 `oh_save/defines.toml` 정책의 file/header/body/window/string/collection cap을 allocation/decompression 전에 적용한다. save 파일에서 policy를 읽지 않는다. bounded serde visitors와 단일 frame 잔여 검사를 사용한다.
- complete encode→동일 디렉터리 tempfile write_all/flush/sync_all→persist 한 번을 파일 commit으로 삼는다. commit 전 오류는 이전 파일/상태/queue 보존, commit 후 directory durability 문제는 committed warning으로 구분한다. 전원 손실까지의 모든 filesystem durability를 보장하지 않는다.
- M1 CLI save-out/resume과 server 시작 load-save를 연결한다. 현재 time queue NationId는 정렬 namespace이고 host0이 정의 국가에 없어도 유효하다. 서버 arrival는 pending sequence 최대값에서 이어간다. WS 저장 DTO/새 client지도/HTTP upload UI는 추가하지 않는다.

## 검토한 대안

| 대안 | 장점 | 채택하지 않은 이유 |
|---|---|---|
| Simulation 직접 Deserialize | 코드량이 적음 | private 유효성/원장·중복키 검증을 우회하고 format과 live 표현을 강결합한다 |
| seed/date만 저장, world는 초기값 재생성 | 파일 작음 | 실제 mutable world/config/queue/rawbits/expiry를 잃고 REQ-SAV-02를 검사하지 못한다 |
| 원장 저장 없이 새 값만 설치 | 파생 중복 감소 | 기존 canonical/조회 결과와 불일치한 저장을 검출할 근거를 잃는다 |
| force로 새 defs hash를 교체 | 팩 변경 폭이 큼 | 기존 state hash 의미/재개 계약을 조용히 바꾸며 미승인 규칙 변경을 수용한다 |
| 기존 target truncate 후 write 또는 remove+rename | std만 사용 | 중간 오류/독자가 불완전 파일을 보는 창을 만든다 |
| multi-frame/무제한 zstd decode | 라이브러리 기본 사용 | trailing/압축 팽창·window memory 한계 계약을 지킬 수 없다 |
| 전체 저장 UI/공유 세션을 WP-11에 함께 구현 | 사용 동선 많음 | WP-37/WP-49 및 client 소유 범위를 넘는다 |

## 결과와 영향

새 header fields definitions_hash/effective_defines_hash는 불변 identity 검증을 보강하는 format v1 기술 선택이다. engine 정확 일치와 v1만 읽는 첫 포맷 정책은 보수적이며 이전/미래 버전은 명확히 거부한다. 구포맷을 실제 도입하면 §7.2 마이그레이션/명시 거부 fixture를 추가한다. 기존 M0 골든·Simulation Serialize·DR-07 FNV 경계는 변경하지 않는다.

`oh_save`는 filesystem/time/compression을 다뤄도 `oh_sim`에는 I/O/clock/OS entropy를 추가하지 않는다. tempfile 이름의 host 난수는 Simulation RNG가 아니며 canonical에 넣지 않는다. 압축은 single-thread/no dictionary/fixed level·window, checksum enabled로 고정하고 번들 native zstd를 사용한다. 서로 다른 C build가 동일 bytes를 낼 것이라는 추정은 actual 3OS fixture 검사로 확인해야 한다. SHA256 증거는 Python 표준 hashlib로 생성하고 제품 hash 알고리즘을 바꾸지 않는다.

### 2026-10-07 직접 확인한 공식 기술 출처

| 공식 출처 | 확인 범위·설계 사용 | 의존성 상태 |
|---|---|---|
| [postcard 1.1.3 take_from_bytes](https://docs.rs/postcard/1.1.3/postcard/fn.take_from_bytes.html), [패키지](https://docs.rs/crate/postcard/1.1.3/source/Cargo.toml.orig) | 잔여 slice를 받아 header/body trailing 검사. MIT OR Apache-2.0 | oh_core의 기존 =1.1.3/alloc 재사용 예정 |
| [fixed 1.31.0 serde source](https://docs.rs/fixed/1.31.0/src/fixed/serdeize.rs.html), [checked_mul](https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html#method.checked_mul) | serde가 bits를 보존하는 표현·checked 산술. DTO도 i64 bits에서 복원 | 기존 =1.31.0 재사용; 새 수치 라이브러리 없음 |
| [zstd 0.13.3 Decoder](https://docs.rs/zstd/0.13.3/zstd/stream/read/struct.Decoder.html) | 기본은 frame 이어 읽기, single_frame/finish와 window_log_max를 명시 사용 | 새 =0.13.3 default-features=false 후보, no_asm 옵션 검토 |
| [zstd-rs v0.13.3 manifest](https://raw.githubusercontent.com/gyscos/zstd-rs/v0.13.3/Cargo.toml), [같은 tag zstd-safe manifest](https://raw.githubusercontent.com/gyscos/zstd-rs/v0.13.3/zstd-safe/Cargo.toml) | wrapper MIT, safe MIT OR Apache-2.0. safe version range로 전이 resolution이 달라질 수 있음 | 실제 lock version/features/cc build는 구현 때 pin·기록; 설치 완료라고 쓰지 않음 |
| [zstd-sys 2.0.16+zstd.1.5.7 manifest](https://docs.rs/crate/zstd-sys/2.0.16%2Bzstd.1.5.7/source/Cargo.toml), [upstream zstd 1.5.7 LICENSE](https://raw.githubusercontent.com/facebook/zstd/v1.5.7/LICENSE) | sys bindings MIT/Apache-2.0와 동봉 C BSD-3-Clause 고지를 따로 확인 | 가능한 pin 후보 확인. 실제 resolver가 채택한 버전의 소스/고지는 재확인 필요 |
| [tempfile 3.27.0 persist](https://docs.rs/tempfile/3.27.0/tempfile/struct.NamedTempFile.html#method.persist), [manifest](https://docs.rs/crate/tempfile/3.27.0/source/Cargo.toml.orig) | 동일 filesystem에서 atomic replace API, contents/directory sync는 별도; MIT OR Apache-2.0 | 새 =3.27.0 후보. host 전이 라이선스/플랫폼 build는 구현 때 확인 |

확인한 라이선스 표현은 02 §14.3 목록에 들어간다. 실제 Cargo.lock 전체 검사·서드파티 고지는 구현 단계에 필요하며 현재 출처 확인만으로 모든 전이 의존성이 허용됐다고 기록하지 않는다. docs.rs의 zstd Cargo.toml/Cargo.toml.orig 및 fixed crate-source 경로 일부는 도구 Internal Error였으므로 그 경로를 열람 근거로 쓰지 않고 위의 정상 GitHub tag/rustdoc source를 사용했다. 위키 본문을 새로 열람한 기록은 없고 [기존 조사 실패 범위](../research/schema-source-review.md)를 유지한다. 원작 문장/수치표를 설계에 사용하지 않았다.

추가 read-only registry 문맥은 현재 oh_data 경계에 없으므로 오케스트레이터의 좁은 소유 배정 뒤 구현한다. root Cargo/lock·binary fixture attributes·assets manifest는 필요한 항목만 조정하고 타세션 변경을 보존한다. 이번 ADR/설계 커밋에서는 제품·의존성·CI·골든·클라이언트를 수정하지 않는다.
