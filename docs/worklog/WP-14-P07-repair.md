# WP-14 작업 로그 — P07 클라우드 인수 오류 수정

| 항목 | 값 |
|---|---|
| 상태 | 검증 대기 (통합·M2 완료 판정 아님) |
| 담당 | 구현 세션, P-06 |
| 브랜치 | fix/p07-cloud-acceptance |
| 기준 HEAD | 743c2a941af5422b94e850b6064b19cffbebac31 |
| 대상 REQ | REQ-NET-01, REQ-NET-06; 기존 지도 팔레트 바이트 동등성 검사 유지 |

## 계획

신규 독립 인수에서 발견한 WebSocket 종료 실패와 팔레트 테스트 시간 초과를 재현하고 최소 수정한다. 경제 규칙, schema, save, protocol 타입, fixture, golden, timeout은 바꾸지 않는다. 본 작업은 기존 미실행 P07 기록이나 WP-14/M2 완료 판정을 대체하지 않는다.

## 테스트 우선 기록

| 검사 | 수정 전 | 수정 후 |
|---|---|---|
| `startup::req_net_01_client_close_handshake` | TCP EOF 전에 Close 프레임 없음, UnexpectedEof, cargo exit101 | pre-Hello / Hello-only / Join 후 Close 상태·사유 echo 및 이후 EOF PASS |
| 기존 `economy_target_native.py` | 12개 중 valid 서버 normal/force 2개 FAIL, query exit1; 10개 PASS | 동일 harness / helper / fixture로 12개 PASS |
| 기존 전체 client 단위 검사, `--maxWorkers=1` | 245 PASS / palette 1 timeout, 5000ms 유지 | 246 PASS, 전체 4.77초 |

## 수정 기록 (P-06)

서버가 받은 Close를 일반 종료 분기로 처리해 TCP를 즉시 버렸다. pinned tungstenite 0.29.0은 Close 응답을 큐에 넣고 다음 read/write/flush에서 내보낸다. Close 뒤 새 send는 SendAfterClosing 오류를 반환하므로 처음 시도한 send 방식도 raw regression과 native control에서 실패했다. 최종 변경은 Close 수신 뒤 다음 recv로 큐의 응답을 전송하고 서버 스트림의 종료를 진행한다. 첫 Hello를 기다릴 때의 같은 원인도 수정했다. 핸드셰이크 상태 3종을 raw TCP/WebSocket 회귀 검사로 확인한다. Node 오류 처리를 가리거나 경제 query helper를 수정하지 않았다.

라이브러리 근거는 설치된 Cargo.lock 대응 소스의 tungstenite `protocol::WebSocket::read`/`WebSocketContext::read` 및 tokio-tungstenite Stream 구현이다. [tungstenite 고정 버전 문서](https://docs.rs/tungstenite/0.29.0/tungstenite/protocol/struct.WebSocket.html)는 클라우드 web 도구에서 접근 오류였으며 로컬 고정 소스를 직접 확인했다. axum 0.8.9 Message::Close 문서의 자동 응답 설명도 확인했다.

24,889개 province 실제 preview fixture의 구간 계측: world 생성 101.558ms, generic palettes 2.282초, Vitest palette 객체 deep equality 2.737초, 두 update 652.203ms, ranges equality 44.646ms, 마지막 bytes equality 811.167ms. 총 6.658초로 timeout이었다. 구간 계측은 임시 변경으로만 사용하고 제거했다. Node 내장 `deepStrictEqual`로 palette 객체와 마지막 Uint8Array 비교를 바꿨다. 전체 객체 키·값, 모든 배열 길이와 모든 바이트를 비교하며 ranges의 기존 `toEqual`은 유지했다. 새 의존성, 샘플링, fixture 축소, timeout 연장은 없다.

## 실행한 검증

증거는 작업 환경의 `scratch/openhoi-p07-repair/` 아래 명령별 `.log`/`.json`과 저장소의 무시된 `target/evidence/WP-14-M2-r3-P06-2/repair-before`, `repair-after-read`, `repair-final`에 보관한다. native 결과는 실행 파일 SHA256·pack identity·input SHA256·normal/force 종료 코드·query 종료 코드를 포함한다. 모든 fixture 추적 파일은 보존한다.

| 명령 | 종료 코드 | 로그 |
|---|---|---|
| `cargo fmt --check` | 0 | fmt.log |
| `cargo clippy --workspace --locked -- -D warnings` | 0 | clippy.log |
| `cargo test --workspace --locked` | 0 | workspace.log |
| `cargo run --locked -p oh_cli -- run --scenario testland --days 365 --seed 1 --hash-out` 2회 | 0 / 0 | determinism-a.log / determinism-b.log |
| `python3 tools/check_docs.py` | 0 | docs.log |
| `python3 tools/check_architecture.py` | 0 | architecture.log |
| `npm --prefix client run build` | 0 | client-build-final.log |
| `npm --prefix client run proto:generate` | 0, 추적 생성물 변경 없음 | proto-generate.log |
| `cargo build --locked -p oh_server -p oh_cli` | 0 | native-build-final.log |
| `python3 crates/oh_server/tests/economy_target_native.py --out target/evidence/WP-14-M2-r3-P06-2/repair-final --bin-dir target/debug` | 0, 12개 PASS | native-target-final.log |
| `python3 crates/oh_server/tests/lifecycle.py` | 0, HTTP/active WS/SIGINT/exit0/port release PASS | native-lifecycle.log |

`npm --prefix client test` 최종 기본 실행은 exit0, 12개 파일/246개 테스트 PASS(2.51초)였다(`client-final.log`). 결정론 두 실행 hash는 모두 `b039d35666b77fc2`였다. 기존 ts-rs의 serde `deny_unknown_fields` 해석 경고는 남아 있으나 fmt/clippy/workspace 테스트 종료 코드는 모두 0이다. 문서 추가 후 docs/whitespace 검사도 다시 실행한다.

## 결정 필요

없음. 게임 규칙·설계 결정 변경 없음.

## 범위 밖 발견

기존 generic map `palettes`/`updateColors`는 province마다 Array.find를 반복해 큰 preview 입력에서 이차 비용이 발생한다. 이번 최소 인수 수정에서 제품 팔레트 최적화는 하지 않았다. preview 전용 fast 경로의 바이트 동등성은 전량 비교한다.

## 못 한 부분과 이유

새 독립 검증 세션의 판정과 main CI·P07 통합은 오케스트레이터의 후속 작업이다. 실제 브라우저/전체 M2 인수와 다른 WIP의 완료는 주장하지 않는다.
