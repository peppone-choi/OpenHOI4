# 육지 간 해협 이동 원 검토

2026-10-07 CEO가 원 후보 전체 SHA256 `2731e99b719f836d42d2100da349392d199cbebba301796cbbeed098104b2e54`와 독립 정수 8경계를 검토해 docs/04 §2의 A를 한정 잠정 채택했다. 원 후보와 실제 산술 JSON을 그대로 보존한다. 제품 구현/독립 P-05 PASS나 D/OPEN의 사용자 추가 확정 답이 아니다. 원 후보의 대기 상태와 현재 문서의 잠정 채택 상태를 구분한다.

원 bytes의 정본은 `originals.zip`의 두 member다. 인접 JSON은 표시용이며 Git의 CRLF→LF 정규화가 바이트를 바꿀 수 있다. ZIP은 원 immutable CEO source와 전수 bytes/SHA가 동일하고 identity.json이 이를 기록한다. 부모가 Git blob과 working copy의 줄바꿈 차이를 발견해 이 원 바이너리 봉인을 추가했으며 이전 기록은 Git 이력에 유지한다.
