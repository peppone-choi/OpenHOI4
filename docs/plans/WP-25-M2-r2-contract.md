# WP25 재현 번들·성능 CI 배정

2026-10-08. 기존 M2-r2 W1-c 작업이다. 세계 프리뷰의 새 독립 P05-4, 기본 브랜치 P07, 통합 커밋 `81deb803944cf6297f13b3f14ca454774cea141b`의 6개 CI·25개 필수 작업 성공과 실제 Save/Trigger 산출물 9개 대조를 인수한 뒤 착수했다. 원 W1/W2 전체 통합은 아직 0/0이며 M2 게이트 판정은 없다.

| 배정 | 값 |
|---|---|
| 구현 앱 | `01a1170c-36c9-7c42-ae3f-9ce84433a934` |
| 전용 worktree | `E:/openhoi/.orchestrator/wt/WP-25-M2-r2` |
| 브랜치 | `codex/wp25-repro-bench-m2r2` |
| 구현 전 base·고정 성능 prior SHA | `81deb803944cf6297f13b3f14ca454774cea141b` |
| 최초 부모 스냅샷 | 11,721개 추적 파일, clean; raw index SHA `0e23f09e517bcd3d2bf5a95d7913c6f541f1d12b5af4086462ce8e2dde654bb9` |
| 실제 전송 프롬프트 | 6,551 bytes, SHA `12eb6b8bd47b916727913b5106dbb674aba76f7e8002f00eb0f40696869cfeb3`; 원파일과 payload UTF-8 bytes 일치 |

REQ-SAV-05와 REQ-PERF-03이 대상이다. oh_cli 재현·벤치, oh_save 별도 재현 모듈, bench 및 전용 검사 도구·새 성능 workflow, 필요한 관련 Cargo manifest/lock과 WP25 문서를 구현자가 소유한다. shared 게임 규칙·기존 codec/골든·기존 6개 workflow·세계 프리뷰·01/02/03/HANDOFF/ASSETS는 보존한다. 실제 게임 기록에 서버나 공유 sim hook이 필요한 경우 구체 호출 경로와 최소 변경을 먼저 부모에게 보고하며, 범위 안의 나머지 작업은 계속한다.

실제 host 입력·enqueue/pump/step 순서, 기록 시작 snapshot과 pending queue, 거부 결과와 종료 상태를 먼저 ADR에 대응한다. 새 native 프로세스의 전체 DTO/canonical/hash 재생과 ZIP 경로·자원·변조 거부, 실패 시 원자료·상태 보존을 검사한다. fixture exporter를 실제 게임 기록으로 표시하지 않는다. 이전 v1/v2/v3 및 새 flags/종료 v4의 호환 검사를 유지한다.

성능 prior는 위 SHA로 구현 전에 고정했다. 동일 Linux runner에서 prior/current의 실제 라이브러리를 각각 빌드하고 같은 workload를 연속 측정한다. 빌드·다운로드·warmup과 측정을 구분하며 15% 초과가 두 번 재현되면 실패한다. prior에 CLI가 없다고 통과시키거나 current를 자기 기준으로 삼지 않는다. 원 측정 환경·명령·작업량·상태 해시·native 결과와 비교 경계를 보존한다.

구현자의 자체 검사는 완료 판정이 아니다. 실제 final·소스·원 증거를 인수한 뒤 구현에 참여하지 않은 새 앱과 exact detached worktree로 P05를 수행한다. 추적 파일·HEAD·전체 SHA·semantic/raw index·diff/status가 검증 전후 같아야 한다. 유효 PASS 후 P07과 같은 기본 브랜치 CI로 W1 전체 조건을 확인한다. 그 다음 W2 WP14/23을 기존 소유·의존 순서로 이어가며 M3는 이번 RUN 범위가 아니다.
