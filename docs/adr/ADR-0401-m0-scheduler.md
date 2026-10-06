# ADR-0401 M0 시간·명령 펌프와 독립 빈 시나리오

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-04, REQ-GEN-01, REQ-GEN-05, REQ-GEN-06, REQ-TIME-01, REQ-TIME-02 |

## 맥락
02 §6.2는 단계 순서를 정했지만, 일시정지 중 명령 펌프, 틱 번호의 경계, M0 빈 시나리오의 날짜 표현과 CLI 입력 경로는 정하지 않았다. WP-03의 기존 테스트는 `data/packs/testland/defines.toml`이 비어 있음을 검사한다. 그 테스트를 지우거나 약하게 만들지 않고 M0 시간 데이터를 실행해야 한다. 실제 Testland 콘텐츠와 데이터 팩 병합은 후속 WP다.

## 결정
- `Simulation::step()`은 명령 펌프다. `tick`은 진행한 시간 수이자 다음 진행 단계의 시작 번호다. 큐는 `(tick, NationId, 서버 도착 순번)` 순서의 BTreeMap이다. 중복 키와 과거 틱은 거부한다. 실제 도착 순번 생성·플레이어/AI 권한은 호스트 책임이다.
- 매 펌프에서 현재 틱의 명령을 적용하고, pause 결과가 false일 때만 한 시간을 진행한다. pause 중에도 현재 틱에 예약한 재개 명령을 처리한다. paused 결과는 Commands → Snapshot이며 시간·일·월 시스템은 진행하지 않는다.
- `State` 필드는 비공개다. 스냅샷은 소유된 읽기 전용 복제다. 직접 상태를 바꾸는 API나 게임 시스템용 임의 콜백은 제공하지 않는다. 시뮬레이션 해시는 State·TimeConfig·미처리 큐 전체의 정규 postcard 바이트를 oh_core의 FNV-1a 64로 계산한다. seed는 상태에 있지만 빈 시스템이 난수를 소비하지 않는다.
- M0 달력은 양의 u32 연도를 지원하는 역산 그레고리력이다. 일·월 경계를 정확하게 검사하기 위한 기술 규약이며, 특정 시대나 실제 국가·경제·전투 수치를 넣지 않는다. 24시간/일, 월 길이·윤년 규칙, 배열의 5개 속도 슬롯은 단위/자료 구조 정의다. 조정 가능한 속도 수치는 defines에만 둔다. `time.initial_speed`도 필수 데이터이고 숫자 기본값으로 대체하지 않는다. 달력 변환은 `oh_sim::formula`에 모은다. 시계 오버플로는 상태·큐 변경 없이 실패한다.
- M0 팩은 `data/packs/examples/m0/testland`이고 manifest ID는 `m0_testland`다. 원래 팩 ID `testland`와 중복하지 않으며 depends/conflicts/load_after는 모두 비어 있다. 입력은 그 팩의 `manifest.toml`, `defines.toml`, `scenarios/testland/scenario.toml`이다. 기존 `data/packs/testland`와 WP-03 테스트는 원본을 유지한다. CLI는 `data/packs/examples/m0/<scenario_id>`를 읽으며 공개 `oh_cli::M0_PACK_ROOT`가 기본 루트이며 `oh_cli::load_scenario(pack_root, scenario_id)`가 데이터 팩 메타데이터·시작일·TimeConfig를 담은 LoadedScenario를 반환한다. `LoadedScenario::simulation(seed)`로 진행 전 세션을 만들고, `oh_cli::run(pack_root, options)`는 이 경계를 호출한 뒤 지정 시간만 진행한다. 호스트가 다른 루트를 지정할 수 있다. 도움말에도 실제 팩 경로·ID·입력을 명시한다. `scenario.toml`에는 start_date 문자열만 허용한다. `2000-01-01`은 임의의 시험 입력이다. 이후 콘텐츠 로더가 전체 시나리오와 팩 해석을 맡아야 한다.
- CLI 입력은 run, 시나리오 ID, seed, 정확히 하나의 ticks/days가 필수다. days는 checked ×24로 변환한다. hash-out은 소문자 hex 16자리와 줄바꿈 하나만 stdout에 낸다. 없는 validate/ai-bench/repro 명령은 성공한 것처럼 처리하지 않는다. CLI 진단은 개발 도구 텍스트이며 제품 화면 문자열은 추가하지 않는다.
- 단계 목록은 후속 시스템이 들어갈 M0 빈 슬롯의 실행 순서를 표시한다. 이동·전투·경제·AI·통계의 게임 규칙은 구현하지 않는다. 날짜는 현재 시간의 도착 경계로 갱신한 다음 해당 0시의 일일·월간 슬롯을 발행한다. 자동 저장은 시뮬레이션 밖에 남긴다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| pause일 때 step 자체를 호출하지 않음 | 단순한 진행 루프 | 큐의 재개 명령이 적용되지 않음 |
| WP-03의 빈 defines 테스트 변경 | 기존 Testland 경로를 그대로 사용 | 다른 WP 테스트 변경이며 보존 지시와 충돌 |
| 별도 시간 define 파서·묵시적 코드 기본값 | 팩 파일 감소 | WP-03 로더 재사용과 데이터 필수 규약을 잃음 |
| 달력·AI·경제 규칙의 전체 구현 | 장기 기능 포함 | M0 범위와 미결정 게임 규칙 금지에 어긋남 |

## 결과와 영향
새 크레이트 버전은 추가하지 않고 기존 잠금 버전 serde 1.0.229와 toml 1.1.6을 재사용한다. 단일 스레드·정수 상태·정렬 큐이며 호스트 I/O/실시간 속도 조절은 CLI·서버 경계에 있다. 저장 포맷·게임 종료 조건·콘텐츠 병합·프로토콜은 정의하지 않는다. 기존 골든 기대값은 수정하지 않았다. 서버/클라이언트 연결은 WP-05가 같은 공개 라이브러리 API를 사용한다.
