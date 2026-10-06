# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-06, M0 진행 중 임시 기록 |
| 현재 마일스톤 | M0 |
| 기본 브랜치 CI | main에는 아직 워크플로 미통합; wp/01-skeleton에서 CI 실행 시작; https://github.com/peppone-choi/OpenHOI4/actions |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

초기 HANDOFF가 없던 새 저장소에서 시작했다. Git 초기화와 GitHub 저장소 생성·푸시 후 사용자 지시로 공개 전환를 수행했다. README와 저장소 설명은 사용자 지시에 따라 한국어 게임 소개로 바꿨다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP-00 | CLI FAIL, 앱 호출 시험 정상 종료 | orch/selftest와 전용 worktree 삭제 | CLI 실패를 성공으로 바꾸지 않음 |
| WP-01 | 통합됨 | wp/01-skeleton; E:/openhoi/.orchestrator/wt/WP-01 | 새 P-05 검증 채팅 결과와 branch CI를 확인 |
| WP-02 | 구현 착수 준비 | wp/02-core 예정 | WP-01 통합 뒤 WP-03과 연달아 시작 |
| WP-03 | 구현 착수 준비 | wp/03-data 예정 | WP-01 통합 뒤 WP-02와 연달아 시작 |
| WP-06 | 검증 대기 | main | WP-01 통합 뒤 마무리·새 독립 검증 |
| WP-04 | 미착수, 프롬프트 준비 | wp/04-simulation 예정 | WP-02·WP-03 통합 뒤 시작 |
| WP-05 | 미착수, 프롬프트 준비 | wp/05-network 예정 | WP-04 통합 뒤 시작 |

## 세션 상태

- tools/orch.sh status: WP-00.impl done 1. 하위 gpt-6.1-sol/ChatGPT 인증 조합이 HTTP 400으로 거부됐다. CLI 모델·인증·샌드박스 설정은 바꾸지 않았다.
- 사용자가 현재 앱 설정의 별도 채팅 실행을 선택했다. 구현·검증은 앱 create_thread local + 명시적 Git worktree로 조율한다. 모든 명령은 지정 worktree에서 실행하도록 지시한다.
- WP-01 구현 채팅 ID: 01a11038-c022-7860-a77f-8f0fe05c4b77, hostId local. wait_threads로 종료 여부를 확인한다. 구현자는 다른 채팅을 만들거나 main에 병합하지 않는다.
- 앱 호출 시험 채팅 ID: 01a11038-2afd-7dd3-9be4-753c94a02911, 정상 종료.
- 처음 요청한 앱 자동 worktree 생성은 client-new-thread:c7dfdf29-3fd1-49fb-a43d-d3ac11104849와 C:/Users/user/.codex/worktrees/788a/openhoi detached 폴더를 관측했으나 실제 threadId·실행은 미확인이다. orch/app-selftest 브랜치를 남겼다. 이 폴더를 구현에 사용하지 않는다.
- .orchestrator/evidence/app-threads.json, prompts/, logs/, out/에 실행 기록·프롬프트가 있다. 앱 turn 상태와 CLI .exit는 구분한다.

## 마지막 로컬 검증

| 명령 | 결과 |
|---|---|
| python tools/check_docs.py | exit 0, 오류 0·경고 0 |
| python tools/check_docs.py --write-trace | exit 0, 부록 A 생성 |
| git diff fd4d167 -- docs/templates tools/orch.sh | 템플릿/스크립트 내용 변경 없음, orch.sh 실행 비트만 100755로 수정 |
| 공식 cargo/npm 레지스트리 조회 | 13개 크레이트와 8개 npm 버전·라이선스 기록; docs/research/M0-versions.md |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D-10 | 기존 기획 결정 | — | M0, 02 §14.3 허용 의존성 | M3 공개 라이선스 확정 |
| D-12 | 기존 기획 결정 | — | M0, 저장소명 OpenHOI4 사용 | M3 공개 제품명 확정 |

## 알려진 문제

| 문제 | 재현 방법 |
|---|---|
| 하위 CLI 인증 모델 거절 | tools/orch.sh selftest; .orchestrator/logs/selftest.err |
| check_docs가 숫자 ADR에서 DR 식별자 부분 일치 | 핵심 검사용 문서에 ADR-숫자 직접 표기 시 발생; 환경 실행 ADR로 참조하여 현재 통과 |
| Actions 사용량 API 접근 불가 | gh api users/peppone-choi/settings/billing/actions → HTTP 404/토큰 범위 부족; 인증 범위 변경 안 함 |

## 사용자 추가 지시

최종 게임 개발 완료 시 docs/, tools/, AGENTS.md를 삭제한다. M0는 전체 개발의 기반 단계이므로 현재는 지침과 증거를 유지한다. 마지막 정리에서 증거를 Git 이력/CI 아티팩트에 남기고 필요한 빌드/CI 참조를 정리하며 검사를 우회하지 않는다.

## 다음 세션이 처음 할 일 (3개 이내)

1. WP-01 앱 채팅의 실제 상태와 전용 worktree 커밋/작업 로그를 확인한다.
2. 구현 종료·커밋 후 detached 검증 worktree와 새 P-05 앱 채팅을 만든다. 전후 HEAD·추적 파일 목록·diff/status 무변경을 직접 확인한다.
3. PASS 후 P-07 통합·main CI를 확인하고 WP-02·WP-03 앱 구현을 연달아 시작한다. 이후 WP-04→WP-05와 P-12 게이트까지 계속한다.

사용자 추가 지시(2026-10-06): 저장소를 공개로 전환한다. D-14를 사용자 답변에 따라 확정으로 갱신했다. 표준 GitHub-hosted 러너의 공개 저장소 실행은 무료이며 macOS도 매 푸시 검증한다. D-10·D-12의 제품 배포 결정은 미결정 상태를 유지한다.

WP-01 구현은 5f5da6273d7177f655ebaaaf4ef763b741920c7d로 정상 종료했다. 독립 검증 채팅 01a11046-cfca-7441-b040-dd1c8fc1ec65(hostId local)가 E:/openhoi/.orchestrator/wt/WP-01-verify detached worktree에서 실행 중이다. 검증 전 HEAD·추적 파일 목록은 .orchestrator/evidence/WP-01.verify.*-before에 저장했다. branch를 원격에 푸시하여 CI 실행을 시작했다.

WP-01 독립 검증 PASS를 docs/verify/WP-01.md와 evidence/WP-01에 저장했다. 검증 전후 무변경을 직접 확인하고 main에 병합했다. 로컬 cargo test/npm ci/npm test/check_docs는 통과했다. 원격 main CI가 녹색이 된 뒤 WP-02·WP-03을 시작한다. 사용자 요구의 웹 GPU 렌더링은 WebGPU 우선·WebGL2 대체 경로로 WP-08/M1에 기록했다.

WP-01 main CI https://github.com/peppone-choi/OpenHOI4/actions/runs/37435480879 (c662cfd) 전체 success를 확인했다. WP-02·WP-03 구현과 WP-06 독립 검증을 다음 단계로 시작한다. 앱 채팅 ID는 .orchestrator/evidence/app-threads.json에 이어 기록한다.
