# WP-11 첫 독립 검증 원본

2026-10-07, exact227·새 앱01a11320-f551-7501-8e23-5466e21e4cc9·새 detached WP-11-verify의 전문은 [첫 FAIL](../../WP-11.attempt1.md)에 있다. 첫 검증은 제품 내용 동일에도 요구된 raw index 불변성을 충족하지 못했다. 제품 완성/독립 PASS/통합 증거로 사용하지 않는다.

부모 postcheck를 실제 Python subprocess로 실행해 exit4를 확인했다. HEAD227·semantic index·9,166 목록/파일SHA·diff/cached/status는 동일하고 rawindex SHA만18da9cc…5513a3→c54b8ab…6d015cb로 달랐다. snapshot before/after·actual Python 명령/exit 원문·최종 index bytes는 parent-before-after-bytes.zip에 있고 parent-preservation.json으로 대조한다. 최초 before raw index bytes는 보존하지 못했으므로 전체 entry metadata 차이와 갱신 프로세스를 확정하지 않는다. 추적 protocol.ts 재생성 및 현재 index의 해당 새 mtime 관측은 별도 근거다. before 덮어쓰기/원 index 복원/설정 변경은 하지 않았다.

original-evidence-bytes.zip은 검증자 target의 원본 1,383파일을 보존한다. preservation.json은 원본 bytes/SHA와 일부 정규화한 검토본 SHA를 구분한다. 부모가 ZIP 멤버와 검토본 SHA를 직접 대조한다. 별도 native 23+31틱/54틱 구조·정수 계산, 실제 UI, 손상·force·3OS 검사는 해당 실행 범위의 결과다. 현재제품227 일반CI의 WP08 DPR FAIL과 flush/sync 자체 오류 주입 미실행을 숨기지 않는다.

다음은 기존 구현 채팅에 파일 shared transaction의 test-double flush/sync/commit 경로 검사 P-06을 배정한 상태다. 실제 OS 고장/전원손실 시험으로 부르지 않는다. 새sourcecommit 뒤 새 독립앱·detached worktree에서 rawindex 전후 원문과 각 필수명령 경계를 보존하고, 실제 프로토콜 생성함수를 ignored 출력→Gitblob 전체바이트 대조로 검사한다. 원 시험/단언을 삭제하거나 before를 교체하지 않는다.

반복 팩/세OS 사본·대형 before/after·entry 상세의 검토 복사본은 원본 ZIP에 모두 남기고, 문서에는 전문·실제명령/기준계산·CI API·핵심 native/result·UI 및 부모 요약을 선별했다. 삭제한 검토 복사본은 원본 target과 ZIP으로 복구할 수 있고 원본 증거는 삭제하지 않았다. preservation.json에 검토본 생략 여부를 기록한다.
