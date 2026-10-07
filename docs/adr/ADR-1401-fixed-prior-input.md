# ADR-1401 고정 prior 측정 입력과 실제 library source 분리

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-14, WP-25, REQ-PERF-03 |

## 맥락
WP23의 신규 m2 파일은 전체 testland hash를 바꾼다. 기존 current pack 대 prior pack 비교는 동일 m1 workload 측정과 콘텐츠 검증을 섞는다. 인수 지시에 따라 policy와 원 실패를 보존한다.

## 결정
prior81deb git archive의 팩 전체를 같은 immutable 경로로 두 library에 제공한다. current는 실제 current commit source이며 prior code로 대체하지 않는다. pack inventory의 모든 상대경로/크기/SHA 및 디렉터리를 생성 직후와 build/load/warmup/sample 전후 대조한다. source/lock/metadata/path dependency/registry/driver/binary provenance를 원증거에 보존하고 binary도 재검사한다. 실제 DTO/canonical/hash comparer를 유지한다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| m1 파일만 선택 | 작은 입력 | 원 전체 pack/identity가 아님 |
| current에서 신규 파일 제거 | 경로 단순 | source 콘텐츠를 prior처럼 편집함 |
| current를 새 baseline으로 | 비교 쉬움 | 고정 prior 정책 위반 |

## 결과와 영향
두 실제 release binary가 prior 전체 pack path를 공유하며 read-only 권한과 content 재검사로 변조를 실패 처리한다. 새 콘텐츠 경제/native/repro와 후속 UI 성능은 별도 증거가 필요하다. dependency·CI·timeout·15% 조건·baseline은 바꾸지 않는다. 외부 버전 선택이나 새 의존성 없음.

## 추가 소유 인수와 역사 경계

`bench/driver.rs`·`bench/compare.py`·`bench/test_compare.py`는 초기 좁은 소유 범위 밖에서 변경됐다. 부모의 보류 뒤 원/working bytes와 diff를 보존하고 CEO가 구체 예외를 조건부 잠정 채택했다. 인수 원문은 `target/evidence/WP-14-M2-r2/planning/bench-driver-additive-CEO/CEO-review.md`, `parent-contract.md`, `CEO-direct-review.json`이다. 소급 승인·원 driver bytes 동일·기존 실측 PASS로 기록하지 않는다.

원 driver 1711 bytes SHA256 `19f8b6c5f821575ea603a416ce80eccfe4976a0a742606fe0986330f1b172663`, 인수된 driver 2076 bytes SHA256 `999502205cd0f75ff2db016b53c29166bb4b09ecbdefed4291aabbee61cd65f7`를 별도로 보존한다. 추가는 run_national/Instant 이전과 elapsed/원 완료 검사 이후 실제 pack_hash, before-after 거부와 출력 두 필드에 한정한다. 원 timer~step~elapsed/완료 검사 문자열과 regression 함수 AST는 기술 검토에서 exact였다. 원 m1/seed/steps/warm/pairs/15%/timeout/release/default features/lock/registry는 유지한다. 비용은 타이머 밖이며 원 native timeout 안이다.

새 driver bytes/SHA를 양 실제 library의 src/main.rs와 대조한다. 두 hash 필드 각각의 누락/null/길이/hex, 표본 내부·두 쌍·양쪽 hash 불일치, 모두 같은 잘못된 hash도 warmup prior-native 바인딩과 다르면 거부하는 회귀를 추가한다. 원 DTO/canonical/hash/완료/elapsed 및 파일 identity·linkage 검사를 보존한다. 원 driver 역사 표본을 새 형식으로 소급 편집하거나 새 판정 표본에 섞지 않는다. 최종 clean source의 실제 Linux 새 양 쌍·독립 P05 및 새 v5 3OS는 부모의 별도 인수 조건이다.
