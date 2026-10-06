# OpenHOI 제작용 문서

| 항목 | 내용 |
|---|---|
| 문서 | 02 제작용 문서 (어떻게 구현하고 검증하는가) |
| 짝 문서 | [01 기획서](01-game-design.md), [03 제작용 프롬프트](03-production-prompts.md) |
| 작성 | 최병호 |
| 기준일 | 2026-10-06 (버전 정보 확인일) |
| 문서 버전 | 0.1 |

---

## 1. 이 문서의 사용 규칙

1. **게임 규칙은 01이 정한다.** 01에 없는 규칙이나 `미결정` 규칙을 구현 중에 만들지 않는다. 필요해지면 03 P-10(결정 요청)을 쓴다.
2. **기술 세부는 에이전트가 정할 수 있다.** 이 문서에 없는 기술 선택(자료 구조, 내부 API, 보조 크레이트)은 구현 세션이 정해도 된다. 정한 내용은 `docs/adr/ADR-NNNN-제목.md`에 기록한다(템플릿: `docs/templates/ADR_TEMPLATE.md`). 이 문서의 결정과 충돌하는 변경은 사용자 승인이 필요하다.
3. **식별자를 바꾸지 않는다.** REQ, WP, DR, AC 식별자는 그대로 둔다. 문서를 고친 뒤에는 `python3 tools/check_docs.py`가 통과해야 한다.
4. **버전 정보는 기준일에 확인한 값이다.** WP-01을 시작할 때 다시 확인하고, 바뀐 값은 §2.1 표와 `Cargo.lock`에 반영한다.

---

## 2. 기술 결정 (D-07, D-16 상세)

### 2.1 확인된 버전 (2026-10-06 조사)

**서버 (Rust)**

| 구성 요소 | 버전 | 라이선스 | 비고 |
|---|---|---|---|
| Rust (stable) | 1.99.0 (2026-10-01) | MIT/Apache-2.0 | 툴체인은 `rust-toolchain.toml`로 고정 |
| `tokio` | 1.53.2 | MIT | 비동기 런타임(네트워크 전용) |
| `axum` | 0.8.9 | MIT | HTTP·WebSocket, 정적 파일 제공 |
| `rmp-serde` | 1.3.1 | MIT | 프로토콜 MessagePack 직렬화 |
| `ts-rs` | 12.0.1 | MIT | 프로토콜 타입 → TypeScript 생성 |
| `fixed` | 1.31.0 | MIT/Apache-2.0 | 고정소수점 |
| `rand_chacha` | 0.10.0 | MIT/Apache-2.0 | 시뮬레이션 RNG(`ChaCha8Rng`) |
| `serde` | 1.0.229 | MIT/Apache-2.0 | |
| `postcard` | 1.1.3 | MIT/Apache-2.0 | 저장 파일·정규 직렬화 |
| `toml` | 1.1.6+spec-1.1.0 | MIT/Apache-2.0 | 데이터 파일 |
| `schemars` | 1.2.2 | MIT | 데이터 스키마 생성 |
| `zstd` | 0.14.0 | BSD-3-Clause | 저장 파일 압축 |
| `proptest` | 1.11.0 | MIT/Apache-2.0 | 속성 기반 테스트 |
| `cargo-deny` | 0.20.2 | MIT/Apache-2.0 | 라이선스·취약점 검사(도구) |

**클라이언트 (웹)**

| 구성 요소 | 버전 | 라이선스 | 비고 |
|---|---|---|---|
| TypeScript | 7.0.2 | Apache-2.0 | |
| Vite | 8.3.3 | MIT | 개발 서버·번들 |
| React | 19.3.0 | MIT | UI 패널 |
| three | 0.186.1 | MIT | WebGPU 우선·WebGL2 대체 지도 렌더링 |
| `@msgpack/msgpack` | 3.1.3 | ISC | 프로토콜 디코딩 |
| `@fluent/bundle` | 0.19.1 | Apache-2.0 | 현지화 |
| Vitest | 5.0.3 | MIT | 단위 테스트 |
| `@playwright/test` | 1.63.0 | Apache-2.0 | 브라우저 E2E·스크린샷 |

2026-10-06 M0 착수 재확인: 서버 크레이트 13개와 클라이언트 패키지 8개를 공식 레지스트리 API에서 조회했다. 버전 번호는 기존 표와 같으며, `toml`의 빌드 메타데이터와 `zstd` 라이선스를 정확히 보완했다. 개별 조회 URL·결과는 [버전 재확인 기록](research/M0-versions.md)에 있다. Rust 1.99.0은 공식 릴리스 공지로 확인했다.

**사용 금지:** `bincode`. RUSTSEC-2025-0141에서 유지보수 중단으로 공지됐고, 3.0.0은 빌드되지 않는 은퇴 릴리스다. `cargo-deny`의 `bans`에 등록한다.

### 2.2 선택 근거

- **클라이언트는 웹이다(D-07 확정).**
  - 브라우저에서 Three.js(WebGPU 우선, WebGL2 대체)로 지도를 그리고, React로 패널을 만든다.
  - 클라이언트는 게임 규칙을 계산하지 않는다. 명령을 보내고, 받은 상태를 그리기만 한다(REQ-NET-05). 규칙 구현이 서버 한 곳에만 있게 된다.
- **서버는 Rust 권위 서버다.**
  - 시뮬레이션·AI·저장은 서버에서만 돈다.
  - 결정론에 필요한 제어(부동소수점 배제, 순회 순서)를 언어 수준에서 다룰 수 있다.
  - GC가 없어 정지 시간이 생기지 않는다.
  - 같은 시뮬레이션 크레이트를 서버와 헤드리스 CLI가 함께 쓴다(REQ-GEN-01).
- **프로토콜은 WebSocket + MessagePack이다.**
  - 타입은 Rust(`oh_proto`)에 한 번만 정의하고 `ts-rs`로 TypeScript를 생성한다. 서버와 클라이언트의 타입이 어긋날 수 없다(REQ-NET-04).
- **실행 파일은 하나다.**
  - 서버가 빌드된 클라이언트 정적 파일을 함께 제공한다.
  - 싱글플레이 = 로컬에서 서버를 실행하고 브라우저로 접속. 호스팅 비용이 없다(D-13).

### 2.3 검토한 대안

| 대안 | 장점 | 채택하지 않은 이유 | 재검토 조건 |
|---|---|---|---|
| 서버를 Kotlin/Spring Boot로 | 사용자에게 익숙한 스택 | 시뮬레이션 결정론 통제(부동소수점·GC), 헤드리스 CLI와의 코드 공유가 Rust보다 불리 | 서버는 종류를 가리지 않는다는 지시가 있었으므로 D-16은 잠정이다. 사용자가 서버 코드 리뷰 용이성을 우선하면 전환한다. 프로토콜 계약(§4.3)은 그대로 유지된다 |
| Godot 4.7 웹 내보내기 | 게임 엔진의 에디터·UI 도구 | Godot 4 C# 프로젝트는 웹으로 내보낼 수 없다. godot-rust의 Wasm 지원은 실험 단계다. 패널이 많은 UI는 React 생태계가 넓다 | — |
| 시뮬레이션을 Wasm으로 만들어 브라우저에서만 실행 | 서버 없이 오프라인 플레이 | 멀티플레이용 권위 서버를 따로 만들어야 하고, 대규모 시뮬레이션 성능이 문제 | 오프라인 플레이가 요구될 때. DR-04 덕분에 시뮬레이션 크레이트는 Wasm 빌드가 가능한 상태로 유지된다 |
| PixiJS 또는 WebGL2 직접 호출 | 2D 특화 / 의존성 없음 | 셰이더 머티리얼과 카메라가 기본 제공되는 Three.js가 구현량이 적다 | 성능 문제가 생기면 |
| JSON 프로토콜 | 디버깅이 쉬움 | 대역폭 | 디버그 모드에서 JSON을 병행 허용 |

### 2.4 참조 환경 (성능 목표 기준)

| 항목 | 기준 |
|---|---|
| 서버 CPU | 6코어 12스레드, 2020년대 초반 중급 데스크톱(로컬 실행 기준) |
| 메모리 | 16GB |
| 서버 OS | Linux x64, Windows 11 x64, macOS(Apple Silicon) |
| 브라우저 | 최신 Chrome·Edge·Firefox·Safari 및 Chromium 계열 데스크톱. WebGL2 필수 |
| GPU | 중급 외장 또는 최신 내장 GPU |
| 해상도 | 1920×1080 |

최소 사양은 M3 플레이 테스트 후 확정한다.

---

## 3. 결정론 규칙 (DR)

REQ-GEN-04를 지키기 위한 규칙이다. 구현·검증 세션은 이 목록으로 점검한다.

| ID | 규칙 |
|---|---|
| DR-01 | 시뮬레이션 상태와 계산에 `f32`/`f64`를 쓰지 않는다. 비율·계수는 `Fx = fixed::types::I32F32`, 누적량은 `Qty = fixed::types::I48F16`, 개수는 `i64`를 쓴다. 표시용 변환(f64)은 프로토콜 계층 `oh_proto`에서만 한다. |
| DR-02 | 상태 안에서 `HashMap`/`HashSet`을 순회하지 않는다. 엔티티는 ID 순서의 `Vec`이나 `BTreeMap`으로 보관한다. |
| DR-03 | 난수는 시뮬레이션 RNG만 쓴다. `ChaCha8Rng`를 `(게임 시드, 시스템 ID, 게임일, 엔티티 ID)`로 결정적으로 파생한다. `thread_rng`, OS 엔트로피, 시스템 시계는 쓰지 않는다. |
| DR-04 | 시뮬레이션 크레이트에서 파일·네트워크 I/O와 현재 시각 조회를 하지 않는다. |
| DR-05 | 시스템 내부를 병렬화할 때는 결과가 처리 순서와 무관한 경우만 허용한다(ID로 정렬한 뒤 병합). 처음에는 단일 스레드로 구현한다. |
| DR-06 | 상태 변경은 명령 적용 단계나 스케줄된 시스템 안에서만 일어난다. |
| DR-07 | 상태 해시는 정규 직렬화 바이트(postcard)에 대한 64비트 FNV-1a다. 직렬화하는 컬렉션은 모두 순서가 정의되어 있어야 한다. |
| DR-08 | AI 예산은 실제 시간이 아니라 **작업 단위 카운터**로 제한한다. 시간 기반 예산은 결정론을 깬다. |
| DR-09 | 나눗셈과 곱셈 순서, 반올림 방식(`fixed`의 기본 동작)을 공식 구현과 테스트에서 같게 유지한다. 공식은 `oh_sim::formula` 모듈에 모은다. |
| DR-10 | 결정론 테스트(같은 시드 2회, 저장 후 재개, 세 OS)는 CI에서 매 커밋 실행한다. macOS는 D-14 기간에 야간 실행이다. |

---

## 4. 아키텍처

### 4.1 계층

```
┌──────────────────────── 브라우저 클라이언트 (client/) ────────────────────────┐
│ React UI 패널 · Three.js 지도(WebGPU/WebGL2) · 입력 · Fluent 현지화 · 상태 스토어 │
│      ▲ 스냅샷 / 델타 / 조회 결과 / 알림                 │ 명령 / 조회 요청        │
└──────┼──────────────────────────────────────────────────┼──────────────────────┘
       │  WebSocket (/ws, MessagePack)  ·  HTTP (정적 파일, 지도·그래픽·현지화, 저장 파일)
┌──────┴────────────────────── oh_server (axum, tokio) ────▼──────────────────────┐
│ 게임 세션 관리 · 플레이어↔국가 배정 · 명령 수신·검증 · 델타 생성 · 조회 처리      │
│ 정적 파일 제공 · 저장/불러오기 API                                                │
└──────┬───────────────────────────────────────────────────────────────────────────┘
       ▼
┌ oh_proto ─────────────────────┐   ┌ oh_cli ─────────────────────────────────┐
│ 프로토콜 타입 → TS 생성(ts-rs) │   │ run · validate · ai-bench · bench · repro  │
└────────────────────────────────┘   │ save · docs · map                          │
┌ oh_sim ── 상태·스케줄러·시스템·명령·원장 ┐   └────────────────────────────────────────────┘
├ oh_ai ─── 전략·경제·연구·외교·군사      ┤   ┌ oh_save ── 저장·재현 번들 ┐
├ oh_data ─ 데이터 팩 로드·병합·검증       ┤   └──────────────────────────┘
├ oh_core ─ Fx/Qty·ID·RNG·정규 직렬화·해시 ┤
└────────────────────────────────────────┘
```

**의존 방향:** `oh_core ← oh_data ← oh_sim ← oh_ai`, `oh_sim ← oh_save`, `oh_sim ← oh_proto ← oh_server`. `oh_server`와 `oh_cli`가 최상위다. 시뮬레이션 쪽 크레이트는 네트워크·비동기 런타임에 의존하지 않는다(CI에서 `cargo tree`로 검사).

### 4.2 실행 모델

- **스레드**: 게임 세션마다 전용 시뮬레이션 스레드가 있다(동기 코드). tokio 런타임은 네트워크만 처리하고, 둘은 채널로 연결된다.
- **명령**: 클라이언트 → 서버 검증 → 게임 세션 큐 → 다음 틱 시작에 적용한다. 순서는 (국가 ID, 서버 도착 순번)이다.
- **델타**
  - 틱 처리 후 바뀐 엔티티를 모은다. 실제 시간 기준 최대 10회/초로 묶어 보낸다.
  - 델타마다 일련번호가 있다. 클라이언트는 번호가 빠지면 전체 스냅샷을 요청한다(REQ-NET-03).
- **속도·일시정지**: 싱글플레이는 플레이어가 정하고, 멀티플레이는 §10 권한 규칙을 따른다.
- **저장**: 서버가 게임 세션을 잠깐 멈추고 직렬화한다.

### 4.3 프로토콜 (`oh_proto`)

| 방향 | 메시지 | 내용 |
|---|---|---|
| C→S | `Hello` | 클라이언트 프로토콜 버전 |
| S→C | `Welcome` | 서버 엔진·프로토콜 버전, 데이터 팩 목록·해시, 열린 게임 세션 목록. 버전이 맞지 않으면 거부 사유 |
| C→S | `Create` / `Join` | 게임 세션 생성(시나리오·시드·모드), 참가(국가 선택, 멀티는 세션 코드) |
| C→S | `Command` | §6.4 명령 + 클라이언트 순번 |
| S→C | `CommandResult` | 수락 또는 거부(사유 현지화 키) |
| S→C | `Snapshot` | 전체 상태(참가·재동기화 시) |
| S→C | `Delta` | 일련번호, 날짜, 바뀐 프로빈스 소유·통제, 부대, 전투, 국가 요약, 알림 |
| C→S, S→C | `Query` / `QueryResult` | 수치 원장 분해, 패널 상세, 지도 모드 색 버퍼 |
| S→C | `Notice` | 이벤트 팝업, 경고 |

- 인코딩은 MessagePack이다. Rust 타입에서 `client/src/proto/`로 TypeScript 타입을 생성한다. 생성물이 최신이 아니면 CI가 실패한다.
- 표시용 수치는 `oh_proto`가 f64로 바꿔 보낸다. 클라이언트는 이 값을 표시에만 쓰고 규칙 계산에 쓰지 않는다.
- 지도 비트맵·그래픽·현지화 파일은 WebSocket이 아니라 HTTP 정적 경로로 보낸다. 파일은 데이터 팩 해시로 캐시를 구분한다.

### 4.4 상태 모델

- 엔티티는 종류별 `Vec`에 저장하고 타입이 있는 ID(`ProvinceId(u16)`, `StateId(u16)`, `NationId(u16)`, `DivisionId(u32)` 등)로 참조한다.
- 지운 엔티티는 슬롯을 재사용하지 않는다. 세대 번호 대신 `alive` 플래그와 단조 증가 ID를 쓴다.
- 데이터 팩에서 읽은 정의(장비, 기술, 의제 등)는 불변 `Defs` 구조로 분리한다. 상태는 정의를 ID로 참조한다. `Defs`는 저장 파일에 넣지 않고 데이터 팩 해시로 확인한다.
- 클라이언트 상태 스토어는 스냅샷으로 초기화하고 델타를 순서대로 적용한다. 이것은 표시용 사본이고, 규칙의 근거가 아니다.

---

## 5. 데이터 규격

### 5.1 데이터 팩 배치

```
data/packs/<pack_id>/
  manifest.toml
  defines.toml                 # 수치 상수(01 §4의 모든 계수)
  common/
    terrain.toml  resources.toml  buildings.toml  ideologies.toml  laws.toml
    equipment.toml  battalions.toml  support_companies.toml
    technologies.toml  traits.toml
  agendas/<nation_or_generic>.toml
  events/*.toml
  decisions/*.toml
  maps/<map_id>/
    provinces.png              # 24비트 RGB, 색 하나 = 프로빈스 하나
    provinces.csv              # id,r,g,b,kind,terrain,coastal
    adjacency_overrides.csv    # a,b,kind  (river_small|river_large|strait|impassable)
    states.toml  regions.toml  positions.toml(라벨·부대 표시 좌표, 선택)
  scenarios/<scenario_id>/
    scenario.toml              # 시작일·종료 조건·지도·국가 목록·초기 외교
    nations/<TAG>.toml         # 정부·이념·법령·지도자·연구 완료 목록·의제 트리 지정
    oob/<TAG>.toml             # 부대 배치
  localisation/<lang>/*.ftl
  gfx/ ...                     # 아이콘·국기 등 (출처는 assets 매니페스트)
  SOURCES.md                   # 역사 수치 출처 (REQ-CNT-05)
```

**저장소 안의 팩**

| 팩 | 내용 | 비고 |
|---|---|---|
| `testland` | 자체 완결형 가상 시나리오 | 골든 테스트 전용. 다른 팩에 의존하지 않는다. 그래서 기본 게임 콘텐츠가 바뀌어도 골든 결과가 흔들리지 않는다. |
| `base` | 기본 게임 공통 정의, 유럽·전 세계 지도와 시나리오 | |
| `examples/*` | M3 예제 모드 2종 | |

### 5.2 매니페스트

```toml
# data/packs/my_mod/manifest.toml
id = "my_mod"                     # 소문자·숫자·밑줄
name_key = "my_mod_name"          # 현지화 키
version = "0.3.0"                 # SemVer
engine = ">=0.1, <0.2"            # 엔진 버전 범위
depends = [{ id = "base", version = ">=0.1" }]
conflicts = ["other_overhaul"]
load_after = ["base"]
```

### 5.3 defines

`defines.toml`은 01 §4에 나온 모든 계수를 담는다. 키 이름은 `시스템.항목`이다. 코드에 같은 값을 리터럴로 두지 않는다(REQ-GEN-02). 아래 값은 잠정이다.

```toml
[time]
speed_ms_per_tick = [500, 200, 80, 25, 0]   # 0 = 성능 한계까지

[combat]
damage_variance = 0.15
hit_factor_unblocked = 0.35
hit_factor_blocked = 0.08
org_damage_share = 0.7
armor_advantage_damage_taken_mult = 0.6

[supply]
decay_per_cost_unit = 0.06
starvation_ratio = 0.3
starvation_days_before_attrition = 5

[diplomacy]
surrender_threshold = 0.8
```

### 5.4 지도 파일

- **`provinces.png`**: 무압축 또는 무손실 PNG, 24비트 RGB, 안티에일리어싱 금지. 검증기는 다음을 검사한다.
  - `provinces.csv`에 없는 색이 있으면 오류.
  - 한 프로빈스가 여러 덩어리로 나뉘면 경고. 섬 예외는 csv에 표시한다.
  - 1픽셀짜리 프로빈스가 있으면 오류.
- **인접 생성**: 4방향 이웃 픽셀의 색이 다르면 인접으로 본다. 결과는 정렬된 간선 목록이고 골든 테스트 대상이다(REQ-MAP-02).
- **거리**: 프로빈스 중심 간 거리를 지도 투영 축척(`regions.toml`의 `km_per_pixel`)으로 환산한다. 거리는 로드 시 계산해서 캐시한다.
- **원천 데이터**: Natural Earth와 검토된 OpenHistoricalMap 요소를 사용하며 사용자 고도 지시 범위에서 독립 DEM 원천을 추가할 수 있다. 공개 플레이 화면의 산맥·강·도시 주변 분할 원칙·밀도 비교만 사용자 승인 예외로 허용한다. 원작 경계 복사·트레이싱·지도 파일 추출/사용·수치표는 금지한다(REQ-LEG-01, 01 §7).
- **취득과 오프라인 생성 분리(REQ-MAP-10)**: 취득은 제작 단계의 명시 작업이다. 원천 파일과 URL·취득일·고정 버전/snapshot·라이선스 검토·SHA256을 보존하고, 생성기는 네트워크 없이 해당 파일을 입력으로 프로빈스 PNG/정의/주/인접/지형 파일을 만든다. 누락 원천·해시 불일치는 명확한 실패다. 같은 입력·도구 버전·옵션의 산출 해시를 2회 비교한다. 원천 자체를 배포에 동봉할지는 파일별 라이선스와 팩 매니페스트에서 판단한다.
- **라이선스 선별**: OHM 기본 CC0 설명과 개별 license 태그·OSM 수입 해안/수역/하천 예외를 분리한다. 현재는 CC0로 검토된 OHM 요소만 사용하고, 허용되지 않은 원천 요소를 전체 CC0로 등록하지 않는다. Natural Earth는 공식 public-domain 파일의 버전을 고정한다. 1936/1939 역사 경계의 실제 자료 충족 범위는 WP-32/45에서 조사·기록한다.
- **실행 경계**: 서버와 브라우저는 동봉 팩을 자체 HTTP로 제공/읽는다. 외부 지리 API·타일 서버·지도 CDN 요청은 없다. WP-08은 실제 브라우저 네트워크 캡처로 이 경계를 검증하고 외부 지도 없이 실행한다. OS 네트워크/샌드박스 설정을 바꾸지 않는다.
- **고도·지형 생성(REQ-MAP-11)**: DEM snapshot의 해상도·파일 용량·NoData·좌표와 수직기준·단위·지역별 능선/고개 정확도를 ADR로 비교한다. 오프라인 능선·하천/도시 자료와 고도를 프로빈스 생성/검수 입력으로 쓰고 입력/산출 hash·도구 버전·옵션을 기록한다. NOAA ETOPO2022는 공식 전세계·CC0 후보이며 아직 취득/채택했다고 기록하지 않는다. 실제 생성 알고리즘은 독립 데이터에서 정하며 게임 이동/전투 수치를 추가하지 않는다.
- **지형의 독립 검수(WP-32/45)**: 원천 좌표계·그리드를 정합한 뒤 고도·하천 선형·프로빈스 경계·인접을 지역 확대 overlay로 대조한다. 하천 연결·호수/해안 진입·주요 능선/고개·좁은 단절을 살피며 평균 고도만으로 대신하지 않는다. M3 표본 후보는 알프스/피레네/카르파티아 및 라인/다뉴브/비스와 일대다. M5는 해협·dateline·극지역도 추가 검수한다. 표본 추천은 새로운 게임 효과·경계를 확정한 것이 아니다. Natural Earth의 `10m`은 1:10 million 축척이며 10미터 격자가 아니다. 공식 평활·정렬·누락 한계를 기록하고 고정 원천의 정확성을 별도로 검토한다([조사 기록](research/M1-map-source-policy.md)).

### 5.5 정의 파일 예시

예시 값은 Testland용 임시 값이다. 원작 값을 옮기지 않는다.

```toml
# common/equipment.toml
[[equipment]]
id = "rifle_gen1"
family = "infantry_arms"
generation = 1
unit_cost = 0.45                  # IC·일 단위
resources = { steel = 0.02 }
stats = { soft_fire = 3.0, hard_fire = 0.5, defense = 18.0, breakthrough = 2.0, piercing = 2.0 }

# common/battalions.toml
[[battalion]]
id = "line_infantry"
frontage = 2
speed_kmh = 4.0
organization = 60.0
strength = 25.0
manpower = 1000
supply_use = 0.06
equipment = { infantry_arms = 100 }
```

```toml
# agendas/generic.toml
[[agenda]]
id = "generic_industrial_drive"
days = 70
requires = { all = [] }
excludes = ["generic_army_first"]
available = { not = { at_war = true } }
effects = [
  { add_building = { scope = "capital_state", building = "industry", levels = 1 } },
  { add_stability = -0.02 },
]
ai_weight = { base = 10, modifiers = [{ when = { stability = { lte = 0.4 } }, mult = 0.5 }] }
```

### 5.6 트리거·효과 문법 (REQ-AGD-05)

**조건 노드.** 키가 정확히 하나인 테이블이다.

| 형태 | 의미 |
|---|---|
| `{ all = [조건…] }` | 모두 참 |
| `{ any = [조건…] }` | 하나 이상 참 |
| `{ not = 조건 }` | 부정 |
| `{ <원시조건> = 인자 }` | 등록된 원시 조건 |

비교 인자는 `{ gte = x }`, `{ lte = x }`, `{ eq = x }` 또는 불리언이다.

**효과 목록.** 키가 정확히 하나인 테이블의 배열이다.

| 형태 | 의미 |
|---|---|
| `{ scope = { target = "...", effects = [...] } }` | 대상 범위 전환 |
| `{ if = { condition = ..., then = [...], else = [...] } }` | 조건 분기 |
| `{ <원시효과> = 인자 }` | 등록된 원시 효과 |

**대상 표기:** `self`, `nation:<TAG>`, `state:<id>`, `capital_state`, `faction_leader`, `owner_of_state:<id>`, `enemy_of_war`(전쟁 문맥).

**규칙**

1. 원시 조건과 원시 효과는 코드의 레지스트리에 등록한다. 각 항목은 인자 스키마·설명·예시를 가진다. 데이터 레퍼런스 문서는 레지스트리에서 자동 생성한다(REQ-MOD-07).
2. 조건 평가는 부작용이 없다. 확률 조건(`chance`)은 이벤트 발동 판정에서만 허용하고 시뮬레이션 RNG를 쓴다.
3. 중첩 깊이는 최대 16이다. 한 번의 효과 실행은 최대 1,000개 원시 효과로 제한한다. 넘으면 로드 오류 또는 실행 중단과 경고를 낸다.
4. 미지원 키, 잘못된 인자, 존재하지 않는 ID 참조는 **로드 오류**다. 실행 중에 발견하면 안 된다.

**초기 원시 항목 (M2, 이후 확장)**

| 종류 | 항목 |
|---|---|
| 조건 | `date_gte`, `at_war`, `at_war_with`, `nation_is`, `stability`, `mobilization`, `political_capital`, `has_law`, `controls_province`, `owns_state`, `has_flag`, `ideology_support` |
| 효과 | `add_stability`, `add_mobilization`, `add_political_capital`, `set_law`, `add_building`, `transfer_state`, `declare_war`, `add_manpower`, `add_equipment`, `set_flag`, `clear_flag`, `end_scenario` |
| M3 추가 | `has_agenda_completed`, `has_technology`, `world_tension`, `in_faction`, `unlock_technology`, `fire_event`, `create_faction`, `join_faction`, `add_casus_belli`, `set_government` |

### 5.7 시나리오

`scenario.toml`에 다음을 정의한다.

- `start_date`, `end_date`
- `map`
- `nations`(참가 국가 태그)
- `ownership`(주 → 소유국), `control_overrides`
- `factions`, `wars`(시작 시 진행 중인 전쟁)
- `end_conditions`(조건 문법)과 `score_weights`

국가별 초기 상태는 `nations/<TAG>.toml`, 부대 배치는 `oob/<TAG>.toml`에 둔다. 새 시나리오를 추가할 때 코드를 고치지 않는다(REQ-MOD-06).

### 5.8 현지화

- 형식은 Fluent(`.ftl`)이고, 기본 언어는 `ko`와 `en`이다.
- 코드와 데이터에는 키만 둔다. 화면에 보이는 문자열을 하드코딩하지 않는다.
- CI가 확인하는 것: 모든 언어에서 키가 빠지지 않았는지, 쓰이지 않는 키가 있는지(REQ-LOC-03).
- 폰트는 OFL 라이선스 CJK 폰트(Noto Sans KR 계열 등)를 쓰고 출처를 매니페스트에 기록한다.

### 5.9 검증 규칙 (`oh validate`)

1. TOML 문법 오류와 스키마 위반(`schemars`로 생성한 JSON 스키마)을 `파일:줄:열 — 메시지`로 보고한다.
2. 참조 무결성: 존재하지 않는 ID, 순환 선행(의제·기술), 배타 관계의 비대칭을 검사한다.
3. 지도: §5.4 검사, 주에 속하지 않은 육지 프로빈스, 두 주에 속한 프로빈스를 검사한다.
4. 현지화: 키 누락.
5. 종료 코드: 오류가 있으면 1, 경고만 있으면 0. `--deny-warnings` 옵션을 주면 경고도 실패로 처리한다.

### 5.10 병합과 패치 (REQ-MOD-03)

- 팩은 의존 관계와 `load_after`로 정한 순서대로 로드한다. 순환이 있으면 오류다.
- 같은 종류에 같은 `id`가 다시 나오면 뒤에 로드한 팩이 **교체**한다.
- 부분 수정은 `patches/*.toml`에 명시적 연산으로 쓴다.

  ```toml
  [[patch]]
  target = "equipment:rifle_gen1"
  op = "set"                          # set | add | remove | append | merge
  path = "stats.soft_fire"
  value = 3.5
  ```

- 병합 결과는 `oh validate --print-merged <kind>:<id>`로 확인할 수 있다.

---

## 6. 시스템 간 연결

### 6.1 시스템 표

| 시스템 | 읽기 | 쓰기 | 주기 |
|---|---|---|---|
| 명령 적용 | 명령 큐 | 각 시스템의 의도 상태(이동 목표, 생산 설정 등) | 틱 |
| 이동 | 경로, 속도, 지형, 인프라, 보급 비율 | 부대 위치, 이동 진행도 | 틱 |
| 전투 | 참여 사단 능력치, 보정치 | 조직력·전력, 퇴각, 통제 | 틱 |
| 보급 | 보급원, 철도, 거리, 통제, 사단 소모 | 프로빈스 보급 용량, 사단 보급 비율 | 일 |
| 경제 | 공업 시설, 보정치, 배분 비율, 자원 | 국가 IC, 부문별 IC, 자원 충족률, 시장 구매 | 일 |
| 생산 | 생산 라인, 군수 IC, 자원 충족률 | 비축, 라인 효율 | 일 |
| 건설 | 대기열, 건설 IC, 인프라 | 건물 수준 | 일 |
| 인력·훈련·보충 | 인구, 징병 법령, 비축 | 인력 풀, 사단 전력, 훈련 진행 | 일 |
| 연구 (M3) | 연구 슬롯, 보정치 | 기술 완료, 해금 | 일 |
| 정치 | 법령, 이벤트 효과 | PC, 안정도, 동원도, 이념 지지율 | 일 |
| 의제·결의 (M3) | 진행 중인 의제, 조건 | 효과 실행 | 일 |
| 이벤트 (M3) | 조건, 평균 발생 일수, RNG | 이벤트 큐 | 일 |
| 외교 | 긴장도, 명분 진행, 통제 VP | 전쟁 상태, 항복, 진영 | 일 |
| AI | 스냅샷 수준의 상태 읽기 | 명령만 발행 | 일(국가별 분산) |
| 수치 원장 | 보정치 스택 | 파생 수치 캐시, 분해 기록 | 변경 시 |

### 6.2 처리 순서 (REQ-TIME-02)

**매 틱**

1. 이번 틱에 예약된 명령을 적용한다. 순서는 (국가 ID, 발행 순번) 오름차순이다.
2. 이동을 진행한다(사단 ID 순).
3. 전투를 해결한다(전투 ID 순). 새로 접촉한 전투를 만든다.
4. 퇴각과 진입을 처리하고 통제를 바꾼다(프로빈스 ID 순).

**게임일 0시 틱**에는 위 1~4 다음에 아래 일일 단계를 실행한다.

5. 보급
6. 경제(IC·배분·자원·시장)
7. 생산
8. 건설
9. 인력·훈련·보충
10. 연구
11. 정치
12. 의제·결의
13. 이벤트 판정
14. 외교(긴장도·명분·항복 진행도·강화)
15. AI(국가 ID 순). 무거운 계층은 `국가 ID mod 7 == 게임일 mod 7`인 날에만 돈다.
16. 수치 원장의 변경된 항목을 갱신한다.

**매월 1일**: 월간 통계를 집계한다. 자동 저장은 시뮬레이션 밖의 `oh_server` 게임 세션 관리자가 수행한다.

**마지막 단계**: 스냅샷을 발행한다.

### 6.3 보정치와 수치 원장 (REQ-UI-04)

- 보정치 구조: `{ source, target_stat, op: Add | Mul, value: Fx, expires }`.
  - `source` 예: `law:conscription_2`, `terrain:forest`, `agenda:...`, `general:<id>`.
- 파생 수치는 `기본값 → Add 합 → Mul 곱` 순서로 계산한다. 같은 대상의 보정치는 `source` 문자열 순으로 정렬해 적용한다(DR-02).
- 수치 원장은 각 파생 수치에 대해 (항목, 연산, 값, 누적 결과) 목록을 보관한다. 요청할 때 분해 결과를 돌려준다.
- **테스트 규칙**: 원장 분해의 최종 누적값은 실제 시스템이 쓴 값과 비트 단위로 같아야 한다.

### 6.4 명령 목록 (초기)

| 범주 | 명령 |
|---|---|
| 시간 | `SetSpeed`, `Pause` (싱글 전용. 멀티플레이에서는 §10 권한 규칙) |
| 경제 | `SetAllocation`, `SetProductionLine`, `RemoveProductionLine`, `QueueConstruction`, `ReorderConstruction`, `CancelConstruction` |
| 정치 | `ChangeLaw`, `StartAgenda`(M3), `TakeDecision`(M3), `ChooseEventOption`(M3) |
| 연구 | `StartResearch`(M3), `CancelResearch`(M3) |
| 외교 | `DeclareWar`, `JustifyCasusBelli`(M3), `CreateFaction`/`InviteToFaction`/`LeaveFaction`(M3), `Guarantee`(M3), `RequestAccess`(M3), `ProposeNonAggression`(M3) |
| 군사 | `CreateTemplate`, `EditTemplate`, `TrainDivision`, `MoveDivisions`, `HoldDivisions`, `AssignToArmy`, `CreateArmy`, `SetReinforcePriority`, `AssignFront`(M3), `SetOffensivePlan`(M3), `SetDefenseLine`(M3), `NavalTransport`(M3) |

모든 명령은 적용 전에 검증한다. 소유권, 비용, 조건을 확인하고, 실패하면 거부 사유를 알림으로 보낸다. AI도 같은 명령만 쓴다(REQ-GEN-05).

---

## 7. 저장·불러오기·재현

### 7.1 저장 파일 (`*.ohsave`)

```
magic "OHSV" (4B) | format_version u16 | header_len u32 | header (postcard) | body (zstd(postcard(State)))
```

**헤더 필드**

- `engine_version`, `format_version`
- `scenario_id`
- `packs`: (id, version, content_hash)의 목록
- `game_date`, `tick`, `seed`, `state_hash`
- `player_nations`
- `saved_at_utc`: 표시용이고 해시에 포함하지 않는다.

저장 파일은 서버의 사용자 데이터 경로에 둔다. 브라우저는 HTTP로 목록 조회·내려받기·올리기를 한다(REQ-SAV-06). 올린 파일은 헤더 검증을 통과해야 불러온다.

**데이터 팩 `content_hash`**: 상대 경로로 정렬한 각 파일의 (경로, 바이트)를 FNV-1a 64로 해시한다. 불러올 때 다르면 경고하고, `--force`가 없으면 거부한다.

### 7.2 버전 관리 (REQ-SAV-04)

- `oh_save::migrate`에 `v(N) → v(N+1)` 변환 함수를 쌓는다.
- 경로가 없으면 이런 메시지를 낸다: "이 저장 파일은 포맷 vN입니다. 이 버전은 vK 이상만 열 수 있습니다."
- 포맷을 바꾸는 PR은 마이그레이션 테스트(이전 포맷 샘플 파일 → 로드 → 해시 기록)를 함께 넣는다.

### 7.3 재현 번들 (REQ-SAV-05)

- 번들은 ZIP 하나다.
  - `bundle.toml`: 시나리오, 팩과 해시, 시드, 시작 저장 파일 여부, 종료 틱, 기대 해시
  - `commands.log`: (틱, 국가, 순번, 명령) 목록. 직렬화 형식과 사람이 읽는 형식 2종
  - 선택: `start.ohsave`
- CLI
  - `oh repro record --out x.zip`: 게임 중 기록
  - `oh repro run x.zip`: 재생한 뒤 기대 해시와 비교. 종료 코드 0 또는 1
- 버그를 수정할 때는 재현 번들을 회귀 테스트로 저장소에 추가한다(03 P-06).

---

## 8. AI 설계

### 8.1 계층과 주기

| 계층 | 주기 | 입력 | 출력(명령) |
|---|---|---|---|
| 전략 | 7일 분산 | 이웃 군사력, 이념 거리, 긴장도, 진영 | 국가 목표(방어·확장 대상·진영 참여) 가중치 |
| 경제 | 일 | 전쟁 상태, 자원 부족, 목표 | 배분 비율, 건설 대기열, 생산 라인 |
| 연구·의제 | 일 | 데이터 AI 가중치, 목표, 역사 경로 옵션 | 연구 시작, 의제 선택, 결의 |
| 외교 | 7일 분산 | 목표, 긴장도, 상대 위협 | 명분, 진영, 보장, 선전포고 |
| 군사 | 일 + 전투 이벤트 | 전선, 지역 전력비, 보급 | 전선 배분, 공세 계획, 이동, 훈련·편제 |

### 8.2 군사 계층 알고리즘 (잠정)

1. **위협 평가**: 적 접경선을 연속 구간으로 나눈다. 각 구간마다 적 전력(인접 프로빈스 사단 전투력 합)과 거리 감쇠를 반영한 위협도를 계산한다.
2. **배분**: 가용 사단을 위협도에 비례해 구간에 배분한다. 구간마다 최소 1개 사단을 보장하고, 남는 전력은 예비로 둔다.
3. **공세 판단**: 구간의 아군 대 적 전투력 비율이 `defines.ai.attack_ratio`(잠정 1.5) 이상이고 보급 비율이 0.7 이상이면 공세 계획을 만든다. 목표는 VP와 적 보급 거점에 가중치를 둔다.
4. **실행**: 공세 계획 실행기는 플레이어 계획과 같은 코드를 쓴다.
   - 매일 목표선까지 경로상의 다음 프로빈스를 공격 대상으로 정한다.
   - 공격 개시 조건: 공격 사단의 평균 조직력이 0.6 이상.
5. **예산**: 국가별 일일 작업 단위 상한(`defines.ai.work_units_per_day`)이 있다. 넘으면 다음 날로 이월한다(DR-08).

### 8.3 평가 하네스 (REQ-AI-07)

```
oh ai-bench --scenario testland --runs 50 --days 1500 --seed-base 1000 --out report.json [--baseline baseline.json]
```

**리포트 지표**

| 지표 |
|---|
| 크래시 수 |
| 종료 사유 분포 |
| 전쟁 발생 수와 시점 |
| 항복까지 걸린 일수 |
| 국가별 IC 성장 |
| 유휴 사단 비율 |
| 보급 부족 사단·일 비율 |
| 미사용 IC 비율 |
| 평균 AI 작업 단위 |

기준선과 비교해 회귀 임계(지표별 `bench/thresholds.toml`)를 넘으면 실패다. 목표값 자체는 사용자가 정한다(01 §4.11).

---

## 9. 모드 설계

1. **탐색**: 서버가 `data/packs/`와 사용자 모드 폴더(서버 OS의 사용자 데이터 경로 `OpenHOI/mods`)에서 찾는다. 클라이언트는 모드를 갖고 있지 않고, 서버가 제공하는 파일을 쓴다.
2. **선택**: 모드 관리 화면이나 CLI 인자로 고른다. 의존성을 해석하고 충돌을 검사한다.
3. **로드**: 정렬된 순서로 정의를 읽는다(§5.10).
4. **검증**: §5.9. 오류가 있으면 게임을 시작하지 않는다. 오류 목록은 화면에 보여준다.
5. **해시**: 활성 팩 목록과 해시를 저장 파일과 멀티플레이 로비에서 비교한다.

**제외:** 기존 HOI4 모드 변환과 원작 파일 읽기(D-09). 원작 스크립트 형식의 파서도 만들지 않는다.

---

## 10. 멀티플레이 설계 (M6, 서버 권위형)

- **구조**: 한 게임 세션에 여러 클라이언트가 접속한다. 시뮬레이션은 서버에서만 돈다. 클라이언트는 명령만 보낸다. 그래서 락스텝과 피어 간 비동기 문제가 없다.
- **참가**: 게임 세션을 만든 사람이 호스트다. 다른 플레이어는 세션 코드(무작위 토큰)로 참가해 국가를 고른다. 국가당 1인, 최대 8인(잠정)이다. 계정과 공개 호스팅은 OPEN-10이다.
- **데이터 일치**: 데이터 팩은 서버에만 있다. 클라이언트는 서버가 제공하는 파일을 쓰므로 모드 불일치가 생기지 않는다. 클라이언트 버전은 프로토콜 버전으로 검사한다.
- **명령 검증(REQ-MP-03)**: 서버는 클라이언트를 신뢰하지 않는다. 모든 명령의 발행 국가가 그 연결에 배정된 국가인지, 비용과 조건을 충족하는지 확인한다.
- **이탈과 재접속(REQ-MP-04)**: 연결이 끊기면 설정에 따라 그 국가를 AI가 맡는다. 같은 세션 코드와 재접속 토큰으로 돌아오면 스냅샷을 보내고 조작권을 돌려준다.
- **권한(REQ-MP-05)**: 일시정지는 누구나 할 수 있고, 해제는 요청한 사람이나 호스트가 한다. 속도는 호스트가 정하고, 다수결 투표로 바꿀 수 있다.
- **정보 공개 범위**: 싱글플레이와 같다(전장의 안개 없음).
- **결정론을 유지하는 이유**: 서버 권위형이라도 재현 번들, 저장 파일 이식, 골든 테스트, 서버 OS 간 결과 일치에 결정론이 필요하다.

---

## 11. 렌더링·UI 설계 (웹 클라이언트)

### 11.1 프로빈스 지도 (Three.js, WebGPU·WebGL2)

1. **인덱스 데이터**: 서버가 로드할 때 `provinces.png`를 프로빈스 인덱스(u16, 리틀엔디언) 배열로 바꿔 HTTP로 제공한다(`/maps/<map_id>/index.bin`, 크기 정보 포함). 클라이언트는 이를 RG8 `DataTexture`로 만든다. 필터는 `NearestFilter`, 밉맵은 쓰지 않는다.
2. **색 조회 텍스처**: 256×256 RGBA8이다. 인덱스 i는 (i mod 256, i div 256)에 있다. 지도 모드를 바꾸거나 델타가 오면 바뀐 프로빈스만 갱신한다. 부분 갱신 방법은 WP-08에서 측정하고 ADR로 남긴다.
3. **셰이더**: 가용 WebGPU와 WebGL2 대체 백엔드에 공통 TSL·노드 머티리얼을 적용한다. WebGPU는 WGSL, WebGL2는 GLSL로 생성해 같은 인덱스·색 조회·국경 동작을 검증한다. 기존 ShaderMaterial 방식의 알고리즘을 그대로 사용할 때에는 별도 백엔드 구현과 동일성 검증을 ADR로 남긴다.
   - `texelFetch`로 인덱스를 읽어 색을 조회한다.
   - 이웃 텍셀과 인덱스를 비교해 국경을 그린다.
   - 소유국·주 조회 텍스처를 비교해 국가 국경과 주 국경의 두께·색을 다르게 한다. 확대 수준에 따라 선 굵기를 조절한다.
4. **선택·호버**: 유니폼으로 강조할 인덱스를 넘긴다. 마우스 위치의 프로빈스는 클라이언트가 인덱스 배열에서 바로 찾는다.
5. **카메라**: `OrthographicCamera`로 확대·이동한다. M5 전 세계 지도의 좌우 순환은 UV 순환으로 처리한다.
6. **텍스처 크기**: WebGPU 어댑터의 텍스처 크기 제한 또는 WebGL2의 `MAX_TEXTURE_SIZE`를 선택한 백엔드에 맞춰 확인한다. 전 세계 지도(M5)가 넘으면 타일로 나눈다.
7. **향후**: 거리장 기반 그라데이션 국경. Paradox·Intel의 Imperator: Rome 기술 글에 나온 방식이고, M5에서 선택 적용한다.

### 11.2 부대·전선 표시

- 부대 아이콘은 `InstancedMesh`로 그린다. 확대 수준에 따라 군 단위 집계 → 사단 단위로 바뀐다(REQ-MAP-06).
- 전선은 접경 간선을 이은 선으로, 공세 계획은 화살표 메시로 그린다.

### 11.3 UI

- React 컴포넌트로 만든다. 패널은 01 §5를 따른다. 지도 캔버스 위에 겹쳐 놓는다.
- 패널은 열릴 때와 관련 델타가 올 때 `Query`로 상세를 요청한다. 전체 상태를 매번 다시 그리지 않는다.
- 숫자를 보여주는 모든 위젯은 수치 원장 툴팁을 지원하는 공용 컴포넌트를 쓴다.
- 테마는 CSS 변수 하나의 체계로 관리한다. UI 배율 설정(REQ-UI-06)과 CJK 웹 폰트 자체 호스팅(REQ-LOC-02)을 지원한다.

Three.js WebGPURenderer의 WebGPU 우선·WebGL2 대체 백엔드를 사용해 가용 GPU 가속을 활용한다. 어댑터 미제공, 보안 컨텍스트 제한, 초기화 실패도 대체 경로에서 처리하고 백엔드별 동등한 지도 기능을 검증한다. 셰이더·인스턴싱·버퍼 갱신·압축 텍스처·LOD 등은 실제 지도 렌더링과 성능 측정에 필요한 범위에서 활용하고 결정론 시뮬레이션 계산을 브라우저 GPU로 옮기지 않는다. 브라우저·OS의 보안 및 가속 설정을 자동으로 바꾸지 않는다. 지원되지 않는 환경에는 현지화 안내를 표시한다(REQ-PLAT-03). CI 소프트웨어 렌더링 검증과 실제 GPU 성능 측정은 구분한다.

### 11.4 화면 검증

- Playwright로 고정 시나리오·고정 카메라 스크린샷을 찍고, 기준 이미지와 허용 오차 안에서 비교한다.
- CI의 헤드리스 Chromium에서 WebGL2가 동작하는지(소프트웨어 렌더링 포함)는 WP-08에서 확인하고 ADR로 남긴다.

---

## 12. 작업 패키지와 의존관계

### 12.1 역할

D-15에 따른 역할이다. 모두 Codex다.

| 역할 | 실행 방식 | 하는 일 |
|---|---|---|
| 오케스트레이터 | 사용자가 시작한 Codex 세션 1개 | 계획, 세션 생성·분배, 통합, 문서 갱신, 결정 요청, 오케스트레이터 담당 WP. 제품 코드는 쓰지 않는다 |
| 구현 | Codex 구현 세션 (`tools/orch.sh start impl`, WP마다 1개, 별도 worktree) | 코드 WP 구현과 테스트(P-03, P-06) |
| 콘텐츠 | Codex 구현 세션 (같은 방식) | 데이터 팩 콘텐츠 작성: 클린룸, 출처 기록(P-11) |
| 검증 | Codex 검증 세션 (`tools/orch.sh start verify`, 구현 브랜치의 분리된 worktree) | 독립 검증(P-05, P-12). 추적 파일을 바꾸면 검증 무효 |

**Codex 네이티브 서브에이전트를 쓰지 않는 이유.** Codex 서브에이전트(`.codex/agents/*.toml`)는 병렬로 실행된다. 하지만 공식 문서에 서브에이전트별 작업 폴더(worktree) 분리가 없다. 병렬 구현에 쓰면 같은 작업 폴더를 동시에 고치게 된다. 그래서 구현과 검증은 worktree별 별도 세션으로 돌린다. 쓰기가 없는 조사·검토 같은 보조 작업에는 서브에이전트를 써도 된다.

### 12.2 작업 패키지 표

`병렬 묶음`은 `마일스톤-W번호`다. 같은 묶음 안의 WP는 동시에 진행할 수 있다. 선행 WP는 반드시 더 앞선 묶음이나 이전 마일스톤에 있다.

넓은 REQ의 단계별 완료 범위는 이 WP 표와 01 §10의 해당 마일스톤 AC를 함께 대조한다. REQ-GEN-06의 M0 증거는 WP-04의 `run`과 AC-M0-02의 빈 시나리오 1,000틱 실행이다. 전체 데이터 검증은 WP-24, 재현·벤치 CLI는 WP-25, AI 대전은 AI WP의 후속 증거이며 M0에서 지원된다고 기록하지 않는다. 최종 게이트에는 이 미구현 기능과 담당 후속 WP를 명시한다. 요구사항 식별자·최종 목표와 테스트는 유지한다.

| ID | 작업 | 마일스톤 | 선행 | 병렬 묶음 | 역할 | 대상 REQ | 완료 증거 |
|---|---|---|---|---|---|---|---|
| WP-01 | 저장소 골격: Cargo 워크스페이스, `client/`(Vite+React+TS), `.gitignore`(`.orchestrator/` 포함), CI 매트릭스(서버 3 OS·클라이언트·문서 검사), 라이선스/에셋 검사(cargo-deny, npm) | M0 | — | M0-W1 | 구현 | REQ-PLAT-01, REQ-LEG-02, REQ-LEG-03 | CI 통과, 위반 픽스처 실패 로그 |
| WP-02 | 코어 타입: Fx/Qty, 타입 ID, 시뮬레이션 RNG, 정규 직렬화, 상태 해시 | M0 | WP-01 | M0-W2 | 구현 | REQ-GEN-04, REQ-PLAT-02 | 단위·속성 테스트, 3 OS 해시 일치 |
| WP-03 | 데이터 팩 로더 골격: 매니페스트, defines, 스키마, 파일·줄 오류 보고 | M0 | WP-01 | M0-W2 | 구현 | REQ-GEN-02 | 오류 픽스처 테스트 |
| WP-06 | 에이전트 규약 적용(AGENTS.md·`tools/orch.sh`·템플릿), `orch.sh selftest` 결과 기록, 클린룸 점검표, 차별화 기록 갱신 규칙, 버전 재확인 기록 | M0 | WP-01 | M0-W2 | 오케스트레이터 | REQ-LEG-01, REQ-LEG-05 | `check_docs.py` 통과 로그 |
| WP-04 | 시뮬레이션 루프·스케줄러(§6.2)·명령 큐·헤드리스 CLI `run` | M0 | WP-02 | M0-W3 | 구현 | REQ-GEN-01, REQ-GEN-05, REQ-GEN-06, REQ-TIME-01, REQ-TIME-02 | 1,000틱 해시 2회 일치, 순서 테스트 |
| WP-05 | `oh_proto`(프로토콜 타입·TS 생성)와 `oh_server`(axum WebSocket·정적 파일 제공), 웹 클라이언트 셸(날짜·일시정지·속도) | M0 | WP-04 | M0-W4 | 구현 | REQ-GEN-01, REQ-GEN-05, REQ-TIME-01, REQ-NET-01, REQ-NET-02, REQ-NET-04, REQ-NET-05, REQ-NET-06 | Playwright 스크린샷, 프로토콜 왕복 테스트, TS 생성 검사 |
| WP-07 | 지도 데이터: 비트맵 파서, 인접 생성, 주, VP, Testland 지도 | M1 | WP-03 | M1-W1 | 구현 | REQ-MAP-01, REQ-MAP-02, REQ-MAP-03, REQ-MAP-09 | 인접 골든 테스트, 검증기 통과 |
| WP-10 | 보정치·수치 원장 | M1 | WP-04 | M1-W1 | 구현 | REQ-UI-04 | 원장 일치 속성 테스트 |
| WP-12 | 현지화(`@fluent/bundle`), CJK 웹 폰트 자체 호스팅, UI 셸·테마 | M1 | WP-05 | M1-W1 | 구현 | REQ-LOC-01, REQ-LOC-02 | 언어 전환 스크린샷 |
| WP-08 | 지도 렌더링(Three.js/WebGPU·WebGL2): 인덱스 텍스처, 색 조회, 국경 셰이더, 선택·호버, 카메라, 화면 검증 방식 확정 | M1 | WP-05, WP-07 | M1-W2 | 구현 | REQ-MAP-04, REQ-MAP-05, REQ-PLAT-03 | Playwright 스크린샷 3종 이상, ADR |
| WP-09 | 국가·주 상태, 소유·통제, 지도 모드 데이터 | M1 | WP-04, WP-07 | M1-W2 | 구현 | REQ-NAT-01, REQ-MAP-08 | 단위 테스트 |
| WP-11 | 저장·불러오기 v1 | M1 | WP-02, WP-09 | M1-W3 | 구현 | REQ-SAV-01, REQ-SAV-02 | DT 테스트 |
| WP-13 | 트리거·효과 문법 엔진, 레지스트리, 시나리오 종료 조건 | M2 | WP-03, WP-10 | M2-W1 | 구현 | REQ-AGD-05, REQ-TIME-04 | 단위 테스트, 로드 오류 픽스처 |
| WP-17 | 이동·경로 탐색 | M2 | WP-09 | M2-W1 | 구현 | REQ-MIL-04 | 경로 골든 테스트 |
| WP-24 | 데이터 팩 시스템 v1: 의존·충돌, `oh validate`, 현지화 누락 검사 | M2 | WP-03, WP-12 | M2-W1 | 구현 | REQ-MOD-01, REQ-MOD-02, REQ-MOD-04, REQ-LOC-03 | CLI 테스트, CI 검사 |
| WP-25 | 재현 번들 CLI, 성능 벤치 CI | M2 | WP-04, WP-11 | M2-W1 | 구현 | REQ-SAV-05, REQ-PERF-03 | 번들 재생 테스트, 벤치 기준선 |
| WP-14 | 경제·정치: IC, 배분, 자원, 건설, 인력, PC, 안정도, 동원도, 법령 | M2 | WP-09, WP-10, WP-13 | M2-W2 | 구현 | REQ-ECO-01, REQ-ECO-02, REQ-ECO-04, REQ-ECO-06, REQ-ECO-07, REQ-NAT-02, REQ-NAT-03, REQ-NAT-04 | 단위·골든 테스트 |
| WP-23 | Testland 콘텐츠: 6개국, 장비·대대·건물·defines, 시나리오 | M2 | WP-07, WP-13 | M2-W2 | 콘텐츠 | REQ-CNT-01, REQ-GEN-03 | `oh validate` 통과 |
| WP-15 | 장비·생산 라인 | M2 | WP-14 | M2-W3 | 구현 | REQ-ECO-03, REQ-MIL-01 | 단위·골든 테스트 |
| WP-19 | 보급 | M2 | WP-14, WP-17 | M2-W3 | 구현 | REQ-SUP-01, REQ-SUP-02, REQ-SUP-04 | 단위·골든 테스트 |
| WP-16 | 편제·사단 생성·보충·군·장군 | M2 | WP-15 | M2-W4 | 구현 | REQ-MIL-02, REQ-MIL-03, REQ-MIL-05, REQ-MIL-09, REQ-NAT-05 | 단위 테스트 |
| WP-18 | 전투 | M2 | WP-16, WP-17 | M2-W5 | 구현 | REQ-CMB-01, REQ-CMB-02, REQ-CMB-03, REQ-CMB-04, REQ-CMB-05 | 단위·골든·결정론 테스트 |
| WP-22 | UI v1: 메뉴 흐름, 상단 바, 산업·건설·편제·외교 기본 패널, 원장 툴팁, 재접속 재동기화 | M2 | WP-08, WP-12, WP-14, WP-16 | M2-W5 | 구현 | REQ-UI-01, REQ-UI-02, REQ-UI-04, REQ-NET-03 | Playwright 스크린샷·스크립트 플레이, 끊김 복구 테스트 |
| WP-20 | 전쟁 상태, 선전포고, 항복 진행도, 통제국 이전 | M2 | WP-18 | M2-W6 | 구현 | REQ-DIP-05, REQ-DIP-06 | 단위·골든 테스트 |
| WP-21 | AI v1(경제·생산·군사 기본)과 AI 벤치 하네스 | M2 | WP-15, WP-18, WP-19, WP-20 | M2-W7 | 구현 | REQ-AI-01, REQ-AI-02, REQ-AI-05, REQ-AI-06, REQ-AI-07 | 50회 벤치 리포트 |
| WP-26 | 연구 | M3 | WP-13, WP-15 | M3-W1 | 구현 | REQ-TEC-01, REQ-TEC-02, REQ-TEC-03 | 단위 테스트 |
| WP-27 | 의제·이벤트·결의, 정부 교체 효과, 범용 의제 구조 | M3 | WP-13 | M3-W1 | 구현 | REQ-AGD-01, REQ-AGD-02, REQ-AGD-03, REQ-AGD-04, REQ-NAT-06 | 단위·결정론 테스트 |
| WP-28 | 외교 확장(긴장도·진영·명분·보장·통행권·불가침)과 단순 강화 | M3 | WP-20 | M3-W1 | 구현 | REQ-DIP-01, REQ-DIP-02, REQ-DIP-03, REQ-DIP-04, REQ-DIP-07 | 단위·통합 테스트 |
| WP-29 | 세계 시장, 철도 용량·철도·보급 거점 건설 | M3 | WP-14, WP-19 | M3-W1 | 구현 | REQ-ECO-05, REQ-SUP-03 | 단위 테스트 |
| WP-30 | 전선·공세 계획·방어선(플레이어 명령 + 공용 실행기) | M3 | WP-21 | M3-W1 | 구현 | REQ-MIL-06, REQ-MIL-07, REQ-MIL-08 | 통합 테스트, 스크립트 플레이 |
| WP-31 | 해상 수송 | M3 | WP-17 | M3-W1 | 구현 | REQ-NAV-01 | 단위·통합 테스트 |
| WP-32 | 버전 고정 로컬 지리·DEM 원천/라이선스 필터, 능선·하천·고도 반영 오프라인 지도 파이프라인과 유럽 지도 | M3 | WP-07 | M3-W1 | 구현 | REQ-MAP-07, REQ-MAP-10, REQ-MAP-11 | 해상도/정확도 ADR, 입력/산출 해시 재생성, 원천 누락/불일치 실패, 검증기 통과 |
| WP-35 | 모드 시스템 완성: 패치 연산, 로드 순서, 모드 관리 UI, 데이터 레퍼런스 자동 생성, 예제 모드 2종 | M3 | WP-22, WP-24 | M3-W1 | 구현 | REQ-MOD-03, REQ-MOD-05, REQ-MOD-06, REQ-MOD-07 | 예제 모드 통합 테스트 |
| WP-37 | 자동 저장, 저장 포맷 버전 관리, 브라우저 저장 파일 내려받기·올리기 | M3 | WP-11 | M3-W1 | 구현 | REQ-SAV-03, REQ-SAV-04, REQ-SAV-06 | 마이그레이션 테스트, E2E 테스트 |
| WP-38 | 아트 패스: UI 테마, 부대 아이콘, 대체 디자인 국기 | M3 | WP-22 | M3-W1 | 콘텐츠 | REQ-CNT-04, REQ-LEG-06 | 스크린샷, 국기 검토 기록 |
| WP-33 | 유럽 1939 콘텐츠: 국가·주·인구·자원·부대 배치·의제·이벤트, 출처 기록 | M3 | WP-26, WP-27, WP-32 | M3-W2 | 콘텐츠 | REQ-CNT-02, REQ-CNT-05, REQ-AGD-02 | `oh validate`, SOURCES.md |
| WP-34 | AI v2: 연구·의제·외교·난이도·역사 경로 | M3 | WP-26, WP-27, WP-28, WP-30 | M3-W2 | 구현 | REQ-AI-03, REQ-AI-04, REQ-AI-08, REQ-AGD-06 | 100회 벤치 리포트 |
| WP-36 | UI v2: 연구·의제·정치·알림·전투 상세·LOD·단축키·자동 일시정지 | M3 | WP-22, WP-26, WP-27 | M3-W2 | 구현 | REQ-UI-03, REQ-UI-05, REQ-UI-06, REQ-MAP-06, REQ-CMB-06, REQ-TIME-03 | 스크린샷, 스크립트 플레이 |
| WP-39 | M3 성능 목표 달성, 비제휴 고지 반영, 검토된 AI 이미지 배포 조건 확인(OPEN-09), 릴리스 0.1 패키징(공개 게이트 판정은 P-12) | M3 | WP-29, WP-31, WP-33, WP-34, WP-35, WP-36, WP-37, WP-38 | M3-W3 | 구현 | REQ-PERF-01, REQ-LEG-02, REQ-LEG-04, REQ-LEG-07 | 벤치 결과, 게이트 체크리스트 |
| WP-40 | 공군: 공역·공중 우세·근접 지원 | M4 | WP-39 | M4-W1 | 구현 | REQ-AIR-01, REQ-AIR-02 | 단위·통합 테스트 |
| WP-41 | 해군 확장: 해역 통제·호송 차단·상륙 | M4 | WP-39 | M4-W1 | 구현 | REQ-NAV-02, REQ-NAV-03 | 단위·통합 테스트 |
| WP-42 | 국가 간 무역, 점령지 산출·저항·준수 및 첩보(OPEN-08 포함 확정, REQUEST-0004 세부 결정 후) | M4 | WP-39 | M4-W1 | 구현 | REQ-ECO-08, REQ-ECO-09, REQ-NAT-08 | 통합 테스트 |
| WP-43 | 집단군, 경험·숙련, 내각·고문 | M4 | WP-39 | M4-W1 | 구현 | REQ-MIL-10, REQ-CMB-07, REQ-NAT-07 | 단위 테스트 |
| WP-44 | 보급 차량 소모(OPEN-03/REQUEST-0002), 간소화 평화 회의(OPEN-02/REQUEST-0003), 기상·계절(OPEN-07/REQUEST-0004). 범위 확정, 세부 결정 후 구현 | M4 | WP-39 | M4-W1 | 구현 | REQ-SUP-05, REQ-DIP-08, REQ-GEN-07 | 통합 테스트 |
| WP-45 | 고정 로컬 지리·DEM 원천에서 능선·하천·고도를 반영해 오프라인 생성한 전 세계 지도와 1936 콘텐츠 | M5 | WP-40, WP-41 | M5-W1 | 콘텐츠 | REQ-CNT-03, REQ-MAP-10, REQ-MAP-11 | 지형 지역 검수, 입력/산출 해시 재생성, `oh validate`, 외부 지도 요청 없는 플레이 테스트 |
| WP-47 | 접근성 팔레트, 튜토리얼·도움말 | M5 | WP-39 | M5-W1 | 구현 | REQ-UI-07, REQ-UI-08 | 스크린샷, 플레이 테스트 |
| WP-48 | 차체·모듈 장비 설계, 전략 폭격·세부 공군 임무, 함대 전투、특수 무기(OPEN-01·04·05·11 포함/범위 확정, REQUEST-0002 세부 결정 후), 결정론 샌드박스 모드 스크립트(OPEN-06 도입 확정). 함선 설계는 이후 로드맵 | M5 | WP-40, WP-41 | M5-W1 | 구현 | REQ-MIL-11, REQ-MIL-12, REQ-AIR-03, REQ-NAV-04, REQ-MOD-08 | 단위·통합 테스트 |
| WP-46 | 전 세계 성능 최적화 | M5 | WP-45 | M5-W2 | 구현 | REQ-PERF-02 | 벤치 결과 |
| WP-49 | 멀티플레이: 서버 권위형 게임 세션, 로비·세션 코드, 명령 권한 검증, 이탈 시 AI 인계·재접속, 속도·일시정지 권한 | M6 | WP-46 | M6-W1 | 구현 | REQ-MP-01, REQ-MP-02, REQ-MP-03, REQ-MP-04, REQ-MP-05 | 통합 테스트, 세션 로그 |
| WP-50 | 1.0 안정화: 업적(OPEN-12 범위 확정, REQUEST-0005 세부 결정 후)、회귀 정리, 문서·모드 SDK 정비, 릴리스 준비. 아이언맨 제외 | M6 | WP-49 | M6-W2 | 구현 | REQ-GEN-08 | 업적 단위·통합 테스트, 게이트 체크리스트 |

### 12.3 핵심 경로와 병렬화

- **M0 핵심 경로**: WP-01 → WP-02 → WP-04 → WP-05. WP-03과 WP-06은 WP-02와 동시에 진행한다. Codex 구현 세션 둘이 WP-02와 WP-03을 하는 동안 오케스트레이터는 WP-06을 직접 한다.
- **M2 핵심 경로**: WP-13 → WP-14 → WP-15 → WP-16 → WP-18 → WP-20 → WP-21.
  - 이 경로와 겹치지 않는 WP-17, WP-24, WP-25, WP-23, WP-19, WP-22는 비는 묶음에 배치한다.
  - 콘텐츠(WP-23)는 데이터 형식만 정해지면 시스템 구현과 동시에 진행할 수 있다.
- **M3**: W1에 독립 WP 10개가 있다. 동시에 돌리는 구현 세션 수는 오케스트레이터가 통합·검증할 수 있는 양(권장 3~4개)으로 제한한다. 남은 WP는 대기열로 돌린다.
- **충돌 방지**: 같은 크레이트의 같은 모듈을 고치는 WP는 같은 묶음에 넣지 않는다. 구현 세션은 모두 별도 worktree에서 돌지만, 같은 모듈을 동시에 고치면 병합 충돌 비용이 커진다. 불가피하면 병합 순서를 계획에 적는다.

### 12.4 세션 생성과 병렬 실행 (D-15)

**CEO 자동 진행의 사용자 지정 경로(2026-10-06).** 사용자가 CEO는 앱 채팅으로 오케스트레이터를 열고 오케스트레이터도 앱 채팅으로 구현·검증을 생성하며 서로 메시지를 교환하도록 지시했다. M1 이후 이 작업은 docs/04 §0의 앱 경로를 적용한다. CEO는 오케스트레이터 채팅만 만든다. WP별 전용 Git worktree·새 독립 검증·검증 전후 불변성 규칙은 아래 M0 대체 경로와 같다. CLI 실패를 앱 성공으로 대체 기록하지 않고 모델·인증·샌드박스 설정은 유지한다.

오케스트레이터 Codex 세션은 `tools/orch.sh`로 구현 세션과 검증 세션을 만든다.

**이번 M0 실행의 사용자 승인 대체 경로(2026-10-06).** 실제 하위 CLI가 모델·ChatGPT 인증 조합의 HTTP 400으로 실패했고, 사용자는 현재 앱 설정의 별도 채팅 실행을 선택했다. 이에 Codex 앱 채팅과 WP별 명시적 Git worktree로 동일한 구현/독립 검증 분리를 유지한다. 기본 cwd가 프로젝트 루트인 앱 채팅은 모든 명령을 지정한 전용 worktree에서 실행한다. 검증 전후 HEAD·추적 파일 목록·diff/status를 오케스트레이터가 비교하며 변경되면 무효다. CLI 설정과 도구는 유지한다. 근거와 시험 결과는 환경 실행 ADR(`docs/adr/`에서 확인), 운영 계획은 [M0 계획](plans/M0.md)에 기록한다.

```
tools/orch.sh selftest                                   # 최초 1회: codex 호출·분리 실행 가능 여부
tools/orch.sh start impl   <WP> <브랜치> <프롬프트 파일>   # 구현 세션 분리 실행 후 바로 반환
tools/orch.sh start verify <WP> <브랜치> <프롬프트 파일>   # 검증 세션 분리 실행 후 바로 반환
tools/orch.sh status                                     # 세션 상태 표
tools/orch.sh wait <WP>[:impl|:verify] ... [--timeout 초] # 대체 수단: 끝날 때까지 대기
```

`WP-00`은 `selftest` 전용 예약 번호다. 시험 후 지우며 병합하지 않는다.

**`orch.sh`가 하는 일**

| 단계 | 구현 세션 (`impl`) | 검증 세션 (`verify`) |
|---|---|---|
| 작업 폴더 | `.orchestrator/wt/<WP>`에 WP 브랜치 worktree. 기본 브랜치 HEAD에서 새로 만들거나, 있으면 재사용 | `.orchestrator/wt/<WP>-verify`에 구현 브랜치 최신 커밋을 분리(detached) 체크아웃 |
| 의존성 | `cargo fetch`로 미리 받음(샌드박스 네트워크 차단 대비) | 같음 |
| 실행 | 작업 폴더에서 `codex exec --sandbox workspace-write --json -o <마지막 메시지> -`. 프롬프트는 표준 입력 | 같음 |
| 사후 검사 | — | 추적 파일이 바뀌었으면 검증 무효(exit 4) |

- Codex는 각 worktree 루트의 `AGENTS.md`를 자동으로 읽는다.
- 모든 세션은 새 세션이라 이전 대화를 공유하지 않는다. 검증의 독립성은 여기서 나온다.
- `.orchestrator/`는 `git info/exclude`에 자동 등록된다. WP-01 이후에는 `.gitignore`에도 들어간다.

**결과 파일** (`.orchestrator/` 아래)

| 파일 | 내용 |
|---|---|
| `prompts/<WP>….md` | 오케스트레이터가 작성한 프롬프트 |
| `logs/<WP>.<모드>.{prompt.md,jsonl,err}` | 보낸 프롬프트 사본, 이벤트 스트림, 진행 로그 |
| `out/<WP>.<모드>.last.md` | 세션의 마지막 메시지(검증 리포트 포함) |
| `out/<WP>.<모드>.{exit,state,worktree}` | 종료 코드, 상태, 작업 폴더 경로 |

**오케스트레이터의 운영 규칙**

| 단계 | 규칙 |
|---|---|
| 프롬프트 작성 | P-03·P-05·P-06·P-11의 변수를 채워 `.orchestrator/prompts/`에 저장한다 |
| 병렬 시작 | 같은 병렬 묶음의 WP를 `start impl`로 연달아 시작한다. 구현 세션은 동시에 3~4개 이하 |
| 대기 중 | 기다리지 않는다. `docs/plans/M#.md`의 오케스트레이터 작업을 하고, 작업 단위 사이에 `status`를 확인한다 |
| 구현 종료 | `exit`가 0이 아니거나 작업 로그가 없으면 실패다. P-06으로 다시 `start impl`(같은 WP·브랜치, worktree 재사용) |
| 검증 | 구현이 끝난 WP는 바로 `start verify`(P-05). `out/<WP>.verify.last.md`의 리포트를 `docs/verify/<WP>.md`로 저장한다. exit 4는 무효이므로 새 검증 세션으로 다시 검증한다 |
| 통합 | PASS를 받은 브랜치를 P-07로 병합한다 |
| 정리 | 기본 브랜치 병합이 확인된 worktree만 `git worktree remove`로 지운다 |

**실행 환경 조건** (`orch.sh selftest`로 확인하고 WP-06에서 결과를 ADR로 남긴다)

1. 오케스트레이터 세션 안에서 하위 `codex exec`가 OpenAI API에 접속할 수 있어야 한다.
   - 오케스트레이터 세션의 샌드박스가 네트워크를 막으면 하위 세션이 실패한다.
   - 이 경우 사용자가 오케스트레이터를 네트워크가 허용된 설정으로 시작해야 한다. 설정 변경은 사용자 확인 사항이다(AGENTS.md §3-3).
2. 분리 실행(`start`)한 프로세스가 오케스트레이터 명령이 끝난 뒤에도 살아 있어야 한다.
   - `selftest`의 5단계로 확인한다.
   - 살아남지 못하는 환경이면 대체 수단을 쓴다: 한 묶음의 `start`를 연달아 실행한 직후 같은 명령에서 `wait`로 기다린다. 이 경우 구현 세션끼리는 병렬로 돌지만, 오케스트레이터는 그동안 다른 일을 하지 못한다. 그 사실을 계획에 적는다.
3. 공식 설정 문서에서 `sandbox_workspace_write.network_access`는 workspace-write 외부 네트워크 허용 boolean 키다(2026-10-06 재확인). 이번 실행은 설정을 변경하지 않았다. CLI 모델 거절과 네트워크 차단을 구분하며, 하위 채팅의 실제 권한은 실행 문맥에서 확인한다. 결과는 환경 실행 ADR에 기록한다.
4. 인증 방식(로그인 또는 `CODEX_API_KEY`)과 사용량 한도도 확인한다.

---

## 13. 테스트 전략

| 종류 | 도구·위치 | 필수 대상 |
|---|---|---|
| 단위 | `cargo test`, 각 크레이트 `tests/` | 모든 공식(`oh_sim::formula`), 명령 검증, 로더 |
| 속성 | `proptest` | 고정소수점 연산 경계, 원장 일치, 직렬화 왕복, 경로 탐색 |
| 골든 | `tests/golden/*.toml` | Testland N일 실행 후 상태 해시와 주요 지표. 인접 생성, 경로, 전투 사례 |
| 결정론 | `oh run`의 해시 출력, CI 비교 작업 | 같은 시드 2회, 저장 후 재개, 3 OS |
| 데이터 | `oh validate --deny-warnings` | 모든 팩 |
| 클라이언트 단위 | Vitest | 델타 적용 스토어, 프로토콜 디코딩, 지도 인덱스 조회 |
| 프로토콜 계약 | `ts-rs` 생성물 최신 여부, Rust 인코딩 → TS 디코딩 왕복 | 모든 메시지 타입 |
| 화면·E2E | Playwright(§11.4) | 지도 모드·국경·패널 스크린샷, 접속·명령·재접속 흐름 |
| 스크립트 플레이 | 재현 번들 형식의 명령 시나리오 | AC-M2-01 같은 완주 경로 |
| AI 벤치 | `oh ai-bench` | REQ-AI-07 |
| 성능 | `oh bench` | REQ-PERF-01·02·03 |
| 퍼징 | `cargo fuzz`(가능할 때) | 데이터 로더, 저장 파일 파서 |

**골든 기대값 변경 규칙.** 골든 기대값은 `oh golden update --reason "<WP와 규칙 변경 이유>"`로만 바꾼다. 이 명령은 변경 이유를 `tests/golden/CHANGELOG.md`에 기록한다. 테스트를 통과시키려고 기대값만 바꾸는 것은 금지다. 검증 세션은 CHANGELOG의 이유와 실제 규칙 변경이 맞는지 확인한다.

---

## 14. 빌드·CI·배포

### 14.1 CI (GitHub Actions)

| 작업 | 실행 조건 | 내용 |
|---|---|---|
| lint | 모든 푸시 | `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, 클라이언트 타입 검사(`tsc --noEmit`) |
| test-linux, test-windows | 모든 푸시 | `cargo test --workspace` |
| client | 모든 푸시 (Linux) | `npm ci`, 프로토콜 TS 생성 후 변경 없음 확인, Vitest, `vite build` |
| e2e | 모든 푸시 (Linux) | 서버 실행 + Playwright(Chromium) 스크린샷·흐름 테스트 |
| test-macos | 비공개 기간: 야간·릴리스 / 공개 후: 모든 푸시 | 위와 같음 |
| determinism | 테스트 작업 후 | 각 OS에서 `oh run --scenario testland --days 365 --seed 1 --hash-out` 실행 후 해시 비교 |
| licenses | 모든 푸시 | `cargo deny check licenses bans advisories`, npm 의존성 라이선스 검사(도구는 WP-01에서 최신 확인) |
| assets | 모든 푸시 | `tools/check_assets.py`: 매니페스트 누락과 AI 생성 표시를 검사 |
| docs | 문서 변경 시 | `python3 tools/check_docs.py` |
| data | 데이터 변경 시 | `oh validate --deny-warnings data/packs/*` |
| bench | 기본 브랜치·PR | Linux에서 기준 커밋과 현재 커밋을 같은 러너에서 연속 실행. 15% 넘게 느려진 결과가 2회 재현되면 실패 |
| release | 태그 | 클라이언트 번들을 포함한 서버 실행 파일을 3 OS로 빌드, 체크섬 |

**CI 비용 (D-13, D-14)**

- 공개 저장소에서는 표준 GitHub 호스팅 러너가 무료다. 사용자의 2026-10-06 추가 지시로 OpenHOI4 저장소를 공개했고 D-14를 갱신했으므로, 현재 실행은 macOS 포함 매 푸시 CI를 적용한다. 공식 재확인 출처: https://docs.github.com/en/billing/concepts/product-billing/github-actions
- 비공개 저장소는 GitHub Free 월 2,000분, Pro 월 3,000분이 포함된다.
- 분당 단가는 Linux 2코어 $0.006, Windows $0.010, macOS $0.062다(2026-10 GitHub 문서 기준).
- 그래서 비공개 기간에는 macOS 작업을 야간 1회로 제한한다.

### 14.2 개발 명령

```
cargo test --workspace
cargo run -p oh_cli -- run --scenario testland --days 365 --seed 1 --hash-out
cargo run -p oh_cli -- validate data/packs/testland
cargo run -p oh_cli -- ai-bench --scenario testland --runs 50 --days 1500 --seed-base 1000 --out target/ai.json
cargo run -p oh_server -- --open          # 로컬 서버 실행 후 브라우저 열기
npm --prefix client ci && npm --prefix client test
npm --prefix client run dev               # 클라이언트 개발 서버(서버 /ws에 프록시)
npx --prefix client playwright test
python3 tools/check_docs.py
```

### 14.3 허용 라이선스 (REQ-LEG-03)

**코드 의존성 허용 목록:** `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `MPL-2.0`, `Unicode-3.0`, `BSL-1.0`, `CC0-1.0`.

사용자 2026-10-06 답변으로 D-10은 코드 GPL-3.0-or-later, 직접 제작 데이터·에셋 CC BY-SA 4.0으로 확정됐다. 기존 코드 의존성 허용 목록은 유지한다. GPL 계열 의존성을 새로 추가하려면 이 목록과 실제 호환성을 별도 검토하며 이번 결정만으로 허용 목록을 확대하지 않는다. 외부 폰트·자료는 원래 라이선스와 고지를 유지한다.

**폰트:** `OFL-1.1`.

**에셋·데이터 허용:** `CC0-1.0`, `CC-BY-4.0`, `CC-BY-SA-4.0`, `OFL-1.1`, 퍼블릭 도메인, 직접 제작.

### 14.4 에셋 매니페스트 (REQ-LEG-02)

`assets/ASSETS.toml`에 배포 에셋마다 항목을 하나씩 둔다.

```toml
[[asset]]
path = "data/packs/base/gfx/flags/POL.png"
author = "OpenHOI contributors"
source = "original"
license = "CC-BY-SA-4.0"     # D-10: 직접 제작 데이터·에셋의 확정 라이선스
modified = false
ai_generated = false         # OPEN-09: true이면 출처·도구·이용 조건·배포 검토 기록 필수
notes = ""
```

### 14.5 릴리스 절차 (M3 이후)

OPEN-09 사용자 결정은 검토된 AI 이미지에 한해서 배포물 포함을 허용한다. 생성 도구·출처·이용 조건·검토자·검토일·검토 증거를 매니페스트에 기록해야 하며, 미검토·누락은 릴리스 검사 실패다. 현재 일괄 거부 검사에서 이 조건으로 바꾸는 것은 WP-01 후속 구현·독립 검증으로 수행하고 실제 개별 배포 검토는 WP-39 공개 게이트에서 다시 확인한다. 공개 배포·태그 승인은 별도다.

1. 공개 게이트(AC-M3-06) 체크리스트를 통과한다.
2. 버전 태그를 붙인다.
3. CI가 내보내기와 체크섬을 만든다.
4. 변경 기록과 서드파티 고지 파일(의존성 라이선스 목록)을 포함한다.
5. 다음 비제휴 고지를 넣는다.

   > 이 프로젝트는 Paradox Interactive와 무관하며, 언급된 상표는 각 소유자의 것이다.

---

## 15. 완료 판정 규칙

**작업 패키지 완료 조건 (셋 다 필요하다)**

1. **구현 증거**: 아래 증거 종류 중 WP 표의 `완료 증거`에 해당하는 것. 경로, 명령, 출력, 종료 코드가 남아 있어야 한다.
2. **독립 검증 통과**: 구현에 참여하지 않은 새 Codex 검증 세션이 03 P-05로 검증하고 `PASS`를 낸다. 검증 세션이 추적 파일을 바꾸지 않았음을 `orch.sh`가 확인한 경우에만 유효하다.
3. **통합 확인**: 오케스트레이터가 기본 브랜치에 병합한 뒤 CI가 녹색이다.

**증거 종류**

| 코드 | 증거 |
|---|---|
| E1 | 실행한 명령, 종료 코드, 출력 발췌(로그 파일 경로) |
| E2 | 테스트 이름 목록과 결과. 새 테스트는 구현 전에 실패하고 구현 후에 통과한 기록 |
| E3 | 스크린샷 파일과 캡처 조건(시나리오, 날짜, 카메라) |
| E4 | 벤치·AI 리포트 JSON과 기준선 비교 |
| E5 | 재현 번들과 재생 결과 |
| E6 | 검증 세션 리포트 |

**증거가 아닌 것**

- "구현했습니다", "테스트가 통과할 것입니다" 같은 자기 보고
- 실행하지 않은 코드 리뷰만으로 내린 판단
- 기대값을 바꿔서 통과시킨 테스트(§13 규칙 위반)

작업 로그는 `docs/worklog/WP-NN.md`에 남긴다(템플릿: `docs/templates/WORKLOG_TEMPLATE.md`).

---

## 16. 조사 근거 (2026-10-06 확인)

| 주제 | 출처 |
|---|---|
| HOI4 최신 패치 1.19.3(2026-09-17) | https://happygamer.com/hoi4-patch-1193-thunder-at-our-gates-final-balance-fixes-164793/ |
| HOI4 DLC 목록 | https://store.steampowered.com/dlc/394360/Hearts_of_Iron_IV/ , https://en.wikipedia.org/wiki/Hearts_of_Iron_IV |
| Godot C# 웹 내보내기 불가, godot-rust Wasm 실험 단계(대안 검토) | https://github.com/godotengine/godot-docs (tutorials/scripting/c_sharp/c_sharp_basics.rst) , https://github.com/godot-rust/gdext |
| OpenVic 구조·라이선스·이중 모드 | https://github.com/OpenVicProject/OpenVic , https://github.com/OpenVicProject/OpenVic-Simulation |
| 프로빈스 지도 렌더링(Imperator: Rome) | https://www.intel.com/content/www/us/en/developer/articles/technical/optimized-gradient-border-rendering-in-imperator-rome.html |
| Rust 1.99.0 | https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/ |
| bincode 유지보수 중단 | https://rustsec.org/advisories/RUSTSEC-2025-0141.html |
| 크레이트 버전 | https://crates.io (tokio, axum, rmp-serde, ts-rs, fixed, rand_chacha, serde, postcard, toml, schemars, zstd, proptest, cargo-deny) |
| 클라이언트 패키지 버전 | https://registry.npmjs.org (typescript, vite, react, three, @msgpack/msgpack, @fluent/bundle, vitest, @playwright/test) |
| "HEARTS OF IRON" 미국 상표 | https://www.trademarkia.com/hearts-of-iron-85610019 |
| 동명 저장소 openHOI | https://github.com/Minecraft3193092/openHOI |
| Paradox 사용자 약관(UGC·비상업) | https://store.steampowered.com/eula/949230_eula_1?eulaLang=english |
| HOI4 위키 라이선스(CC BY-SA 3.0) | https://hoi4.paradoxwikis.com/Hearts_of_Iron_4_Wiki |
| 게임 규칙과 저작권(미국) | https://www.copyright.gov/fls/fl108.pdf |
| 게임 규칙 조합의 보호(한국, 2017다212095) | https://www.legaltimes.co.kr/news/articleView.html?idxno=47467 |
| Natural Earth 이용 조건 | https://www.naturalearthdata.com/about/terms-of-use/ |
| OpenHistoricalMap 재사용(CC0) | https://wiki.openstreetmap.org/wiki/OpenHistoricalMap/Reuse |
| CShapes 2.0(CC BY-NC-SA) | https://icr.ethz.ch/data/cshapes/ |
| historical-basemaps(GPL-3.0) | https://github.com/aourednik/historical-basemaps |
| GitHub Actions 과금 | https://docs.github.com/en/billing/concepts/product-billing/github-actions |
| Codex 서브에이전트(`.codex/agents/*.toml`, 병렬, 서브에이전트별 sandbox_mode) | https://learn.chatgpt.com/docs/agent-configuration/subagents |
| Codex 비대화형 실행(`codex exec`, `-`, `--sandbox`, `--json`, `-o`, git 저장소 요구) | https://learn.chatgpt.com/docs/non-interactive-mode |
| Codex의 AGENTS.md 탐색(루트→현재 폴더, 기본 32 KiB 한도) | https://learn.chatgpt.com/docs/agent-configuration/agents-md |

**확인이 덜 된 항목**

- 한국 상표(KIPRIS)는 조회하지 못했다.
- Paradox 현행 약관의 역설계 조항은 원문을 직접 확인해야 한다.
- OpenVic이 주장하는 Paradox 승인은 Paradox 측 공식 확인이 없다.

---

## 17. 위험 목록

| 위험 | 영향 | 대응 |
|---|---|---|
| 브라우저 GPU 성능·호환 차이 | 지도 프레임 저하, 브라우저별 버그 | WebGPU 우선·WebGL2 대체, 초기화 실패 처리, 세 브라우저 엔진 회귀 검사와 제품별 수동 확인을 게이트에 포함 |
| 델타 대역폭 | 최고 속도에서 지연 | 묶음 전송(최대 10회/초), 변경분만 전송, REQ-PERF-01 측정 |
| 공개 호스팅 비용 | 예산 0과 충돌(D-13) | 로컬·자가 호스팅만(OPEN-10) |
| 플랫폼 간 결정론 붕괴 | 멀티·재현 불가 | DR 규칙, 매 커밋 3 OS 해시 비교 |
| 지도 제작 노동량 | M3·M5 지연 | WP-32 도구 우선, 지도 작업을 시스템 작업과 병렬로 진행 |
| AI 품질 | 재미 저하 | 벤치 지표, 플레이어와 공용 실행기, 단계적 개선 |
| 원작 구성 재현에 따른 법적 위험(한국 판례) | 배포 중단 | 차별화 기록(01 §11), 클린룸 점검 |
| 상표(명칭) | 이름 변경 비용 | 코드네임 유지, 공개 전 D-12 확정 |
| 사용자에게 Rust가 익숙하지 않음 | 리뷰 부담 | 증거 기반 검증 체계, 공식 모듈 집중, ADR |
| 비공개 기간 CI 비용 | 무료 한도 초과 | macOS 야간 실행, 캐시 사용 |
| Codex 샌드박스 네트워크 차단 | 의존성 추가 WP의 빌드 실패 | 세션 생성 전 `cargo fetch`, 새 의존성은 작업 로그로 요청, 네트워크 설정은 사용자 확인 후 변경(§12.4) |
| 병렬 세션 간 병합 충돌 | 통합 지연 | 같은 모듈을 고치는 WP를 같은 묶음에 넣지 않음(§12.3), 병합 순서 계획 |
| 유럽 지도(M3)와 전 세계 지도(M5)가 따로 존재 | 이중 유지 | M5에서 유럽 시나리오를 전 세계 지도로 다시 기반화할지 결정(M5 착수 시 사용자 확인) |

---

## 부록 A. 추적 매트릭스

`tools/check_docs.py --write-trace`가 01 §8과 02 §12.2에서 자동 생성한다. 손으로 고치지 않는다.

<!-- TRACE:BEGIN -->
| REQ | 마일스톤 | 우선순위 | 구현 WP | 검증 |
|---|---|---|---|---|
| REQ-GEN-01 | M0 | 필수 | WP-04, WP-05 | CI |
| REQ-GEN-02 | M0 | 필수 | WP-03 | RV, DV |
| REQ-GEN-03 | M2 | 필수 | WP-23 | IT, DV |
| REQ-GEN-04 | M0 | 필수 | WP-02 | DT |
| REQ-GEN-05 | M0 | 필수 | WP-04, WP-05 | UT, RV |
| REQ-GEN-06 | M0 | 필수 | WP-04 | IT |
| REQ-GEN-07 | M4 | 필수 | WP-44 | UT, IT |
| REQ-GEN-08 | M6 | 필수 | WP-50 | UT, IT |
| REQ-LEG-01 | M0 | 필수 | WP-06 | RV |
| REQ-LEG-02 | M0 | 필수 | WP-01, WP-39 | CI |
| REQ-LEG-03 | M0 | 필수 | WP-01 | CI |
| REQ-LEG-04 | M3 | 필수 | WP-39 | RV |
| REQ-LEG-05 | M2 | 필수 | WP-06 | RV |
| REQ-LEG-06 | M3 | 필수 | WP-38 | RV |
| REQ-LEG-07 | M3 | 필수 | WP-39 | RV |
| REQ-TIME-01 | M1 | 필수 | WP-04, WP-05 | UT, SS |
| REQ-TIME-02 | M1 | 필수 | WP-04 | UT, DT |
| REQ-TIME-03 | M3 | 권장 | WP-36 | PT |
| REQ-TIME-04 | M2 | 필수 | WP-13 | IT, DV |
| REQ-MAP-01 | M1 | 필수 | WP-07 | UT, DV |
| REQ-MAP-02 | M1 | 필수 | WP-07 | GT |
| REQ-MAP-03 | M1 | 필수 | WP-07 | DV |
| REQ-MAP-04 | M1 | 필수 | WP-08 | SS |
| REQ-MAP-05 | M1 | 필수 | WP-08 | SS |
| REQ-MAP-06 | M3 | 필수 | WP-36 | SS |
| REQ-MAP-07 | M3 | 필수 | WP-32 | IT, DV |
| REQ-MAP-08 | M2 | 필수 | WP-09 | UT, SS |
| REQ-MAP-09 | M2 | 필수 | WP-07 | DV, SS |
| REQ-MAP-10 | M3 | 필수 | WP-32, WP-45 | IT, DV |
| REQ-MAP-11 | M3 | 필수 | WP-32, WP-45 | IT, DV, RV |
| REQ-NAT-01 | M1 | 필수 | WP-09 | DV |
| REQ-NAT-02 | M2 | 필수 | WP-14 | UT |
| REQ-NAT-03 | M2 | 필수 | WP-14 | UT |
| REQ-NAT-04 | M2 | 필수 | WP-14 | UT, IT |
| REQ-NAT-05 | M2 | 필수 | WP-16 | UT, DV |
| REQ-NAT-06 | M3 | 권장 | WP-27 | IT |
| REQ-NAT-07 | M4 | 후순위 | WP-43 | UT |
| REQ-NAT-08 | M4 | 필수 | WP-42 | UT, IT |
| REQ-ECO-01 | M2 | 필수 | WP-14 | UT |
| REQ-ECO-02 | M2 | 필수 | WP-14 | UT, PT |
| REQ-ECO-03 | M2 | 필수 | WP-15 | UT, GT |
| REQ-ECO-04 | M2 | 필수 | WP-14 | UT |
| REQ-ECO-05 | M3 | 필수 | WP-29 | UT |
| REQ-ECO-06 | M2 | 필수 | WP-14 | UT, IT |
| REQ-ECO-07 | M2 | 필수 | WP-14 | UT |
| REQ-ECO-08 | M4 | 후순위 | WP-42 | IT |
| REQ-ECO-09 | M4 | 후순위 | WP-42 | IT |
| REQ-TEC-01 | M3 | 필수 | WP-26 | DV, UT |
| REQ-TEC-02 | M3 | 필수 | WP-26 | UT |
| REQ-TEC-03 | M3 | 필수 | WP-26 | UT, IT |
| REQ-AGD-01 | M3 | 필수 | WP-27 | UT, DV |
| REQ-AGD-02 | M3 | 필수 | WP-27, WP-33 | DV |
| REQ-AGD-03 | M3 | 필수 | WP-27 | UT, DT |
| REQ-AGD-04 | M3 | 필수 | WP-27 | UT |
| REQ-AGD-05 | M2 | 필수 | WP-13 | UT, DV |
| REQ-AGD-06 | M3 | 권장 | WP-34 | BM |
| REQ-DIP-01 | M3 | 필수 | WP-28 | UT |
| REQ-DIP-02 | M3 | 필수 | WP-28 | UT, IT |
| REQ-DIP-03 | M3 | 필수 | WP-28 | UT |
| REQ-DIP-04 | M3 | 필수 | WP-28 | UT |
| REQ-DIP-05 | M2 | 필수 | WP-20 | UT, IT |
| REQ-DIP-06 | M2 | 필수 | WP-20 | UT, GT |
| REQ-DIP-07 | M3 | 필수 | WP-28 | IT |
| REQ-DIP-08 | M4 | 후순위 | WP-44 | IT |
| REQ-MIL-01 | M2 | 필수 | WP-15 | UT |
| REQ-MIL-02 | M2 | 필수 | WP-16 | UT, GT |
| REQ-MIL-03 | M2 | 필수 | WP-16 | UT |
| REQ-MIL-04 | M2 | 필수 | WP-17 | UT, GT |
| REQ-MIL-05 | M2 | 필수 | WP-16 | UT |
| REQ-MIL-06 | M3 | 필수 | WP-30 | IT, PT |
| REQ-MIL-07 | M3 | 필수 | WP-30 | IT, PT |
| REQ-MIL-08 | M3 | 권장 | WP-30 | IT |
| REQ-MIL-09 | M2 | 필수 | WP-16 | UT |
| REQ-MIL-10 | M4 | 후순위 | WP-43 | UT |
| REQ-MIL-11 | M5 | 필수 | WP-48 | UT, IT |
| REQ-MIL-12 | M5 | 필수 | WP-48 | UT, IT |
| REQ-CMB-01 | M2 | 필수 | WP-18 | UT |
| REQ-CMB-02 | M2 | 필수 | WP-18 | UT |
| REQ-CMB-03 | M2 | 필수 | WP-18 | UT, GT, DT |
| REQ-CMB-04 | M2 | 필수 | WP-18 | UT |
| REQ-CMB-05 | M2 | 필수 | WP-18 | UT, IT |
| REQ-CMB-06 | M3 | 필수 | WP-36 | SS |
| REQ-CMB-07 | M4 | 후순위 | WP-43 | UT |
| REQ-SUP-01 | M2 | 필수 | WP-19 | UT |
| REQ-SUP-02 | M2 | 필수 | WP-19 | UT, GT |
| REQ-SUP-03 | M3 | 필수 | WP-29 | UT, IT |
| REQ-SUP-04 | M2 | 필수 | WP-19 | UT |
| REQ-SUP-05 | M4 | 후순위 | WP-44 | UT |
| REQ-AIR-01 | M4 | 필수 | WP-40 | UT |
| REQ-AIR-02 | M4 | 필수 | WP-40 | UT |
| REQ-AIR-03 | M5 | 후순위 | WP-48 | UT |
| REQ-NAV-01 | M3 | 필수 | WP-31 | UT, IT |
| REQ-NAV-02 | M4 | 필수 | WP-41 | UT |
| REQ-NAV-03 | M4 | 필수 | WP-41 | IT |
| REQ-NAV-04 | M5 | 후순위 | WP-48 | UT |
| REQ-AI-01 | M2 | 필수 | WP-21 | IT |
| REQ-AI-02 | M2 | 필수 | WP-21 | IT, BM |
| REQ-AI-03 | M3 | 필수 | WP-34 | IT, BM |
| REQ-AI-04 | M3 | 필수 | WP-34 | IT, BM |
| REQ-AI-05 | M2 | 필수 | WP-21 | IT, BM |
| REQ-AI-06 | M2 | 필수 | WP-21 | DT, BM |
| REQ-AI-07 | M2 | 필수 | WP-21 | BM, CI |
| REQ-AI-08 | M3 | 권장 | WP-34 | UT |
| REQ-UI-01 | M2 | 필수 | WP-22 | PT, SS |
| REQ-UI-02 | M2 | 필수 | WP-22 | SS |
| REQ-UI-03 | M3 | 필수 | WP-36 | SS, PT |
| REQ-UI-04 | M2 | 필수 | WP-10, WP-22 | UT, SS |
| REQ-UI-05 | M3 | 필수 | WP-36 | SS |
| REQ-UI-06 | M3 | 필수 | WP-36 | SS, PT |
| REQ-UI-07 | M5 | 권장 | WP-47 | SS |
| REQ-UI-08 | M5 | 권장 | WP-47 | PT |
| REQ-MOD-01 | M2 | 필수 | WP-24 | UT, DV |
| REQ-MOD-02 | M2 | 필수 | WP-24 | IT |
| REQ-MOD-03 | M3 | 필수 | WP-35 | UT |
| REQ-MOD-04 | M2 | 필수 | WP-24 | UT, DV |
| REQ-MOD-05 | M3 | 필수 | WP-35 | SS, PT |
| REQ-MOD-06 | M3 | 필수 | WP-35 | IT |
| REQ-MOD-07 | M3 | 필수 | WP-35 | CI, RV |
| REQ-MOD-08 | M5 | 후순위 | WP-48 | UT |
| REQ-SAV-01 | M1 | 필수 | WP-11 | UT |
| REQ-SAV-02 | M1 | 필수 | WP-11 | DT |
| REQ-SAV-03 | M3 | 필수 | WP-37 | UT |
| REQ-SAV-04 | M3 | 필수 | WP-37 | UT |
| REQ-SAV-05 | M2 | 필수 | WP-25 | DT, IT |
| REQ-SAV-06 | M3 | 필수 | WP-37 | IT, PT |
| REQ-MP-01 | M6 | 필수 | WP-49 | IT, PT |
| REQ-MP-02 | M6 | 필수 | WP-49 | IT |
| REQ-MP-03 | M6 | 필수 | WP-49 | UT, IT |
| REQ-MP-04 | M6 | 필수 | WP-49 | IT |
| REQ-MP-05 | M6 | 필수 | WP-49 | IT |
| REQ-PERF-01 | M3 | 필수 | WP-39 | BM |
| REQ-PERF-02 | M5 | 필수 | WP-46 | BM |
| REQ-PERF-03 | M2 | 필수 | WP-25 | BM, CI |
| REQ-PLAT-01 | M0 | 필수 | WP-01 | CI |
| REQ-PLAT-02 | M0 | 필수 | WP-02 | DT, CI |
| REQ-PLAT-03 | M1 | 필수 | WP-08 | IT, SS |
| REQ-NET-01 | M0 | 필수 | WP-05 | IT |
| REQ-NET-02 | M1 | 필수 | WP-05 | IT, PT |
| REQ-NET-03 | M2 | 필수 | WP-22 | IT |
| REQ-NET-04 | M0 | 필수 | WP-05 | CI |
| REQ-NET-05 | M0 | 필수 | WP-05 | RV |
| REQ-NET-06 | M0 | 필수 | WP-05 | IT, PT |
| REQ-LOC-01 | M1 | 필수 | WP-12 | CI, SS |
| REQ-LOC-02 | M1 | 필수 | WP-12 | RV, SS |
| REQ-LOC-03 | M2 | 필수 | WP-24 | CI |
| REQ-CNT-01 | M2 | 필수 | WP-23 | DV, PT |
| REQ-CNT-02 | M3 | 필수 | WP-33 | DV, PT |
| REQ-CNT-03 | M5 | 필수 | WP-45 | DV, PT |
| REQ-CNT-04 | M3 | 필수 | WP-38 | SS, RV |
| REQ-CNT-05 | M3 | 필수 | WP-33 | RV |
<!-- TRACE:END -->


### 웹 렌더링 기술 추가 확인 (2026-10-06 사용자 지시)

WebGPURenderer는 WebGPU 백엔드를 우선 선택하고 WebGL2 백엔드로 대체할 수 있다. 공식 문서: https://threejs.org/docs/pages/WebGPURenderer.html . WebGL은 브라우저에서 OpenGL ES 기반 그래픽을 제공하며 OpenGL을 별도 웹 API로 직접 호출하는 구조는 쓰지 않는다. 출처: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API . WebGPU 가용 여부·보안 컨텍스트와 초기화 조건은 https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API 를 확인한다. 구현·검증은 WP-08/M1에 배정하고 M0 셸 검증과 구분한다.

셰이더 백엔드의 공식 근거: https://threejs.org/docs/pages/WGSLNodeBuilder.html , https://threejs.org/docs/pages/GLSLNodeBuilder.html . WebGPU와 WebGL2에서 동일한 지도 알고리즘을 검증하며, 원시 GLSL을 WebGPU에 그대로 전달하는 방식으로 처리하지 않는다.
