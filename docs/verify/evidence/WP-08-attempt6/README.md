# WP-08 독립 검증 6

구현 비참여 새 앱 채팅의 최종 PASS 전문은 `../../WP-08.attempt6.md`에 있다. 대상은 9fbc05a1e29062f93bb362450aaddc2068aa41e9, 지정 detached worktree의 추적 5008개다. 부모는 실제 final/completed 뒤 HEAD·semantic index·목록/각 SHA·diff/status 불변성을 재확인했다(`parent/`). 원시 index 바이트 동일은 검증자 before/after의 별도 관측이다.

원본 2040파일을 `original-evidence-bytes.zip`에 보존했다. 67,839,746byte, SHA256 d21b1b80cc9e52821892066abef1669d9898718a7f99c10926fe179bea148bbe. `preservation.json`의 각 source byte/SHA와 ZIP member는 모두 일치한다. 텍스트 검토 사본의 정규화·Git 줄바꿈과 원본 ZIP 바이트를 구분한다.

컴파일 산출물·node_modules·Playwright transform cache·재빌드용 red62/red953의 확장 소스 사본은 제외했다. 포함된 commit별 입력 ZIP과 source-identity는 필요한 Git blob 전부의 일치와 명시적 제외 경로를 보여 준다. 원본 Linux 진단 artifact 459MB는 `preserved-separately.json`의 ignored 위치에 삭제 없이 유지하고 선택 trace·API digest·검증 결과를 보존했다.

Windows 기본 M1 205/M0 60·unit130/Rust80, native lifecycle10·pipeline6·compile-DPR2·malformed30·redirect120·독립 PNG72가 PASS다. 기존 250ms 전환 및 locator DPR 복원 계측 실패, 임시 workspace fixture 오류·bind 충돌은 원본대로 보존하고 최종 제품 판정과 구분했다. branch CI124 성공을 main CI·M1 게이트 완료로 확대하지 않는다.
