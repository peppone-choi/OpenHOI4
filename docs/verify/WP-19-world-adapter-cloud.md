# 검증 리포트 — WP-19 등록·World/Map 입력 단위

| 항목 | 값 |
|---|---|
| 판정 | PASS — 등록된 계산 모듈과 순수 World/Map 네트워크 입력 부분 |
| 검증자 | 구현 비참여 새 독립 검증 담당, 분리된 worktree, 추적 소스 무변경 |
| 구현·검증 후보 | `ae5bfba50e7dab59c93a28fee620ec17cf68059c` / `7cbb5572c0a11dfb279016ddafb95777bb53456b` |
| 기준 main | `6d1d1d9278c7771be9b9ba2943e745a9094b9104` |
| 검증 일시 | 2026-10-08 |

## 직접 실행한 명령

Rust 1.99.0과 기존 offline cache를 사용했다. 전체 workspace의 서버 빌드는 실제 client/dist가 필요하다. 변경되지 않은 클라이언트 추적 입력 81파일의 bytes를 증명하고 기존 인수 build를 hardlink로 재사용했다. 새 client/browser 검사를 실행한 것으로 기록하지 않는다. 기본 재현 명령은 [모듈 README](../../crates/oh_sim/src/supply/README.md), 입력 권위·단위·방향은 [ADR-1902](../adr/ADR-1902-world-network-adapter.md)를 따른다.

| 명령·검사 | exit | 실제 결과 |
|---|---|---|
| `cargo test -p oh_sim --locked --offline` | 0 | 136 PASS, 등록 supply 24개·World adapter 12개 포함 |
| `cargo test --workspace --locked --offline` | 0 | 57 suites, 306 PASS |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 0 | client/dist 선행조건을 충족한 후 PASS |
| `cargo fmt --all --check` | 0 | PASS |
| 문서·release 에셋·architecture·diff 검사 | 0 | PASS |
| 별도 실제 source-path 하네스 | 0 | 아래 독립 oracle·오류·유효 controls PASS |
| 새 process 네트워크 출력 2회 | 0 / 0 | 실제 World 81변형, raw 972행 동일 |
| 새 schema/TS 생성 후 bytes 비교 | 0 | 11개 기존 생성물 동일, 추적 파일 무변경 |
| 실제 `oh_cli validate --deny-warnings` | 0 | testland·testland_m2 두 팩 PASS |
| M1 365일 두 번 / 180일 저장 뒤 새 process 185일 재개 | 0 | 최종 hash·decoded canonical 동일 |
| 전후 HEAD·추적 목록/SHA·diff/status | 0 | 571파일 동일·clean, 기존 baseline 561파일 bytes 보존 |

독립 하네스의 잘못된 state/province fixture 구성 두 번은 exit101이었다. 공개 state membership만 바꾸면 private MapData cache와 일치하지 않았으므로 실제 기존 cross-state province 10/30을 사용해 수정했다. 초기 workspace clippy는 ignored client/dist가 없어 exit101이었다. M1 비교의 초기 aggregate는 save envelope의 UTC 작성 시각까지 같다고 가정해 실패했다. 이후 실제 decoded canonical과 작성 시각만 고정한 envelope bytes를 비교했다. 이 네 준비/비교 실패 기록은 보존했으며 제품 변경이나 P06 finding으로 계산하지 않는다. 최종 PASS가 모든 시도 exit0을 뜻하지 않는다.

## REQ별 확인

| REQ·경계 | 독립 검사 | 실제 결과 |
|---|---|---|
| REQ-SUP-01 부분 | 독립 simple-path oracle과 widest → shortest directed cost → 전체 경로 순 정렬 41,553개 비교 | PASS |
| REQ-SUP-02 부분 | 방향별 육상 거리 oracle 2,444개, shared capacity·손실·보존 128개 | PASS |
| REQ-SUP-02 부분 | 인수된 이전 source와 양방향 동일 비용 Day 결과 288개 정확한 값 비교 | PASS |
| REQ-SUP-01/04 부분 | 실제 국가0·waterNone·owner와 다른 controller·수도/refs/kind·landNone 그래프 차단 | PASS |
| 기존 REQ-MIL-04 비용 경계 | 실제 cross-state World 81개에 독립 raw-bit 공식, fractional modifier 만료·stale context 4개 | PASS |
| 입력 오류 | missing/음수/0/refs/level/overflow/rounded-zero·허용된 양수 unused 표 entries 등 25 controls, 오류 시 원본 불변 | PASS |
| 보존 | 기존 standalone 22개 검사·85 assertions 보존, pack/golden/repro/assets/lock 113파일 동일 | PASS |

## 결정론과 저장 경계

- 두 새 process 네트워크 출력 SHA256은 각각 `105ab9370d8da2bb716d4b60812407b66cc73760930493a36335ae1c4f3a4538`다. 이 값은 순수 Network 메타데이터이며 새로운 전체 Simulation 또는 supply save hash가 아니다.
- 기존 M1 365일 두 번과 180+185일 저장 재개의 최종 hash는 `b595dc2a1e5b4f8c`, split hash는 `b7eca53b4860eadd`다. Decoded canonical SHA256은 `a4819acd3db21703ff9d1325854c02729ddd5cbb9a87c137eaf32a07cba41a10`로 같다. 기존 save envelope의 UTC 작성 시각은 비교에서 명시적으로 구분했다.
- 추적 manifest SHA256은 `65f73480ea1027df004a055b5845a9decad1e56a7a0c4a2654abc51bcc623de2`로 전후 같다. 제품 후보와 구현 tree는 `b8b5a2423bee7f02d23760fdfc1b1f988e909c02`로 같다.

## 금지 사항과 작업 로그 대조

새 어댑터는 immutable World에서 현재 실제 통제·지도·수도·지형·주 인프라를 읽는다. `None`을 국가0으로 변환하지 않는다. 방향별 비용과 무방향 공유 capacity를 구분한다. 명시 양수 인프라 표에 현재 level이 없으면 오류이며 default1·보간 curve·새 placement를 만들지 않는다. 어댑터는 Network만 반환하며 빈 demand로 Day를 계산하지 않는다.

Float·HashMap 순회·RNG·시계/I/O·unsafe·원작 자료·새 의존성/에셋·golden 변경은 없다. 기존 schema/save/protocol/Simulation fields·클라이언트·팩·locks는 그대로다. [작업 로그](../worklog/WP-19-world-adapter.md)의 최초 실패·필터 실행·136개 최종 검사를 직접 재현한 독립 결과와 구분했다. 내부 하네스·명령 원장·raw 출력은 Git에 포함하지 않는다.

## 남은 범위와 통합 게이트

현재 유효 World의 육상 controllerNone producer는 없고 restore도 거부한다. 실제 waterNone/국가0/owner-controller 차이와 그래프 landNone 차단을 각각 검사했으며 유효 육상 None World를 검증했다고 주장하지 않는다. 인프라 전세계 curve·실제 hub/port instance·rail 건설/손상 권위·WP-16 사단 demand/strength/org/attack·현재/다음 movement leg 효과·daily phase/원자성·supply canonical/save/query/wire/served UI는 후속이다. Static rails는 명시 참조 입력이며 건설 상태가 아니다.

한 번의 새 독립 검토에서 제품 finding은 없었고 P06은 필요하지 않았다. 이 문서 작성 시 draft PR·같은 main CI는 후속 통합 게이트다. 기존 [standalone 판정](WP-19-standalone-cloud.md)의 미등록 상태는 그 당시의 기록이다. 이 후속 단위가 전체 WP-19/REQ/M2 완료를 뜻하지 않는다.

## FAIL 항목

지정한 등록·World/Map 입력 범위의 제품 FAIL 없음. 준비 과정의 실패와 미구현 consumer는 위에 구분했다.
