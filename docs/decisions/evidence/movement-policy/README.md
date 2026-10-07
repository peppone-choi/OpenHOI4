# 중간 이동 정책 원 검토

2026-10-07 CEO는 원 REQUEST-0006 후보 전체 SHA256 `0b9e2ba3756fa1022a876f11369dc8b4e4909fc49e84ea35bfbf9b7d75734b07`와 자체 정수 8경계 산술을 검토해 docs/04 §2의 추천 A를 한정 잠정 채택했다. 원 후보와 실제 산술 JSON bytes를 그대로 보존한다. 제품 구현/독립 P-05 PASS가 아니며 사용자 D/OPEN 확정 답을 추가한 기록이 아니다. 원 후보는 채택 전 상태를 포함하고 현재 REQUEST/01/02/03의 채택 기록과 구분된다.

원 bytes의 정본은 `originals.zip`의 두 member다. 인접 JSON은 표시용이며 Git의 CRLF→LF 정규화가 바이트를 바꿀 수 있다. ZIP은 원 immutable CEO source와 전수 bytes/SHA가 동일하고 identity.json이 이를 기록한다. 부모가 Git blob과 working copy의 줄바꿈 차이를 발견해 이 원 바이너리 봉인을 추가했으며 이전 기록은 Git 이력에 유지한다.
