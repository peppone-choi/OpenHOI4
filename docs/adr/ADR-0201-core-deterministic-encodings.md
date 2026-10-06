# ADR-0201 코어 ID·RNG·직렬화 인코딩

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-02, REQ-GEN-04, REQ-PLAT-02, DR-01, DR-02, DR-03, DR-04, DR-07, DR-09 |

## 맥락

02 §3과 §4.4는 고정소수점 타입, 엔티티 ID 폭, ChaCha8Rng 파생 튜플, postcard 및 FNV-1a를 정한다. ID API, 시스템·게임일 표현, 종류가 다른 엔티티의 RNG 네임스페이스, 튜플의 바이트 배치와 직렬화 호출 계약은 정하지 않았다. 코어는 게임 공식·계수·시나리오 규칙을 추가하지 않는다.

## 결정

- `Fx`와 `Qty`는 각각 `fixed::types::I32F32`와 `I48F16`의 별칭이다. 기본 `fixed` 반올림을 그대로 유지한다. 검사 연산의 정수 기준값과 음수의 곱셈·나눗셈 반올림을 테스트한다. 후속 게임 공식은 `oh_sim::formula`에 둔다.
- `ProvinceId(u16)`, `StateId(u16)`, `NationId(u16)`, `DivisionId(u32)`는 서로 다른 투명 newtype이다. 숫자 정렬, 폭이 같은 명시적 변환, getter, 표시와 serde를 제공한다. 0도 유효하다. 슬롯 할당·삭제·alive 처리는 상태 소유자의 범위다.
- `SystemId(u32)`는 안정적인 시스템 네임스페이스, `GameDay(u64)`는 시나리오 시작 이후의 경과 일수다. 시스템 ID 값의 실제 배정과 달력은 후속 WP가 다룬다. 시스템 시각을 참조하지 않는다.
- `EntityId`의 인코딩 태그는 Global=0, Province=1, State=2, Nation=3, Division=4다. 다른 종류의 같은 숫자 ID가 같은 RNG 스트림을 공유하지 않는다. 새로운 종류는 태그를 뒤에 추가하며 기존 태그를 재배정하지 않는다.
- RNG 키를 32바이트로 일대일 대응시킨다: 게임 시드 u64(0..8), 시스템 u32(8..12), 게임일 u64(12..20), 엔티티 태그 u32(20..24), 엔티티 값 u64(24..32). 모두 little-endian이고 Global 값은 0이다. 이 바이트를 `ChaCha8Rng::from_seed`에 전달한다. 해시로 키를 압축하지 않으며 OS 엔트로피를 쓰지 않는다. 같은 키로 새 인스턴스를 만들면 위치가 초기화되므로 호출자는 한 번의 엔티티·시스템·게임일 평가에서 인스턴스를 유지한다. 이 방식은 게임 재현용이며 비밀 키 생성용이 아니다.
- `canonical_bytes`는 이미 순서가 정해진 상태를 postcard로 직렬화한다. 호출자는 모든 필드에서 정렬된 BTree 컬렉션 또는 ID 순서 Vec, 고정 폭 정수, Fx/Qty를 사용해야 한다. Hash 컬렉션·부동소수점·포인터 폭 상태 정수 및 환경 의존 사용자 직렬화를 넣지 않는다. 일반 serde API는 필드의 Rust 타입을 검사할 수 없으므로 이 계약을 자동 강제한다고 주장하지 않는다. `from_canonical_bytes`는 남은 바이트를 거부하지만 최소 varint나 상태 순서를 검증하는 정규화 검사는 아니다.
- `state_hash`는 전체 postcard 바이트가 생성된 경우에만 64비트 FNV-1a를 적용한다. 직렬화 오류를 그대로 반환한다. `Fnv1a64`는 청크 경계와 무관한 증분 해시도 제공한다. FNV 상수와 RNG 태그·바이트 폭은 포맷 정의이며 defines에서 바꿀 게임 수치가 아니다.
- 구현은 `no_std` + alloc이다. 일반 의존성은 시계·네트워크·OS RNG 기능을 사용하지 않는다. `proptest`는 dev 전용이며 테스트 RNG의 시드를 고정하고 실패 파일 자동 저장을 끈다. 테스트 실행기는 std를 사용한다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 원시 정수를 모든 ID에 사용 | API가 짧다 | 종류 혼용을 컴파일 시 막지 못한다 |
| 공유 RNG 하나 또는 호출 순번으로 seed 파생 | 관리가 단순하다 | 다른 엔티티의 처리 순서·난수 소비에 결과가 의존한다 |
| 튜플을 64비트 해시한 뒤 RNG seed로 확장 | 임의 길이 키를 받기 쉽다 | 현재 튜플은 32바이트에 손실 없이 들어가며 압축 충돌이 불필요하다 |
| 모든 Serialize 구현의 컬렉션을 자동 정렬 | 호출자가 편하다 | serde는 BTreeMap과 HashMap의 타입을 구분해 주지 않고, Vec의 의미 있는 순서를 임의로 바꿀 수 없다 |
| 별도 고정소수점 연산 래퍼·사용자 반올림 | 전용 API를 제공한다 | DR-01/09가 지정한 fixed 타입과 기본 동작을 그대로 사용하는 편이 일관된다 |

## 결과와 영향

ID 폭, 엔티티 태그, 필드 순서와 RNG 키 배치를 바꾸면 저장·재현·해시 호환성에 영향을 준다. 후속 변경은 포맷 버전과 호환성 판단을 별도로 해야 한다. 이 WP는 아직 저장 포맷이나 완전한 시뮬레이션 상태를 만들지 않는다. 테스트의 `core-format-v1`은 정수 경계·타입 ID·고비트 튜플 RNG·postcard를 포함하는 합성 포맷 참조이며 Testland 게임 골든이 아니다. Python의 독립 정수 ChaCha8·varint·FNV 구현으로 처음 생성했고, 체크인된 참조 스크립트는 비교만 하며 파일을 덮어쓰지 않는다.

직접 의존성은 명세의 정확한 버전을 고정했다. 공통 Cargo.toml은 바꾸지 않았고, Cargo.lock에는 코어와 dev 테스트 의존성 해석 결과가 추가됐다. `proptest`의 rand_chacha 0.9와 제품의 0.10은 별개이며 제품 RNG는 0.10만 사용한다. `syn` 2·3 중복은 전이 의존성이고 기존 cargo-deny 정책에서 경고다.

2026-10-06 공식 조회: fixed 1.31.0, rand_chacha 0.10.0, serde 1.0.229, postcard 1.1.3, proptest 1.11.0은 모두 MIT 또는 Apache-2.0 허용 라이선스다. [공식 레지스트리 조회와 feature 기록](../worklog/evidence/WP-02/versions.txt), [전체 lockfile의 cargo-deny 검사](../worklog/evidence/WP-02/licenses.txt)를 남겼다. API는 [fixed](https://docs.rs/fixed/1.31.0/fixed/), [rand_chacha](https://docs.rs/rand_chacha/0.10.0/rand_chacha/), [postcard](https://docs.rs/postcard/1.1.3/postcard/) 및 [postcard wire spec](https://postcard.rs/spec-c68ec81.pdf)을 확인했다. FNV 상수·연산·테스트 벡터는 [RFC 9923](https://datatracker.ietf.org/doc/html/rfc9923)을 확인했다.
