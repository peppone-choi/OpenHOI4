# ADR-1001 보정치·수치 원장의 기술 경계

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-10, REQ-UI-04, DR-01~DR-10 |

## 맥락
02 §6.3은 Modifier 필드, 기본값 → Add 합 → source 문자열순 Mul, 항목별 누적 기록과 실제 적용값의 비트 일치를 정한다. expires의 저장 단위·경계, 동일 source의 동률, 오류 및 공개 연결 API는 정하지 않았다. M1의 국가/주 상태는 WP-09, 프로토콜과 화면은 후속 담당이 연결한다. 미결정 게임 규칙이나 보정치의 허용 범위를 이 WP에서 정하지 않는다.

## 결정
- `Modifier`는 데이터 식별자 source/target_stat, Add 또는 Mul, Fx, `expires: Option<u64>`를 가진다. expires는 시나리오 시작 이후 절대 tick이며 **그 tick부터 제외**한다. None은 만료 없음이다. u64는 기존 Simulation::tick 단위와 같다. 실제 효과의 지속시간은 데이터/해당 시스템 담당이 공급한다.
- 대상별 활성 보정치를 `(op, source)`로 정렬한다. Add 단계가 Mul보다 앞선다. 각 단계 안의 source 순서는 Rust 문자열의 사전순이며 locale/입력 순서에 의존하지 않는다. 기본값에 Add를 하나씩 checked_add하고 그 결과에 Mul을 하나씩 checked_mul한다. 곱셈마다 fixed 기본 반올림을 유지하며 결합·재배치·합성 연산을 하지 않는다.
- 활성 동일 `(target_stat, op, source)` 중복은 오류다. 동일 source가 Add와 Mul을 각각 제공하는 것은 허용한다. 여러 기여를 표현해야 하면 데이터 작성자가 구별되는 source 식별자를 준다. 만료된 항목과 다른 대상은 계산·중복 검사에서 제외한다. 요청 target_stat 및 활성 source의 빈 식별자를 거부한다. 게임별 수치 범위/음수 제한/클램핑 규칙은 추가하지 않는다.
- 어느 중간 연산이든 범위를 넘으면 `LedgerError::Overflow { source, op }`를 반환한다. wrapping/saturating/후속 상쇄로 오류를 숨기지 않는다. 계산은 순수 함수이고 상태를 수정하지 않으며 실패 시 부분 StatLedger를 반환하지 않는다.
- `formula::stat_value`와 `StatLedger::evaluate`는 같은 `formula::visit_stat` 계산 경로를 쓴다. 후자는 기본값 행 및 각 보정치 행을 기록하며 시스템은 `ledger.value()`를 실제 적용값으로 쓴다. 별도로 저장한 값은 `verify_applied_value`가 raw i64 비트로 검사하고 불일치 오류를 반환한다.
- StatLedger는 target_stat, 계산 tick, 최종 Fx, 정렬 entries를 소유한다. 기본행은 `source: None`, `LedgerOp::Base`, value=accumulated=base다. 보정치 행은 source, Add/Mul, value, accumulated를 보존한다. getter만 제공하고 Deserialize를 제공하지 않아 검증되지 않은 외부 원장을 계산 결과로 수용하지 않는다. Modifier는 향후 저장 경계를 위해 Serialize/Deserialize를 지원한다.
- WP-09가 엔티티별 저장·재계산 시점을 소유한다. 기본값/보정치 변경 및 만료 경계에서 해당 tick으로 evaluate해 결과 전체를 교체한다. Query는 저장 결과와 실제 적용값을 비교한 뒤 oh_proto에서 표시용으로 변환한다. 클라이언트는 서버의 accumulated/최종값만 표시한다. source/target_stat는 현지화용 식별자이며 진단 오류 문자열은 사용자 화면 문구가 아니다.
- 기존 Simulation 상태/스케줄러/해시 직렬화에 원장 필드를 추가하지 않는다. 게임 계수·에셋·화면 문구·실제 국가/주 계산식·네트워크 타입은 추가하지 않는다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 날짜별 expires | 데이터에서 읽기 쉽다 | 시간 단위 효과를 표현하려면 별도 시간 경계를 도입해야 한다. 기존 절대 tick을 재사용한다 |
| 동일 source 동률을 입력 순서로 적용 | 중복 데이터도 계산 가능 | Mul 중간 반올림이 입력 순서에 따라 달라진다. 암묵적 게임 중첩 규칙을 만들지 않고 오류로 돌려준다 |
| 값·만료일을 추가 정렬 키로 사용 | 중복을 허용하면서 결정론 유지 | 문서가 지정하지 않은 우선순위를 계산에 추가한다 |
| 클라이언트에서 항목 합산 | 작은 서버 응답 | 클라이언트 규칙 계산 금지 및 고정소수점 비트 일치 요구와 충돌한다 |
| mutable 캐시를 즉시 Simulation에 추가 | 조회 저장소까지 한 번에 제공 | WP-09 소유 상태 및 기존 M0 해시에 영향을 준다. 순수 결과의 저장 경계를 먼저 제공한다 |

## 결과와 영향
정렬은 O(n log n), 계산은 O(n)이다. HashMap 순회/float/시계/I/O/난수/네트워크 의존성을 계산에 추가하지 않는다. Modifier의 동일 source 유일성은 데이터 계약이며 게임 효과의 수치 제한이 아니다. WP-09가 재계산과 캐시 무효화 및 raw bits를 보존할 저장 형식을 연결해야 한다. 기존 골든과 저장 포맷은 수정하지 않았다.

새 런타임 의존성은 없다. 기존 허용 목록의 proptest를 oh_sim **dev-dependency만** 동일 버전 `=1.11.0`, default-features=false/std로 재사용한다. Cargo.lock에는 oh_sim 의존 연결 한 줄만 추가된다. 각 속성 테스트는 고정 seed 20261006, 512 cases, failure_persistence=None이다. 테스트 runner는 시뮬레이션 RNG가 아니며 제품 계산에서 사용하지 않는다.

2026-10-06 공식 패키지 문서 확인:
- [proptest 1.11.0 패키지·라이선스](https://docs.rs/crate/proptest/1.11.0/source/Cargo.toml.orig): MIT OR Apache-2.0, 허용 목록과 일치. 버전 변경 없음.
- [proptest Config](https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html): 고정 seed/cases/persistence 설정.
- [fixed 1.31.0 산술 API](https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html#method.checked_mul): checked 곱셈으로 중간 overflow 검출. 기존 oh_core의 고정소수점 타입을 재사용한다.

별도 i128 기준 계산에서 Add는 raw 정수 합, Mul은 `(left * right) >> 32`로 각 단계의 Fx 기본 동작을 검사한다. 임의 전체 i64 비트 입력·음수·만료·다른 대상·입력 역순에서도 결과 또는 overflow 분류가 일치하는지 테스트한다. 독립 검증 PASS/세 OS CI는 이 ADR의 로컬 검사로 대신하지 않는다.
