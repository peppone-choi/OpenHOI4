# 검증 리포트 — WP-19 standalone 계산 단위

| 항목 | 값 |
|---|---|
| 판정 | PASS — 아래의 standalone graph/formula 부분만 |
| 검증자 | 구현 비참여 새 독립 검증 담당, 분리된 worktree, 추적 소스 무변경 |
| 브랜치·커밋 | `wp/19-supply-cloud`, `74bc16bb4278479d2742786a6e2513c251934317` |
| 검증 일시 | 2026-10-08 |

## 직접 실행한 검사

실제 checkout의 `oh_core`와 `supply/mod.rs`를 path로 참조하는 외부 Cargo 하네스를 사용했다. 제품 구현을 복사하지 않았다. 기본 하네스 재현 명령은 [모듈 README](../../crates/oh_sim/src/supply/README.md)에 있다. 독립 검증자는 별도 테스트와 출력 프로그램을 작성했다. 이 문서는 내부 실행 원장을 Git에 포함하지 않은 공개 요약이다.

| 검사 | 종료 코드 | 실제 결과 |
|---|---|---|
| 외부 하네스 `cargo test --offline --manifest-path <harness>/Cargo.toml` | 0 | 소스 22개 + 독립 9개 = 31개 PASS |
| 같은 하네스 `cargo clippy --offline --manifest-path <harness>/Cargo.toml --all-targets -- -D warnings` | 0 | 경고 오류 없음 |
| `rustfmt --edition 2024 --check crates/oh_sim/src/supply/mod.rs` | 0 | PASS |
| `python3 tools/check_docs.py` | 0 | PASS |
| `python3 tools/check_assets.py --release` | 0 | PASS; 새 deps/에셋 없음 |
| `git diff --check` | 0 | PASS |
| 별도 출력 프로그램의 새 process 2회 | 0 / 0 | 실제 DayResult raw 원장 64개 입력 출력 SHA 동일 |
| 전후 HEAD·추적 목록·SHA·diff/status 비교 | 0 | 545개 추적 파일 동일, 기존 baseline 536개 bytes 보존 |

보조 탐색에서 존재하지 않는 루트 LICENSE 읽기는 exit1이었다. 이후 실제 파일·의존성 inventory를 확인했으며 그 실패 명령을 라이선스 검사 PASS로 기록하지 않는다.

## REQ별 확인

| REQ | 독립 검사 | 실제 결과 |
|---|---|---|
| REQ-SUP-01 부분 | 4노드 graph 729개를 독립 simple-path oracle의 widest → shortest → 전체 경로 사전식 결과와 비교; 단절·통제·route tie·진행 component freeze | PASS |
| REQ-SUP-02 부분 | 작은 raw capacity/demand/attenuation 7,128개 ledger의 source/shared rail/loss 보존·입력 반전; 닫힌 공식 capacity/loss 단조성 390개 | PASS |
| REQ-SUP-02 부분 | scalar 1,025개 비율점의 범위·양수 time, 엄격 threshold·grace 5일/손실 6일·회복/무수요 reset | PASS |
| REQ-SUP-04 부분 | 고립·제외 edge·수도 통제 상실·도달 불가 수요와 정상 controls | PASS |
| 오류 계약 | ID0 유효; duplicate/ref/음수/cost/합계/path/attenuation/counter/reciprocal overflow 거부, 입력 불변 | PASS |

소스의 1,323개 보존 예제·100,620개 단조성 trial도 실행했다. 고정 weights 때문에 대체 보급원 용량이 남을 수 있는 한계를 명시적으로 검사한다. 이를 adaptive routing으로 바꾸지 않았다.

## 결정론과 범위

- 새 process 1 SHA256: `877d0ab72e53ddc4edde062e9fa28951cae0e2a17905a3c20a571ef4b59df9ae`.
- 새 process 2 SHA256: `877d0ab72e53ddc4edde062e9fa28951cae0e2a17905a3c20a571ef4b59df9ae`.
- 이는 standalone DayResult 출력이며 전체 Simulation hash·save/resume 결과가 아니다.
- 검증 snapshot digest: `b9445c804f94f24f8fdc45aeda9fb4a55b80f0a89d01eb9dcca3c72cc567d63a`.

## 금지 사항과 작업 로그 대조

새 source unit은 float·HashMap 순회·RNG·시계/I/O·unsafe를 사용하지 않는다. 새 수치는 `defines.toml`의 provisional 입력이며 아직 pack-authoritative loader가 아니다. 원작 자료·신규 의존성·에셋·기존 골든 변경은 없다. 구현 작업 로그의 22개 검사와 별도 출력 해시를 확인했고 독립 원장 프로그램의 입력·해시는 따로 기록했다. 구현 시점의 ‘독립 검증 대기’ 문구는 역사 기록이며 이 후속 판정과 구분한다.

## 남은 범위와 통합 게이트

모듈은 `oh_sim/lib.rs`에 등록되지 않았다. 기존 workspace CI는 이 코드를 compile/test하지 않으므로 위 별도 QA를 대체하지 못한다. 실제 pack/World 어댑터·division demand/strength/org/attack·이동 leg 소비·전체 step rollback·canonical/hash·save·wire/query·server/browser는 미구현/미검증이다. 공유 변경은 WP-15 인수 뒤 직렬 진행한다. 후보 통합과 같은 main CI는 이 문서 작성 시 후속 확인 대상이다. 전체 WP-19/REQ·M2 완료는 아니다.

## FAIL 항목

지정한 standalone 범위의 FAIL 항목 없음. 미구현 consumer와 runtime 통합을 PASS로 확대하지 않는다.
