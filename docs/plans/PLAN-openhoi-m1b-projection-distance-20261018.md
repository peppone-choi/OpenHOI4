# PLAN M-1b 정거원통 거리·경도 순환 최소 구현 계약

- taskId: t_0e69f169 (기획) / 다음 단위: M-1b-1 (openhoi-lead 채택 후 openhoi-implement)
- 기준: main `1374f4960653c6a910f56dd10d13ee312e208924`
- 설계 근거: `docs/adr/ADR-0702-equirectangular-distance.md` (상태: 제안)
- 범위 경계: 국가·소유 레이어(Q1), 실제 세계 팩 생성기, `states.toml` 분할, AC10, 클라이언트 `MapView`는 범위 밖이다. 이 단위는 로더가 새 `regions.toml` 모드를 이해하게 만드는 것만 한다.

## 1. lead 채택 시 단일 확정할 것

권고안 그대로 채택하면 아래 이름을 쓴다. 바꾸려면 lead가 하나로 정해 이 문서와 구현 카드에 함께 적는다.

| 항목 | 권고 값 |
|---|---|
| 모드 필드 | `projection = "equirectangular"` (없으면 균일) |
| 범위 필드 | `extent = [west, south, east, north]` (도) |
| 축척 의미 | `km_per_pixel` = 균일 모드 축척 / 정거원통 모드 남북 행당 km |
| 순환 | 별도 필드 없음, `east − west == 360`일 때 x 순환 |
| Rust 타입 | `oh_data::map::MapProjection { Uniform, Equirectangular { west, south, east, north: Fixed, wrap_x: bool } }`, `MapData.projection` |
| 해시 | Uniform은 현 튜플 그대로, Equirectangular는 `state_hash(&(definitions_hash, projection_identity))` |

## 2. 다음 단위 M-1b-1: 로더 정거원통 모드

### 파일 소유

| 파일 | 변경 |
|---|---|
| `crates/oh_data/src/map.rs` | `scale` → regions 파서 확장, `MapProjection`, wrap 중심·인접, `cos_fx`, 거리 함수, `regions_schema` |
| `crates/oh_data/schema/regions.schema.json` | `regions_schema()` 출력과 일치하도록 갱신 |
| `crates/oh_data/tests/map_projection.rs` (신규) | 아래 시험. 기존 `tests/map.rs`의 Fixture 유틸은 복사 없이 재사용 가능하면 모듈로 분리하지 말고 신규 파일 안 최소 fixture를 둔다 |
| `crates/oh_sim/src/world.rs` | `definitions_hash` 계산 구역만(현 약 488–575행). 다른 구역 수정 금지 |
| `docs/adr/ADR-0702-equirectangular-distance.md` | 상태를 채택으로, 확정 이름 반영 |

수정 금지: `data/`, `assets/`, `client/`, `tools/`, `.github/`, 기존 팩·fixture, `HANDOFF.md`, ADR-0701 본문, 01/02/03 기획 문서. 새 의존성 금지(Cargo.toml·Cargo.lock 변경 없음).

### 입력·출력·오류 계약

- 입력: `<pack>/maps/<id>/regions.toml`.
- `projection` 없음 → 현재 동작과 비트 동일(파싱 오류 메시지·`km_per_pixel` 값·중심·간선·거리·`definitions_hash`).
- `projection = "equirectangular"` + 유효 `extent` → `MapData.projection = Equirectangular{..}`, ADR-0702 §3–5 중심·인접·거리.
- 오류(모두 `DataError`, 파일 `regions.toml`, 해당 키 위치, 메시지에 키 이름 포함):
  - `projection`이 문자열 `"equirectangular"`가 아님
  - `projection` 있는데 `extent` 없음 / `extent`만 있음
  - `extent` 길이 ≠ 4, 숫자 아님, nan/inf, `west < -180`, `east > 180`, `west ≥ east`, `south < -90`, `north > 90`, `south ≥ north`
  - 알 수 없는 필드(기존 `unknown regions field` 유지)
  - 거리 오버플로(기존 `distance overflow for a/b` 유지), 정거원통 모드의 0 거리 간선(`zero distance for a/b`)
- 결정론: float·libm 금지. 모든 곱·나눗셈은 I32F32/i128/u128 정수, 내림. 동률 선택은 ADR-0702 §3 규칙.

### 인수조건 (Given/When/Then)

1. Given 기존 testland 및 testland_m2* 팩(`km_per_pixel = 2`, `projection` 없음), When `load_map`과 `World` 생성, Then 모든 `centers`·`edges`·`km_per_pixel` 비트와 `definitions_hash`가 main과 같고 기존 시험·결정론 워크플로가 변경 없이 통과한다.
2. Given 소형 정거원통 fixture(`extent = [-180,-90,180,90]`, 행 0의 x=0과 x=3이 서로 다른 프로빈스이고 그 밖에서는 붙지 않음), When 로드, Then 그 두 프로빈스 사이 `Normal` 간선이 있고, 같은 비트맵에 `extent = [-90,-90,90,90]`(비순환)이면 그 간선이 없다.
3. Given 날짜변경선을 걸친 프로빈스(x=0과 x=W−1 열 점유, 가운데 열 비점유) fixture, When 순환 모드 로드, Then 중심 x가 `0` 또는 `W−1+0.5` 근처(가장자리 1픽셀 이내)이고 W/2 근처가 아니다.
4. Given 적도 대칭 행과 고위도 행에 같은 픽셀 dx를 가진 두 간선, When 로드, Then 고위도 간선 거리 < 적도 간선 거리이며, 적도 근처 간선은 `dx·km_per_pixel·r`과 Q32.32 몇 ulp 이내로 같다.
5. Given 모든 열을 차지하는 극관 프로빈스와 그 이웃, When 로드, Then 그 간선 거리는 `dy·km_per_pixel`의 내림과 비트 동일(동서 성분 0)이다.
6. Given ADR-0702 §1 오류 목록 각 입력, When 로드, Then `regions.toml` 스키마 오류로 실패하고 부분 결과를 반환하지 않는다.
7. Given 같은 정거원통 팩, When `World` 생성, Then `definitions_hash`가 같은 비트맵의 균일 팩과 다르고, 두 번 로드해도 같다.

fixture 주의: ADR-0701 규칙상 1픽셀 프로빈스는 로드 오류이고 모든 육지는 정확히 한 주에 속해야 하므로, 시험 비트맵은 프로빈스마다 2픽셀 이상이고 바다 위주이거나 최소 `states.toml`을 둔다. 크기는 인수조건에 필요한 만큼만(예: 8×4) 키운다.

### 깨지면 실패해야 하는 시험 (`map_projection.rs`)

- `cos_fx` 고정 벡터: `cos_fx(0) == 1`(비트), `cos_fx(π/3)`가 0.5와 2⁻²⁸ 이내, `cos_fx(π/2) ≥ 0`이고 2⁻²⁸ 이하, 입력 0..π/2 등간격 64점에서 단조 비증가. 이 시험은 Taylor 항 수를 줄이거나 내림 위치를 바꾸면 깨져야 한다.
- 위 인수조건 2–6을 각각 별도 `#[test]`로. 거리 기대값은 시험 안에서 정수로 직접 계산한 비트값과 `assert_eq!` 비교(근사 비교는 인수조건 4의 "몇 ulp"만).
- 회귀 고정: 기존 `scale_is_exact_fixed_positive_and_checked`와 `structural_map_schemas_match_checked_in_files`를 수정 없이 통과(스키마 파일만 갱신).
- 해시: 인수조건 7.

### 검증 절차 (GitHub-hosted 전용)

로컬 빌드·무거운 시험은 하지 않는다. 구현자는 브랜치를 push하고 PR을 열어 hosted 결과로 증명한다.

1. `ci.yml`: `cargo fmt --check`, `cargo clippy --workspace --locked -- -D warnings`, `cargo build/test --workspace --locked`, `tools/check_architecture.py`.
2. `sim-determinism.yml`, `save-determinism.yml`, `core-determinism.yml`: 기존 골든·OS 간 해시 비교가 변경 없이 PASS(인수조건 1의 근거).
3. PR 본문에 exact head SHA와 각 워크플로 run URL, 신규 시험 이름 목록을 남긴다.
4. 독립 리뷰(openhoi-review)는 `cos_fx` 정수 경로, wrap 중심 동률, 균일 모드 비트 보존을 중점으로 본다.

### 비목표

- 실제 세계 팩 생성기·`provinces.png` 내보내기, 국가·주 분할, 클라이언트 표시, 성능 측정(8192×4096 로드 시간은 M-1 본체 위험으로 남김), 구면 대원거리.

## 3. 이후 순서 (배정은 lead)

1. lead: ADR-0702 채택·이름 확정 → M-1b-1 카드 생성(openhoi-implement).
2. M-1b-1 PR → openhoi-review → openhoi-verify → openhoi-merge.
3. M-1 본체(Q1 답 뒤 기획): 생성기가 `projection`/`extent`/`km_per_pixel`을 metadata에서 써 넣고, wrap 간선이 미리보기 `adjacency.json`과 같은지 대조하는 시험을 둔다.

## 4. 남은 위험

- 평균위도 근사의 고위도 오차(80° 이상 p95 1.038, 최대 1.677, 대리점 측정)는 게임 체감상 허용 여부가 미확정이다. 문제 시 대안 C(대원거리)로 거리 함수만 교체 가능하게 `distance` 경계를 하나로 둔다.
- 측정은 대표 셀 대리점 기준이라 실제 중심 기준 수치와 다를 수 있다.
