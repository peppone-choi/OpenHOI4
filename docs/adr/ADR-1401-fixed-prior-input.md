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
