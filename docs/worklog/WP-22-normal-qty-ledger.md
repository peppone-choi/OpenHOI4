# WP-22 정상 편제 Qty 원장 producer WIP

## 범위와 계획

기준 main은 `0d251ce46a9c108db3a3295c87757132fbeb925a`다. 실제 Claude
Opus5.5 medium 계획(session `d6eb17de-4783-4849-9248-0fa36e4e7425`, exit0)을
인계받았다. 계획 자체는 Read/Grep만 수행했고 테스트는 NOT_RUN이었다.
구현 담당은 먼저 미확인 서버·프로토콜·검증기·수명·응답 한도를 확인했다.

이번 체크포인트는 producer와 전용 회귀만 다룬다. 기존 `aggregate`는 같은
계산의 `aggregate_with_ledger`에서 Normal만 반환한다. 7개 합산 Qty 필드의
실제 checked 루프에서 component 값·누적값과 정렬 후 role별 position을
기록한다. 중복 component는 `{field}:{role}:{position}`로 구별된다.
기존 오류 문자열·manpower→speed→zero→weighted→equipment→sum 순서를
유지한다. 실패 시 부분 원장을 반환하지 않는다.

새 진단 타입에는 Serialize/Deserialize가 없다. 기존 Normal의 네 필드와
군사 상태·정의 identity·canonical/hash/save representation은 유지한다.
StatLedger/Fx를 재사용하거나 Qty bits를 좁히지 않았다. 가중 평균 3개와
speed 최솟값, 감편 효과·실제 보급 소비·명령/편집 UI는 이번 범위 밖이다.
기존 aggregate 호출도 진단을 만들고 버리므로 추가 allocation 비용은 있다.
전체 성능 gate는 이 표적 검사로 인수하지 않는다.

## 구현 담당의 실제 검사

공식 Rust1.99 환경과 기존 `/tmp/openhoi-wp15-target`을 단독으로 사용했다.
jobs1, incremental0, test/dev debug0, offline, locked를 유지했다. 각 실행은
부모 프로세스가 cgroup memory를 관측하며90% 도달 시 자기 child group을
중단하도록 했다. 다른 프로세스·보안 설정·도구는 변경하지 않았다.

| 명령 | 종료 | 실제 결과 |
|---|---:|---|
| `cargo test -p oh_sim --test military_template_qty_ledger --locked` (구현 전) |101| 새 API 부재 E0432. 런타임 assertion 실패로 기록하지 않음 |
| `cargo test -p oh_sim --test military_template_qty_ledger --test military_templates --test military --locked` |0| 새7·기존4+9, 총20 PASS, 실패/ignored0 |
| `cargo test -p oh_save --test military --test m2_military_pack --locked` |0| 기존9+2, 총11 PASS, 실패/ignored0. V7/V8·실제 codec·canonical/hash·재개·회계 거부 |
| `cargo clippy -p oh_sim --all-targets --locked -- -D warnings` |0| PASS |

새 검사는 실제 기여 값과 원 bits, 정상 최종값·마지막 누적값, 원래 Normal
직렬화 키와 왕복, 정의 identity 불변, 반복/서로 다른 component의 정렬 후
위치, JS 안전 정수보다 큰 Qty, 부분 분수, 오류 우선순위,12/4 구성 한도,
1MiB 이하 정의 문서의4096개 최대 구성 template을 확인한다. 저장 회귀는
후보의 codec/재개 일치이며 옛 commit의 전체 save 파일을 재생성해 bytes를
대조한 별도 cross-commit 검사로 확대하지 않는다.

원 로그·명령/exit·메모리 관측·preflight는 Git 밖
`/workspace/scratch/openhoi-wp22-military-ledger/`에 있다. 저장 검사 관측 최고
메모리는15,459,557,376bytes로90% 한도15,461,882,265bytes보다 낮았으나
여유가 매우 작았다. 한 번에 무거운 시험 하나만 실행했다.

## 프로토콜·표시를 보류한 실제 이유

기존 client의 MilitaryTemplateView는 exact key count 검사를 한다. 필수
`ledgers`를 기존 응답에 추가하면 이미 열린 옛 client가 거부한다. 서버가
같은 binary에 client를 embed하고 `no-cache`로 제공하는 사실만으로 이미
열린 client·별도 접속 client의 원자적 갱신을 보장할 수 없다. 계획의
필수 필드/legacy malformed 전제를 현재 wire에 적용하지 않았다.

`network.max_message_bytes=65536`은 WebSocket 수신에 적용된다. `send`는
encode 결과를 보내며 동일한 송신 guard가 없다.4096개 template 각각
12combat/4support인 문서는 실제 parser 한도 안에서 유효했다. Claude가
제안한 DTO의 짧은 ID·값1 fixture를 기존 MessagePack 라이브러리로 측정한
원장 추가 필드만72,548,352bytes(69.1875MiB)다. 이는 아직 구현되지 않은
DTO projection의 크기 증거이며 실제 server 응답/정확한 최대 전체 크기로
주장하지 않는다. 긴 ID·다른 값·기존 payload도 더해질 수 있다.

부모에 요청한 보완은 legacy `military` 조회를 유지하면서 versioned opt-in
원장을 template 단위 또는 bounded page로 반환하는 계약, 응답 budget,
optional capability/unsupported와 base view 수명 분리를 실제 Claude 계획으로
정하는 것이다. 새 query/type/페이지 정책은 이 체크포인트에서 구현하지 않았다.

연결 끊김은 마지막 검증본 stale 보존, scope/연결 전환과 unsupported는
해당 view 제거라는 기존 코드 계약도 확인했다. UI를 변경하지 않았으므로
그 계약은 유지된다. 새 원장 channel의 동일 수명·ABA·구 서버 fallback은
보완 계획과 후속 실제 검사의 대상이다.

## 남은 범위

server/protocol/generated TS/validator/tooltip/현지화·served browser 검사,
독립 QA/Claude 리뷰·exact SHA CI·main 병합과 같은 main CI는 아직 인수하지
않았다. 이 WIP를 전체 WP-22/REQ-UI-04/M2 완료로 기록하지 않는다. 승인된
기존 컴파일 정리 결과·소스/증거 보존은 별도 감사이며 제품 기능 증거가 아니다.

원작 대비 차별화 초안은 자체 합성 입력에서 권위 Qty 계산의 실제 발생별
기여를 보존하는 것이다. 원작 코드·자산·수치나 새 게임 규칙은 쓰지 않았다.
01의 사용자 결정·차별화 절을 승인 없이 변경하지 않았다.
