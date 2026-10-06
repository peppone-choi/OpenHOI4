# oh_data — M0 데이터 팩 로더

`load_pack(root)`는 `manifest.toml`과 `defines.toml`을 모두 읽고 검사한 뒤 `DataPack`을 반환한다. 실패 시 파일 경로, 1부터 시작하는 줄·열, `Io`/`Syntax`/`Schema` 분류를 가진 `DataError`를 반환한다. 표시 형식은 `파일:줄:열 — 메시지`다. 열은 Unicode 문자 수이며, 누락 파일은 `1:1`, 누락 필드는 해당 컨테이너의 시작 위치를 사용한다(문서 루트는 `1:1`).

매니페스트 필수 키는 `id`, `name_key`, `version`, `engine`이다. `depends`, `conflicts`, `load_after`를 생략하면 빈 목록이다. 알 수 없는 필드는 거부한다. 팩 ID 형식, SemVer 버전과 버전 범위 문법을 검사한다. 의존 팩의 존재, 충돌 해결, 엔진 버전 호환 판정, 현지화 키의 존재는 WP-24 범위다.

Defines는 시스템 테이블 안에 정수·소수 또는 수치 배열을 둔다. 정수는 `i64`, 소수는 `fixed::types::I32F32`다. TOML 소수의 원문을 직접 고정소수점 파서에 전달하므로 부동소수점 중간값을 사용하지 않는다. 범위 초과, 무한대, NaN, 문자열, 날짜, 불리언, 중첩 테이블·배열은 거부한다. 시스템·계수 이름과 게임상 범위는 각 시스템 WP가 정하며, M0에는 게임 기본값을 만들지 않는다. 결과 맵은 `BTreeMap`으로 정렬한다.

JSON 구조 스키마는 공개 타입에서 `schemars`로 생성한다. 매니페스트 구조 검사는 같은 타입의 Serde 역직렬화로, Defines 구조 검사는 TOML 구문 트리 순회로 실행한다. 버전 범위와 수치 표현 한계는 별도의 의미 검사다. 임의 외부 JSON 스키마를 실행하는 범용 검증기는 제공하지 않는다.

```powershell
cargo test -p oh_data --test loader
cargo run -p oh_data --example schema
```

두 번째 명령은 `schema/*.schema.json`을 갱신한다. 테스트는 생성 스키마와 체크인한 JSON의 일치를 검사한다. 게임 골든 데이터를 갱신하는 명령이 아니다.

`tests/fixtures/`는 자체 제작한 개발용 입력이며 게임에 배포하지 않는다. 특히 그 수치들은 게임 계수가 아니다. M0 메타데이터와 빈 Defines 및 기존 provenance는 보존한다. D-10은 2026-10-06 코드 GPL-3.0-or-later, 직접 제작 데이터·에셋 CC-BY-SA-4.0으로 확정됐으며 신규 데이터는 그 라이선스로 등록한다.

## M1 지도 경계 (WP-07)

`map::load_map(pack_root, "testland")`는 `<pack>/maps/testland`의 RGB8 PNG·프로빈스 CSV·인접 override·states·regions와 `common/`의 지형/자원/건물 ID 레지스트리를 읽어 원자적으로 검증한다. 기존 `load_pack`은 이 함수를 자동 호출하지 않는다. 파일 계약·반올림·경고·후속 연결은 [ADR-0701](../../docs/adr/ADR-0701-map-data-contract.md)에 기록했다.

`MapData`의 `provinces`, `states`, `victory_points`, `edges`는 ID 순으로 정렬된다. `index`는 좌상단부터 행 우선인 밀집 u16 배열이고 `index_le_bytes()`는 HTTP 지도 배열을 만든다. 인덱스 0도 유효하다. `province_index(id)`, `state_for_province(id)`, `edge(a,b)`로 조회하고 `centers`/간선 `distance_km`은 I32F32 캐시다. 프로빈스/주 ID 저장은 oh_core와 같은 u16이며 시뮬레이션 경계에서 typed ID로 변환한다.

인접은 4방향만 쓰며 override로 하천·해협·통행불가를 명시한다. 1픽셀·unknown RGB·미지/중복 ID·잘못된 종류/레지스트리 참조·육지 주 누락/중복·음수 주 개수·잘못된 VP/축척은 오류다. 분리된 덩어리는 성공 결과의 경고이며 CSV의 선택 `island` 열로 그 경고만 억제할 수 있다. 범용 모드 병합/validate CLI와 현지화 누락 검사는 WP-24다. VP 표시·실제 국가 상태 로드는 WP-08/09다.

최소 자체 제작 Testland 지도는 8×6, 6프로빈스·2주·2VP다. `tests/map.rs`는 그 직사각형 경계에서 직접 도출한 정렬 간선/픽셀 배열과 오류 변형을 검사한다. 기존 골든은 변경하지 않는다.

```powershell
cargo test -p oh_data --test map
cargo run -p oh_data --example schema
```

두 번째 명령은 기존 스키마와 함께 province/states/regions 구조 스키마를 생성한다. province 스키마는 CSV를 변환한 공개 Rust 레코드(rgb 배열·island bool 포함)의 구조이며 원본 CSV 열 검사는 Rust 파서가 담당한다. 값·참조·지형 무결성은 로더의 의미 검사가 담당한다. 신규 이름 키는 팩의 `localisation/ko/map.ftl`·`en/map.ftl`에 있다.
