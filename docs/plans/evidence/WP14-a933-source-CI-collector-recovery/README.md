# WP14 소스 CI 수집 실패와 복구 원본

최초 gh run view --log는 C: GitHub CLI run-log 캐시에 저장할 공간이 없어 native1이었다. 원 stdout·stderr·실제 argv/exit와 수집 도구를 보존한다. 새 namespace의 수집기는 gh api actions/runs/ID/logs의 원 ZIP을 E:에 받아 실제 API argv/exit와 CRC·추출 원 로그를 보존했고 native0이었다. 전역 설정·정책·워크플로·측정·소스는 바꾸지 않았다. 원 purpose command receipt와 actual API command receipt는 서로 구분한다.

실제 7개 소스 CI와 9개 Save/Trigger artifact 원본은 인접 WP14-a933-source-CI-final-api-recovery에 있다. 이 원 수집 실패를 CI 실패 또는 CI 재실행으로 기록하지 않는다. 독립 판정과 통합 후 동일 기본 브랜치 CI는 별도다.
