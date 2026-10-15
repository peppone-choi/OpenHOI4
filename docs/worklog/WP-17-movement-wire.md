# WP-17 S1: 이동 명령 wire와 이동 조회

## 작업 경계

- taskId: `openhoi-s1-movement-wire-20261015`, stage: implement, 담당: openhoi-implement (고정 Sol).
- contract: `be6b4ef3-6ceb-4ada-a800-b0077be15519`.
- exact base: `fb1539adc231e34eccf42895d46d456448bff1dc` (PR21 head), main 관측값: `33d6fb19388477b3687102e5c2a52978d6cf37ac`.
- 별도 브랜치: `buzz/openhoi-s1-movement-wire-20261015/openhoi-implement`.
- 계획 SHA256: `a1cd3bb503c00340528f195d55220308cdb603b3b8cd36af8daeca983e41040f` 실제 대조.
- 착수 시 GitHub 열린 PR은 #21 하나였고 head/base가 위 값과 일치했다. worktree 목록·PR-QUEUE·활성 세션에서 중복 S1 구현은 없었다.
- PR21 영수증 차단을 유지한다. PR21 변경·병합·기존 브랜치 덮어쓰기는 하지 않았다.
- **PR21 병합 전 S1 병합 금지. 병합 뒤 main과 정합/rebase 및 새 exact-head 검증이 필요하다.**
- S2, M-1, 지도 로딩, 국가 레이어, 데이터·에셋·저장 형식, oh_sim/src/lib.rs는 변경하지 않는다. 1936 요구 유지, 1939 가정 없음.

## 구현

`MovementCommand {sequence, command}`의 Move{division,destination}/Stop{division}만 추가했다.
Join된 player_nation만 권한으로 쓰고, 기존 군사 명령과 같은 canonical 단조 sequence·관전자/not-joined 거부·sim.enqueue/step·CommandResult 뒤 Snapshot 흐름을 따른다.
ID는 기존 DivisionId의 u32를 decimal text로 운반한다. 클라이언트 속도·계수·allowed·국가는 받지 않는다.

`Query{kind:"movement"}`는 같은 sim thread에서 MovementView의 state_hash/tick와 Unit getter를 읽는다.
route는 현재 위치 다음부터 목적지까지, 시간은 기존 FixedValue I32F32 decimal/bits이다.
현재 간선이 없으면 current_leg_hours=0이고, 이동 컨텍스트가 없으면 빈 units로 성공한다.
계산/직렬화 실패는 simulation-error로 반환한다. 기본 이동·Stop 규칙은 그대로다.
movement.rs에는 current_leg_hours 읽기 getter 하나만 추가했다.

ADR-1702는 ADR-1603/2203 선례로 PROTOCOL_VERSION=m0-v1과 기존 DTO·바이트를 보존하는 이유를 기록한다.
TS는 hosted CI run 38822260564 client job 116721369216의 실제 ts-rs 생성 diff를 회수해 반영했다.
새 DTO 필드 문서 주석이 만드는 줄끝 공백은 생성기의 기존 선행 선언 loop와 같은 trim_end 처리를 후행 선언 loop에도 적용해 정규화했다.
타입을 수동 작성하지 않았으며 최종 CI의 재생성/no-diff 검사가 이를 대조한다. UI 코드는 수정하지 않는다.

## 검사와 증거

- RED commit `24a6f21f1f297af89853ecf9529b6e303be753db`: GitHub CI run `38821667927`, Ubuntu job `116719437062`에서 unknown variant MovementCommand로 0 PASS/1 FAIL. `reports/s1/red-ubuntu.log` 보존.
- Protocol 검사: canonical ID/숫자·추가 authority 필드 거부, wire roundtrip, 빈 context, 실제 Train→Ready→Deploy, Unit getter 대조·상태 불변, 간선 합/elapsed, Stop·도착·군사 roster 일치.
- Native 검사: `movement_wire.rs`가 기존 m2 팩의 독립 복사와 Ready save를 만든 뒤 실제 executable + Node WebSocket MessagePack으로 검사한다. `cargo test --workspace`에 포함되어 기존 CI server 3 OS에서 실행된다.
- Native 거부: foreign unit Move/Stop, 없는 ID, 허용 밖/없는 목적지, canonical 오류, 공용 sequence 중복, 관전자, not-joined, 합성 disconnected allowed snapshot의 NoPath. paused 상태의 movement/military 전체 view·state_hash 불변을 대조한다.
- Native 성공: 인접 Move 수락·route, ACK→Snapshot 순서, 다중 간선 이동 중 Stop의 현재 간선 보존, 최종 도착과 Snapshot 이후 military province 일치, 사단 없는 빈 성공.
- 테스트 fixture의 disconnected allowed는 save DTO의 검증된 읽기 복원 API로 만든 **시험 입력**이며 제품 팩·이동 규칙은 바꾸지 않는다.
- 소유 server process는 finally/watchdog로 정리하고 PID 부재를 검사한다. Unix SIGINT exit0, Windows는 소유 child의 명시 종료이다. Windows graceful Ctrl+C 인수는 기존 lifecycle.py의 별도 검사이며 이번 native 종료를 그 성공으로 부르지 않는다. GUI/HOI4 실행 없음.
- 로컬 경량 rustfmt 1.99.0, Node syntax, docs, diff check PASS. architecture는 cargo 부재로 로컬 NOT_RUN이며 기존 hosted CI에서 실행한다.
- draft PR #22: https://github.com/peppone-choi/OpenHOI4/pull/22 . main diff에는 PR21이 포함되므로 S1 리뷰는 exact base부터만 비교한다.
- 최종 exact head와 CI 결과는 아래 인계 및 담당 reports/dispatch 산출물에 기록한다. 진행 중 결과를 PASS로 쓰지 않는다.

## 미실행과 다음 단계

독립 리뷰, 병합, 실제 세계지도·클릭 이동·화살표·움직이는 엔티티 제품 QA, WebGPU, Windows GUI, 원작 HOI4, GCP, 운영 배포, 게임 초기화는 NOT_RUN이다.
부분 서버 검사로 전체 제품/실제 지도 QA 완료를 선언하지 않는다. 산출물은 main에 반환하고 다른 담당을 배정하지 않는다.


## 승인된 transport 보완 및 비강제 main 정합

총괄이 질문 `e8fcf5c4-a6e5-4882-ad68-7e08706b0c6c`의 기획 답변을 확인하고 H1–H4 범위를 승인했다.
보완 원문 SHA256: `d6cdf2955da3d71d0a9a24a245eb84f9ff7778e6972181e4117edea445c568c9`.
`network.ts`와 `networkValidation.test.ts`만 제품 소유에 추가한다. 기존 guard와 satisfies, P-06 valid/invalid 및 fixture 6건은 유지한다.
새 guard는 exact envelope/view/unit/fixed, canonical u32/u64, dense route/units, division 수치 오름차순·중복 거부,
I32F32 half-even value/bits 일치만 검사한다. 게임 의미·UI·S2·지도는 변경하지 않는다.

- 원 후보 head `ceccc9adf2d61a5031a79e74a4176adac9c32f79`, RED `24a6f21f1f297af89853ecf9529b6e303be753db`.
- 원 base `fb1539adc231e34eccf42895d46d456448bff1dc`와 PR21 squash main `3b7ac90f4420043e50a2f8605c34a7327d9bd01d`의 tree는 실제 git 확인에서 모두 `59617d3784d57e8f8046ec303cdf6aa9049716c2`다.
- rebase 게시의 강제 갱신을 피하기 위해 동일 main tree를 merge했다. 정합 commit `b1d637192712d5322116c22c0db8852858d58d96`의 tree는 원 후보와 동일하다. 원 PR21 코드 중복 변경 없이 S1의 HANDOFF 추가만 보존해 문서 충돌을 해소했다.
- 원 head와 RED는 `archive/openhoi-s1-pre-sync-ceccc9a`, `archive/openhoi-s1-red-24a6f21` ref 및 `reports/s1-amend/pre-sync.bundle`로 보존했다. 기존 `reports/s1/`의 모든 파일 SHA를 별도로 기록했으며 덮어쓰지 않는다.
- 원 실패 CI `38822700197`/`38822695989`의 TS2741 및 RED CI `38821667927`의 unknown MovementCommand 실패는 그대로 보존한다. 새 시험의 RED를 별도로 꾸미지 않는다.
- 이 보완까지의 exact head·14파일 diff·새 hosted CI 실행/로그는 `reports/s1-amend/`와 별도 RESULT에 기록한다. 과거 NOT_RUN을 과거 PASS로 변경하지 않는다.

앞 절의 stacked/PR21 차단 설명은 당시 사실 기록이다. 현재 PR21은 외부에서 병합되었고 본 후보는 main을 조상으로 포함한다.
PR22의 최신 diff는 main 기준 S1 12파일+transport 보완 2파일만이다. 새 exact-head 독립 리뷰와 검증은 총괄이 다음 담당에게 배정하며 이번 구현에서는 NOT_RUN이다.
