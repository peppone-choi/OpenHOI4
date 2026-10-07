# ADR-2501 실제 CLI 세션 재현과 고정 기준 성능 비교

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-25, REQ-SAV-05, REQ-PERF-03 |

## 맥락
02 §7.3은 입력원과 paused pump, ZIP 하위 스키마를 정하지 않았다. 실제 oh_server Session은 TimeCommand를 enqueue 뒤 step하고 timeout에도 step한다. Simulation 공개 enqueue/step과 SaveContext/codec으로 독립 CLI Host를 구성할 수 있어 서버 파일 변경은 필요 없다.

## 결정
`oh repro record --out x.zip [--pack ROOT --scenario ID --seed N | --load SAVE --pack ROOT]`는 실제 headless 게임 Host다. 표준입력 JSON 한 줄을 즉시 Simulation에 적용하고 실제 결과를 stdout으로 flush한다. EOF는 기록 종료다. enqueue(tick,nation,sequence,command), pump, step만 제공하며 pump/step은 각각 Simulation::step 한 번이다. paused일 때도 명령을 처리하며 자동 wall-clock step은 없다. 입력 순서가 권위이고 예약 큐 키 정렬과 구별한다. 기존 Pause/SetSpeed/Move/Stop/Effects만 제공하고 새 게임 규칙을 만들지 않는다.

시작 저장은 기존 bounded v1/v2/v3/v4 codec으로 전체 상태·큐를 담는다. 기존 큐는 시작 snapshot에서만 복원하고 새 입력만 로그로 enqueue한다. machine commands.log는 순서·실행전 tick·실제 enqueue/step 결과·실행후 hash를 저장한다. commands.txt는 machine log에서만 생성해 대조한다. 의미오류는 성공 step의 command results로 기록되며 소비되고 whole-phase error는 Err와 보존된 queue/clock/hash로 기록된다. 종료후 enqueue/step 거부도 실제 결과다. 종료 report는 version별 전체 DTO·canonical hex·hash·ended를 담는다. final.json은 Host의 canonical JSON bytes와 직접 대조하여 untrusted Value tree를 할당하지 않는다. Event array는 선언된 events 수를 넘는 다음 항목을 할당하기 전에 거부하고, phases array는 host policy32, command results는 기존 save queue 상한으로 제한한다. 재생은 각 경계와 최종 report를 계산해 비교하며 기대 hash 불일치는 native exit 1이다. 실패는 로컬 candidate에만 영향을 주고 입력저장/팩/기존대상파일을 변경하지 않는다.

bundle schema 1 필수 필드: engine_version, scenario, national, pack_id/version/hash, effective_defines_hash, seed, start_save, start_tick/hash, end_tick, expected_hash, events. 기본 새 세션은 현재 national Testland m1이며 팩은 CLI --pack 또는 기본 national testland/M0 경로에서 Active validation하고 caller purpose/force를 받지 않는다. 기존 정상 resume 호환 경로는 그대로다. 원 M0 예제의 Active 현지화 오류는 거부하며 caller가 호환 purpose를 선택하지 못한다. start_save=false는 context/seed 초기화 후 start tick/hash로 검증한다. writer는 항상 시작저장을 제공한다.

ZIP32 stored(method 0)만 지원한다. start.ohsave 자체는 기존 zstd codec으로 압축·검증한다. deflate/encryption/ZIP64/extra/data-descriptor/comment/multi-disk는 명확히 거부한다. 중앙/local header·순서·offset·CRC·실길이·선언길이를 대조하고 허용 root 이름만 받는다. 절대/drive/UNC/../backslash/NUL/duplicate/alias/symlink를 거부하며 추출하지 않는다. host 예산은 crates/oh_save/repro-defines.toml에 둔다. 원파일/출력은 regular path만 허용하고 output은 sibling tempfile sync 뒤 persist한다.

성능 driver는 고정 SHA `81deb803944cf6297f13b3f14ca454774cea141b`와 현재 checkout의 oh_cli/oh_sim/oh_core 실제 library를 동일 소스로 각각 별도 release build한다. baseline에 bench CLI가 없어도 같은 driver로 실제 library를 실행한다. 로드·빌드·warmup은 측정 밖이다. 정의된 seed/step 수만큼 national Simulation을 실행하여 native Instant nanoseconds와 전체 종료 DTO/canonical/hash를 출력한다. runner는 baseline/current를 2쌍 연속 측정하고 정확한 정수 교차곱으로 두 쌍 모두 >15%일 때 실패한다. 실패·0·누락·비정수·음수·workload/fullstate 불일치는 오류다. Python 정수는 overflow가 없다. baseline은 git archive로 분리한 읽기 전용 소스이며 수정하지 않는다. 새 external workspace의 lock은 각 source lock을 복사한 뒤 offline metadata로 root 추가/불필요 workspace package 정리만 허용한다. registry package/version/checksum이 원 lock과 달라지면 오류이며 build는 --locked다. 원 source lock은 변경하지 않는다. Linux CI가 gate 증거이고 Windows는 local driver 증거로 표시한다. 새 workflow는 main push/PR/workflow_dispatch와 검증 브랜치 codex/wp25-repro-bench-m2r2 push에서 동일 job을 실행한다. 신규 workflow의 branch-only 최초 dispatch에 의존하지 않는다. 부모만 exact 브랜치 게시를 담당한다. 기존 6 workflow/jobs와 Save·Trigger 3OS 검사는 보존한다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| seed만 재실행 | 단순 | snapshot/pending/pump 유실 |
| 서버 hook | 브라우저 세션 직접 기록 | CLI Host용 공개 API가 이미 존재 |
| current 자체 baseline | 편리 | 고정 기준 위반 |
| ZIP 추출 | 범용 | 파일 쓰기가 불필요 |

## 결과와 영향
기존 hash/save codec·골든·게임 defines를 보존한다. CLI 진단/protocol만 추가하며 화면 문자열은 추가하지 않는다. 기존 lock의 serde_json =1.0.151(MIT OR Apache-2.0)을 normal dependency로 재사용한다. 공식 확인(2026-10-08): https://docs.rs/crate/serde_json/1.0.151 및 https://github.com/serde-rs/json . ZIP layout: https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT . 동일 runner: https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job . M3/M5 성능목표·브라우저/WebSocket 기록은 이 headless CLI 증거로 선언하지 않는다.
