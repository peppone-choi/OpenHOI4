# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-06, M0 병렬 작업 진행 중 |
| 현재 마일스톤 | M0 |
| 기본 브랜치 CI | 1ad982b의 CI 전체 success: https://github.com/peppone-choi/OpenHOI4/actions/runs/37436885058 . 모든 7개 작업 성공을 직접 확인했다. |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

WP-01의 구현·독립 검증·main CI를 모두 확인하고 통합을 마쳤다. WP-02·WP-03을 실제 앱 채팅에서 병렬로 시작했다. 이전 HANDOFF의 초기 CI/검증 대기 설명과 누적 메모가 최신 상태와 모순되어 이 문서를 현재 상태로 다시 쓴다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP-00 | CLI FAIL, 앱 호출 시험 정상 종료 | orch/selftest·CLI 시험 worktree 삭제 | CLI done 1 로그를 유지 |
| WP-01 | 통합됨 | wp/01-skeleton; 구현·검증 worktree 정리됨 | 재실행 필요 없음, 증거 docs/verify/WP-01.md |
| WP-02 | 구현 중 | wp/02-core; E:/openhoi/.orchestrator/wt/WP-02 | 종료·커밋 뒤 새 P-05 검증 |
| WP-03 | 구현 중 | wp/03-data; E:/openhoi/.orchestrator/wt/WP-03 | 종료·커밋 뒤 새 P-05 검증 |
| WP-06 | 통합됨 | main; 검증 worktree 정리 예정 | 독립 PASS·main CI 성공과 첫 FAIL 리포트 보존 |
| WP-04 | 미착수, 프롬프트 준비 | wp/04-simulation 예정 | WP-02·WP-03 통합 뒤 시작 |
| WP-05 | 미착수, 프롬프트 준비 | wp/05-network 예정 | WP-04 통합 뒤 시작 |

## 세션 상태

- CLI status는 WP-00.impl done 1이다. 실제 오류는 gpt-6.1-sol/ChatGPT 인증 조합의 HTTP 400이다. 모델·인증·샌드박스·승인 설정은 바꾸지 않았다.
- 사용자 승인 경로는 현재 앱 설정의 별도 채팅이다. 앱 local 채팅에 WP별 Git worktree를 명시하며 모든 명령은 그 경로에서만 실행한다. CLI exit와 앱 turn 상태는 구분한다.
- WP-02 구현: 01a11051-f7f6-7281-bb6a-90b46f813be8, hostId local.
- WP-03 구현: 01a11052-0003-76c0-8de5-242585836ae7, hostId local.
- WP-06 첫 검증 종료 FAIL(과거): 01a11052-0946-7cd0-9b02-166427582f0b, hostId local, 대상 커밋 88ef41f. 첫 판정은 WP-06.attempt1.md에 보존했다. 새 검증 01a11058-8222-7cc0-891b-ba78fddf0ff4는 1ad982b에 PASS를 냈다.
- WP-01 구현/검증은 정상 종료했고 PASS와 무변경 증거를 docs/verify/WP-01.md 및 evidence/WP-01에 보존했다.
- 앱 호출 시험 01a11038-2afd-7dd3-9be4-753c94a02911은 정상 종료했다. 처음 자동 worktree 요청 client-new-thread:c7dfdf29-3fd1-49fb-a43d-d3ac11104849의 threadId는 미확인이다. C:/Users/user/.codex/worktrees/788a/openhoi detached 폴더와 orch/app-selftest 브랜치는 보존했고 구현에 사용하지 않는다.
- 최신 앱 채팅 등록은 .orchestrator/evidence/app-threads.json에 있다. wait_threads로 직접 확인하며 구현·검증 세션의 자기 보고만으로 완료를 판정하지 않는다.

## 마지막 로컬 검증

| 명령 | 결과 |
|---|---|
| cargo test --workspace (WP-01 통합) | exit 0; 아직 도메인 테스트가 없는 골격 단계 |
| npm --prefix client ci / test | exit 0; Vitest 1개 통과 |
| python tools/check_docs.py --write-trace | exit 0, 오류 0·경고 0; REQ 143개, M0 필수 14개 |
| WP-01 검증 전후 HEAD·추적 파일 목록·diff/status | 동일; 검증자가 확인한 79개 SHA-256도 동일 |
| 문서·운영 도구 원본 대조 | 템플릿과 orch.sh 내용 동일, 실행 비트만 100755로 보완 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D-10 | 기존 기획 결정 | — | M0, 허용 의존성 사용 | 제품 배포 라이선스 확정 |
| D-12 | 기존 기획 결정 | — | 저장소명 OpenHOI4 사용과 M0 | 제품 공개 명칭 확정 |

M0를 막는 미결정 게임 규칙은 없다. D-14는 사용자 지시로 공개 저장소로 확정했다.

## 알려진 문제

| 문제 | 재현 방법 |
|---|---|
| CLI 모델·인증 미지원 | 원본 로그 docs/worklog/evidence/WP-06/ 및 .orchestrator/logs/ 확인. 사용자 승인 앱 실행으로 진행 |
| 이전 HANDOFF의 CI 상태 모순 | 검증 대상 88ef41f에서 상단 미통합과 하단 main CI success가 함께 존재. 현재 문서에서 정리했고 새 독립 검증 필요 |
| 문서 검사기의 숫자 ADR 부분 일치 | 핵심 검사용 문서는 환경 실행 ADR이라고 참조하여 통과. 검사 도구를 약화하지 않음 |

## 사용자 추가 지시

- 저장소는 PUBLIC인 https://github.com/peppone-choi/OpenHOI4 이다. 한국어 설명과 게임 중심 README를 유지한다. 표준 공개 CI로 세 OS를 매 push/PR 검증한다.
- 서버 실행 편의는 REQ-NET-06/WP-05/M0: 기본 로컬 실행, 접속 주소·사용법, --open, 시작 실패와 정상 종료.
- 웹 렌더링은 REQ-PLAT-03/WP-08/M1: Three.js WebGPU 우선·WebGL2 대체, 가용 GPU 활용, 초기화 실패/미지원 안내. 현재 M0 셸에 지도 렌더링이 구현됐다고 기록하지 않는다.
- 최종 게임 개발 완료 시 docs/, tools/, AGENTS.md를 삭제한다. M0 진행 중에는 규약·검증 자료를 유지하고 마지막 정리에서 증거와 필요한 CI 참조를 보존한다.

## 다음 세션이 처음 할 일 (3개 이내)

1. WP-02·WP-03 구현과 WP-06 첫 검증의 실제 종료/진행 상태를 wait_threads로 확인한다.
2. WP-06 재검증 PASS 증거를 유지하고, WP-02·WP-03은 커밋 확인 후 새 detached P-05 검증 → P-07 통합·main CI로 진행한다.
3. WP-04→WP-05를 진행하고 P-12 M0 게이트 및 최종 HANDOFF를 작성한다. 미완료 항목은 이유를 따로 남긴다.
