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

`tests/fixtures/`는 자체 제작한 개발용 입력이며 게임에 배포하지 않는다. 특히 그 수치들은 게임 계수가 아니다. `data/packs/testland/`에는 M0 메타데이터와 빈 Defines만 배포하고 두 파일 모두 `assets/ASSETS.toml`에 등록한다. 프로젝트 라이선스 선택은 D-10의 미결정 상태를 유지한다.
