# ADR-0702 M-1b 비균일 투영(정거원통) 간선 거리·경도 순환 계약

| 항목 | 값 |
|---|---|
| 상태 | 채택 (openhoi-lead, t_229a12aa: 권고안 B와 본문 필드명·API 확정. Q1·제품 인수와 별개) |
| 날짜 | 2026-10-18 |
| 관련 WP·REQ | WP-07 후속(M-1b), ADR-0701, 02 §5.4 거리 항목 |
| 기준 커밋 | main `1374f4960653c6a910f56dd10d13ee312e208924` |

## 맥락

ADR-0701의 `regions.toml`은 균일 축척 `km_per_pixel` 하나만 받는다. 프로빈스 중심은 픽셀 중심 산술평균, 간선 거리는 픽셀 유클리드 거리 × 축척이며, 지도 가장자리 순환은 추가하지 않는다(`crates/oh_data/src/map.rs` `geometry`·`adjacency`·`scale`·`distance`). ADR-0701은 비균일 지역 투영·전 세계 순환을 후속 WP로 미뤘다.

실제 세계 미리보기 `client/public/preview/world/metadata.json`은 `projection = "EPSG:4326 Plate Carree"`, `extent = [-180,-90,180,90]`, `width = 8192`, `height = 4096`, `axis = "x east / y south"`, `cell_degrees = 0.0439453125`이다. 생성기 `tools/maps/world_preview.py`는 거리·면적을 구(평균 반경 `earth_radius_km = 6371.0088`, `data/preview/world/defines.toml`)로 계산하고, `adjacency()`는 `index[:,-1]`↔`index[:,0]` 경도 순환 인접을 포함한다. 따라서 이 미리보기를 ADR-0701 팩으로 그대로 내보내면 (1) 동서 거리가 위도 φ에서 1/cos φ배로 과대, (2) 날짜변경선 인접이 사라지고, (3) 날짜변경선을 걸친 프로빈스의 중심이 지도 가운데로 끌려간다. 이는 현재 testland 동작의 결함이 아니라 실제 세계 팩에 적용할 때의 계약 공백이다.

현재 소비자: 간선 `distance_km`만 쓴다 — `oh_sim::movement`(`formula::movement_hours`/`strait_hours`, 0 이하 거부), `oh_sim::supply::world_adapter`(0 이하 거부), `oh_data::supply_network`. `centers`와 `km_per_pixel`은 `oh_sim::world` `definitions_hash` 튜플에 들어가고, 그 해시는 `oh_save` 헤더 `definitions_hash`로 저장·복원 검사된다. 클라이언트(`client/src`)는 `km_per_pixel`을 쓰지 않는다.

### 근거 수치 (미리보기 대리점 측정, 판단 보조용)

`provinces.json` 대표 셀(중심 아님) 좌표로 `adjacency.json` 47,554개 간선을 비교했다(구 반경 6371.0088 km, 하버사인 기준). 대리점이므로 절대값이 아니라 방식 간 비교로만 쓴다.

| 평균 위도대 | 간선 수 | 현 균일식/대원거리 p50·p95 | 정거원통 평균위도식/대원거리 p50·p95·최대 |
|---|---|---|---|
| 0–10° | 5,844 | 1.001·1.011 | 1.000·1.000·1.000 |
| 40–50° | 6,772 | 1.229·1.474 | 1.000·1.000·1.004 |
| 60–70° | 6,473 | 1.809·2.622 | 1.000·1.001·1.013 |
| 70–80° | 2,809 | 2.597·4.670 | 1.000·1.003·1.015 |
| 80° 이상 | 597 | 5.406·15.115 | 1.001·1.038·1.677 |

- 대표점 x차가 W/2를 넘는 간선(날짜변경선 순환 후보) 92개. 균일식은 이들을 수백~수천 배 과대 계산한다.
- 양쪽 가장자리(x=0, x=W-1)에 모두 닿거나 경계상자가 W/2보다 넓은 프로빈스는 2개뿐이며 둘 다 전 경도 극관이다: 6788(바다, 행 0–101), 18014(육지, 행 4050–4095).

## 결정 (권고안)

### 1. `regions.toml` 형식

```toml
# 기존(변경 없음): 균일
km_per_pixel = 2

# 신규: 정거원통
km_per_pixel = 4.8865025493
projection = "equirectangular"
extent = [-180, -90, 180, 90]
```

- `projection`이 없으면 기존 균일 모드다. 파싱·중심·인접·거리·해시가 현재와 비트 단위로 같다.
- `projection = "equirectangular"`이면 `extent = [west, south, east, north]`(도)가 필수다. 그 밖의 문자열·`extent` 단독·알 수 없는 필드는 `regions.toml` 스키마 오류다.
- `km_per_pixel`은 두 모드 모두 필수다. 정거원통 모드에서는 **남북 1픽셀 행당 km**(자오선 방향 축척)를 뜻한다. 반경 상수는 받지 않는다. 생성기가 `π·R·(north−south)/(180·height)`를 소수 문자열로 써 넣는다(세계 4096행, R=6371.0088이면 약 4.8865025493).
- 모든 수치는 ADR-0701과 같이 TOML 원문 lexeme에서 I32F32로 직접 파싱한다. 정수·소수만 받고 문자열·nan·inf는 거부한다.
- `extent` 검사: 정확히 4개, `-180 ≤ west < east ≤ 180`, `-90 ≤ south < north ≤ 90`. 위반은 `regions.toml`의 `extent` 오류다.
- 경도 순환 `wrap_x`는 별도 필드가 아니라 `east − west == 360`일 때만 참이다. 위도(극) 방향 순환은 없다.

### 2. 좌표와 위도

- 축은 기존과 같이 x 동쪽, y 남쪽, 좌상단 원점이다. 픽셀 중심 y(Q32.32, `y+0.5` 포함)의 위도는 `φ(y) = north − y·(north−south)/height`도다. 픽셀 중심만 쓰므로 φ는 극(±90°)에 닿지 않는다.
- 비율 상수 `r = ((east−west)/width) / ((north−south)/height)`를 로드 시 I32F32로 1회 계산한다(세계 8192×4096·360×180이면 정확히 1).

### 3. 중심

- 균일 모드와 `wrap_x = false` 정거원통 모드: 기존 산술평균 그대로.
- `wrap_x = true`: 프로빈스마다 두 후보를 정수로 계산한다. 후보 A는 x 그대로, 후보 B는 `x' = (x + W/2) mod W`. 각 후보의 정수 분산 분자 `n·Σx² − (Σx)²`(u128)가 더 작은 쪽을 고르고, 같으면 A다. B를 고르면 평균(+0.5) 뒤 `W/2`를 빼고 `[0, W)`로 되돌린다. y는 기존 산술평균.
- 모든 열(x=0..W-1)에 픽셀이 있는 프로빈스는 **전 경도 극관**으로 표시한다. 중심 x는 후보 A 값을 저장하지만(결정론용) 거리 계산에서 이 프로빈스가 낀 간선의 동서 성분은 0으로 둔다.
- 폭이 W/2를 넘지만 전 열이 아닌 프로빈스는 두 후보 모두 근사다. 미리보기에는 없다(측정). 로더는 결정론 값을 그대로 쓰며 새 오류·경고를 만들지 않는다. `shortcut: W/2 초과 비극관 프로빈스 중심은 근사, 실제 팩 생성기가 이런 프로빈스를 만들 때 원형 평균으로 교체`.

### 4. 인접

- `wrap_x = true`면 기존 오른쪽·아래 이웃 검사에 더해 각 행의 `x = W−1`과 `x = 0`을 비교해 다른 프로빈스면 `Normal` 간선을 추가한다. 이후 override(`river_*`/`impassable` 수정, `strait` 추가) 규칙은 ADR-0701 그대로다. 대각선·극 횡단 인접은 추가하지 않는다. 이는 미리보기 `adjacency()`와 같은 집합이다.

### 5. 거리

간선 (a, b)마다 로드 중 1회 계산해 `Edge.distance_km`에 저장한다. 모든 단계는 정수·I32F32 내림이며 float·libm을 쓰지 않는다.

1. `dx = |xa − xb|`(Q32.32). `wrap_x`이고 `dx > W/2`이면 `dx = W − dx`. a 또는 b가 전 경도 극관이면 `dx = 0`.
2. `dy = |ya − yb|`.
3. 평균 위도 `φm = φ((ya + yb)/2)`(내림), 라디안 변환은 `fixed::consts::PI`/180.
4. `c = cos_fx(|φm|)`. `cos_fx`는 저장소 안의 정수 전용 함수다: 입력 `[0, π/2]` Q32.32, i128 Horner로 x¹⁸항까지 Taylor 급수(절단 오차 < 1e-12, Q32.32 해상도 2.3e-10 미만), 각 곱 뒤 내림, 결과를 `[0, 1]`로 클램프. fixed 1.31은 `sqrt`·`PI`를 제공하지만 삼각함수는 제공하지 않으므로 새 의존성 없이 직접 둔다.
5. `ex = dx · r · c`(checked_mul, 내림).
6. 기존 `distance`와 같은 경로로 `floor(isqrt(ex² + dy²))`를 Q64.64 u128로 계산하고 `km_per_pixel`을 checked_mul 한다.
7. 산술 오버플로는 기존과 같은 `distance overflow for a/b` 로드 오류다. 정거원통 모드에서는 결과가 0인 간선도 같은 위치의 로드 오류로 거부한다(이동·보급이 0 거리를 거부하므로 로드 시 조기 실패). 균일 모드의 0 거리 처리는 바꾸지 않는다.

### 6. 해시·저장 호환

- `MapData`에 투영 정보를 추가한다(권고 이름: `pub projection: MapProjection`, `enum MapProjection { Uniform, Equirectangular { west, south, east, north: Fixed, wrap_x: bool } }`). `km_per_pixel`·`centers`·`edges` 필드 의미는 유지한다.
- `oh_sim::world`의 `definitions_hash`는 `Uniform`이면 현재 튜플 그대로다. `Equirectangular`이면 기존 결과를 `state_hash(&(definitions_hash, projection_identity))`로 한 번 더 감싼다(같은 파일의 military identity 처리와 같은 패턴). 따라서 testland·testland_m2* 팩, `oh_save` fixture, 결정론 골든 해시는 바뀌지 않는다.
- 저장 포맷 버전·필드는 바꾸지 않는다. 투영이 다른 팩의 저장은 기존 `definitions_hash` 불일치 경로로 거부된다.

## 검토한 대안

| 대안 | 정밀도(미리보기 대리 측정) | 구현 비용 | 판단 |
|---|---|---|---|
| A. 현행 균일 유지 | 60°대 p50 1.81배, 80°대 p50 5.4배, 순환 간선 수백 배 | 0 | 실제 세계 이동·보급 시간이 위도에 따라 크게 왜곡되어 기각 |
| B. 정거원통 평균위도식(권고) | 80° 미만 p95 ≤ 1.003, 80° 이상 p95 1.038 | 로더 내부 수십~백여 줄, 새 파일·의존성 없음 | 채택 권고 |
| C. 구면 대원거리(하버사인) 정수 구현 | 기준값 | sin·cos·asin/atan 정수 구현 필요, 시험 범위 확대 | 인접 간선 거리에서 B 대비 이득이 극지 일부뿐이라 보류 |
| D. 생성기가 간선별 거리 CSV를 미리 계산해 팩에 저장 | 생성기 정밀도에 따름 | 새 팩 파일·스키마·override 규칙, 생성기·로더 양쪽 변경 | ADR-0701의 별도 파일 회피와 새 포맷 최소화 원칙에 어긋나 기각 |
| E. 행별 축척표를 `regions.toml`에 저장 | B와 같음 | 행 수(4096)만큼 데이터, 사람이 검토하기 어려움 | 기각 |
| F. 생성 시 원통 등적 등 다른 투영으로 재투영 | 투영마다 다름 | 미리보기·클라이언트 지도·생성기 전부 변경 | 범위 밖, 기각 |

## 결과와 영향

- 지금 바로 바뀌는 데이터·화면·저장은 없다. 기존 팩은 `projection`을 쓰지 않으므로 모든 기존 시험·골든이 그대로 통과해야 한다.
- 실제 세계 팩 생성기(M-1 본체, Q1 의존)가 이 형식으로 `regions.toml`을 쓰면 로더가 위도 보정 거리와 날짜변경선 간선을 제공한다. 생성기·국가 레이어·AC10은 이 ADR 범위가 아니다.
- 남은 한계: 평균위도 근사는 고위도 긴 간선에서 과대/과소가 커질 수 있다(80° 이상 최대 1.677배, 대리점 측정). 극관 프로빈스 동서 성분 0은 의도된 단순화다. 해협(`strait`) 거리도 같은 공식이며 별도 해상 거리 모델은 없다.

## 출처

- 정거원통(Plate Carrée) 투영 정의와 위도에 따른 동서 축척: <https://en.wikipedia.org/wiki/Equirectangular_projection>
- EPSG:4326 좌표계: <https://epsg.io/4326>
- 하버사인·정거원통 근사 거리식: <https://www.movable-type.co.uk/scripts/latlong.html>
- fixed 1.31 상수(`PI`)와 크레이트 범위: <https://docs.rs/fixed/1.31.0/fixed/consts/index.html>, <https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html>(`sqrt`는 있고 삼각함수는 없음을 문서 HTML에서 확인)
- 평균 지구 반경 6371.0088 km: 저장소 `data/preview/world/defines.toml`의 기존 값(외부 수치 재확인 아님)
