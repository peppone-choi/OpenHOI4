# M2-r1 독립 검증과 producer 경계

2026-10-07. 부모가 수행하는 앱별 exact P-05 준비이며 전역 하네스/강제 리뷰 기능을 활성화하는 지시가 아니다.

## 최초 baseline

구현자의 실제 final/HEAD/clean status/worklog/producer 파일이 존재하는지 확인한 뒤 그 HEAD의 새 detached worktree를 만든다. 사전에 필요한 최신 기획/원 증거 사본은 ignored target에 준비하고 경로·SHA를 기록한다. 부모 `.orchestrator/m2_snapshot.py`로 앱 생성 **전에** HEAD·raw index bytes/semantic index·전체 tracked list/각 SHA256·diff/cached/status를 저장한다. 검증은 `git --no-optional-locks` 또는 해당 process의 GIT_OPTIONAL_LOCKS=0으로 선택적 index refresh를 피하고 원 raw baseline을 유지한다. 이것은 승인/샌드박스/네트워크 설정 변경이 아니다. 원 baseline은 절대 교체하지 않는다.

검증자는 자기 target에 before/after와 원 명령 stdout/stderr/exit를 보존한다. 추적 파일을 바꾸거나 index를 복구하지 않는다. 종료 뒤 부모도 전체 snapshot을 다시 얻어 최초와 직접 비교한다. raw 또는 tracked mismatch는 무효로 기록하고 새 앱 검증이 필요하다. 구현 branch green과 통합 main green은 별개다.

## 담당별 독립 확인

| WP | 정확 설계 문서 | 원시 기대와 실패 검사 | 후속 경계 |
|---|---|---|---|
| WP-24 | WP-24-schema.md 및 담당 ADR | graph permutations/diamond/cycle/version/conflict/missing; 실제 지도/국가/주 key/ref; Fluent multiline/select/term/attribute·누락/중복/순환; file/Unicode line/column·warnings exit; root 무관 identity 및 source byte 변조; 실제 CLI/CI checker | WP-13/14의 새 registry는 producer 구현 뒤 연결; patch/UI는 WP-35 |
| WP-17 | WP-17-schema.md/ADR-1701 및 REQUEST-0006/0007의 실제 채택 상태 | 거리10/속도2×시간배율1.5=7.5h, 7/8/15틱; shortest/tie/noncontinuousID/noPath/foreign owner; 해협 명시계수2의9/10틱·대체8h·kind/하천슬롯 분리·missing/ref/0/음수/overflow/underflow; 중간edge·정지/paused·실패 state/queue/clock; 모든 신규상태/예약 hash sensitivity; exact v1/v2 save/restart 구조 | unit/template 생성 WP-16, infra/supply/통행 갱신 WP-14/19/20, 실제 UI WP-22; 미연결을 전체게임 증거로 쓰지 않음 |
| WP-13 | WP-13-schema.md/담당 ADR | 키 하나 AST/인자/scope/미존재ID·깊이16/17·효과1000/1001; side effects 없는 조건; registry와 실제 adapter 차이; 실제 종료시간/원인/가중치 및 save/restart/hash; malformed wire가 생기면 회귀 | 아직 없는 경제/장비/전쟁 adapter는 WP-14/15/20, 의제/이벤트는 WP-27 |
| WP-25 | WP-25-schema.md/담당 ADR | 원 snapshot+ordered 명령의 fresh native replay 및 hash one-bit/identity/명령/ZIP member·경로/size/version 오류; pending 중복 금지; bench threshold 정확15% 및 초과 두회·0/NaN/missing; CI 기준과 current 같은 runner 연속 | 전체 M2 actual state/3OS/AI/player 경로는 후속 최종 게이트 |
| WP-14 | WP-14-schema.md 및 실제 잠정 규칙 문서 | 독립 integer Fx ledger/IC allocation/resource/construction/manpower/PC/law·invalid/overflow/atomicity; 실제 data/state/command/save/wire·runtime/3OS | 생산·장비 WP-15, 인력소비·편제 WP-16, 공급 WP-19 |
| WP-23 | WP-23-schema.md와 WP-14 실제 producer 계약 | 6국·100~200 province/ID/reference·전용scenario 초기값·missing/schema/version·실제 validate; source/asset records, M0/M1 bytes 보존과 별도 pack identity | 시스템 전체 플레이는 WP-21/22/게이트 |

## v1 frozen producer

현재 원 committed m1-v1 save는 helper의 live testland copy와 비교되므로 새 pack.ftl 추가만으로 identity가 바뀐다. WP-24가 b6516ed 원 Testland 19파일의 frozen v1 fixture를 추가하고 `copy_v1_pack/mutable_v1_pack` 별도 경로를 save_fixture capture에만 연결한다. 기존 current generic helper/테스트와 원 assertion/expected/save 파일은 유지한다. WP-24 소유는 support/copy.rs·mod.rs, examples/save_fixture.rs, 신규 fixture directory이며 WP-17은 이 경로를 수정하지 않는다. 독립 검증은 Git 원 source의 각 bytes와 frozen copy, 원 v1 fresh capture의 byte/SHA 일치, current pack save 왕복, 변경팩 old save 명시 거부를 직접 각각 확인한다.

## WP-24 첫 FAIL과 새 P-05 경계

exact `c2c2f638fe5179d11ce5a318f6d991b4b2a0cd4d`의 첫 독립 P-05는 유효 FAIL이다. 등록 legacy scenario defines의 문법 오류를 CLI가 수용했고, native 서버 startup은 잘못된 dependency와 깨진 FTL에도 HTTP 200을 열었다. 실제 앱 마지막 전문은 `docs/verify/WP-24.M2-r1.attempt1.md`, 원 명령·별도 fullreport·최초 9771개 추적 파일과 raw index·최종 불변성·전수 ZIP 봉인은 `docs/verify/evidence/WP-24-M2-r1-P05-1/`에 있다. 수정 구현 결과로 이 기록을 대체하지 않는다.

새 source는 새 detached worktree와 최초 기준선, 새 검증 앱으로 검사한다. 서버 startup와 실제 save restore, CLI validate의 등록 defines를 직접 확인한다. 잘못된 dependency/version/engine/conflict/schema/reference/FTL/defines는 거부되고 HTTP/WS가 열리지 않아야 한다. 정상 startup의 HTTP/WS와 정상 종료 코드 0은 검증자의 강제종료 결과와 분리한다.

REQUEST-0008 A는 한정 CEO 잠정 채택이다. 팩 밖 engine-owned 정책의 허용 caller·실제 v1 header/context와 원 3·19파일의 전체 경로 집합, 각 길이·SHA256, FNV·팩 ID/버전·시나리오·kind·원천을 모두 대조한다. 원천은 `docs/decisions/evidence/historical-load-policy/source-originals.zip`의 22개 원 bytes다. 파일 추가·삭제·한 바이트 변경·symlink/reparse·검증 중 변경, 다른 caller/scenario/key/lang/format, plain frozen 19파일, current pack, `--force`의 면제 시도를 직접 검사한다. 정확한 `manifest.name_key=testland_name`의 ko/en `missing_message_value` 두 진단만 `warning_unavailable`로 보존하며 진단 위치·키·언어·원인을 유지한다. 일반 LOC-03 성공과 구분한다. strict standalone 원 M0 거부, 기존 headless M0 run 계약, 원 v1 fresh resume 전체 상태·hash와 codec의 모든 거부 검사, 실패 후 원 저장·팩 bytes 보존을 각각 확인한다. 원 fixture/expected/golden 및 reader/writer를 바꾸거나 강제 migration으로 통과시키지 않는다.

## 저장 CI의 actual producer/consumer와 무효 검증

통합 c28의 새 WP-17 P-05는 필수 Save workflow가 원 frozen mutable-v1 capture를 서버 positive 입력으로 소비하는 실패를 직접 확인했다. REQUEST-0008은 그 원천의 CLI 재개만 허용하므로 현재 정책의 서버 거부는 그대로 유지한다. 원 frozen 두 capture/fixture/expected/CLI/3OS comparer를 보존하고 별도 current 두 capture·fresh native 전체 DTO/canonical/hash·원장·신원·3OS 비교를 기존 server positive에 연결하는 좁은 WP-24 P-06을 수행한다. HTTP/WS/query/servedJS/정상 종료 단언을 유지하고 역사 서버 native1·무HTTPWS는 별도 negative로 검사한다. 기존 checker의 단언을 줄이거나 server policy entry·codec/header/golden을 바꾸지 않는다.

같은 P-05는 tracked TS generator가 동일 bytes를 다시 쓴 뒤 protocol.ts 한 entry의 mtime cache가 달라져 raw index 불변성 FAIL이었다. 실제 generator 호출·mtime 구간은 확인됐고 index 갱신 주체는 미확인이다. read-only req_net_04 equality test 자체는 쓰기를 하지 않는다. 원 최초/최종 raw와 report·ZIP을 보존하며 source/semantic 동일성을 근거로 면제하지 않는다. 다음 새 exact P-05는 tracked examples/generate를 실행하지 않고 read-only equality 또는 actual typescript()의 ignored 출력 bytes와 기존 tracked TS를 비교한다. 원 index·mtime·baseline을 복구하거나 기존 시도를 PASS로 바꾸지 않는다.

## 원본 보존

앱 read_thread의 실제 final 전문을 저장한다. 요약된 wait text를 원 리포트 전문으로 대신하지 않는다. command receipts/manifest·ZIP member 전수 bytes/SHA를 부모가 대조하고 source/commit/최초·최종 snapshot·app status/cursor/원 prompt를 기록한다. 최초 FAIL/환경 준비 오류와 수정 새 exact PASS를 구분한다. 예상 CI/명령은 실행 증거가 아니다.
