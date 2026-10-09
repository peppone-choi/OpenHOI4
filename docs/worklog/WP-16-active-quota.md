# WP-16 작업 로그 — 국가별 활성 훈련 한도와 V8 경계

작성: 최병호 · 2026-10-09

| 항목 | 값 |
|---|---|
| 상태 | 구현 담당 검사 통과, 실제 독립 delta 리뷰 대기 |
| 기준 | PR #9 후보 `b9c0b5639c68f7884faf037991c655edbdeb2df5` |
| 범위 | REQ-MIL-03/09 훈련 접수·복원 한도, 기존 우선순위 명시, 확장 이력 저장 태그 |

## 승인과 변경

사용자는 전 세계 평생 누적4096건 제한을 없애고 국가별 진행·배치 대기 훈련만 기술 상한으로 계산하는 제안을 승인했다. 기존 크기4096을 유지하며 Pending·Training·Ready가 계산되고 Cancelled·Deployed가 슬롯을 반환한다. Ready 도달만으로 배치 대기 슬롯을 해제하지 않는다. 실제 독립 리뷰의 상세 계획에 따라 template MAX_ENTRIES와 runtime MAX_ACTIVE_JOBS_PER_NATION을 분리하고 ActiveLimit을 추가했다. 기존 transport의 invalid-message 매핑은 그대로다.

Train 적용에서 국가별 한도를 ID 증가 전에 검사한다. 복원/상태 검증도 같은 국가별 합계를 검사한다. 기존 copy/swap 원자성, 소유·자원 회계, 전체 연속 이력0..next_job_id와 다음 사단 ID를 보존하며, terminal 기록을 삭제하거나 재번호·재활용하지 않는다. 우선순위는 기존 오름차순으로0이 최우선이며 ID tie-break를 유지한다.

실제 리뷰의 저장 계획을 반영해 V7은 전체 이력4096건 상한을 그대로 유지한다. 기존 범위의 writer는 계속 V7로 같은 bytes를 만들고, 전체 기록이4096을 넘는 새 상태만 V8 prefix/header 태그를 사용한다. V8 body는 V7과 같은 canonical 필드·enum 순서·정의 identity·상태 hash다. V8을 V7로 relabel하면 force 여부와 관계없이 거절된다. 기존 V7·V1~V6·생산 입력은 reset/migration 없이 보존한다. [ADR-1602](../adr/ADR-1602-training-ownership.md)에 경계를 명시했다.

## 실패와 직접 검사

처음 작성한 새 경계3개는 기존 코드에서 모두 실패(exit101): 종료4097건 및 두 국가8192건 복원은 military:InvalidValue, 한 국가4096건 상태에서 다른 국가 Train도 InvalidValue였다. 한도 수정 뒤 기존 V7 검사와 함께6개가 통과했다. 실제 리뷰 계획을 받은 뒤 V8 태그와 기존 V7 거절을 추가하고 새 범위 검사를 확장했다. 새 save 검사 첫 컴파일은 private module 경로와 LoadedSave Debug 미구현을 잘못 사용해 실패했다. 공개 write_atomic과 err 추출로 검사 준비 오류만 고쳤다.

최종 `cargo test -p oh_sim --test military -p oh_save --test military --locked --offline`는18개 PASS(exit0)다. 새 검사7개는 종료4097건, 두 국가 각각4096건, 세 active 상태, 국가 격리, 취소/배치 슬롯 반환, 실패 ID 미소비, 같은 step 접수 순서와 cancel-before-Train, semantic rejection의 상태 보존, terminal 재시도, 복원 후 큐 중복 거절, hash/canonical/save bytes 동일성, 정확한 terminal/division identity, 구 V7 거절과 V8 왕복, 저장 byte/entry/allocation/file 경계에서 이전 정상 atomic 파일 보존을 검사한다. 0 대 65535 우선순위를 사단/군 ID 반대로 배치해 실제 보충 순서도 확인했다. 큰 기록 경계 fixture는 실제 authority가 만든 레코드를 복원 경로로 확장하며4097회 실제 훈련을 돌렸다고 기록하지 않는다.

전체 `cargo test --workspace --locked --offline`의 첫 시도는 공유 target이 다른 worktree의 군사 모듈 없는 라이브러리를 재사용해 컴파일 단계에서 실패했다. 해당 라이브러리 캐시를 갱신한 새 실행은383개 PASS·실패0·ignore0(exit0)이며 native13개와 기존 저장/복원·legacy·서버·결정론 검사를 포함한다. `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`도 exit0이다. 기존 ts-rs serde attribute 경고는 남았으며 새 lint 실패는 없다. fmt·문서·whitespace 검사를 수행한다. 이전 바이너리·증거는 bytes 검증 후 별도 압축 보존하고 기존 source/worktree를 보존했다.

## 계약과 남은 경계

큐의 동일(tick,nation,sequence) 거절과 기존 wire의 연결별 sequence 거절을 유지한다. 새로운 연결 간 영속 request ledger는 추가하지 않는다. 기존 원자적 production/daily 단계 rollback 검사를 보존했다. terminal 이력의 저장·조회·복제는 여전히 기록 수에 비례하는 memory/CPU/bandwidth를 쓴다. 기존 one-million-entry·256 MiB body/allocation·64 MiB file 한도와 nested allocation charge는 독립적으로 남는다. 무제한 신뢰 가능한 저장이나 이력의 자동 보존 기간을 약속하지 않는다. Pagination·archive·retention·전체 WP-16/M2는 별도 범위다.

독립 delta 리뷰와 최종 SHA CI/main 통합은 대기한다. O2 데이터 팩·O3 경제 UI에 이 수정의 소스를 섞지 않았다. 내부 raw 로그·실행 파일·이력 확장 fixture 실행물은 Git에 포함하지 않는다.
