# ADR-0501 M0 프로토콜과 호스트 경계

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-05, REQ-GEN-01, REQ-GEN-05, REQ-NET-01, REQ-NET-04, REQ-NET-05 |

## 맥락
02 §4.2·§4.3은 전용 시뮬레이션 스레드, 채널, MessagePack, Rust 단일 타입 정의를 요구한다. M0 메시지의 세부 필드와 로비 이전 세션 수명은 정하지 않았다. WP-04 로더가 최상위 `oh_cli` 안에 있어 서버가 그대로 의존하면 호스트 간 의존이 생긴다. 이번 지시에서 공용 로드 경계의 최소 추출을 허용했다.

## 결정
- `oh_proto`에 Hello, Create, Join, Command, Welcome, CommandResult, Snapshot, Delta, Query, QueryResult, Notice의 Rust 타입을 둔다. `serde(tag = "type")`와 `rmp_serde::to_vec_named`로 필드 이름이 있는 MessagePack을 보낸다. 프로토콜 식별자는 `m0-v1`이다. ts-rs 선언으로 `client/src/proto/protocol.ts` 하나를 생성하고 Rust 테스트에서 파일 전문 일치를 검사한다. CI도 재생성 후 diff를 검사한다.
- u64 틱·시드·순번은 십진 문자열로 전송한다. JS의 안전 정수 한계를 피한다. 생성물에 수동 TS 타입을 보태지 않는다. Rust 인코딩 픽스처 12개를 TS에서 실제 MessagePack 디코더로 검사하며 u64::MAX도 다룬다.
- M0 `Join(local, null)`과 `Create(testland, seed, single)`은 연결별 빈 시나리오 하나를 생성한다. 다른 세션·국가·모드·시나리오는 Notice로 거부한다. 연결이 끝나면 해당 스레드도 종료한다. 연결 간 공유 로비·국가 소유권·재접속 복구는 M6/M2 범위다. 미래 게임 시스템 규칙을 만들지 않는다.
- 각 연결의 서버는 동기 `oh_sim::Simulation`을 전용 OS 스레드에서만 호출한다. bounded std 채널로 명령을 보내고 oneshot으로 적용 결과를 받는다. 명령은 `enqueue` 후 다음 `step`의 명령 단계에서 적용한다. 서버 도착 순번과 클라이언트 중복 방지 순번은 따로 둔다. NationId(0)은 국가 없는 M0 시나리오의 기술적 큐 식별자이고 국가 게임 규칙이 아니다.
- 시계·sleep·네트워크는 `oh_server`에만 둔다. 속도 비율은 WP-04 `TimeConfig`와 기존 `defines.toml`을 그대로 사용한다. 전송 주기·채널 용량·메시지 상한·접속 제한시간·기본 시드는 동일 팩의 `network` defines에 둔다. Delta는 최소 100ms 간격(02 §4.2 최대 10회/초)으로 최신 시간 상태를 보내며, 전체 후속 엔티티 필드는 해당 WP가 추가한다.
- `oh_data::m0::load_m0_scenario`로 기존 CLI의 파일 로드 부분만 추출한다. `oh_cli::M0_PACK_ROOT`, `load_scenario`, `LoadedScenario::simulation`, CLI 인수·오류·해시는 유지한다. 날짜/시간 검증과 시뮬레이션 생성은 두 호스트에 남겨 oh_data→oh_sim 역방향 의존을 만들지 않는다.
- Welcome에는 엔진/프로토콜 버전과 팩 ID·버전·원본 세 입력 파일의 정규 직렬화 xxHash64를 보낸다. 미래 팩 병합 해시 규칙은 만들지 않는다. HTTP는 M0에서 no-cache로 제공한다.
- Query(time)만 현재 서버 시간 상태를 반환한다. 원장·지도 조회는 supported=false와 현지화 사유를 반환한다. Notice는 현지화 키다. 순번 거부와 잘못된 속도는 CommandResult로 명시한다.
- 브라우저는 받은 Snapshot/Delta를 저장하고 그대로 표시한다. 자체 날짜 진행·틱 타이머·속도 계수·게임 공식은 없다. 종료 뒤 입력을 비활성화한다. React StrictMode 정리 때 오래된 소켓의 콜백을 해제한다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| 서버가 oh_cli에 의존 | 로더를 바로 재사용 | 최상위 호스트 간 의존을 만든다 |
| tokio 태스크에서 시뮬레이션 실행 | 코드량 감소 | 02 §4.2 동기 전용 스레드 경계를 위반한다 |
| JSON 또는 수동 TS 타입 | 시작이 단순 | MessagePack·Rust 단일 정의 요구를 충족하지 않는다 |
| u64를 JS number로 전달 | 표시가 쉬움 | 2^53 이후 정밀도 손실 |
| M0에 공유 로비·재접속 추가 | 상태 지속 | 후속 WP 규칙·범위를 침범한다 |

## 결과와 영향
시뮬레이션 코드와 공식·골든 기대값은 수정하지 않는다. 호스트 전송 설정은 TimeConfig에 포함되지 않아 기존 결정론 상태 해시를 바꾸지 않는다. 로컬 사용자 인증과 원격 멀티플레이는 아직 지원하지 않으며 서버는 loopback에만 바인드한다. 독립 검증과 기본 브랜치 CI 결과는 오케스트레이터가 별도로 판정한다.

2026-10-06 공식 registry/docs 확인: [axum 0.8.9, MIT](https://docs.rs/crate/axum/0.8.9), [Tokio 1.53.2, MIT](https://docs.rs/crate/tokio/1.53.2), [ts-rs 12.0.1, MIT](https://docs.rs/crate/ts-rs/12.0.1), [rmp-serde 1.3.1, MIT](https://docs.rs/crate/rmp-serde/1.3.1), [named MessagePack encoding](https://docs.rs/rmp-serde/1.3.1/rmp_serde/encode/fn.to_vec_named.html), [Serde enum representation](https://serde.rs/enum-representations.html), [@msgpack/msgpack 3.1.3, ISC](https://github.com/msgpack/msgpack-javascript). `cargo info`와 `npm view`로 실제 registry 버전·라이선스도 확인했다. 기존 serde_json 1.0.151을 재사용한다. @types/node 24.19.1은 registry에서 MIT를 확인했다. 모든 추가 라이선스는 02 §14.3 허용 목록에 있다.
