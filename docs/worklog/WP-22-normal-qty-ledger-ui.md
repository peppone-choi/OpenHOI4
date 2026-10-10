# WP-22 정상 편제 Qty 기여 조회·툴팁 후보

## 범위와 출발점

producer PR15는 source `bbebbe111dcea346234f1249ed9d391fac095090`에서 실제
독립 source review required0 뒤13개 workflow/51개 checkrun 성공을 확인하고
정상 merge `f94f2c0376621f5f507fff0a4bfe562e4cbc418b`로 병합됐다.
같은 main CI7개도 모두 성공했다. 이 후속 후보는 그 main에서 시작한
`wp/22-normal-ledger-ui`다. 기존 producer 작업트리와 이전 증거는 보존했다.

이 변경은 선택한 정상 편제의7개 additive Qty를 실제 서버 조회→응답→
client validation→툴팁으로 연결하는 사용자 기능 하나다. 실제 Claude
계획의 versioned query에 capability-first·선택generation·호스트 예산을
보완한 사용자 승인 경계를 따른다. 독립 QA·리뷰·통합은 아직 대기다.

## 구현

- oh_proto는 별도 capability/result와 normal Qty DTO를 생성한다. legacy
  MilitaryView·MilitaryTemplateView·Hello/Welcome·protocol version을 유지한다.
- oh_server/session은 같은 sim thread에서 선택한 편제 하나의 aggregate,
  정의/상태 hash와 tick을 읽는다. 전체 MessagePack 응답을 호스트 전송
  예산으로 검사하며 초과 시 같은 request의 typed failure를 반환한다.
- client는7개 필드·순서·원시 Qty·role별 occurrence·참조를 검증하고 base의
  정의 hash/편제 존재/최종bits와 연결한다. 서버 합산을 JS로 재구성하지 않는다.
- 별도 조회 채널은 probe/data pending1개와 최신 선택을 관리한다. A→B→A,
  close/socket/scope 변경·CONNECTING·disconnect/stale을 구분하고 Delta마다
  정적 정의를 재조회하지 않는다. domain error는 다른 편제를 막지 않는다.
- 정상 편제에서만 Qty details 툴팁을 연다. ko/en, 키보드, mobile을 지원하고
  declared supply_use를 실제 사단 소비나 부족분으로 표시하지 않는다.

[ADR-2203](../adr/ADR-2203-normal-qty-ledger-channel.md)과
[재현 절차](../../tests/repro/WP-22-normal-qty-ledger/README.md)에 계약과
직접 검사 경계를 남겼다. 팩·canonical/hash/save·게임 규칙·의존성 변경은 없다.

## 작성자 검증

아래 통과 결과는 독립 QA가 아닌 작성자 실행 결과다. 최종 예산 수정 뒤
같은 소스의 실제 browser 검사도 다시 통과했으며 실패 이력을 보존한다.

| 실행 | 관측 결과 |
|---|---|
| cargo test -p oh_proto | 9개 통과, 실패/ignored0 |
| cargo test -p oh_server | 최종 실제 최대 fixture budget 포함23개 통과, 실패/ignored0 |
| npm --prefix client test -- --maxWorkers=1 | 마지막 수명 표시 보완 뒤 전체276개 통과 |
| cargo test -p oh_save --test m2_military_pack | 2개 통과, 현재 팩의 새 V7 checkpoint 생성 |
| 실제 Chromium 군사 Qty 기능 | 최종11개 통과, current/restored·legacy server·native domain/budget·injected lifecycle 구분 |
| production client + embedded server build | 통과; served JS bytes가 현재 dist와 동일 |
| cargo fmt --check / clippy -p oh_proto -p oh_server --all-targets -- -D warnings | fmt 통과·clippy 수정 뒤 exit0 |
| docs/diff | 최종 문서 오류0/경고0·diff 오류0 |

최대12combat/4support·64자 component/template·최종 raw Qty
8388608000000000000의 실제 전체 응답은30985bytes였다.256-byte 실제
호스트의 ledger-too-large 뒤 socket/legacy 군사 표시를 유지했다. 새 client를
포함한 실제 baseline 서버는 같은 request의 generic unsupported fallback을
보냈다. 정상 응답7개를 실제 ko/en 키보드/mobile 화면에서 열었다.
새 V7 복원에서는 tick144와 paused state hash668a2d664ea11fe8을 유지했고
6개 Training job과 정상 기여 값을 실제 응답으로 확인했다. 검사 소유 host4개는
exit0·잔존 PID0이었다. mock lifecycle 결과를 native feature로 확대하지 않았다.

## 실패와 수정 이력

- 첫 client requirement test는 구현 모듈이 없어 exit1이었다. 첫 Rust RED
  시도는 memory guard 중단이므로 assertion RED로 기록하지 않는다.
- 중첩 fixture module의 독립 serial이 같은 경로를 만들며 두 protocol
  테스트가 duplicate key로 실패했다. 공유 factory로 root를 먼저 발급한 뒤
  최대 입력을 만드는 helper로 고쳤고 원 시험을 그대로 통과시켰다.
- 두 작업트리의 공용 Cargo target이 이전 protocol 산출물을 재사용해 새
  서버 build가 export 부재로 실패했다. 해당 source timestamp를 갱신해
  실제 재빌드를 강제하고 baseline/current binary provenance를 분리했다.
- 이전 checkpoint의 팩 hash가 달라 복원 host가 PackMismatch로 거부했다.
  --force를 쓰지 않고 현재 팩에서 새 checkpoint를 생성해 실제 복원을 확인했다.
- fmt 차이와 clippy collapsible_if를 확인해 수정했다. lint를 완화하지 않았다.
- 마지막 budget 검토에서 잘못된 긴 request를 generic 응답에 echo하는 경로를
  고쳤다. 유효하지 않은 v1 request는 예산 안의 기존 invalid-message Notice로
  거부하며 실제 작은 예산/긴 request 뒤 legacy 정상 조회를 검사한다.
- memory/CPU90% guard 중단은 통과로 세지 않는다. 공유 캐시/기존 파일을
  삭제하지 않고 읽기 전용 page-cache advice와 작업 자식의 CPU2개 제한으로
  재시도했다. 마지막 Chromium의 peak memory14.76GB, quota 대비 샘플 CPU50.82%
  이었고 guard 중단이 없었다. 이 숫자는 전체 게임 성능 인수가 아니다.

## 남은 게이트와 범위 밖

단일 기능 draft PR 제출 후보이며 독립 QA·실제 Claude source review·exact-head
CI·정상 병합·같은 main CI·명시 연결 issue 정리는 대기다. optional timeout은 없으며
영원히 사라진 응답은 패널 close/reopen 또는 reconnect로 복구한다.
weighted 평균3개, Fx 최소 속도 원장, 군사 명령 UI·편제 편집·동적 부족분과
실제 소비·전체 WP-22/REQ-UI-04/AC-M2-03/M2 완료는 이 후보로 선언하지 않는다.
