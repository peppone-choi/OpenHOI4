# 검증 리포트 — WP-14 P07 cloud repair

| 항목 | 값 |
|---|---|
| 판정 | PASS — 지정한 repair 및 WP-14 producer 인수 범위 |
| 검증자 | 새 Codex 검증 세션, 구현 비참여, 추적 파일 무변경 |
| 커밋 | `4ae0446a01487e9d6d1415e7191a78407a8e0b6f` |
| 검증 일시 | 2026-10-08 UTC |

공개용 실행 요약이다. 과거 제외된 P07 실행 원장을 복원하지 않았고, 새 검증의 raw 로그·API·세션 경로는 Git에 넣지 않는다. AGENTS·HANDOFF·M2 계획·수정 작업 로그를 읽은 별도 세션이 직접 실행했다.

## 직접 실행한 명령

| 명령 / 검사 | 종료 코드 | 결과 요약 |
|---|---:|---|
| `cargo fmt --check` | 0 | PASS |
| `cargo clippy --workspace --locked -- -D warnings` | 0 | PASS |
| `cargo test --workspace --locked` | 0 | 251개 PASS, 실패·ignored 0 |
| client `ci`, `build`, `typecheck`, 기본 `test` | 0 | 246개 PASS, timeout 변경 없음 |
| client `proto:generate` | 0 | 생성물 bytes 동일 |
| `cargo build --locked -p oh_server -p oh_cli` | 0 | 클라이언트 빌드 뒤 별도 서버 빌드 |
| raw Close regression | 0 | pre-Hello / Hello-only / Join의 Close echo 후 EOF |
| unchanged `economy_target_native.py` | 0 | 12개 PASS |
| unchanged `economy_restore_native.py` | 0 | 4개 PASS |
| `lifecycle.py` | 0 | HTTP·active WS·SIGINT·정상 종료·포트 해제 |
| 365일 결정론 2회 | 0 / 0 | 동일 hash |
| active Testland strict validate | 0 | PASS |
| docs / release assets / architecture | 0 | PASS |
| economy schema 독립 생성 비교 | 0 | JSON bytes 동일 |
| current-save capture·normal/force native·aggregate | 0 | M1·additive economy-v5 PASS |
| frozen historical-save·historical server | 0 | 원 bytes 보존, normal/force 서버 거부 |
| Python save evidence / localisation | 0 | 34개 / 3개 PASS; checker의 기존 unused-key 경고는 남음 |
| npm license checker | 0 | 103개 의존성, 오류 0 |
| 시스템 Chromium M0 / M1 national·malformed-world | 0 | 12개 / 3개 PASS |
| palette comparator 변조 검사 | 0 | byte·길이·키·중첩 값 변조 5종 거부 |

34개 실행 receipt는 모두 exit 0이다. 추가 normal control과 실제 `--force` 실행은 별도 기록으로 구분했다. 시스템 Chromium 151을 사용했으며 다운로드가 차단된 pinned Playwright 브라우저나 Firefox/WebKit을 실행한 것으로 계산하지 않는다.

## REQ별 확인

| REQ / 계약 | 검사 | 실제 검사 여부 | 결과 |
|---|---|---|---|
| REQ-NET-01 | 세 단계 Close 상태·사유 echo, 기존 query 정상 종료 | 실행 | PASS |
| REQ-NET-02·06 | embedded assets·active WS·SIGINT·exit 0·포트 재사용 | 실행 | PASS |
| REQ-ECO-01·02·04·06·07, REQ-NAT-02·03·04의 producer 범위 | 경제 단위 13개·독립 formula 8개·restore 4개·원자성 | 실행 | PASS |
| restore-stage | CLI/server normal·force의 `economy:TargetConflict`, exit 1 | 실행 | PASS |
| MIN target | CLI/server normal·force의 `economy:InvalidValue`, exit 1 | 실행 | PASS |
| valid target2 | CLI exit 0; 양쪽 서버 HTTP 200·query/server exit 0 | 실행 | PASS |
| REQ-SAV-02 및 v5/wire | 새 프로세스 재개·canonical/hash·queue·wire·음성 변조 | 실행 | PASS |
| 기존 client/server 연동 | 별도 서버 M0/M1 브라우저 검사 | 실행 | PASS |

두 fixture manifest·save SHA를 직접 확인했다. restore-stage는 22파일/5681bytes, target-range는 22파일/5617bytes이며 원 bytes·CRLF를 보존했다.

생산·항복·군사 훈련 consumer는 01 §4.3의 후속 WP 경계로 남는다. 위 producer 검사를 해당 REQ 전체 또는 후속 consumer의 인수로 확대하지 않는다.

## 결정론

- 365일 hash 1·2: `b039d35666b77fc2` / `b039d35666b77fc2`.
- current M1 두 capture: split `4eb703aad086c7aa`, resumed `d19d028857bb7485` 동일.
- economy-v5 두 capture: split `48e02ba01e8fddfe`, paused `263071b34065aa8d`, continued `373a5a6eeea13715` 동일.
- frozen save SHA256: `57b13c62ef1a6f88ade1851d0b3ae398157f95ca5bc246068eac3b2a964322fa` 보존.

## 금지 사항 점검

| 항목 | 결과 |
|---|---|
| 시뮬레이션 f32/f64·HashMap·thread_rng·SystemTime | sim/ai 소스 검색 일치 없음 |
| 코드 내 게임 수치·미결정 규칙 | 후보가 시뮬레이션 제품 코드를 변경하지 않음 |
| golden·fixture·의존성·에셋 변경 | 후보 diff에 없음 |
| 클린룸 | 원작 자료·외부 클라이언트 사용 없음 |
| 테스트 약화 | 전체 equality 유지, timeout·golden·fixture 축소 없음 |
| 검증 파일 무변경 | HEAD·추적 목록·505개 SHA·diff/status 전후 동일 |

## 작업 로그 증거와 실제 결과 대조

서버는 pinned tungstenite가 큐에 넣은 Close 응답을 다음 read로 flush한다. raw regression은 클라이언트의 TCP 종료 전에 응답과 EOF를 확인한다. 기존 query helper·경제 harness는 그대로다. 팔레트는 `deepStrictEqual`로 전체 객체와 전체 Uint8Array를 비교하고 update ranges equality도 유지한다. 기본 전체 테스트와 변조 거부 검사가 통과했다.

## FAIL 항목

지정한 새 검증 범위에서 없음. 초기 인수의 Close 실패와 palette timeout 기록은 후속 결과로 덮어쓰지 않는다. Firefox/WebKit·추가 M1 지도 suites·대규모 게임 성능·main 통합 및 해당 main CI는 이번 판정 범위 밖이다. 이 PASS는 M2 또는 전체 WIP 완료 판정이 아니다.
