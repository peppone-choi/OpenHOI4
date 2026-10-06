# ADR-0901 M1 국가 상태·조회·정규 해시 경계

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-09, REQ-NAT-01, REQ-MAP-08, AC-M1-03, DR-01~DR-10 |

## 맥락
02 §5.7은 시나리오 국가·주 소유·통제 입력을 정하지만 M1 최소 스키마와 조회 wire를 정하지 않았다. 기존 M0 시간 골든과 빈 Testland defines 테스트를 보존해야 한다. WP-07 지도, WP-10 원장, WP-12 Fluent와 표시 컴포넌트를 실제 서버 상태로 연결하고 WP-08·WP-11에 읽기 계약을 제공해야 한다.

## 결정
- `scenarios/<id>/scenario.toml`의 국가 태그 목록, 주→소유국, 프로빈스→통제국 override를 검증한다. 초기 국가 파일은 같은 시나리오의 `nations/<TAG>.toml`이다. 초기에는 두 합성 국가와 기존 두 주만 쓴다. 정부·이념은 현지화 키이며 새 정부 효과·정치 규칙은 없다.
- 이념 비율은 TOML 문자열에서 I32F32로 직접 파싱한다. 각 값은 0..1, 비트 단위 합은 1이다. 이는 지지율의 저장 단위 선택이다. schema 타입/색 범위·길이와 실제 수도·국가·주·프로빈스 참조도 검사한다.
- root defines 위에 `scenarios/<id>/defines.toml`을 system/item별로 덮어쓴다. M1 fixture는 기존 M0의 명세 time/network 값만 재사용한다. root Testland defines·M0 pack 경로·골든은 그대로다. map `visuals.toml`은 state/terrain/neutral RGB만 정의하며 게임 효과가 없다.
- sim의 `World`는 ID순 Vec와 typed IDs, private 필드와 읽기 accessor를 쓴다. 불변 `Defs`는 Arc로 공유하고 정규 직렬화에서 제외한다. 해시에 불변 정의 identity와 전체 mutable world를 포함한다. 주 소유국과 각 프로빈스 소유/통제국을 따로 보관한다. 무소유 sea/lake는 Option None이다.
- 실제 주 인프라 base와 데이터 modifier를 WP-10 `StatLedger`로 계산해 매 snapshot tick에 적용하고 `verify_applied_value`를 검사한다. 실패하면 시뮬레이션 clock·world·queue 전체가 보존된다. 기본 팩에는 임의 보정치를 넣지 않는다. Add/Mul/expiry/overflow 검사는 테스트 fixture에서만 사용한다.
- `Simulation` 정규 직렬화는 M0 world None 필드를 생략하고 M1 Some을 포함한다. `Simulation::state_hash`와 `oh_core::state_hash(&Simulation)`이 같은 경계를 쓴다. 시간·설정·pending queue, 국가 정부/지지율, 주 인구/자원/건물/base/modifier(만료 포함)/적용값/원장, 프로빈스 소유/통제와 정의 identity를 해시한다.
- 기존 protocol version `m0-v1`과 모든 기존 variant·fixture 12개를 유지한다. Query(kind world/nation:<ID>/state:<ID>)는 새 `WorldResult`를 받는다. 서버는 simulation thread에 조회를 보내 그 시점의 world와 ledger를 함께 만든다. 표시 변환은 oh_proto뿐이고 TS는 생성물이다. runtime guard는 모든 새 nested 필드/색/ID/숫자 wire 범위를 검사한다.
- 기본 서버는 `data/packs/testland`의 m1을 로드한다. 명시 `--pack-root data/packs/examples/m0`는 M0다. CLI 기본 M0 run은 유지하고 `--pack data/packs/testland --scenario m1`을 명시할 때 국가 상태를 로드한다.
- ko/en map/national Fluent 파일은 로컬 `/pack/localisation/<language>/<file>`에서 제공한다. client는 두 언어를 검증한 뒤 번들을 등록한다. `NationalPanels`는 WorldView와 선택 ID props, `ledgerProps`는 서버 문자열·행 순서 그대로를 기존 LedgerValue에 전달한다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| M0 상태 타입에 국가 기본값을 강제로 넣기 | 단일 초기 경로 | 기존 골든과 빈 예제 계약을 깨뜨린다 |
| server에서 정적 fixture 값을 패널에 넣기 | 연결이 짧다 | 실제 private simulation과 해시·원장·재개 경계를 검사하지 못한다 |
| 브라우저에서 팔레트·인프라 계산 | client 구현이 짧다 | Rust 권위 상태와 표시값을 분리시키고 REQ-NET-05에 맞지 않는다 |

## 결과와 영향
WP-11은 State/TimeConfig/순서가 있는 pending commands와 WorldInputs의 모든 값을 저장하고 같은 pack/definitions identity를 확인한 뒤 saved tick의 원장을 재생성해야 한다. Defs·지도 캐시·로컬 파일 경로·OS clock은 저장하지 않는다. 복원 API·파일 포맷은 WP-11 소유다. WP-08은 `province_ids[dense_index]`로 실제 ID를 얻고 서버의 owner/controller/state/terrain RGB를 사용한다. 지도 index HTTP와 GPU 렌더링은 이 ADR의 구현 범위 밖이다.

새 외부 의존성·폰트·버전을 추가하지 않았다. 기존 workspace oh_core/oh_data 의존 연결만 추가했다. 신규 직접 제작 fixture/번역/팔레트는 D-10의 CC-BY-SA-4.0으로 ASSETS에 등록했다. 실제 3 OS 결과는 main CI가 실행해 확인할 때까지 대기다.
