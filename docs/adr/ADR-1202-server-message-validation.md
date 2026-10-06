# ADR-1202 P-06 서버 수신 메시지 구조 검증

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-12 P-06, REQ-LOC-01, REQ-NET-04, REQ-NET-05, REQ-GEN-05 |

## 맥락
원본 a91d324의 독립 검증이 valid MessagePack Snapshot.state.date 객체 치환에서 React error #31과 빈 셸을 재현했다. M0 network.ts의 decode 결과 타입 단언은 런타임 값을 검증하지 않고 React 렌더는 수신 try/catch 밖에서 발생한다. 이 상속 결함을 오케스트레이터가 P-06 범위에 명시적으로 추가했다.

## 결정
MessagePack decode 결과는 unknown으로 받는다. UI callback 전에 생성된 ServerMessage/TimeState/PackInfo에 연결된 Guard와 Shape mapped type으로 모든 현재 variant와 필드를 검사한다. `satisfies { [K in ServerMessage['type']]: Shape<Extract<ServerMessage,{type:K}>> }`가 Rust에서 생성한 필드·variant·타입의 변경과 검증기의 누락을 typecheck에서 드러낸다. TS에 별도 프로토콜 interface/union을 정의하지 않는다. Rust와 생성 TS는 변경하지 않는다.

문자열·불리언·nullable·배열·중첩 객체를 검사한다. Rust TimeState의 hour/speed는 u8이므로 유한 정수 0..255의 wire 표현만 검사한다. 날짜 파싱, 시간 진행, 속도 게임 규칙 계산, numeric string 변환을 하지 않는다. tick/sequence는 string 그대로 두어 u64 정밀도를 보존한다. 명시적인 nullable 필드는 null을 허용하며 필수 생성 필드 누락은 거부한다. 추가 필드는 Rust serde의 기본 동작과 같이 허용한다. TypeScript 생성에서는 Rust 정수 폭을 number로 표현하므로 Rust의 u8 폭 변경은 해당 guard 재검토가 필요하다.

비binary frame, 디코딩 오류, 잘못된 구조는 메시지를 보정하거나 UI에 전달하지 않고 즉시 reject한다. socket handler를 제거하고 연결을 닫으며 단 한 번의 local `invalid-server-message` 오류를 onClose callback에 전달한다. App은 disconnected·ko/en 오류 notice를 표시하고 기존 마지막 정상 상태를 보존한다. 기존 connected 조건으로 시간 입력을 비활성화한다. 닫힌 연결의 send와 후속 수신도 차단한다. 일반 WS close/error 이벤트를 local 오류 key로 잘못 전달하지 않도록 callback을 감싼다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| decode 뒤 as ServerMessage 유지 | 코드가 짧음 | 구조가 틀린 정상 MessagePack을 UI로 전달 |
| UI에서 객체를 string으로 바꿈 | React error를 피함 | 깨진 데이터를 정상값처럼 표시 |
| React error fallback만 추가 | 셸 전체 소실 완화 | 잘못된 입력을 도메인 경계에서 거부하지 못함 |
| TS 프로토콜 interface 별도 정의 | 검증기 작성 편의 | Rust 유일 타입 원칙과 맞지 않음 |

## 결과와 영향
원본 독립 실패와 직접 재현 red→green, 모든 Rust wire 12개 기존 계약, 서버 variant 6개의 구조 검사, 큰 u64 string·단 한 번 종료 callback, 실제 Rust proxy 회귀를 보존한다. 신규 의존성·에셋·게임 규칙·프로토콜·시뮬레이션 변경은 없다. 기존 a91d324 증거는 덮어쓰지 않고 P-06 증거를 evidence/WP-12/p06에 저장한다. 현지화 E2E에 OH_E2E_EVIDENCE 경로 선택만 추가해 기존 파일 보존과 ignored 검증 출력을 지원한다.
