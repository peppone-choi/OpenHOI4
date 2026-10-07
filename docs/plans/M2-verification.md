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

## 원본 보존

앱 read_thread의 실제 final 전문을 저장한다. 요약된 wait text를 원 리포트 전문으로 대신하지 않는다. command receipts/manifest·ZIP member 전수 bytes/SHA를 부모가 대조하고 source/commit/최초·최종 snapshot·app status/cursor/원 prompt를 기록한다. 최초 FAIL/환경 준비 오류와 수정 새 exact PASS를 구분한다. 예상 CI/명령은 실행 증거가 아니다.
