# ADR-0301 M0 데이터 로더의 수치 표현과 오류 위치

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 (Asia/Seoul) |
| 관련 WP·REQ | WP-03, REQ-GEN-02, AC-M0-03, DR-01, DR-02 |

## 맥락

02 §5.2·§5.3은 매니페스트와 Defines의 TOML 형식을 정했고, §5.9는 스키마 오류를 파일·줄·열로 보고하도록 정했다. M0에서 사용할 공개 로더 API, 소수의 변환 경로, 누락 값의 진단 위치와 구조 스키마 검사 방식은 정하지 않았다. WP-02가 `oh_core`를 별도로 구현하므로 그 브랜치의 API를 임의로 가정하지 않는다.

## 결정

- `load_pack(root) -> Result<DataPack, DataError>`로 두 필수 파일을 검사한다. 부분 팩은 반환하지 않는다. I/O는 `oh_data`의 로딩 경계에만 둔다.
- `toml::de::DeTable`과 `DeValue`의 원문 span을 사용한다. 문법 오류를 먼저 분리한 뒤, 매니페스트를 Serde 타입으로 읽고 필드·ID·SemVer를 검사한다.
- Defines의 구조는 `BTreeMap<시스템, BTreeMap<항목, 수치 또는 수치 배열>>`이다. 계수 이름 목록과 게임 범위는 시스템 WP에 남긴다. 누락 수치를 코드 기본값으로 채우지 않는다.
- 정수는 `i64`, 소수는 DR-01과 같은 `I32F32`다. `DeFloat`가 원문을 보관하므로 `f64`로 변환하지 않고 `fixed`의 checked 문자열 파서에 직접 전달한다. TOML 밑줄 표기를 제거하고 지수 표기는 파서가 처리한다. 범위 초과·NaN·무한대는 오류다. `oh_core`의 향후 `Fx` 별칭과 같은 구체 타입이므로 별도 수치 알고리즘을 만들지 않는다.
- `DataError`는 `Io`/`Syntax`/`Schema`를 구분하고 `파일:줄:열 — 메시지`로 표시한다. 줄과 열은 1부터 시작한다. UTF-8 byte span을 Unicode 문자 열로 바꾸며 CRLF도 처리한다. 누락 파일은 `1:1`, 누락 필드는 그 컨테이너의 시작 위치로 진단한다(문서 루트는 `1:1`). 오류 하나를 만나면 로드를 중단한다.
- JSON 구조 스키마는 `schemars`로 공개 타입에서 생성한다. 구조 검사는 매니페스트의 같은 Serde 타입과 Defines의 타입 형태를 따른 구문 트리 검사로 실행한다. ID 패턴과 SemVer·수치 범위는 추가로 검사한다. JSON 구조 스키마가 정수/소수의 TOML 표기 차이나 고정소수점 반올림을 모두 표현한다고 주장하지 않는다. 범용 JSON Schema 평가기는 이 골격의 API에 넣지 않는다.
- `schema` 예제로 개발용 JSON을 생성하고 스냅샷 일치 테스트를 둔다. 스키마와 오류 픽스처는 `crates/oh_data/` 아래의 개발 자료다. 배포 Testland에는 메타데이터와 빈 Defines만 두며 에셋 기록의 허용 직접 제작 값 `original`을 사용한다. D-10은 그대로 미결정이며 프로젝트 라이선스를 정하지 않는다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| TOML Value의 소수를 f64로 변환한 뒤 Fx로 변환 | 단순한 일반 값 API | 원문 정밀도를 잃어 반올림 경계에서 결과가 달라질 수 있다 |
| 모든 계수를 문자열로 작성 | 숫자 lexeme를 쉽게 보존 | 02 §5.3의 숫자 TOML 예시 형식을 바꾼다 |
| 모든 게임 계수 키와 기본값을 지금 고정 | 시스템별 강한 타입 제공 | 후속 WP와 미결정 규칙을 앞서 확정한다 |
| 범용 JSON Schema 엔진을 추가 | 임의 스키마를 실행 가능 | M0의 두 타입 형태 검사에 필요하지 않고 TOML 위치·정확한 수치 변환은 별도 구현이 필요하다 |

## 결과와 영향

로드 결과에 `f32`/`f64`나 순서 없는 컬렉션이 없다. 소수는 fixed의 기본 문자열 반올림을 따른다. 정확한 half-ULP 바로 위 원문을 읽는 테스트로 중간 부동소수점 변환이 없음을 검사한다. 이 ADR은 저장 포맷이나 게임 공식을 추가하지 않는다. 누적량 `Qty`를 사용하는 시스템 정의, 참조 무결성, 팩 의존성 해석·병합, 엔진 호환 판정, CLI `oh validate`는 해당 후속 WP가 구현한다.

2026-10-06 공식 crates.io API에서 직접 의존성의 최신 stable 버전과 라이선스를 다시 조회했다. 모두 02 §14.3 허용 목록에 있다. 선택한 버전을 `oh_data/Cargo.toml`과 Cargo.lock에 고정했다. 전체 전이 의존성은 `cargo deny check licenses bans advisories sources`로 검사했다.

| 크레이트 | 확인·선택 버전 | 라이선스 | 공식 조회 |
|---|---|---|---|
| serde | 1.0.229 | MIT OR Apache-2.0 | [crates.io](https://crates.io/api/v1/crates/serde) |
| toml | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/api/v1/crates/toml) |
| schemars | 1.2.2 | MIT | [crates.io](https://crates.io/api/v1/crates/schemars) |
| fixed | 1.31.0 | MIT/Apache-2.0 | [crates.io](https://crates.io/api/v1/crates/fixed) |
| semver | 1.0.28 | MIT OR Apache-2.0 | [crates.io](https://crates.io/api/v1/crates/semver) |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | [crates.io](https://crates.io/api/v1/crates/serde_json) |

API 동작은 Cargo가 내려받은 위 버전의 공식 소스 중 `toml/src/de/parser/{detable,devalue}.rs`, `fixed/src/macros_from_to.rs`, `schemars/src/json_schema_impls/semver1.rs`와 실제 테스트로 확인했다. 참고: [toml 공식 문서](https://docs.rs/toml/1.1.6+spec-1.1.0/toml/), [schemars 공식 문서](https://docs.rs/schemars/1.2.2/schemars/), [fixed 문자열 변환](https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html#method.from_str).
